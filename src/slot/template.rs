//! Template Expansion Engine
//!
//! Converts protocol specifications + skeleton templates into SlotSpecs
//! ready for the slot filling model.

use std::collections::HashMap;
use std::fs;

use super::parser::{CaptureSpec, CommandHandler, HandlerType, ProtocolCommand, ProtocolSpec};
use super::spec::{DataItem, Slot, SlotContext, SlotSpec, TestCase, TestStep};
use super::types::{Capture, CaptureType, SlotType};

/// Template expansion error
#[derive(Debug)]
pub enum TemplateError {
    /// IO error
    Io(std::io::Error),
    /// Template not found
    NotFound(String),
    /// Invalid template syntax
    Syntax(String),
    /// Missing required field
    Missing(String),
    /// Protocol spec error
    Protocol(String),
}

impl std::fmt::Display for TemplateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TemplateError::Io(e) => write!(f, "IO error: {}", e),
            TemplateError::NotFound(s) => write!(f, "Template not found: {}", s),
            TemplateError::Syntax(s) => write!(f, "Syntax error: {}", s),
            TemplateError::Missing(s) => write!(f, "Missing field: {}", s),
            TemplateError::Protocol(s) => write!(f, "Protocol error: {}", s),
        }
    }
}

impl std::error::Error for TemplateError {}

impl From<std::io::Error> for TemplateError {
    fn from(e: std::io::Error) -> Self {
        TemplateError::Io(e)
    }
}

/// Template expander configuration
#[derive(Debug, Clone)]
pub struct ExpanderConfig {
    /// Directory containing skeleton templates
    pub templates_dir: String,
    /// Directory containing protocol specs
    pub specs_dir: String,
    /// Default hostname for responses
    pub hostname: String,
    /// Input buffer size
    pub input_buffer_size: usize,
    /// Output buffer size
    pub output_buffer_size: usize,
    /// Maximum connections
    pub max_connections: usize,
}

impl Default for ExpanderConfig {
    fn default() -> Self {
        ExpanderConfig {
            templates_dir: "templates".to_string(),
            specs_dir: "specs/protocols".to_string(),
            hostname: "localhost".to_string(),
            input_buffer_size: 4096,
            output_buffer_size: 4096,
            max_connections: 100,
        }
    }
}

/// Template expander that converts protocol specs to SlotSpecs
pub struct TemplateExpander {
    config: ExpanderConfig,
    /// Cached skeleton templates
    skeletons: HashMap<String, String>,
}

impl TemplateExpander {
    /// Create a new template expander
    pub fn new(config: ExpanderConfig) -> Self {
        TemplateExpander {
            config,
            skeletons: HashMap::new(),
        }
    }

    /// Create with default config
    pub fn with_defaults() -> Self {
        Self::new(ExpanderConfig::default())
    }

    /// Load a skeleton template
    pub fn load_skeleton(&mut self, name: &str) -> Result<&str, TemplateError> {
        if !self.skeletons.contains_key(name) {
            let path = format!("{}/{}.skeleton", self.config.templates_dir, name);
            let content = fs::read_to_string(&path).map_err(|_| TemplateError::NotFound(path))?;
            self.skeletons.insert(name.to_string(), content);
        }
        Ok(self.skeletons.get(name).unwrap())
    }

    /// Expand a protocol spec into a SlotSpec
    pub fn expand(&self, spec: &ProtocolSpec) -> Result<SlotSpec, TemplateError> {
        let mut slot_spec =
            SlotSpec::new(format!("{}_server", spec.name), spec.description.clone());
        slot_spec.protocol = Some(spec.name.clone());

        // Add state constants
        for (i, state) in spec.states.iter().enumerate() {
            slot_spec.add_data(
                DataItem::constant(format!("STATE_{}", state.name), i as u64)
                    .with_description(state.description.clone().unwrap_or_default()),
            );
        }

        // Add error response strings
        for (name, response) in &spec.errors {
            slot_spec.add_data(DataItem::string(
                format!("ERR_{}", name.to_uppercase()),
                response.clone(),
            ));
        }

        // Add buffers
        slot_spec.add_data(DataItem::buffer(
            "input_buffer",
            self.config.input_buffer_size,
        ));
        slot_spec.add_data(DataItem::buffer(
            "output_buffer",
            self.config.output_buffer_size,
        ));

        // Generate skeleton with slot markers
        let skeleton = self.generate_skeleton(spec)?;
        slot_spec.skeleton = skeleton;

        // Generate slots for each command
        let mut slot_id = 1;
        for cmd in &spec.commands {
            let slots = self.expand_command(cmd, spec, &mut slot_id)?;
            for slot in slots {
                slot_spec.add_slot(slot);
            }
        }

        // Add dispatch slot for command routing
        let dispatch_slot = self.create_dispatch_slot(spec, &mut slot_id)?;
        slot_spec.add_slot(dispatch_slot);

        // Convert protocol tests to SlotSpec tests
        for test in &spec.tests {
            let test_case = self.convert_test(test)?;
            slot_spec.add_test(test_case);
        }

        Ok(slot_spec)
    }

    /// Generate the code skeleton with slot markers
    fn generate_skeleton(&self, spec: &ProtocolSpec) -> Result<String, TemplateError> {
        let mut skeleton = String::new();

        // Header comment
        skeleton.push_str(&format!(
            "; {} Server\n; Generated from protocol spec: {}\n;\n",
            spec.name.to_uppercase(),
            spec.name
        ));
        skeleton.push_str("; @server: true\n\n");

        // Data section
        skeleton.push_str(".data:\n");
        skeleton.push_str("    ; State constants\n");
        for (i, state) in spec.states.iter().enumerate() {
            skeleton.push_str(&format!("    STATE_{} = {}\n", state.name, i));
        }
        skeleton.push('\n');
        skeleton.push_str("    ; Buffer sizes\n");
        skeleton.push_str(&format!(
            "    INPUT_BUFFER_SIZE = {}\n",
            self.config.input_buffer_size
        ));
        skeleton.push_str(&format!(
            "    OUTPUT_BUFFER_SIZE = {}\n",
            self.config.output_buffer_size
        ));
        skeleton.push('\n');

        // Text section
        skeleton.push_str(".text:\n");
        skeleton.push_str(".entry:\n");
        skeleton.push_str("    ; Initialize server\n");
        skeleton.push_str("    {{SLOT_INIT}}\n\n");

        skeleton.push_str(".accept_loop:\n");
        skeleton.push_str("    ; Accept new connection\n");
        skeleton.push_str("    {{SLOT_ACCEPT}}\n\n");

        // Greeting (if defined)
        if spec.greeting.is_some() {
            skeleton.push_str(".send_greeting:\n");
            skeleton.push_str("    ; Send initial greeting\n");
            skeleton.push_str("    {{SLOT_GREETING}}\n\n");
        }

        skeleton.push_str(".main_loop:\n");
        skeleton.push_str("    ; Read command from client\n");
        skeleton.push_str("    {{SLOT_READ_CMD}}\n\n");

        skeleton.push_str(".dispatch:\n");
        skeleton.push_str("    ; Dispatch to command handler\n");
        skeleton.push_str("    {{SLOT_DISPATCH}}\n\n");

        // Command handlers
        for cmd in &spec.commands {
            skeleton.push_str(&format!(".handle_{}:\n", cmd.name.to_lowercase()));
            skeleton.push_str(&format!("    ; Handle {} command\n", cmd.name));
            skeleton.push_str(&format!(
                "    {{{{SLOT_{}_CHECK}}}}\n",
                cmd.name.to_uppercase()
            ));
            skeleton.push_str(&format!(
                "    {{{{SLOT_{}_HANDLER}}}}\n",
                cmd.name.to_uppercase()
            ));

            // State transition if defined
            if let Some(ref next_state) = cmd.handler.next_state {
                if next_state != "SAME" {
                    skeleton.push_str(&format!(
                        "    {{{{SLOT_{}_TRANSITION}}}}\n",
                        cmd.name.to_uppercase()
                    ));
                }
            }
            skeleton.push_str("    b .main_loop\n\n");
        }

        // Error handlers
        skeleton.push_str(".error_syntax:\n");
        skeleton.push_str("    {{SLOT_ERROR_SYNTAX}}\n");
        skeleton.push_str("    b .main_loop\n\n");

        skeleton.push_str(".error_sequence:\n");
        skeleton.push_str("    {{SLOT_ERROR_SEQUENCE}}\n");
        skeleton.push_str("    b .main_loop\n\n");

        skeleton.push_str(".error_unknown:\n");
        skeleton.push_str("    {{SLOT_ERROR_UNKNOWN}}\n");
        skeleton.push_str("    b .main_loop\n\n");

        skeleton.push_str(".client_disconnect:\n");
        skeleton.push_str("    ; Close client socket and return to accept\n");
        skeleton.push_str("    {{SLOT_CLOSE}}\n");
        skeleton.push_str("    b .accept_loop\n");

        Ok(skeleton)
    }

    /// Expand a command into slots
    fn expand_command(
        &self,
        cmd: &ProtocolCommand,
        _spec: &ProtocolSpec,
        slot_id: &mut usize,
    ) -> Result<Vec<Slot>, TemplateError> {
        let mut slots = Vec::new();

        // State check slot (if not ANY)
        if !cmd.valid_states.contains(&"ANY".to_string()) {
            let state_check = Slot::new(
                format!("SLOT_{}_CHECK", cmd.name.to_uppercase()),
                format!("{}_state_check", cmd.name.to_lowercase()),
                SlotType::StateCheck {
                    state_reg: "r20".to_string(),
                    valid_states: cmd
                        .valid_states
                        .iter()
                        .map(|s| format!("STATE_{}", s))
                        .collect(),
                    ok_label: format!("{}_state_ok", cmd.name.to_lowercase()),
                    error_label: ".error_sequence".to_string(),
                },
            )
            .with_context(
                SlotContext::new()
                    .register("r20", "current state")
                    .label(format!("{}_state_ok", cmd.name.to_lowercase()))
                    .label(".error_sequence"),
            );
            slots.push(state_check);
            *slot_id += 1;
        }

        // Handler slot(s) based on handler type
        let handler_slots = self.expand_handler(cmd, &cmd.handler, slot_id)?;
        slots.extend(handler_slots);

        // State transition slot (if next_state defined)
        if let Some(ref next_state) = cmd.handler.next_state {
            if next_state != "SAME" {
                let transition = Slot::new(
                    format!("SLOT_{}_TRANSITION", cmd.name.to_uppercase()),
                    format!("{}_transition", cmd.name.to_lowercase()),
                    SlotType::StateTransition {
                        state_reg: "r20".to_string(),
                        new_state: format!("STATE_{}", next_state),
                    },
                )
                .with_context(
                    SlotContext::new()
                        .register("r20", "current state")
                        .state_constant(format!("STATE_{}", next_state)),
                );
                slots.push(transition);
                *slot_id += 1;
            }
        }

        Ok(slots)
    }

    /// Expand a handler into slots
    fn expand_handler(
        &self,
        cmd: &ProtocolCommand,
        handler: &CommandHandler,
        slot_id: &mut usize,
    ) -> Result<Vec<Slot>, TemplateError> {
        let mut slots = Vec::new();

        match handler.handler_type {
            HandlerType::SimpleResponse => {
                if let Some(ref response) = handler.response {
                    // Extract variables from pattern captures
                    let captures = cmd.capture_names();
                    let mut variables = HashMap::new();
                    for (i, cap) in captures.iter().enumerate() {
                        variables.insert(cap.clone(), format!("r{}", 3 + i));
                    }
                    // Add hostname if used
                    if response.contains("{hostname}") {
                        variables.insert("hostname".to_string(), "r30".to_string());
                    }

                    let response_slot = Slot::new(
                        format!("SLOT_{}_HANDLER", cmd.name.to_uppercase()),
                        format!("{}_response", cmd.name.to_lowercase()),
                        SlotType::ResponseBuilder {
                            template: response.clone(),
                            variables,
                            output_reg: "r6".to_string(),
                            length_reg: "r7".to_string(),
                        },
                    )
                    .with_context(
                        SlotContext::new()
                            .register("r6", "output_buffer")
                            .register("r7", "output_length")
                            .register("r10", "socket_fd")
                            .temp_regs(&["r1", "r2", "r8", "r9"]),
                    );
                    slots.push(response_slot);
                    *slot_id += 1;

                    // Send response slot
                    let send_slot = Slot::new(
                        format!("SLOT_{}_SEND", cmd.name.to_uppercase()),
                        format!("{}_send", cmd.name.to_lowercase()),
                        SlotType::SendResponse {
                            socket_reg: "r10".to_string(),
                            buffer_reg: "r6".to_string(),
                            length_reg: "r7".to_string(),
                        },
                    )
                    .with_context(
                        SlotContext::new()
                            .register("r10", "socket_fd")
                            .register("r6", "output_buffer")
                            .register("r7", "output_length"),
                    );
                    slots.push(send_slot);
                    *slot_id += 1;
                }
            }

            HandlerType::MultiLineResponse => {
                if let Some(ref lines) = handler.lines {
                    // Build multi-line response template
                    let template = lines
                        .iter()
                        .map(|l| format!("{}\r\n", l))
                        .collect::<String>();

                    let response_slot = Slot::new(
                        format!("SLOT_{}_HANDLER", cmd.name.to_uppercase()),
                        format!("{}_multiline", cmd.name.to_lowercase()),
                        SlotType::ResponseBuilder {
                            template,
                            variables: HashMap::new(),
                            output_reg: "r6".to_string(),
                            length_reg: "r7".to_string(),
                        },
                    );
                    slots.push(response_slot);
                    *slot_id += 1;

                    let send_slot = Slot::new(
                        format!("SLOT_{}_SEND", cmd.name.to_uppercase()),
                        format!("{}_send", cmd.name.to_lowercase()),
                        SlotType::SendResponse {
                            socket_reg: "r10".to_string(),
                            buffer_reg: "r6".to_string(),
                            length_reg: "r7".to_string(),
                        },
                    );
                    slots.push(send_slot);
                    *slot_id += 1;
                }
            }

            HandlerType::ValidatedResponse => {
                // Validation slot
                if let Some(ref validation) = handler.validation {
                    let validation_slot = Slot::new(
                        format!("SLOT_{}_VALIDATE", cmd.name.to_uppercase()),
                        format!("{}_validate", cmd.name.to_lowercase()),
                        SlotType::ValidationHook {
                            validation_type: format!("{:?}", validation.validation_type)
                                .to_lowercase(),
                            value_reg: "r3".to_string(),
                            ok_label: format!("{}_valid", cmd.name.to_lowercase()),
                            error_label: format!("{}_invalid", cmd.name.to_lowercase()),
                        },
                    );
                    slots.push(validation_slot);
                    *slot_id += 1;
                }

                // Success response
                if let Some(ref response) = handler.response_ok {
                    let ok_slot = Slot::new(
                        format!("SLOT_{}_OK", cmd.name.to_uppercase()),
                        format!("{}_ok_response", cmd.name.to_lowercase()),
                        SlotType::ResponseBuilder {
                            template: response.clone(),
                            variables: HashMap::new(),
                            output_reg: "r6".to_string(),
                            length_reg: "r7".to_string(),
                        },
                    );
                    slots.push(ok_slot);
                    *slot_id += 1;
                }

                // Error response
                if let Some(ref response) = handler.response_err {
                    let err_slot = Slot::new(
                        format!("SLOT_{}_ERR", cmd.name.to_uppercase()),
                        format!("{}_err_response", cmd.name.to_lowercase()),
                        SlotType::ErrorResponse {
                            socket_reg: "r10".to_string(),
                            error_code: 0,
                            error_message: response.clone(),
                            close_after: false,
                        },
                    );
                    slots.push(err_slot);
                    *slot_id += 1;
                }
            }

            HandlerType::MultilineReader => {
                // Send initial response
                if let Some(ref response) = handler.response {
                    let init_slot = Slot::new(
                        format!("SLOT_{}_INIT", cmd.name.to_uppercase()),
                        format!("{}_init", cmd.name.to_lowercase()),
                        SlotType::ResponseBuilder {
                            template: response.clone(),
                            variables: HashMap::new(),
                            output_reg: "r6".to_string(),
                            length_reg: "r7".to_string(),
                        },
                    );
                    slots.push(init_slot);
                    *slot_id += 1;
                }

                // Read until terminator
                let terminator = handler.terminator.clone().unwrap_or("\r\n".to_string());
                let read_slot = Slot::new(
                    format!("SLOT_{}_READ", cmd.name.to_uppercase()),
                    format!("{}_read", cmd.name.to_lowercase()),
                    SlotType::ReadUntil {
                        socket_reg: "r10".to_string(),
                        buffer_reg: "r4".to_string(),
                        delimiter: terminator,
                        max_len: handler.max_size.unwrap_or(10485760) as u32,
                        length_reg: "r5".to_string(),
                        eof_label: ".client_disconnect".to_string(),
                    },
                );
                slots.push(read_slot);
                *slot_id += 1;

                // On complete handler
                if let Some(ref on_complete) = handler.on_complete {
                    let complete_slots = self.expand_handler(cmd, on_complete, slot_id)?;
                    slots.extend(complete_slots);
                }
            }

            HandlerType::CloseConnection => {
                // Send goodbye response
                if let Some(ref response) = handler.response {
                    let bye_slot = Slot::new(
                        format!("SLOT_{}_HANDLER", cmd.name.to_uppercase()),
                        format!("{}_goodbye", cmd.name.to_lowercase()),
                        SlotType::ResponseBuilder {
                            template: response.clone(),
                            variables: HashMap::new(),
                            output_reg: "r6".to_string(),
                            length_reg: "r7".to_string(),
                        },
                    );
                    slots.push(bye_slot);
                    *slot_id += 1;

                    let send_slot = Slot::new(
                        format!("SLOT_{}_SEND", cmd.name.to_uppercase()),
                        format!("{}_send", cmd.name.to_lowercase()),
                        SlotType::SendResponse {
                            socket_reg: "r10".to_string(),
                            buffer_reg: "r6".to_string(),
                            length_reg: "r7".to_string(),
                        },
                    );
                    slots.push(send_slot);
                    *slot_id += 1;
                }
            }

            HandlerType::Custom => {
                // Custom handlers are passed through as-is
                // The LLM will need to fill these based on slot definitions
                let custom_slot = Slot::new(
                    format!("SLOT_{}_HANDLER", cmd.name.to_uppercase()),
                    format!("{}_custom", cmd.name.to_lowercase()),
                    SlotType::ExtensionCall {
                        extension: format!("handle {} command", cmd.name.to_lowercase()),
                        args: vec!["r0".to_string(), "r1".to_string()],
                        result_reg: "r0".to_string(),
                    },
                );
                slots.push(custom_slot);
                *slot_id += 1;
            }
        }

        Ok(slots)
    }

    /// Create the command dispatch slot
    fn create_dispatch_slot(
        &self,
        spec: &ProtocolSpec,
        slot_id: &mut usize,
    ) -> Result<Slot, TemplateError> {
        let cases: Vec<(String, String)> = spec
            .commands
            .iter()
            .map(|cmd| {
                // Extract command keyword from pattern
                let keyword = cmd
                    .pattern
                    .split_whitespace()
                    .next()
                    .unwrap_or(&cmd.name)
                    .to_string();
                (keyword, format!(".handle_{}", cmd.name.to_lowercase()))
            })
            .collect();

        let dispatch = Slot::new(
            "SLOT_DISPATCH".to_string(),
            "command_dispatch".to_string(),
            SlotType::PatternSwitch {
                input_reg: "r0".to_string(),
                cases,
                default_label: ".error_unknown".to_string(),
            },
        )
        .with_context(
            SlotContext::new()
                .register("r0", "input_buffer")
                .label(".error_unknown"),
        );

        *slot_id += 1;
        Ok(dispatch)
    }

    /// Convert a protocol test to a SlotSpec test case
    fn convert_test(&self, test: &super::parser::ProtocolTest) -> Result<TestCase, TemplateError> {
        let mut test_case = TestCase::new(&test.name);
        if let Some(ref desc) = test.description {
            test_case = test_case.with_description(desc);
        }

        for step in &test.steps {
            let slot_step = if let Some(ref send) = step.send {
                if let Some(ref expect) = step.expect {
                    TestStep::send_expect(send, expect)
                } else if let Some(ref contains) = step.expect_contains {
                    let mut s = TestStep::send(send);
                    s.expect_contains = Some(contains.clone());
                    s
                } else {
                    TestStep::send(send)
                }
            } else if let Some(ref expect) = step.expect {
                TestStep::expect(expect)
            } else if let Some(ref contains) = step.expect_contains {
                TestStep {
                    send: None,
                    expect: None,
                    expect_pattern: None,
                    expect_contains: Some(contains.clone()),
                    expect_not_contains: None,
                    timeout_ms: step.timeout_ms,
                    description: String::new(),
                }
            } else {
                continue;
            };

            test_case = test_case.step(slot_step.timeout(step.timeout_ms));
        }

        Ok(test_case)
    }
}

/// Create pattern match captures from command pattern
pub fn extract_captures(cmd: &ProtocolCommand) -> Vec<Capture> {
    let names = cmd.capture_names();
    names
        .iter()
        .enumerate()
        .map(|(i, name)| {
            let capture_type = match cmd.capture_type(name) {
                CaptureSpec::Word => CaptureType::Word,
                CaptureSpec::Until(c) => CaptureType::UntilChar { char: c },
                CaptureSpec::Quoted => CaptureType::Quoted,
                CaptureSpec::Rest => CaptureType::Rest,
                CaptureSpec::Int => CaptureType::Integer,
            };
            Capture {
                name: name.clone(),
                output_reg: format!("r{}", 3 + i),
                capture_type,
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slot::parser::parse_protocol_spec_str;
    use crate::slot::spec::DataType;

    fn sample_spec() -> ProtocolSpec {
        let json = r#"{
            "name": "test",
            "description": "Test protocol",
            "port": 1234,
            "greeting": {"format": "220 Welcome\r\n"},
            "states": [
                {"name": "INIT", "initial": true},
                {"name": "READY"},
                {"name": "QUIT", "terminal": true}
            ],
            "commands": [
                {
                    "name": "HELLO",
                    "pattern": "HELLO {name}",
                    "valid_states": ["INIT"],
                    "handler": {
                        "type": "simple_response",
                        "response": "200 Hello {name}\r\n",
                        "next_state": "READY"
                    }
                },
                {
                    "name": "QUIT",
                    "pattern": "QUIT",
                    "valid_states": ["ANY"],
                    "handler": {
                        "type": "close_connection",
                        "response": "221 Bye\r\n"
                    }
                }
            ],
            "errors": {
                "syntax": "500 Syntax error\r\n",
                "unknown": "502 Unknown command\r\n"
            },
            "tests": [
                {
                    "name": "basic",
                    "steps": [
                        {"expect": "220"},
                        {"send": "HELLO world\r\n", "expect": "200 Hello world\r\n"}
                    ]
                }
            ]
        }"#;
        parse_protocol_spec_str(json).unwrap()
    }

    #[test]
    fn test_expand_basic() {
        let spec = sample_spec();
        let expander = TemplateExpander::with_defaults();
        let slot_spec = expander.expand(&spec).unwrap();

        assert_eq!(slot_spec.name, "test_server");
        assert!(slot_spec.protocol.as_ref().unwrap() == "test");
        assert!(!slot_spec.slots.is_empty());
    }

    #[test]
    fn test_expand_generates_skeleton() {
        let spec = sample_spec();
        let expander = TemplateExpander::with_defaults();
        let slot_spec = expander.expand(&spec).unwrap();

        assert!(slot_spec.skeleton.contains(".entry:"));
        assert!(slot_spec.skeleton.contains(".main_loop:"));
        assert!(slot_spec.skeleton.contains(".handle_hello:"));
        assert!(slot_spec.skeleton.contains(".handle_quit:"));
    }

    #[test]
    fn test_expand_generates_data_items() {
        let spec = sample_spec();
        let expander = TemplateExpander::with_defaults();
        let slot_spec = expander.expand(&spec).unwrap();

        // Check state constants
        let state_names: Vec<&str> = slot_spec
            .data_items
            .iter()
            .filter(|d| d.name.starts_with("STATE_"))
            .map(|d| d.name.as_str())
            .collect();
        assert!(state_names.contains(&"STATE_INIT"));
        assert!(state_names.contains(&"STATE_READY"));

        // Check buffers
        let has_input_buffer = slot_spec
            .data_items
            .iter()
            .any(|d| d.name == "input_buffer" && d.data_type == DataType::Buffer);
        assert!(has_input_buffer);
    }

    #[test]
    fn test_expand_generates_tests() {
        let spec = sample_spec();
        let expander = TemplateExpander::with_defaults();
        let slot_spec = expander.expand(&spec).unwrap();

        assert_eq!(slot_spec.tests.len(), 1);
        assert_eq!(slot_spec.tests[0].name, "basic");
        assert_eq!(slot_spec.tests[0].steps.len(), 2);
    }

    #[test]
    fn test_extract_captures() {
        let cmd = ProtocolCommand {
            name: "TEST".to_string(),
            pattern: "CMD {arg1} {arg2:until:>}".to_string(),
            valid_states: vec![],
            handler: CommandHandler {
                handler_type: HandlerType::SimpleResponse,
                response: Some("OK".to_string()),
                lines: None,
                next_state: None,
                validation: None,
                response_ok: None,
                response_err: None,
                terminator: None,
                max_size: None,
                on_complete: None,
                slots: None,
            },
        };

        let captures = extract_captures(&cmd);
        assert_eq!(captures.len(), 2);
        assert_eq!(captures[0].name, "arg1");
        assert_eq!(captures[0].capture_type, CaptureType::Word);
        assert_eq!(captures[1].name, "arg2");
        assert!(matches!(
            captures[1].capture_type,
            CaptureType::UntilChar { char: '>' }
        ));
    }
}
