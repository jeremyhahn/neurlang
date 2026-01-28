//! Slot Cache
//!
//! Caches generated slot code for fast retrieval. Many slots are identical
//! across programs (e.g., PatternMatch("QUIT") always generates the same code).
//!
//! Cache key: hash(slot_type + params)
//! Expected hit rate: 60-80% after warmup

use std::collections::hash_map::DefaultHasher;
use std::collections::HashMap;
use std::fs;
use std::hash::{Hash, Hasher};
use std::path::Path;
use std::time::{Duration, Instant};

use super::spec::Slot;
use super::types::SlotType;

/// Cache statistics
#[derive(Debug, Clone, Default)]
pub struct CacheStats {
    /// Total cache lookups
    pub lookups: u64,
    /// Cache hits
    pub hits: u64,
    /// Cache misses
    pub misses: u64,
    /// Number of entries in cache
    pub entries: usize,
    /// Total memory used (approximate)
    pub memory_bytes: usize,
    /// Evictions due to capacity
    pub evictions: u64,
}

impl CacheStats {
    /// Get hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        if self.lookups == 0 {
            0.0
        } else {
            (self.hits as f64 / self.lookups as f64) * 100.0
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of entries
    pub max_entries: usize,
    /// Maximum memory usage in bytes (0 = unlimited)
    pub max_memory_bytes: usize,
    /// Entry TTL (0 = never expire)
    pub ttl: Duration,
    /// Whether to persist cache to disk
    pub persist: bool,
    /// Persistence file path
    pub persist_path: String,
}

impl Default for CacheConfig {
    fn default() -> Self {
        CacheConfig {
            max_entries: 10000,
            max_memory_bytes: 100 * 1024 * 1024, // 100MB
            ttl: Duration::ZERO,                 // Never expire
            persist: false,
            persist_path: ".slot_cache".to_string(),
        }
    }
}

/// Cache entry
#[derive(Debug, Clone)]
struct CacheEntry {
    /// Generated code
    code: String,
    /// When this entry was created
    created: Instant,
    /// Number of times accessed
    access_count: u64,
    /// Last access time
    last_access: Instant,
}

impl CacheEntry {
    fn new(code: String) -> Self {
        let now = Instant::now();
        CacheEntry {
            code,
            created: now,
            access_count: 1,
            last_access: now,
        }
    }

    fn access(&mut self) -> &str {
        self.access_count += 1;
        self.last_access = Instant::now();
        &self.code
    }

    fn memory_size(&self) -> usize {
        self.code.len() + std::mem::size_of::<Self>()
    }
}

/// Slot code cache
pub struct SlotCache {
    /// Cache entries
    entries: HashMap<u64, CacheEntry>,
    /// Configuration
    config: CacheConfig,
    /// Statistics
    stats: CacheStats,
    /// Current memory usage
    memory_used: usize,
}

impl SlotCache {
    /// Create a new cache with default configuration
    pub fn new() -> Self {
        SlotCache {
            entries: HashMap::new(),
            config: CacheConfig::default(),
            stats: CacheStats::default(),
            memory_used: 0,
        }
    }

    /// Create with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        let mut cache = SlotCache {
            entries: HashMap::new(),
            config,
            stats: CacheStats::default(),
            memory_used: 0,
        };

        // Try to load persisted cache
        if cache.config.persist {
            let _ = cache.load();
        }

        cache
    }

    /// Look up a slot in the cache
    pub fn get(&mut self, slot: &Slot) -> Option<String> {
        self.stats.lookups += 1;
        let key = self.hash_slot(slot);

        if let Some(entry) = self.entries.get_mut(&key) {
            // Check TTL
            if self.config.ttl > Duration::ZERO && entry.created.elapsed() > self.config.ttl {
                // Entry expired
                self.stats.misses += 1;
                return None;
            }

            self.stats.hits += 1;
            Some(entry.access().to_string())
        } else {
            self.stats.misses += 1;
            None
        }
    }

    /// Insert a slot's generated code into the cache
    pub fn put(&mut self, slot: &Slot, code: String) {
        let key = self.hash_slot(slot);
        let entry = CacheEntry::new(code);
        let entry_size = entry.memory_size();

        // Check if we need to evict
        self.ensure_capacity(entry_size);

        self.memory_used += entry_size;
        self.entries.insert(key, entry);
        self.stats.entries = self.entries.len();
    }

    /// Get or insert with a closure
    pub fn get_or_insert<F>(&mut self, slot: &Slot, f: F) -> String
    where
        F: FnOnce() -> String,
    {
        if let Some(code) = self.get(slot) {
            return code;
        }

        let code = f();
        self.put(slot, code.clone());
        code
    }

    /// Clear the cache
    pub fn clear(&mut self) {
        self.entries.clear();
        self.memory_used = 0;
        self.stats.entries = 0;
    }

    /// Get cache statistics
    pub fn stats(&self) -> &CacheStats {
        &self.stats
    }

    /// Reset statistics
    pub fn reset_stats(&mut self) {
        self.stats = CacheStats::default();
        self.stats.entries = self.entries.len();
    }

    /// Get number of entries
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Check if cache is empty
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    /// Save cache to disk
    pub fn save(&self) -> std::io::Result<()> {
        if !self.config.persist {
            return Ok(());
        }

        // Serialize entries as JSON
        let mut data = Vec::new();
        for (key, entry) in &self.entries {
            data.push(format!("{}:{}", key, entry.code.replace('\n', "\\n")));
        }

        fs::write(&self.config.persist_path, data.join("\n"))?;
        Ok(())
    }

    /// Load cache from disk
    pub fn load(&mut self) -> std::io::Result<usize> {
        if !self.config.persist {
            return Ok(0);
        }

        let path = Path::new(&self.config.persist_path);
        if !path.exists() {
            return Ok(0);
        }

        let content = fs::read_to_string(path)?;
        let mut loaded = 0;

        for line in content.lines() {
            if let Some(idx) = line.find(':') {
                if let Ok(key) = line[..idx].parse::<u64>() {
                    let code = line[idx + 1..].replace("\\n", "\n");
                    let entry = CacheEntry::new(code);
                    let entry_size = entry.memory_size();
                    self.memory_used += entry_size;
                    self.entries.insert(key, entry);
                    loaded += 1;
                }
            }
        }

        self.stats.entries = self.entries.len();
        Ok(loaded)
    }

    /// Ensure there's capacity for a new entry
    fn ensure_capacity(&mut self, new_entry_size: usize) {
        // Check entry count
        while self.entries.len() >= self.config.max_entries {
            self.evict_one();
        }

        // Check memory
        if self.config.max_memory_bytes > 0 {
            while self.memory_used + new_entry_size > self.config.max_memory_bytes {
                if !self.evict_one() {
                    break;
                }
            }
        }
    }

    /// Evict the least recently used entry
    fn evict_one(&mut self) -> bool {
        if self.entries.is_empty() {
            return false;
        }

        // Find LRU entry
        let lru_key = self
            .entries
            .iter()
            .min_by_key(|(_, e)| e.last_access)
            .map(|(k, _)| *k);

        if let Some(key) = lru_key {
            if let Some(entry) = self.entries.remove(&key) {
                self.memory_used = self.memory_used.saturating_sub(entry.memory_size());
                self.stats.evictions += 1;
                return true;
            }
        }

        false
    }

    /// Hash a slot for cache key
    fn hash_slot(&self, slot: &Slot) -> u64 {
        let mut hasher = DefaultHasher::new();

        // Hash slot type with all parameters
        match &slot.slot_type {
            SlotType::PatternMatch {
                pattern,
                input_reg,
                captures,
                match_label,
                no_match_label,
            } => {
                "PatternMatch".hash(&mut hasher);
                pattern.hash(&mut hasher);
                input_reg.hash(&mut hasher);
                captures.len().hash(&mut hasher);
                match_label.hash(&mut hasher);
                no_match_label.hash(&mut hasher);
            }
            SlotType::PatternSwitch {
                input_reg,
                cases,
                default_label,
            } => {
                "PatternSwitch".hash(&mut hasher);
                input_reg.hash(&mut hasher);
                cases.len().hash(&mut hasher);
                for (pattern, label) in cases {
                    pattern.hash(&mut hasher);
                    label.hash(&mut hasher);
                }
                default_label.hash(&mut hasher);
            }
            SlotType::ResponseBuilder {
                template,
                variables,
                output_reg,
                length_reg,
            } => {
                "ResponseBuilder".hash(&mut hasher);
                template.hash(&mut hasher);
                variables.len().hash(&mut hasher);
                output_reg.hash(&mut hasher);
                length_reg.hash(&mut hasher);
            }
            SlotType::StateCheck {
                state_reg,
                valid_states,
                ok_label,
                error_label,
            } => {
                "StateCheck".hash(&mut hasher);
                state_reg.hash(&mut hasher);
                for state in valid_states {
                    state.hash(&mut hasher);
                }
                ok_label.hash(&mut hasher);
                error_label.hash(&mut hasher);
            }
            SlotType::StateTransition {
                state_reg,
                new_state,
            } => {
                "StateTransition".hash(&mut hasher);
                state_reg.hash(&mut hasher);
                new_state.hash(&mut hasher);
            }
            SlotType::SendResponse {
                socket_reg,
                buffer_reg,
                length_reg,
            } => {
                "SendResponse".hash(&mut hasher);
                socket_reg.hash(&mut hasher);
                buffer_reg.hash(&mut hasher);
                length_reg.hash(&mut hasher);
            }
            SlotType::ReadUntil {
                socket_reg,
                buffer_reg,
                delimiter,
                max_len,
                length_reg,
                eof_label,
            } => {
                "ReadUntil".hash(&mut hasher);
                socket_reg.hash(&mut hasher);
                buffer_reg.hash(&mut hasher);
                delimiter.hash(&mut hasher);
                max_len.hash(&mut hasher);
                length_reg.hash(&mut hasher);
                eof_label.hash(&mut hasher);
            }
            SlotType::ExtensionCall {
                extension,
                args,
                result_reg,
            } => {
                "ExtensionCall".hash(&mut hasher);
                extension.hash(&mut hasher);
                for arg in args {
                    arg.hash(&mut hasher);
                }
                result_reg.hash(&mut hasher);
            }
            SlotType::ErrorResponse {
                socket_reg,
                error_code,
                error_message,
                close_after,
            } => {
                "ErrorResponse".hash(&mut hasher);
                socket_reg.hash(&mut hasher);
                error_code.hash(&mut hasher);
                error_message.hash(&mut hasher);
                close_after.hash(&mut hasher);
            }
            // Hash other types by their debug representation
            other => {
                format!("{:?}", other).hash(&mut hasher);
            }
        }

        hasher.finish()
    }

    /// Preload common patterns into cache
    pub fn preload_common_patterns(&mut self) {
        // Common QUIT command pattern
        let quit_slot = Slot::new(
            "QUIT_PATTERN",
            "quit_pattern_match",
            SlotType::PatternMatch {
                pattern: "QUIT".to_string(),
                input_reg: "r0".to_string(),
                captures: vec![],
                match_label: "quit_match".to_string(),
                no_match_label: "quit_no_match".to_string(),
            },
        );
        self.put(
            &quit_slot,
            r#"; PatternMatch: QUIT
    load.b r1, [r0]
    mov r2, 81  ; 'Q'
    bne r1, r2, quit_no_match
    load.b r1, [r0 + 1]
    mov r2, 85  ; 'U'
    bne r1, r2, quit_no_match
    load.b r1, [r0 + 2]
    mov r2, 73  ; 'I'
    bne r1, r2, quit_no_match
    load.b r1, [r0 + 3]
    mov r2, 84  ; 'T'
    bne r1, r2, quit_no_match
    b quit_match
"#
            .to_string(),
        );

        // Common OK response
        let ok_response = Slot::new(
            "OK_RESPONSE",
            "ok_response_send",
            SlotType::SendResponse {
                socket_reg: "r11".to_string(),
                buffer_reg: "resp_ok".to_string(),
                length_reg: "resp_ok_len".to_string(),
            },
        );
        self.put(
            &ok_response,
            r#"; SendResponse: OK
    mov r0, r11
    lea r1, resp_ok
    load r2, [resp_ok_len]
    io.send r0, r0, r1, r2
"#
            .to_string(),
        );
    }
}

impl Default for SlotCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Drop for SlotCache {
    fn drop(&mut self) {
        if self.config.persist {
            let _ = self.save();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_test_slot(id: &str, state: &str) -> Slot {
        Slot::new(
            id,
            id, // use id as name too
            SlotType::StateTransition {
                state_reg: "r13".to_string(),
                new_state: state.to_string(),
            },
        )
    }

    #[test]
    fn test_cache_basic() {
        let mut cache = SlotCache::new();
        let slot = make_test_slot("TEST", "STATE_A");

        // Miss
        assert!(cache.get(&slot).is_none());
        assert_eq!(cache.stats().misses, 1);

        // Insert
        cache.put(&slot, "mov r13, STATE_A".to_string());

        // Hit
        let code = cache.get(&slot);
        assert!(code.is_some());
        assert_eq!(code.unwrap(), "mov r13, STATE_A");
        assert_eq!(cache.stats().hits, 1);
    }

    #[test]
    fn test_cache_different_params() {
        let mut cache = SlotCache::new();

        let slot_a = make_test_slot("SLOT", "STATE_A");
        let slot_b = make_test_slot("SLOT", "STATE_B");

        cache.put(&slot_a, "code_a".to_string());
        cache.put(&slot_b, "code_b".to_string());

        assert_eq!(cache.get(&slot_a).unwrap(), "code_a");
        assert_eq!(cache.get(&slot_b).unwrap(), "code_b");
    }

    #[test]
    fn test_cache_eviction() {
        let config = CacheConfig {
            max_entries: 3,
            ..Default::default()
        };
        let mut cache = SlotCache::with_config(config);

        // Fill cache
        for i in 0..5 {
            let slot = make_test_slot(&format!("SLOT_{}", i), &format!("STATE_{}", i));
            cache.put(&slot, format!("code_{}", i));
        }

        // Should have evicted some entries
        assert_eq!(cache.len(), 3);
        assert!(cache.stats().evictions > 0);
    }

    #[test]
    fn test_cache_get_or_insert() {
        let mut cache = SlotCache::new();
        let slot = make_test_slot("TEST", "STATE_X");

        let mut called = false;
        let code = cache.get_or_insert(&slot, || {
            called = true;
            "generated_code".to_string()
        });

        assert!(called);
        assert_eq!(code, "generated_code");

        // Second call should not invoke closure
        called = false;
        let code2 = cache.get_or_insert(&slot, || {
            called = true;
            "should_not_see_this".to_string()
        });

        assert!(!called);
        assert_eq!(code2, "generated_code");
    }

    #[test]
    fn test_cache_hit_rate() {
        let mut cache = SlotCache::new();
        let slot = make_test_slot("TEST", "STATE_A");

        cache.put(&slot, "code".to_string());

        // 10 lookups, all hits
        for _ in 0..10 {
            cache.get(&slot);
        }

        assert_eq!(cache.stats().hit_rate(), 100.0);
    }

    #[test]
    fn test_preload_common_patterns() {
        let mut cache = SlotCache::new();
        cache.preload_common_patterns();

        assert!(cache.len() >= 2);
    }
}
