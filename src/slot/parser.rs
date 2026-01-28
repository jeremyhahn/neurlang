//! Protocol Specification Parser
//!
//! Parses YAML protocol specification files into structured data
//! that can be used to generate SlotSpecs.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// A parsed protocol specification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolSpec {
    /// Protocol name (e.g., "smtp", "http")
    pub name: String,

    /// Human-readable description
    pub description: String,

    /// Spec version
    #[serde(default = "default_version")]
    pub version: String,

    /// Transport layer
    #[serde(default = "default_transport")]
    pub transport: Transport,

    /// Default port
    pub port: u16,

    /// Line ending (e.g., "\r\n")
    #[serde(default)]
    pub line_ending: Option<String>,

    /// Greeting sent on connect
    #[serde(default)]
    pub greeting: Option<Greeting>,

    /// State machine definition
    pub states: Vec<ProtocolState>,

    /// Command definitions
    pub commands: Vec<ProtocolCommand>,

    /// Error response definitions
    #[serde(default)]
    pub errors: HashMap<String, String>,

    /// Test cases
    #[serde(default)]
    pub tests: Vec<ProtocolTest>,
}

fn default_version() -> String {
    "1.0".to_string()
}

fn default_transport() -> Transport {
    Transport::Tcp
}

impl ProtocolSpec {
    /// Get the initial state
    pub fn initial_state(&self) -> Option<&ProtocolState> {
        self.states.iter().find(|s| s.initial)
    }

    /// Get all terminal states
    pub fn terminal_states(&self) -> Vec<&ProtocolState> {
        self.states.iter().filter(|s| s.terminal).collect()
    }

    /// Find a state by name
    pub fn get_state(&self, name: &str) -> Option<&ProtocolState> {
        self.states.iter().find(|s| s.name == name)
    }

    /// Find a command by name
    pub fn get_command(&self, name: &str) -> Option<&ProtocolCommand> {
        self.commands.iter().find(|c| c.name == name)
    }

    /// Get all commands valid in a given state
    pub fn commands_for_state(&self, state: &str) -> Vec<&ProtocolCommand> {
        self.commands
            .iter()
            .filter(|c| {
                c.valid_states.contains(&state.to_string())
                    || c.valid_states.contains(&"ANY".to_string())
            })
            .collect()
    }
}

/// Transport layer protocol
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Transport {
    /// TCP connection
    Tcp,
    /// UDP datagram
    Udp,
    /// Unix domain socket
    Unix,
}

/// Initial greeting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Greeting {
    /// Response format with {variable} placeholders
    pub format: String,
}

/// A protocol state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolState {
    /// State name (uppercase by convention)
    pub name: String,

    /// Whether this is the initial state
    #[serde(default)]
    pub initial: bool,

    /// Whether this is a terminal state (connection closes)
    #[serde(default)]
    pub terminal: bool,

    /// Description
    #[serde(default)]
    pub description: Option<String>,
}

/// A protocol command definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolCommand {
    /// Command name
    pub name: String,

    /// Pattern to match (with {capture} placeholders)
    pub pattern: String,

    /// States in which this command is valid
    pub valid_states: Vec<String>,

    /// How to handle this command
    pub handler: CommandHandler,
}

impl ProtocolCommand {
    /// Extract capture names from the pattern
    pub fn capture_names(&self) -> Vec<String> {
        let mut captures = Vec::new();
        let mut in_capture = false;
        let mut current = String::new();

        for c in self.pattern.chars() {
            if c == '{' {
                in_capture = true;
                current.clear();
            } else if c == '}' {
                if in_capture {
                    // Strip type annotation if present (e.g., "domain:word" -> "domain")
                    let name = current.split(':').next().unwrap_or(&current);
                    captures.push(name.to_string());
                }
                in_capture = false;
            } else if in_capture {
                current.push(c);
            }
        }

        captures
    }

    /// Parse capture type from pattern
    pub fn capture_type(&self, name: &str) -> CaptureSpec {
        // Find the capture in the pattern
        let search = format!("{{{}", name);
        if let Some(start) = self.pattern.find(&search) {
            let rest = &self.pattern[start + search.len()..];
            if let Some(end) = rest.find('}') {
                let spec = &rest[..end];
                if spec.is_empty() || spec == "}" {
                    return CaptureSpec::Word;
                }
                if spec.starts_with(':') {
                    return CaptureSpec::parse(&spec[1..]);
                }
            }
        }
        CaptureSpec::Word
    }
}

/// Capture type specification from pattern
#[derive(Debug, Clone, PartialEq)]
pub enum CaptureSpec {
    /// Capture until whitespace (default)
    Word,
    /// Capture until specific character
    Until(char),
    /// Capture quoted string
    Quoted,
    /// Capture rest of line
    Rest,
    /// Parse as integer
    Int,
}

impl CaptureSpec {
    fn parse(s: &str) -> Self {
        let s = s.trim();
        if s == "word" || s.is_empty() {
            CaptureSpec::Word
        } else if s == "quoted" {
            CaptureSpec::Quoted
        } else if s == "rest" {
            CaptureSpec::Rest
        } else if s == "int" {
            CaptureSpec::Int
        } else if s.starts_with("until:") {
            let ch = s[6..].chars().next().unwrap_or('>');
            CaptureSpec::Until(ch)
        } else {
            CaptureSpec::Word
        }
    }
}

/// Command handler configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CommandHandler {
    /// Handler type
    #[serde(rename = "type")]
    pub handler_type: HandlerType,

    /// Response template (for simple_response)
    #[serde(default)]
    pub response: Option<String>,

    /// Response lines (for multi_line_response)
    #[serde(default)]
    pub lines: Option<Vec<String>>,

    /// Next state after handling
    #[serde(default)]
    pub next_state: Option<String>,

    /// Validation configuration
    #[serde(default)]
    pub validation: Option<ValidationConfig>,

    /// Response on validation success
    #[serde(default)]
    pub response_ok: Option<String>,

    /// Response on validation failure
    #[serde(default)]
    pub response_err: Option<String>,

    /// Terminator for multiline reader
    #[serde(default)]
    pub terminator: Option<String>,

    /// Max size for multiline reader
    #[serde(default)]
    pub max_size: Option<u64>,

    /// Handler after multiline complete
    #[serde(default)]
    pub on_complete: Option<Box<CommandHandler>>,

    /// Custom slots (for custom handler type)
    #[serde(default)]
    pub slots: Option<Vec<serde_json::Value>>,
}

/// Handler type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum HandlerType {
    /// Send a simple response
    SimpleResponse,
    /// Send multiple response lines
    MultiLineResponse,
    /// Validate before responding
    ValidatedResponse,
    /// Read multiple lines until terminator
    MultilineReader,
    /// Close the connection
    CloseConnection,
    /// Custom slot sequence
    Custom,
}

/// Validation configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConfig {
    /// Validation type
    #[serde(rename = "type")]
    pub validation_type: ValidationType,

    /// SQL query (for db_lookup)
    #[serde(default)]
    pub query: Option<String>,

    /// Parameter to validate
    #[serde(default)]
    pub param: Option<String>,

    /// Regex pattern (for regex validation)
    #[serde(default)]
    pub pattern: Option<String>,

    /// Minimum value (for range validation)
    #[serde(default)]
    pub min: Option<i64>,

    /// Maximum value (for range validation)
    #[serde(default)]
    pub max: Option<i64>,

    /// Extension name (for extension validation)
    #[serde(default)]
    pub extension: Option<String>,
}

/// Validation type enumeration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ValidationType {
    /// Database lookup
    DbLookup,
    /// Regular expression match
    Regex,
    /// Numeric range check
    Range,
    /// Custom extension
    Extension,
}

/// Protocol test case
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolTest {
    /// Test name
    pub name: String,

    /// Description
    #[serde(default)]
    pub description: Option<String>,

    /// Setup commands
    #[serde(default)]
    pub setup: Option<TestSetup>,

    /// Test steps
    pub steps: Vec<ProtocolTestStep>,
}

/// Test setup configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestSetup {
    /// SQL to execute
    #[serde(default)]
    pub sql: Option<String>,

    /// Initial state
    #[serde(default)]
    pub state: Option<String>,
}

/// Protocol test step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolTestStep {
    /// Data to send
    #[serde(default)]
    pub send: Option<String>,

    /// Expected response (exact or prefix)
    #[serde(default)]
    pub expect: Option<String>,

    /// Expected response pattern (regex)
    #[serde(default)]
    pub expect_pattern: Option<String>,

    /// Response must contain
    #[serde(default)]
    pub expect_contains: Option<String>,

    /// Response must not contain
    #[serde(default)]
    pub expect_not_contains: Option<String>,

    /// Timeout in milliseconds
    #[serde(default = "default_test_timeout")]
    pub timeout_ms: u32,
}

fn default_test_timeout() -> u32 {
    5000
}

/// Protocol specification error
#[derive(Debug, Clone)]
pub struct ProtocolError {
    /// Error message
    pub message: String,
    /// Source location (if known)
    pub location: Option<String>,
}

impl std::fmt::Display for ProtocolError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(ref loc) = self.location {
            write!(f, "{}: {}", loc, self.message)
        } else {
            write!(f, "{}", self.message)
        }
    }
}

impl std::error::Error for ProtocolError {}

/// Parse error wrapper
#[derive(Debug)]
pub enum ParseError {
    /// IO error reading file
    Io(std::io::Error),
    /// YAML/JSON parse error
    Format(String),
    /// Validation error
    Validation(ProtocolError),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ParseError::Io(e) => write!(f, "IO error: {}", e),
            ParseError::Format(e) => write!(f, "Parse error: {}", e),
            ParseError::Validation(e) => write!(f, "Validation error: {}", e),
        }
    }
}

impl std::error::Error for ParseError {}

impl From<std::io::Error> for ParseError {
    fn from(e: std::io::Error) -> Self {
        ParseError::Io(e)
    }
}

impl From<serde_json::Error> for ParseError {
    fn from(e: serde_json::Error) -> Self {
        ParseError::Format(e.to_string())
    }
}

/// Parse a protocol specification from a YAML file
pub fn parse_protocol_spec<P: AsRef<Path>>(path: P) -> Result<ProtocolSpec, ParseError> {
    let content = fs::read_to_string(path.as_ref())?;
    parse_protocol_spec_str(&content)
}

/// Parse a protocol specification from a YAML string
pub fn parse_protocol_spec_str(content: &str) -> Result<ProtocolSpec, ParseError> {
    // Try YAML first (using serde_json after converting)
    // Since we don't have serde_yaml in deps, we'll use a simple YAML-like parser
    // or expect JSON format for now

    // For now, support JSON format (YAML support can be added with serde_yaml)
    let spec: ProtocolSpec = if content.trim().starts_with('{') {
        serde_json::from_str(content)?
    } else {
        // Simple YAML-like parsing (basic subset)
        parse_yaml_subset(content)?
    };

    validate_protocol_spec(&spec)?;
    Ok(spec)
}

/// Simple YAML subset parser (for basic protocol specs)
fn parse_yaml_subset(_content: &str) -> Result<ProtocolSpec, ParseError> {
    // This is a simplified parser that handles the basic YAML structure
    // For full YAML support, add serde_yaml to Cargo.toml

    // For now, return an error suggesting JSON format
    // In production, we'd add serde_yaml dependency
    Err(ParseError::Format(
        "YAML parsing requires serde_yaml. Please use JSON format or add serde_yaml to dependencies.".to_string()
    ))
}

/// Validate a parsed protocol specification
fn validate_protocol_spec(spec: &ProtocolSpec) -> Result<(), ParseError> {
    // Check for initial state
    if spec.initial_state().is_none() {
        return Err(ParseError::Validation(ProtocolError {
            message: "No initial state defined (set initial: true on one state)".to_string(),
            location: Some("states".to_string()),
        }));
    }

    // Check that command valid_states reference existing states
    for cmd in &spec.commands {
        for state in &cmd.valid_states {
            if state != "ANY" && spec.get_state(state).is_none() {
                return Err(ParseError::Validation(ProtocolError {
                    message: format!(
                        "Command '{}' references unknown state '{}'",
                        cmd.name, state
                    ),
                    location: Some(format!("commands.{}.valid_states", cmd.name)),
                }));
            }
        }

        // Check that next_state references existing state
        if let Some(handler) = Some(&cmd.handler) {
            if let Some(ref next) = handler.next_state {
                if next != "SAME" && spec.get_state(next).is_none() {
                    return Err(ParseError::Validation(ProtocolError {
                        message: format!(
                            "Command '{}' transitions to unknown state '{}'",
                            cmd.name, next
                        ),
                        location: Some(format!("commands.{}.handler.next_state", cmd.name)),
                    }));
                }
            }
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_spec_json() -> &'static str {
        r#"{
            "name": "test",
            "description": "Test protocol",
            "port": 1234,
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
            "tests": []
        }"#
    }

    #[test]
    fn test_parse_json_spec() {
        let spec = parse_protocol_spec_str(sample_spec_json()).unwrap();
        assert_eq!(spec.name, "test");
        assert_eq!(spec.port, 1234);
        assert_eq!(spec.states.len(), 3);
        assert_eq!(spec.commands.len(), 2);
    }

    #[test]
    fn test_initial_state() {
        let spec = parse_protocol_spec_str(sample_spec_json()).unwrap();
        let initial = spec.initial_state().unwrap();
        assert_eq!(initial.name, "INIT");
        assert!(initial.initial);
    }

    #[test]
    fn test_terminal_states() {
        let spec = parse_protocol_spec_str(sample_spec_json()).unwrap();
        let terminals = spec.terminal_states();
        assert_eq!(terminals.len(), 1);
        assert_eq!(terminals[0].name, "QUIT");
    }

    #[test]
    fn test_capture_names() {
        let cmd = ProtocolCommand {
            name: "TEST".to_string(),
            pattern: "MAIL FROM:<{sender:until:>}> SIZE={size:int}".to_string(),
            valid_states: vec!["INIT".to_string()],
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

        let names = cmd.capture_names();
        assert_eq!(names, vec!["sender", "size"]);
    }

    #[test]
    fn test_capture_types() {
        let cmd = ProtocolCommand {
            name: "TEST".to_string(),
            pattern: "CMD {word} {quoted:quoted} {email:until:>} {rest:rest}".to_string(),
            valid_states: vec![],
            handler: CommandHandler {
                handler_type: HandlerType::SimpleResponse,
                response: None,
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

        assert_eq!(cmd.capture_type("word"), CaptureSpec::Word);
        assert_eq!(cmd.capture_type("quoted"), CaptureSpec::Quoted);
        assert_eq!(cmd.capture_type("email"), CaptureSpec::Until('>'));
        assert_eq!(cmd.capture_type("rest"), CaptureSpec::Rest);
    }

    #[test]
    fn test_commands_for_state() {
        let spec = parse_protocol_spec_str(sample_spec_json()).unwrap();

        let init_cmds = spec.commands_for_state("INIT");
        assert_eq!(init_cmds.len(), 2); // HELLO + QUIT (ANY)

        let ready_cmds = spec.commands_for_state("READY");
        assert_eq!(ready_cmds.len(), 1); // Just QUIT (ANY)
    }

    #[test]
    fn test_missing_initial_state() {
        let json = r#"{
            "name": "test",
            "description": "Test",
            "port": 1234,
            "states": [{"name": "READY"}],
            "commands": []
        }"#;

        let result = parse_protocol_spec_str(json);
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_state_reference() {
        let json = r#"{
            "name": "test",
            "description": "Test",
            "port": 1234,
            "states": [{"name": "INIT", "initial": true}],
            "commands": [{
                "name": "CMD",
                "pattern": "CMD",
                "valid_states": ["NONEXISTENT"],
                "handler": {"type": "simple_response", "response": "OK"}
            }]
        }"#;

        let result = parse_protocol_spec_str(json);
        assert!(result.is_err());
    }
}
