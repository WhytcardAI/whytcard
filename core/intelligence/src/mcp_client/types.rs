//! Common types for MCP clients

use serde::{Deserialize, Serialize};

/// Result from calling an external MCP tool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolResult {
    /// Tool name that was called
    pub tool_name: String,

    /// Server that handled the call
    pub server: String,

    /// Whether the call succeeded
    pub success: bool,

    /// Result content (text)
    pub content: String,

    /// Structured data if available
    pub data: Option<serde_json::Value>,

    /// Error message if failed
    pub error: Option<String>,
}

/// Configuration for an MCP server connection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpServerConfig {
    /// Server name/identifier
    pub name: String,

    /// Transport type
    pub transport: McpTransport,

    /// Environment variables to set
    #[serde(default)]
    pub env: std::collections::HashMap<String, String>,

    /// Whether to auto-reconnect on disconnect
    #[serde(default = "default_true")]
    pub auto_reconnect: bool,

    /// Connection timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_true() -> bool {
    true
}

fn default_timeout() -> u64 {
    30
}

/// Transport configuration for MCP connection
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum McpTransport {
    /// Standard I/O with child process
    Stdio {
        /// Command to run
        command: String,
        /// Arguments
        #[serde(default)]
        args: Vec<String>,
    },

    /// Server-Sent Events over HTTP
    Sse {
        /// SSE endpoint URL
        url: String,
        /// Optional auth token
        auth_token: Option<String>,
    },

    /// Streamable HTTP
    Http {
        /// HTTP endpoint URL
        url: String,
        /// Optional auth token
        auth_token: Option<String>,
    },
}

/// Status of an MCP client connection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum McpClientStatus {
    /// Not connected
    Disconnected,
    /// Attempting to connect
    Connecting,
    /// Connected and ready
    Connected,
    /// Connection failed
    Failed,
}

/// Information about an available tool from an MCP server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpToolInfo {
    /// Tool name
    pub name: String,

    /// Tool description
    pub description: Option<String>,

    /// Input schema as JSON
    pub input_schema: Option<serde_json::Value>,

    /// Server that provides this tool
    pub server: String,
}

/// Aggregated result from multiple MCP sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMcpResult {
    /// Query that was executed
    pub query: String,

    /// Results from each source
    pub results: Vec<McpToolResult>,

    /// Combined summary
    pub summary: Option<String>,

    /// Total execution time in ms
    pub execution_time_ms: u64,
}

impl McpToolResult {
    /// Create a successful result
    pub fn success(tool_name: impl Into<String>, server: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            server: server.into(),
            success: true,
            content: content.into(),
            data: None,
            error: None,
        }
    }

    /// Create a failed result
    pub fn failure(tool_name: impl Into<String>, server: impl Into<String>, error: impl Into<String>) -> Self {
        Self {
            tool_name: tool_name.into(),
            server: server.into(),
            success: false,
            content: String::new(),
            data: None,
            error: Some(error.into()),
        }
    }

    /// Add structured data to the result
    pub fn with_data(mut self, data: serde_json::Value) -> Self {
        self.data = Some(data);
        self
    }
}

impl McpServerConfig {
    /// Create a stdio config for npx command
    pub fn npx(name: impl Into<String>, package: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transport: McpTransport::Stdio {
                command: "npx".to_string(),
                args: vec!["-y".to_string(), package.into()],
            },
            env: Default::default(),
            auto_reconnect: true,
            timeout_secs: 30,
        }
    }

    /// Create a stdio config for uvx (Python) command
    pub fn uvx(name: impl Into<String>, package: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transport: McpTransport::Stdio {
                command: "uvx".to_string(),
                args: vec![package.into()],
            },
            env: Default::default(),
            auto_reconnect: true,
            timeout_secs: 30,
        }
    }

    /// Create an SSE config
    pub fn sse(name: impl Into<String>, url: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            transport: McpTransport::Sse {
                url: url.into(),
                auth_token: None,
            },
            env: Default::default(),
            auto_reconnect: true,
            timeout_secs: 30,
        }
    }

    /// Add environment variable
    pub fn with_env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }

    /// Set auth token (for SSE/HTTP)
    pub fn with_auth(mut self, token: impl Into<String>) -> Self {
        match &mut self.transport {
            McpTransport::Sse { auth_token, .. } => *auth_token = Some(token.into()),
            McpTransport::Http { auth_token, .. } => *auth_token = Some(token.into()),
            McpTransport::Stdio { .. } => {}
        }
        self
    }
}

/// Predefined MCP server configurations
pub struct PredefinedServers;

impl PredefinedServers {
    // =========================================================================
    // OFFICIAL MCP SERVERS (from @modelcontextprotocol)
    // =========================================================================

    /// Sequential Thinking server - structured problem decomposition
    pub fn sequential_thinking() -> McpServerConfig {
        McpServerConfig::npx("sequential-thinking", "@modelcontextprotocol/server-sequential-thinking")
    }

    /// Memory server (official MCP) - persistent memory across sessions
    pub fn memory() -> McpServerConfig {
        McpServerConfig::npx("memory", "@modelcontextprotocol/server-memory")
    }

    /// Filesystem server - read/write files with allowed paths
    pub fn filesystem(paths: Vec<String>) -> McpServerConfig {
        let mut args = vec!["-y".to_string(), "@modelcontextprotocol/server-filesystem".to_string()];
        args.extend(paths);
        McpServerConfig {
            name: "filesystem".into(),
            transport: McpTransport::Stdio {
                command: "npx".to_string(),
                args,
            },
            env: Default::default(),
            auto_reconnect: true,
            timeout_secs: 30,
        }
    }

    /// Fetch server - HTTP requests
    pub fn fetch() -> McpServerConfig {
        McpServerConfig::npx("fetch", "@modelcontextprotocol/server-fetch")
    }

    /// Puppeteer server - browser automation
    pub fn puppeteer() -> McpServerConfig {
        McpServerConfig::npx("puppeteer", "@modelcontextprotocol/server-puppeteer")
    }

    /// GitHub server - GitHub API access
    pub fn github(token: &str) -> McpServerConfig {
        McpServerConfig::npx("github", "@modelcontextprotocol/server-github")
            .with_env("GITHUB_PERSONAL_ACCESS_TOKEN", token)
    }

    /// GitLab server - GitLab API access
    pub fn gitlab(token: &str, url: Option<&str>) -> McpServerConfig {
        let mut config = McpServerConfig::npx("gitlab", "@modelcontextprotocol/server-gitlab")
            .with_env("GITLAB_PERSONAL_ACCESS_TOKEN", token);
        if let Some(u) = url {
            config = config.with_env("GITLAB_API_URL", u);
        }
        config
    }

    /// Postgres server - PostgreSQL database access
    pub fn postgres(connection_string: &str) -> McpServerConfig {
        McpServerConfig::npx("postgres", "@modelcontextprotocol/server-postgres")
            .with_env("POSTGRES_CONNECTION_STRING", connection_string)
    }

    /// Sqlite server - SQLite database access
    pub fn sqlite(db_path: &str) -> McpServerConfig {
        McpServerConfig {
            name: "sqlite".into(),
            transport: McpTransport::Stdio {
                command: "npx".to_string(),
                args: vec![
                    "-y".to_string(),
                    "@modelcontextprotocol/server-sqlite".to_string(),
                    db_path.to_string(),
                ],
            },
            env: Default::default(),
            auto_reconnect: true,
            timeout_secs: 30,
        }
    }

    /// Slack server - Slack workspace access
    pub fn slack(bot_token: &str, team_id: &str) -> McpServerConfig {
        McpServerConfig::npx("slack", "@modelcontextprotocol/server-slack")
            .with_env("SLACK_BOT_TOKEN", bot_token)
            .with_env("SLACK_TEAM_ID", team_id)
    }

    /// Google Drive server - Google Drive access
    pub fn google_drive() -> McpServerConfig {
        McpServerConfig::npx("google-drive", "@modelcontextprotocol/server-gdrive")
    }

    /// Google Maps server - Google Maps API
    pub fn google_maps(api_key: &str) -> McpServerConfig {
        McpServerConfig::npx("google-maps", "@modelcontextprotocol/server-google-maps")
            .with_env("GOOGLE_MAPS_API_KEY", api_key)
    }

    /// Brave Search server - Brave search API
    pub fn brave_search(api_key: &str) -> McpServerConfig {
        McpServerConfig::npx("brave-search", "@modelcontextprotocol/server-brave-search")
            .with_env("BRAVE_API_KEY", api_key)
    }

    /// Everart server - AI image generation
    pub fn everart(api_key: &str) -> McpServerConfig {
        McpServerConfig::npx("everart", "@modelcontextprotocol/server-everart")
            .with_env("EVERART_API_KEY", api_key)
    }

    /// EXA server - Neural search
    pub fn exa(api_key: &str) -> McpServerConfig {
        McpServerConfig::npx("exa", "@modelcontextprotocol/server-exa")
            .with_env("EXA_API_KEY", api_key)
    }

    /// Sentry server - Error tracking
    pub fn sentry(auth_token: &str, org: &str, project: &str) -> McpServerConfig {
        McpServerConfig::npx("sentry", "@modelcontextprotocol/server-sentry")
            .with_env("SENTRY_AUTH_TOKEN", auth_token)
            .with_env("SENTRY_ORG", org)
            .with_env("SENTRY_PROJECT", project)
    }

    // =========================================================================
    // THIRD-PARTY MCP SERVERS
    // =========================================================================

    /// Context7 documentation server (Upstash) - library documentation
    pub fn context7() -> McpServerConfig {
        McpServerConfig::npx("context7", "@upstash/context7-mcp@latest")
    }

    /// Tavily search server - web search with AI
    pub fn tavily(api_key: &str) -> McpServerConfig {
        McpServerConfig::npx("tavily", "tavily-mcp@latest")
            .with_env("TAVILY_API_KEY", api_key)
    }

    /// Playwright browser automation server - full browser control
    pub fn playwright() -> McpServerConfig {
        McpServerConfig::npx("playwright", "@playwright/mcp@latest")
    }

    /// Microsoft Learn documentation server (HTTP)
    pub fn microsoft_learn() -> McpServerConfig {
        McpServerConfig {
            name: "microsoft-learn".into(),
            transport: McpTransport::Http {
                url: "https://learn.microsoft.com/api/mcp".into(),
                auth_token: None,
            },
            env: Default::default(),
            auto_reconnect: true,
            timeout_secs: 30,
        }
    }

    /// MarkItDown server (Python/uvx) - converts documents to markdown
    pub fn markitdown() -> McpServerConfig {
        McpServerConfig::uvx("markitdown", "markitdown-mcp==0.0.1a4")
    }

    /// Chrome DevTools MCP server
    pub fn chrome_devtools() -> McpServerConfig {
        McpServerConfig::npx("chrome-devtools", "chrome-devtools-mcp@latest")
    }

    /// Firecrawl server - web scraping
    pub fn firecrawl(api_key: &str) -> McpServerConfig {
        McpServerConfig::npx("firecrawl", "firecrawl-mcp")
            .with_env("FIRECRAWL_API_KEY", api_key)
    }

    /// Browserbase server - cloud browser automation
    pub fn browserbase(api_key: &str, project_id: &str) -> McpServerConfig {
        McpServerConfig::npx("browserbase", "@browserbasehq/mcp-server-browserbase")
            .with_env("BROWSERBASE_API_KEY", api_key)
            .with_env("BROWSERBASE_PROJECT_ID", project_id)
    }

    /// Raygun server - error tracking
    pub fn raygun(api_key: &str) -> McpServerConfig {
        McpServerConfig::npx("raygun", "@raygun.io/mcp-server-raygun")
            .with_env("RAYGUN_API_KEY", api_key)
    }

    /// Neon server - serverless Postgres
    pub fn neon(api_key: &str) -> McpServerConfig {
        McpServerConfig::npx("neon", "@neondatabase/mcp-server-neon")
            .with_env("NEON_API_KEY", api_key)
    }

    /// Tinybird server - analytics
    pub fn tinybird(api_key: &str) -> McpServerConfig {
        McpServerConfig::npx("tinybird", "@tinybirdco/mcp-tinybird")
            .with_env("TINYBIRD_API_KEY", api_key)
    }

    /// Cloudflare server - Cloudflare Workers/KV/D1
    pub fn cloudflare(api_token: &str, account_id: &str) -> McpServerConfig {
        McpServerConfig::npx("cloudflare", "@cloudflare/mcp-server-cloudflare")
            .with_env("CLOUDFLARE_API_TOKEN", api_token)
            .with_env("CLOUDFLARE_ACCOUNT_ID", account_id)
    }

    /// Linear server - issue tracking
    pub fn linear(api_key: &str) -> McpServerConfig {
        McpServerConfig::npx("linear", "mcp-linear")
            .with_env("LINEAR_API_KEY", api_key)
    }

    /// Notion server - Notion workspace
    pub fn notion(api_key: &str) -> McpServerConfig {
        McpServerConfig::npx("notion", "@notionhq/mcp-server-notion")
            .with_env("NOTION_API_KEY", api_key)
    }

    /// Stripe server - Stripe API
    pub fn stripe(api_key: &str) -> McpServerConfig {
        McpServerConfig::npx("stripe", "@stripe/mcp-server")
            .with_env("STRIPE_API_KEY", api_key)
    }

    /// Perplexity server - AI search
    pub fn perplexity(api_key: &str) -> McpServerConfig {
        McpServerConfig::npx("perplexity", "perplexity-mcp")
            .with_env("PERPLEXITY_API_KEY", api_key)
    }

    // =========================================================================
    // PYTHON-BASED SERVERS (uvx)
    // =========================================================================

    /// AWS KB Retrieval server (Python)
    pub fn aws_kb_retrieval() -> McpServerConfig {
        McpServerConfig::uvx("aws-kb-retrieval", "awslabs.aws-kb-retrieval-mcp-server")
    }

    /// Time server (Python) - time and timezone operations
    pub fn time() -> McpServerConfig {
        McpServerConfig::uvx("time", "mcp-server-time")
    }

    // =========================================================================
    // HELPER METHODS
    // =========================================================================

    /// Get all predefined server names (without API keys required)
    pub fn available_without_keys() -> Vec<&'static str> {
        vec![
            "sequential-thinking",
            "memory",
            "fetch",
            "puppeteer",
            "playwright",
            "context7",
            "markitdown",
            "chrome-devtools",
            "time",
            "google-drive",
        ]
    }

    /// Get servers that require API keys
    pub fn requiring_keys() -> Vec<(&'static str, &'static str)> {
        vec![
            ("tavily", "TAVILY_API_KEY"),
            ("github", "GITHUB_PERSONAL_ACCESS_TOKEN"),
            ("gitlab", "GITLAB_PERSONAL_ACCESS_TOKEN"),
            ("brave-search", "BRAVE_API_KEY"),
            ("google-maps", "GOOGLE_MAPS_API_KEY"),
            ("firecrawl", "FIRECRAWL_API_KEY"),
            ("neon", "NEON_API_KEY"),
            ("linear", "LINEAR_API_KEY"),
            ("notion", "NOTION_API_KEY"),
            ("stripe", "STRIPE_API_KEY"),
            ("perplexity", "PERPLEXITY_API_KEY"),
            ("cloudflare", "CLOUDFLARE_API_TOKEN"),
            ("exa", "EXA_API_KEY"),
            ("everart", "EVERART_API_KEY"),
        ]
    }
}
