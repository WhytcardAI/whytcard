//! Common test utilities and fixtures
//!
//! Provides shared setup for all integration tests.

use std::path::Path;
use tempfile::TempDir;

/// Test context containing server and temporary resources
pub struct TestContext {
    pub server: whytcard_intelligence::IntelligenceServer,
    pub temp_dir: TempDir,
}

impl TestContext {
    /// Create a new test context with isolated environment
    pub async fn new() -> Self {
        let temp_dir = TempDir::new().expect("Failed to create temp directory");
        let server = whytcard_intelligence::IntelligenceServer::for_testing(temp_dir.path())
            .await
            .expect("Failed to create test server");

        Self { server, temp_dir }
    }

    /// Get the path to the temporary directory
    pub fn path(&self) -> &Path {
        self.temp_dir.path()
    }
}

/// Helper macro to create test parameters with JSON
#[macro_export]
macro_rules! json_params {
    ($($json:tt)+) => {
        serde_json::from_value(serde_json::json!($($json)+))
            .expect("Failed to parse test parameters")
    };
}

/// Helper to assert result is successful
#[macro_export]
macro_rules! assert_success {
    ($result:expr) => {
        assert!($result.is_ok(), "Expected success but got: {:?}", $result.err());
    };
}

/// Helper to assert result is an error
#[macro_export]
macro_rules! assert_error {
    ($result:expr) => {
        assert!($result.is_err(), "Expected error but got success");
    };
}

/// Generate random test data
pub fn random_key() -> String {
    uuid::Uuid::new_v4().to_string()
}

/// Generate test content with unique identifier
pub fn test_content(prefix: &str) -> String {
    format!("{}-{}", prefix, random_key())
}

/// Wait for async operations to complete (for stress tests)
pub async fn wait_ms(ms: u64) {
    tokio::time::sleep(std::time::Duration::from_millis(ms)).await;
}
