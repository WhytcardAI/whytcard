//! Procedural Memory - Rules, patterns, and learned workflows
//!
//! Stores learned procedures, routing rules, and patterns.
//! These are "how to do" knowledge that evolves with learning.

use crate::error::{IntelligenceError, Result};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Constants
const RULES_FILE: &str = "rules.yaml";
const PATTERNS_FILE: &str = "patterns.yaml";
const ROUTING_FILE: &str = "routing.yaml";
const MIN_CONFIDENCE: f32 = 0.5;
const CONFIDENCE_INCREMENT: f32 = 0.1;
const CONFIDENCE_DECREMENT: f32 = 0.15;

/// Procedural memory for rules and learned procedures
pub struct ProceduralMemory {
    /// Base path for YAML files
    base_path: PathBuf,

    /// Cached rules
    rules: HashMap<String, Rule>,

    /// Cached patterns
    patterns: HashMap<String, Pattern>,

    /// Cached routing rules
    routing: HashMap<String, RoutingRule>,

    /// Whether using in-memory mode
    in_memory: bool,

    /// Whether initialized
    initialized: bool,
}

impl ProceduralMemory {
    /// Create new procedural memory at the given path
    pub async fn new(base_path: &Path) -> Result<Self> {
        // Ensure directory exists
        if !base_path.exists() {
            std::fs::create_dir_all(base_path)?;
        }

        let mut mem = Self {
            base_path: base_path.to_path_buf(),
            rules: HashMap::new(),
            patterns: HashMap::new(),
            routing: HashMap::new(),
            in_memory: false,
            initialized: false,
        };

        mem.load_all().await?;
        mem.initialized = true;

        Ok(mem)
    }

    /// Create in-memory procedural memory for testing
    #[cfg(test)]
    pub async fn in_memory() -> Result<Self> {
        let mut mem = Self {
            base_path: PathBuf::from(":memory:"),
            rules: HashMap::new(),
            patterns: HashMap::new(),
            routing: HashMap::new(),
            in_memory: true,
            initialized: true,
        };

        // Add default rules and patterns
        mem.add_default_rules();
        mem.add_default_patterns();

        Ok(mem)
    }

    /// Load all YAML files
    async fn load_all(&mut self) -> Result<()> {
        self.load_rules()?;
        self.load_patterns()?;
        self.load_routing()?;
        Ok(())
    }

    /// Load rules from YAML
    fn load_rules(&mut self) -> Result<()> {
        let path = self.base_path.join(RULES_FILE);

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let data: RulesFile = serde_yaml::from_str(&content)
                .map_err(|e| IntelligenceError::config(format!("Failed to parse rules.yaml: {}", e)))?;

            for rule in data.rules {
                self.rules.insert(rule.id.clone(), rule);
            }
        } else {
            self.add_default_rules();
            self.save_rules()?;
        }

        Ok(())
    }

    /// Load patterns from YAML
    fn load_patterns(&mut self) -> Result<()> {
        let path = self.base_path.join(PATTERNS_FILE);

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let data: PatternsFile = serde_yaml::from_str(&content)
                .map_err(|e| IntelligenceError::config(format!("Failed to parse patterns.yaml: {}", e)))?;

            for pattern in data.patterns {
                self.patterns.insert(pattern.id.clone(), pattern);
            }
        } else {
            self.add_default_patterns();
            self.save_patterns()?;
        }

        Ok(())
    }

    /// Load routing from YAML
    fn load_routing(&mut self) -> Result<()> {
        let path = self.base_path.join(ROUTING_FILE);

        if path.exists() {
            let content = std::fs::read_to_string(&path)?;
            let data: RoutingFile = serde_yaml::from_str(&content)
                .map_err(|e| IntelligenceError::config(format!("Failed to parse routing.yaml: {}", e)))?;

            for rule in data.routing {
                self.routing.insert(rule.id.clone(), rule);
            }
        } else {
            self.add_default_routing();
            self.save_routing()?;
        }

        Ok(())
    }

    /// Save rules to YAML
    fn save_rules(&self) -> Result<()> {
        if self.in_memory {
            return Ok(());
        }

        let path = self.base_path.join(RULES_FILE);
        let data = RulesFile {
            rules: self.rules.values().cloned().collect(),
        };
        let content = serde_yaml::to_string(&data)
            .map_err(|e| IntelligenceError::config(format!("Failed to serialize rules: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Save patterns to YAML
    fn save_patterns(&self) -> Result<()> {
        if self.in_memory {
            return Ok(());
        }

        let path = self.base_path.join(PATTERNS_FILE);
        let data = PatternsFile {
            patterns: self.patterns.values().cloned().collect(),
        };
        let content = serde_yaml::to_string(&data)
            .map_err(|e| IntelligenceError::config(format!("Failed to serialize patterns: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Save routing to YAML
    fn save_routing(&self) -> Result<()> {
        if self.in_memory {
            return Ok(());
        }

        let path = self.base_path.join(ROUTING_FILE);
        let data = RoutingFile {
            routing: self.routing.values().cloned().collect(),
        };
        let content = serde_yaml::to_string(&data)
            .map_err(|e| IntelligenceError::config(format!("Failed to serialize routing: {}", e)))?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Add default rules
    fn add_default_rules(&mut self) {
        let now = chrono::Utc::now().to_rfc3339();

        self.rules.insert("rule-001".into(), Rule {
            id: "rule-001".into(),
            name: "code_request".into(),
            condition: "generate|create|write|implement".into(),
            action: "route_to_code_agent".into(),
            confidence: 0.9,
            success_count: 0,
            failure_count: 0,
            created_at: now.clone(),
            updated_at: now.clone(),
        });

        self.rules.insert("rule-002".into(), Rule {
            id: "rule-002".into(),
            name: "search_request".into(),
            condition: "find|search|where|locate".into(),
            action: "route_to_search_agent".into(),
            confidence: 0.9,
            success_count: 0,
            failure_count: 0,
            created_at: now.clone(),
            updated_at: now,
        });
    }

    /// Add default patterns
    fn add_default_patterns(&mut self) {
        self.patterns.insert("pat-001".into(), Pattern {
            id: "pat-001".into(),
            name: "code_generation".into(),
            regex: r"(?i)(generate|create|write|implement|add)\s+.*(code|function|class|method)".into(),
            category: "query_type".into(),
            priority: 1,
            metadata: None,
        });

        self.patterns.insert("pat-002".into(), Pattern {
            id: "pat-002".into(),
            name: "file_search".into(),
            regex: r"(?i)(find|search|locate|where)\s+.*(file|in|is)".into(),
            category: "query_type".into(),
            priority: 2,
            metadata: None,
        });

        self.patterns.insert("pat-003".into(), Pattern {
            id: "pat-003".into(),
            name: "explanation".into(),
            regex: r"(?i)(explain|what\s+is|how\s+does|why)".into(),
            category: "query_type".into(),
            priority: 3,
            metadata: None,
        });
    }

    /// Add default routing
    fn add_default_routing(&mut self) {
        self.routing.insert("route-001".into(), RoutingRule {
            id: "route-001".into(),
            pattern_id: "pat-001".into(),
            target_agent: "code".into(),
            confidence: 0.9,
            usage_count: 0,
        });

        self.routing.insert("route-002".into(), RoutingRule {
            id: "route-002".into(),
            pattern_id: "pat-002".into(),
            target_agent: "search".into(),
            confidence: 0.9,
            usage_count: 0,
        });
    }

    /// Match patterns against text
    pub fn match_patterns(&self, text: &str, category: Option<&str>) -> Vec<PatternMatch> {
        let mut matches = Vec::new();

        for pattern in self.patterns.values() {
            // Filter by category if provided
            if let Some(cat) = category {
                if pattern.category != cat {
                    continue;
                }
            }

            // Compile and test regex
            if let Ok(re) = Regex::new(&pattern.regex) {
                if re.is_match(text) {
                    matches.push(PatternMatch {
                        pattern_id: pattern.id.clone(),
                        pattern_name: pattern.name.clone(),
                        category: pattern.category.clone(),
                        priority: pattern.priority,
                    });
                }
            }
        }

        // Sort by priority (lower = better)
        matches.sort_by_key(|m| m.priority);
        matches
    }

    /// Get routing recommendation for a query
    pub fn get_routing(&self, query: &str) -> Option<RoutingRecommendation> {
        let mut candidates: Vec<_> = self.routing.values()
            .filter_map(|rule| {
                // Get the associated pattern
                let pattern = self.patterns.get(&rule.pattern_id)?;

                // Check if pattern matches
                let re = Regex::new(&pattern.regex).ok()?;
                if !re.is_match(query) {
                    return None;
                }

                // Check minimum confidence
                if rule.confidence < MIN_CONFIDENCE {
                    return None;
                }

                Some((rule, pattern))
            })
            .collect();

        // Sort by confidence (desc) then usage_count (desc)
        candidates.sort_by(|a, b| {
            b.0.confidence.partial_cmp(&a.0.confidence)
                .unwrap_or(std::cmp::Ordering::Equal)
                .then_with(|| b.0.usage_count.cmp(&a.0.usage_count))
        });

        candidates.first().map(|(rule, pattern)| RoutingRecommendation {
            target_agent: rule.target_agent.clone(),
            confidence: rule.confidence,
            pattern_name: pattern.name.clone(),
            routing_id: rule.id.clone(),
        })
    }

    /// Get applicable rules for a context
    pub fn get_applicable_rules(&self, context: &serde_json::Value) -> Vec<Rule> {
        let context_str = context.to_string().to_lowercase();

        self.rules.values()
            .filter(|rule| {
                // Check if condition pattern matches context
                if let Ok(re) = Regex::new(&rule.condition) {
                    re.is_match(&context_str) && rule.confidence >= MIN_CONFIDENCE
                } else {
                    false
                }
            })
            .cloned()
            .collect()
    }

    /// Add a new rule
    pub fn add_rule(&mut self, name: String, condition: String, action: String, confidence: f32) -> Result<String> {
        let id = format!("rule-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());
        let now = chrono::Utc::now().to_rfc3339();

        let rule = Rule {
            id: id.clone(),
            name,
            condition,
            action,
            confidence,
            success_count: 0,
            failure_count: 0,
            created_at: now.clone(),
            updated_at: now,
        };

        self.rules.insert(id.clone(), rule);
        self.save_rules()?;

        Ok(id)
    }

    /// Add a new pattern
    pub fn add_pattern(&mut self, name: String, regex: String, category: String, priority: i32) -> Result<String> {
        // Validate regex
        Regex::new(&regex)
            .map_err(|e| IntelligenceError::config(format!("Invalid regex: {}", e)))?;

        let id = format!("pat-{}", uuid::Uuid::new_v4().to_string().split('-').next().unwrap());

        let pattern = Pattern {
            id: id.clone(),
            name,
            regex,
            category,
            priority,
            metadata: None,
        };

        self.patterns.insert(id.clone(), pattern);
        self.save_patterns()?;

        Ok(id)
    }

    /// Update confidence for a rule based on success/failure
    pub fn update_confidence(&mut self, rule_id: &str, success: bool) -> Result<f32> {
        let rule = self.rules.get_mut(rule_id)
            .ok_or_else(|| IntelligenceError::KeyNotFound(format!("Rule not found: {}", rule_id)))?;

        if success {
            rule.success_count += 1;
            rule.confidence = (rule.confidence + CONFIDENCE_INCREMENT).min(1.0);
        } else {
            rule.failure_count += 1;
            rule.confidence = (rule.confidence - CONFIDENCE_DECREMENT).max(0.0);
        }

        rule.updated_at = chrono::Utc::now().to_rfc3339();
        let new_confidence = rule.confidence;

        self.save_rules()?;

        Ok(new_confidence)
    }

    /// Increment routing usage count
    pub fn increment_routing_usage(&mut self, routing_id: &str) -> Result<()> {
        if let Some(rule) = self.routing.get_mut(routing_id) {
            rule.usage_count += 1;
            self.save_routing()?;
        }
        Ok(())
    }

    /// Get statistics
    pub fn get_stats(&self) -> ProceduralStats {
        let avg_confidence = if self.rules.is_empty() {
            0.0
        } else {
            self.rules.values().map(|r| r.confidence).sum::<f32>() / self.rules.len() as f32
        };

        let categories: Vec<_> = self.patterns.values()
            .map(|p| p.category.clone())
            .collect::<std::collections::HashSet<_>>()
            .into_iter()
            .collect();

        ProceduralStats {
            total_rules: self.rules.len(),
            total_patterns: self.patterns.len(),
            total_routing: self.routing.len(),
            average_confidence: avg_confidence,
            categories,
            initialized: self.initialized,
        }
    }
}

// YAML file structures
#[derive(Serialize, Deserialize)]
struct RulesFile {
    rules: Vec<Rule>,
}

#[derive(Serialize, Deserialize)]
struct PatternsFile {
    patterns: Vec<Pattern>,
}

#[derive(Serialize, Deserialize)]
struct RoutingFile {
    routing: Vec<RoutingRule>,
}

/// A learned rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Rule {
    pub id: String,
    pub name: String,
    pub condition: String,
    pub action: String,
    pub confidence: f32,
    pub success_count: i32,
    pub failure_count: i32,
    pub created_at: String,
    pub updated_at: String,
}

/// A pattern for matching
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pattern {
    pub id: String,
    pub name: String,
    pub regex: String,
    pub category: String,
    pub priority: i32,
    pub metadata: Option<serde_json::Value>,
}

/// A routing rule
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRule {
    pub id: String,
    pub pattern_id: String,
    pub target_agent: String,
    pub confidence: f32,
    pub usage_count: i32,
}

/// A pattern match result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternMatch {
    pub pattern_id: String,
    pub pattern_name: String,
    pub category: String,
    pub priority: i32,
}

/// A routing recommendation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingRecommendation {
    pub target_agent: String,
    pub confidence: f32,
    pub pattern_name: String,
    pub routing_id: String,
}

/// Statistics for procedural memory
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralStats {
    pub total_rules: usize,
    pub total_patterns: usize,
    pub total_routing: usize,
    pub average_confidence: f32,
    pub categories: Vec<String>,
    pub initialized: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_procedural_memory() {
        let mem = ProceduralMemory::in_memory().await.unwrap();
        assert!(mem.initialized);
        assert!(!mem.rules.is_empty());
        assert!(!mem.patterns.is_empty());
    }

    #[tokio::test]
    async fn test_pattern_matching() {
        let mem = ProceduralMemory::in_memory().await.unwrap();

        let matches = mem.match_patterns("generate a function for sorting", None);
        assert!(!matches.is_empty());
        assert_eq!(matches[0].pattern_name, "code_generation");
    }

    #[tokio::test]
    async fn test_routing() {
        let mut mem = ProceduralMemory::in_memory().await.unwrap();
        mem.add_default_routing();

        let routing = mem.get_routing("create a new class for user management");
        assert!(routing.is_some());
        assert_eq!(routing.unwrap().target_agent, "code");
    }
}
