//! Protocol Spec Validator
//!
//! Validates protocol specifications for correctness before use.
//! Users run: `nl spec validate specs/protocols/smtp.json`

use std::collections::{HashMap, HashSet};
use std::path::Path;

use super::parser::{parse_protocol_spec, ProtocolCommand, ProtocolSpec, ProtocolState};

/// Validation result
#[derive(Debug)]
pub struct ValidationResult {
    /// Spec name
    pub spec_name: String,
    /// Whether validation passed
    pub valid: bool,
    /// Errors found (spec is invalid)
    pub errors: Vec<ValidationError>,
    /// Warnings (spec is valid but may have issues)
    pub warnings: Vec<ValidationWarning>,
    /// Statistics about the spec
    pub stats: SpecStats,
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        self.errors.is_empty()
    }

    pub fn summary(&self) -> String {
        let mut s = format!("Spec '{}': ", self.spec_name);
        if self.is_valid() {
            s.push_str("VALID");
        } else {
            s.push_str("INVALID");
        }
        s.push_str(&format!(
            " ({} states, {} commands, {} tests)",
            self.stats.state_count, self.stats.command_count, self.stats.test_count
        ));
        if !self.errors.is_empty() {
            s.push_str(&format!(", {} errors", self.errors.len()));
        }
        if !self.warnings.is_empty() {
            s.push_str(&format!(", {} warnings", self.warnings.len()));
        }
        s
    }
}

/// Validation error (spec is invalid)
#[derive(Debug, Clone)]
pub enum ValidationError {
    /// No initial state defined
    NoInitialState,
    /// Multiple initial states
    MultipleInitialStates(Vec<String>),
    /// State referenced but not defined
    UndefinedState {
        referenced_in: String,
        state: String,
    },
    /// Command references undefined state
    CommandInvalidState { command: String, state: String },
    /// Duplicate state name
    DuplicateState(String),
    /// Duplicate command name
    DuplicateCommand(String),
    /// Ambiguous patterns (multiple commands match same input)
    AmbiguousPatterns { pattern1: String, pattern2: String },
    /// Invalid pattern syntax
    InvalidPattern {
        command: String,
        pattern: String,
        error: String,
    },
    /// Missing required field
    MissingField { context: String, field: String },
    /// Invalid state transition (next_state doesn't exist)
    InvalidTransition { command: String, next_state: String },
}

impl std::fmt::Display for ValidationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationError::NoInitialState => {
                write!(
                    f,
                    "No initial state defined (add 'initial: true' to one state)"
                )
            }
            ValidationError::MultipleInitialStates(states) => {
                write!(f, "Multiple initial states: {:?}", states)
            }
            ValidationError::UndefinedState {
                referenced_in,
                state,
            } => {
                write!(
                    f,
                    "Undefined state '{}' referenced in '{}'",
                    state, referenced_in
                )
            }
            ValidationError::CommandInvalidState { command, state } => {
                write!(
                    f,
                    "Command '{}' references undefined state '{}'",
                    command, state
                )
            }
            ValidationError::DuplicateState(name) => {
                write!(f, "Duplicate state name: '{}'", name)
            }
            ValidationError::DuplicateCommand(name) => {
                write!(f, "Duplicate command name: '{}'", name)
            }
            ValidationError::AmbiguousPatterns { pattern1, pattern2 } => {
                write!(
                    f,
                    "Ambiguous patterns: '{}' and '{}' may match same input",
                    pattern1, pattern2
                )
            }
            ValidationError::InvalidPattern {
                command,
                pattern,
                error,
            } => {
                write!(
                    f,
                    "Invalid pattern '{}' in command '{}': {}",
                    pattern, command, error
                )
            }
            ValidationError::MissingField { context, field } => {
                write!(f, "Missing required field '{}' in {}", field, context)
            }
            ValidationError::InvalidTransition {
                command,
                next_state,
            } => {
                write!(
                    f,
                    "Command '{}' transitions to undefined state '{}'",
                    command, next_state
                )
            }
        }
    }
}

/// Validation warning (spec is valid but may have issues)
#[derive(Debug, Clone)]
pub enum ValidationWarning {
    /// State is unreachable
    UnreachableState(String),
    /// No terminal state
    NoTerminalState,
    /// Command has no test coverage
    UntestedCommand(String),
    /// State has no outgoing transitions
    DeadEndState(String),
    /// Pattern is very permissive
    PermissivePattern { command: String, pattern: String },
    /// Missing error handler
    MissingErrorHandler(String),
}

impl std::fmt::Display for ValidationWarning {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ValidationWarning::UnreachableState(state) => {
                write!(f, "State '{}' is unreachable", state)
            }
            ValidationWarning::NoTerminalState => {
                write!(f, "No terminal state defined")
            }
            ValidationWarning::UntestedCommand(cmd) => {
                write!(f, "Command '{}' has no test coverage", cmd)
            }
            ValidationWarning::DeadEndState(state) => {
                write!(f, "State '{}' has no outgoing transitions", state)
            }
            ValidationWarning::PermissivePattern { command, pattern } => {
                write!(
                    f,
                    "Pattern '{}' in command '{}' is very permissive",
                    pattern, command
                )
            }
            ValidationWarning::MissingErrorHandler(err_type) => {
                write!(f, "Missing error handler for '{}'", err_type)
            }
        }
    }
}

/// Statistics about the spec
#[derive(Debug, Default)]
pub struct SpecStats {
    pub state_count: usize,
    pub command_count: usize,
    pub test_count: usize,
    pub test_step_count: usize,
    pub terminal_state_count: usize,
    pub capture_count: usize,
}

/// Protocol spec validator
pub struct SpecValidator {
    errors: Vec<ValidationError>,
    warnings: Vec<ValidationWarning>,
    stats: SpecStats,
}

impl SpecValidator {
    pub fn new() -> Self {
        SpecValidator {
            errors: Vec::new(),
            warnings: Vec::new(),
            stats: SpecStats::default(),
        }
    }

    /// Validate a protocol spec file
    pub fn validate_file(&mut self, path: &Path) -> Result<ValidationResult, String> {
        let spec =
            parse_protocol_spec(path).map_err(|e| format!("Failed to parse spec: {:?}", e))?;

        Ok(self.validate(&spec))
    }

    /// Validate a protocol spec
    pub fn validate(&mut self, spec: &ProtocolSpec) -> ValidationResult {
        self.errors.clear();
        self.warnings.clear();
        self.stats = SpecStats::default();

        // Collect statistics
        self.stats.state_count = spec.states.len();
        self.stats.command_count = spec.commands.len();
        self.stats.test_count = spec.tests.len();
        self.stats.test_step_count = spec.tests.iter().map(|t| t.steps.len()).sum();

        // Build state name set
        let state_names: HashSet<&str> = spec.states.iter().map(|s| s.name.as_str()).collect();

        // Validate states
        self.validate_states(&spec.states, &state_names);

        // Validate commands
        self.validate_commands(&spec.commands, &state_names);

        // Check for ambiguous patterns
        self.check_ambiguous_patterns(&spec.commands);

        // Check reachability
        self.check_reachability(spec, &state_names);

        // Check test coverage
        self.check_test_coverage(spec);

        // Check error handlers
        self.check_error_handlers(spec);

        ValidationResult {
            spec_name: spec.name.clone(),
            valid: self.errors.is_empty(),
            errors: self.errors.clone(),
            warnings: self.warnings.clone(),
            stats: std::mem::take(&mut self.stats),
        }
    }

    fn validate_states(&mut self, states: &[ProtocolState], _state_names: &HashSet<&str>) {
        let mut seen_names = HashSet::new();
        let mut initial_states = Vec::new();
        let mut has_terminal = false;

        for state in states {
            // Check for duplicates
            if !seen_names.insert(&state.name) {
                self.errors
                    .push(ValidationError::DuplicateState(state.name.clone()));
            }

            // Track initial states
            if state.initial {
                initial_states.push(state.name.clone());
            }

            // Track terminal states
            if state.terminal {
                has_terminal = true;
                self.stats.terminal_state_count += 1;
            }
        }

        // Must have exactly one initial state
        if initial_states.is_empty() {
            self.errors.push(ValidationError::NoInitialState);
        } else if initial_states.len() > 1 {
            self.errors
                .push(ValidationError::MultipleInitialStates(initial_states));
        }

        // Warning if no terminal state
        if !has_terminal {
            self.warnings.push(ValidationWarning::NoTerminalState);
        }
    }

    fn validate_commands(&mut self, commands: &[ProtocolCommand], state_names: &HashSet<&str>) {
        let mut seen_names = HashSet::new();

        for cmd in commands {
            // Check for duplicate command names
            if !seen_names.insert(&cmd.name) {
                self.errors
                    .push(ValidationError::DuplicateCommand(cmd.name.clone()));
            }

            // Validate valid_states references
            for state in &cmd.valid_states {
                if state != "ANY" && !state_names.contains(state.as_str()) {
                    self.errors.push(ValidationError::CommandInvalidState {
                        command: cmd.name.clone(),
                        state: state.clone(),
                    });
                }
            }

            // Validate handler next_state
            if let Some(next_state) = &cmd.handler.next_state {
                if !state_names.contains(next_state.as_str()) {
                    self.errors.push(ValidationError::InvalidTransition {
                        command: cmd.name.clone(),
                        next_state: next_state.clone(),
                    });
                }
            }

            // Validate pattern syntax
            self.validate_pattern(&cmd.name, &cmd.pattern);

            // Count captures
            self.stats.capture_count += cmd.pattern.matches('{').count();
        }
    }

    fn validate_pattern(&mut self, command: &str, pattern: &str) {
        let mut in_capture = false;

        for c in pattern.chars() {
            match c {
                '{' => {
                    if in_capture {
                        self.errors.push(ValidationError::InvalidPattern {
                            command: command.to_string(),
                            pattern: pattern.to_string(),
                            error: "Nested braces not allowed".to_string(),
                        });
                        return;
                    }
                    in_capture = true;
                }
                '}' => {
                    if !in_capture {
                        self.errors.push(ValidationError::InvalidPattern {
                            command: command.to_string(),
                            pattern: pattern.to_string(),
                            error: "Unmatched closing brace".to_string(),
                        });
                        return;
                    }
                    in_capture = false;
                }
                _ => {}
            }
        }

        if in_capture {
            self.errors.push(ValidationError::InvalidPattern {
                command: command.to_string(),
                pattern: pattern.to_string(),
                error: "Unclosed capture".to_string(),
            });
        }
    }

    fn check_ambiguous_patterns(&mut self, commands: &[ProtocolCommand]) {
        // Check for patterns with same literal prefix
        let mut prefixes: HashMap<String, Vec<&str>> = HashMap::new();

        for cmd in commands {
            // Extract literal prefix (before first capture)
            let prefix: String = cmd.pattern.chars().take_while(|&c| c != '{').collect();

            prefixes.entry(prefix).or_default().push(&cmd.pattern);
        }

        for (prefix, patterns) in &prefixes {
            if patterns.len() > 1 && !prefix.is_empty() {
                // Same literal prefix - may be ambiguous
                // This is a heuristic - not all same-prefix patterns are ambiguous
                if patterns.len() == 2 {
                    self.warnings.push(ValidationWarning::PermissivePattern {
                        command: format!("{} commands", patterns.len()),
                        pattern: format!("Same prefix '{}': {:?}", prefix, patterns),
                    });
                }
            }
        }
    }

    fn check_reachability(&mut self, spec: &ProtocolSpec, state_names: &HashSet<&str>) {
        // Find reachable states via BFS from initial
        let mut reachable = HashSet::new();
        let mut queue: Vec<&str> = Vec::new();

        // Start from initial state
        for state in &spec.states {
            if state.initial {
                reachable.insert(state.name.as_str());
                queue.push(&state.name);
            }
        }

        // BFS through transitions
        while let Some(current) = queue.pop() {
            for cmd in &spec.commands {
                // Check if command is valid from current state
                let valid_from_current = cmd.valid_states.contains(&current.to_string())
                    || cmd.valid_states.contains(&"ANY".to_string());

                if valid_from_current {
                    if let Some(next) = &cmd.handler.next_state {
                        if !reachable.contains(next.as_str()) {
                            reachable.insert(next.as_str());
                            queue.push(next);
                        }
                    }
                }
            }
        }

        // Check for unreachable states
        for state_name in state_names {
            if !reachable.contains(state_name) {
                self.warnings
                    .push(ValidationWarning::UnreachableState(state_name.to_string()));
            }
        }

        // Check for dead-end states (no outgoing transitions, not terminal)
        for state in &spec.states {
            if state.terminal {
                continue; // Terminal states are expected to have no outgoing
            }

            let has_outgoing = spec.commands.iter().any(|cmd| {
                cmd.valid_states.contains(&state.name)
                    || cmd.valid_states.contains(&"ANY".to_string())
            });

            if !has_outgoing {
                self.warnings
                    .push(ValidationWarning::DeadEndState(state.name.clone()));
            }
        }
    }

    fn check_test_coverage(&mut self, spec: &ProtocolSpec) {
        // Build set of tested commands
        let mut tested_commands = HashSet::new();

        for test in &spec.tests {
            for step in &test.steps {
                if let Some(send) = &step.send {
                    // Extract command from send (first word before space)
                    let cmd_name: String = send
                        .chars()
                        .take_while(|&c| c != ' ' && c != '\r' && c != '\n')
                        .collect();
                    tested_commands.insert(cmd_name.to_uppercase());
                }
            }
        }

        // Check which commands lack tests
        for cmd in &spec.commands {
            if !tested_commands.contains(&cmd.name.to_uppercase()) {
                self.warnings
                    .push(ValidationWarning::UntestedCommand(cmd.name.clone()));
            }
        }
    }

    fn check_error_handlers(&mut self, spec: &ProtocolSpec) {
        // Check for common error handlers
        let expected_errors = ["syntax", "sequence", "unknown"];

        for err_type in &expected_errors {
            if !spec.errors.contains_key(*err_type) {
                self.warnings
                    .push(ValidationWarning::MissingErrorHandler(err_type.to_string()));
            }
        }
    }
}

impl Default for SpecValidator {
    fn default() -> Self {
        Self::new()
    }
}

/// Quick validation function
pub fn validate_spec(spec: &ProtocolSpec) -> ValidationResult {
    let mut validator = SpecValidator::new();
    validator.validate(spec)
}

/// Validate spec file
pub fn validate_spec_file(path: &Path) -> Result<ValidationResult, String> {
    let mut validator = SpecValidator::new();
    validator.validate_file(path)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slot::parser::{CommandHandler, HandlerType, ProtocolState, Transport};

    fn minimal_valid_spec() -> ProtocolSpec {
        ProtocolSpec {
            name: "test".to_string(),
            description: "Test protocol".to_string(),
            version: "1.0".to_string(),
            transport: Transport::Tcp,
            port: 1234,
            line_ending: Some("\r\n".to_string()),
            greeting: None,
            states: vec![
                ProtocolState {
                    name: "INIT".to_string(),
                    initial: true,
                    terminal: false,
                    description: None,
                },
                ProtocolState {
                    name: "DONE".to_string(),
                    initial: false,
                    terminal: true,
                    description: None,
                },
            ],
            commands: vec![ProtocolCommand {
                name: "QUIT".to_string(),
                pattern: "QUIT".to_string(),
                valid_states: vec!["INIT".to_string()],
                handler: CommandHandler {
                    handler_type: HandlerType::SimpleResponse,
                    response: Some("OK\r\n".to_string()),
                    next_state: Some("DONE".to_string()),
                    lines: None,
                    validation: None,
                    response_ok: None,
                    response_err: None,
                    terminator: None,
                    max_size: None,
                    on_complete: None,
                    slots: None,
                },
            }],
            errors: HashMap::new(),
            tests: vec![],
        }
    }

    #[test]
    fn test_valid_spec() {
        let spec = minimal_valid_spec();
        let result = validate_spec(&spec);
        assert!(result.is_valid(), "Errors: {:?}", result.errors);
    }

    #[test]
    fn test_no_initial_state() {
        let mut spec = minimal_valid_spec();
        spec.states[0].initial = false;

        let result = validate_spec(&spec);
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::NoInitialState)));
    }

    #[test]
    fn test_undefined_state() {
        let mut spec = minimal_valid_spec();
        spec.commands[0].valid_states = vec!["NONEXISTENT".to_string()];

        let result = validate_spec(&spec);
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::CommandInvalidState { .. })));
    }

    #[test]
    fn test_invalid_transition() {
        let mut spec = minimal_valid_spec();
        spec.commands[0].handler.next_state = Some("NOWHERE".to_string());

        let result = validate_spec(&spec);
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidTransition { .. })));
    }

    #[test]
    fn test_unreachable_state_warning() {
        let mut spec = minimal_valid_spec();
        spec.states.push(ProtocolState {
            name: "ORPHAN".to_string(),
            initial: false,
            terminal: false,
            description: None,
        });

        let result = validate_spec(&spec);
        assert!(result.is_valid()); // Unreachable is warning, not error
        assert!(result
            .warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::UnreachableState(s) if s == "ORPHAN")));
    }

    #[test]
    fn test_untested_command_warning() {
        let spec = minimal_valid_spec();
        let result = validate_spec(&spec);

        // No tests defined, so QUIT should be untested
        assert!(result
            .warnings
            .iter()
            .any(|w| matches!(w, ValidationWarning::UntestedCommand(c) if c == "QUIT")));
    }

    #[test]
    fn test_invalid_pattern() {
        let mut spec = minimal_valid_spec();
        spec.commands[0].pattern = "TEST {unclosed".to_string();

        let result = validate_spec(&spec);
        assert!(!result.is_valid());
        assert!(result
            .errors
            .iter()
            .any(|e| matches!(e, ValidationError::InvalidPattern { .. })));
    }
}
