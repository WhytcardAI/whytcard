//! External MCP Integration Tools
//!
//! Tools for calling external MCP servers:
//! - sequential_thinking: Complex problem decomposition
//! - external_docs: Get documentation from Context7
//! - external_search: Web search via Tavily
//! - external_call: Generic tool call to any connected MCP server

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// =============================================================================
// Sequential Thinking Tool
// =============================================================================

/// Parameters for sequential thinking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SequentialThinkingParams {
    /// The problem or question to analyze
    pub problem: String,

    /// Estimated number of thinking steps needed (default: 5)
    #[serde(default = "default_steps")]
    pub estimated_steps: u32,

    /// Whether to use external MCP server if available
    #[serde(default)]
    pub use_external: bool,
}

fn default_steps() -> u32 {
    5
}

/// Result from sequential thinking
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SequentialThinkingResult {
    /// All thinking steps
    pub steps: Vec<ThinkingStep>,

    /// Final conclusion
    pub conclusion: Option<String>,

    /// Whether analysis is complete
    pub complete: bool,

    /// Source (internal or external MCP)
    pub source: String,
}

/// A single thinking step
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ThinkingStep {
    /// Step number
    pub number: u32,

    /// Step content
    pub content: String,

    /// Whether this was a revision
    #[serde(default)]
    pub is_revision: bool,
}

// =============================================================================
// External Documentation Tool
// =============================================================================

/// Parameters for fetching external documentation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExternalDocsParams {
    /// Library name (e.g., "react", "axum", "tokio")
    pub library: String,

    /// Specific topic to focus on (e.g., "hooks", "routing")
    #[serde(default)]
    pub topic: Option<String>,

    /// Maximum tokens to retrieve (default: 5000)
    #[serde(default = "default_tokens")]
    pub max_tokens: u32,

    /// Source preference: "context7", "mslearn", "auto"
    #[serde(default = "default_source")]
    pub source: String,
}

fn default_tokens() -> u32 {
    5000
}

fn default_source() -> String {
    "auto".to_string()
}

/// Result from external documentation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExternalDocsResult {
    /// Library identifier
    pub library: String,

    /// Topic searched
    pub topic: Option<String>,

    /// Documentation content
    pub content: String,

    /// Code snippets extracted
    pub code_snippets: Vec<String>,

    /// Source URL
    pub url: Option<String>,

    /// Provider name
    pub provider: String,
}

// =============================================================================
// External Search Tool
// =============================================================================

/// Parameters for external web search
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExternalSearchParams {
    /// Search query
    pub query: String,

    /// Maximum results (default: 10)
    #[serde(default = "default_max_results")]
    pub max_results: u32,

    /// Search type: "general", "news"
    #[serde(default = "default_search_type")]
    pub search_type: String,

    /// Domains to include (optional)
    #[serde(default)]
    pub include_domains: Vec<String>,

    /// Domains to exclude (optional)
    #[serde(default)]
    pub exclude_domains: Vec<String>,
}

fn default_max_results() -> u32 {
    10
}

fn default_search_type() -> String {
    "general".to_string()
}

/// Result from external search
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExternalSearchResult {
    /// Search query
    pub query: String,

    /// Search results
    pub results: Vec<SearchResultItem>,

    /// Provider name
    pub provider: String,

    /// Total results found
    pub total: usize,
}

/// A single search result
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SearchResultItem {
    /// Result title
    pub title: String,

    /// Content snippet
    pub content: String,

    /// Source URL
    pub url: Option<String>,

    /// Relevance score
    pub score: f32,
}

// =============================================================================
// Generic External MCP Tool Call
// =============================================================================

/// Parameters for calling any external MCP tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExternalMcpCallParams {
    /// Server name (e.g., "context7", "tavily", "sequential-thinking")
    pub server: String,

    /// Tool name to call
    pub tool: String,

    /// Arguments as JSON
    #[serde(default)]
    pub arguments: Option<serde_json::Value>,
}

/// Result from external MCP call
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExternalMcpCallResult {
    /// Server that handled the call
    pub server: String,

    /// Tool that was called
    pub tool: String,

    /// Whether the call succeeded
    pub success: bool,

    /// Result content
    pub content: String,

    /// Structured data if available
    pub data: Option<serde_json::Value>,

    /// Error message if failed
    pub error: Option<String>,
}

// =============================================================================
// MCP Status Tool
// =============================================================================

/// Parameters for getting MCP status (no params)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpStatusParams {}

/// Result showing MCP connection status
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpStatusResult {
    /// Status of each configured server
    pub servers: Vec<McpServerStatus>,

    /// Available tools across all servers
    pub available_tools: Vec<ToolInfo>,

    /// Total connected servers
    pub connected_count: usize,
}

/// Status of a single MCP server
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpServerStatus {
    /// Server name
    pub name: String,

    /// Connection status
    pub status: String,

    /// Number of tools available
    pub tool_count: usize,
}

/// Brief tool information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolInfo {
    /// Tool name
    pub name: String,

    /// Server providing this tool
    pub server: String,

    /// Tool description
    pub description: Option<String>,
}

// =============================================================================
// MCP Connect Tool
// =============================================================================

/// Parameters for connecting to an MCP server
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpConnectParams {
    /// Server name to connect (predefined or custom)
    pub server: String,

    /// Custom server configuration (optional, for non-predefined servers)
    #[serde(default)]
    pub custom_config: Option<CustomServerConfig>,
}

/// Custom server configuration for non-predefined servers
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CustomServerConfig {
    /// Transport type: "stdio", "sse", "http"
    pub transport: String,

    /// Command for stdio (e.g., "npx", "uvx")
    #[serde(default)]
    pub command: Option<String>,

    /// Arguments for stdio
    #[serde(default)]
    pub args: Vec<String>,

    /// URL for SSE/HTTP transports
    #[serde(default)]
    pub url: Option<String>,

    /// Environment variables
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,
}

/// Result from connecting to MCP server
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpConnectResult {
    /// Server name
    pub server: String,

    /// Whether connection succeeded
    pub connected: bool,

    /// Available tools on this server
    pub tools: Vec<String>,

    /// Error message if connection failed
    pub error: Option<String>,
}

// =============================================================================
// MCP Disconnect Tool
// =============================================================================

/// Parameters for disconnecting from an MCP server
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpDisconnectParams {
    /// Server name to disconnect
    pub server: String,
}

/// Result from disconnecting
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpDisconnectResult {
    /// Server name
    pub server: String,

    /// Whether disconnection succeeded
    pub disconnected: bool,

    /// Error message if failed
    pub error: Option<String>,
}

// =============================================================================
// MCP List Tools
// =============================================================================

/// Parameters for listing available MCP tools
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpListToolsParams {
    /// Filter by server name (optional, all if not specified)
    #[serde(default)]
    pub server: Option<String>,

    /// Filter by tool name pattern (optional)
    #[serde(default)]
    pub name_pattern: Option<String>,
}

/// Result listing available MCP tools
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpListToolsResult {
    /// Total tools available
    pub total: usize,

    /// Tools grouped by server
    pub tools_by_server: std::collections::HashMap<String, Vec<McpToolDetail>>,
}

/// Detailed tool information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpToolDetail {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: Option<String>,

    /// Input schema (JSON Schema)
    pub input_schema: Option<serde_json::Value>,
}

// =============================================================================
// MCP Available Servers Tool
// =============================================================================

/// Parameters for listing available predefined servers
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpAvailableServersParams {}

/// Result listing available predefined servers
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpAvailableServersResult {
    /// Servers that don't require API keys
    pub free_servers: Vec<ServerDescription>,

    /// Servers that require API keys (with env var name)
    pub key_required_servers: Vec<KeyRequiredServer>,

    /// Currently connected servers
    pub connected: Vec<String>,
}

/// Description of a predefined server
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServerDescription {
    /// Server name
    pub name: String,

    /// Description of what it does
    pub description: String,
}

/// Server that requires an API key
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KeyRequiredServer {
    /// Server name
    pub name: String,

    /// Environment variable name for the API key
    pub env_var: String,

    /// Whether the API key is currently set
    pub key_present: bool,
}

// =============================================================================
// MCP Install Tool
// =============================================================================

/// Parameters for installing an MCP server
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpInstallParams {
    /// Server name (unique identifier)
    pub name: String,

    /// Package name (npm: @org/package, pip: package-name)
    pub package: String,

    /// Package type: "npm", "pip", "binary"
    #[serde(default = "default_npm")]
    pub package_type: String,

    /// Human-readable description
    #[serde(default)]
    pub description: String,

    /// Environment variables (for API keys, etc.)
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,

    /// Auto-connect on startup
    #[serde(default)]
    pub auto_connect: bool,

    /// Connect immediately after installation
    #[serde(default = "default_true")]
    pub connect_now: bool,
}

fn default_npm() -> String {
    "npm".to_string()
}

fn default_true() -> bool {
    true
}

/// Result from installing MCP server
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpInstallResult {
    /// Server name
    pub name: String,

    /// Whether installation succeeded
    pub installed: bool,

    /// Whether connection succeeded (if connect_now was true)
    pub connected: bool,

    /// Available tools (if connected)
    pub tools: Vec<String>,

    /// Error message if failed
    pub error: Option<String>,
}

// =============================================================================
// MCP Uninstall Tool
// =============================================================================

/// Parameters for uninstalling an MCP server
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpUninstallParams {
    /// Server name to uninstall
    pub name: String,

    /// Disconnect if currently connected
    #[serde(default = "default_true")]
    pub disconnect: bool,
}

/// Result from uninstalling MCP server
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpUninstallResult {
    /// Server name
    pub name: String,

    /// Whether uninstallation succeeded
    pub uninstalled: bool,

    /// Error message if failed
    pub error: Option<String>,
}

// =============================================================================
// MCP Configure Tool
// =============================================================================

/// Parameters for configuring an MCP server
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpConfigureParams {
    /// Server name to configure
    pub name: String,

    /// Set environment variable (key-value pair)
    #[serde(default)]
    pub set_env: Option<SetEnvParam>,

    /// Remove environment variable by key
    #[serde(default)]
    pub remove_env: Option<String>,

    /// Enable the server
    #[serde(default)]
    pub enable: Option<bool>,

    /// Set auto-connect on startup
    #[serde(default)]
    pub auto_connect: Option<bool>,

    /// Update description
    #[serde(default)]
    pub description: Option<String>,
}

/// Key-value pair for setting environment variable
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct SetEnvParam {
    /// Environment variable key
    pub key: String,

    /// Environment variable value
    pub value: String,
}

/// Result from configuring MCP server
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpConfigureResult {
    /// Server name
    pub name: String,

    /// Whether configuration succeeded
    pub configured: bool,

    /// Current configuration after changes
    pub current_config: Option<McpServerInfo>,

    /// Error message if failed
    pub error: Option<String>,
}

/// Current server configuration info
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpServerInfo {
    /// Server name
    pub name: String,

    /// Package name
    pub package: String,

    /// Package type
    pub package_type: String,

    /// Description
    pub description: String,

    /// Is enabled
    pub enabled: bool,

    /// Auto-connect on startup
    pub auto_connect: bool,

    /// Environment variable keys (values hidden)
    pub env_keys: Vec<String>,

    /// Installation timestamp
    pub installed_at: Option<String>,

    /// Last connection timestamp
    pub last_connected: Option<String>,
}

// =============================================================================
// MCP List Installed Tool
// =============================================================================

/// Parameters for listing installed MCP servers
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpListInstalledParams {
    /// Include disabled servers
    #[serde(default = "default_true")]
    pub include_disabled: bool,
}

/// Result listing installed MCP servers
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct McpListInstalledResult {
    /// Total installed servers
    pub total: usize,

    /// Installed servers
    pub servers: Vec<McpServerInfo>,
}
