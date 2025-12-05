//! MCP Server implementation for WhytCard Intelligence
//!
//! Uses the official rmcp SDK for MCP protocol handling.

use crate::config::IntelligenceConfig;
use crate::cortex::{CortexConfig, CortexEngine};
use crate::error::IntelligenceError;
use crate::integrations::{Context7Client, IntegrationClient, MSLearnClient, TavilyClient};
use crate::mcp_client::{InstalledMcpServer, McpClientManager, McpConfigManager, PredefinedServers, SequentialThinkingClient};
use crate::tools::{
    // CORTEX tools
    CortexCleanupParams, CortexCleanupResult, CortexExecuteParams, CortexExecuteResult,
    CortexFeedbackParams, CortexFeedbackResult, CortexInstructionsParams, CortexInstructionsResult,
    CortexProcessParams, CortexProcessResult, CortexStatsParams, CortexStatsResult,
    InstructionInfo, InstructionsAction,
    // External tools
    ExternalDocsParams, ExternalDocsResult, ExternalMcpCallParams,
    ExternalMcpCallResult, ExternalSearchParams, ExternalSearchResult, KeyRequiredServer,
    McpAvailableServersParams, McpAvailableServersResult, McpConfigureParams, McpConfigureResult,
    McpConnectParams, McpConnectResult, McpDisconnectParams, McpDisconnectResult,
    McpInstallParams, McpInstallResult, McpListInstalledParams, McpListInstalledResult,
    McpListToolsParams, McpListToolsResult, McpServerInfo, McpServerStatus, McpStatusParams,
    McpStatusResult, McpToolDetail, McpUninstallParams, McpUninstallResult, SearchResultItem,
    SequentialThinkingParams, SequentialThinkingResult, ServerDescription, ThinkingStep, ToolInfo,
    // Knowledge tools
    EntityInfo, ExportGraphParams, ExportGraphResult, KnowledgeAddEntityParams,
    KnowledgeAddEntityResult, KnowledgeAddObservationParams, KnowledgeAddObservationResult,
    KnowledgeAddRelationParams, KnowledgeAddRelationResult, KnowledgeDeleteEntityParams,
    KnowledgeDeleteEntityResult, KnowledgeDeleteObservationParams, KnowledgeDeleteObservationResult,
    KnowledgeDeleteRelationParams, KnowledgeDeleteRelationResult, KnowledgeFindPathParams,
    KnowledgeFindPathResult, KnowledgeGetEntityParams, KnowledgeGetEntityResult,
    KnowledgeGetNeighborsParams, KnowledgeGetNeighborsResult, KnowledgeReadGraphParams,
    KnowledgeReadGraphResult, KnowledgeSearchParams, KnowledgeSearchResult, NeighborInfo,
    // Memory tools
    BatchStoreParams, BatchStoreResult, ContextScores, EpisodicItem,
    GetContextParams, GetContextResult, HybridSearchParams, HybridSearchResult,
    ManageTagsParams, ManageTagsResult, MemoryDeleteParams, MemoryDeleteResult, MemoryGetParams,
    MemoryGetResult, MemoryListParams, MemoryListResult, MemorySearchParams, MemorySearchResult,
    MemoryStoreParams, MemoryStoreResult, ProceduralItem, RelationInfo, SemanticItem,
    // Pipeline types (ACID workflow)
    pipelines::{
        AnalyzeParams, AnalyzeResult, AnalyzeSource, PipelineResponse,
        PrepareParams, PrepareResult,
        CodeParams, CodeResult,
        VerifyParams, VerifyResult, VerifyCheck,
        DocumentParams, DocumentResult,
        ManageParams, ManageResult, ManageAction,
    },
};
use rmcp::{
    handler::server::router::tool::ToolRouter,
    model::*,
    tool, tool_handler, tool_router,
    ErrorData as McpError, Json, ServiceExt,
};
use std::sync::Arc;
use tokio::sync::RwLock;
use whytcard_database::{
    Config as DbConfig, CreateEntity, CreateRelation, Database, StorageMode,
    VectorConfig,
};
use whytcard_rag::RagEngine;

/// WhytCard Intelligence MCP Server
#[derive(Clone)]
pub struct IntelligenceServer {
    /// Configuration
    config: Arc<IntelligenceConfig>,

    /// Database (SurrealDB)
    db: Arc<Database>,

    /// RAG engine
    rag: Arc<RwLock<RagEngine>>,

    /// CORTEX cognitive engine
    cortex: Arc<CortexEngine>,

    /// Context7 client for library documentation
    context7: Arc<RwLock<Context7Client>>,

    /// Tavily client for web search
    tavily: Arc<RwLock<TavilyClient>>,

    /// MS Learn client for Microsoft documentation
    mslearn: Arc<RwLock<MSLearnClient>>,

    /// Sequential thinking client (internal implementation)
    thinking: Arc<RwLock<SequentialThinkingClient>>,

    /// MCP client manager for external MCP servers
    mcp_clients: Arc<McpClientManager>,

    /// MCP configuration manager for persistence
    mcp_config: Arc<RwLock<McpConfigManager>>,

    /// Tool router
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl IntelligenceServer {
    /// Create a new Intelligence server
    pub async fn new(config: IntelligenceConfig) -> crate::Result<Self> {
        let paths = config.get_paths()?;
        paths.ensure_directories()?;

        tracing::info!("Initializing Intelligence server");
        tracing::info!("Data directory: {:?}", paths.root);

        // Initialize SurrealDB database
        let storage = if paths.database.to_str().is_some_and(|s| s.contains(":memory:")) {
            StorageMode::Memory
        } else {
            StorageMode::Persistent(paths.database.clone())
        };

        let db_config = DbConfig {
            storage,
            namespace: "whytcard".into(),
            database: "intelligence".into(),
            vector_config: VectorConfig {
                dimension: config.rag.model.dimensions(),
                distance: whytcard_database::DistanceMetric::Cosine,
            },
        };

        tracing::info!("Initializing database: {:?}", paths.database);
        let db = Database::new(db_config).await?;

        // Initialize RAG engine
        tracing::info!("Initializing RAG engine: {:?}", paths.vectors);
        let rag = whytcard_rag::RagEngineBuilder::new()
            .db_path(paths.vectors.to_str().unwrap_or("vectors"))
            .embedding_model(config.rag.model.clone())
            .chunking_config(config.rag.chunking.clone())
            .search_config(config.rag.search.clone())
            .build()
            .await?;

        // Initialize CORTEX cognitive engine
        tracing::info!("Initializing CORTEX engine");
        let cortex_config = CortexConfig::default();
        let cortex = CortexEngine::new(&paths.root, cortex_config).await?;

        // Initialize integration clients
        tracing::info!("Initializing external integration clients");
        let mut context7 = Context7Client::from_env();
        let mut tavily = TavilyClient::from_env();
        let mut mslearn = MSLearnClient::new();

        // Try to initialize clients (non-blocking, failures are logged)
        if let Err(e) = context7.initialize().await {
            tracing::warn!("Context7 initialization failed: {}", e);
        }
        if let Err(e) = tavily.initialize().await {
            tracing::warn!("Tavily initialization failed: {}", e);
        }
        if let Err(e) = mslearn.initialize().await {
            tracing::warn!("MS Learn initialization failed: {}", e);
        }

        let thinking = SequentialThinkingClient::new();

        // Initialize MCP client manager for external MCP servers
        tracing::info!("Initializing MCP client manager");
        let mcp_clients = McpClientManager::new();

        // Initialize MCP configuration manager for persistence
        // Use core/mcp/ directory for local-first installation
        let mcp_dir = paths.root
            .parent()
            .unwrap_or(&paths.root)
            .join("mcp");

        tracing::info!("MCP directory: {:?}", mcp_dir);

        let mcp_config = McpConfigManager::new(&mcp_dir)
            .map_err(|e| IntelligenceError::Config(format!("MCP config error: {}", e)))?;

        // Load installed servers from persistent config
        for (name, server) in mcp_config.list_all() {
            if server.enabled {
                let config = server.to_server_config();
                mcp_clients.add_config(config).await;
                tracing::debug!("Loaded MCP server from config: {}", name);
            }
        }

        // Register predefined MCP servers (connection is lazy)
        mcp_clients.add_config(PredefinedServers::sequential_thinking()).await;
        mcp_clients.add_config(PredefinedServers::context7()).await;
        mcp_clients.add_config(PredefinedServers::playwright()).await;
        mcp_clients.add_config(PredefinedServers::memory()).await;
        mcp_clients.add_config(PredefinedServers::microsoft_learn()).await;
        mcp_clients.add_config(PredefinedServers::markitdown()).await;
        mcp_clients.add_config(PredefinedServers::chrome_devtools()).await;

        // Add Tavily if API key is available
        if let Ok(api_key) = std::env::var("TAVILY_API_KEY") {
            mcp_clients.add_config(PredefinedServers::tavily(&api_key)).await;
        }

        Ok(Self {
            config: Arc::new(config),
            db: Arc::new(db),
            rag: Arc::new(RwLock::new(rag)),
            cortex: Arc::new(cortex),
            context7: Arc::new(RwLock::new(context7)),
            tavily: Arc::new(RwLock::new(tavily)),
            mslearn: Arc::new(RwLock::new(mslearn)),
            thinking: Arc::new(RwLock::new(thinking)),
            mcp_clients: Arc::new(mcp_clients),
            mcp_config: Arc::new(RwLock::new(mcp_config)),
            tool_router: Self::tool_router(),
        })
    }

    /// Create server for testing with in-memory database
    #[cfg(test)]
    pub async fn for_testing(temp_dir: &std::path::Path) -> crate::Result<Self> {
        let paths = DataPaths::for_testing(temp_dir);
        paths.ensure_directories()?;

        let db_config = DbConfig {
            storage: StorageMode::Memory,
            namespace: "whytcard".into(),
            database: "test".into(),
            vector_config: VectorConfig::default(),
        };
        let db = Database::new(db_config).await?;

        // Use in-memory RAG for tests
        let rag = whytcard_rag::RagEngineBuilder::new()
            .db_path(":memory:")
            .min_chunk_size(10)
            .build()
            .await?;

        // Initialize CORTEX for testing
        let cortex_config = CortexConfig::default();
        let cortex = CortexEngine::new(temp_dir, cortex_config).await?;

        // Create non-initialized clients for testing
        let context7 = Context7Client::new(None);
        let tavily = TavilyClient::new(None);
        let mslearn = MSLearnClient::new();
        let thinking = SequentialThinkingClient::new();
        let mcp_clients = McpClientManager::new();
        let mcp_config = McpConfigManager::new(temp_dir)
            .map_err(|e| IntelligenceError::Config(format!("MCP config error: {}", e)))?;

        Ok(Self {
            config: Arc::new(IntelligenceConfig::default()),
            paths: Arc::new(paths),
            db: Arc::new(db),
            rag: Arc::new(RwLock::new(rag)),
            cortex: Arc::new(cortex),
            context7: Arc::new(RwLock::new(context7)),
            tavily: Arc::new(RwLock::new(tavily)),
            mslearn: Arc::new(RwLock::new(mslearn)),
            thinking: Arc::new(RwLock::new(thinking)),
            mcp_clients: Arc::new(mcp_clients),
            mcp_config: Arc::new(RwLock::new(mcp_config)),
            tool_router: Self::tool_router(),
        })
    }

    // ========================================================================
    // MEMORY TOOLS
    // ========================================================================

    #[tool(description = "Store information in persistent memory with optional semantic indexing")]
    async fn memory_store(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<MemoryStoreParams>,
    ) -> std::result::Result<Json<MemoryStoreResult>, McpError> {
        let params = params.0;
        let key = params
            .key
            .unwrap_or_else(|| uuid::Uuid::new_v4().to_string());

        let now = chrono::Utc::now().timestamp();

        // Store in database as a document
        let doc_input = whytcard_database::CreateDocument::new(&params.content)
            .with_key(&key)
            .with_metadata(params.metadata.unwrap_or_default())
            .with_tags(params.tags);

        // Add title if present
        let doc_input = if let Some(title) = params.title {
            doc_input.with_title(title)
        } else {
            doc_input
        };

        self.db
            .create_document(doc_input)
            .await
            .map_err(IntelligenceError::from)?;

        let mut indexed = false;

        // Index in RAG if enabled
        if params.index && self.config.rag.auto_index {
            let doc = whytcard_rag::Document::new(&params.content)
                .with_id(&key)
                .with_metadata_field("type", "memory")
                .with_metadata_field("key", key.clone());

            let mut rag = self.rag.write().await;
            if let Err(e) = rag.index(&doc).await {
                tracing::warn!("Failed to index memory in RAG: {}", e);
            } else {
                indexed = true;
            }
        }

        Ok(Json(MemoryStoreResult {
            key,
            indexed,
            stored_at: now,
        }))
    }

    #[tool(description = "Search memories using semantic search")]
    async fn memory_search(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<MemorySearchParams>,
    ) -> std::result::Result<Json<MemorySearchResult>, McpError> {
        let params = params.0;

        let mut rag = self.rag.write().await;
        let results = rag
            .search(&params.query, Some(params.limit))
            .await
            .map_err(IntelligenceError::from)?;

        let items = results
            .into_iter()
            .filter(|r| params.min_score.is_none_or(|min| r.score >= min))
            .map(|r| {
                // Extract title from metadata if present
                let title = r.chunk.metadata.as_ref().and_then(|m| {
                    m.get("title").and_then(|v| v.as_str()).map(String::from)
                });

                crate::tools::MemorySearchResultItem {
                    key: r.chunk.document_id.clone(),
                    content: r.chunk.text.clone(),
                    title,
                    score: r.score,
                    tags: Vec::new(),
                    stored_at: 0,
                }
            })
            .collect::<Vec<_>>();

        let total = items.len();

        Ok(Json(MemorySearchResult {
            results: items,
            total,
            query: params.query,
        }))
    }

    #[tool(description = "Get a specific memory by key")]
    async fn memory_get(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<MemoryGetParams>,
    ) -> std::result::Result<Json<MemoryGetResult>, McpError> {
        let params = params.0;

        let doc = self
            .db
            .get_document_by_key(&params.key)
            .await
            .map_err(IntelligenceError::from)?
            .ok_or_else(|| IntelligenceError::KeyNotFound(params.key.clone()))?;

        Ok(Json(MemoryGetResult {
            key: doc.key.unwrap_or_default(),
            content: doc.content,
            title: doc.title,
            tags: doc.tags,
            metadata: doc.metadata,
            stored_at: doc.created_at.map(|d| d.timestamp()).unwrap_or(0),
            updated_at: doc.updated_at.map(|d| d.timestamp()).unwrap_or(0),
        }))
    }

    #[tool(description = "Delete a memory by key")]
    async fn memory_delete(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<MemoryDeleteParams>,
    ) -> std::result::Result<Json<MemoryDeleteResult>, McpError> {
        let params = params.0;

        // Delete from database
        let deleted_from_db = self
            .db
            .delete_document_by_key(&params.key)
            .await
            .map_err(IntelligenceError::from)?;

        // Delete from RAG index (key is used as document_id)
        let mut rag = self.rag.write().await;
        if let Err(e) = rag.delete_document(&params.key).await {
            tracing::warn!("Failed to delete memory from RAG: {}", e);
        }

        Ok(Json(MemoryDeleteResult {
            key: params.key,
            deleted: deleted_from_db,
        }))
    }

    #[tool(description = "List all memories with pagination")]
    async fn memory_list(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<MemoryListParams>,
    ) -> std::result::Result<Json<MemoryListResult>, McpError> {
        let params = params.0;

        // Fetch limit + 1 to check if there are more results
        let fetch_limit = params.limit + 1;

        let mut docs = self
            .db
            .list_documents(
                if params.tags.is_empty() {
                    None
                } else {
                    Some(&params.tags)
                },
                fetch_limit,
                params.offset,
            )
            .await
            .map_err(IntelligenceError::from)?;

        let has_more = docs.len() > params.limit;
        if has_more {
            docs.pop();
        }

        let memories = docs
            .into_iter()
            .map(|d| crate::tools::MemorySummary {
                key: d.key.unwrap_or_default(),
                title: d.title.unwrap_or_else(|| {
                    d.content.chars().take(50).collect::<String>() + "..."
                }),
                tags: d.tags,
                stored_at: d.created_at.map(|d| d.timestamp()).unwrap_or(0),
            })
            .collect();

        // Total count would require a separate query, returning 0 for now as it's optional
        let total = 0;

        Ok(Json(MemoryListResult {
            memories,
            total,
            has_more,
        }))
    }

    #[tool(description = "Store multiple memories in batch with optional semantic indexing")]
    async fn batch_store(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<BatchStoreParams>,
    ) -> std::result::Result<Json<BatchStoreResult>, McpError> {
        let params = params.0;

        let mut stored = 0;
        let mut keys = Vec::new();
        let mut errors = Vec::new();

        for item in params.items {
            let key = uuid::Uuid::new_v4().to_string();

            // Store in database
            let doc_input = whytcard_database::CreateDocument::new(&item.content)
                .with_key(&key)
                .with_metadata(item.metadata.clone().unwrap_or_default())
                .with_tags(item.tags.clone());

            match self.db.create_document(doc_input).await {
                Ok(_) => {
                    stored += 1;
                    keys.push(key.clone());

                    // Index in RAG if enabled
                    if self.config.rag.auto_index {
                        let doc = whytcard_rag::Document::new(&item.content)
                            .with_id(&key)
                            .with_metadata_field("type", "memory")
                            .with_metadata_field("source", item.source.clone())
                            .with_metadata_field("category", item.category.clone());

                        let mut rag = self.rag.write().await;
                        if let Err(e) = rag.index(&doc).await {
                            tracing::warn!("Failed to index batch item: {}", e);
                        }
                    }
                }
                Err(e) => {
                    errors.push(format!("Failed to store item: {}", e));
                }
            }
        }

        Ok(Json(BatchStoreResult {
            stored,
            keys,
            errors,
        }))
    }

    #[tool(description = "Perform hybrid search across semantic, episodic and procedural memories")]
    async fn hybrid_search(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<HybridSearchParams>,
    ) -> std::result::Result<Json<HybridSearchResult>, McpError> {
        let params = params.0;
        let limit = params.top_k;
        let min_score = params.min_relevance;

        // Semantic search via RAG
        let mut semantic = Vec::new();
        {
            let mut rag = self.rag.write().await;
            if let Ok(results) = rag.search(&params.query, Some(limit)).await {
                semantic = results
                    .into_iter()
                    .filter(|r| r.score >= min_score)
                    .map(|r| SemanticItem {
                        id: r.chunk.document_id.clone(),
                        content: r.chunk.text.clone(),
                        source: r.chunk.metadata.as_ref()
                            .and_then(|m| m.get("source"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("unknown")
                            .to_string(),
                        score: r.score,
                        category: r.chunk.metadata.as_ref()
                            .and_then(|m| m.get("category"))
                            .and_then(|v| v.as_str())
                            .unwrap_or("general")
                            .to_string(),
                        tags: Vec::new(),
                    })
                    .collect();
            }
        }

        // Episodic search via CORTEX
        let mut episodic = Vec::new();
        if let Ok(episodes) = self.cortex.search_episodic(&params.query, limit).await {
            episodic = episodes
                .into_iter()
                .map(|e| EpisodicItem {
                    id: e.id,
                    content: e.content,
                    episode_type: e.episode_type.as_str().to_string(),
                    session_id: Some(e.session_id),
                    timestamp: e.created_at.timestamp(),
                })
                .collect();
        }

        // Procedural search via CORTEX
        let mut procedural = Vec::new();
        if let Ok(rules) = self.cortex.search_procedural(&params.query, limit).await {
            procedural = rules
                .into_iter()
                .filter(|r| r.confidence >= min_score)
                .map(|r| ProceduralItem {
                    id: r.id.clone(),
                    name: r.id,
                    description: format!("{} -> {}", r.trigger, r.action),
                    confidence: r.confidence,
                })
                .collect();
        }

        let summary = format!(
            "Found {} semantic, {} episodic, {} procedural results",
            semantic.len(),
            episodic.len(),
            procedural.len()
        );

        Ok(Json(HybridSearchResult {
            semantic,
            episodic,
            procedural,
            graph: Vec::new(),
            summary,
        }))
    }

    #[tool(description = "Manage tags on stored memories (add, remove, get, search)")]
    async fn manage_tags(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<ManageTagsParams>,
    ) -> std::result::Result<Json<ManageTagsResult>, McpError> {
        let params = params.0;

        match params.action.as_str() {
            "add" => {
                if let Some(doc_id) = &params.doc_id {
                    if let Ok(Some(doc)) = self.db.get_document_by_key(doc_id).await {
                        let mut new_tags = doc.tags.clone();
                        for tag in &params.tags {
                            if !new_tags.contains(tag) {
                                new_tags.push(tag.clone());
                            }
                        }
                        // Note: would need UpdateDocument support
                        return Ok(Json(ManageTagsResult {
                            success: true,
                            doc_id: Some(doc_id.clone()),
                            tags: new_tags,
                            results: Vec::new(),
                            message: format!("Added {} tags", params.tags.len()),
                        }));
                    }
                }
                Ok(Json(ManageTagsResult {
                    success: false,
                    doc_id: params.doc_id,
                    tags: Vec::new(),
                    results: Vec::new(),
                    message: "Document not found".to_string(),
                }))
            }
            "remove" => {
                if let Some(doc_id) = &params.doc_id {
                    if let Ok(Some(doc)) = self.db.get_document_by_key(doc_id).await {
                        let new_tags: Vec<_> = doc.tags.iter()
                            .filter(|t| !params.tags.contains(t))
                            .cloned()
                            .collect();
                        return Ok(Json(ManageTagsResult {
                            success: true,
                            doc_id: Some(doc_id.clone()),
                            tags: new_tags,
                            results: Vec::new(),
                            message: format!("Removed {} tags", params.tags.len()),
                        }));
                    }
                }
                Ok(Json(ManageTagsResult {
                    success: false,
                    doc_id: params.doc_id,
                    tags: Vec::new(),
                    results: Vec::new(),
                    message: "Document not found".to_string(),
                }))
            }
            "get" => {
                if let Some(doc_id) = &params.doc_id {
                    if let Ok(Some(doc)) = self.db.get_document_by_key(doc_id).await {
                        return Ok(Json(ManageTagsResult {
                            success: true,
                            doc_id: Some(doc_id.clone()),
                            tags: doc.tags,
                            results: Vec::new(),
                            message: "Tags retrieved".to_string(),
                        }));
                    }
                }
                Ok(Json(ManageTagsResult {
                    success: false,
                    doc_id: params.doc_id,
                    tags: Vec::new(),
                    results: Vec::new(),
                    message: "Document not found".to_string(),
                }))
            }
            "search" => {
                // Search for documents with the given tags
                let docs = self.db.list_documents(
                    if params.tags.is_empty() { None } else { Some(&params.tags) },
                    100,
                    0,
                ).await.unwrap_or_default();

                let results: Vec<crate::tools::MemorySummary> = docs
                    .into_iter()
                    .map(|d| crate::tools::MemorySummary {
                        key: d.key.unwrap_or_default(),
                        title: d.title.unwrap_or_else(|| d.content.chars().take(50).collect::<String>() + "..."),
                        tags: d.tags,
                        stored_at: d.created_at.map(|dt| dt.timestamp()).unwrap_or(0),
                    })
                    .collect();

                Ok(Json(ManageTagsResult {
                    success: true,
                    doc_id: None,
                    tags: params.tags,
                    results,
                    message: "Search completed".to_string(),
                }))
            }
            _ => Ok(Json(ManageTagsResult {
                success: false,
                doc_id: None,
                tags: Vec::new(),
                results: Vec::new(),
                message: format!("Unknown action: {}", params.action),
            })),
        }
    }

    #[tool(description = "Get aggregated context for a query from all memory sources")]
    async fn get_context(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<GetContextParams>,
    ) -> std::result::Result<Json<GetContextResult>, McpError> {
        let params = params.0;
        let limit = 5;
        let min_relevance = 0.3;

        // Gather context from all sources
        let mut semantic_items = Vec::new();
        let mut episodic_items = Vec::new();
        let mut procedural_rules = Vec::new();

        // Semantic search
        {
            let mut rag = self.rag.write().await;
            if let Ok(results) = rag.search(&params.query, Some(limit)).await {
                semantic_items = results
                    .into_iter()
                    .filter(|r| r.score >= min_relevance)
                    .map(|r| SemanticItem {
                        id: r.chunk.document_id.clone(),
                        content: r.chunk.text.clone(),
                        source: "semantic".to_string(),
                        score: r.score,
                        category: "memory".to_string(),
                        tags: Vec::new(),
                    })
                    .collect();
            }
        }

        // Episodic context
        if let Ok(episodes) = self.cortex.search_episodic(&params.query, limit).await {
            episodic_items = episodes
                .into_iter()
                .map(|e| EpisodicItem {
                    id: e.id,
                    content: e.content,
                    episode_type: e.episode_type.as_str().to_string(),
                    session_id: Some(e.session_id),
                    timestamp: e.created_at.timestamp(),
                })
                .collect();
        }

        // Procedural rules
        if let Ok(rules) = self.cortex.search_procedural(&params.query, limit).await {
            procedural_rules = rules
                .into_iter()
                .filter(|r| r.confidence >= min_relevance)
                .map(|r| ProceduralItem {
                    id: r.id.clone(),
                    name: r.id,
                    description: format!("{} -> {}", r.trigger, r.action),
                    confidence: r.confidence,
                })
                .collect();
        }

        // Calculate relevance scores
        let semantic_score = semantic_items.first().map(|s| s.score).unwrap_or(0.0);
        let episodic_score = if episodic_items.is_empty() { 0.0 } else { 0.5 };
        let procedural_score = procedural_rules.first().map(|p| p.confidence).unwrap_or(0.0);
        let overall = (semantic_score + episodic_score + procedural_score) / 3.0;

        let summary = format!(
            "Context for '{}': {} semantic, {} episodic, {} procedural items",
            params.query,
            semantic_items.len(),
            episodic_items.len(),
            procedural_rules.len()
        );

        Ok(Json(GetContextResult {
            query: params.query,
            semantic_items,
            episodic_items,
            procedural_rules,
            graph_entities: Vec::new(),
            scores: ContextScores {
                semantic: semantic_score,
                episodic: episodic_score,
                procedural: procedural_score,
                graph: 0.0,
                overall,
            },
            summary,
        }))
    }

    // ========================================================================
    // KNOWLEDGE GRAPH TOOLS
    // ========================================================================

    #[tool(description = "Add an entity to the knowledge graph")]
    async fn knowledge_add_entity(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<KnowledgeAddEntityParams>,
    ) -> std::result::Result<Json<KnowledgeAddEntityResult>, McpError> {
        let params = params.0;

        // Check if entity already exists
        let existing = self
            .db
            .get_entity_by_name(&params.name)
            .await
            .map_err(IntelligenceError::from)?;

        let (created, observations_added) = if let Some(entity) = existing {
            // Entity exists, add observations
            let entity_id = entity
                .id
                .ok_or_else(|| IntelligenceError::EntityNotFound(params.name.clone()))?;
            let id_str = entity_id.key().to_string();

            let mut added = 0;
            for obs in &params.observations {
                if self.db.add_observation(&id_str, obs).await.is_ok() {
                    added += 1;
                }
            }
            (false, added)
        } else {
            // Create new entity
            let input = CreateEntity::new(&params.name, &params.entity_type)
                .with_observations(params.observations.clone());

            self.db
                .create_entity(input)
                .await
                .map_err(IntelligenceError::from)?;

            (true, params.observations.len())
        };

        Ok(Json(KnowledgeAddEntityResult {
            name: params.name,
            entity_type: params.entity_type,
            created,
            observations_added,
        }))
    }

    #[tool(description = "Add observations to an existing entity")]
    async fn knowledge_add_observation(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<KnowledgeAddObservationParams>,
    ) -> std::result::Result<Json<KnowledgeAddObservationResult>, McpError> {
        let params = params.0;

        // Find entity by name
        let entity = self
            .db
            .get_entity_by_name(&params.entity_name)
            .await
            .map_err(IntelligenceError::from)?
            .ok_or_else(|| IntelligenceError::EntityNotFound(params.entity_name.clone()))?;

        let entity_id = entity
            .id
            .ok_or_else(|| IntelligenceError::EntityNotFound(params.entity_name.clone()))?;
        let id_str = entity_id.key().to_string();

        let mut added = 0;
        for obs in &params.observations {
            if self.db.add_observation(&id_str, obs).await.is_ok() {
                added += 1;
            }
        }

        Ok(Json(KnowledgeAddObservationResult {
            entity_name: params.entity_name,
            added,
        }))
    }

    #[tool(description = "Create a relation between two entities")]
    async fn knowledge_add_relation(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<KnowledgeAddRelationParams>,
    ) -> std::result::Result<Json<KnowledgeAddRelationResult>, McpError> {
        let params = params.0;

        // Find source entity
        let from_entity = self
            .db
            .get_entity_by_name(&params.from)
            .await
            .map_err(IntelligenceError::from)?
            .ok_or_else(|| IntelligenceError::EntityNotFound(params.from.clone()))?;

        let from_id = from_entity
            .id
            .ok_or_else(|| IntelligenceError::EntityNotFound(params.from.clone()))?;

        // Find target entity
        let to_entity = self
            .db
            .get_entity_by_name(&params.to)
            .await
            .map_err(IntelligenceError::from)?
            .ok_or_else(|| IntelligenceError::EntityNotFound(params.to.clone()))?;

        let to_id = to_entity
            .id
            .ok_or_else(|| IntelligenceError::EntityNotFound(params.to.clone()))?;

        // Create relation
        let input = CreateRelation::new(from_id, to_id, &params.relation_type);
        self.db
            .create_relation(input)
            .await
            .map_err(IntelligenceError::from)?;

        Ok(Json(KnowledgeAddRelationResult {
            from: params.from,
            to: params.to,
            relation_type: params.relation_type,
            created: true,
        }))
    }

    #[tool(description = "Search the knowledge graph")]
    async fn knowledge_search(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<KnowledgeSearchParams>,
    ) -> std::result::Result<Json<KnowledgeSearchResult>, McpError> {
        let params = params.0;

        // Search entities by name pattern
        let entities = self
            .db
            .search_entities(&params.query)
            .await
            .map_err(IntelligenceError::from)?;

        let entity_infos: Vec<EntityInfo> = entities
            .into_iter()
            .take(params.limit)
            .map(|e| EntityInfo {
                name: e.name,
                entity_type: e.entity_type,
                observations: e.observations,
            })
            .collect();

        Ok(Json(KnowledgeSearchResult {
            entities: entity_infos,
            relations: Vec::new(), // Relations between found entities could be added
        }))
    }

    #[tool(description = "Get a specific entity and its relations")]
    async fn knowledge_get_entity(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<KnowledgeGetEntityParams>,
    ) -> std::result::Result<Json<KnowledgeGetEntityResult>, McpError> {
        let params = params.0;

        // Find entity by name
        let entity = self
            .db
            .get_entity_by_name(&params.name)
            .await
            .map_err(IntelligenceError::from)?
            .ok_or_else(|| IntelligenceError::EntityNotFound(params.name.clone()))?;

        let entity_id = entity
            .id
            .clone()
            .ok_or_else(|| IntelligenceError::EntityNotFound(params.name.clone()))?;
        let id_str = entity_id.key().to_string();

        let entity_info = EntityInfo {
            name: entity.name,
            entity_type: entity.entity_type,
            observations: entity.observations,
        };

        let (outgoing, incoming) = if params.include_relations {
            // Get outgoing relations
            let out_rels = self
                .db
                .get_outgoing_relations(&id_str)
                .await
                .map_err(IntelligenceError::from)?;

            let mut outgoing_infos = Vec::new();
            for rel in out_rels {
                let to_id = rel.to.key().to_string();
                if let Ok(to_entity) = self.db.get_entity(&to_id).await {
                    outgoing_infos.push(RelationInfo {
                        from: params.name.clone(),
                        to: to_entity.name,
                        relation_type: rel.relation_type,
                    });
                }
            }

            // Get incoming relations
            let in_rels = self
                .db
                .get_incoming_relations(&id_str)
                .await
                .map_err(IntelligenceError::from)?;

            let mut incoming_infos = Vec::new();
            for rel in in_rels {
                let from_id = rel.from.key().to_string();
                if let Ok(from_entity) = self.db.get_entity(&from_id).await {
                    incoming_infos.push(RelationInfo {
                        from: from_entity.name,
                        to: params.name.clone(),
                        relation_type: rel.relation_type,
                    });
                }
            }

            (outgoing_infos, incoming_infos)
        } else {
            (Vec::new(), Vec::new())
        };

        Ok(Json(KnowledgeGetEntityResult {
            entity: entity_info,
            outgoing,
            incoming,
        }))
    }

    #[tool(description = "Delete entities from the knowledge graph")]
    async fn knowledge_delete_entity(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<KnowledgeDeleteEntityParams>,
    ) -> std::result::Result<Json<KnowledgeDeleteEntityResult>, McpError> {
        let params = params.0;

        let mut deleted = Vec::new();
        let mut relations_removed = 0;

        for name in params.names {
            if let Ok(Some(entity)) = self.db.get_entity_by_name(&name).await {
                if let Some(entity_id) = entity.id {
                    let id_str = entity_id.key().to_string();

                    // Count relations before deletion
                    let out_count = self
                        .db
                        .get_outgoing_relations(&id_str)
                        .await
                        .map(|r| r.len())
                        .unwrap_or(0);
                    let in_count = self
                        .db
                        .get_incoming_relations(&id_str)
                        .await
                        .map(|r| r.len())
                        .unwrap_or(0);

                    if self.db.delete_entity(&id_str).await.is_ok() {
                        deleted.push(name);
                        relations_removed += out_count + in_count;
                    }
                }
            }
        }

        Ok(Json(KnowledgeDeleteEntityResult {
            deleted,
            relations_removed,
        }))
    }

    #[tool(description = "Delete relations from the knowledge graph")]
    async fn knowledge_delete_relation(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<KnowledgeDeleteRelationParams>,
    ) -> std::result::Result<Json<KnowledgeDeleteRelationResult>, McpError> {
        let params = params.0;

        let mut deleted_count = 0;

        for rel_spec in params.relations {
            // Find entities by name
            if let (Ok(Some(from_entity)), Ok(Some(to_entity))) = (
                self.db.get_entity_by_name(&rel_spec.from).await,
                self.db.get_entity_by_name(&rel_spec.to).await,
            ) {
                if let (Some(from_id), Some(to_id)) = (from_entity.id, to_entity.id) {
                    let from_str = from_id.key().to_string();
                    let to_str = to_id.key().to_string();

                    let count = self
                        .db
                        .delete_relations_between(&from_str, &to_str, Some(&rel_spec.relation_type))
                        .await
                        .unwrap_or(0);
                    deleted_count += count;
                }
            }
        }

        Ok(Json(KnowledgeDeleteRelationResult {
            deleted: deleted_count,
        }))
    }

    #[tool(description = "Read the entire knowledge graph")]
    async fn knowledge_read_graph(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<KnowledgeReadGraphParams>,
    ) -> std::result::Result<Json<KnowledgeReadGraphResult>, McpError> {
        let params = params.0;

        // Get all entities (limited if specified)
        let all_entities: Vec<whytcard_database::Entity> = self
            .db
            .inner()
            .select("entity")
            .await
            .map_err(|e| IntelligenceError::Database(Box::new(whytcard_database::DatabaseError::from(e))))?;

        let entities: Vec<EntityInfo> = all_entities
            .into_iter()
            .take(if params.limit > 0 {
                params.limit
            } else {
                usize::MAX
            })
            .map(|e| EntityInfo {
                name: e.name,
                entity_type: e.entity_type,
                observations: e.observations,
            })
            .collect();

        // Get all relations
        let all_relations: Vec<whytcard_database::Relation> = self
            .db
            .inner()
            .select("relates_to")
            .await
            .map_err(|e| IntelligenceError::Database(Box::new(whytcard_database::DatabaseError::from(e))))?;

        // Convert relations to RelationInfo (need to lookup entity names)
        let mut relations = Vec::new();
        for rel in all_relations {
            let from_id = rel.from.key().to_string();
            let to_id = rel.to.key().to_string();

            if let (Ok(from_entity), Ok(to_entity)) = (
                self.db.get_entity(&from_id).await,
                self.db.get_entity(&to_id).await,
            ) {
                relations.push(RelationInfo {
                    from: from_entity.name,
                    to: to_entity.name,
                    relation_type: rel.relation_type,
                });
            }
        }

        let total_entities = self.db.count_entities().await.unwrap_or(0);
        let total_relations = self.db.count_relations().await.unwrap_or(0);

        Ok(Json(KnowledgeReadGraphResult {
            entities,
            relations,
            total_entities,
            total_relations,
        }))
    }

    #[tool(description = "Delete specific observations from an entity")]
    async fn knowledge_delete_observation(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<KnowledgeDeleteObservationParams>,
    ) -> std::result::Result<Json<KnowledgeDeleteObservationResult>, McpError> {
        let params = params.0;

        // Find entity by name
        let entity = self
            .db
            .get_entity_by_name(&params.entity_name)
            .await
            .map_err(IntelligenceError::from)?
            .ok_or_else(|| IntelligenceError::EntityNotFound(params.entity_name.clone()))?;

        let entity_id = entity
            .id
            .ok_or_else(|| IntelligenceError::EntityNotFound(params.entity_name.clone()))?;
        let _id_str = entity_id.key().to_string();

        // Get current observations and remove the specified ones
        let mut remaining = entity.observations.clone();
        let mut removed = 0;
        for obs in &params.observations {
            if let Some(pos) = remaining.iter().position(|o| o == obs) {
                remaining.remove(pos);
                removed += 1;
            }
        }

        // Note: Would need UpdateEntity support in whytcard_database
        // For now, just report what would be removed

        Ok(Json(KnowledgeDeleteObservationResult {
            entity_name: params.entity_name,
            removed,
        }))
    }

    #[tool(description = "Export the knowledge graph in various formats")]
    async fn export_graph(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<ExportGraphParams>,
    ) -> std::result::Result<Json<ExportGraphResult>, McpError> {
        let params = params.0;

        // Get all entities
        let all_entities: Vec<whytcard_database::Entity> = self
            .db
            .inner()
            .select("entity")
            .await
            .map_err(|e| IntelligenceError::Database(Box::new(whytcard_database::DatabaseError::from(e))))?;

        // Filter by entity type if specified
        let entities: Vec<_> = if params.entity_types.is_empty() {
            all_entities
        } else {
            all_entities
                .into_iter()
                .filter(|e| params.entity_types.contains(&e.entity_type))
                .collect()
        };

        // Get relations if requested
        let relations: Vec<RelationInfo> = if params.include_relations {
            let all_relations: Vec<whytcard_database::Relation> = self
                .db
                .inner()
                .select("relates_to")
                .await
                .unwrap_or_default();

            // Build entity map for relation lookup
            let entity_map: std::collections::HashMap<String, String> = entities
                .iter()
                .filter_map(|e| e.id.as_ref().map(|id| (id.key().to_string(), e.name.clone())))
                .collect();

            all_relations
                .into_iter()
                .filter_map(|rel| {
                    let from_id = rel.from.key().to_string();
                    let to_id = rel.to.key().to_string();
                    if let (Some(from_name), Some(to_name)) =
                        (entity_map.get(&from_id), entity_map.get(&to_id))
                    {
                        Some(RelationInfo {
                            from: from_name.clone(),
                            to: to_name.clone(),
                            relation_type: rel.relation_type,
                        })
                    } else {
                        None
                    }
                })
                .collect()
        } else {
            Vec::new()
        };

        // Convert entities to EntityInfo
        let entity_infos: Vec<EntityInfo> = entities
            .into_iter()
            .map(|e| EntityInfo {
                name: e.name,
                entity_type: e.entity_type,
                observations: e.observations,
            })
            .collect();

        let entity_count = entity_infos.len();
        let relation_count = relations.len();

        Ok(Json(ExportGraphResult {
            entities: entity_infos,
            entity_count,
            relations,
            relation_count,
            format: params.format,
            exported_at: chrono::Utc::now().timestamp(),
        }))
    }

    #[tool(description = "Get neighboring entities of a given entity")]
    async fn knowledge_get_neighbors(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<KnowledgeGetNeighborsParams>,
    ) -> std::result::Result<Json<KnowledgeGetNeighborsResult>, McpError> {
        let params = params.0;
        let max_depth = params.max_depth;

        // Find entity
        let entity = self
            .db
            .get_entity_by_name(&params.entity_name)
            .await
            .map_err(IntelligenceError::from)?
            .ok_or_else(|| IntelligenceError::EntityNotFound(params.entity_name.clone()))?;

        let entity_id = entity
            .id
            .ok_or_else(|| IntelligenceError::EntityNotFound(params.entity_name.clone()))?;
        let id_str = entity_id.key().to_string();

        let mut neighbors = Vec::new();
        let mut visited = std::collections::HashSet::new();
        visited.insert(params.entity_name.clone());

        // BFS to find neighbors up to depth
        let mut queue = vec![(id_str.clone(), Vec::<String>::new(), 0usize)];
        while let Some((current_id, path, current_depth)) = queue.pop() {
            if current_depth >= max_depth {
                continue;
            }

            // Get outgoing relations
            if let Ok(rels) = self.db.get_outgoing_relations(&current_id).await {
                for rel in rels {
                    // Filter by relation types if specified
                    if !params.relation_types.is_empty() && !params.relation_types.contains(&rel.relation_type) {
                        continue;
                    }

                    let to_id = rel.to.key().to_string();
                    if let Ok(to_entity) = self.db.get_entity(&to_id).await {
                        if !visited.contains(&to_entity.name) {
                            visited.insert(to_entity.name.clone());
                            let mut new_path = path.clone();
                            new_path.push(rel.relation_type.clone());
                            neighbors.push(NeighborInfo {
                                entity: EntityInfo {
                                    name: to_entity.name.clone(),
                                    entity_type: to_entity.entity_type,
                                    observations: to_entity.observations,
                                },
                                distance: current_depth + 1,
                                path: new_path.clone(),
                            });
                            queue.push((to_id, new_path, current_depth + 1));
                        }
                    }
                }
            }

            // Get incoming relations
            if let Ok(rels) = self.db.get_incoming_relations(&current_id).await {
                for rel in rels {
                    if !params.relation_types.is_empty() && !params.relation_types.contains(&rel.relation_type) {
                        continue;
                    }

                    let from_id = rel.from.key().to_string();
                    if let Ok(from_entity) = self.db.get_entity(&from_id).await {
                        if !visited.contains(&from_entity.name) {
                            visited.insert(from_entity.name.clone());
                            let mut new_path = path.clone();
                            new_path.push(format!("~{}", rel.relation_type));
                            neighbors.push(NeighborInfo {
                                entity: EntityInfo {
                                    name: from_entity.name.clone(),
                                    entity_type: from_entity.entity_type,
                                    observations: from_entity.observations,
                                },
                                distance: current_depth + 1,
                                path: new_path.clone(),
                            });
                            queue.push((from_id, new_path, current_depth + 1));
                        }
                    }
                }
            }
        }

        let total = neighbors.len();

        Ok(Json(KnowledgeGetNeighborsResult {
            start_entity: params.entity_name,
            neighbors,
            total,
        }))
    }

    #[tool(description = "Find path between two entities in the knowledge graph")]
    async fn knowledge_find_path(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<KnowledgeFindPathParams>,
    ) -> std::result::Result<Json<KnowledgeFindPathResult>, McpError> {
        let params = params.0;
        let max_depth = params.max_depth;

        // Find source entity
        let from_entity = self
            .db
            .get_entity_by_name(&params.from)
            .await
            .map_err(IntelligenceError::from)?
            .ok_or_else(|| IntelligenceError::EntityNotFound(params.from.clone()))?;

        let from_id = from_entity
            .id
            .ok_or_else(|| IntelligenceError::EntityNotFound(params.from.clone()))?;

        // Find target entity
        let _to_entity = self
            .db
            .get_entity_by_name(&params.to)
            .await
            .map_err(IntelligenceError::from)?
            .ok_or_else(|| IntelligenceError::EntityNotFound(params.to.clone()))?;

        // BFS to find shortest path
        let mut visited: std::collections::HashMap<String, (Option<String>, Option<RelationInfo>)> = std::collections::HashMap::new();
        visited.insert(params.from.clone(), (None, None));

        let mut queue = vec![(from_id.key().to_string(), params.from.clone(), 0usize)];
        let mut found = false;

        while let Some((current_id, current_name, depth)) = queue.pop() {
            if current_name == params.to {
                found = true;
                break;
            }

            if depth >= max_depth {
                continue;
            }

            // Get outgoing relations
            if let Ok(rels) = self.db.get_outgoing_relations(&current_id).await {
                for rel in rels {
                    let to_id = rel.to.key().to_string();
                    if let Ok(entity) = self.db.get_entity(&to_id).await {
                        if !visited.contains_key(&entity.name) {
                            visited.insert(
                                entity.name.clone(),
                                (Some(current_name.clone()), Some(RelationInfo {
                                    from: current_name.clone(),
                                    to: entity.name.clone(),
                                    relation_type: rel.relation_type.clone(),
                                })),
                            );
                            queue.push((to_id, entity.name.clone(), depth + 1));
                        }
                    }
                }
            }

            // Get incoming relations
            if let Ok(rels) = self.db.get_incoming_relations(&current_id).await {
                for rel in rels {
                    let other_id = rel.from.key().to_string();
                    if let Ok(entity) = self.db.get_entity(&other_id).await {
                        if !visited.contains_key(&entity.name) {
                            visited.insert(
                                entity.name.clone(),
                                (Some(current_name.clone()), Some(RelationInfo {
                                    from: entity.name.clone(),
                                    to: current_name.clone(),
                                    relation_type: rel.relation_type.clone(),
                                })),
                            );
                            queue.push((other_id, entity.name.clone(), depth + 1));
                        }
                    }
                }
            }
        }

        // Reconstruct path
        let path = if found {
            let mut path_relations = Vec::new();
            let mut current = params.to.clone();

            while let Some((prev, rel_opt)) = visited.get(&current) {
                if let (Some(_p), Some(rel)) = (prev, rel_opt) {
                    path_relations.push(rel.clone());
                    current = prev.clone().unwrap();
                } else {
                    break;
                }
            }

            path_relations.reverse();
            path_relations
        } else {
            Vec::new()
        };

        let length = path.len();

        Ok(Json(KnowledgeFindPathResult {
            found,
            path,
            length,
        }))
    }

    // ========================================================================
    // CORTEX TOOLS
    // ========================================================================

    #[tool(description = "Process a query through CORTEX cognitive engine for intelligent analysis and execution")]
    async fn cortex_process(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<CortexProcessParams>,
    ) -> std::result::Result<Json<CortexProcessResult>, McpError> {
        let params = params.0;
        let mut loaded_prompts: Vec<String> = Vec::new();
        let mut instructions_count = 0;

        // Start session if requested
        let session_id = if params.session_id.is_some() {
            match self.cortex.start_session(None).await {
                Ok(sid) => Some(sid),
                Err(e) => {
                    tracing::warn!("Failed to start session: {}", e);
                    None
                }
            }
        } else {
            None
        };

        // ====================================================================
        // PROMPT INJECTION: Load prompts from memory
        // ====================================================================
        let mut prompt_context = String::new();

        // 0. ALWAYS inject .instructions.md files if inject_instructions is true (default)
        if params.inject_instructions {
            let instructions_prompt = self.cortex.get_instructions_prompt(params.file_path.as_deref()).await;
            if !instructions_prompt.is_empty() {
                prompt_context.push_str("# System Instructions (from .instructions.md files)\n\n");
                prompt_context.push_str(&instructions_prompt);
                prompt_context.push_str("\n\n---\n\n");

                // Count instructions
                if let Some(ref file_path) = params.file_path {
                    instructions_count = self.cortex.get_instructions_for_file(file_path).await.len();
                } else {
                    let stats = self.cortex.get_instructions_stats().await;
                    instructions_count = stats.get("count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                }
                loaded_prompts.push(format!("instructions:{}", instructions_count));
            }
        }

        // 1. Always load doubt prompt if inject_doubt is true (default)
        if params.inject_doubt {
            if let Ok(Some(doubt_doc)) = self.db.get_document_by_key("prompt:root:doubt").await {
                prompt_context.push_str(&doubt_doc.content);
                prompt_context.push_str("\n\n---\n\n");
                loaded_prompts.push("prompt:root:doubt".to_string());
            }
        }

        // 2. Load language-specific prompt if language is provided
        if let Some(ref lang) = params.language {
            let lang_key = format!("prompt:code:{}", lang.to_lowercase());
            if let Ok(Some(lang_doc)) = self.db.get_document_by_key(&lang_key).await {
                prompt_context.push_str(&lang_doc.content);
                prompt_context.push_str("\n\n---\n\n");
                loaded_prompts.push(lang_key);
            }
        }

        // 3. Load task-specific prompt if task_type is provided
        if let Some(ref task_type) = params.task_type {
            let task_key = format!("prompt:{}", task_type.prompt_key());
            if let Ok(Some(task_doc)) = self.db.get_document_by_key(&task_key).await {
                prompt_context.push_str(&task_doc.content);
                prompt_context.push_str("\n\n---\n\n");
                loaded_prompts.push(task_key);
            }
        }

        // Build enriched context
        let context = if prompt_context.is_empty() {
            params
                .context
                .as_ref()
                .map(|c| serde_json::json!({ "user_context": c }))
        } else {
            let user_ctx = params.context.as_deref().unwrap_or("");
            Some(serde_json::json!({
                "system_prompts": prompt_context,
                "user_context": user_ctx,
                "loaded_prompts": &loaded_prompts
            }))
        };

        // Process through CORTEX
        let result = self
            .cortex
            .process(&params.query, context)
            .await
            .map_err(|e| {
                McpError::internal_error(format!("CORTEX processing failed: {}", e), None)
            })?;

        // End session if we started one
        if session_id.is_some() {
            let _ = self.cortex.end_session().await;
        }

        // Convert result
        let mut output = CortexProcessResult {
            success: result.success,
            output: result.result.to_string(),
            intent: format!("{:?}", result.perception.intent),
            labels: result
                .perception
                .labels
                .iter()
                .map(|l| l.as_str().to_string())
                .collect(),
            confidence: result.confidence,
            research_needed: result.execution.research_performed,
            steps_executed: result.execution.steps_executed,
            duration_ms: result.execution.duration_ms as u128,
            recommendations: result.next_actions,
            session_id: None,
            loaded_prompts,
            instructions_count,
        };
        output.session_id = session_id;

        Ok(Json(output))
    }

    #[tool(description = "Provide feedback to CORTEX for adaptive learning and rule improvement")]
    async fn cortex_feedback(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<CortexFeedbackParams>,
    ) -> std::result::Result<Json<CortexFeedbackResult>, McpError> {
        let params = params.0;

        let new_confidence = self
            .cortex
            .provide_feedback(&params.rule_id, params.success)
            .await
            .map_err(|e| McpError::internal_error(format!("Feedback failed: {}", e), None))?;

        Ok(Json(CortexFeedbackResult {
            recorded: true,
            new_confidence,
            message: format!(
                "Feedback recorded for rule {}. New confidence: {:.2}%",
                params.rule_id,
                new_confidence * 100.0
            ),
        }))
    }

    #[tool(description = "Get CORTEX cognitive engine statistics including memory usage")]
    async fn cortex_stats(
        &self,
        _params: rmcp::handler::server::wrapper::Parameters<CortexStatsParams>,
    ) -> std::result::Result<Json<CortexStatsResult>, McpError> {
        let stats = self.cortex.get_stats().await;

        // Extract values from JSON
        let semantic_facts = stats
            .get("semantic")
            .and_then(|s| s.get("total_facts"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let episodic_events = stats
            .get("episodic")
            .and_then(|s| s.get("total_episodes"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        let procedural_rules = stats
            .get("procedural")
            .and_then(|s| s.get("total_rules"))
            .and_then(|v| v.as_u64())
            .unwrap_or(0) as usize;

        Ok(Json(CortexStatsResult {
            memory: crate::tools::MemoryStatsDetail {
                semantic_facts,
                episodic_events,
                procedural_rules,
            },
            status: "running".to_string(),
            uptime_secs: 0,
        }))
    }

    #[tool(description = "Manage workspace instructions from .instructions.md files. Actions: list (show all), reload (refresh from disk), get (get content by name), for_file (filter by file path pattern)")]
    async fn cortex_instructions(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<CortexInstructionsParams>,
    ) -> std::result::Result<Json<CortexInstructionsResult>, McpError> {
        let params = params.0;

        match params.action {
            InstructionsAction::List => {
                let stats = self.cortex.get_instructions_stats().await;
                let count = stats.get("count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;
                let instructions = stats.get("instructions")
                    .and_then(|v| v.as_array())
                    .map(|arr| {
                        arr.iter().filter_map(|v| {
                            Some(InstructionInfo {
                                name: v.get("name")?.as_str()?.to_string(),
                                description: v.get("description").and_then(|d| d.as_str()).map(|s| s.to_string()),
                                apply_to: v.get("apply_to").and_then(|a| a.as_str()).map(|s| s.to_string()),
                            })
                        }).collect()
                    })
                    .unwrap_or_default();

                Ok(Json(CortexInstructionsResult {
                    success: true,
                    action: "list".to_string(),
                    count,
                    instructions,
                    content: None,
                    message: format!("Found {} instructions loaded", count),
                }))
            }
            InstructionsAction::Reload => {
                self.cortex.reload_instructions().await
                    .map_err(|e| McpError::internal_error(format!("Failed to reload instructions: {}", e), None))?;

                let stats = self.cortex.get_instructions_stats().await;
                let count = stats.get("count").and_then(|v| v.as_u64()).unwrap_or(0) as usize;

                Ok(Json(CortexInstructionsResult {
                    success: true,
                    action: "reload".to_string(),
                    count,
                    instructions: Vec::new(),
                    content: None,
                    message: format!("Reloaded {} instructions from workspace", count),
                }))
            }
            InstructionsAction::Get => {
                let name = params.name.ok_or_else(|| {
                    McpError::invalid_params("name parameter required for get action", None)
                })?;

                let content = self.cortex.get_instruction_content(&name).await;
                let found = content.is_some();

                Ok(Json(CortexInstructionsResult {
                    success: found,
                    action: "get".to_string(),
                    count: if found { 1 } else { 0 },
                    instructions: Vec::new(),
                    content,
                    message: if found {
                        format!("Found instruction: {}", name)
                    } else {
                        format!("Instruction not found: {}", name)
                    },
                }))
            }
            InstructionsAction::ForFile => {
                let file_path = params.file_path.ok_or_else(|| {
                    McpError::invalid_params("file_path parameter required for for_file action", None)
                })?;

                let matching = self.cortex.get_instructions_for_file(&file_path).await;
                let instructions: Vec<InstructionInfo> = matching.iter().map(|i| {
                    InstructionInfo {
                        name: i.name.clone(),
                        description: Some(i.description.clone()),
                        apply_to: Some(i.apply_to.clone()),
                    }
                }).collect();
                let count = instructions.len();

                Ok(Json(CortexInstructionsResult {
                    success: true,
                    action: "for_file".to_string(),
                    count,
                    instructions,
                    content: None,
                    message: format!("Found {} instructions applicable to {}", count, file_path),
                }))
            }
        }
    }

    #[tool(description = "Cleanup old CORTEX data based on retention policy")]
    async fn cortex_cleanup(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<CortexCleanupParams>,
    ) -> std::result::Result<Json<CortexCleanupResult>, McpError> {
        let params = params.0;

        let cleaned = self
            .cortex
            .cleanup(params.retention_days)
            .await
            .map_err(|e| McpError::internal_error(format!("Cleanup failed: {}", e), None))?;

        Ok(Json(CortexCleanupResult {
            cleaned_count: cleaned,
            message: format!(
                "Cleaned {} old records (retention: {} days)",
                cleaned, params.retention_days
            ),
        }))
    }

    #[tool(description = "Execute a shell command. Use this to run terminal commands like npm install, cargo build, git, etc. Returns stdout, stderr, and exit code.")]
    async fn cortex_execute(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<CortexExecuteParams>,
    ) -> std::result::Result<Json<CortexExecuteResult>, McpError> {
        use std::time::Instant;
        use tokio::process::Command;

        let params = params.0;
        let start = Instant::now();

        // Determine shell based on OS
        #[cfg(windows)]
        let (shell, shell_arg) = ("powershell", "-Command");
        #[cfg(not(windows))]
        let (shell, shell_arg) = ("sh", "-c");

        let mut cmd = Command::new(shell);
        cmd.arg(shell_arg).arg(&params.command);

        // Set working directory if provided
        if let Some(ref cwd) = params.cwd {
            cmd.current_dir(cwd);
        }

        // Set environment variables if provided
        if let Some(ref env_vars) = params.env {
            for (key, value) in env_vars {
                cmd.env(key, value);
            }
        }

        // Execute with timeout
        let output = tokio::time::timeout(
            std::time::Duration::from_secs(params.timeout_secs),
            cmd.output(),
        )
        .await
        .map_err(|_| {
            McpError::internal_error(
                format!("Command timed out after {} seconds", params.timeout_secs),
                None,
            )
        })?
        .map_err(|e| McpError::internal_error(format!("Failed to execute command: {}", e), None))?;

        let duration_ms = start.elapsed().as_millis() as u64;
        let exit_code = output.status.code().unwrap_or(-1);
        let success = output.status.success();

        let stdout = String::from_utf8_lossy(&output.stdout).to_string();
        let stderr = if params.separate_stderr {
            String::from_utf8_lossy(&output.stderr).to_string()
        } else {
            // Merge stderr into stdout if not separate
            String::new()
        };

        // If not separating stderr, append it to stdout
        let stdout = if !params.separate_stderr && !output.stderr.is_empty() {
            format!(
                "{}\n--- stderr ---\n{}",
                stdout,
                String::from_utf8_lossy(&output.stderr)
            )
        } else {
            stdout
        };

        // Log the execution for learning
        tracing::info!(
            command = %params.command,
            exit_code = exit_code,
            duration_ms = duration_ms,
            success = success,
            "CORTEX executed command"
        );

        Ok(Json(CortexExecuteResult {
            success,
            exit_code,
            stdout,
            stderr,
            duration_ms,
            command: params.command,
        }))
    }

    // ========================================================================
    // EXTERNAL INTEGRATION TOOLS
    // ========================================================================

    #[tool(description = "Perform sequential thinking for complex problem analysis and decomposition")]
    async fn sequential_thinking(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<SequentialThinkingParams>,
    ) -> std::result::Result<Json<SequentialThinkingResult>, McpError> {
        let params = params.0;

        let mut thinking = self.thinking.write().await;
        thinking.start_session();

        // Decompose the problem using internal sequential thinking
        let result = thinking
            .decompose_problem(&params.problem)
            .await
            .map_err(|e| McpError::internal_error(format!("Thinking failed: {}", e), None))?;

        let steps: Vec<ThinkingStep> = result
            .thoughts
            .into_iter()
            .map(|t| ThinkingStep {
                number: t.number,
                content: t.content,
                is_revision: t.is_revision,
            })
            .collect();

        Ok(Json(SequentialThinkingResult {
            steps,
            conclusion: result.conclusion,
            complete: result.complete,
            source: "internal".to_string(),
        }))
    }

    #[tool(description = "Get documentation from external sources (Context7, MS Learn) for libraries and frameworks")]
    async fn external_docs(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<ExternalDocsParams>,
    ) -> std::result::Result<Json<ExternalDocsResult>, McpError> {
        let params = params.0;

        // Try Context7 first for library documentation
        if params.source == "auto" || params.source == "context7" {
            let context7 = self.context7.read().await;
            if context7.is_ready() {
                if let Ok(Some(doc)) = context7
                    .get_library_docs(&params.library, params.topic.as_deref(), params.max_tokens)
                    .await
                {
                    return Ok(Json(ExternalDocsResult {
                        library: doc.source,
                        topic: doc.topic,
                        content: doc.content,
                        code_snippets: doc.code_snippets,
                        url: doc.url,
                        provider: doc.provider,
                    }));
                }
            }
        }

        // Try MS Learn for Microsoft/Azure libraries
        if params.source == "auto" || params.source == "mslearn" {
            let mslearn = self.mslearn.read().await;
            if mslearn.is_ready() {
                let query = if let Some(t) = &params.topic {
                    format!("{} {}", params.library, t)
                } else {
                    params.library.clone()
                };

                if let Ok(Some(doc)) = mslearn.fetch_docs(&query).await {
                    return Ok(Json(ExternalDocsResult {
                        library: doc.source,
                        topic: doc.topic,
                        content: doc.content,
                        code_snippets: doc.code_snippets,
                        url: doc.url,
                        provider: doc.provider,
                    }));
                }
            }
        }

        // Return empty result if no docs found
        Ok(Json(ExternalDocsResult {
            library: params.library,
            topic: params.topic,
            content: "Documentation not found. Check if the library name is correct or try a different source.".to_string(),
            code_snippets: Vec::new(),
            url: None,
            provider: "none".to_string(),
        }))
    }

    #[tool(description = "Search the web using external search provider (Tavily)")]
    async fn external_search(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<ExternalSearchParams>,
    ) -> std::result::Result<Json<ExternalSearchResult>, McpError> {
        let params = params.0;

        let tavily = self.tavily.read().await;
        if !tavily.is_ready() {
            return Ok(Json(ExternalSearchResult {
                query: params.query,
                results: Vec::new(),
                provider: "tavily".to_string(),
                total: 0,
            }));
        }

        use crate::integrations::tavily::{SearchOptions, SearchTopic};

        let options = SearchOptions {
            topic: if params.search_type == "news" {
                Some(SearchTopic::News)
            } else {
                Some(SearchTopic::General)
            },
            max_results: Some(params.max_results as usize),
            include_domains: if params.include_domains.is_empty() {
                None
            } else {
                Some(params.include_domains)
            },
            exclude_domains: if params.exclude_domains.is_empty() {
                None
            } else {
                Some(params.exclude_domains)
            },
            ..Default::default()
        };

        let results = tavily
            .search_with_options(&params.query, options)
            .await
            .map_err(|e| McpError::internal_error(format!("Search failed: {}", e), None))?;

        let items: Vec<SearchResultItem> = results
            .into_iter()
            .map(|r| SearchResultItem {
                title: r.title,
                content: r.content,
                url: r.url,
                score: r.score,
            })
            .collect();

        let total = items.len();

        Ok(Json(ExternalSearchResult {
            query: params.query,
            results: items,
            provider: "tavily".to_string(),
            total,
        }))
    }

    #[tool(description = "Generic call to external MCP server tool")]
    async fn external_mcp_call(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<ExternalMcpCallParams>,
    ) -> std::result::Result<Json<ExternalMcpCallResult>, McpError> {
        let params = params.0;

        // For now, route to the appropriate internal client based on server name
        match params.server.as_str() {
            "context7" => {
                // Handle context7 calls
                if params.tool == "get-library-docs" {
                    if let Some(args) = params.arguments {
                        let library_id = args
                            .get("context7CompatibleLibraryID")
                            .and_then(|v| v.as_str())
                            .unwrap_or("");
                        let topic = args.get("topic").and_then(|v| v.as_str());
                        let tokens = args.get("tokens").and_then(|v| v.as_u64()).unwrap_or(5000) as u32;

                        let context7 = self.context7.read().await;
                        if let Ok(Some(doc)) = context7.get_library_docs(library_id, topic, tokens).await {
                            return Ok(Json(ExternalMcpCallResult {
                                server: params.server,
                                tool: params.tool,
                                success: true,
                                content: doc.content,
                                data: Some(serde_json::json!({
                                    "library": doc.source,
                                    "url": doc.url,
                                    "code_snippets": doc.code_snippets
                                })),
                                error: None,
                            }));
                        }
                    }
                }
                Ok(Json(ExternalMcpCallResult {
                    server: params.server,
                    tool: params.tool,
                    success: false,
                    content: String::new(),
                    data: None,
                    error: Some("Context7 call failed or invalid parameters".to_string()),
                }))
            }
            "tavily" => {
                if params.tool == "tavily-search" {
                    if let Some(args) = params.arguments {
                        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");
                        let max_results = args.get("max_results").and_then(|v| v.as_u64()).unwrap_or(10) as usize;

                        let tavily = self.tavily.read().await;
                        if let Ok(results) = tavily.search(query, max_results).await {
                            let content = results
                                .iter()
                                .map(|r| format!("## {}\n{}\n", r.title, r.content))
                                .collect::<Vec<_>>()
                                .join("\n");

                            return Ok(Json(ExternalMcpCallResult {
                                server: params.server,
                                tool: params.tool,
                                success: true,
                                content,
                                data: Some(serde_json::to_value(&results).unwrap_or_default()),
                                error: None,
                            }));
                        }
                    }
                }
                Ok(Json(ExternalMcpCallResult {
                    server: params.server,
                    tool: params.tool,
                    success: false,
                    content: String::new(),
                    data: None,
                    error: Some("Tavily call failed or invalid parameters".to_string()),
                }))
            }
            "mslearn" | "microsoft-learn" => {
                if params.tool == "microsoft_docs_search" {
                    if let Some(args) = params.arguments {
                        let query = args.get("query").and_then(|v| v.as_str()).unwrap_or("");

                        let mslearn = self.mslearn.read().await;
                        if let Ok(results) = mslearn.search(query, 10).await {
                            let content = results
                                .iter()
                                .map(|r| format!("## {}\n{}\n", r.title, r.content))
                                .collect::<Vec<_>>()
                                .join("\n");

                            return Ok(Json(ExternalMcpCallResult {
                                server: params.server,
                                tool: params.tool,
                                success: true,
                                content,
                                data: Some(serde_json::to_value(&results).unwrap_or_default()),
                                error: None,
                            }));
                        }
                    }
                }
                Ok(Json(ExternalMcpCallResult {
                    server: params.server,
                    tool: params.tool,
                    success: false,
                    content: String::new(),
                    data: None,
                    error: Some("MS Learn call failed or invalid parameters".to_string()),
                }))
            }
            // For all other servers, try to use the MCP client manager
            _ => {
                // Check if we're connected to this server
                if self.mcp_clients.is_connected(&params.server).await {
                    // Call the tool via MCP protocol
                    match self
                        .mcp_clients
                        .call_tool(&params.server, &params.tool, params.arguments.clone())
                        .await
                    {
                        Ok(result) => Ok(Json(ExternalMcpCallResult {
                            server: params.server,
                            tool: params.tool,
                            success: result.success,
                            content: result.content,
                            data: result.data,
                            error: result.error,
                        })),
                        Err(e) => Ok(Json(ExternalMcpCallResult {
                            server: params.server,
                            tool: params.tool,
                            success: false,
                            content: String::new(),
                            data: None,
                            error: Some(format!("MCP call failed: {}", e)),
                        })),
                    }
                } else {
                    // Server not connected - try to connect first
                    if let Err(e) = self.mcp_clients.connect(&params.server).await {
                        return Ok(Json(ExternalMcpCallResult {
                            server: params.server.clone(),
                            tool: params.tool,
                            success: false,
                            content: String::new(),
                            data: None,
                            error: Some(format!(
                                "Server '{}' not connected and auto-connect failed: {}. Use mcp_connect first.",
                                params.server, e
                            )),
                        }));
                    }
                    // Retry after connection
                    match self
                        .mcp_clients
                        .call_tool(&params.server, &params.tool, params.arguments)
                        .await
                    {
                        Ok(result) => Ok(Json(ExternalMcpCallResult {
                            server: params.server,
                            tool: params.tool,
                            success: result.success,
                            content: result.content,
                            data: result.data,
                            error: result.error,
                        })),
                        Err(e) => Ok(Json(ExternalMcpCallResult {
                            server: params.server,
                            tool: params.tool,
                            success: false,
                            content: String::new(),
                            data: None,
                            error: Some(format!("MCP call failed after connect: {}", e)),
                        })),
                    }
                }
            }
        }
    }

    #[tool(description = "Connect to an external MCP server by name (predefined) or custom config")]
    async fn mcp_connect(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<McpConnectParams>,
    ) -> std::result::Result<Json<McpConnectResult>, McpError> {
        let params = params.0;

        // If custom config provided, add it first
        if let Some(custom) = params.custom_config {
            let transport = match custom.transport.as_str() {
                "stdio" => {
                    let command = custom.command.unwrap_or_else(|| "npx".to_string());
                    crate::mcp_client::McpTransport::Stdio {
                        command,
                        args: custom.args,
                    }
                }
                "sse" => crate::mcp_client::McpTransport::Sse {
                    url: custom.url.unwrap_or_default(),
                    auth_token: None,
                },
                "http" => crate::mcp_client::McpTransport::Sse {
                    url: custom.url.unwrap_or_default(),
                    auth_token: None,
                },
                _ => {
                    return Ok(Json(McpConnectResult {
                        server: params.server,
                        connected: false,
                        tools: Vec::new(),
                        error: Some("Invalid transport type. Use 'stdio', 'sse', or 'http'".to_string()),
                    }));
                }
            };

            let _config = crate::mcp_client::McpServerConfig {
                name: params.server.clone(),
                transport,
                env: custom.env,
                auto_reconnect: true,
                timeout_secs: 30,
            };

            // Note: McpClientManager needs to be mutable for add_config
            // For now, we can only connect to predefined servers
            tracing::warn!(
                "Custom config for '{}' provided but dynamic config not yet supported",
                params.server
            );
        }

        // Try to connect
        match self.mcp_clients.connect(&params.server).await {
            Ok(()) => {
                // Get available tools
                let tools = self
                    .mcp_clients
                    .list_server_tools(&params.server)
                    .await
                    .into_iter()
                    .map(|t| t.name)
                    .collect();

                Ok(Json(McpConnectResult {
                    server: params.server,
                    connected: true,
                    tools,
                    error: None,
                }))
            }
            Err(e) => Ok(Json(McpConnectResult {
                server: params.server,
                connected: false,
                tools: Vec::new(),
                error: Some(format!("Connection failed: {}", e)),
            })),
        }
    }

    #[tool(description = "Disconnect from an external MCP server")]
    async fn mcp_disconnect(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<McpDisconnectParams>,
    ) -> std::result::Result<Json<McpDisconnectResult>, McpError> {
        let params = params.0;

        match self.mcp_clients.disconnect(&params.server).await {
            Ok(()) => Ok(Json(McpDisconnectResult {
                server: params.server,
                disconnected: true,
                error: None,
            })),
            Err(e) => Ok(Json(McpDisconnectResult {
                server: params.server,
                disconnected: false,
                error: Some(format!("Disconnection failed: {}", e)),
            })),
        }
    }

    #[tool(description = "List available tools from connected MCP servers")]
    async fn mcp_list_tools(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<McpListToolsParams>,
    ) -> std::result::Result<Json<McpListToolsResult>, McpError> {
        let params = params.0;

        let mut tools_by_server = std::collections::HashMap::new();

        // Get tools from MCP client manager
        if let Some(server) = &params.server {
            let server_tools = self.mcp_clients.list_server_tools(server).await;
            let filtered: Vec<McpToolDetail> = server_tools
                .into_iter()
                .filter(|t| {
                    if let Some(pattern) = &params.name_pattern {
                        t.name.contains(pattern)
                    } else {
                        true
                    }
                })
                .map(|t| McpToolDetail {
                    name: t.name,
                    description: t.description,
                    input_schema: t.input_schema,
                })
                .collect();
            if !filtered.is_empty() {
                tools_by_server.insert(server.clone(), filtered);
            }
        } else {
            // Get all tools
            let all_tools = self.mcp_clients.list_all_tools().await;
            for tool_info in all_tools {
                let matches = if let Some(pattern) = &params.name_pattern {
                    tool_info.name.contains(pattern)
                } else {
                    true
                };

                if matches {
                    let detail = McpToolDetail {
                        name: tool_info.name,
                        description: tool_info.description,
                        input_schema: tool_info.input_schema,
                    };
                    tools_by_server
                        .entry(tool_info.server)
                        .or_insert_with(Vec::new)
                        .push(detail);
                }
            }
        }

        let total = tools_by_server.values().map(|v| v.len()).sum();

        Ok(Json(McpListToolsResult {
            total,
            tools_by_server,
        }))
    }

    #[tool(description = "List all available predefined MCP servers and their requirements")]
    async fn mcp_available_servers(
        &self,
        _params: rmcp::handler::server::wrapper::Parameters<McpAvailableServersParams>,
    ) -> std::result::Result<Json<McpAvailableServersResult>, McpError> {
        let free_servers = vec![
            ServerDescription {
                name: "sequential-thinking".to_string(),
                description: "Structured problem decomposition and analysis".to_string(),
            },
            ServerDescription {
                name: "memory".to_string(),
                description: "Official MCP persistent memory server".to_string(),
            },
            ServerDescription {
                name: "fetch".to_string(),
                description: "HTTP requests and web content fetching".to_string(),
            },
            ServerDescription {
                name: "puppeteer".to_string(),
                description: "Browser automation with Puppeteer".to_string(),
            },
            ServerDescription {
                name: "playwright".to_string(),
                description: "Full browser automation with Playwright".to_string(),
            },
            ServerDescription {
                name: "context7".to_string(),
                description: "Library documentation from Context7/Upstash".to_string(),
            },
            ServerDescription {
                name: "markitdown".to_string(),
                description: "Convert documents to markdown".to_string(),
            },
            ServerDescription {
                name: "chrome-devtools".to_string(),
                description: "Chrome DevTools protocol access".to_string(),
            },
            ServerDescription {
                name: "time".to_string(),
                description: "Time and timezone operations".to_string(),
            },
            ServerDescription {
                name: "google-drive".to_string(),
                description: "Google Drive file access (requires OAuth)".to_string(),
            },
        ];

        let key_required = PredefinedServers::requiring_keys();
        let key_required_servers: Vec<KeyRequiredServer> = key_required
            .into_iter()
            .map(|(name, env_var)| KeyRequiredServer {
                name: name.to_string(),
                env_var: env_var.to_string(),
                key_present: std::env::var(env_var).is_ok(),
            })
            .collect();

        // Get currently connected servers
        let status = self.mcp_clients.get_status().await;
        let connected: Vec<String> = status
            .into_iter()
            .filter(|(_, s)| *s == crate::mcp_client::McpClientStatus::Connected)
            .map(|(name, _)| name)
            .collect();

        Ok(Json(McpAvailableServersResult {
            free_servers,
            key_required_servers,
            connected,
        }))
    }

    #[tool(description = "Get status of external MCP integrations")]
    async fn mcp_status(
        &self,
        _params: rmcp::handler::server::wrapper::Parameters<McpStatusParams>,
    ) -> std::result::Result<Json<McpStatusResult>, McpError> {
        let context7 = self.context7.read().await;
        let tavily = self.tavily.read().await;
        let mslearn = self.mslearn.read().await;

        // Start with REST client status
        let mut servers = vec![
            McpServerStatus {
                name: "context7".to_string(),
                status: if context7.is_ready() { "connected" } else { "disconnected" }.to_string(),
                tool_count: if context7.is_ready() { 2 } else { 0 }, // resolve-library-id, get-library-docs
            },
            McpServerStatus {
                name: "tavily".to_string(),
                status: if tavily.is_ready() { "connected" } else { "disconnected" }.to_string(),
                tool_count: if tavily.is_ready() { 4 } else { 0 }, // search, extract, map, crawl
            },
            McpServerStatus {
                name: "microsoft-learn".to_string(),
                status: if mslearn.is_ready() { "connected" } else { "disconnected" }.to_string(),
                tool_count: if mslearn.is_ready() { 3 } else { 0 }, // search, fetch, code_sample_search
            },
            McpServerStatus {
                name: "sequential-thinking".to_string(),
                status: "internal".to_string(),
                tool_count: 1,
            },
        ];

        // Add MCP client manager status
        let mcp_status = self.mcp_clients.get_status().await;
        for (name, status) in mcp_status {
            // Skip if already in the list (REST clients)
            if servers.iter().any(|s| s.name == name) {
                continue;
            }
            let tool_count = self.mcp_clients.list_server_tools(&name).await.len();
            let status_str = match status {
                crate::mcp_client::McpClientStatus::Connected => "connected",
                crate::mcp_client::McpClientStatus::Connecting => "connecting",
                crate::mcp_client::McpClientStatus::Disconnected => "disconnected",
                crate::mcp_client::McpClientStatus::Failed => "failed",
            };
            servers.push(McpServerStatus {
                name,
                status: status_str.to_string(),
                tool_count,
            });
        }

        let mut available_tools = Vec::new();

        if context7.is_ready() {
            available_tools.push(ToolInfo {
                name: "resolve-library-id".to_string(),
                server: "context7".to_string(),
                description: Some("Resolve library name to Context7 ID".to_string()),
            });
            available_tools.push(ToolInfo {
                name: "get-library-docs".to_string(),
                server: "context7".to_string(),
                description: Some("Get documentation for a library".to_string()),
            });
        }

        if tavily.is_ready() {
            available_tools.push(ToolInfo {
                name: "tavily-search".to_string(),
                server: "tavily".to_string(),
                description: Some("Search the web".to_string()),
            });
            available_tools.push(ToolInfo {
                name: "tavily-extract".to_string(),
                server: "tavily".to_string(),
                description: Some("Extract content from URLs".to_string()),
            });
        }

        if mslearn.is_ready() {
            available_tools.push(ToolInfo {
                name: "microsoft_docs_search".to_string(),
                server: "microsoft-learn".to_string(),
                description: Some("Search Microsoft documentation".to_string()),
            });
            available_tools.push(ToolInfo {
                name: "microsoft_docs_fetch".to_string(),
                server: "microsoft-learn".to_string(),
                description: Some("Fetch a documentation page".to_string()),
            });
        }

        available_tools.push(ToolInfo {
            name: "sequentialthinking".to_string(),
            server: "sequential-thinking".to_string(),
            description: Some("Complex problem decomposition".to_string()),
        });

        // Add tools from connected MCP servers
        let mcp_tools = self.mcp_clients.list_all_tools().await;
        for tool in mcp_tools {
            available_tools.push(ToolInfo {
                name: tool.name,
                server: tool.server,
                description: tool.description,
            });
        }

        let connected_count = servers.iter().filter(|s| s.status == "connected" || s.status == "internal").count();

        Ok(Json(McpStatusResult {
            servers,
            available_tools,
            connected_count,
        }))
    }

    // ========================================================================
    // MCP DYNAMIC MANAGEMENT TOOLS
    // ========================================================================

    #[tool(description = "Install a new MCP server (predefined or custom). Returns installation status and available tools.")]
    async fn mcp_install(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<McpInstallParams>,
    ) -> std::result::Result<Json<McpInstallResult>, McpError> {
        let params = params.0;

        // Create the InstalledMcpServer based on package_type
        let server = match params.package_type.as_str() {
            "npm" => InstalledMcpServer::npm(&params.name, &params.package),
            "pip" => InstalledMcpServer::pip(&params.name, &params.package),
            _ => {
                return Ok(Json(McpInstallResult {
                    name: params.name,
                    installed: false,
                    connected: false,
                    tools: Vec::new(),
                    error: Some(format!("Invalid package_type: {}. Use npm or pip", params.package_type)),
                }));
            }
        };

        // Add environment variables and description
        let mut server = server
            .with_description(&params.description)
            .with_auto_connect(params.auto_connect);

        for (key, value) in params.env {
            server = server.with_env(key, value);
        }

        // Install in persistent config
        {
            let mut config = self.mcp_config.write().await;
            if let Err(e) = config.install(server.clone()) {
                return Ok(Json(McpInstallResult {
                    name: params.name,
                    installed: false,
                    connected: false,
                    tools: Vec::new(),
                    error: Some(format!("Failed to save config: {}", e)),
                }));
            }
        }

        // Add to runtime config
        let mcp_config = server.to_server_config();
        self.mcp_clients.add_config(mcp_config).await;

        // Connect if requested
        let mut connected = false;
        let mut tools = Vec::new();

        if params.connect_now {
            if let Ok(()) = self.mcp_clients.connect(&params.name).await {
                connected = true;
                tools = self.mcp_clients
                    .list_server_tools(&params.name)
                    .await
                    .into_iter()
                    .map(|t| t.name)
                    .collect();
            }
        }

        Ok(Json(McpInstallResult {
            name: params.name,
            installed: true,
            connected,
            tools,
            error: None,
        }))
    }

    #[tool(description = "Uninstall an MCP server. Disconnects if connected and removes from persistent config.")]
    async fn mcp_uninstall(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<McpUninstallParams>,
    ) -> std::result::Result<Json<McpUninstallResult>, McpError> {
        let params = params.0;

        // Disconnect if requested and connected
        if params.disconnect {
            let _ = self.mcp_clients.disconnect(&params.name).await;
        }

        // Remove from runtime config
        self.mcp_clients.remove_config(&params.name).await;

        // Remove from persistent config
        let mut config = self.mcp_config.write().await;
        match config.uninstall(&params.name) {
            Ok(Some(_)) => Ok(Json(McpUninstallResult {
                name: params.name,
                uninstalled: true,
                error: None,
            })),
            Ok(None) => Ok(Json(McpUninstallResult {
                name: params.name,
                uninstalled: false,
                error: Some("Server was not installed".to_string()),
            })),
            Err(e) => Ok(Json(McpUninstallResult {
                name: params.name,
                uninstalled: false,
                error: Some(format!("Failed to remove from config: {}", e)),
            })),
        }
    }

    #[tool(description = "Configure an installed MCP server (enable/disable, set environment variables, auto-connect).")]
    async fn mcp_configure(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<McpConfigureParams>,
    ) -> std::result::Result<Json<McpConfigureResult>, McpError> {
        let params = params.0;
        let name = params.name.clone();

        {
            let mut config = self.mcp_config.write().await;

            // Check if server exists
            if !config.is_installed(&name) {
                return Ok(Json(McpConfigureResult {
                    name,
                    configured: false,
                    current_config: None,
                    error: Some("Server not installed".to_string()),
                }));
            }

            // Apply enable/disable
            if let Some(enable) = params.enable {
                if enable {
                    if let Err(e) = config.enable(&name) {
                        return Ok(Json(McpConfigureResult {
                            name,
                            configured: false,
                            current_config: None,
                            error: Some(format!("Failed to enable: {}", e)),
                        }));
                    }
                } else {
                    if let Err(e) = config.disable(&name) {
                        return Ok(Json(McpConfigureResult {
                            name,
                            configured: false,
                            current_config: None,
                            error: Some(format!("Failed to disable: {}", e)),
                        }));
                    }
                    // Disconnect if currently connected
                    let _ = self.mcp_clients.disconnect(&name).await;
                }
            }

            // Set environment variable
            if let Some(env_param) = &params.set_env {
                if let Err(e) = config.set_env(&name, &env_param.key, &env_param.value) {
                    return Ok(Json(McpConfigureResult {
                        name,
                        configured: false,
                        current_config: None,
                        error: Some(format!("Failed to set env: {}", e)),
                    }));
                }
            }

            // Remove environment variable
            if let Some(key) = &params.remove_env {
                if let Err(e) = config.remove_env(&name, key) {
                    return Ok(Json(McpConfigureResult {
                        name,
                        configured: false,
                        current_config: None,
                        error: Some(format!("Failed to remove env: {}", e)),
                    }));
                }
            }
        }

        // Build current config info
        let current_config = {
            let config = self.mcp_config.read().await;
            config.get(&name).map(|server| McpServerInfo {
                name: name.clone(),
                package: server.package.clone(),
                package_type: server.package_type.clone(),
                description: server.description.clone(),
                enabled: server.enabled,
                auto_connect: server.auto_connect,
                env_keys: server.env.keys().cloned().collect(),
                installed_at: server.installed_at.clone(),
                last_connected: server.last_connected.clone(),
            })
        };

        Ok(Json(McpConfigureResult {
            name,
            configured: true,
            current_config,
            error: None,
        }))
    }

    #[tool(description = "List all installed MCP servers with their configuration and status.")]
    async fn mcp_list_installed(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<McpListInstalledParams>,
    ) -> std::result::Result<Json<McpListInstalledResult>, McpError> {
        let params = params.0;

        let config = self.mcp_config.read().await;
        let mut servers = Vec::new();

        for (name, server) in config.list_all() {
            // Filter by include_disabled
            if !params.include_disabled && !server.enabled {
                continue;
            }

            servers.push(McpServerInfo {
                name: name.clone(),
                package: server.package.clone(),
                package_type: server.package_type.clone(),
                description: server.description.clone(),
                enabled: server.enabled,
                auto_connect: server.auto_connect,
                env_keys: server.env.keys().cloned().collect(),
                installed_at: server.installed_at.clone(),
                last_connected: server.last_connected.clone(),
            });
        }

        Ok(Json(McpListInstalledResult {
            total: servers.len(),
            servers,
        }))
    }

    // ========================================================================
    // ACID PIPELINE TOOLS
    // ========================================================================

    #[tool(description = "Phase A - ANALYZE: Research and understand before coding. Combines sequential_thinking + memory_search + knowledge_search + external_docs/search. Use this FIRST to gather context about any task.")]
    async fn analyze(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<AnalyzeParams>,
    ) -> std::result::Result<Json<PipelineResponse<AnalyzeResult>>, McpError> {
        use crate::tools::pipelines::{ThinkingStep, MemoryResult, KnowledgeResult, DocsResult, WebResult};

        let params = params.0;
        let start = std::time::Instant::now();
        let mut warnings: Vec<String> = Vec::new();

        // 1. Sequential thinking if requested
        let mut thinking_steps = Vec::new();
        let mut thinking_conclusion: Option<String> = None;

        if params.think {
            let mut thinking = self.thinking.write().await;
            match thinking.decompose_problem(&params.query).await {
                Ok(result) => {
                    thinking_steps = result.thoughts.iter().map(|s| ThinkingStep {
                        number: s.number,
                        content: s.content.clone(),
                        is_revision: s.is_revision,
                    }).collect();
                    thinking_conclusion = result.conclusion;
                }
                Err(e) => {
                    warnings.push(format!("Sequential thinking failed: {}", e));
                }
            }
        }

        // 2. Search sources
        let mut memory_results = Vec::new();
        let mut knowledge_results = Vec::new();
        let mut docs_results = Vec::new();
        let mut web_results = Vec::new();
        let mut sources_searched = Vec::new();

        for source in &params.sources {
            match source {
                AnalyzeSource::Memory => {
                    sources_searched.push("memory".to_string());
                    let mut rag = self.rag.write().await;
                    if let Ok(results) = rag.search(&params.query, Some(params.max_per_source)).await {
                        memory_results = results.into_iter()
                            .filter(|r| r.score >= params.min_score)
                            .map(|r| MemoryResult {
                                key: r.chunk.document_id,
                                content: r.chunk.text,
                                title: r.chunk.metadata.as_ref()
                                    .and_then(|m| m.get("title"))
                                    .and_then(|v| v.as_str())
                                    .map(String::from),
                                score: r.score,
                                tags: Vec::new(),
                            })
                            .collect();
                    }
                }
                AnalyzeSource::Knowledge => {
                    sources_searched.push("knowledge".to_string());
                    if let Ok(results) = self.db.search_entities(&params.query).await {
                        knowledge_results = results.into_iter()
                            .take(params.max_per_source)
                            .map(|e| KnowledgeResult {
                                name: e.name,
                                entity_type: e.entity_type,
                                observations: e.observations,
                                related: Vec::new(),
                            })
                            .collect();
                    }
                }
                AnalyzeSource::Docs => {
                    if let Some(library) = &params.library {
                        sources_searched.push("docs".to_string());
                        let context7 = self.context7.read().await;
                        if let Ok(Some(result)) = context7.get_library_docs(library, params.topic.as_deref(), 5000).await {
                            docs_results.push(DocsResult {
                                library: library.clone(),
                                content: result.content,
                                code_snippets: result.code_snippets,
                                url: result.url,
                                provider: "context7".to_string(),
                            });
                        }
                    }
                }
                AnalyzeSource::Web => {
                    sources_searched.push("web".to_string());
                    let tavily = self.tavily.read().await;
                    if let Ok(results) = tavily.search(&params.query, params.max_per_source).await {
                        web_results = results.into_iter()
                            .map(|r| WebResult {
                                title: r.title,
                                content: r.content,
                                url: r.url,
                                score: r.score,
                            })
                            .collect();
                    }
                }
                AnalyzeSource::Microsoft => {
                    sources_searched.push("microsoft".to_string());
                    let mslearn = self.mslearn.read().await;
                    if let Ok(results) = mslearn.search(&params.query, params.max_per_source).await {
                        for r in results {
                            docs_results.push(DocsResult {
                                library: "microsoft".to_string(),
                                content: r.content,
                                code_snippets: Vec::new(),
                                url: r.url,
                                provider: "mslearn".to_string(),
                            });
                        }
                    }
                }
            }
        }

        // Calculate confidence
        let total_results = memory_results.len() + knowledge_results.len() +
                           docs_results.len() + web_results.len();
        let confidence = if total_results == 0 { 0.2 }
                        else if total_results < 3 { 0.5 }
                        else if total_results < 10 { 0.7 }
                        else { 0.9 };

        let needs_more_research = confidence < 0.5;
        let suggested_query = if needs_more_research {
            Some(format!("{} best practices", params.query))
        } else { None };

        // Build summary
        let mut summary_parts = Vec::new();
        if let Some(ref c) = thinking_conclusion {
            summary_parts.push(format!("Thinking: {}", c));
        }
        if !memory_results.is_empty() {
            summary_parts.push(format!("{} memories found", memory_results.len()));
        }
        if !knowledge_results.is_empty() {
            summary_parts.push(format!("{} entities found", knowledge_results.len()));
        }
        if !docs_results.is_empty() {
            summary_parts.push(format!("{} docs found", docs_results.len()));
        }
        if !web_results.is_empty() {
            summary_parts.push(format!("{} web results", web_results.len()));
        }
        let summary = if summary_parts.is_empty() {
            "No relevant information found".to_string()
        } else {
            summary_parts.join(". ")
        };

        let recommendation = if confidence >= 0.7 { "Proceed to Phase B (prepare)".to_string() }
                            else if confidence >= 0.5 { "Consider additional research".to_string() }
                            else { "More research needed".to_string() };

        let result = AnalyzeResult {
            query: params.query,
            thinking: thinking_steps,
            thinking_conclusion,
            memory_results,
            knowledge_results,
            docs_results,
            web_results,
            summary,
            sources_searched,
            confidence,
            recommendation,
            needs_more_research,
            suggested_query,
        };

        let duration_ms = start.elapsed().as_millis() as u64;
        let mut response = if confidence >= 0.5 {
            PipelineResponse::ok_with_next(result, duration_ms, "prepare")
        } else {
            PipelineResponse::ok(result, duration_ms)
        };
        for w in warnings { response = response.with_warning(w); }

        Ok(Json(response))
    }

    #[tool(description = "Phase B - PREPARE: Document decisions BEFORE coding. Store notes in memory and add entities/relations to knowledge graph. Use after 'analyze' to record what you learned.")]
    async fn prepare(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<PrepareParams>,
    ) -> std::result::Result<Json<PipelineResponse<PrepareResult>>, McpError> {
        use crate::tools::pipelines::{RememberResult, EntityResult, RelationResult, ObservationResult};

        let params = params.0;
        let start = std::time::Instant::now();
        let mut errors = Vec::new();

        let mut remembered = Vec::new();
        let mut entities_created = Vec::new();
        let mut relations_created = Vec::new();
        let mut observations_added = Vec::new();

        // 1. Store memories
        for item in params.remember {
            let key = uuid::Uuid::new_v4().to_string();
            let now = chrono::Utc::now().timestamp();

            let doc = whytcard_database::CreateDocument::new(&item.content)
                .with_key(&key)
                .with_tags(item.tags);
            let doc = if let Some(title) = &item.title {
                doc.with_title(title)
            } else { doc };

            match self.db.create_document(doc).await {
                Ok(_) => {
                    let mut indexed = false;
                    if params.index {
                        let rag_doc = whytcard_rag::Document::new(&item.content)
                            .with_id(&key)
                            .with_metadata_field("category", item.category);
                        let mut rag = self.rag.write().await;
                        if rag.index(&rag_doc).await.is_ok() { indexed = true; }
                    }
                    remembered.push(RememberResult { key, indexed, stored_at: now });
                }
                Err(e) => errors.push(format!("Memory store failed: {}", e)),
            }
        }

        // 2. Add entities
        for entity in params.entities {
            let input = CreateEntity::new(&entity.name, &entity.entity_type)
                .with_observations(entity.observations.clone());
            match self.db.create_entity(input).await {
                Ok(_) => entities_created.push(EntityResult {
                    name: entity.name,
                    entity_type: entity.entity_type,
                    created: true,
                    observations_added: entity.observations.len(),
                }),
                Err(e) => errors.push(format!("Entity create failed: {}", e)),
            }
        }

        // 3. Add relations (need to get entity IDs first)
        for rel in params.relations {
            // Get from entity
            let from_entity = match self.db.get_entity_by_name(&rel.from).await {
                Ok(Some(e)) => e,
                _ => {
                    errors.push(format!("Entity not found: {}", rel.from));
                    continue;
                }
            };
            let from_id = match from_entity.id {
                Some(id) => id,
                None => {
                    errors.push(format!("Entity has no ID: {}", rel.from));
                    continue;
                }
            };
            // Get to entity
            let to_entity = match self.db.get_entity_by_name(&rel.to).await {
                Ok(Some(e)) => e,
                _ => {
                    errors.push(format!("Entity not found: {}", rel.to));
                    continue;
                }
            };
            let to_id = match to_entity.id {
                Some(id) => id,
                None => {
                    errors.push(format!("Entity has no ID: {}", rel.to));
                    continue;
                }
            };
            let input = CreateRelation::new(from_id, to_id, &rel.relation_type);
            match self.db.create_relation(input).await {
                Ok(_) => relations_created.push(RelationResult {
                    from: rel.from,
                    to: rel.to,
                    relation_type: rel.relation_type,
                    created: true,
                }),
                Err(e) => errors.push(format!("Relation create failed: {}", e)),
            }
        }

        // 4. Add observations (one at a time)
        for obs in params.observations {
            let mut added_count = 0;
            for observation in &obs.observations {
                match self.db.add_observation(&obs.entity_name, observation).await {
                    Ok(_) => added_count += 1,
                    Err(e) => errors.push(format!("Observation add failed: {}", e)),
                }
            }
            observations_added.push(ObservationResult {
                entity_name: obs.entity_name,
                added: added_count,
            });
        }

        // 5. Save user instructions to DB (persisted for future sessions)
        let mut user_instructions_saved = Vec::new();
        for ui_def in &params.user_instructions {
            use crate::tools::pipelines::UserInstructionResult;

            let user_instruction = ui_def.to_user_instruction(&params.user_id);

            // Store in DB as a document with special category
            let key = format!("user_instruction:{}:{}", params.user_id, ui_def.key);
            let content = serde_json::json!({
                "key": ui_def.key,
                "value": ui_def.value,
                "category": ui_def.category,
                "priority": ui_def.priority,
                "user_id": params.user_id,
            });

            let doc = whytcard_database::CreateDocument::new(content.to_string())
                .with_key(&key)
                .with_title(format!("User Instruction: {}", ui_def.key))
                .with_tags(vec!["user_instruction".to_string(), ui_def.category.clone()]);

            let replaced = self.db.get_document(&key).await.map(|d| d.is_some()).unwrap_or(false);

            match self.db.create_document(doc).await {
                Ok(_) => {
                    // Also add to CORTEX instructions manager for immediate use
                    self.cortex.add_user_instruction(user_instruction).await;

                    user_instructions_saved.push(UserInstructionResult {
                        key: ui_def.key.clone(),
                        category: ui_def.category.clone(),
                        saved: true,
                        replaced,
                    });
                }
                Err(e) => {
                    errors.push(format!("User instruction save failed: {}", e));
                    user_instructions_saved.push(UserInstructionResult {
                        key: ui_def.key.clone(),
                        category: ui_def.category.clone(),
                        saved: false,
                        replaced: false,
                    });
                }
            }
        }

        let total_stored = remembered.len() + entities_created.len() +
                          relations_created.len() + observations_added.len() +
                          user_instructions_saved.iter().filter(|u| u.saved).count();
        let total_processed = total_stored + errors.len();

        let summary = format!(
            "Prepared {} items: {} memories, {} entities, {} relations, {} observations, {} user instructions. {} errors.",
            total_stored, remembered.len(), entities_created.len(),
            relations_created.len(), observations_added.len(),
            user_instructions_saved.iter().filter(|u| u.saved).count(),
            errors.len()
        );

        let result = PrepareResult {
            remembered,
            entities_created,
            relations_created,
            observations_added,
            user_instructions_saved,
            total_processed,
            total_stored,
            errors,
            summary,
        };

        let duration_ms = start.elapsed().as_millis() as u64;
        Ok(Json(PipelineResponse::ok_with_next(result, duration_ms, "code")))
    }

    #[tool(description = "Phase C - CODE: Execute shell commands during development. Run compilers, tests, linters. Use after 'prepare' to execute and verify code.")]
    async fn code(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<CodeParams>,
    ) -> std::result::Result<Json<PipelineResponse<CodeResult>>, McpError> {
        use crate::tools::pipelines::{ExecuteResult, FeedbackResult, DetectedError};
        use std::process::Stdio;
        use tokio::process::Command;

        let params = params.0;
        let start = std::time::Instant::now();

        let mut executions = Vec::new();
        let mut feedbacks = Vec::new();
        let mut all_success = true;
        let mut commands_failed = 0;

        // Execute commands
        for cmd in &params.commands {
            let cmd_start = std::time::Instant::now();

            let shell = if cfg!(windows) { "powershell" } else { "sh" };
            let shell_arg = if cfg!(windows) { "-Command" } else { "-c" };

            let mut command = Command::new(shell);
            command.arg(shell_arg).arg(&cmd.command);
            if let Some(cwd) = &cmd.cwd {
                command.current_dir(cwd);
            }
            for (k, v) in &cmd.env {
                command.env(k, v);
            }
            command.stdout(Stdio::piped()).stderr(Stdio::piped());

            let timeout = tokio::time::Duration::from_secs(cmd.timeout_secs);
            let result = tokio::time::timeout(timeout, command.output()).await;

            let duration_ms = cmd_start.elapsed().as_millis() as u64;

            let exec_result = match result {
                Ok(Ok(output)) => {
                    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
                    let stderr = String::from_utf8_lossy(&output.stderr).to_string();
                    ExecuteResult {
                        command: cmd.command.clone(),
                        label: cmd.label.clone(),
                        success: output.status.success(),
                        exit_code: output.status.code().unwrap_or(-1),
                        stdout: if params.separate_stderr { stdout } else { format!("{}\n{}", stdout, stderr) },
                        stderr: if params.separate_stderr { stderr } else { String::new() },
                        duration_ms,
                    }
                }
                Ok(Err(e)) => ExecuteResult {
                    command: cmd.command.clone(),
                    label: cmd.label.clone(),
                    success: false,
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: format!("Exec failed: {}", e),
                    duration_ms,
                },
                Err(_) => ExecuteResult {
                    command: cmd.command.clone(),
                    label: cmd.label.clone(),
                    success: false,
                    exit_code: -1,
                    stdout: String::new(),
                    stderr: format!("Timeout after {}s", cmd.timeout_secs),
                    duration_ms,
                },
            };

            if !exec_result.success {
                all_success = false;
                commands_failed += 1;
                if params.stop_on_failure {
                    executions.push(exec_result);
                    break;
                }
            }
            executions.push(exec_result);
        }

        // Process feedback
        for fb in &params.feedback {
            if let Ok(new_confidence) = self.cortex.provide_feedback(&fb.rule_id, fb.success).await {
                feedbacks.push(FeedbackResult {
                    rule_id: fb.rule_id.clone(),
                    recorded: true,
                    new_confidence,
                });
            }
        }

        // Parse errors from output
        let mut detected_errors = Vec::new();
        let mut detected_warnings = Vec::new();
        for exec in &executions {
            let output = format!("{}\n{}", exec.stdout, exec.stderr);
            for line in output.lines() {
                let lower = line.to_lowercase();
                if lower.contains("error[") || lower.contains("error:") {
                    detected_errors.push(DetectedError {
                        error_type: "compile".to_string(),
                        file: None,
                        line: None,
                        message: line.to_string(),
                        severity: "error".to_string(),
                    });
                } else if lower.contains("warning[") || lower.contains("warning:") {
                    detected_warnings.push(DetectedError {
                        error_type: "lint".to_string(),
                        file: None,
                        line: None,
                        message: line.to_string(),
                        severity: "warning".to_string(),
                    });
                }
            }
        }

        let commands_executed = executions.len();
        let summary = if all_success {
            format!("All {} commands succeeded", commands_executed)
        } else {
            format!("{}/{} commands failed", commands_failed, commands_executed)
        };

        let recommendation = if all_success && detected_errors.is_empty() {
            Some("Proceed to Phase I (verify)".to_string())
        } else if !detected_errors.is_empty() {
            Some(format!("Fix {} errors before proceeding", detected_errors.len()))
        } else {
            Some("Review failures before proceeding".to_string())
        };

        let result = CodeResult {
            executions,
            feedbacks,
            all_success,
            commands_executed,
            commands_failed,
            detected_errors,
            detected_warnings,
            summary,
            recommendation,
        };

        let duration_ms = start.elapsed().as_millis() as u64;
        let response = if all_success {
            PipelineResponse::ok_with_next(result, duration_ms, "verify")
        } else {
            PipelineResponse::ok(result, duration_ms)
        };

        Ok(Json(response))
    }

    #[tool(description = "Phase I - VERIFY: Validate completely before commit. Run build, tests, lint, format checks. Use after 'code' to ensure everything passes.")]
    async fn verify(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<VerifyParams>,
    ) -> std::result::Result<Json<PipelineResponse<VerifyResult>>, McpError> {
        use crate::tools::pipelines::{CheckResult, TestSummary, VerifyCommand};
        use std::process::Stdio;
        use tokio::process::Command;

        let params = params.0;
        let start = std::time::Instant::now();

        // Get commands based on language preset or custom
        let commands: Vec<VerifyCommand> = if !params.custom_commands.is_empty() {
            params.custom_commands.clone()
        } else if let Some(lang) = &params.language {
            let all = lang.default_commands();
            if params.checks.contains(&VerifyCheck::All) {
                all
            } else {
                all.into_iter().filter(|c| params.checks.contains(&c.check_type)).collect()
            }
        } else {
            return Err(McpError::invalid_params(
                "No commands - specify language or custom_commands", None
            ));
        };

        let mut checks = Vec::new();
        let mut all_passed = true;
        let mut total_errors = 0;
        let mut total_warnings = 0;
        let mut checks_passed = 0;
        let mut checks_failed = 0;
        let mut test_summary: Option<TestSummary> = None;
        let mut blockers = Vec::new();
        let mut warnings_list = Vec::new();

        for cmd in commands {
            let cmd_start = std::time::Instant::now();

            let shell = if cfg!(windows) { "powershell" } else { "sh" };
            let shell_arg = if cfg!(windows) { "-Command" } else { "-c" };

            let mut command = Command::new(shell);
            command.arg(shell_arg).arg(&cmd.command);
            if let Some(cwd) = &params.cwd {
                command.current_dir(cwd);
            }
            for (k, v) in &params.env {
                command.env(k, v);
            }
            command.stdout(Stdio::piped()).stderr(Stdio::piped());

            let timeout = tokio::time::Duration::from_secs(cmd.timeout_secs);
            let output_result = tokio::time::timeout(timeout, command.output()).await;

            let (success, exit_code, stdout, stderr) = match output_result {
                Ok(Ok(out)) => (
                    out.status.success(),
                    out.status.code().unwrap_or(-1),
                    String::from_utf8_lossy(&out.stdout).to_string(),
                    String::from_utf8_lossy(&out.stderr).to_string(),
                ),
                _ => (false, -1, String::new(), "Timeout or exec error".to_string()),
            };

            let output = format!("{}\n{}", stdout, stderr).to_lowercase();
            let error_count = output.matches("error[").count() + output.matches("error:").count();
            let warning_count = output.matches("warning[").count() + output.matches("warning:").count();

            let passed = success && (error_count == 0 || !params.strict);

            if passed {
                checks_passed += 1;
            } else {
                checks_failed += 1;
                all_passed = false;
                blockers.push(format!("{} failed", cmd.label.as_deref().unwrap_or(&cmd.command)));
            }

            if warning_count > 0 {
                warnings_list.push(format!("{} warnings in {}", warning_count, cmd.label.as_deref().unwrap_or(&cmd.command)));
            }

            total_errors += error_count;
            total_warnings += warning_count;

            // Extract test summary if test command
            if matches!(cmd.check_type, VerifyCheck::Test) {
                if let Some(line) = format!("{}\n{}", stdout, stderr).lines().find(|l| l.contains("test result:")) {
                    let extract_num = |text: &str, after: &str| -> usize {
                        text.split_whitespace()
                            .zip(text.split_whitespace().skip(1))
                            .find(|(_, next)| *next == after || next.starts_with(after))
                            .and_then(|(num, _)| num.parse().ok())
                            .unwrap_or(0)
                    };
                    let passed_t = extract_num(line, "passed");
                    let failed_t = extract_num(line, "failed");
                    let skipped_t = extract_num(line, "ignored");
                    test_summary = Some(TestSummary {
                        total: passed_t + failed_t + skipped_t,
                        passed: passed_t,
                        failed: failed_t,
                        skipped: skipped_t,
                        coverage_percent: None,
                    });
                }
            }

            let trunc_output = if format!("{}\n{}", stdout, stderr).len() > 2000 {
                format!("{}... (truncated)", &format!("{}\n{}", stdout, stderr)[..2000])
            } else {
                format!("{}\n{}", stdout, stderr)
            };

            checks.push(CheckResult {
                check_type: format!("{:?}", cmd.check_type).to_lowercase(),
                command: cmd.command,
                label: cmd.label,
                passed,
                exit_code,
                output: trunc_output,
                duration_ms: cmd_start.elapsed().as_millis() as u64,
                error_count,
                warning_count,
            });

            if !passed && params.stop_on_failure {
                break;
            }
        }

        let ready_to_commit = all_passed && (total_warnings == 0 || !params.strict);
        let total_duration_ms = start.elapsed().as_millis() as u64;
        let total_checks = checks.len();

        let summary = if ready_to_commit {
            "All checks passed - ready to commit".to_string()
        } else if all_passed {
            format!("All checks passed but {} warnings - review before commit", total_warnings)
        } else {
            format!("{} checks failed - fix issues before commit", checks_failed)
        };

        let result = VerifyResult {
            checks,
            test_summary,
            all_passed,
            ready_to_commit,
            total_checks,
            checks_passed,
            checks_failed,
            total_errors,
            total_warnings,
            total_duration_ms,
            summary,
            blockers,
            warnings: warnings_list,
        };

        let response = if ready_to_commit {
            PipelineResponse::ok_with_next(result, total_duration_ms, "document")
        } else {
            PipelineResponse::ok(result, total_duration_ms)
        };

        Ok(Json(response))
    }

    #[tool(description = "Phase D - DOCUMENT: Trace and learn after completing work. Log tasks, record decisions, store patterns, provide feedback. Use after 'verify' to document what was done.")]
    async fn document(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<DocumentParams>,
    ) -> std::result::Result<Json<PipelineResponse<DocumentResult>>, McpError> {
        use crate::tools::pipelines::{DocumentedItem, FeedbackDocResult};

        let params = params.0;
        let start = std::time::Instant::now();

        let mut documented = Vec::new();
        let mut feedbacks = Vec::new();
        let mut total_failed = 0;

        // 1. Log tasks
        for log in &params.task_logs {
            let key = uuid::Uuid::new_v4().to_string();
            let content = match serde_json::to_string_pretty(log) {
                Ok(c) => c,
                Err(e) => {
                    total_failed += 1;
                    documented.push(DocumentedItem {
                        item_type: "task_log".to_string(),
                        id: String::new(),
                        success: false,
                        error: Some(e.to_string()),
                    });
                    continue;
                }
            };

            let mut tags = params.global_tags.clone();
            tags.push("task_log".to_string());
            tags.push(log.outcome.clone());

            let doc = whytcard_database::CreateDocument::new(&content)
                .with_key(&key)
                .with_title(&log.task)
                .with_tags(tags);

            match self.db.create_document(doc).await {
                Ok(_) => documented.push(DocumentedItem {
                    item_type: "task_log".to_string(),
                    id: key,
                    success: true,
                    error: None,
                }),
                Err(e) => {
                    total_failed += 1;
                    documented.push(DocumentedItem {
                        item_type: "task_log".to_string(),
                        id: String::new(),
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        // 2. Record decisions
        for decision in &params.decisions {
            let key = uuid::Uuid::new_v4().to_string();
            let content = match serde_json::to_string_pretty(decision) {
                Ok(c) => c,
                Err(e) => {
                    total_failed += 1;
                    documented.push(DocumentedItem {
                        item_type: "decision".to_string(),
                        id: String::new(),
                        success: false,
                        error: Some(e.to_string()),
                    });
                    continue;
                }
            };

            let mut tags = params.global_tags.clone();
            tags.push("decision".to_string());

            let doc = whytcard_database::CreateDocument::new(&content)
                .with_key(&key)
                .with_title(&decision.decision)
                .with_tags(tags);

            match self.db.create_document(doc).await {
                Ok(_) => documented.push(DocumentedItem {
                    item_type: "decision".to_string(),
                    id: key,
                    success: true,
                    error: None,
                }),
                Err(e) => {
                    total_failed += 1;
                    documented.push(DocumentedItem {
                        item_type: "decision".to_string(),
                        id: String::new(),
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        // 3. Store patterns
        for pattern in &params.patterns {
            let key = uuid::Uuid::new_v4().to_string();
            let content = match serde_json::to_string_pretty(pattern) {
                Ok(c) => c,
                Err(e) => {
                    total_failed += 1;
                    documented.push(DocumentedItem {
                        item_type: "pattern".to_string(),
                        id: String::new(),
                        success: false,
                        error: Some(e.to_string()),
                    });
                    continue;
                }
            };

            let mut tags = params.global_tags.clone();
            tags.push("pattern".to_string());

            let doc = whytcard_database::CreateDocument::new(&content)
                .with_key(&key)
                .with_title(&pattern.name)
                .with_tags(tags);

            match self.db.create_document(doc).await {
                Ok(_) => documented.push(DocumentedItem {
                    item_type: "pattern".to_string(),
                    id: key,
                    success: true,
                    error: None,
                }),
                Err(e) => {
                    total_failed += 1;
                    documented.push(DocumentedItem {
                        item_type: "pattern".to_string(),
                        id: String::new(),
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        // 4. Provide feedbacks
        for fb in &params.feedbacks {
            if let Ok(new_confidence) = self.cortex.provide_feedback(&fb.rule_id, fb.success).await {
                feedbacks.push(FeedbackDocResult {
                    rule_id: fb.rule_id.clone(),
                    new_confidence,
                });
            }
        }

        // 5. Add knowledge observations (one at a time)
        for knowledge in &params.knowledge {
            let mut success = true;
            for observation in &knowledge.observations {
                if let Err(e) = self.db.add_observation(&knowledge.entity_name, observation).await {
                    total_failed += 1;
                    documented.push(DocumentedItem {
                        item_type: "knowledge".to_string(),
                        id: knowledge.entity_name.clone(),
                        success: false,
                        error: Some(e.to_string()),
                    });
                    success = false;
                    break;
                }
            }
            if success {
                documented.push(DocumentedItem {
                    item_type: "knowledge".to_string(),
                    id: knowledge.entity_name.clone(),
                    success: true,
                    error: None,
                });
            }
        }

        // 6. Document error fixes
        for fix in &params.error_fixes {
            let key = uuid::Uuid::new_v4().to_string();
            let content = match serde_json::to_string_pretty(fix) {
                Ok(c) => c,
                Err(e) => {
                    total_failed += 1;
                    documented.push(DocumentedItem {
                        item_type: "error_fix".to_string(),
                        id: String::new(),
                        success: false,
                        error: Some(e.to_string()),
                    });
                    continue;
                }
            };

            let mut tags = params.global_tags.clone();
            tags.push("error_fix".to_string());

            let doc = whytcard_database::CreateDocument::new(&content)
                .with_key(&key)
                .with_title(&fix.error)
                .with_tags(tags);

            match self.db.create_document(doc).await {
                Ok(_) => documented.push(DocumentedItem {
                    item_type: "error_fix".to_string(),
                    id: key,
                    success: true,
                    error: None,
                }),
                Err(e) => {
                    total_failed += 1;
                    documented.push(DocumentedItem {
                        item_type: "error_fix".to_string(),
                        id: String::new(),
                        success: false,
                        error: Some(e.to_string()),
                    });
                }
            }
        }

        let total_processed = documented.len() + feedbacks.len();
        let total_documented = documented.iter().filter(|d| d.success).count() + feedbacks.len();

        let summary = format!(
            "Documented {} items: {} successful, {} failed",
            total_processed, total_documented, total_failed
        );

        let result = DocumentResult {
            documented,
            feedbacks,
            total_processed,
            total_documented,
            total_failed,
            summary,
            session_id: params.session_id,
        };

        let duration_ms = start.elapsed().as_millis() as u64;
        Ok(Json(PipelineResponse::ok(result, duration_ms)))
    }

    #[tool(description = "MANAGE: Administrative tool for MCP servers and CORTEX. Install/uninstall servers, get status, manage instructions. Use for system administration tasks.")]
    async fn manage(
        &self,
        params: rmcp::handler::server::wrapper::Parameters<ManageParams>,
    ) -> std::result::Result<Json<PipelineResponse<ManageResult>>, McpError> {
        use crate::tools::pipelines::{ServerInfo, ToolInfoItem, CortexStatsInfo, InstructionInfoItem};

        let params = params.0;
        let start = std::time::Instant::now();

        let result = match params.action {
            ManageAction::Status => {
                let status_map = self.mcp_clients.get_status().await;
                let connected_count = status_map.values().filter(|s| **s == crate::mcp_client::McpClientStatus::Connected).count();
                let mut servers = Vec::new();
                for (name, status) in &status_map {
                    let tool_count = self.mcp_clients.list_server_tools(name).await.len();
                    servers.push(ServerInfo {
                        name: name.clone(),
                        status: format!("{:?}", status),
                        tool_count,
                        package: None,
                        description: None,
                        enabled: true,
                        auto_connect: false,
                    });
                }
                ManageResult {
                    action: "status".to_string(),
                    success: true,
                    message: format!("{} servers connected", connected_count),
                    servers,
                    tools: Vec::new(),
                    cortex_stats: None,
                    cleaned_count: None,
                    tool_result: None,
                    instructions: Vec::new(),
                    instruction_content: None,
                    connected_count,
                    error: None,
                }
            }
            ManageAction::ListAvailable => {
                // Predefined servers - hardcoded list of commonly used servers
                let predefined_names = vec![
                    ("sequential-thinking", "@modelcontextprotocol/server-sequential-thinking", "Structured problem decomposition"),
                    ("memory", "@modelcontextprotocol/server-memory", "Persistent memory across sessions"),
                    ("filesystem", "@modelcontextprotocol/server-filesystem", "File system access"),
                    ("fetch", "@modelcontextprotocol/server-fetch", "HTTP requests"),
                    ("puppeteer", "@modelcontextprotocol/server-puppeteer", "Browser automation"),
                    ("github", "@modelcontextprotocol/server-github", "GitHub API access"),
                    ("context7", "context7-mcp", "Library documentation"),
                    ("tavily", "tavily-mcp", "Web search"),
                ];
                let servers: Vec<ServerInfo> = predefined_names.iter().map(|(name, package, desc)| ServerInfo {
                    name: name.to_string(),
                    status: "available".to_string(),
                    tool_count: 0,
                    package: Some(package.to_string()),
                    description: Some(desc.to_string()),
                    enabled: true,
                    auto_connect: false,
                }).collect();
                let count = servers.len();
                let status_map = self.mcp_clients.get_status().await;
                let connected_count = status_map.values().filter(|s| **s == crate::mcp_client::McpClientStatus::Connected).count();
                ManageResult {
                    action: "list_available".to_string(),
                    success: true,
                    message: format!("{} predefined servers", count),
                    servers,
                    tools: Vec::new(),
                    cortex_stats: None,
                    cleaned_count: None,
                    tool_result: None,
                    instructions: Vec::new(),
                    instruction_content: None,
                    connected_count,
                    error: None,
                }
            }
            ManageAction::ListInstalled => {
                let config = self.mcp_config.read().await;
                let servers: Vec<ServerInfo> = config.list_all().map(|(name, server)| {
                    ServerInfo {
                        name: name.clone(),
                        status: if server.enabled { "installed" } else { "disabled" }.to_string(),
                        tool_count: 0,
                        package: Some(server.package.clone()),
                        description: Some(server.description.clone()),
                        enabled: server.enabled,
                        auto_connect: server.auto_connect,
                    }
                }).collect();
                let count = servers.len();
                let status_map = self.mcp_clients.get_status().await;
                let connected_count = status_map.values().filter(|s| **s == crate::mcp_client::McpClientStatus::Connected).count();
                ManageResult {
                    action: "list_installed".to_string(),
                    success: true,
                    message: format!("{} servers installed", count),
                    servers,
                    tools: Vec::new(),
                    cortex_stats: None,
                    cleaned_count: None,
                    tool_result: None,
                    instructions: Vec::new(),
                    instruction_content: None,
                    connected_count,
                    error: None,
                }
            }
            ManageAction::ListTools => {
                let all_tools = self.mcp_clients.list_all_tools().await;
                let mut tools = Vec::new();
                for tool in all_tools {
                    if let Some(filter) = &params.filter_server {
                        if &tool.server != filter { continue; }
                    }
                    tools.push(ToolInfoItem {
                        name: tool.name.clone(),
                        server: tool.server.clone(),
                        description: tool.description.clone(),
                    });
                }
                let count = tools.len();
                let status_map = self.mcp_clients.get_status().await;
                let connected_count = status_map.values().filter(|s| **s == crate::mcp_client::McpClientStatus::Connected).count();
                ManageResult {
                    action: "list_tools".to_string(),
                    success: true,
                    message: format!("{} tools available", count),
                    servers: Vec::new(),
                    tools,
                    cortex_stats: None,
                    cleaned_count: None,
                    tool_result: None,
                    instructions: Vec::new(),
                    instruction_content: None,
                    connected_count,
                    error: None,
                }
            }
            ManageAction::CortexStats => {
                let stats = self.cortex.get_stats().await;
                let semantic_facts = stats.get("semantic")
                    .and_then(|s| s.get("total_facts"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
                let episodic_events = stats.get("episodic")
                    .and_then(|s| s.get("total_episodes"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
                let procedural_rules = stats.get("procedural")
                    .and_then(|s| s.get("total_rules"))
                    .and_then(|v| v.as_u64())
                    .unwrap_or(0) as usize;
                let status_map = self.mcp_clients.get_status().await;
                let connected_count = status_map.values().filter(|s| **s == crate::mcp_client::McpClientStatus::Connected).count();
                ManageResult {
                    action: "cortex_stats".to_string(),
                    success: true,
                    message: format!("CORTEX: {} facts, {} events, {} rules", semantic_facts, episodic_events, procedural_rules),
                    servers: Vec::new(),
                    tools: Vec::new(),
                    cortex_stats: Some(CortexStatsInfo {
                        semantic_facts,
                        episodic_events,
                        procedural_rules,
                        status: "running".to_string(),
                    }),
                    cleaned_count: None,
                    tool_result: None,
                    instructions: Vec::new(),
                    instruction_content: None,
                    connected_count,
                    error: None,
                }
            }
            ManageAction::CortexCleanup => {
                let status_map = self.mcp_clients.get_status().await;
                let connected_count = status_map.values().filter(|s| **s == crate::mcp_client::McpClientStatus::Connected).count();
                match self.cortex.cleanup(params.retention_days).await {
                    Ok(cleaned) => ManageResult {
                        action: "cortex_cleanup".to_string(),
                        success: true,
                        message: format!("Cleaned {} old records", cleaned),
                        servers: Vec::new(),
                        tools: Vec::new(),
                        cortex_stats: None,
                        cleaned_count: Some(cleaned),
                        tool_result: None,
                        instructions: Vec::new(),
                        instruction_content: None,
                        connected_count,
                        error: None,
                    },
                    Err(e) => ManageResult {
                        action: "cortex_cleanup".to_string(),
                        success: false,
                        message: "Cleanup failed".to_string(),
                        servers: Vec::new(),
                        tools: Vec::new(),
                        cortex_stats: None,
                        cleaned_count: None,
                        tool_result: None,
                        instructions: Vec::new(),
                        instruction_content: None,
                        connected_count,
                        error: Some(e.to_string()),
                    }
                }
            }
            ManageAction::InstructionsList => {
                let instructions_list = self.cortex.get_global_instructions().await;
                let instructions: Vec<InstructionInfoItem> = instructions_list.iter().map(|i| {
                    InstructionInfoItem {
                        name: i.name.clone(),
                        description: Some(i.description.clone()),
                        apply_to: Some(i.apply_to.clone()),
                    }
                }).collect();
                let count = instructions.len();
                let status_map = self.mcp_clients.get_status().await;
                let connected_count = status_map.values().filter(|s| **s == crate::mcp_client::McpClientStatus::Connected).count();
                ManageResult {
                    action: "instructions_list".to_string(),
                    success: true,
                    message: format!("{} instructions loaded", count),
                    servers: Vec::new(),
                    tools: Vec::new(),
                    cortex_stats: None,
                    cleaned_count: None,
                    tool_result: None,
                    instructions,
                    instruction_content: None,
                    connected_count,
                    error: None,
                }
            }
            ManageAction::InstructionsReload => {
                let status_map = self.mcp_clients.get_status().await;
                let connected_count = status_map.values().filter(|s| **s == crate::mcp_client::McpClientStatus::Connected).count();
                match self.cortex.reload_instructions().await {
                    Ok(_) => {
                        let count = self.cortex.get_global_instructions().await.len();
                        ManageResult {
                            action: "instructions_reload".to_string(),
                            success: true,
                            message: format!("Reloaded {} instructions", count),
                            servers: Vec::new(),
                            tools: Vec::new(),
                            cortex_stats: None,
                            cleaned_count: None,
                            tool_result: None,
                            instructions: Vec::new(),
                            instruction_content: None,
                            connected_count,
                            error: None,
                        }
                    }
                    Err(e) => ManageResult {
                        action: "instructions_reload".to_string(),
                        success: false,
                        message: "Reload failed".to_string(),
                        servers: Vec::new(),
                        tools: Vec::new(),
                        cortex_stats: None,
                        cleaned_count: None,
                        tool_result: None,
                        instructions: Vec::new(),
                        instruction_content: None,
                        connected_count,
                        error: Some(e.to_string()),
                    }
                }
            }
            _ => {
                let status_map = self.mcp_clients.get_status().await;
                let connected_count = status_map.values().filter(|s| **s == crate::mcp_client::McpClientStatus::Connected).count();
                ManageResult {
                    action: format!("{:?}", params.action).to_lowercase(),
                    success: false,
                    message: "Action not yet implemented in pipeline".to_string(),
                    servers: Vec::new(),
                    tools: Vec::new(),
                    cortex_stats: None,
                    cleaned_count: None,
                    tool_result: None,
                    instructions: Vec::new(),
                    instruction_content: None,
                    connected_count,
                    error: Some("Use atomic tools for this action".to_string()),
                }
            }
        };

        let duration_ms = start.elapsed().as_millis() as u64;
        Ok(Json(PipelineResponse::ok(result, duration_ms)))
    }
}

// Implement the server handler for MCP
#[tool_handler]
impl rmcp::ServerHandler for IntelligenceServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: Default::default(),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: self.config.server_name.clone(),
                version: self.config.version.clone(),
                ..Default::default()
            },
            instructions: Some(
                "WhytCard Intelligence - A cognitive memory and knowledge system powered by CORTEX. \
                 Use cortex_* tools for intelligent query processing and learning. \
                 Use memory_* tools to store and search information. \
                 Use knowledge_* tools to manage entities and relations."
                    .to_string(),
            ),
        }
    }
}

impl IntelligenceServer {
    /// Run the server with stdio transport
    pub async fn run_stdio(self) -> crate::Result<()> {
        tracing::info!("Starting Intelligence MCP server on stdio");

        let service = self
            .serve(rmcp::transport::stdio())
            .await
            .map_err(|e| IntelligenceError::config(format!("Failed to start server: {}", e)))?;

        service
            .waiting()
            .await
            .map_err(|e| IntelligenceError::config(format!("Server error: {}", e)))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use rmcp::ServerHandler;
    use tempfile::TempDir;

    #[tokio::test]
    async fn test_server_creation() {
        let temp = TempDir::new().unwrap();
        let server = IntelligenceServer::for_testing(temp.path()).await;
        assert!(server.is_ok());
    }

    #[tokio::test]
    async fn test_server_info() {
        let temp = TempDir::new().unwrap();
        let server = IntelligenceServer::for_testing(temp.path()).await.unwrap();

        let info = server.get_info();
        assert_eq!(info.server_info.name, "whytcard-intelligence");
        assert!(info.instructions.is_some());
    }
}
