//! WhytCard Intelligence MCP Server
//!
//! Binary entry point for the MCP server.
//!
//! # Usage
//!
//! ```bash
//! # Default instance
//! whytcard-intelligence
//!
//! # Named namespace (isolated data partition)
//! whytcard-intelligence --namespace copilot
//! whytcard-intelligence -n vscode
//!
//! # Via environment variable
//! WHYTCARD_NAMESPACE=copilot whytcard-intelligence
//! ```

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};
use whytcard_intelligence::{IntelligenceConfig, IntelligenceServer};

/// Environment variable for namespace
const NAMESPACE_ENV: &str = "WHYTCARD_NAMESPACE";

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
    let namespace = parse_namespace();

    if let Some(ref ns) = namespace {
        tracing::info!("WhytCard Intelligence MCP Server starting with namespace: {}", ns);
    } else {
        tracing::info!("WhytCard Intelligence MCP Server starting (default namespace)...");
    }

    // Load configuration with optional namespace
    let config = match namespace {
        Some(ns) => IntelligenceConfig::default().with_namespace(ns),
        None => IntelligenceConfig::default(),
    };

    // Create and run server
    let server = IntelligenceServer::new(config).await?;
    server.run_stdio().await?;

    Ok(())
}

/// Parse namespace from CLI args or environment variable
fn parse_namespace() -> Option<String> {
    let args: Vec<String> = std::env::args().collect();

    // Check CLI args: --namespace <name> or -n <name>
    for i in 0..args.len() {
        if (args[i] == "--namespace" || args[i] == "-n") && i + 1 < args.len() {
            return Some(args[i + 1].clone());
        }
        // Also support --namespace=value format
        if args[i].starts_with("--namespace=") {
            return Some(args[i].trim_start_matches("--namespace=").to_string());
        }
        if args[i].starts_with("-n=") {
            return Some(args[i].trim_start_matches("-n=").to_string());
        }
    }

    // Check environment variable
    std::env::var(NAMESPACE_ENV).ok()
}
