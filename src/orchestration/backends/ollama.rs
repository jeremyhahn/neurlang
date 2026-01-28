//! Ollama Backend
//!
//! Local LLM support via Ollama for offline operation.

use super::{BackendError, DecomposeResult, LlmBackend, Subtask};
use std::env;

/// Ollama local LLM backend
pub struct OllamaBackend {
    host: String,
    model: String,
}

impl OllamaBackend {
    /// Create a new Ollama backend with default settings
    pub fn new() -> Self {
        Self {
            host: env::var("OLLAMA_HOST").unwrap_or_else(|_| "http://localhost:11434".to_string()),
            model: env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3".to_string()),
        }
    }

    /// Create with explicit configuration
    pub fn with_config(host: String, model: String) -> Self {
        Self { host, model }
    }

    /// Set the model to use
    pub fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }

    /// Set the host URL
    pub fn set_host(&mut self, host: &str) {
        self.host = host.to_string();
    }

    /// Build the decomposition prompt
    fn build_decompose_prompt(&self, task: &str, context: &str) -> String {
        format!(
            r#"You are a task decomposition assistant. Break down this programming task into smaller subtasks.

Task: {task}

Context: {context}

Respond in JSON format:
{{
  "reasoning": "Your approach",
  "complexity": <1-10>,
  "subtasks": [
    {{
      "id": 1,
      "description": "What to implement",
      "depends_on": [],
      "test_hints": ["Test case"],
      "priority": 1
    }}
  ]
}}

Only output JSON."#,
            task = task,
            context = if context.is_empty() { "None" } else { context }
        )
    }

    /// Parse the decomposition response
    fn parse_decompose_response(&self, response: &str) -> Result<DecomposeResult, BackendError> {
        // Try to extract JSON from the response
        let json_str = if response.starts_with('{') {
            response
        } else if let Some(start) = response.find('{') {
            if let Some(end) = response.rfind('}') {
                &response[start..=end]
            } else {
                return Err(BackendError::ParseError {
                    message: "No closing brace found".to_string(),
                });
            }
        } else {
            return Err(BackendError::ParseError {
                message: "No JSON found in response".to_string(),
            });
        };

        // Parse JSON
        let value: serde_json::Value =
            serde_json::from_str(json_str).map_err(|e| BackendError::ParseError {
                message: format!("JSON parse error: {}", e),
            })?;

        // Extract fields
        let reasoning = value["reasoning"]
            .as_str()
            .unwrap_or("No reasoning provided")
            .to_string();

        let complexity = value["complexity"].as_u64().unwrap_or(5) as u8;

        let subtasks = value["subtasks"]
            .as_array()
            .ok_or_else(|| BackendError::ParseError {
                message: "Missing subtasks array".to_string(),
            })?
            .iter()
            .filter_map(|s| {
                Some(Subtask {
                    id: s["id"].as_u64()? as usize,
                    description: s["description"].as_str()?.to_string(),
                    depends_on: s["depends_on"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_u64().map(|n| n as usize))
                                .collect()
                        })
                        .unwrap_or_default(),
                    test_hints: s["test_hints"]
                        .as_array()
                        .map(|arr| {
                            arr.iter()
                                .filter_map(|v| v.as_str().map(|s| s.to_string()))
                                .collect()
                        })
                        .unwrap_or_default(),
                    priority: s["priority"].as_u64().unwrap_or(5) as u32,
                })
            })
            .collect();

        Ok(DecomposeResult {
            subtasks,
            reasoning,
            complexity,
        })
    }

    /// Check if Ollama is running
    fn check_available(&self) -> bool {
        #[cfg(feature = "llm-backends")]
        {
            let client = ureq::AgentBuilder::new()
                .timeout(std::time::Duration::from_secs(2))
                .build();

            client
                .get(&format!("{}/api/tags", self.host))
                .call()
                .is_ok()
        }

        #[cfg(not(feature = "llm-backends"))]
        {
            false
        }
    }

    /// Make an API request to Ollama
    #[cfg(feature = "llm-backends")]
    fn call_api(&self, prompt: &str) -> Result<String, BackendError> {
        let client = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(120))
            .build();

        let response = client
            .post(&format!("{}/api/generate", self.host))
            .set("content-type", "application/json")
            .send_json(ureq::json!({
                "model": self.model,
                "prompt": prompt,
                "stream": false
            }))
            .map_err(|e| BackendError::NetworkError {
                message: format!("Ollama connection failed: {}. Is Ollama running?", e),
            })?;

        let body: serde_json::Value =
            response.into_json().map_err(|e| BackendError::ParseError {
                message: e.to_string(),
            })?;

        body["response"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| BackendError::ParseError {
                message: "No response in Ollama output".to_string(),
            })
    }

    /// Stub API call when llm-backends feature is disabled
    #[cfg(not(feature = "llm-backends"))]
    fn call_api(&self, _prompt: &str) -> Result<String, BackendError> {
        Err(BackendError::NotConfigured {
            backend: "ollama".to_string(),
        })
    }
}

impl Default for OllamaBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmBackend for OllamaBackend {
    fn name(&self) -> &str {
        "ollama"
    }

    fn is_available(&self) -> bool {
        self.check_available()
    }

    fn decompose_task(&self, task: &str, context: &str) -> Result<DecomposeResult, BackendError> {
        let prompt = self.build_decompose_prompt(task, context);
        let response = self.call_api(&prompt)?;
        self.parse_decompose_response(&response)
    }

    fn get_fix_hint(&self, task: &str, error: &str, code: &str) -> Result<String, BackendError> {
        let prompt = format!(
            "Task: {}\nError: {}\nCode:\n```\n{}\n```\n\nProvide a one-sentence fix hint.",
            task, error, code
        );
        self.call_api(&prompt)
    }

    fn supports_streaming(&self) -> bool {
        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ollama_backend_creation() {
        let backend = OllamaBackend::new();
        assert_eq!(backend.name(), "ollama");
    }

    #[test]
    fn test_ollama_with_config() {
        let backend = OllamaBackend::with_config(
            "http://localhost:11434".to_string(),
            "codellama".to_string(),
        );
        assert_eq!(backend.model, "codellama");
    }

    #[test]
    fn test_parse_response() {
        let backend = OllamaBackend::new();

        let response = r#"{
            "reasoning": "Simple task",
            "complexity": 2,
            "subtasks": [
                {
                    "id": 1,
                    "description": "Do the thing",
                    "depends_on": [],
                    "test_hints": [],
                    "priority": 1
                }
            ]
        }"#;

        let result = backend.parse_decompose_response(response).unwrap();
        assert_eq!(result.complexity, 2);
        assert_eq!(result.subtasks.len(), 1);
    }
}
