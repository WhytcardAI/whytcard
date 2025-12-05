//! WhytCard Intelligence MCP Server
//!
//! Binary entry point for the MCP server.
//!
//! # Usage
//!
//! ## Single Instance (stdio - default)
//! ```bash
//! # Default instance (stdio transport, single client)
//! whytcard-intelligence
//!
//! # Named namespace (isolated data partition)
//! whytcard-intelligence --namespace copilot
//! whytcard-intelligence -n vscode
//! ```
//!
//! ## Multi-Session Mode (SSE server)
//! ```bash
//! # Start in SSE server mode (multiple concurrent clients)
//! whytcard-intelligence --port 3000
//! whytcard-intelligence -p 8080 --namespace shared
//!
//! # Clients connect via SSE to http://localhost:3000/sse
//! # and POST messages to http://localhost:3000/message
//! ```
//!
//! ## Environment Variables
//! ```bash
//! WHYTCARD_NAMESPACE=copilot whytcard-intelligence
//! WHYTCARD_PORT=3000 whytcard-intelligence
//! ```

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use whytcard_intelligence::{IntelligenceConfig, IntelligenceServer};

/// Environment variable for namespace
const NAMESPACE_ENV: &str = "WHYTCARD_NAMESPACE";
/// Environment variable for port (SSE mode)
const PORT_ENV: &str = "WHYTCARD_PORT";

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize logging (stderr to not interfere with stdio MCP transport)
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info,whytcard_intelligence=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer().with_writer(std::io::stderr))
        .init();

    // Parse command line arguments
    let (namespace, port) = parse_args();

    // Determine transport mode
    let transport_mode = if let Some(p) = port {
        TransportMode::Sse(p)
    } else {
        TransportMode::Stdio
    };

    match &transport_mode {
        TransportMode::Stdio => {
            if let Some(ref ns) = namespace {
                tracing::info!("WhytCard Intelligence MCP Server starting (stdio) with namespace: {}", ns);
            } else {
                tracing::info!("WhytCard Intelligence MCP Server starting (stdio, default namespace)...");
            }
        }
        TransportMode::Sse(port) => {
            tracing::info!(
                "WhytCard Intelligence MCP Server starting (SSE multi-session) on port {}{}",
                port,
                namespace.as_ref().map(|ns| format!(", namespace: {}", ns)).unwrap_or_default()
            );
        }
    }

    // Load configuration with optional namespace
    let config = match namespace {
        Some(ns) => IntelligenceConfig::default().with_namespace(ns),
        None => IntelligenceConfig::default(),
    };

    // Create server
    let server = IntelligenceServer::new(config).await?;

    // Run with appropriate transport
    match transport_mode {
        TransportMode::Stdio => {
            server.run_stdio().await?;
        }
        TransportMode::Sse(port) => {
            server.run_sse(port).await?;
        }
    }

    Ok(())
}

/// Transport mode for the server
enum TransportMode {
    /// Single client via stdio (default)
    Stdio,
    /// Multiple clients via SSE server
    Sse(u16),
}

/// Parse namespace and port from CLI args or environment variables
fn parse_args() -> (Option<String>, Option<u16>) {
    let args: Vec<String> = std::env::args().collect();
    let mut namespace = None;
    let mut port = None;

    // Parse CLI args
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--namespace" | "-n" if i + 1 < args.len() => {
                namespace = Some(args[i + 1].clone());
                i += 2;
            }
            arg if arg.starts_with("--namespace=") => {
                namespace = Some(arg.trim_start_matches("--namespace=").to_string());
                i += 1;
            }
            arg if arg.starts_with("-n=") => {
                namespace = Some(arg.trim_start_matches("-n=").to_string());
                i += 1;
            }
            "--port" | "-p" if i + 1 < args.len() => {
                port = args[i + 1].parse().ok();
                i += 2;
            }
            arg if arg.starts_with("--port=") => {
                port = arg.trim_start_matches("--port=").parse().ok();
                i += 1;
            }
            arg if arg.starts_with("-p=") => {
                port = arg.trim_start_matches("-p=").parse().ok();
                i += 1;
            }
            _ => {
                i += 1;
            }
        }
    }

    // Check environment variables as fallback
    if namespace.is_none() {
        namespace = std::env::var(NAMESPACE_ENV).ok();
    }
    if port.is_none() {
        port = std::env::var(PORT_ENV).ok().and_then(|p| p.parse().ok());
    }

    (namespace, port)
}
