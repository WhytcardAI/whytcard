//! Triple Memory System for CORTEX
//!
//! Implements the three-layer memory architecture:
//! - Semantic Memory: Facts, knowledge, patterns (vector-based)
//! - Episodic Memory: Events, interactions, temporal history
//! - Procedural Memory: Learned workflows, rules, routing

pub mod semantic;
pub mod episodic;
pub mod procedural;

pub use semantic::*;
pub use episodic::*;
pub use procedural::*;

use crate::error::Result;
use crate::paths::DataPaths;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Unified interface to the Triple Memory System
#[derive(Clone)]
pub struct TripleMemory {
    /// Semantic memory (facts, knowledge - vector search)
    pub semantic: Arc<RwLock<SemanticMemory>>,

    /// Episodic memory (events, interactions - temporal)
    pub episodic: Arc<RwLock<EpisodicMemory>>,

    /// Procedural memory (rules, patterns, routing)
    pub procedural: Arc<RwLock<ProceduralMemory>>,
}

impl TripleMemory {
    /// Create a new Triple Memory system
    pub async fn new(paths: &DataPaths) -> Result<Self> {
        tracing::info!("Initializing Triple Memory System");

        // Initialize semantic memory (vectors) - uses cortex_memory to avoid conflict with RAG
        let semantic_path = paths.cortex_memory.join("semantic");
        std::fs::create_dir_all(&semantic_path)?;
        let semantic = SemanticMemory::new(&semantic_path).await?;
        tracing::debug!("Semantic memory initialized at {:?}", semantic_path);

        // Initialize episodic memory (SQLite)
        let episodic_path = paths.cortex_memory.join("episodic.db");
        let episodic = EpisodicMemory::new(&episodic_path).await?;
        tracing::debug!("Episodic memory initialized at {:?}", episodic_path);

        // Initialize procedural memory (YAML rules)
        let procedural_path = paths.cortex_memory.join("procedures");
        let procedural = ProceduralMemory::new(&procedural_path).await?;
        tracing::debug!("Procedural memory initialized at {:?}", procedural_path);

        Ok(Self {
            semantic: Arc::new(RwLock::new(semantic)),
            episodic: Arc::new(RwLock::new(episodic)),
            procedural: Arc::new(RwLock::new(procedural)),
        })
    }

    /// Create in-memory Triple Memory for testing
    #[cfg(test)]
    pub async fn for_testing() -> Result<Self> {
        let semantic = SemanticMemory::in_memory().await?;
        let episodic = EpisodicMemory::in_memory().await?;
        let procedural = ProceduralMemory::in_memory().await?;

        Ok(Self {
            semantic: Arc::new(RwLock::new(semantic)),
            episodic: Arc::new(RwLock::new(episodic)),
            procedural: Arc::new(RwLock::new(procedural)),
        })
    }

    /// Get statistics from all memory types
    pub async fn get_stats(&self) -> MemoryStats {
        let semantic = self.semantic.read().await;
        let episodic = self.episodic.read().await;
        let procedural = self.procedural.read().await;

        MemoryStats {
            semantic: semantic.get_stats().await,
            episodic: episodic.get_stats().await,
            procedural: procedural.get_stats(),
        }
    }
}

/// Combined statistics from all memory types
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MemoryStats {
    pub semantic: SemanticStats,
    pub episodic: EpisodicStats,
    pub procedural: ProceduralStats,
}
