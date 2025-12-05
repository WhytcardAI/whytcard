//! MCP Tools for WhytCard Intelligence
//!
//! This module contains two categories of tools:
//!
//! ## ACID Pipeline Tools (Recommended)
//! 6 workflow-based pipelines aligned with the ACID methodology:
//! - `analyze` - Phase A: Research and understand
//! - `prepare` - Phase B: Document decisions
//! - `code` - Phase C: Execute and verify
//! - `verify` - Phase I: Validate completely
//! - `document` - Phase D: Trace and learn
//! - `manage` - Admin: MCP server administration
//!
//! ## Atomic Tools (Internal)
//! The original 39 atomic tools are kept for internal use by pipelines.
//! Direct usage is discouraged - use pipelines instead.

// ACID Pipeline Tools (public API)
// Export module but NOT with glob to avoid conflicts
pub mod pipelines;

// Atomic Tools (internal implementation)
pub mod cortex;
pub mod external;
pub mod knowledge;
pub mod memory;

// Re-export atomic tools for internal use
pub use cortex::*;
pub use external::*;
pub use knowledge::*;
pub use memory::*;
