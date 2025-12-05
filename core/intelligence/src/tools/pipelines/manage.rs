//! Admin - Manage Pipeline
//!
//! MCP server administration.
//! Combines all mcp_* tools: status, install, uninstall, configure, connect, disconnect, list
//!
//! This pipeline is for administrative tasks, not part of the ACID workflow.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Management action to perform
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ManageAction {
    /// Get status of all MCP servers
    Status,
    /// List available predefined servers
    ListAvailable,
    /// List installed servers
    ListInstalled,
    /// List tools from connected servers
    ListTools,
    /// Install a new MCP server
    Install,
    /// Uninstall an MCP server
    Uninstall,
    /// Configure an MCP server
    Configure,
    /// Connect to an MCP server
    Connect,
    /// Disconnect from an MCP server
    Disconnect,
    /// Call a tool on a connected server
    CallTool,
    /// Get CORTEX stats
    CortexStats,
    /// Cleanup old CORTEX data
    CortexCleanup,
    /// Get/reload instructions (legacy, use InstructionsList/InstructionsReload)
    Instructions,
    /// List all loaded instructions
    InstructionsList,
    /// Reload instructions from workspace
    InstructionsReload,
}

/// Server installation parameters
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InstallConfig {
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
    pub env: HashMap<String, String>,
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

/// Server configuration changes
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ConfigureConfig {
    /// Server name to configure
    pub name: String,
    /// Set environment variable
    #[serde(default)]
    pub set_env: Option<(String, String)>,
    /// Remove environment variable by key
    #[serde(default)]
    pub remove_env: Option<String>,
    /// Enable/disable the server
    #[serde(default)]
    pub enable: Option<bool>,
    /// Set auto-connect on startup
    #[serde(default)]
    pub auto_connect: Option<bool>,
    /// Update description
    #[serde(default)]
    pub description: Option<String>,
}

/// Tool call parameters
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolCallConfig {
    /// Server name
    pub server: String,
    /// Tool name
    pub tool: String,
    /// Arguments as JSON
    #[serde(default)]
    pub arguments: Option<serde_json::Value>,
}

/// Instructions action
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InstructionsAction {
    /// List all loaded instructions
    List,
    /// Reload from workspace
    Reload,
    /// Get specific instruction by name
    Get,
    /// Get instructions for a file path
    ForFile,
}

/// Instructions config
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InstructionsConfig {
    /// Action to perform
    pub action: InstructionsAction,
    /// Instruction name (for Get action)
    #[serde(default)]
    pub name: Option<String>,
    /// File path (for ForFile action)
    #[serde(default)]
    pub file_path: Option<String>,
}

/// Parameters for the manage pipeline
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ManageParams {
    /// Action to perform
    pub action: ManageAction,

    /// Server name (for connect, disconnect, uninstall)
    #[serde(default)]
    pub server: Option<String>,

    /// Installation config (for install action)
    #[serde(default)]
    pub install: Option<InstallConfig>,

    /// Configuration changes (for configure action)
    #[serde(default)]
    pub configure: Option<ConfigureConfig>,

    /// Tool call config (for call_tool action)
    #[serde(default)]
    pub tool_call: Option<ToolCallConfig>,

    /// Filter by server name (for list_tools)
    #[serde(default)]
    pub filter_server: Option<String>,

    /// Filter by tool name pattern (for list_tools)
    #[serde(default)]
    pub filter_tool: Option<String>,

    /// Retention days (for cortex_cleanup)
    #[serde(default = "default_retention")]
    pub retention_days: i64,

    /// Instructions config (for instructions action)
    #[serde(default)]
    pub instructions: Option<InstructionsConfig>,
}

fn default_retention() -> i64 {
    30
}

/// Server status info
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ServerInfo {
    /// Server name
    pub name: String,
    /// Connection status
    pub status: String,
    /// Number of tools available
    pub tool_count: usize,
    /// Package name
    pub package: Option<String>,
    /// Description
    pub description: Option<String>,
    /// Is enabled
    pub enabled: bool,
    /// Auto-connect on startup
    pub auto_connect: bool,
}

/// Tool info
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolInfoItem {
    /// Tool name
    pub name: String,
    /// Server providing this tool
    pub server: String,
    /// Description
    pub description: Option<String>,
}

/// CORTEX stats
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CortexStatsInfo {
    /// Semantic memory facts
    pub semantic_facts: usize,
    /// Episodic memory events
    pub episodic_events: usize,
    /// Procedural memory rules
    pub procedural_rules: usize,
    /// Engine status
    pub status: String,
}

/// Instruction info
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct InstructionInfoItem {
    /// Instruction name
    pub name: String,
    /// Description
    pub description: Option<String>,
    /// ApplyTo pattern
    pub apply_to: Option<String>,
}

/// Result from the manage pipeline
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ManageResult {
    /// Action that was performed
    pub action: String,

    /// Whether action succeeded
    pub success: bool,

    /// Message
    pub message: String,

    /// Server list (for status, list_available, list_installed)
    #[serde(default)]
    pub servers: Vec<ServerInfo>,

    /// Tool list (for list_tools)
    #[serde(default)]
    pub tools: Vec<ToolInfoItem>,

    /// CORTEX stats (for cortex_stats)
    #[serde(default)]
    pub cortex_stats: Option<CortexStatsInfo>,

    /// Cleanup count (for cortex_cleanup)
    #[serde(default)]
    pub cleaned_count: Option<usize>,

    /// Tool call result (for call_tool)
    #[serde(default)]
    pub tool_result: Option<serde_json::Value>,

    /// Instructions list (for instructions action)
    #[serde(default)]
    pub instructions: Vec<InstructionInfoItem>,

    /// Instruction content (for instructions get action)
    #[serde(default)]
    pub instruction_content: Option<String>,

    /// Connected server count
    pub connected_count: usize,

    /// Error details if failed
    #[serde(default)]
    pub error: Option<String>,
}

impl Default for ManageParams {
    fn default() -> Self {
        Self {
            action: ManageAction::Status,
            server: None,
            install: None,
            configure: None,
            tool_call: None,
            filter_server: None,
            filter_tool: None,
            retention_days: 30,
            instructions: None,
        }
    }
}

impl ManageParams {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manage_params_status() {
        let params = ManageParams::status();
        assert!(matches!(params.action, ManageAction::Status));
    }

    #[test]
    fn test_manage_params_connect() {
        let params = ManageParams::connect("tavily");
        assert!(matches!(params.action, ManageAction::Connect));
        assert_eq!(params.server, Some("tavily".to_string()));
    }

    #[test]
    fn test_manage_params_install() {
        let config = InstallConfig {
            name: "test-server".to_string(),
            package: "@test/mcp".to_string(),
            package_type: "npm".to_string(),
            description: "Test server".to_string(),
            env: HashMap::new(),
            auto_connect: true,
            connect_now: true,
        };

        let params = ManageParams::install(config);
        assert!(matches!(params.action, ManageAction::Install));
        assert!(params.install.is_some());
    }

    #[test]
    fn test_manage_result() {
        let result = ManageResult {
            action: "status".to_string(),
            success: true,
            message: "3 servers connected".to_string(),
            servers: vec![ServerInfo {
                name: "tavily".to_string(),
                status: "connected".to_string(),
                tool_count: 4,
                package: Some("@tavily/mcp".to_string()),
                description: Some("Web search".to_string()),
                enabled: true,
                auto_connect: true,
            }],
            tools: vec![],
            cortex_stats: None,
            cleaned_count: None,
            tool_result: None,
            instructions: vec![],
            instruction_content: None,
            connected_count: 3,
            error: None,
        };

        assert!(result.success);
        assert_eq!(result.servers.len(), 1);
        assert_eq!(result.connected_count, 3);
    }

    #[test]
    fn test_manage_params_cortex_cleanup() {
        let params = ManageParams::cortex_cleanup(7);
        assert!(matches!(params.action, ManageAction::CortexCleanup));
        assert_eq!(params.retention_days, 7);
    }

    #[test]
    fn test_manage_params_instructions() {
        let params = ManageParams::list_instructions();
        assert!(matches!(params.action, ManageAction::Instructions));
        assert!(params.instructions.is_some());
    }
}
