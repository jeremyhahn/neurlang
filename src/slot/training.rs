//! Slot Training Data Generation
//!
//! Extracts slot-level training examples from:
//! - Protocol specifications (specs/protocols/*.json)
//! - Existing examples (examples/*.nl)
//! - Standard library (lib/*.nl)
//!
//! Output format (JSONL):
//! ```json
//! {"slot_type": "PatternMatch", "input": {...}, "output": "assembly code"}
//! ```

use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufWriter, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

use super::parser::{parse_protocol_spec, ProtocolSpec};
use super::spec::Slot;
use super::template::TemplateExpander;
use super::types::SlotType;

/// Slot training example (JSONL format)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotTrainingExample {
    /// Slot type name (e.g., "PatternMatch", "StateCheck")
    pub slot_type: String,
    /// Category for stratified sampling
    pub category: String,
    /// Serialized slot input parameters
    pub input: SlotInput,
    /// Expected assembly output
    pub output: String,
    /// Difficulty level (1-5)
    #[serde(default = "default_difficulty")]
    pub difficulty: u8,
    /// Source file (for debugging)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

fn default_difficulty() -> u8 {
    2
}

/// Slot input parameters (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotInput {
    /// Slot type with parameters
    #[serde(flatten)]
    pub params: SlotParams,
    /// Available registers
    #[serde(default)]
    pub registers: HashMap<String, String>,
    /// Available labels
    #[serde(default)]
    pub labels: Vec<String>,
    /// Data section references
    #[serde(default)]
    pub data_refs: Vec<String>,
}

/// Slot parameters by type
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum SlotParams {
    PatternMatch {
        pattern: String,
        input_reg: String,
        captures: Vec<CaptureParam>,
        match_label: String,
        no_match_label: String,
    },
    PatternSwitch {
        input_reg: String,
        cases: Vec<(String, String)>,
        default_label: String,
    },
    ResponseBuilder {
        template: String,
        variables: HashMap<String, String>,
        output_reg: String,
        length_reg: String,
    },
    StringCompare {
        str1_reg: String,
        str2_reg: String,
        result_reg: String,
    },
    StringCopy {
        src_reg: String,
        dst_reg: String,
        max_len: u32,
        copied_len_reg: String,
    },
    IntToString {
        value_reg: String,
        output_reg: String,
        length_reg: String,
    },
    StringToInt {
        input_reg: String,
        result_reg: String,
        success_label: String,
        error_label: String,
    },
    RangeCheck {
        value_reg: String,
        min: i64,
        max: i64,
        ok_label: String,
        error_label: String,
    },
    StateCheck {
        state_reg: String,
        valid_states: Vec<String>,
        ok_label: String,
        error_label: String,
    },
    StateTransition {
        state_reg: String,
        new_state: String,
    },
    LoopUntil {
        condition_type: String,
        condition_reg: String,
        condition_value: Option<u8>,
        body_label: String,
        exit_label: String,
    },
    SendResponse {
        socket_reg: String,
        buffer_reg: String,
        length_reg: String,
    },
    ReadUntil {
        socket_reg: String,
        buffer_reg: String,
        delimiter: String,
        max_len: u32,
        length_reg: String,
        eof_label: String,
    },
    ReadNBytes {
        socket_reg: String,
        buffer_reg: String,
        count_reg: String,
        eof_label: String,
    },
    ExtensionCall {
        extension: String,
        args: Vec<String>,
        result_reg: String,
    },
    ValidationHook {
        validation_type: String,
        value_reg: String,
        ok_label: String,
        error_label: String,
    },
    ErrorResponse {
        socket_reg: String,
        error_code: u32,
        error_message: String,
        close_after: bool,
    },
    BufferWrite {
        buffer_reg: String,
        offset_type: String,
        offset_value: String,
        value_reg: String,
        width: String,
    },
    BufferRead {
        buffer_reg: String,
        offset_type: String,
        offset_value: String,
        result_reg: String,
        width: String,
    },
}

/// Capture parameter (serializable)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CaptureParam {
    pub name: String,
    pub output_reg: String,
    pub capture_type: String,
}

/// Statistics from training data extraction
#[derive(Debug, Default, Clone)]
pub struct ExtractionStats {
    /// Total examples extracted
    pub total_examples: usize,
    /// Examples by slot type
    pub by_type: HashMap<String, usize>,
    /// Examples by category
    pub by_category: HashMap<String, usize>,
    /// Files processed
    pub files_processed: usize,
    /// Errors encountered
    pub errors: Vec<String>,
}

/// Training data extractor
pub struct SlotTrainingExtractor {
    /// Output examples
    examples: Vec<SlotTrainingExample>,
    /// Statistics
    stats: ExtractionStats,
    /// Reference implementations (slot_type -> code)
    #[allow(dead_code)]
    reference_code: HashMap<String, Vec<String>>,
}

impl SlotTrainingExtractor {
    /// Create a new extractor
    pub fn new() -> Self {
        SlotTrainingExtractor {
            examples: Vec::new(),
            stats: ExtractionStats::default(),
            reference_code: HashMap::new(),
        }
    }

    /// Extract training data from protocol specs directory
    pub fn extract_from_specs(&mut self, specs_dir: &Path) -> Result<usize, std::io::Error> {
        let mut count = 0;

        if !specs_dir.exists() {
            return Ok(0);
        }

        for entry in fs::read_dir(specs_dir)? {
            let entry = entry?;
            let path = entry.path();

            if path.extension().map(|e| e == "json").unwrap_or(false) {
                match self.extract_from_spec_file(&path) {
                    Ok(n) => {
                        count += n;
                        self.stats.files_processed += 1;
                    }
                    Err(e) => {
                        self.stats.errors.push(format!("{}: {}", path.display(), e));
                    }
                }
            }
        }

        Ok(count)
    }

    /// Extract from a single protocol spec file
    pub fn extract_from_spec_file(&mut self, path: &Path) -> Result<usize, String> {
        let spec = parse_protocol_spec(path).map_err(|e| format!("Parse error: {:?}", e))?;

        self.extract_from_protocol_spec(&spec, path.to_string_lossy().as_ref())
    }

    /// Extract training examples from a ProtocolSpec
    pub fn extract_from_protocol_spec(
        &mut self,
        spec: &ProtocolSpec,
        source: &str,
    ) -> Result<usize, String> {
        let mut count = 0;

        // Expand spec to SlotSpec
        let expander = TemplateExpander::new(Default::default());
        let slot_spec = expander
            .expand(spec)
            .map_err(|e| format!("Expansion error: {:?}", e))?;

        // Extract examples from each slot
        for slot in &slot_spec.slots {
            if let Some(example) = self.slot_to_example(slot, source) {
                self.add_example(example);
                count += 1;
            }
        }

        Ok(count)
    }

    /// Convert a Slot to a training example
    fn slot_to_example(&self, slot: &Slot, source: &str) -> Option<SlotTrainingExample> {
        let params = self.slot_type_to_params(&slot.slot_type)?;
        let category = format!("{:?}", slot.slot_type.category());
        let difficulty = self.estimate_difficulty(&slot.slot_type);

        // Generate reference output code
        let output = self.generate_reference_code(&slot.slot_type);

        Some(SlotTrainingExample {
            slot_type: slot.slot_type.name().to_string(),
            category,
            input: SlotInput {
                params,
                registers: slot.context.registers.clone(),
                labels: slot.context.labels.clone(),
                data_refs: slot.context.data_refs.clone(),
            },
            output,
            difficulty,
            source: Some(source.to_string()),
        })
    }

    /// Convert SlotType to serializable params
    fn slot_type_to_params(&self, slot_type: &SlotType) -> Option<SlotParams> {
        match slot_type {
            SlotType::PatternMatch {
                pattern,
                input_reg,
                captures,
                match_label,
                no_match_label,
            } => Some(SlotParams::PatternMatch {
                pattern: pattern.clone(),
                input_reg: input_reg.clone(),
                captures: captures
                    .iter()
                    .map(|c| CaptureParam {
                        name: c.name.clone(),
                        output_reg: c.output_reg.clone(),
                        capture_type: format!("{:?}", c.capture_type),
                    })
                    .collect(),
                match_label: match_label.clone(),
                no_match_label: no_match_label.clone(),
            }),
            SlotType::PatternSwitch {
                input_reg,
                cases,
                default_label,
            } => Some(SlotParams::PatternSwitch {
                input_reg: input_reg.clone(),
                cases: cases.clone(),
                default_label: default_label.clone(),
            }),
            SlotType::ResponseBuilder {
                template,
                variables,
                output_reg,
                length_reg,
            } => Some(SlotParams::ResponseBuilder {
                template: template.clone(),
                variables: variables.clone(),
                output_reg: output_reg.clone(),
                length_reg: length_reg.clone(),
            }),
            SlotType::StateCheck {
                state_reg,
                valid_states,
                ok_label,
                error_label,
            } => Some(SlotParams::StateCheck {
                state_reg: state_reg.clone(),
                valid_states: valid_states.clone(),
                ok_label: ok_label.clone(),
                error_label: error_label.clone(),
            }),
            SlotType::StateTransition {
                state_reg,
                new_state,
            } => Some(SlotParams::StateTransition {
                state_reg: state_reg.clone(),
                new_state: new_state.clone(),
            }),
            SlotType::SendResponse {
                socket_reg,
                buffer_reg,
                length_reg,
            } => Some(SlotParams::SendResponse {
                socket_reg: socket_reg.clone(),
                buffer_reg: buffer_reg.clone(),
                length_reg: length_reg.clone(),
            }),
            SlotType::ReadUntil {
                socket_reg,
                buffer_reg,
                delimiter,
                max_len,
                length_reg,
                eof_label,
            } => Some(SlotParams::ReadUntil {
                socket_reg: socket_reg.clone(),
                buffer_reg: buffer_reg.clone(),
                delimiter: delimiter.clone(),
                max_len: *max_len,
                length_reg: length_reg.clone(),
                eof_label: eof_label.clone(),
            }),
            SlotType::ExtensionCall {
                extension,
                args,
                result_reg,
            } => Some(SlotParams::ExtensionCall {
                extension: extension.clone(),
                args: args.clone(),
                result_reg: result_reg.clone(),
            }),
            SlotType::ErrorResponse {
                socket_reg,
                error_code,
                error_message,
                close_after,
            } => Some(SlotParams::ErrorResponse {
                socket_reg: socket_reg.clone(),
                error_code: *error_code,
                error_message: error_message.clone(),
                close_after: *close_after,
            }),
            // Add more slot types as needed
            _ => None,
        }
    }

    /// Generate reference assembly code for a slot type
    fn generate_reference_code(&self, slot_type: &SlotType) -> String {
        match slot_type {
            SlotType::PatternMatch {
                pattern,
                input_reg,
                captures,
                match_label,
                no_match_label,
            } => {
                let mut code = format!("; PatternMatch: \"{}\"\n", pattern);
                code.push_str(&format!("    mov r1, {}\n", input_reg));

                // Match literal prefix
                let mut offset = 0;
                for c in pattern.chars() {
                    if c == '{' {
                        break; // Start of capture
                    }
                    code.push_str(&format!("    load.b r2, [r1 + {}]\n", offset));
                    code.push_str(&format!(
                        "    mov r3, {}  ; '{}'\n",
                        c as u8,
                        c.escape_default()
                    ));
                    code.push_str(&format!("    bne r2, r3, {}\n", no_match_label));
                    offset += 1;
                }

                // Handle captures
                for capture in captures {
                    code.push_str(&format!(
                        "    ; Capture '{}' -> {}\n",
                        capture.name, capture.output_reg
                    ));
                    code.push_str(&format!(
                        "    addi {}, r1, {}\n",
                        capture.output_reg, offset
                    ));
                }

                code.push_str(&format!("    b {}\n", match_label));
                code
            }

            SlotType::StateCheck {
                state_reg,
                valid_states,
                ok_label,
                error_label,
            } => {
                let mut code = format!("; StateCheck: {} in {:?}\n", state_reg, valid_states);
                for state in valid_states {
                    code.push_str(&format!("    mov r1, {}\n", state));
                    code.push_str(&format!("    beq {}, r1, {}\n", state_reg, ok_label));
                }
                code.push_str(&format!("    b {}\n", error_label));
                code
            }

            SlotType::StateTransition {
                state_reg,
                new_state,
            } => {
                format!("; StateTransition\n    mov {}, {}\n", state_reg, new_state)
            }

            SlotType::SendResponse {
                socket_reg,
                buffer_reg,
                length_reg,
            } => {
                let mut code = String::from("; SendResponse\n");
                code.push_str(&format!("    mov r0, {}\n", socket_reg));
                code.push_str(&format!("    lea r1, {}\n", buffer_reg));
                code.push_str(&format!("    load r2, [{}]\n", length_reg));
                code.push_str("    io.send r0, r0, r1, r2\n");
                code
            }

            SlotType::ReadUntil {
                socket_reg,
                buffer_reg,
                delimiter,
                max_len,
                length_reg,
                eof_label,
            } => {
                let mut code = format!("; ReadUntil '{}'\n", delimiter.escape_default());
                code.push_str(&format!("    mov r0, {}\n", socket_reg));
                code.push_str(&format!("    lea r1, {}\n", buffer_reg));
                code.push_str(&format!("    mov r2, {}\n", max_len));
                code.push_str(&format!("    io.recv {}, r0, r1, r2\n", length_reg));
                code.push_str(&format!("    beqz {}, {}\n", length_reg, eof_label));
                code
            }

            SlotType::ExtensionCall {
                extension,
                args,
                result_reg,
            } => {
                let mut code = format!("; ExtensionCall: {}\n", extension);
                for (i, arg) in args.iter().enumerate() {
                    code.push_str(&format!("    mov r{}, {}\n", i, arg));
                }
                code.push_str(&format!(
                    "    ext.call {}, @\"{}\"\n",
                    result_reg, extension
                ));
                code
            }

            SlotType::ErrorResponse {
                socket_reg,
                error_code,
                error_message,
                close_after,
            } => {
                let mut code = format!("; ErrorResponse: {} \"{}\"\n", error_code, error_message);
                code.push_str(&format!("    lea r0, err_{}\n", error_code));
                code.push_str(&format!("    load r1, [err_{}_len]\n", error_code));
                code.push_str(&format!("    mov r2, {}\n", socket_reg));
                code.push_str("    io.send r2, r2, r0, r1\n");
                if *close_after {
                    code.push_str(&format!("    mov r0, {}\n", socket_reg));
                    code.push_str("    io.close r0, r0\n");
                }
                code
            }

            SlotType::ResponseBuilder {
                template,
                variables: _,
                output_reg,
                length_reg,
            } => {
                let mut code = format!("; ResponseBuilder: \"{}\"\n", template);
                code.push_str(&format!("    lea r0, {}\n", output_reg));
                code.push_str("    mov r1, 0  ; offset\n");

                for c in template.chars() {
                    if c == '{' || c == '}' {
                        continue; // Skip variable markers (simplified)
                    }
                    code.push_str(&format!(
                        "    mov r2, {}  ; '{}'\n",
                        c as u8,
                        c.escape_default()
                    ));
                    code.push_str("    store.b r2, [r0 + r1]\n");
                    code.push_str("    addi r1, r1, 1\n");
                }

                code.push_str(&format!("    mov {}, r1\n", length_reg));
                code
            }

            _ => format!("; {} (not implemented)\n    nop\n", slot_type.name()),
        }
    }

    /// Estimate difficulty based on slot type complexity
    fn estimate_difficulty(&self, slot_type: &SlotType) -> u8 {
        let (min, max) = slot_type.instruction_range();
        let avg = (min + max) / 2;

        if avg <= 5 {
            1
        } else if avg <= 15 {
            2
        } else if avg <= 30 {
            3
        } else if avg <= 50 {
            4
        } else {
            5
        }
    }

    /// Add an example and update statistics
    fn add_example(&mut self, example: SlotTrainingExample) {
        *self
            .stats
            .by_type
            .entry(example.slot_type.clone())
            .or_insert(0) += 1;
        *self
            .stats
            .by_category
            .entry(example.category.clone())
            .or_insert(0) += 1;
        self.stats.total_examples += 1;
        self.examples.push(example);
    }

    /// Write examples to JSONL file
    pub fn write_jsonl(&self, path: &Path) -> std::io::Result<usize> {
        let file = File::create(path)?;
        let mut writer = BufWriter::new(file);

        for example in &self.examples {
            let json = serde_json::to_string(example).map_err(std::io::Error::other)?;
            writeln!(writer, "{}", json)?;
        }

        writer.flush()?;
        Ok(self.examples.len())
    }

    /// Get statistics
    pub fn stats(&self) -> &ExtractionStats {
        &self.stats
    }

    /// Get all examples
    pub fn examples(&self) -> &[SlotTrainingExample] {
        &self.examples
    }

    /// Generate synthetic variations for data augmentation
    pub fn augment(&mut self, variations_per_example: usize) {
        let original_examples: Vec<_> = self.examples.clone();

        for example in original_examples {
            for _ in 0..variations_per_example {
                if let Some(augmented) = self.create_variation(&example) {
                    self.add_example(augmented);
                }
            }
        }
    }

    /// Create a variation of an example
    fn create_variation(&self, example: &SlotTrainingExample) -> Option<SlotTrainingExample> {
        // For now, just vary register names
        let mut varied = example.clone();

        // Simple augmentation: vary some string fields
        match &mut varied.input.params {
            SlotParams::StateTransition { new_state, .. } => {
                // Vary state names
                let old_state = new_state.clone();
                let new_state_varied = format!("{}_VAR", old_state);
                varied.output = varied.output.replace(&old_state, &new_state_varied);
                *new_state = new_state_varied;
            }
            SlotParams::StateCheck { valid_states, .. } => {
                // Add variation to state names
                for state in valid_states.iter_mut() {
                    *state = format!("{}_VAR", state);
                }
            }
            _ => return None, // Skip types we can't easily vary
        }

        varied.source = Some(format!(
            "{} (augmented)",
            example.source.as_deref().unwrap_or("unknown")
        ));
        Some(varied)
    }
}

impl Default for SlotTrainingExtractor {
    fn default() -> Self {
        Self::new()
    }
}

/// Extract slot training data from protocol specs
pub fn extract_slot_training_data(
    specs_dir: &Path,
    output_path: &Path,
) -> Result<ExtractionStats, String> {
    let mut extractor = SlotTrainingExtractor::new();

    extractor
        .extract_from_specs(specs_dir)
        .map_err(|e| format!("Failed to extract from specs: {}", e))?;

    extractor
        .write_jsonl(output_path)
        .map_err(|e| format!("Failed to write JSONL: {}", e))?;

    Ok(extractor.stats().clone())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slot_to_params() {
        let extractor = SlotTrainingExtractor::new();

        let slot_type = SlotType::StateTransition {
            state_reg: "r13".to_string(),
            new_state: "STATE_READY".to_string(),
        };

        let params = extractor.slot_type_to_params(&slot_type);
        assert!(params.is_some());

        if let Some(SlotParams::StateTransition {
            state_reg,
            new_state,
        }) = params
        {
            assert_eq!(state_reg, "r13");
            assert_eq!(new_state, "STATE_READY");
        }
    }

    #[test]
    fn test_generate_reference_code() {
        let extractor = SlotTrainingExtractor::new();

        let slot_type = SlotType::StateCheck {
            state_reg: "r13".to_string(),
            valid_states: vec!["STATE_A".to_string(), "STATE_B".to_string()],
            ok_label: "state_ok".to_string(),
            error_label: "state_error".to_string(),
        };

        let code = extractor.generate_reference_code(&slot_type);
        assert!(code.contains("StateCheck"));
        assert!(code.contains("STATE_A"));
        assert!(code.contains("STATE_B"));
        assert!(code.contains("state_ok"));
        assert!(code.contains("state_error"));
    }

    #[test]
    fn test_extract_from_protocol_spec() {
        use super::super::parser::{Greeting, ProtocolState, Transport};

        let mut extractor = SlotTrainingExtractor::new();

        // Create a minimal protocol spec
        let spec = ProtocolSpec {
            name: "test".to_string(),
            description: "Test protocol".to_string(),
            version: "1.0".to_string(),
            port: 1234,
            transport: Transport::Tcp,
            line_ending: Some("\\r\\n".to_string()),
            greeting: Some(Greeting {
                format: "220 Test\r\n".to_string(),
            }),
            states: vec![ProtocolState {
                name: "INIT".to_string(),
                initial: true,
                terminal: false,
                description: None,
            }],
            commands: vec![],
            errors: HashMap::new(),
            tests: vec![],
        };

        let result = extractor.extract_from_protocol_spec(&spec, "test.json");
        assert!(result.is_ok());
    }

    #[test]
    fn test_difficulty_estimation() {
        let extractor = SlotTrainingExtractor::new();

        // Simple slot
        let simple = SlotType::StateTransition {
            state_reg: "r13".to_string(),
            new_state: "STATE_A".to_string(),
        };
        assert!(extractor.estimate_difficulty(&simple) <= 2);

        // Complex slot
        let complex = SlotType::PatternSwitch {
            input_reg: "r0".to_string(),
            cases: vec![
                ("HELO".to_string(), "helo".to_string()),
                ("EHLO".to_string(), "ehlo".to_string()),
                ("QUIT".to_string(), "quit".to_string()),
                ("MAIL".to_string(), "mail".to_string()),
                ("RCPT".to_string(), "rcpt".to_string()),
            ],
            default_label: "unknown".to_string(),
        };
        assert!(extractor.estimate_difficulty(&complex) >= 3);
    }
}
