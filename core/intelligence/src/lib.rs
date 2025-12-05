//! WhytCard Intelligence MCP Server
//!
//! A Model Context Protocol server providing:
//! - Triple Memory System (Semantic, Episodic, Procedural)
//! - CORTEX Engine (Perceive, Execute, Learn)
//! - Knowledge graph management
//! - Semantic search (via RAG)
//!
//! # Architecture
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────┐
//! │                    CORTEX ENGINE                         │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
//! │  │ PERCEIVE │─▶│ EXECUTE  │─▶│  LEARN   │              │
//! │  └──────────┘  └──────────┘  └──────────┘              │
//! └─────────────────────────────────────────────────────────┘
//! ┌─────────────────────────────────────────────────────────┐
//! │                  TRIPLE MEMORY                           │
//! │  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
//! │  │ SEMANTIC │  │ EPISODIC │  │PROCEDURAL│              │
//! │  │ (Vectors)│  │ (Events) │  │ (Rules)  │              │
//! │  └──────────┘  └──────────┘  └──────────┘              │
//! └─────────────────────────────────────────────────────────┘
//! ```
//!
//! # Usage
//!
//! ## As MCP Server (stdio transport)
//!
//! ```bash
//! whytcard-intelligence
//! ```
//!
//! ## With custom data directory
//!
//! ```bash
//! WHYTCARD_DATA_DIR=/path/to/data whytcard-intelligence
//! ```
//!
//! # MCP Tools
//!
//! ## Memory Tools
//! - `memory_store`: Store information with optional semantic indexing
//! - `memory_search`: Semantic search across all stored information
//! - `memory_get`: Retrieve by key
//! - `memory_delete`: Delete by key
//!
//! ## Knowledge Tools
//! - `knowledge_add_entity`: Add entity to knowledge graph
//! - `knowledge_add_relation`: Create relation between entities
//! - `knowledge_search`: Query knowledge graph
//!
//! ## CORTEX Tools
//! - `cortex_process`: Main entry point - Perceive, Execute, Learn pipeline

mod config;
mod cortex;
mod error;
pub mod integrations;
mod memory;
pub mod mcp_client;
mod paths;
mod server;
mod tools;

pub use config::IntelligenceConfig;
pub use cortex::{CortexEngine, CortexConfig, CortexResult};
pub use error::{IntelligenceError, Result};
pub use integrations::{IntegrationHub, Context7Client, TavilyClient, MSLearnClient};
pub use mcp_client::{McpClientManager, McpToolResult, McpServerConfig, SequentialThinkingClient};
pub use memory::{TripleMemory, MemoryStats};
pub use paths::DataPaths;
pub use server::IntelligenceServer;
pub use tools::cortex::{init_cortex, cortex_process, cortex_feedback, cortex_stats, cortex_cleanup};
