//! Slot Filler
//!
//! Generates code to fill slots in a SlotSpec. Supports multiple backends:
//! - Mock backend for testing
//! - ONNX model backend for production
//! - Template-based backend for common patterns
//!
//! Key feature: Parallel batch generation for all slots simultaneously.

use std::collections::HashMap;
use std::time::Instant;

use super::spec::{Slot, SlotSpec};
use super::types::SlotType;

/// Result of filling a single slot
#[derive(Debug, Clone)]
pub struct FilledSlot {
    /// Slot ID (e.g., "SLOT_HELO_HANDLER")
    pub id: String,
    /// Generated assembly code
    pub code: String,
    /// Generation time in milliseconds
    pub generation_time_ms: f64,
    /// Whether this came from cache
    pub from_cache: bool,
    /// Confidence score (0.0-1.0)
    pub confidence: f32,
}

/// Result of filling all slots in a spec
#[derive(Debug)]
pub struct FillResult {
    /// All filled slots
    pub slots: Vec<FilledSlot>,
    /// Total generation time
    pub total_time_ms: f64,
    /// Number of cache hits
    pub cache_hits: usize,
    /// Number of cache misses (generated)
    pub cache_misses: usize,
}

/// Error during slot filling
#[derive(Debug)]
pub enum FillError {
    /// Model inference failed
    InferenceFailed(String),
    /// Invalid slot type
    InvalidSlotType(String),
    /// Generation timeout
    Timeout,
    /// Backend not available
    BackendUnavailable(String),
}

impl std::fmt::Display for FillError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FillError::InferenceFailed(msg) => write!(f, "Inference failed: {}", msg),
            FillError::InvalidSlotType(t) => write!(f, "Invalid slot type: {}", t),
            FillError::Timeout => write!(f, "Generation timeout"),
            FillError::BackendUnavailable(b) => write!(f, "Backend unavailable: {}", b),
        }
    }
}

impl std::error::Error for FillError {}

/// Trait for slot filling backends
pub trait SlotFillerBackend: Send + Sync {
    /// Fill a single slot
    fn fill_slot(&self, slot: &Slot) -> Result<String, FillError>;

    /// Fill multiple slots in parallel (batch)
    /// Default implementation calls fill_slot sequentially
    fn fill_batch(&self, slots: &[&Slot]) -> Result<Vec<String>, FillError> {
        slots.iter().map(|s| self.fill_slot(s)).collect()
    }

    /// Get backend name
    fn name(&self) -> &str;

    /// Check if backend is available
    fn is_available(&self) -> bool {
        true
    }
}

/// Mock backend for testing - generates placeholder code
pub struct MockBackend {
    /// Delay per slot in milliseconds (simulates inference time)
    delay_ms: u64,
    /// Whether to generate realistic-looking code
    realistic: bool,
}

impl MockBackend {
    /// Create a new mock backend
    pub fn new() -> Self {
        MockBackend {
            delay_ms: 0,
            realistic: false,
        }
    }

    /// Create with simulated delay
    pub fn with_delay(delay_ms: u64) -> Self {
        MockBackend {
            delay_ms,
            realistic: false,
        }
    }

    /// Create with realistic code generation
    pub fn realistic() -> Self {
        MockBackend {
            delay_ms: 0,
            realistic: true,
        }
    }

    /// Generate code based on slot type
    fn generate_for_type(&self, slot: &Slot) -> String {
        if !self.realistic {
            return format!(
                "; Placeholder for {}\n; Type: {:?}\nnop\n",
                slot.id,
                slot.slot_type.category()
            );
        }

        // Generate more realistic code based on slot type
        match &slot.slot_type {
            SlotType::PatternMatch {
                pattern,
                input_reg,
                captures: _,
                match_label,
                no_match_label,
            } => {
                let mut code = format!("; PatternMatch: {}\n", pattern);
                code.push_str(&format!("    ; Input: {}\n", input_reg));
                code.push_str(&format!("    mov r1, {}\n", input_reg));
                code.push_str("    ; Compare pattern bytes\n");
                for (i, c) in pattern.chars().take(4).enumerate() {
                    if c != '{' {
                        code.push_str(&format!("    load.b r2, [r1 + {}]\n", i));
                        code.push_str(&format!("    mov r3, {}\n", c as u8));
                        code.push_str(&format!("    bne r2, r3, {}\n", no_match_label));
                    }
                }
                code.push_str(&format!("    b {}\n", match_label));
                code
            }
            SlotType::ResponseBuilder {
                template,
                variables: _,
                output_reg,
                length_reg,
            } => {
                let mut code = format!("; ResponseBuilder: {}\n", template);
                code.push_str(&format!("    lea r0, {}\n", output_reg));
                code.push_str("    mov r1, 0  ; offset\n");
                for c in template.chars().take(20) {
                    if c != '{' && c != '}' {
                        code.push_str(&format!(
                            "    mov r2, {}  ; '{}'\n",
                            c as u8,
                            c.escape_default()
                        ));
                        code.push_str("    store.b r2, [r0 + r1]\n");
                        code.push_str("    addi r1, r1, 1\n");
                    }
                }
                code.push_str(&format!("    mov {}, r1\n", length_reg));
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
                format!(
                    "; StateTransition: {} = {}\n    mov {}, {}\n",
                    state_reg, new_state, state_reg, new_state
                )
            }
            SlotType::SendResponse {
                socket_reg,
                buffer_reg,
                length_reg,
            } => {
                format!(
                    "; SendResponse\n    mov r0, {}\n    mov r1, {}\n    mov r2, {}\n    io.send r0, r0, r1, r2\n",
                    socket_reg, buffer_reg, length_reg
                )
            }
            SlotType::ReadUntil {
                socket_reg,
                buffer_reg,
                delimiter,
                max_len,
                length_reg,
                eof_label,
            } => {
                format!(
                    "; ReadUntil '{}'\n    mov r0, {}\n    lea r1, {}\n    mov r2, {}\n    io.recv {}, r0, r1, r2\n    beqz {}, {}\n",
                    delimiter.escape_default(), socket_reg, buffer_reg, max_len, length_reg, length_reg, eof_label
                )
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
                let mut code = format!("; ErrorResponse: {} {}\n", error_code, error_message);
                code.push_str(&format!("    lea r0, err_{}\n", error_code));
                code.push_str(&format!("    load r1, [err_{}_len]\n", error_code));
                code.push_str(&format!("    mov r2, {}\n", socket_reg));
                code.push_str("    io.send r2, r2, r0, r1\n");
                if *close_after {
                    code.push_str(&format!("    io.close r2, {}\n", socket_reg));
                }
                code
            }
            _ => {
                // Default placeholder for other types
                format!(
                    "; {} ({:?})\n    nop  ; TODO: implement\n",
                    slot.id,
                    slot.slot_type.category()
                )
            }
        }
    }
}

impl Default for MockBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SlotFillerBackend for MockBackend {
    fn fill_slot(&self, slot: &Slot) -> Result<String, FillError> {
        if self.delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.delay_ms));
        }
        Ok(self.generate_for_type(slot))
    }

    fn fill_batch(&self, slots: &[&Slot]) -> Result<Vec<String>, FillError> {
        // Simulate batch processing - total delay is same as single slot
        if self.delay_ms > 0 {
            std::thread::sleep(std::time::Duration::from_millis(self.delay_ms));
        }
        slots
            .iter()
            .map(|s| self.generate_for_type(s))
            .map(Ok)
            .collect()
    }

    fn name(&self) -> &str {
        "mock"
    }
}

/// Template-based backend for common patterns
/// Uses pre-written templates for high accuracy on known slot types
pub struct TemplateBackend {
    /// Templates keyed by (slot_type_name, pattern_hash)
    templates: HashMap<String, String>,
}

impl TemplateBackend {
    /// Create a new template backend with built-in templates
    pub fn new() -> Self {
        let mut backend = TemplateBackend {
            templates: HashMap::new(),
        };
        backend.load_builtin_templates();
        backend
    }

    fn load_builtin_templates(&mut self) {
        // Common state check pattern
        self.templates.insert(
            "state_check_2".to_string(),
            r#"; StateCheck for 2 valid states
    mov r1, {{STATE_1}}
    beq {{STATE_REG}}, r1, {{OK_LABEL}}
    mov r1, {{STATE_2}}
    beq {{STATE_REG}}, r1, {{OK_LABEL}}
    b {{ERROR_LABEL}}
"#
            .to_string(),
        );

        // Simple response send
        self.templates.insert(
            "send_response".to_string(),
            r#"; Send response buffer
    mov r0, {{SOCKET_REG}}
    lea r1, {{BUFFER_REG}}
    load r2, [{{LENGTH_LABEL}}]
    io.send r0, r0, r1, r2
"#
            .to_string(),
        );

        // State transition
        self.templates.insert(
            "state_transition".to_string(),
            r#"; State transition
    mov {{STATE_REG}}, {{NEW_STATE}}
"#
            .to_string(),
        );
    }

    fn lookup_template(&self, slot: &Slot) -> Option<String> {
        match &slot.slot_type {
            SlotType::StateCheck {
                state_reg,
                valid_states,
                ok_label,
                error_label,
            } => {
                if valid_states.len() == 2 {
                    let template = self.templates.get("state_check_2")?;
                    Some(
                        template
                            .replace("{{STATE_REG}}", state_reg)
                            .replace("{{STATE_1}}", &valid_states[0])
                            .replace("{{STATE_2}}", &valid_states[1])
                            .replace("{{OK_LABEL}}", ok_label)
                            .replace("{{ERROR_LABEL}}", error_label),
                    )
                } else {
                    None
                }
            }
            SlotType::StateTransition {
                state_reg,
                new_state,
            } => {
                let template = self.templates.get("state_transition")?;
                Some(
                    template
                        .replace("{{STATE_REG}}", state_reg)
                        .replace("{{NEW_STATE}}", new_state),
                )
            }
            SlotType::SendResponse {
                socket_reg,
                buffer_reg,
                length_reg,
            } => {
                let template = self.templates.get("send_response")?;
                Some(
                    template
                        .replace("{{SOCKET_REG}}", socket_reg)
                        .replace("{{BUFFER_REG}}", buffer_reg)
                        .replace("{{LENGTH_LABEL}}", length_reg),
                )
            }
            _ => None,
        }
    }
}

impl Default for TemplateBackend {
    fn default() -> Self {
        Self::new()
    }
}

impl SlotFillerBackend for TemplateBackend {
    fn fill_slot(&self, slot: &Slot) -> Result<String, FillError> {
        self.lookup_template(slot)
            .ok_or_else(|| FillError::InvalidSlotType(format!("{:?}", slot.slot_type.category())))
    }

    fn name(&self) -> &str {
        "template"
    }
}

/// Configuration for the slot filler
#[derive(Debug, Clone)]
pub struct FillerConfig {
    /// Enable caching
    pub use_cache: bool,
    /// Maximum parallel slots per batch
    pub max_batch_size: usize,
    /// Timeout per slot in milliseconds
    pub timeout_ms: u64,
    /// Retry failed slots
    pub retry_on_failure: bool,
    /// Maximum retries per slot
    pub max_retries: usize,
}

impl Default for FillerConfig {
    fn default() -> Self {
        FillerConfig {
            use_cache: true,
            max_batch_size: 64,
            timeout_ms: 5000,
            retry_on_failure: true,
            max_retries: 3,
        }
    }
}

/// Main slot filler that orchestrates filling
pub struct SlotFiller {
    /// Primary backend for slot generation
    backend: Box<dyn SlotFillerBackend>,
    /// Fallback backend (e.g., templates for common patterns)
    fallback: Option<Box<dyn SlotFillerBackend>>,
    /// Configuration
    config: FillerConfig,
    /// Cache for filled slots (slot_hash -> code)
    cache: HashMap<u64, String>,
}

impl SlotFiller {
    /// Create a new slot filler with the given backend
    pub fn new(backend: Box<dyn SlotFillerBackend>) -> Self {
        SlotFiller {
            backend,
            fallback: None,
            config: FillerConfig::default(),
            cache: HashMap::new(),
        }
    }

    /// Create with mock backend for testing
    pub fn mock() -> Self {
        Self::new(Box::new(MockBackend::realistic()))
    }

    /// Create with mock backend and template fallback
    pub fn mock_with_templates() -> Self {
        let mut filler = Self::new(Box::new(MockBackend::realistic()));
        filler.fallback = Some(Box::new(TemplateBackend::new()));
        filler
    }

    /// Set configuration
    pub fn with_config(mut self, config: FillerConfig) -> Self {
        self.config = config;
        self
    }

    /// Set fallback backend
    pub fn with_fallback(mut self, fallback: Box<dyn SlotFillerBackend>) -> Self {
        self.fallback = Some(fallback);
        self
    }

    /// Fill all slots in a spec
    pub fn fill(&mut self, spec: &SlotSpec) -> Result<FillResult, FillError> {
        let start = Instant::now();
        let mut filled_slots = Vec::with_capacity(spec.slots.len());
        let mut cache_hits = 0;
        let mut cache_misses = 0;

        // Collect slots to fill (checking cache first)
        let mut to_fill: Vec<(usize, &Slot)> = Vec::new();

        for (i, slot) in spec.slots.iter().enumerate() {
            let slot_hash = self.hash_slot(slot);

            if self.config.use_cache {
                if let Some(cached_code) = self.cache.get(&slot_hash) {
                    filled_slots.push(FilledSlot {
                        id: slot.id.clone(),
                        code: cached_code.clone(),
                        generation_time_ms: 0.0,
                        from_cache: true,
                        confidence: 1.0,
                    });
                    cache_hits += 1;
                    continue;
                }
            }

            to_fill.push((i, slot));
            cache_misses += 1;
        }

        // Fill remaining slots in batch
        if !to_fill.is_empty() {
            let slot_refs: Vec<&Slot> = to_fill.iter().map(|(_, s)| *s).collect();
            let batch_start = Instant::now();

            let codes = self.backend.fill_batch(&slot_refs)?;
            let batch_time = batch_start.elapsed().as_secs_f64() * 1000.0;
            let per_slot_time = batch_time / codes.len() as f64;

            for ((_orig_idx, slot), code) in to_fill.into_iter().zip(codes.into_iter()) {
                let slot_hash = self.hash_slot(slot);

                // Cache the result
                if self.config.use_cache {
                    self.cache.insert(slot_hash, code.clone());
                }

                filled_slots.push(FilledSlot {
                    id: slot.id.clone(),
                    code,
                    generation_time_ms: per_slot_time,
                    from_cache: false,
                    confidence: 0.9, // Mock confidence
                });
            }
        }

        // Sort by original order (cache hits may be out of order)
        // For now, we maintain order during collection

        Ok(FillResult {
            slots: filled_slots,
            total_time_ms: start.elapsed().as_secs_f64() * 1000.0,
            cache_hits,
            cache_misses,
        })
    }

    /// Fill a single slot
    pub fn fill_one(&mut self, slot: &Slot) -> Result<FilledSlot, FillError> {
        let start = Instant::now();
        let slot_hash = self.hash_slot(slot);

        // Check cache
        if self.config.use_cache {
            if let Some(cached_code) = self.cache.get(&slot_hash) {
                return Ok(FilledSlot {
                    id: slot.id.clone(),
                    code: cached_code.clone(),
                    generation_time_ms: 0.0,
                    from_cache: true,
                    confidence: 1.0,
                });
            }
        }

        // Try primary backend
        let code = match self.backend.fill_slot(slot) {
            Ok(code) => code,
            Err(e) => {
                // Try fallback if available
                if let Some(ref fallback) = self.fallback {
                    fallback.fill_slot(slot)?
                } else {
                    return Err(e);
                }
            }
        };

        // Cache result
        if self.config.use_cache {
            self.cache.insert(slot_hash, code.clone());
        }

        Ok(FilledSlot {
            id: slot.id.clone(),
            code,
            generation_time_ms: start.elapsed().as_secs_f64() * 1000.0,
            from_cache: false,
            confidence: 0.9,
        })
    }

    /// Clear the cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    /// Get cache statistics
    pub fn cache_stats(&self) -> (usize, usize) {
        (self.cache.len(), self.cache.capacity())
    }

    /// Hash a slot for caching
    fn hash_slot(&self, slot: &Slot) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        slot.id.hash(&mut hasher);
        // Hash the slot type details
        format!("{:?}", slot.slot_type).hash(&mut hasher);
        hasher.finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::slot::spec::{SlotContext, SlotSpec};

    #[test]
    fn test_mock_backend() {
        let backend = MockBackend::new();
        let slot = Slot {
            id: "TEST_SLOT".to_string(),
            name: "TEST_SLOT".to_string(),
            depends_on: Vec::new(),
            optional: false,
            slot_type: SlotType::StateTransition {
                state_reg: "r13".to_string(),
                new_state: "STATE_READY".to_string(),
            },
            context: SlotContext::default(),
            unit_test: None,
        };

        let code = backend.fill_slot(&slot).unwrap();
        assert!(code.contains("TEST_SLOT") || code.contains("Placeholder"));
    }

    #[test]
    fn test_realistic_mock_backend() {
        let backend = MockBackend::realistic();
        let slot = Slot {
            id: "STATE_CHECK".to_string(),
            name: "STATE_CHECK".to_string(),
            depends_on: Vec::new(),
            optional: false,
            slot_type: SlotType::StateCheck {
                state_reg: "r13".to_string(),
                valid_states: vec!["STATE_A".to_string(), "STATE_B".to_string()],
                ok_label: "state_ok".to_string(),
                error_label: "state_error".to_string(),
            },
            context: SlotContext::default(),
            unit_test: None,
        };

        let code = backend.fill_slot(&slot).unwrap();
        assert!(code.contains("StateCheck"));
        assert!(code.contains("STATE_A"));
        assert!(code.contains("state_ok"));
    }

    #[test]
    fn test_template_backend() {
        let backend = TemplateBackend::new();
        let slot = Slot {
            id: "STATE_TRANS".to_string(),
            name: "STATE_TRANS".to_string(),
            depends_on: Vec::new(),
            optional: false,
            slot_type: SlotType::StateTransition {
                state_reg: "r13".to_string(),
                new_state: "STATE_READY".to_string(),
            },
            context: SlotContext::default(),
            unit_test: None,
        };

        let code = backend.fill_slot(&slot).unwrap();
        assert!(code.contains("r13"));
        assert!(code.contains("STATE_READY"));
    }

    #[test]
    fn test_slot_filler_caching() {
        let mut filler = SlotFiller::mock();
        let slot = Slot {
            id: "CACHED_SLOT".to_string(),
            name: "CACHED_SLOT".to_string(),
            depends_on: Vec::new(),
            optional: false,
            slot_type: SlotType::StateTransition {
                state_reg: "r13".to_string(),
                new_state: "STATE_X".to_string(),
            },
            context: SlotContext::default(),
            unit_test: None,
        };

        // First fill - should miss cache
        let result1 = filler.fill_one(&slot).unwrap();
        assert!(!result1.from_cache);

        // Second fill - should hit cache
        let result2 = filler.fill_one(&slot).unwrap();
        assert!(result2.from_cache);
        assert_eq!(result1.code, result2.code);
    }

    #[test]
    fn test_batch_filling() {
        let mut filler = SlotFiller::mock();

        let mut spec = SlotSpec::new("test", "Test spec");
        spec.slots.push(Slot {
            id: "SLOT_1".to_string(),
            name: "SLOT_1".to_string(),
            depends_on: Vec::new(),
            optional: false,
            slot_type: SlotType::StateTransition {
                state_reg: "r13".to_string(),
                new_state: "STATE_A".to_string(),
            },
            context: SlotContext::default(),
            unit_test: None,
        });
        spec.slots.push(Slot {
            id: "SLOT_2".to_string(),
            name: "SLOT_2".to_string(),
            depends_on: Vec::new(),
            optional: false,
            slot_type: SlotType::StateTransition {
                state_reg: "r13".to_string(),
                new_state: "STATE_B".to_string(),
            },
            context: SlotContext::default(),
            unit_test: None,
        });

        let result = filler.fill(&spec).unwrap();
        assert_eq!(result.slots.len(), 2);
        assert_eq!(result.cache_misses, 2);
    }
}
