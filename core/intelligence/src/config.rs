//! Configuration for WhytCard Intelligence

use crate::paths::DataPaths;
use serde::{Deserialize, Serialize};
use std::path::Path;
use whytcard_rag::{ChunkingConfig, EmbeddingModel, SearchConfig};

/// Main configuration for Intelligence server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IntelligenceConfig {
    /// Server name for MCP
    #[serde(default = "default_server_name")]
    pub server_name: String,

    /// Server version
    #[serde(default = "default_version")]
    pub version: String,

    /// Namespace for data isolation (allows multiple instances)
    /// Each namespace gets its own subdirectory for data storage
    #[serde(default)]
    pub namespace: Option<String>,

    /// Data paths (optional, resolved at runtime if not set)
    #[serde(skip)]
    pub paths: Option<DataPaths>,

    /// RAG configuration
    #[serde(default)]
    pub rag: RagSettings,

    /// Memory settings
    #[serde(default)]
    pub memory: MemorySettings,

    /// Knowledge graph settings
    #[serde(default)]
    pub knowledge: KnowledgeSettings,
}

/// RAG-specific settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RagSettings {
    /// Embedding model to use
    #[serde(default)]
    pub model: EmbeddingModel,

    /// Chunking configuration
    #[serde(default)]
    pub chunking: ChunkingConfig,

    /// Search configuration
    #[serde(default)]
    pub search: SearchConfig,

    /// Auto-index new memories
    #[serde(default = "default_true")]
    pub auto_index: bool,
}

/// Memory storage settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemorySettings {
    /// Maximum memory entries (0 = unlimited)
    #[serde(default)]
    pub max_entries: usize,

    /// Enable semantic search for memories
    #[serde(default = "default_true")]
    pub semantic_search: bool,

    /// Auto-generate titles if not provided
    #[serde(default = "default_true")]
    pub auto_title: bool,
}

/// Knowledge graph settings
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct KnowledgeSettings {
    /// Maximum entities (0 = unlimited)
    #[serde(default)]
    pub max_entities: usize,

    /// Maximum relations per entity (0 = unlimited)
    #[serde(default)]
    pub max_relations_per_entity: usize,

    /// Enable relation type validation
    #[serde(default)]
    pub strict_relation_types: bool,

    /// Allowed relation types (empty = all allowed)
    #[serde(default)]
    pub allowed_relation_types: Vec<String>,
}

// Default value helpers
fn default_server_name() -> String {
    "whytcard-intelligence".to_string()
}

fn default_version() -> String {
    env!("CARGO_PKG_VERSION").to_string()
}

fn default_true() -> bool {
    true
}

impl Default for IntelligenceConfig {
    fn default() -> Self {
        Self {
            server_name: default_server_name(),
            version: default_version(),
            namespace: None,
            paths: None,
            rag: RagSettings::default(),
            memory: MemorySettings::default(),
            knowledge: KnowledgeSettings::default(),
        }
    }
}

impl Default for RagSettings {
    fn default() -> Self {
        Self {
            model: EmbeddingModel::default(),
            chunking: ChunkingConfig::default(),
            search: SearchConfig::default(),
            auto_index: true,
        }
    }
}

impl Default for MemorySettings {
    fn default() -> Self {
        Self {
            max_entries: 0, // unlimited
            semantic_search: true,
            auto_title: true,
        }
    }
}

impl IntelligenceConfig {
    /// Create config with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set namespace for data isolation
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = Some(namespace.into());
        self
    }

    /// Load config from file or create default
    pub fn load_or_default(paths: &DataPaths) -> Self {
        Self::load(&paths.config).unwrap_or_default()
    }

    /// Load config from TOML file
    pub fn load(path: &Path) -> Result<Self, ConfigError> {
        let content = std::fs::read_to_string(path).map_err(ConfigError::Io)?;
        toml::from_str(&content).map_err(ConfigError::Parse)
    }

    /// Save config to TOML file
    pub fn save(&self, path: &Path) -> Result<(), ConfigError> {
        let content = toml::to_string_pretty(self).map_err(ConfigError::Serialize)?;
        std::fs::write(path, content).map_err(ConfigError::Io)?;
        Ok(())
    }

    /// Set data paths
    pub fn with_paths(mut self, paths: DataPaths) -> Self {
        self.paths = Some(paths);
        self
    }

    /// Get paths, resolving if not set
    pub fn get_paths(&self) -> crate::Result<DataPaths> {
        match &self.paths {
            Some(p) => Ok(p.clone()),
            None => {
                let base_paths = DataPaths::resolve()?;
                // Apply namespace if set
                match &self.namespace {
                    Some(ns) => Ok(base_paths.with_namespace(ns)),
                    None => Ok(base_paths),
                }
            }
        }
    }
}

/// Configuration errors
#[derive(Debug, thiserror::Error)]
pub enum ConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Parse error: {0}")]
    Parse(#[from] toml::de::Error),

    #[error("Serialize error: {0}")]
    Serialize(#[from] toml::ser::Error),
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_default_config() {
        let config = IntelligenceConfig::default();
        assert_eq!(config.server_name, "whytcard-intelligence");
        assert!(config.rag.auto_index);
        assert!(config.memory.semantic_search);
    }

    #[test]
    fn test_save_and_load() {
        let temp = TempDir::new().unwrap();
        let path = temp.path().join("config.toml");

        let config = IntelligenceConfig::default();
        config.save(&path).unwrap();

        let loaded = IntelligenceConfig::load(&path).unwrap();
        assert_eq!(loaded.server_name, config.server_name);
    }

    #[test]
    fn test_toml_serialization() {
        let config = IntelligenceConfig::default();
        let toml_str = toml::to_string_pretty(&config).unwrap();

        assert!(toml_str.contains("server_name"));
        assert!(toml_str.contains("whytcard-intelligence"));
    }
}
