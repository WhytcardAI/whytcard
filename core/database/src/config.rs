//! Database configuration

use std::path::PathBuf;

/// Database configuration
#[derive(Debug, Clone)]
pub struct Config {
    /// Storage mode
    pub storage: StorageMode,

    /// Namespace name
    pub namespace: String,

    /// Database name
    pub database: String,

    /// Vector index configuration
    pub vector_config: VectorConfig,
}

/// Storage mode
#[derive(Debug, Clone)]
pub enum StorageMode {
    /// In-memory storage (data lost on restart)
    Memory,

    /// Persistent storage using RocksDB
    Persistent(PathBuf),
}

/// Vector index configuration
#[derive(Debug, Clone)]
pub struct VectorConfig {
    /// Embedding dimension (e.g., 384 for all-MiniLM-L6-v2)
    pub dimension: usize,

    /// Distance metric
    pub distance: DistanceMetric,
}

/// Distance metric for vector similarity
#[derive(Debug, Clone, Copy, Default)]
pub enum DistanceMetric {
    /// Cosine similarity (default)
    #[default]
    Cosine,

    /// Euclidean distance
    Euclidean,

    /// Manhattan distance
    Manhattan,
}

impl DistanceMetric {
    /// Get SurrealDB distance function name
    pub fn as_surreal_str(&self) -> &'static str {
        match self {
            Self::Cosine => "COSINE",
            Self::Euclidean => "EUCLIDEAN",
            Self::Manhattan => "MANHATTAN",
        }
    }
}

impl Default for Config {
    fn default() -> Self {
        Self {
            storage: StorageMode::Memory,
            namespace: "whytcard".to_string(),
            database: "main".to_string(),
            vector_config: VectorConfig::default(),
        }
    }
}

impl Default for VectorConfig {
    fn default() -> Self {
        Self {
            dimension: 384, // all-MiniLM-L6-v2
            distance: DistanceMetric::Cosine,
        }
    }
}

impl Config {
    /// Create a new in-memory configuration
    pub fn memory() -> Self {
        Self::default()
    }

    /// Create a new persistent configuration
    pub fn persistent(path: impl Into<PathBuf>) -> Self {
        Self {
            storage: StorageMode::Persistent(path.into()),
            ..Default::default()
        }
    }

    /// Set the namespace
    pub fn with_namespace(mut self, namespace: impl Into<String>) -> Self {
        self.namespace = namespace.into();
        self
    }

    /// Set the database name
    pub fn with_database(mut self, database: impl Into<String>) -> Self {
        self.database = database.into();
        self
    }

    /// Set vector dimension
    pub fn with_dimension(mut self, dimension: usize) -> Self {
        self.vector_config.dimension = dimension;
        self
    }

    /// Set distance metric
    pub fn with_distance(mut self, distance: DistanceMetric) -> Self {
        self.vector_config.distance = distance;
        self
    }
}
