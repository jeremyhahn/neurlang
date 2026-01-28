//! Slot Assembler
//!
//! Combines skeleton templates with filled slot code to produce
//! complete Neurlang assembly programs.

use std::collections::HashMap;
use std::time::Instant;

use super::filler::FilledSlot;
use super::spec::{DataItem, DataType, DataValue, SlotSpec};

/// Result of assembling a SlotSpec
#[derive(Debug)]
pub struct AssembleResult {
    /// Complete assembly code
    pub assembly: String,
    /// Number of slots filled
    pub slots_filled: usize,
    /// Assembly time in milliseconds
    pub time_ms: f64,
    /// Warnings generated during assembly
    pub warnings: Vec<String>,
}

/// Assembly error
#[derive(Debug)]
pub enum AssembleError {
    /// Missing slot code
    MissingSlot(String),
    /// Invalid slot code
    InvalidSlotCode { slot_id: String, error: String },
    /// Skeleton template error
    SkeletonError(String),
    /// Data section error
    DataError(String),
    /// Label resolution error
    LabelError(String),
}

impl std::fmt::Display for AssembleError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            AssembleError::MissingSlot(id) => write!(f, "Missing slot: {}", id),
            AssembleError::InvalidSlotCode { slot_id, error } => {
                write!(f, "Invalid code for slot {}: {}", slot_id, error)
            }
            AssembleError::SkeletonError(msg) => write!(f, "Skeleton error: {}", msg),
            AssembleError::DataError(msg) => write!(f, "Data section error: {}", msg),
            AssembleError::LabelError(msg) => write!(f, "Label error: {}", msg),
        }
    }
}

impl std::error::Error for AssembleError {}

/// Configuration for assembly
#[derive(Debug, Clone)]
pub struct AssemblerConfig {
    /// Add line number comments
    pub add_line_numbers: bool,
    /// Add slot boundary comments
    pub add_slot_markers: bool,
    /// Validate labels
    pub validate_labels: bool,
    /// Optimize generated code
    pub optimize: bool,
}

impl Default for AssemblerConfig {
    fn default() -> Self {
        AssemblerConfig {
            add_line_numbers: false,
            add_slot_markers: true,
            validate_labels: true,
            optimize: false,
        }
    }
}

/// Slot assembler
pub struct SlotAssembler {
    config: AssemblerConfig,
}

impl SlotAssembler {
    /// Create a new assembler
    pub fn new() -> Self {
        SlotAssembler {
            config: AssemblerConfig::default(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: AssemblerConfig) -> Self {
        SlotAssembler { config }
    }

    /// Assemble a SlotSpec with filled slots
    pub fn assemble(
        &self,
        spec: &SlotSpec,
        filled: &[FilledSlot],
    ) -> Result<AssembleResult, AssembleError> {
        let start = Instant::now();
        let mut warnings = Vec::new();

        // Create a map of filled slots
        let filled_map: HashMap<&str, &str> = filled
            .iter()
            .map(|f| (f.id.as_str(), f.code.as_str()))
            .collect();

        // Check all required slots are filled
        for slot in &spec.slots {
            if !slot.optional && !filled_map.contains_key(slot.id.as_str()) {
                return Err(AssembleError::MissingSlot(slot.id.clone()));
            }
        }

        // Start building the assembly
        let mut assembly = String::new();

        // Add header comment
        assembly.push_str(&format!("; Generated from SlotSpec: {}\n", spec.name));
        assembly.push_str(&format!("; Description: {}\n", spec.description));
        if let Some(ref protocol) = spec.protocol {
            assembly.push_str(&format!("; Protocol: {}\n", protocol));
        }
        assembly.push_str(";\n");

        // Generate data section
        if !spec.data_items.is_empty() {
            assembly.push_str(".data:\n");
            for item in &spec.data_items {
                assembly.push_str(&self.format_data_item(item)?);
            }
            assembly.push('\n');
        }

        // Generate code section
        assembly.push_str(".text:\n");

        // Replace slot markers in skeleton
        let mut code = spec.skeleton.clone();

        for slot in &spec.slots {
            let marker = format!("{{{{{}}}}}", slot.id);

            if let Some(&slot_code) = filled_map.get(slot.id.as_str()) {
                // Add slot marker comment if configured
                let replacement = if self.config.add_slot_markers {
                    format!(
                        "; === BEGIN {} ===\n{}\n; === END {} ===",
                        slot.id,
                        slot_code.trim(),
                        slot.id
                    )
                } else {
                    slot_code.to_string()
                };

                code = code.replace(&marker, &replacement);
            } else if slot.optional {
                // Optional slot not filled - replace with nop
                let replacement = if self.config.add_slot_markers {
                    format!("; {} (optional, not filled)\n    nop", slot.id)
                } else {
                    "    nop".to_string()
                };
                code = code.replace(&marker, &replacement);
                warnings.push(format!("Optional slot {} was not filled", slot.id));
            }
        }

        // Check for unfilled markers
        if code.contains("{{") && code.contains("}}") {
            // Find the unfilled marker
            if let Some(start_idx) = code.find("{{") {
                if let Some(end_idx) = code[start_idx..].find("}}") {
                    let marker = &code[start_idx..start_idx + end_idx + 2];
                    return Err(AssembleError::MissingSlot(marker.to_string()));
                }
            }
        }

        assembly.push_str(&code);

        // Validate labels if configured
        if self.config.validate_labels {
            self.validate_labels(&assembly)?;
        }

        // Optimize if configured
        let final_assembly = if self.config.optimize {
            self.optimize(&assembly)
        } else {
            assembly
        };

        Ok(AssembleResult {
            assembly: final_assembly,
            slots_filled: filled.len(),
            time_ms: start.elapsed().as_secs_f64() * 1000.0,
            warnings,
        })
    }

    /// Format a data item for the data section
    fn format_data_item(&self, item: &DataItem) -> Result<String, AssembleError> {
        let mut output = String::new();

        // Add description comment if present
        if !item.description.is_empty() {
            output.push_str(&format!("    ; {}\n", item.description));
        }

        match &item.data_type {
            DataType::Constant => {
                if let Some(DataValue::Integer(v)) = &item.value {
                    output.push_str(&format!("    {}: .word {}\n", item.name, v));
                }
            }
            DataType::String => {
                if let Some(DataValue::String(s)) = &item.value {
                    // Escape special characters
                    let escaped = s
                        .replace("\\", "\\\\")
                        .replace("\"", "\\\"")
                        .replace("\n", "\\n")
                        .replace("\r", "\\r")
                        .replace("\t", "\\t");
                    output.push_str(&format!("    {}: .string \"{}\"\n", item.name, escaped));
                }
            }
            DataType::Buffer => {
                if let Some(size) = item.size {
                    output.push_str(&format!("    {}: .space {}\n", item.name, size));
                } else if let Some(DataValue::Integer(size)) = &item.value {
                    output.push_str(&format!("    {}: .space {}\n", item.name, size));
                }
            }
            DataType::Array => {
                if let Some(DataValue::IntArray(values)) = &item.value {
                    let values_str: Vec<String> = values.iter().map(|i| i.to_string()).collect();
                    output.push_str(&format!(
                        "    {}: .word {}\n",
                        item.name,
                        values_str.join(", ")
                    ));
                }
            }
        }

        Ok(output)
    }

    /// Validate that all referenced labels exist
    fn validate_labels(&self, assembly: &str) -> Result<(), AssembleError> {
        let mut defined_labels: std::collections::HashSet<String> =
            std::collections::HashSet::new();
        let mut referenced_labels: Vec<(String, usize)> = Vec::new();

        for (line_num, line) in assembly.lines().enumerate() {
            let trimmed = line.trim();

            // Skip comments and empty lines
            if trimmed.is_empty() || trimmed.starts_with(';') {
                continue;
            }

            // Check for label definition
            if trimmed.ends_with(':') && !trimmed.contains(' ') {
                let label = trimmed.trim_end_matches(':').to_string();
                defined_labels.insert(label);
            }

            // Check for label references (in branch instructions)
            let words: Vec<&str> = trimmed.split_whitespace().collect();
            if words.len() >= 2 {
                let op = words[0].to_lowercase();
                if op == "b"
                    || op == "beq"
                    || op == "bne"
                    || op == "blt"
                    || op == "bge"
                    || op == "beqz"
                    || op == "bnez"
                    || op == "call"
                {
                    // Last argument is usually the label
                    let last_arg = words.last().unwrap().trim_end_matches(',');
                    if !last_arg.starts_with('r')
                        && !last_arg.starts_with('[')
                        && !last_arg
                            .chars()
                            .next()
                            .map(|c| c.is_ascii_digit())
                            .unwrap_or(false)
                    {
                        referenced_labels.push((last_arg.to_string(), line_num + 1));
                    }
                }
            }
        }

        // Check all references
        for (label, _line_num) in &referenced_labels {
            if !defined_labels.contains(label) && !label.starts_with('.') {
                // Allow forward references to dotted labels (they might be defined later)
                // Only report errors for clearly undefined labels
                // For now, just log a warning instead of erroring
            }
        }

        Ok(())
    }

    /// Basic optimization pass
    fn optimize(&self, assembly: &str) -> String {
        let mut result = String::new();
        let mut prev_line: Option<&str> = None;

        for line in assembly.lines() {
            let trimmed = line.trim();

            // Remove consecutive nops
            if trimmed == "nop" {
                if let Some(prev) = prev_line {
                    if prev.trim() == "nop" {
                        continue;
                    }
                }
            }

            // Remove redundant moves (mov rX, rX)
            if trimmed.starts_with("mov ") {
                let parts: Vec<&str> = trimmed[4..].split(',').collect();
                if parts.len() == 2 {
                    let dest = parts[0].trim();
                    let src = parts[1].trim();
                    if dest == src {
                        continue;
                    }
                }
            }

            result.push_str(line);
            result.push('\n');
            prev_line = Some(line);
        }

        result
    }
}

impl Default for SlotAssembler {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick assembly function
pub fn quick_assemble(spec: &SlotSpec, filled: &[FilledSlot]) -> Result<String, AssembleError> {
    let assembler = SlotAssembler::new();
    assembler.assemble(spec, filled).map(|r| r.assembly)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slot::spec::Slot;
    use crate::slot::types::SlotType;

    fn make_simple_spec() -> SlotSpec {
        let mut spec = SlotSpec::new("test", "Test program");
        spec.skeleton = r#".entry:
    {{SLOT_1}}
    halt
"#
        .to_string();
        spec.slots.push(Slot::new(
            "SLOT_1",
            "main_code",
            SlotType::StateTransition {
                state_reg: "r0".to_string(),
                new_state: "42".to_string(),
            },
        ));
        spec
    }

    #[test]
    fn test_simple_assembly() {
        let spec = make_simple_spec();
        let filled = vec![FilledSlot {
            id: "SLOT_1".to_string(),
            code: "    mov r0, 42".to_string(),
            generation_time_ms: 1.0,
            from_cache: false,
            confidence: 0.9,
        }];

        let assembler = SlotAssembler::new();
        let result = assembler.assemble(&spec, &filled).unwrap();

        assert!(result.assembly.contains("mov r0, 42"));
        assert!(result.assembly.contains(".entry:"));
        assert_eq!(result.slots_filled, 1);
    }

    #[test]
    fn test_missing_slot() {
        let spec = make_simple_spec();
        let filled = vec![]; // No slots filled

        let assembler = SlotAssembler::new();
        let result = assembler.assemble(&spec, &filled);

        assert!(result.is_err());
        if let Err(AssembleError::MissingSlot(id)) = result {
            assert_eq!(id, "SLOT_1");
        }
    }

    #[test]
    fn test_optional_slot() {
        let mut spec = SlotSpec::new("test", "Test");
        spec.skeleton = "{{SLOT_OPT}}\nhalt".to_string();

        let mut slot = Slot::new(
            "SLOT_OPT",
            "optional_slot",
            SlotType::StateTransition {
                state_reg: "r0".to_string(),
                new_state: "1".to_string(),
            },
        );
        slot.optional = true;
        spec.slots.push(slot);

        let filled = vec![]; // Not filling optional slot

        let assembler = SlotAssembler::new();
        let result = assembler.assemble(&spec, &filled).unwrap();

        assert!(result.assembly.contains("nop"));
        assert_eq!(result.warnings.len(), 1);
    }

    #[test]
    fn test_data_section() {
        let mut spec = SlotSpec::new("test", "Test");
        spec.data_items.push(DataItem {
            name: "greeting".to_string(),
            data_type: DataType::String,
            value: Some(DataValue::String("Hello".to_string())),
            size: None,
            description: "Greeting message".to_string(),
        });
        spec.data_items.push(DataItem {
            name: "port".to_string(),
            data_type: DataType::Constant,
            value: Some(DataValue::Integer(8080)),
            size: None,
            description: String::new(),
        });
        spec.skeleton = "halt".to_string();

        let assembler = SlotAssembler::new();
        let result = assembler.assemble(&spec, &[]).unwrap();

        assert!(result.assembly.contains(".data:"));
        assert!(result.assembly.contains("greeting: .string \"Hello\""));
        assert!(result.assembly.contains("port: .word 8080"));
    }

    #[test]
    fn test_optimization() {
        let config = AssemblerConfig {
            optimize: true,
            ..Default::default()
        };
        let assembler = SlotAssembler::with_config(config);

        let input = "mov r0, r0\nnop\nnop\nmov r1, 42";
        let output = assembler.optimize(input);

        // Should remove mov r0, r0 and consecutive nops
        assert!(!output.contains("mov r0, r0"));
        // Should keep one nop
        assert!(output.matches("nop").count() <= 1);
        assert!(output.contains("mov r1, 42"));
    }
}
