//! Phase I - Verify/Integrate Pipeline
//!
//! Validate completely before commit.
//! Combines: cortex_execute (build) + tests + error checking
//!
//! Workflow from instructions:
//! 1. cortex_execute: build complet OK
//! 2. runTests: TOUS les tests passent
//! 3. get_errors: zero erreurs
//! 4. Commit si TOUT passe

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// What to verify
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerifyCheck {
    /// Run full build
    Build,
    /// Run all tests
    Test,
    /// Run linter (clippy, eslint)
    Lint,
    /// Check formatting
    Format,
    /// Check types (tsc --noEmit, cargo check)
    Types,
    /// All checks
    All,
}

/// Language preset for verification
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum VerifyLanguage {
    /// Rust: cargo build, cargo test, cargo clippy, cargo fmt --check
    Rust,
    /// TypeScript: tsc, npm test, eslint, prettier
    TypeScript,
    /// Python: python -m py_compile, pytest, ruff, black
    Python,
    /// Custom: use custom_commands
    Custom,
}

/// Custom command definition for verification
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerifyCommand {
    /// Check type this command performs
    pub check_type: VerifyCheck,
    /// Command to execute
    pub command: String,
    /// Label for reporting
    #[serde(default)]
    pub label: Option<String>,
    /// Timeout in seconds
    #[serde(default = "default_timeout")]
    pub timeout_secs: u64,
}

fn default_timeout() -> u64 {
    120
}

/// Parameters for the verify pipeline
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerifyParams {
    /// Which checks to run (default: All)
    #[serde(default = "default_checks")]
    pub checks: Vec<VerifyCheck>,

    /// Language preset to use
    #[serde(default)]
    pub language: Option<VerifyLanguage>,

    /// Working directory
    #[serde(default)]
    pub cwd: Option<String>,

    /// Custom commands (overrides language preset)
    #[serde(default)]
    pub custom_commands: Vec<VerifyCommand>,

    /// Environment variables
    #[serde(default)]
    pub env: HashMap<String, String>,

    /// Stop on first failure (default: false for integration)
    #[serde(default)]
    pub stop_on_failure: bool,

    /// Generate coverage report (default: false)
    #[serde(default)]
    pub coverage: bool,

    /// Test filter pattern (run specific tests)
    #[serde(default)]
    pub test_filter: Option<String>,

    /// Strict mode: fail on warnings too (default: false)
    #[serde(default)]
    pub strict: bool,
}

fn default_checks() -> Vec<VerifyCheck> {
    vec![VerifyCheck::All]
}

/// Result of a single verification check
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct CheckResult {
    /// Check type that was performed
    pub check_type: String,
    /// Command executed
    pub command: String,
    /// Label
    pub label: Option<String>,
    /// Whether check passed
    pub passed: bool,
    /// Exit code
    pub exit_code: i32,
    /// Output (truncated if too long)
    pub output: String,
    /// Duration in milliseconds
    pub duration_ms: u64,
    /// Error count (parsed from output)
    pub error_count: usize,
    /// Warning count (parsed from output)
    pub warning_count: usize,
}

/// Test results summary
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct TestSummary {
    /// Total tests
    pub total: usize,
    /// Passed tests
    pub passed: usize,
    /// Failed tests
    pub failed: usize,
    /// Skipped tests
    pub skipped: usize,
    /// Coverage percentage (if coverage=true)
    pub coverage_percent: Option<f32>,
}

/// Result from the verify pipeline
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct VerifyResult {
    /// Results of each check
    pub checks: Vec<CheckResult>,

    /// Test summary (if tests were run)
    pub test_summary: Option<TestSummary>,

    /// All checks passed
    pub all_passed: bool,

    /// Ready to commit
    pub ready_to_commit: bool,

    /// Total checks run
    pub total_checks: usize,

    /// Checks passed
    pub checks_passed: usize,

    /// Checks failed
    pub checks_failed: usize,

    /// Total errors across all checks
    pub total_errors: usize,

    /// Total warnings across all checks
    pub total_warnings: usize,

    /// Total duration in milliseconds
    pub total_duration_ms: u64,

    /// Summary message
    pub summary: String,

    /// Blocking issues (must fix before commit)
    #[serde(default)]
    pub blockers: Vec<String>,

    /// Non-blocking issues (should fix)
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl Default for VerifyParams {
    fn default() -> Self {
        Self {
            checks: vec![VerifyCheck::All],
            language: None,
            cwd: None,
            custom_commands: Vec::new(),
            env: HashMap::new(),
            stop_on_failure: false,
            coverage: false,
            test_filter: None,
            strict: false,
        }
    }
}

impl VerifyParams {}

impl VerifyLanguage {
    /// Get default commands for this language
    pub fn default_commands(&self) -> Vec<VerifyCommand> {
        match self {
            VerifyLanguage::Rust => vec![
                VerifyCommand {
                    check_type: VerifyCheck::Types,
                    command: "cargo check".to_string(),
                    label: Some("Rust type check".to_string()),
                    timeout_secs: 120,
                },
                VerifyCommand {
                    check_type: VerifyCheck::Build,
                    command: "cargo build".to_string(),
                    label: Some("Rust build".to_string()),
                    timeout_secs: 300,
                },
                VerifyCommand {
                    check_type: VerifyCheck::Test,
                    command: "cargo test".to_string(),
                    label: Some("Rust tests".to_string()),
                    timeout_secs: 300,
                },
                VerifyCommand {
                    check_type: VerifyCheck::Lint,
                    command: "cargo clippy -- -D warnings".to_string(),
                    label: Some("Rust lint".to_string()),
                    timeout_secs: 180,
                },
                VerifyCommand {
                    check_type: VerifyCheck::Format,
                    command: "cargo fmt --check".to_string(),
                    label: Some("Rust format".to_string()),
                    timeout_secs: 30,
                },
            ],
            VerifyLanguage::TypeScript => vec![
                VerifyCommand {
                    check_type: VerifyCheck::Types,
                    command: "npx tsc --noEmit".to_string(),
                    label: Some("TypeScript check".to_string()),
                    timeout_secs: 120,
                },
                VerifyCommand {
                    check_type: VerifyCheck::Build,
                    command: "npm run build".to_string(),
                    label: Some("npm build".to_string()),
                    timeout_secs: 300,
                },
                VerifyCommand {
                    check_type: VerifyCheck::Test,
                    command: "npm test".to_string(),
                    label: Some("npm test".to_string()),
                    timeout_secs: 300,
                },
                VerifyCommand {
                    check_type: VerifyCheck::Lint,
                    command: "npx eslint .".to_string(),
                    label: Some("ESLint".to_string()),
                    timeout_secs: 120,
                },
                VerifyCommand {
                    check_type: VerifyCheck::Format,
                    command: "npx prettier --check .".to_string(),
                    label: Some("Prettier".to_string()),
                    timeout_secs: 60,
                },
            ],
            VerifyLanguage::Python => vec![
                VerifyCommand {
                    check_type: VerifyCheck::Types,
                    command: "python -m py_compile *.py".to_string(),
                    label: Some("Python syntax".to_string()),
                    timeout_secs: 30,
                },
                VerifyCommand {
                    check_type: VerifyCheck::Test,
                    command: "pytest".to_string(),
                    label: Some("pytest".to_string()),
                    timeout_secs: 300,
                },
                VerifyCommand {
                    check_type: VerifyCheck::Lint,
                    command: "ruff check .".to_string(),
                    label: Some("Ruff lint".to_string()),
                    timeout_secs: 60,
                },
                VerifyCommand {
                    check_type: VerifyCheck::Format,
                    command: "black --check .".to_string(),
                    label: Some("Black format".to_string()),
                    timeout_secs: 60,
                },
            ],
            VerifyLanguage::Custom => vec![],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_default_commands() {
        let commands = VerifyLanguage::Rust.default_commands();

        assert_eq!(commands.len(), 5);
        assert!(commands.iter().any(|c| c.command.contains("cargo check")));
        assert!(commands.iter().any(|c| c.command.contains("cargo test")));
        assert!(commands.iter().any(|c| c.command.contains("cargo clippy")));
    }

    #[test]
    fn test_verify_result() {
        let result = VerifyResult {
            checks: vec![CheckResult {
                check_type: "build".to_string(),
                command: "cargo build".to_string(),
                label: Some("Rust build".to_string()),
                passed: true,
                exit_code: 0,
                output: "Finished".to_string(),
                duration_ms: 5000,
                error_count: 0,
                warning_count: 2,
            }],
            test_summary: Some(TestSummary {
                total: 74,
                passed: 74,
                failed: 0,
                skipped: 0,
                coverage_percent: Some(85.5),
            }),
            all_passed: true,
            ready_to_commit: true,
            total_checks: 1,
            checks_passed: 1,
            checks_failed: 0,
            total_errors: 0,
            total_warnings: 2,
            total_duration_ms: 5000,
            summary: "All checks passed".to_string(),
            blockers: vec![],
            warnings: vec!["2 warnings".to_string()],
        };

        assert!(result.ready_to_commit);
        assert_eq!(result.test_summary.unwrap().passed, 74);
    }
}
