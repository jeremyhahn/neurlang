//! Session Persistence for Neurlang Agent
//!
//! Manages session state including:
//! - Session metadata (id, timestamps)
//! - Conversation history
//! - Generated functions and IR
//! - Vector index for requirements
//!
//! Sessions are persisted to ~/.neurlang/sessions/{session_id}/

use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{BufReader, BufWriter};
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

use crate::ir::{Instruction, Program};

/// Session identifier (UUID-like)
pub type SessionId = String;

/// A conversation turn between user and agent
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConversationTurn {
    /// User request
    User(String),
    /// Agent response
    Agent(String),
    /// Error message
    Error(String),
    /// System message
    System(String),
}

/// Session metadata (serialized to session.json)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionMeta {
    /// Unique session ID
    pub id: SessionId,
    /// Session name/title
    pub name: String,
    /// Creation timestamp (Unix epoch seconds)
    pub created_at: u64,
    /// Last modified timestamp
    pub modified_at: u64,
    /// Hash of requirements files (for caching)
    pub requirements_hash: u64,
    /// Total iterations performed
    pub iteration_count: usize,
    /// Current status
    pub status: SessionStatus,
}

/// Session status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum SessionStatus {
    /// Session is active
    Active,
    /// Session completed successfully
    Completed,
    /// Session failed
    Failed,
    /// Session was interrupted
    Interrupted,
}

/// Full session state
#[derive(Debug, Clone)]
pub struct Session {
    /// Session metadata
    pub meta: SessionMeta,
    /// Conversation history
    pub history: Vec<ConversationTurn>,
    /// Named functions built so far
    pub functions: HashMap<String, Program>,
    /// Current working IR
    pub current_ir: Vec<Instruction>,
}

impl Session {
    /// Create a new session
    pub fn new(name: impl Into<String>) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        let id = generate_session_id();

        Self {
            meta: SessionMeta {
                id,
                name: name.into(),
                created_at: now,
                modified_at: now,
                requirements_hash: 0,
                iteration_count: 0,
                status: SessionStatus::Active,
            },
            history: Vec::new(),
            functions: HashMap::new(),
            current_ir: Vec::new(),
        }
    }

    /// Get session ID
    pub fn id(&self) -> &str {
        &self.meta.id
    }

    /// Get session name
    pub fn name(&self) -> &str {
        &self.meta.name
    }

    /// Get session directory path
    pub fn session_dir(&self) -> PathBuf {
        sessions_base_dir().join(&self.meta.id)
    }

    /// Add a user message to history
    pub fn add_user_message(&mut self, message: impl Into<String>) {
        self.history.push(ConversationTurn::User(message.into()));
        self.touch();
    }

    /// Add an agent response to history
    pub fn add_agent_response(&mut self, response: impl Into<String>) {
        self.history.push(ConversationTurn::Agent(response.into()));
        self.touch();
    }

    /// Add an error to history
    pub fn add_error(&mut self, error: impl Into<String>) {
        self.history.push(ConversationTurn::Error(error.into()));
        self.touch();
    }

    /// Increment iteration count
    pub fn increment_iteration(&mut self) {
        self.meta.iteration_count += 1;
        self.touch();
    }

    /// Update modified timestamp
    fn touch(&mut self) {
        self.meta.modified_at = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
    }

    /// Set session status
    pub fn set_status(&mut self, status: SessionStatus) {
        self.meta.status = status;
        self.touch();
    }

    /// Update current IR
    pub fn set_current_ir(&mut self, ir: Vec<Instruction>) {
        self.current_ir = ir;
        self.touch();
    }

    /// Add a named function
    pub fn add_function(&mut self, name: impl Into<String>, program: Program) {
        self.functions.insert(name.into(), program);
        self.touch();
    }

    /// Get recent conversation context (last N turns)
    pub fn recent_context(&self, n: usize) -> Vec<&ConversationTurn> {
        self.history.iter().rev().take(n).rev().collect()
    }

    /// Get full conversation history as formatted string
    pub fn format_history(&self) -> String {
        let mut output = String::new();
        for turn in &self.history {
            match turn {
                ConversationTurn::User(msg) => {
                    output.push_str("User: ");
                    output.push_str(msg);
                }
                ConversationTurn::Agent(msg) => {
                    output.push_str("Agent: ");
                    output.push_str(msg);
                }
                ConversationTurn::Error(msg) => {
                    output.push_str("Error: ");
                    output.push_str(msg);
                }
                ConversationTurn::System(msg) => {
                    output.push_str("System: ");
                    output.push_str(msg);
                }
            }
            output.push('\n');
        }
        output
    }

    /// Save session to disk
    pub fn save(&self) -> std::io::Result<()> {
        let dir = self.session_dir();
        fs::create_dir_all(&dir)?;

        // Save metadata
        let meta_path = dir.join("session.json");
        let file = File::create(&meta_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer_pretty(writer, &self.meta)?;

        // Save history
        let history_path = dir.join("history.json");
        let file = File::create(&history_path)?;
        let writer = BufWriter::new(file);
        serde_json::to_writer(writer, &self.history)?;

        // Save current IR as binary
        if !self.current_ir.is_empty() {
            let program = Program::from_instructions(self.current_ir.clone());
            let ir_path = dir.join("current.nlb");
            fs::write(&ir_path, program.encode())?;
        }

        // Save functions
        let functions_dir = dir.join("functions");
        if !self.functions.is_empty() {
            fs::create_dir_all(&functions_dir)?;
            for (name, program) in &self.functions {
                let func_path = functions_dir.join(format!("{}.nlb", name));
                fs::write(&func_path, program.encode())?;
            }
        }

        Ok(())
    }

    /// Load session from disk
    pub fn load(id: &str) -> std::io::Result<Self> {
        let dir = sessions_base_dir().join(id);

        if !dir.exists() {
            return Err(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!("Session not found: {}", id),
            ));
        }

        // Load metadata
        let meta_path = dir.join("session.json");
        let file = File::open(&meta_path)?;
        let reader = BufReader::new(file);
        let meta: SessionMeta = serde_json::from_reader(reader)?;

        // Load history
        let history_path = dir.join("history.json");
        let history = if history_path.exists() {
            let file = File::open(&history_path)?;
            let reader = BufReader::new(file);
            serde_json::from_reader(reader)?
        } else {
            Vec::new()
        };

        // Load current IR
        let ir_path = dir.join("current.nlb");
        let current_ir = if ir_path.exists() {
            let data = fs::read(&ir_path)?;
            Program::decode(&data)
                .map(|p| p.instructions)
                .unwrap_or_default()
        } else {
            Vec::new()
        };

        // Load functions
        let mut functions = HashMap::new();
        let functions_dir = dir.join("functions");
        if functions_dir.exists() {
            for entry in fs::read_dir(&functions_dir)? {
                let entry = entry?;
                let path = entry.path();
                if path.extension().map(|e| e == "nlb").unwrap_or(false) {
                    if let Some(name) = path.file_stem().and_then(|n| n.to_str()) {
                        let data = fs::read(&path)?;
                        if let Some(program) = Program::decode(&data) {
                            functions.insert(name.to_string(), program);
                        }
                    }
                }
            }
        }

        Ok(Self {
            meta,
            history,
            functions,
            current_ir,
        })
    }

    /// Delete session from disk
    pub fn delete(&self) -> std::io::Result<()> {
        let dir = self.session_dir();
        if dir.exists() {
            fs::remove_dir_all(&dir)?;
        }
        Ok(())
    }
}

/// List all sessions
pub fn list_sessions() -> std::io::Result<Vec<SessionMeta>> {
    let base = sessions_base_dir();

    if !base.exists() {
        return Ok(Vec::new());
    }

    let mut sessions = Vec::new();

    for entry in fs::read_dir(&base)? {
        let entry = entry?;
        let path = entry.path();

        if path.is_dir() {
            let meta_path = path.join("session.json");
            if meta_path.exists() {
                if let Ok(file) = File::open(&meta_path) {
                    let reader = BufReader::new(file);
                    if let Ok(meta) = serde_json::from_reader::<_, SessionMeta>(reader) {
                        sessions.push(meta);
                    }
                }
            }
        }
    }

    // Sort by modified time, most recent first
    sessions.sort_by(|a, b| b.modified_at.cmp(&a.modified_at));

    Ok(sessions)
}

/// Get the base directory for sessions
pub fn sessions_base_dir() -> PathBuf {
    // Get home directory from environment or use current directory as fallback
    let home = std::env::var("HOME")
        .or_else(|_| std::env::var("USERPROFILE"))
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("."));

    home.join(".neurlang").join("sessions")
}

/// Generate a unique session ID
fn generate_session_id() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};

    let now = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default();

    let timestamp = now.as_nanos();
    let random: u32 = rand::random();

    format!("{:x}{:08x}", timestamp as u64, random)
}

/// Find a session by partial ID match
pub fn find_session(partial_id: &str) -> std::io::Result<Option<SessionMeta>> {
    let sessions = list_sessions()?;

    // Exact match first
    if let Some(session) = sessions.iter().find(|s| s.id == partial_id) {
        return Ok(Some(session.clone()));
    }

    // Prefix match
    let matches: Vec<_> = sessions
        .iter()
        .filter(|s| s.id.starts_with(partial_id))
        .collect();

    match matches.len() {
        0 => Ok(None),
        1 => Ok(Some(matches[0].clone())),
        _ => Err(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!(
                "Ambiguous session ID '{}' matches {} sessions",
                partial_id,
                matches.len()
            ),
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_creation() {
        let session = Session::new("test session");
        assert!(!session.id().is_empty());
        assert_eq!(session.name(), "test session");
        assert_eq!(session.meta.status, SessionStatus::Active);
        assert_eq!(session.meta.iteration_count, 0);
    }

    #[test]
    fn test_conversation_history() {
        let mut session = Session::new("test");
        session.add_user_message("Hello");
        session.add_agent_response("Hi there");
        session.add_error("Something went wrong");

        assert_eq!(session.history.len(), 3);

        let formatted = session.format_history();
        assert!(formatted.contains("User: Hello"));
        assert!(formatted.contains("Agent: Hi there"));
        assert!(formatted.contains("Error: Something went wrong"));
    }

    #[test]
    fn test_recent_context() {
        let mut session = Session::new("test");
        session.add_user_message("One");
        session.add_agent_response("Two");
        session.add_user_message("Three");
        session.add_agent_response("Four");
        session.add_user_message("Five");

        let recent = session.recent_context(3);
        assert_eq!(recent.len(), 3);
    }

    #[test]
    fn test_session_id_generation() {
        let id1 = generate_session_id();
        let id2 = generate_session_id();
        assert_ne!(id1, id2);
        assert!(id1.len() >= 16);
    }
}
