//! ACID Pipeline Tools for WhytCard Intelligence
//!
//! These pipelines replace the 39 atomic tools with 6 workflow-based tools
//! aligned with the ACID methodology from workflows.instructions.md:
//!
//! - A (Analyse): `analyze` - Research and understand before coding
//! - B (Best Practices): `prepare` - Document decisions before coding
//! - C (Code): `code` - Execute and verify code
//! - I (Integrate): `verify` - Validate completely
//! - D (Document): `document` - Trace and learn
//! - Admin: `manage` - MCP server administration

pub mod analyze;
pub mod code;
pub mod document;
pub mod manage;
pub mod prepare;
pub mod verify;

pub use analyze::*;
pub use code::*;
pub use document::*;
pub use manage::*;
pub use prepare::*;
pub use verify::*;

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Common response wrapper for all pipelines
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PipelineResponse<T> {
    /// Whether the pipeline executed successfully
    pub success: bool,

    /// The pipeline result data
    pub data: T,

    /// Execution time in milliseconds
    pub duration_ms: u64,

    /// Any warnings encountered
    #[serde(default)]
    pub warnings: Vec<String>,

    /// Suggested next pipeline to call
    #[serde(default)]
    pub next_pipeline: Option<String>,
}

impl<T> PipelineResponse<T> {
    pub fn ok(data: T, duration_ms: u64) -> Self {
        Self {
            success: true,
            data,
            duration_ms,
            warnings: Vec::new(),
            next_pipeline: None,
        }
    }

    pub fn ok_with_next(data: T, duration_ms: u64, next: &str) -> Self {
        Self {
            success: true,
            data,
            duration_ms,
            warnings: Vec::new(),
            next_pipeline: Some(next.to_string()),
        }
    }

    pub fn with_warning(mut self, warning: impl Into<String>) -> Self {
        self.warnings.push(warning.into());
        self
    }
}
