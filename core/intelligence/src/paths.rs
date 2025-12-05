//! Cross-platform data path resolution for WhytCard
//!
//! Resolves data paths in this priority order:
//! 1. `WHYTCARD_DATA_DIR` environment variable
//! 2. Platform-specific application data directory
//!
//! # Platform Defaults
//!
//! | Platform | Path |
//! |----------|------|
//! | Windows  | `%LOCALAPPDATA%\WhytCard\` |
//! | Linux    | `~/.local/share/whytcard/` |
//! | macOS    | `~/Library/Application Support/WhytCard/` |

use crate::error::{IntelligenceError, Result};
use directories::ProjectDirs;
use std::path::{Path, PathBuf};

/// Application identifier constants
const APP_QUALIFIER: &str = "com";
const APP_ORGANIZATION: &str = "WhytCard";
const APP_NAME: &str = "WhytCard";

/// Environment variable for custom data directory
const DATA_DIR_ENV: &str = "WHYTCARD_DATA_DIR";

/// Resolved data paths for WhytCard
#[derive(Debug, Clone)]
pub struct DataPaths {
    /// Root data directory
    pub root: PathBuf,
    /// SQLite database path
    pub database: PathBuf,
    /// Vector store directory
    pub vectors: PathBuf,
    /// CORTEX memory directory (separate from main vectors)
    pub cortex_memory: PathBuf,
    /// Models directory (ONNX)
    pub models: PathBuf,
    /// Logs directory
    pub logs: PathBuf,
    /// Config file path
    pub config: PathBuf,
    /// Namespace (for multi-instance support)
    pub namespace: Option<String>,
}

impl DataPaths {
    /// Resolve data paths from environment or platform defaults
    pub fn resolve() -> Result<Self> {
        let root = Self::resolve_root()?;
        Ok(Self::from_root(root))
    }

    /// Resolve data paths with a specific namespace
    pub fn resolve_with_namespace(namespace: &str) -> Result<Self> {
        let root = Self::resolve_root()?;
        Ok(Self::from_root_with_namespace(root, namespace))
    }

    /// Create paths from a specific root directory
    pub fn from_root(root: PathBuf) -> Self {
        Self {
            database: root.join("whytcard.db"),
            vectors: root.join("vectors"),
            cortex_memory: root.join("cortex"),
            models: root.join("models"),
            logs: root.join("logs"),
            config: root.join("config.toml"),
            namespace: None,
            root,
        }
    }

    /// Create paths from a root with a namespace subdirectory
    pub fn from_root_with_namespace(root: PathBuf, namespace: &str) -> Self {
        let namespaced_root = root.join("namespaces").join(namespace);
        Self {
            database: namespaced_root.join("whytcard.db"),
            vectors: namespaced_root.join("vectors"),
            cortex_memory: namespaced_root.join("cortex"),
            // Models are shared across namespaces (they're large)
            models: root.join("models"),
            logs: namespaced_root.join("logs"),
            config: namespaced_root.join("config.toml"),
            namespace: Some(namespace.to_string()),
            root: namespaced_root,
        }
    }

    /// Apply namespace to existing paths (creates namespaced subdirectory)
    pub fn with_namespace(self, namespace: &str) -> Self {
        // Get the original root (before any namespace was applied)
        let original_root = if self.namespace.is_some() {
            // Already namespaced, go up two levels to get original root
            self.root.parent().and_then(|p| p.parent()).map(|p| p.to_path_buf())
                .unwrap_or(self.root.clone())
        } else {
            self.root.clone()
        };
        Self::from_root_with_namespace(original_root, namespace)
    }

    /// Create paths for testing (in-memory or temp)
    pub fn for_testing(temp_dir: &Path) -> Self {
        Self::from_root(temp_dir.to_path_buf())
    }

    /// Ensure all directories exist
    pub fn ensure_directories(&self) -> Result<()> {
        std::fs::create_dir_all(&self.root)?;
        std::fs::create_dir_all(&self.vectors)?;
        std::fs::create_dir_all(&self.cortex_memory)?;
        std::fs::create_dir_all(&self.models)?;
        std::fs::create_dir_all(&self.logs)?;
        Ok(())
    }

    /// Resolve the root data directory
    fn resolve_root() -> Result<PathBuf> {
        // 1. Check environment variable
        if let Ok(custom_path) = std::env::var(DATA_DIR_ENV) {
            let path = PathBuf::from(custom_path);
            tracing::info!("Using custom data directory from {}: {:?}", DATA_DIR_ENV, path);
            return Ok(path);
        }

        // 2. Use platform-specific directory
        let project_dirs = ProjectDirs::from(APP_QUALIFIER, APP_ORGANIZATION, APP_NAME)
            .ok_or_else(|| {
                IntelligenceError::path(
                    "Could not determine platform-specific data directory. \
                     Set WHYTCARD_DATA_DIR environment variable.",
                )
            })?;

        let path = project_dirs.data_local_dir().to_path_buf();
        tracing::info!("Using platform data directory: {:?}", path);
        Ok(path)
    }

    /// Get the database connection string for SQLite
    pub fn database_url(&self) -> String {
        format!("sqlite:{}", self.database.display())
    }

    /// Check if this is a fresh installation (no database exists)
    pub fn is_fresh_install(&self) -> bool {
        !self.database.exists()
    }
}

impl Default for DataPaths {
    fn default() -> Self {
        Self::resolve().unwrap_or_else(|_| {
            // Fallback to current directory
            Self::from_root(PathBuf::from("."))
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_from_root() {
        let root = PathBuf::from("/test/data");
        let paths = DataPaths::from_root(root.clone());

        assert_eq!(paths.root, root);
        assert_eq!(paths.database, root.join("whytcard.db"));
        assert_eq!(paths.vectors, root.join("vectors"));
        assert_eq!(paths.cortex_memory, root.join("cortex"));
        assert_eq!(paths.models, root.join("models"));
        assert_eq!(paths.logs, root.join("logs"));
        assert_eq!(paths.config, root.join("config.toml"));
    }

    #[test]
    fn test_ensure_directories() {
        let temp = TempDir::new().unwrap();
        let paths = DataPaths::for_testing(temp.path());

        paths.ensure_directories().unwrap();

        assert!(paths.root.exists());
        assert!(paths.vectors.exists());
        assert!(paths.cortex_memory.exists());
        assert!(paths.models.exists());
        assert!(paths.logs.exists());
    }

    #[test]
    fn test_database_url() {
        let paths = DataPaths::from_root(PathBuf::from("/data"));
        let url = paths.database_url();

        assert!(url.starts_with("sqlite:"));
        assert!(url.contains("whytcard.db"));
    }

    #[test]
    fn test_is_fresh_install() {
        let temp = TempDir::new().unwrap();
        let paths = DataPaths::for_testing(temp.path());

        assert!(paths.is_fresh_install());
    }

    #[test]
    fn test_env_override() {
        let temp = TempDir::new().unwrap();
        let custom_path = temp.path().to_str().unwrap();

        // Set environment variable
        std::env::set_var(DATA_DIR_ENV, custom_path);

        let paths = DataPaths::resolve().unwrap();
        assert_eq!(paths.root, temp.path());

        // Cleanup
        std::env::remove_var(DATA_DIR_ENV);
    }
}
