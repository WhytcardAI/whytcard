//! MCP Client Module
//!
//! Provides clients for connecting to external MCP servers:
//! - Sequential Thinking for complex problem decomposition
//! - Context7 for library documentation (MCP protocol)
//! - Tavily for web search (MCP protocol)
//! - Microsoft Learn for MS/Azure docs (MCP protocol)
//! - Playwright for browser automation

pub mod config;
pub mod manager;
pub mod sequential_thinking;
pub mod types;

pub use config::{InstalledMcpServer, McpConfigManager, McpServersConfig};
pub use manager::McpClientManager;
pub use sequential_thinking::SequentialThinkingClient;
pub use types::*;
