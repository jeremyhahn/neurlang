//! SlotSpec Definition
//!
//! The universal intermediate format that both rule-based and LLM paths produce.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

use super::types::SlotType;

/// The universal intermediate format for slot-based generation
///
/// Both rule-based decomposition and LLM decomposition produce a SlotSpec,
/// which is then filled by the slot filling model.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotSpec {
    /// Program name
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Protocol this spec was generated from (if any)
    pub protocol: Option<String>,

    /// Template skeleton used (if any)
    pub template: Option<String>,

    /// Data section items (constants, buffers, strings)
    pub data_items: Vec<DataItem>,

    /// Code skeleton with `{{SLOT_N}}` markers
    ///
    /// Example:
    /// ```text
    /// .entry:
    ///     ; Server initialization
    ///     {{SLOT_1}}
    /// .main_loop:
    ///     ; Read command
    ///     {{SLOT_2}}
    ///     ; Dispatch
    ///     {{SLOT_3}}
    /// ```
    pub skeleton: String,

    /// Slots to be filled by the model
    pub slots: Vec<Slot>,

    /// Integration test cases for the complete program
    pub tests: Vec<TestCase>,

    /// Metadata for tracking and debugging
    #[serde(default)]
    pub metadata: HashMap<String, String>,
}

impl SlotSpec {
    /// Create a new empty SlotSpec
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        SlotSpec {
            name: name.into(),
            description: description.into(),
            protocol: None,
            template: None,
            data_items: Vec::new(),
            skeleton: String::new(),
            slots: Vec::new(),
            tests: Vec::new(),
            metadata: HashMap::new(),
        }
    }

    /// Add a data item
    pub fn add_data(&mut self, item: DataItem) {
        self.data_items.push(item);
    }

    /// Add a slot
    pub fn add_slot(&mut self, slot: Slot) {
        self.slots.push(slot);
    }

    /// Add a test case
    pub fn add_test(&mut self, test: TestCase) {
        self.tests.push(test);
    }

    /// Get a slot by ID
    pub fn get_slot(&self, id: &str) -> Option<&Slot> {
        self.slots.iter().find(|s| s.id == id)
    }

    /// Get a mutable slot by ID
    pub fn get_slot_mut(&mut self, id: &str) -> Option<&mut Slot> {
        self.slots.iter_mut().find(|s| s.id == id)
    }

    /// Count total estimated instructions
    pub fn estimated_instructions(&self) -> usize {
        self.slots
            .iter()
            .map(|s| {
                let (min, max) = s.slot_type.instruction_range();
                (min + max) / 2
            })
            .sum()
    }

    /// Generate the final assembly by replacing slot markers
    pub fn assemble(&self, filled_slots: &HashMap<String, String>) -> Result<String, String> {
        let mut result = self.skeleton.clone();

        for slot in &self.slots {
            let marker = format!("{{{{{}}}}}", slot.id);
            if let Some(code) = filled_slots.get(&slot.id) {
                result = result.replace(&marker, code);
            } else {
                return Err(format!("Missing filled code for slot {}", slot.id));
            }
        }

        // Check for any unfilled markers
        if result.contains("{{SLOT_") {
            return Err("Some slots were not filled".to_string());
        }

        Ok(result)
    }
}

/// A slot to be filled by the model
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Slot {
    /// Unique identifier (e.g., "SLOT_1", "SLOT_2")
    pub id: String,

    /// Descriptive name (e.g., "helo_pattern_match")
    pub name: String,

    /// The slot type with parameters
    pub slot_type: SlotType,

    /// Context available when filling this slot
    pub context: SlotContext,

    /// Optional per-slot unit test
    pub unit_test: Option<SlotTest>,

    /// Whether this slot is optional
    #[serde(default)]
    pub optional: bool,

    /// Dependencies on other slots (must be filled first)
    #[serde(default)]
    pub depends_on: Vec<String>,
}

impl Slot {
    /// Create a new slot
    pub fn new(id: impl Into<String>, name: impl Into<String>, slot_type: SlotType) -> Self {
        Slot {
            id: id.into(),
            name: name.into(),
            slot_type,
            context: SlotContext::default(),
            unit_test: None,
            optional: false,
            depends_on: Vec::new(),
        }
    }

    /// Set the context
    pub fn with_context(mut self, context: SlotContext) -> Self {
        self.context = context;
        self
    }

    /// Add a unit test
    pub fn with_test(mut self, test: SlotTest) -> Self {
        self.unit_test = Some(test);
        self
    }

    /// Mark as optional
    pub fn optional(mut self) -> Self {
        self.optional = true;
        self
    }

    /// Add a dependency
    pub fn depends_on(mut self, slot_id: impl Into<String>) -> Self {
        self.depends_on.push(slot_id.into());
        self
    }
}

/// Context available when filling a slot
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SlotContext {
    /// Available registers and their contents/purpose
    /// e.g., {"r0": "input_buffer", "r10": "socket_fd"}
    pub registers: HashMap<String, String>,

    /// Available labels that can be jumped to
    pub labels: Vec<String>,

    /// Available data section references
    pub data_refs: Vec<String>,

    /// Available state constants
    pub state_constants: Vec<String>,

    /// Caller-saved registers (can be used as temporaries)
    #[serde(default)]
    pub temp_registers: Vec<String>,

    /// Additional context notes
    #[serde(default)]
    pub notes: Vec<String>,
}

impl SlotContext {
    /// Create a new empty context
    pub fn new() -> Self {
        SlotContext::default()
    }

    /// Add a register mapping
    pub fn register(mut self, reg: impl Into<String>, description: impl Into<String>) -> Self {
        self.registers.insert(reg.into(), description.into());
        self
    }

    /// Add a label
    pub fn label(mut self, label: impl Into<String>) -> Self {
        self.labels.push(label.into());
        self
    }

    /// Add a data reference
    pub fn data_ref(mut self, name: impl Into<String>) -> Self {
        self.data_refs.push(name.into());
        self
    }

    /// Add a state constant
    pub fn state_constant(mut self, name: impl Into<String>) -> Self {
        self.state_constants.push(name.into());
        self
    }

    /// Add temp registers
    pub fn temp_regs(mut self, regs: &[&str]) -> Self {
        self.temp_registers
            .extend(regs.iter().map(|s| s.to_string()));
        self
    }
}

/// Per-slot unit test
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SlotTest {
    /// Assembly code to set up test state
    pub setup: String,

    /// Test input specification
    pub input: TestInput,

    /// Expected outputs/behavior
    pub expected: TestExpected,
}

/// Test input specification
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TestInput {
    /// Register values to set
    #[serde(default)]
    pub registers: HashMap<String, u64>,

    /// Memory buffer contents (address -> bytes)
    #[serde(default)]
    pub memory: HashMap<u64, Vec<u8>>,

    /// String buffer contents (buffer_name -> string)
    #[serde(default)]
    pub buffers: HashMap<String, String>,
}

/// Expected test outputs
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct TestExpected {
    /// Expected register values after execution
    #[serde(default)]
    pub registers: HashMap<String, u64>,

    /// Expected branch taken (label name)
    #[serde(default)]
    pub branch_taken: Option<String>,

    /// Expected memory contents
    #[serde(default)]
    pub memory: HashMap<u64, Vec<u8>>,

    /// Expected string in buffer
    #[serde(default)]
    pub buffer_contains: HashMap<String, String>,
}

/// Data section item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataItem {
    /// Item name/label
    pub name: String,

    /// Data type
    pub data_type: DataType,

    /// Initial value (if constant)
    pub value: Option<DataValue>,

    /// Size in bytes (for buffers)
    pub size: Option<usize>,

    /// Description
    #[serde(default)]
    pub description: String,
}

impl DataItem {
    /// Create a constant
    pub fn constant(name: impl Into<String>, value: u64) -> Self {
        DataItem {
            name: name.into(),
            data_type: DataType::Constant,
            value: Some(DataValue::Integer(value)),
            size: None,
            description: String::new(),
        }
    }

    /// Create a string constant
    pub fn string(name: impl Into<String>, value: impl Into<String>) -> Self {
        let s = value.into();
        let size = s.len() + 1; // +1 for null terminator
        DataItem {
            name: name.into(),
            data_type: DataType::String,
            value: Some(DataValue::String(s)),
            size: Some(size),
            description: String::new(),
        }
    }

    /// Create a buffer
    pub fn buffer(name: impl Into<String>, size: usize) -> Self {
        DataItem {
            name: name.into(),
            data_type: DataType::Buffer,
            value: None,
            size: Some(size),
            description: String::new(),
        }
    }

    /// Add description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// Type of data item
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum DataType {
    /// Numeric constant (fits in register)
    Constant,
    /// String constant (null-terminated)
    String,
    /// Byte buffer (uninitialized or zero-filled)
    Buffer,
    /// Array of values
    Array,
}

/// Data value
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DataValue {
    /// Integer value
    Integer(u64),
    /// String value
    String(String),
    /// Byte array
    Bytes(Vec<u8>),
    /// Array of integers
    IntArray(Vec<u64>),
}

/// Integration test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestCase {
    /// Test name
    pub name: String,

    /// Test description
    #[serde(default)]
    pub description: String,

    /// Test steps
    pub steps: Vec<TestStep>,

    /// Setup SQL or commands (optional)
    #[serde(default)]
    pub setup: Option<String>,

    /// Expected final state name
    #[serde(default)]
    pub expected_state: Option<String>,
}

impl TestCase {
    /// Create a new test case
    pub fn new(name: impl Into<String>) -> Self {
        TestCase {
            name: name.into(),
            description: String::new(),
            steps: Vec::new(),
            setup: None,
            expected_state: None,
        }
    }

    /// Add a step
    pub fn step(mut self, step: TestStep) -> Self {
        self.steps.push(step);
        self
    }

    /// Add description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }
}

/// A single test step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestStep {
    /// Data to send (if any)
    #[serde(default)]
    pub send: Option<String>,

    /// Expected response (exact match)
    #[serde(default)]
    pub expect: Option<String>,

    /// Expected response pattern (regex)
    #[serde(default)]
    pub expect_pattern: Option<String>,

    /// Response must contain this substring
    #[serde(default)]
    pub expect_contains: Option<String>,

    /// Response must not contain this substring
    #[serde(default)]
    pub expect_not_contains: Option<String>,

    /// Timeout in milliseconds
    #[serde(default = "default_timeout")]
    pub timeout_ms: u32,

    /// Description of this step
    #[serde(default)]
    pub description: String,
}

fn default_timeout() -> u32 {
    5000
}

impl TestStep {
    /// Create a send-and-expect step
    pub fn send_expect(send: impl Into<String>, expect: impl Into<String>) -> Self {
        TestStep {
            send: Some(send.into()),
            expect: Some(expect.into()),
            expect_pattern: None,
            expect_contains: None,
            expect_not_contains: None,
            timeout_ms: 5000,
            description: String::new(),
        }
    }

    /// Create an expect-only step (for initial greeting)
    pub fn expect(expect: impl Into<String>) -> Self {
        TestStep {
            send: None,
            expect: Some(expect.into()),
            expect_pattern: None,
            expect_contains: None,
            expect_not_contains: None,
            timeout_ms: 5000,
            description: String::new(),
        }
    }

    /// Create a send-only step
    pub fn send(send: impl Into<String>) -> Self {
        TestStep {
            send: Some(send.into()),
            expect: None,
            expect_pattern: None,
            expect_contains: None,
            expect_not_contains: None,
            timeout_ms: 5000,
            description: String::new(),
        }
    }

    /// Set timeout
    pub fn timeout(mut self, ms: u32) -> Self {
        self.timeout_ms = ms;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slot::types::SlotType;

    #[test]
    fn test_slot_spec_basic() {
        let mut spec = SlotSpec::new("test_server", "A test server");
        spec.protocol = Some("smtp".to_string());

        spec.add_data(DataItem::constant("STATE_INIT", 0));
        spec.add_data(DataItem::buffer("input_buffer", 1024));

        let slot = Slot::new(
            "SLOT_1",
            "helo_matcher",
            SlotType::PatternMatch {
                pattern: "HELO {domain}".to_string(),
                input_reg: "r0".to_string(),
                captures: vec![],
                match_label: "match".to_string(),
                no_match_label: "no_match".to_string(),
            },
        );
        spec.add_slot(slot);

        assert_eq!(spec.name, "test_server");
        assert_eq!(spec.slots.len(), 1);
        assert_eq!(spec.data_items.len(), 2);
    }

    #[test]
    fn test_assemble() {
        let mut spec = SlotSpec::new("test", "Test");
        spec.skeleton = r#"
.entry:
    {{SLOT_1}}
    {{SLOT_2}}
"#
        .to_string();

        spec.add_slot(Slot::new(
            "SLOT_1",
            "first",
            SlotType::StateTransition {
                state_reg: "r20".to_string(),
                new_state: "STATE_A".to_string(),
            },
        ));
        spec.add_slot(Slot::new(
            "SLOT_2",
            "second",
            SlotType::StateTransition {
                state_reg: "r20".to_string(),
                new_state: "STATE_B".to_string(),
            },
        ));

        let mut filled = HashMap::new();
        filled.insert("SLOT_1".to_string(), "mov r20, STATE_A".to_string());
        filled.insert("SLOT_2".to_string(), "mov r20, STATE_B".to_string());

        let result = spec.assemble(&filled).unwrap();
        assert!(result.contains("mov r20, STATE_A"));
        assert!(result.contains("mov r20, STATE_B"));
        assert!(!result.contains("{{"));
    }

    #[test]
    fn test_assemble_missing_slot() {
        let mut spec = SlotSpec::new("test", "Test");
        spec.skeleton = "{{SLOT_1}}".to_string();
        spec.add_slot(Slot::new(
            "SLOT_1",
            "first",
            SlotType::StateTransition {
                state_reg: "r20".to_string(),
                new_state: "STATE_A".to_string(),
            },
        ));

        let filled = HashMap::new();
        let result = spec.assemble(&filled);
        assert!(result.is_err());
    }

    #[test]
    fn test_data_items() {
        let constant = DataItem::constant("PORT", 8080);
        assert_eq!(constant.data_type, DataType::Constant);

        let string = DataItem::string("GREETING", "Hello\r\n");
        assert_eq!(string.data_type, DataType::String);
        assert_eq!(string.size, Some(8)); // 7 + null

        let buffer = DataItem::buffer("input", 1024);
        assert_eq!(buffer.data_type, DataType::Buffer);
        assert_eq!(buffer.size, Some(1024));
    }

    #[test]
    fn test_test_case() {
        let test = TestCase::new("basic_session")
            .with_description("Tests basic SMTP session")
            .step(TestStep::expect("220"))
            .step(TestStep::send_expect(
                "HELO test.com\r\n",
                "250 Hello test.com\r\n",
            ))
            .step(TestStep::send_expect("QUIT\r\n", "221 Bye\r\n"));

        assert_eq!(test.steps.len(), 3);
        assert_eq!(test.steps[0].send, None);
        assert!(test.steps[0].expect.as_ref().unwrap().contains("220"));
    }
}
