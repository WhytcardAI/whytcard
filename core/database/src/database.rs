//! Main database connection and operations

use crate::config::StorageMode;
use crate::{Config, Result, Schema};
use surrealdb::engine::local::Db;
use surrealdb::Surreal;
use std::sync::Arc;

/// WhytCard database instance
#[derive(Clone)]
pub struct Database {
    /// Inner SurrealDB connection
    inner: Arc<Surreal<Db>>,

    /// Configuration
    config: Arc<Config>,
}

impl Database {
    /// Create a new in-memory database
    pub async fn new_memory() -> Result<Self> {
        Self::new(Config::memory()).await
    }

    /// Create a new persistent database
    pub async fn new_persistent(path: impl Into<std::path::PathBuf>) -> Result<Self> {
        Self::new(Config::persistent(path)).await
    }

    /// Create a new database with custom configuration
    pub async fn new(config: Config) -> Result<Self> {
        // Connect based on storage mode
        let db: Surreal<Db> = match &config.storage {
            StorageMode::Memory => {
                let db = Surreal::new::<surrealdb::engine::local::Mem>(()).await?;
                tracing::info!("Connected to in-memory database");
                db
            }
            StorageMode::Persistent(path) => {
                // Ensure directory exists
                if let Some(parent) = path.parent() {
                    std::fs::create_dir_all(parent).ok();
                }
                let db = Surreal::new::<surrealdb::engine::local::RocksDb>(path.to_string_lossy().as_ref()).await?;
                tracing::info!("Connected to persistent database at {:?}", path);
                db
            }
        };

        // Select namespace and database
        db.use_ns(&config.namespace).use_db(&config.database).await?;

        // Initialize schema before wrapping in Arc
        Schema::init(&db, &config).await?;

        let database = Self {
            inner: Arc::new(db),
            config: Arc::new(config),
        };

        Ok(database)
    }

    /// Get the inner SurrealDB connection
    pub fn inner(&self) -> &Surreal<Db> {
        &self.inner
    }

    /// Get the configuration
    pub fn config(&self) -> &Config {
        &self.config
    }

    /// Execute a raw query
    pub async fn query(&self, query: &str) -> Result<surrealdb::Response> {
        Ok(self.inner.query(query).await?)
    }

    /// Check if the database is healthy
    pub async fn health(&self) -> Result<bool> {
        let result: Option<i32> = self.inner.query("RETURN 1").await?.take(0)?;
        Ok(result == Some(1))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_database() {
        let db = Database::new_memory().await.unwrap();
        assert!(db.health().await.unwrap());
    }

    #[tokio::test]
    async fn test_custom_config() {
        let config = Config::memory()
            .with_namespace("test")
            .with_database("testdb")
            .with_dimension(768);

        let db = Database::new(config).await.unwrap();
        assert!(db.health().await.unwrap());
        assert_eq!(db.config().namespace, "test");
        assert_eq!(db.config().database, "testdb");
        assert_eq!(db.config().vector_config.dimension, 768);
    }
}
