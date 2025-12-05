//! MCP Client Manager
//!
//! Manages connections to multiple external MCP servers and provides
//! unified interface for calling their tools.

use super::types::*;
use crate::error::{IntelligenceError, Result};
use rmcp::{
    model::{CallToolRequestParam, ClientCapabilities, ClientInfo, Implementation},
    service::ServiceExt,
    transport::TokioChildProcess,
};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::process::Command;
use tokio::sync::RwLock;

/// Type alias for a connected MCP client - using InitializeRequestParam as peer data
type McpClient = rmcp::service::RunningService<rmcp::RoleClient, rmcp::model::InitializeRequestParam>;

/// Manager for multiple MCP client connections
pub struct McpClientManager {
    /// Connected clients by server name
    clients: Arc<RwLock<HashMap<String, ClientConnection>>>,

    /// Server configurations (now dynamic with RwLock)
    configs: Arc<RwLock<HashMap<String, McpServerConfig>>>,

    /// Available tools cache (server -> tools)
    tools_cache: Arc<RwLock<HashMap<String, Vec<McpToolInfo>>>>,
}

/// A connected MCP client with metadata
struct ClientConnection {
    /// The actual MCP client
    client: McpClient,

    /// Connection status
    status: McpClientStatus,
}

impl McpClientManager {
    /// Create a new manager
    pub fn new() -> Self {
        Self {
            clients: Arc::new(RwLock::new(HashMap::new())),
            configs: Arc::new(RwLock::new(HashMap::new())),
            tools_cache: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Create manager with predefined servers
    pub async fn with_defaults() -> Self {
        let manager = Self::new();

        // Add common MCP servers
        manager.add_config(PredefinedServers::sequential_thinking()).await;
        manager.add_config(PredefinedServers::context7()).await;
        manager.add_config(PredefinedServers::memory()).await;

        // Add tavily if API key is available
        if let Ok(api_key) = std::env::var("TAVILY_API_KEY") {
            manager.add_config(PredefinedServers::tavily(&api_key)).await;
        }

        manager
    }

    /// Add a server configuration (now async and takes &self)
    pub async fn add_config(&self, config: McpServerConfig) {
        let mut configs = self.configs.write().await;
        configs.insert(config.name.clone(), config);
    }

    /// Remove a server configuration
    pub async fn remove_config(&self, server_name: &str) -> Option<McpServerConfig> {
        let mut configs = self.configs.write().await;
        configs.remove(server_name)
    }

    /// Check if a server is configured
    pub async fn has_config(&self, server_name: &str) -> bool {
        let configs = self.configs.read().await;
        configs.contains_key(server_name)
    }

    /// Get all configured server names
    pub async fn configured_servers(&self) -> Vec<String> {
        let configs = self.configs.read().await;
        configs.keys().cloned().collect()
    }

    /// Connect to a specific server
    pub async fn connect(&self, server_name: &str) -> Result<()> {
        let config = {
            let configs = self.configs.read().await;
            configs.get(server_name).cloned().ok_or_else(|| {
                IntelligenceError::Config(format!("Unknown server: {}", server_name))
            })?
        };

        tracing::info!(server = %server_name, "Connecting to MCP server");

        let client = match &config.transport {
            McpTransport::Stdio { command, args } => {
                self.connect_stdio(command, args, &config.env).await?
            }
            McpTransport::Sse { url, auth_token } => {
                self.connect_sse(url, auth_token.as_deref()).await?
            }
            McpTransport::Http { url, auth_token } => {
                self.connect_http(url, auth_token.as_deref()).await?
            }
        };

        // Cache available tools
        let tools = client.list_tools(Default::default()).await.map_err(|e| {
            IntelligenceError::Config(format!("Failed to list tools: {}", e))
        })?;

        let tool_infos: Vec<McpToolInfo> = tools
            .tools
            .into_iter()
            .map(|t| McpToolInfo {
                name: t.name.to_string(),
                description: t.description.map(|d| d.to_string()),
                input_schema: Some(serde_json::to_value(&t.input_schema).unwrap_or_default()),
                server: server_name.to_string(),
            })
            .collect();

        // Store connection
        {
            let mut clients = self.clients.write().await;
            clients.insert(
                server_name.to_string(),
                ClientConnection {
                    client,
                    status: McpClientStatus::Connected,
                },
            );
        }

        // Store tools cache
        {
            let mut cache = self.tools_cache.write().await;
            cache.insert(server_name.to_string(), tool_infos);
        }

        tracing::info!(server = %server_name, "Connected to MCP server");
        Ok(())
    }

    /// Connect via stdio transport
    async fn connect_stdio(
        &self,
        command: &str,
        args: &[String],
        env: &HashMap<String, String>,
    ) -> Result<McpClient> {
        let mut cmd = Command::new(command);
        cmd.args(args);
        for (k, v) in env {
            cmd.env(k, v);
        }

        // Configure command - TokioChildProcess takes ownership of Command
        let transport = TokioChildProcess::new(cmd)
            .map_err(|e| IntelligenceError::Config(format!("Failed to create transport: {}", e)))?;

        // Create client info
        let client_info = ClientInfo {
            protocol_version: Default::default(),
            capabilities: ClientCapabilities::default(),
            client_info: Implementation {
                name: "whytcard-intelligence".to_string(),
                title: None,
                version: env!("CARGO_PKG_VERSION").to_string(),
                website_url: None,
                icons: None,
            },
        };

        // Connect
        let client = client_info.serve(transport).await.map_err(|e| {
            IntelligenceError::Config(format!("Failed to connect: {}", e))
        })?;

        Ok(client)
    }

    /// Connect via SSE transport
    async fn connect_sse(&self, _url: &str, _auth_token: Option<&str>) -> Result<McpClient> {
        // SSE transport requires additional setup
        // For now, return an error indicating it's not yet implemented
        Err(IntelligenceError::Config(
            "SSE transport not yet implemented in this version".to_string(),
        ))
    }

    /// Connect via HTTP transport
    async fn connect_http(&self, _url: &str, _auth_token: Option<&str>) -> Result<McpClient> {
        // HTTP transport requires additional setup
        Err(IntelligenceError::Config(
            "HTTP transport not yet implemented in this version".to_string(),
        ))
    }

    /// Connect to all configured servers
    pub async fn connect_all(&self) -> Vec<(String, Result<()>)> {
        let server_names: Vec<String> = {
            let configs = self.configs.read().await;
            configs.keys().cloned().collect()
        };
        let mut results = Vec::new();

        for name in server_names {
            let result = self.connect(&name).await;
            results.push((name, result));
        }

        results
    }

    /// Call a tool on a specific server
    pub async fn call_tool(
        &self,
        server_name: &str,
        tool_name: &str,
        arguments: Option<serde_json::Value>,
    ) -> Result<McpToolResult> {
        let clients = self.clients.read().await;

        let conn = clients.get(server_name).ok_or_else(|| {
            IntelligenceError::Config(format!("Server not connected: {}", server_name))
        })?;

        let args = arguments
            .and_then(|v| v.as_object().cloned());

        // Clone tool_name to owned String for the request
        let tool_name_owned = tool_name.to_string();

        let result = conn
            .client
            .call_tool(CallToolRequestParam {
                name: tool_name_owned.into(),
                arguments: args,
            })
            .await
            .map_err(|e| {
                IntelligenceError::Config(format!("Tool call failed: {}", e))
            })?;

        // Extract text content from result
        let content = result
            .content
            .iter()
            .filter_map(|c| {
                // Check if this is a text content by trying to get the text field
                c.raw.as_text().map(|text| text.text.to_string())
            })
            .collect::<Vec<_>>()
            .join("\n");

        let is_error = result.is_error.unwrap_or(false);

        if is_error {
            Ok(McpToolResult::failure(tool_name, server_name, &content))
        } else {
            Ok(McpToolResult::success(tool_name, server_name, &content))
        }
    }

    /// Get available tools from all connected servers
    pub async fn list_all_tools(&self) -> Vec<McpToolInfo> {
        let cache = self.tools_cache.read().await;
        cache.values().flatten().cloned().collect()
    }

    /// Get tools from a specific server
    pub async fn list_server_tools(&self, server_name: &str) -> Vec<McpToolInfo> {
        let cache = self.tools_cache.read().await;
        cache.get(server_name).cloned().unwrap_or_default()
    }

    /// Check if a server is connected
    pub async fn is_connected(&self, server_name: &str) -> bool {
        let clients = self.clients.read().await;
        clients
            .get(server_name)
            .map(|c| c.status == McpClientStatus::Connected)
            .unwrap_or(false)
    }

    /// Get connection status for all servers
    pub async fn get_status(&self) -> HashMap<String, McpClientStatus> {
        let clients = self.clients.read().await;
        let configs = self.configs.read().await;
        let mut status = HashMap::new();

        for name in configs.keys() {
            let s = clients
                .get(name)
                .map(|c| c.status)
                .unwrap_or(McpClientStatus::Disconnected);
            status.insert(name.clone(), s);
        }

        status
    }

    /// Disconnect from a server
    pub async fn disconnect(&self, server_name: &str) -> Result<()> {
        let mut clients = self.clients.write().await;

        if let Some(conn) = clients.remove(server_name) {
            conn.client.cancel().await.map_err(|e| {
                IntelligenceError::Config(format!("Failed to disconnect: {}", e))
            })?;
        }

        // Clear tools cache
        let mut cache = self.tools_cache.write().await;
        cache.remove(server_name);

        tracing::info!(server = %server_name, "Disconnected from MCP server");
        Ok(())
    }

    /// Disconnect from all servers
    pub async fn disconnect_all(&self) -> Result<()> {
        let server_names: Vec<String> = {
            let clients = self.clients.read().await;
            clients.keys().cloned().collect()
        };

        for name in server_names {
            if let Err(e) = self.disconnect(&name).await {
                tracing::warn!(server = %name, error = %e, "Failed to disconnect");
            }
        }

        Ok(())
    }

    // ==========================================================================
    // Convenience methods for specific MCP servers
    // ==========================================================================

    /// Call sequential thinking tool
    pub async fn sequential_thinking(
        &self,
        thought: &str,
        thought_number: u32,
        total_thoughts: u32,
        next_needed: bool,
    ) -> Result<McpToolResult> {
        let args = serde_json::json!({
            "thought": thought,
            "thoughtNumber": thought_number,
            "totalThoughts": total_thoughts,
            "nextThoughtNeeded": next_needed
        });

        self.call_tool("sequential-thinking", "sequentialthinking", Some(args))
            .await
    }

    /// Call context7 to get library docs
    pub async fn get_library_docs(
        &self,
        library_id: &str,
        topic: Option<&str>,
        tokens: Option<u32>,
    ) -> Result<McpToolResult> {
        let mut args = serde_json::json!({
            "context7CompatibleLibraryID": library_id
        });

        if let Some(t) = topic {
            args["topic"] = serde_json::json!(t);
        }
        if let Some(n) = tokens {
            args["tokens"] = serde_json::json!(n);
        }

        self.call_tool("context7", "get-library-docs", Some(args))
            .await
    }

    /// Call context7 to resolve library ID
    pub async fn resolve_library_id(&self, library_name: &str) -> Result<McpToolResult> {
        let args = serde_json::json!({
            "libraryName": library_name
        });

        self.call_tool("context7", "resolve-library-id", Some(args))
            .await
    }

    /// Call tavily search
    pub async fn tavily_search(
        &self,
        query: &str,
        max_results: Option<u32>,
    ) -> Result<McpToolResult> {
        let mut args = serde_json::json!({
            "query": query
        });

        if let Some(n) = max_results {
            args["max_results"] = serde_json::json!(n);
        }

        self.call_tool("tavily", "tavily-search", Some(args)).await
    }

    /// Call memory store
    pub async fn memory_store(
        &self,
        content: &str,
        key: Option<&str>,
        tags: Option<Vec<String>>,
    ) -> Result<McpToolResult> {
        let mut args = serde_json::json!({
            "content": content
        });

        if let Some(k) = key {
            args["key"] = serde_json::json!(k);
        }
        if let Some(t) = tags {
            args["tags"] = serde_json::json!(t);
        }

        self.call_tool("memory", "memory_store", Some(args)).await
    }

    /// Call memory search
    pub async fn memory_search(
        &self,
        query: &str,
        limit: Option<u32>,
    ) -> Result<McpToolResult> {
        let mut args = serde_json::json!({
            "query": query
        });

        if let Some(n) = limit {
            args["limit"] = serde_json::json!(n);
        }

        self.call_tool("memory", "memory_search", Some(args)).await
    }
}

impl Default for McpClientManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = McpClientManager::new();
        let configs = manager.configs.read().await;
        assert!(configs.is_empty());
    }

    #[tokio::test]
    async fn test_add_config() {
        let manager = McpClientManager::new();
        manager.add_config(PredefinedServers::sequential_thinking()).await;
        let configs = manager.configs.read().await;
        assert!(configs.contains_key("sequential-thinking"));
    }

    #[test]
    fn test_predefined_servers() {
        let config = PredefinedServers::sequential_thinking();
        assert_eq!(config.name, "sequential-thinking");

        let config = PredefinedServers::context7();
        assert_eq!(config.name, "context7");
    }

    #[tokio::test]
    async fn test_not_connected() {
        let manager = McpClientManager::new();
        let result = manager.call_tool("unknown", "test", None).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_remove_config() {
        let manager = McpClientManager::new();
        manager.add_config(PredefinedServers::sequential_thinking()).await;

        let removed = manager.remove_config("sequential-thinking").await;
        assert!(removed.is_some());

        let has = manager.has_config("sequential-thinking").await;
        assert!(!has);
    }

    #[tokio::test]
    async fn test_configured_servers() {
        let manager = McpClientManager::new();
        manager.add_config(PredefinedServers::sequential_thinking()).await;
        manager.add_config(PredefinedServers::context7()).await;

        let servers = manager.configured_servers().await;
        assert_eq!(servers.len(), 2);
        assert!(servers.contains(&"sequential-thinking".to_string()));
        assert!(servers.contains(&"context7".to_string()));
    }
}
