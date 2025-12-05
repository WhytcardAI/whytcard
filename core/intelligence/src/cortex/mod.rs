//! CORTEX Engine - Cognitive ORchestration for Task EXecution
//!
//! A unified intelligent system that:
//! - Perceives and understands queries
//! - Retrieves relevant memory and researches when needed
//! - Executes with adaptation using OODA loops
//! - Reflects and learns from every interaction
//!
//! Unlike traditional multi-agent MCP systems, CORTEX uses a single entry point
//! that discovers, learns, and adapts continuously.

pub mod engine;
pub mod instructions;
mod perceiver;
mod executor;
mod learner;
mod context;

pub use engine::{CortexEngine, CortexResult};
// instructions module re-exports types used internally by CortexEngine

/// Configuration for the CORTEX engine
#[derive(Debug, Clone)]
pub struct CortexConfig {
    /// Confidence threshold below which research is triggered
    pub research_threshold: f32,

    /// Maximum execution steps before forced stop
    pub max_execution_steps: usize,

    /// Enable automatic learning after execution
    pub auto_learn: bool,

    /// Enable research pipeline
    pub enable_research: bool,
}

impl Default for CortexConfig {
    fn default() -> Self {
        Self {
            research_threshold: 0.7,
            max_execution_steps: 20,
            auto_learn: true,
            enable_research: true,
        }
    }
}
