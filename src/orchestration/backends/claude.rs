//! Claude Backend
//!
//! Anthropic Claude API integration for task decomposition.

use super::{BackendError, DecomposeResult, LlmBackend, Subtask};
use std::env;

/// Claude API backend
pub struct ClaudeBackend {
    api_key: Option<String>,
    model: String,
    base_url: String,
}

impl ClaudeBackend {
    /// Create a new Claude backend
    ///
    /// Reads API key from ANTHROPIC_API_KEY environment variable
    pub fn new() -> Self {
        Self {
            api_key: env::var("ANTHROPIC_API_KEY").ok(),
            model: "claude-sonnet-4-20250514".to_string(),
            base_url: "https://api.anthropic.com/v1".to_string(),
        }
    }

    /// Create with explicit configuration
    pub fn with_config(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key: Some(api_key),
            model: model.unwrap_or_else(|| "claude-sonnet-4-20250514".to_string()),
            base_url: "https://api.anthropic.com/v1".to_string(),
        }
    }

    /// Set the model to use
    pub fn set_model(&mut self, model: &str) {
        self.model = model.to_string();
    }

    /// Get the API base URL
    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Build the decomposition prompt
    fn build_decompose_prompt(&self, task: &str, context: &str) -> String {
        format!(
            r#"You are a task decomposition assistant for Neurlang, an AI coding system.

Your job is to break down a complex programming task into smaller subtasks that a small local model can handle. Each subtask should be:
1. Self-contained and testable
2. Simple enough for a 50M parameter model to handle
3. Clear about inputs and expected outputs

The local model knows how to:
- Basic arithmetic and logic operations
- Memory operations (load, store)
- Control flow (branches, loops)
- Call extensions for: JSON, HTTP, file I/O, crypto, database, regex, datetime

Task to decompose:
{task}

Additional context:
{context}

Respond in this exact JSON format:
{{
  "reasoning": "Brief explanation of your decomposition approach",
  "complexity": <1-10>,
  "subtasks": [
    {{
      "id": 1,
      "description": "Clear description of what to implement",
      "depends_on": [],
      "test_hints": ["Example test case"],
      "priority": 1
    }}
  ]
}}

Only output valid JSON, no other text."#,
            task = task,
            context = if context.is_empty() {
                "None provided"
            } else {
                context
            }
        )
    }

    /// Build the fix hint prompt
    fn build_fix_prompt(&self, task: &str, error: &str, code: &str) -> String {
        format!(
            r#"A Neurlang program failed with an error. Provide a brief hint on how to fix it.

Task: {task}

Error: {error}

Current code:
```
{code}
```

Provide a single sentence hint on what to fix. Be specific and actionable."#,
            task = task,
            error = error,
            code = code
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

    /// Make an API request to Claude
    #[cfg(feature = "llm-backends")]
    fn call_api(&self, prompt: &str) -> Result<String, BackendError> {
        let api_key = self
            .api_key
            .as_ref()
            .ok_or_else(|| BackendError::NotConfigured {
                backend: "claude".to_string(),
            })?;

        let client = ureq::AgentBuilder::new()
            .timeout(std::time::Duration::from_secs(60))
            .build();

        let response = client
            .post(&format!("{}/messages", self.base_url))
            .set("x-api-key", api_key)
            .set("anthropic-version", "2023-06-01")
            .set("content-type", "application/json")
            .send_json(ureq::json!({
                "model": self.model,
                "max_tokens": 4096,
                "messages": [{
                    "role": "user",
                    "content": prompt
                }]
            }))
            .map_err(|e| match e {
                ureq::Error::Status(429, _) => BackendError::RateLimited {
                    retry_after_secs: Some(60),
                },
                ureq::Error::Status(status, resp) => BackendError::ApiError {
                    status,
                    message: resp.into_string().unwrap_or_default(),
                },
                _ => BackendError::NetworkError {
                    message: e.to_string(),
                },
            })?;

        let body: serde_json::Value =
            response.into_json().map_err(|e| BackendError::ParseError {
                message: e.to_string(),
            })?;

        body["content"][0]["text"]
            .as_str()
            .map(|s| s.to_string())
            .ok_or_else(|| BackendError::ParseError {
                message: "No text in response".to_string(),
            })
    }

    /// Stub API call when llm-backends feature is disabled
    #[cfg(not(feature = "llm-backends"))]
    fn call_api(&self, _prompt: &str) -> Result<String, BackendError> {
        // Return a mock response for testing without the feature
        Ok(r#"{
            "reasoning": "Mock decomposition for testing",
            "complexity": 3,
            "subtasks": [
                {
                    "id": 1,
                    "description": "Implement the main logic",
                    "depends_on": [],
                    "test_hints": ["Should work correctly"],
                    "priority": 1
                }
            ]
        }"#
        .to_string())
    }
}

impl Default for ClaudeBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl LlmBackend for ClaudeBackend {
    fn name(&self) -> &str {
        "claude"
    }

    fn is_available(&self) -> bool {
        self.api_key.is_some()
    }

    fn decompose_task(&self, task: &str, context: &str) -> Result<DecomposeResult, BackendError> {
        let prompt = self.build_decompose_prompt(task, context);
        let response = self.call_api(&prompt)?;
        self.parse_decompose_response(&response)
    }

    fn get_fix_hint(&self, task: &str, error: &str, code: &str) -> Result<String, BackendError> {
        let prompt = self.build_fix_prompt(task, error, code);
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
    fn test_claude_backend_creation() {
        let backend = ClaudeBackend::new();
        assert_eq!(backend.name(), "claude");
    }

    #[test]
    fn test_parse_decompose_response() {
        let backend = ClaudeBackend::new();

        let response = r#"{
            "reasoning": "Split into input handling and calculation",
            "complexity": 3,
            "subtasks": [
                {
                    "id": 1,
                    "description": "Parse input",
                    "depends_on": [],
                    "test_hints": ["Parse '5' to 5"],
                    "priority": 1
                },
                {
                    "id": 2,
                    "description": "Calculate result",
                    "depends_on": [1],
                    "test_hints": ["5! = 120"],
                    "priority": 2
                }
            ]
        }"#;

        let result = backend.parse_decompose_response(response).unwrap();
        assert_eq!(result.complexity, 3);
        assert_eq!(result.subtasks.len(), 2);
        assert_eq!(result.subtasks[0].id, 1);
        assert_eq!(result.subtasks[1].depends_on, vec![1]);
    }

    #[test]
    fn test_decompose_without_api_key() {
        let backend = ClaudeBackend::new();

        // Without llm-backends feature, this returns a mock response
        // With the feature but no API key, it would fail
        #[cfg(not(feature = "llm-backends"))]
        {
            let result = backend.decompose_task("test task", "");
            assert!(result.is_ok());
        }
    }
}
