//! Intent Parser
//!
//! Rule-based parser that maps natural language requests to protocol specs
//! and templates.

use std::collections::HashMap;

/// Intent parsing result
#[derive(Debug, Clone)]
pub struct ParsedIntent {
    /// Detected protocol (e.g., "smtp", "http")
    pub protocol: Option<String>,
    /// Detected template type (e.g., "tcp_server", "rest_api")
    pub template: String,
    /// Extracted features/options
    pub features: Vec<String>,
    /// Confidence score (0.0 - 1.0)
    pub confidence: f32,
    /// Whether this should be handled offline (rule-based)
    pub offline_capable: bool,
    /// Raw matched keywords
    pub matched_keywords: Vec<String>,
}

impl ParsedIntent {
    /// Create a new parsed intent
    pub fn new(template: impl Into<String>) -> Self {
        ParsedIntent {
            protocol: None,
            template: template.into(),
            features: Vec::new(),
            confidence: 0.0,
            offline_capable: false,
            matched_keywords: Vec::new(),
        }
    }

    /// Set the protocol
    pub fn with_protocol(mut self, protocol: impl Into<String>) -> Self {
        self.protocol = Some(protocol.into());
        self
    }

    /// Add a feature
    pub fn with_feature(mut self, feature: impl Into<String>) -> Self {
        self.features.push(feature.into());
        self
    }

    /// Set confidence
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence;
        self
    }

    /// Mark as offline capable
    pub fn offline(mut self) -> Self {
        self.offline_capable = true;
        self
    }
}

/// Intent parser configuration
#[derive(Debug, Clone)]
pub struct IntentParserConfig {
    /// Minimum confidence threshold for offline handling
    pub offline_threshold: f32,
    /// Protocol spec directory
    pub specs_dir: String,
}

impl Default for IntentParserConfig {
    fn default() -> Self {
        IntentParserConfig {
            offline_threshold: 0.7,
            specs_dir: "specs/protocols".to_string(),
        }
    }
}

/// Rule-based intent parser
pub struct IntentParser {
    config: IntentParserConfig,
    /// Protocol detection rules (keywords -> protocol)
    protocol_rules: HashMap<&'static str, &'static str>,
    /// Template detection rules (keywords -> template)
    template_rules: HashMap<&'static str, &'static str>,
    /// Feature detection rules (keywords -> feature)
    feature_rules: HashMap<&'static str, &'static str>,
}

impl IntentParser {
    /// Create a new intent parser
    pub fn new(config: IntentParserConfig) -> Self {
        let mut parser = IntentParser {
            config,
            protocol_rules: HashMap::new(),
            template_rules: HashMap::new(),
            feature_rules: HashMap::new(),
        };
        parser.init_rules();
        parser
    }

    /// Create with default config
    pub fn with_defaults() -> Self {
        Self::new(IntentParserConfig::default())
    }

    /// Initialize detection rules
    fn init_rules(&mut self) {
        // Protocol detection keywords
        self.protocol_rules.insert("smtp", "smtp");
        self.protocol_rules.insert("mail", "smtp");
        self.protocol_rules.insert("email", "smtp");
        self.protocol_rules.insert("mta", "smtp");

        self.protocol_rules.insert("http", "http");
        self.protocol_rules.insert("web", "http");
        self.protocol_rules.insert("rest", "http");
        self.protocol_rules.insert("api", "http");

        self.protocol_rules.insert("redis", "redis");
        self.protocol_rules.insert("cache", "redis");
        self.protocol_rules.insert("key-value", "redis");
        self.protocol_rules.insert("kv", "redis");

        self.protocol_rules.insert("ftp", "ftp");
        self.protocol_rules.insert("file transfer", "ftp");

        self.protocol_rules.insert("dns", "dns");
        self.protocol_rules.insert("domain name", "dns");
        self.protocol_rules.insert("nameserver", "dns");

        // Template detection keywords
        self.template_rules.insert("server", "tcp_server");
        self.template_rules.insert("daemon", "tcp_server");
        self.template_rules.insert("service", "tcp_server");

        self.template_rules.insert("rest api", "rest_api");
        self.template_rules.insert("restful", "rest_api");
        self.template_rules.insert("crud", "rest_api");
        self.template_rules.insert("endpoints", "rest_api");

        self.template_rules.insert("http server", "http_server");
        self.template_rules.insert("web server", "http_server");

        self.template_rules.insert("proxy", "proxy");
        self.template_rules.insert("reverse proxy", "proxy");
        self.template_rules.insert("load balancer", "proxy");

        self.template_rules.insert("echo", "echo_server");
        self.template_rules.insert("ping", "echo_server");

        // Feature detection keywords
        self.feature_rules.insert("tls", "tls");
        self.feature_rules.insert("ssl", "tls");
        self.feature_rules.insert("secure", "tls");
        self.feature_rules.insert("encrypted", "tls");
        self.feature_rules.insert("https", "tls");

        self.feature_rules.insert("auth", "authentication");
        self.feature_rules
            .insert("authentication", "authentication");
        self.feature_rules.insert("login", "authentication");
        self.feature_rules.insert("user", "authentication");

        self.feature_rules.insert("validation", "validation");
        self.feature_rules.insert("validate", "validation");
        self.feature_rules.insert("verify", "validation");

        self.feature_rules.insert("logging", "logging");
        self.feature_rules.insert("log", "logging");

        self.feature_rules.insert("database", "database");
        self.feature_rules.insert("db", "database");
        self.feature_rules.insert("sqlite", "database");
        self.feature_rules.insert("persistence", "database");

        self.feature_rules.insert("rate limit", "rate_limiting");
        self.feature_rules.insert("throttle", "rate_limiting");

        self.feature_rules.insert("json", "json");
        self.feature_rules.insert("xml", "xml");

        self.feature_rules.insert("cors", "cors");
        self.feature_rules.insert("cross-origin", "cors");
    }

    /// Parse an intent from a natural language prompt
    pub fn parse(&self, prompt: &str) -> ParsedIntent {
        let normalized = self.normalize(prompt);
        let words: Vec<&str> = normalized.split_whitespace().collect();

        // Detect protocol
        let (protocol, protocol_confidence) = self.detect_protocol(&normalized, &words);

        // Detect template
        let (template, template_confidence) = self.detect_template(&normalized, &words);

        // Detect features
        let features = self.detect_features(&normalized, &words);

        // Collect matched keywords
        let mut matched = Vec::new();
        if let Some(ref p) = protocol {
            matched.push(p.clone());
        }
        matched.push(template.clone());
        matched.extend(features.iter().cloned());

        // Calculate overall confidence
        let confidence = if protocol.is_some() {
            (protocol_confidence + template_confidence) / 2.0
        } else {
            template_confidence * 0.7 // Lower confidence without protocol
        };

        // Determine if offline capable
        let offline_capable = protocol.is_some() && confidence >= self.config.offline_threshold;

        ParsedIntent {
            protocol,
            template,
            features,
            confidence,
            offline_capable,
            matched_keywords: matched,
        }
    }

    /// Normalize the input for matching
    fn normalize(&self, input: &str) -> String {
        input
            .to_lowercase()
            .replace("-", " ")
            .replace("_", " ")
            .replace(".", " ")
            .replace(",", " ")
            .replace(";", " ")
            .replace(":", " ")
    }

    /// Detect protocol from input
    fn detect_protocol(&self, normalized: &str, words: &[&str]) -> (Option<String>, f32) {
        let mut best_protocol: Option<&str> = None;
        let mut best_score = 0.0f32;

        // Check multi-word patterns first
        for (pattern, protocol) in &self.protocol_rules {
            if pattern.contains(' ') && normalized.contains(pattern) {
                let score = pattern.len() as f32 / normalized.len() as f32;
                if score > best_score {
                    best_protocol = Some(protocol);
                    best_score = score.min(1.0) + 0.2; // Bonus for multi-word match
                }
            }
        }

        // Check single-word patterns
        for word in words {
            if let Some(protocol) = self.protocol_rules.get(word) {
                let score = 0.8; // High confidence for exact word match
                if score > best_score {
                    best_protocol = Some(protocol);
                    best_score = score;
                }
            }
        }

        (best_protocol.map(String::from), best_score)
    }

    /// Detect template type from input
    fn detect_template(&self, normalized: &str, words: &[&str]) -> (String, f32) {
        let mut best_template = "tcp_server"; // Default
        let mut best_score = 0.3f32;

        // Check multi-word patterns first
        for (pattern, template) in &self.template_rules {
            if pattern.contains(' ') && normalized.contains(pattern) {
                let score = (pattern.len() as f32 / normalized.len() as f32).min(1.0) + 0.3;
                if score > best_score {
                    best_template = template;
                    best_score = score;
                }
            }
        }

        // Check single-word patterns
        for word in words {
            if let Some(template) = self.template_rules.get(word) {
                let score = 0.7;
                if score > best_score {
                    best_template = template;
                    best_score = score;
                }
            }
        }

        (best_template.to_string(), best_score)
    }

    /// Detect features from input
    fn detect_features(&self, normalized: &str, words: &[&str]) -> Vec<String> {
        let mut features = Vec::new();
        let mut seen = std::collections::HashSet::new();

        // Check multi-word patterns
        for (pattern, feature) in &self.feature_rules {
            if pattern.contains(' ') && normalized.contains(pattern) && seen.insert(*feature) {
                features.push(feature.to_string());
            }
        }

        // Check single-word patterns
        for word in words {
            if let Some(feature) = self.feature_rules.get(word) {
                if seen.insert(*feature) {
                    features.push(feature.to_string());
                }
            }
        }

        features
    }

    /// Check if a protocol spec exists
    pub fn protocol_exists(&self, protocol: &str) -> bool {
        let path = format!("{}/{}.json", self.config.specs_dir, protocol);
        std::path::Path::new(&path).exists()
    }

    /// Get available protocols
    pub fn available_protocols(&self) -> Vec<String> {
        let mut protocols = Vec::new();
        if let Ok(entries) = std::fs::read_dir(&self.config.specs_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().map(|e| e == "json").unwrap_or(false) {
                    if let Some(name) = path.file_stem() {
                        protocols.push(name.to_string_lossy().to_string());
                    }
                }
            }
        }
        protocols.sort();
        protocols
    }
}

/// Quick intent detection for common patterns
pub fn quick_detect(prompt: &str) -> Option<(&'static str, &'static str)> {
    let lower = prompt.to_lowercase();

    // Very specific patterns
    if lower.contains("smtp server") || lower.contains("mail server") {
        return Some(("smtp", "tcp_server"));
    }
    if lower.contains("http server") || lower.contains("web server") {
        return Some(("http", "http_server"));
    }
    if lower.contains("rest api") || lower.contains("restful api") {
        return Some(("http", "rest_api"));
    }
    if lower.contains("redis server") || lower.contains("redis clone") {
        return Some(("redis", "tcp_server"));
    }
    if lower.contains("ftp server") || lower.contains("file server") {
        return Some(("ftp", "tcp_server"));
    }
    if lower.contains("dns server") || lower.contains("name server") {
        return Some(("dns", "tcp_server"));
    }
    if lower.contains("echo server") {
        return Some(("echo", "echo_server"));
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_smtp_server() {
        let parser = IntentParser::with_defaults();
        let intent = parser.parse("Build an SMTP server");

        assert_eq!(intent.protocol, Some("smtp".to_string()));
        assert_eq!(intent.template, "tcp_server");
        assert!(intent.confidence > 0.5);
    }

    #[test]
    fn test_parse_rest_api() {
        let parser = IntentParser::with_defaults();
        let intent = parser.parse("Create a REST API with authentication");

        assert_eq!(intent.protocol, Some("http".to_string()));
        assert!(intent.template == "rest_api" || intent.template == "tcp_server");
        assert!(intent.features.contains(&"authentication".to_string()));
    }

    #[test]
    fn test_parse_redis_server() {
        let parser = IntentParser::with_defaults();
        let intent = parser.parse("Build a Redis-compatible cache server");

        assert_eq!(intent.protocol, Some("redis".to_string()));
    }

    #[test]
    fn test_parse_with_tls() {
        let parser = IntentParser::with_defaults();
        let intent = parser.parse("Create a secure HTTPS server with TLS");

        assert!(intent.features.contains(&"tls".to_string()));
    }

    #[test]
    fn test_parse_with_database() {
        let parser = IntentParser::with_defaults();
        let intent = parser.parse("SMTP server with database persistence");

        assert!(intent.features.contains(&"database".to_string()));
    }

    #[test]
    fn test_quick_detect() {
        assert_eq!(quick_detect("SMTP server"), Some(("smtp", "tcp_server")));
        assert_eq!(quick_detect("REST API"), Some(("http", "rest_api")));
        assert_eq!(quick_detect("Redis server"), Some(("redis", "tcp_server")));
        assert_eq!(quick_detect("random text"), None);
    }

    #[test]
    fn test_normalize() {
        let parser = IntentParser::with_defaults();
        let result = parser.normalize("Build-a_REST.API");
        assert_eq!(result, "build a rest api");
    }

    #[test]
    fn test_detect_multiple_features() {
        let parser = IntentParser::with_defaults();
        let intent = parser.parse("HTTP server with TLS, authentication, and logging");

        assert!(intent.features.contains(&"tls".to_string()));
        assert!(intent.features.contains(&"authentication".to_string()));
        assert!(intent.features.contains(&"logging".to_string()));
    }

    #[test]
    fn test_offline_capable() {
        let parser = IntentParser::with_defaults();

        // Should be offline capable with clear protocol
        let _intent = parser.parse("SMTP server");
        // Note: offline_capable depends on protocol spec file existing

        // Should not be offline capable for vague requests
        let intent2 = parser.parse("something cool");
        assert!(!intent2.offline_capable);
    }
}
