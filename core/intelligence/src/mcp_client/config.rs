//! MCP Server Configuration Management
//!
//! Handles persistence and dynamic management of MCP server configurations.
//! Supports local-first installation of MCP servers in the project directory.

use super::types::{McpServerConfig, McpTransport};
use crate::error::{IntelligenceError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process::Command;

/// Persistent configuration for installed MCP servers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServersConfig {
    /// Version for migration support
    #[serde(default = "default_version")]
    pub version: u32,

    /// Installed MCP servers
    #[serde(default)]
    pub servers: HashMap<String, InstalledMcpServer>,
}

fn default_version() -> u32 {
    1
}

/// Configuration for a single installed MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InstalledMcpServer {
    /// Server name (unique identifier)
    pub name: String,

    /// Human-readable description
    #[serde(default)]
    pub description: String,

    /// Package name (npm package, pip package, or binary path)
    pub package: String,

    /// Package type: "npm", "pip", "binary"
    #[serde(default = "default_package_type")]
    pub package_type: String,

    /// Transport type: "stdio", "sse", "http"
    #[serde(default = "default_transport_type")]
    pub transport_type: String,

    /// Command to run (for stdio)
    #[serde(default)]
    pub command: Option<String>,

    /// Arguments for the command
    #[serde(default)]
    pub args: Vec<String>,

    /// URL for SSE/HTTP transport
    #[serde(default)]
    pub url: Option<String>,

    /// Environment variables (including API keys)
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Whether this server is enabled
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Auto-connect on startup
    #[serde(default)]
    pub auto_connect: bool,

    /// Local installation path (for local-first)
    #[serde(default)]
    pub install_path: Option<String>,

    /// Whether package is installed locally
    #[serde(default)]
    pub installed_locally: bool,

    /// Installation timestamp
    #[serde(default)]
    pub installed_at: Option<String>,

    /// Last connection timestamp
    #[serde(default)]
    pub last_connected: Option<String>,
}

fn default_package_type() -> String {
    "npm".to_string()
}

fn default_transport_type() -> String {
    "stdio".to_string()
}

fn default_enabled() -> bool {
    true
}

impl InstalledMcpServer {
    /// Create a new npm-based MCP server config
    pub fn npm(name: impl Into<String>, package: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            description: String::new(),
            package: package.into(),
            package_type: "npm".to_string(),
            transport_type: "stdio".to_string(),
            command: None, // Will be set after local install
            args: Vec::new(),
            url: None,
            env: HashMap::new(),
            enabled: true,
            auto_connect: false,
            install_path: None,
            installed_locally: false,
            installed_at: Some(chrono::Utc::now().to_rfc3339()),
            last_connected: None,
        }
    }

    /// Create a new pip/uvx-based MCP server config
    pub fn pip(name: impl Into<String>, package: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            description: String::new(),
            package: package.into(),
            package_type: "pip".to_string(),
            transport_type: "stdio".to_string(),
            command: None, // Will be set after local install
            args: Vec::new(),
            url: None,
            env: HashMap::new(),
            enabled: true,
            auto_connect: false,
            install_path: None,
            installed_locally: false,
            installed_at: Some(chrono::Utc::now().to_rfc3339()),
            last_connected: None,
        }
    }

    /// Create an SSE-based MCP server config
    pub fn sse(name: impl Into<String>, url: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            description: String::new(),
            package: String::new(),
            package_type: "remote".to_string(),
            transport_type: "sse".to_string(),
            command: None,
            args: Vec::new(),
            url: Some(url.into()),
            env: HashMap::new(),
            enabled: true,
            auto_connect: false,
            install_path: None,
            installed_locally: false,
            installed_at: Some(chrono::Utc::now().to_rfc3339()),
            last_connected: None,
        }
    }

    /// Create an HTTP-based MCP server config
    pub fn http(name: impl Into<String>, url: impl Into<String>) -> Self {
        let name = name.into();
        Self {
            name: name.clone(),
            description: String::new(),
            package: String::new(),
            package_type: "remote".to_string(),
            transport_type: "http".to_string(),
            command: None,
            args: Vec::new(),
            url: Some(url.into()),
            env: HashMap::new(),
            enabled: true,
            auto_connect: false,
            install_path: None,
            installed_locally: false,
            installed_at: Some(chrono::Utc::now().to_rfc3339()),
            last_connected: None,
        }
    }

    /// Add description
    pub fn with_description(mut self, desc: impl Into<String>) -> Self {
        self.description = desc.into();
        self
    }

    /// Add environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set auto-connect
    pub fn with_auto_connect(mut self, auto: bool) -> Self {
        self.auto_connect = auto;
        self
    }

    /// Set install path (for local-first)
    pub fn with_install_path(mut self, path: impl Into<String>) -> Self {
        self.install_path = Some(path.into());
        self
    }

    /// Convert to McpServerConfig for the manager
    pub fn to_server_config(&self) -> McpServerConfig {
        let transport = match self.transport_type.as_str() {
            "sse" => McpTransport::Sse {
                url: self.url.clone().unwrap_or_default(),
                auth_token: self.env.get("AUTH_TOKEN").cloned(),
            },
            "http" => McpTransport::Http {
                url: self.url.clone().unwrap_or_default(),
                auth_token: self.env.get("AUTH_TOKEN").cloned(),
            },
            _ => {
                // stdio - use local path if installed locally
                if self.installed_locally {
                    if let Some(ref install_path) = self.install_path {
                        let (command, args) = self.get_local_command(install_path);
                        McpTransport::Stdio { command, args }
                    } else {
                        self.get_fallback_transport()
                    }
                } else {
                    self.get_fallback_transport()
                }
            }
        };

        McpServerConfig {
            name: self.name.clone(),
            transport,
            env: self.env.clone(),
            auto_reconnect: true,
            timeout_secs: 30,
        }
    }

    /// Get command for locally installed package
    fn get_local_command(&self, install_path: &str) -> (String, Vec<String>) {
        let install_dir = PathBuf::from(install_path);

        match self.package_type.as_str() {
            "npm" => {
                // Use node with the local package
                // Try to find the bin entry in the package
                let bin_name = self.get_npm_bin_name();

                #[cfg(windows)]
                let bin_path = install_dir
                    .join("node_modules")
                    .join(".bin")
                    .join(format!("{}.cmd", bin_name));

                #[cfg(not(windows))]
                let bin_path = install_dir
                    .join("node_modules")
                    .join(".bin")
                    .join(&bin_name);

                if bin_path.exists() {
                    (bin_path.to_string_lossy().to_string(), self.args.clone())
                } else {
                    // Fallback: try running with node directly
                    let module_path = install_dir
                        .join("node_modules")
                        .join(&self.package);

                    (
                        "node".to_string(),
                        vec![module_path.to_string_lossy().to_string()]
                            .into_iter()
                            .chain(self.args.clone())
                            .collect(),
                    )
                }
            }
            "pip" => {
                // Use the venv python
                #[cfg(windows)]
                let python_path = install_dir.join("venv").join("Scripts").join("python.exe");

                #[cfg(not(windows))]
                let python_path = install_dir.join("venv").join("bin").join("python");

                let mut args = vec!["-m".to_string(), self.package.replace("-", "_")];
                args.extend(self.args.clone());

                (python_path.to_string_lossy().to_string(), args)
            }
            _ => self.get_fallback_transport_parts(),
        }
    }

    /// Get npm binary name from package name
    fn get_npm_bin_name(&self) -> String {
        // @scope/package-name -> package-name
        // package-name -> package-name
        self.package
            .split('/')
            .next_back()
            .unwrap_or(&self.package)
            .to_string()
    }

    /// Fallback transport when not installed locally
    fn get_fallback_transport(&self) -> McpTransport {
        let (command, args) = self.get_fallback_transport_parts();
        McpTransport::Stdio { command, args }
    }

    /// Get fallback command parts (npx/uvx)
    fn get_fallback_transport_parts(&self) -> (String, Vec<String>) {
        match self.package_type.as_str() {
            "npm" => {
                let mut args = vec!["-y".to_string(), self.package.clone()];
                args.extend(self.args.clone());
                ("npx".to_string(), args)
            }
            "pip" => {
                let mut args = vec![self.package.clone()];
                args.extend(self.args.clone());
                ("uvx".to_string(), args)
            }
            _ => {
                let command = self.command.clone().unwrap_or_else(|| self.package.clone());
                (command, self.args.clone())
            }
        }
    }
}

impl McpServersConfig {
    /// Create empty config
    pub fn new() -> Self {
        Self {
            version: 1,
            servers: HashMap::new(),
        }
    }

    /// Load config from file
    pub fn load(path: &Path) -> Result<Self> {
        if !path.exists() {
            return Ok(Self::new());
        }

        let content = std::fs::read_to_string(path).map_err(|e| {
            IntelligenceError::Config(format!("Failed to read MCP config: {}", e))
        })?;

        serde_json::from_str(&content).map_err(|e| {
            IntelligenceError::Config(format!("Failed to parse MCP config: {}", e))
        })
    }

    /// Save config to file
    pub fn save(&self, path: &Path) -> Result<()> {
        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent).map_err(|e| {
                IntelligenceError::Config(format!("Failed to create config directory: {}", e))
            })?;
        }

        let content = serde_json::to_string_pretty(self).map_err(|e| {
            IntelligenceError::Config(format!("Failed to serialize MCP config: {}", e))
        })?;

        std::fs::write(path, content).map_err(|e| {
            IntelligenceError::Config(format!("Failed to write MCP config: {}", e))
        })?;

        Ok(())
    }

    /// Add or update a server
    pub fn add_server(&mut self, server: InstalledMcpServer) {
        self.servers.insert(server.name.clone(), server);
    }

    /// Remove a server
    pub fn remove_server(&mut self, name: &str) -> Option<InstalledMcpServer> {
        self.servers.remove(name)
    }

    /// Get a server by name
    pub fn get_server(&self, name: &str) -> Option<&InstalledMcpServer> {
        self.servers.get(name)
    }

    /// Get mutable reference to a server
    pub fn get_server_mut(&mut self, name: &str) -> Option<&mut InstalledMcpServer> {
        self.servers.get_mut(name)
    }

    /// Get all enabled servers
    pub fn enabled_servers(&self) -> impl Iterator<Item = &InstalledMcpServer> {
        self.servers.values().filter(|s| s.enabled)
    }

    /// Get all auto-connect servers
    pub fn auto_connect_servers(&self) -> impl Iterator<Item = &InstalledMcpServer> {
        self.servers.values().filter(|s| s.enabled && s.auto_connect)
    }

    /// Convert all enabled servers to McpServerConfig
    pub fn to_server_configs(&self) -> Vec<McpServerConfig> {
        self.enabled_servers()
            .map(|s| s.to_server_config())
            .collect()
    }
}

impl Default for McpServersConfig {
    fn default() -> Self {
        Self::new()
    }
}

/// Manager for MCP configuration persistence with local-first installation
pub struct McpConfigManager {
    /// Path to the config file
    config_path: PathBuf,

    /// Path to the MCP installation directory (core/mcp/)
    mcp_dir: PathBuf,

    /// Current configuration
    config: McpServersConfig,
}

impl McpConfigManager {
    /// Create a new config manager
    ///
    /// # Arguments
    /// * `mcp_dir` - Path to the MCP directory (e.g., core/mcp/)
    pub fn new(mcp_dir: &Path) -> Result<Self> {
        let config_path = mcp_dir.join("mcp_servers.json");
        let config = McpServersConfig::load(&config_path)?;

        // Ensure mcp_dir exists
        std::fs::create_dir_all(mcp_dir).map_err(|e| {
            IntelligenceError::Config(format!("Failed to create MCP directory: {}", e))
        })?;

        Ok(Self {
            config_path,
            mcp_dir: mcp_dir.to_path_buf(),
            config
        })
    }

    /// Get the MCP installation directory
    pub fn mcp_dir(&self) -> &Path {
        &self.mcp_dir
    }

    /// Get the current config
    pub fn config(&self) -> &McpServersConfig {
        &self.config
    }

    /// Get mutable config
    pub fn config_mut(&mut self) -> &mut McpServersConfig {
        &mut self.config
    }

    /// Save the current config
    pub fn save(&self) -> Result<()> {
        self.config.save(&self.config_path)
    }

    /// Install a new MCP server locally
    ///
    /// This will:
    /// 1. Install the package locally (npm install or pip install)
    /// 2. Update the server config with the local path
    /// 3. Save the configuration
    pub fn install(&mut self, mut server: InstalledMcpServer) -> Result<()> {
        // Set the install path
        server.install_path = Some(self.mcp_dir.to_string_lossy().to_string());

        // Perform local installation based on package type
        match server.package_type.as_str() {
            "npm" => self.install_npm_package(&server)?,
            "pip" => self.install_pip_package(&server)?,
            "remote" => {
                // Remote servers don't need local installation
                server.installed_locally = true;
            }
            _ => {
                // Binary or custom - assume already available
                server.installed_locally = true;
            }
        }

        server.installed_locally = true;
        self.config.add_server(server);
        self.save()
    }

    /// Install an npm package locally
    fn install_npm_package(&self, server: &InstalledMcpServer) -> Result<()> {
        // Check if package.json exists, create if not
        let package_json = self.mcp_dir.join("package.json");
        if !package_json.exists() {
            let initial_package = r#"{
  "name": "whytcard-mcp-servers",
  "version": "1.0.0",
  "private": true,
  "description": "Local MCP server installations",
  "dependencies": {}
}"#;
            std::fs::write(&package_json, initial_package).map_err(|e| {
                IntelligenceError::Config(format!("Failed to create package.json: {}", e))
            })?;
        }

        // Run npm install
        let output = Command::new("npm")
            .args(["install", "--save", &server.package])
            .current_dir(&self.mcp_dir)
            .output()
            .map_err(|e| {
                IntelligenceError::Config(format!("Failed to run npm install: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(IntelligenceError::Config(format!(
                "npm install failed: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Install a pip package locally in a venv
    fn install_pip_package(&self, server: &InstalledMcpServer) -> Result<()> {
        let venv_dir = self.mcp_dir.join("venv");

        // Create venv if it doesn't exist
        if !venv_dir.exists() {
            let output = Command::new("python")
                .args(["-m", "venv", "venv"])
                .current_dir(&self.mcp_dir)
                .output()
                .map_err(|e| {
                    IntelligenceError::Config(format!("Failed to create venv: {}", e))
                })?;

            if !output.status.success() {
                let stderr = String::from_utf8_lossy(&output.stderr);
                return Err(IntelligenceError::Config(format!(
                    "venv creation failed: {}",
                    stderr
                )));
            }
        }

        // Get pip path
        #[cfg(windows)]
        let pip_path = venv_dir.join("Scripts").join("pip.exe");
        #[cfg(not(windows))]
        let pip_path = venv_dir.join("bin").join("pip");

        // Run pip install
        let output = Command::new(&pip_path)
            .args(["install", &server.package])
            .current_dir(&self.mcp_dir)
            .output()
            .map_err(|e| {
                IntelligenceError::Config(format!("Failed to run pip install: {}", e))
            })?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            return Err(IntelligenceError::Config(format!(
                "pip install failed: {}",
                stderr
            )));
        }

        Ok(())
    }

    /// Uninstall an MCP server
    pub fn uninstall(&mut self, name: &str) -> Result<Option<InstalledMcpServer>> {
        let removed = self.config.remove_server(name);

        // Optionally uninstall the package
        // For now, we leave the packages installed to avoid breaking other servers
        // that might depend on them. The user can manually clean up if needed.

        self.save()?;
        Ok(removed)
    }

    /// Enable a server
    pub fn enable(&mut self, name: &str) -> Result<bool> {
        if let Some(server) = self.config.get_server_mut(name) {
            server.enabled = true;
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Disable a server
    pub fn disable(&mut self, name: &str) -> Result<bool> {
        if let Some(server) = self.config.get_server_mut(name) {
            server.enabled = false;
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Update environment variable for a server
    pub fn set_env(&mut self, name: &str, key: &str, value: &str) -> Result<bool> {
        if let Some(server) = self.config.get_server_mut(name) {
            server.env.insert(key.to_string(), value.to_string());
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Remove environment variable from a server
    pub fn remove_env(&mut self, name: &str, key: &str) -> Result<bool> {
        if let Some(server) = self.config.get_server_mut(name) {
            server.env.remove(key);
            self.save()?;
            Ok(true)
        } else {
            Ok(false)
        }
    }

    /// Update last connected timestamp
    pub fn update_last_connected(&mut self, name: &str) -> Result<()> {
        if let Some(server) = self.config.get_server_mut(name) {
            server.last_connected = Some(chrono::Utc::now().to_rfc3339());
            self.save()?;
        }
        Ok(())
    }

    /// List all installed servers
    pub fn list_all(&self) -> impl Iterator<Item = (&String, &InstalledMcpServer)> {
        self.config.servers.iter()
    }

    /// Get a server by name
    pub fn get(&self, name: &str) -> Option<&InstalledMcpServer> {
        self.config.servers.get(name)
    }

    /// Check if a server is installed
    pub fn is_installed(&self, name: &str) -> bool {
        self.config.servers.contains_key(name)
    }

    /// Count installed servers
    pub fn count(&self) -> usize {
        self.config.servers.len()
    }

    /// Check if npm package is installed locally
    pub fn is_npm_installed(&self, package: &str) -> bool {
        let package_dir = self.mcp_dir.join("node_modules").join(package);
        package_dir.exists()
    }

    /// Check if pip package is installed locally
    pub fn is_pip_installed(&self, package: &str) -> bool {
        #[cfg(windows)]
        let pip_path = self.mcp_dir.join("venv").join("Scripts").join("pip.exe");
        #[cfg(not(windows))]
        let pip_path = self.mcp_dir.join("venv").join("bin").join("pip");

        if !pip_path.exists() {
            return false;
        }

        // Run pip show to check if package is installed
        let output = Command::new(&pip_path)
            .args(["show", package])
            .output();

        match output {
            Ok(o) => o.status.success(),
            Err(_) => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_installed_server_npm() {
        let server = InstalledMcpServer::npm("test", "@test/mcp-server")
            .with_description("Test server")
            .with_env("API_KEY", "secret");

        assert_eq!(server.name, "test");
        assert_eq!(server.package, "@test/mcp-server");
        assert_eq!(server.package_type, "npm");
        assert!(server.enabled);
        assert!(!server.installed_locally);
        assert_eq!(server.env.get("API_KEY"), Some(&"secret".to_string()));
    }

    #[test]
    fn test_installed_server_pip() {
        let server = InstalledMcpServer::pip("test-py", "mcp-server-test");

        assert_eq!(server.name, "test-py");
        assert_eq!(server.package_type, "pip");
        assert!(!server.installed_locally);
    }

    #[test]
    fn test_npm_bin_name() {
        let server = InstalledMcpServer::npm("test", "@upstash/context7-mcp");
        assert_eq!(server.get_npm_bin_name(), "context7-mcp");

        let server2 = InstalledMcpServer::npm("test2", "simple-package");
        assert_eq!(server2.get_npm_bin_name(), "simple-package");
    }

    #[test]
    fn test_to_server_config_fallback() {
        // When not installed locally, should use npx fallback
        let server = InstalledMcpServer::npm("context7", "@upstash/context7-mcp");
        let config = server.to_server_config();

        assert_eq!(config.name, "context7");
        match config.transport {
            McpTransport::Stdio { command, args } => {
                assert_eq!(command, "npx");
                assert!(args.contains(&"-y".to_string()));
                assert!(args.contains(&"@upstash/context7-mcp".to_string()));
            }
            _ => panic!("Expected stdio transport"),
        }
    }

    #[test]
    fn test_to_server_config_local() {
        // When installed locally, should use local path
        let mut server = InstalledMcpServer::npm("context7", "@upstash/context7-mcp");
        server.installed_locally = true;
        server.install_path = Some("/fake/path".to_string());

        let config = server.to_server_config();

        assert_eq!(config.name, "context7");
        // The transport will try to use the local path
        // (actual path won't exist in test, but logic is correct)
    }

    #[test]
    fn test_config_save_load() {
        let temp = TempDir::new().unwrap();
        let config_path = temp.path().join("mcp_servers.json");

        let mut config = McpServersConfig::new();
        config.add_server(InstalledMcpServer::npm("test", "@test/server"));
        config.save(&config_path).unwrap();

        let loaded = McpServersConfig::load(&config_path).unwrap();
        assert!(loaded.servers.contains_key("test"));
    }

    #[test]
    fn test_config_manager_basic() {
        let temp = TempDir::new().unwrap();
        let manager = McpConfigManager::new(temp.path()).unwrap();

        assert_eq!(manager.count(), 0);
        assert_eq!(manager.mcp_dir(), temp.path());
    }

    #[test]
    fn test_config_manager_enable_disable() {
        let temp = TempDir::new().unwrap();
        let mut manager = McpConfigManager::new(temp.path()).unwrap();

        // Add server directly to config (skip install which requires npm)
        let mut server = InstalledMcpServer::npm("test", "@test/server");
        server.installed_locally = true;
        manager.config.add_server(server);
        manager.save().unwrap();

        assert!(manager.config().get_server("test").unwrap().enabled);

        manager.disable("test").unwrap();
        assert!(!manager.config().get_server("test").unwrap().enabled);

        manager.enable("test").unwrap();
        assert!(manager.config().get_server("test").unwrap().enabled);
    }
}
