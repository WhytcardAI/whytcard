//! External Tools Tests for WhytCard Intelligence
//!
//! Tests pour les outils d'intégration externe:
//! - sequential_thinking
//! - external_docs (Context7, MS Learn)
//! - external_search (Tavily)
//! - external_mcp_call

mod common;

use common::TestContext;
use whytcard_intelligence::tools::{
    SequentialThinkingParams, ExternalDocsParams, ExternalSearchParams,
    ExternalMcpCallParams, McpStatusParams,
};

// =============================================================================
// SEQUENTIAL THINKING TESTS
// =============================================================================

#[tokio::test]
async fn test_sequential_thinking_basic() {
    let ctx = TestContext::new().await;

    let params = SequentialThinkingParams {
        problem: "Comment implémenter une API REST en Rust ?".to_string(),
        estimated_steps: 5,
        use_external: false,
    };

    let result = ctx.server.call_sequential_thinking(params).await.unwrap();

    assert!(!result.steps.is_empty(), "Should have thinking steps");
    assert!(!result.conclusion.is_empty(), "Should have a conclusion");
}

#[tokio::test]
async fn test_sequential_thinking_complex_problem() {
    let ctx = TestContext::new().await;

    let params = SequentialThinkingParams {
        problem: r#"
        Problème: Concevoir un système de cache distribué avec les contraintes suivantes:
        1. Haute disponibilité (99.9%)
        2. Cohérence éventuelle acceptable
        3. Latence < 10ms pour les lectures
        4. Support de 1M ops/sec
        5. TTL configurable par clé
        "#.to_string(),
        estimated_steps: 10,
        use_external: false,
    };

    let result = ctx.server.call_sequential_thinking(params).await.unwrap();

    assert!(result.steps.len() >= 3, "Complex problem should have multiple steps");
    println!("Thinking steps: {}", result.steps.len());
    println!("Conclusion: {}", result.conclusion);
}

#[tokio::test]
async fn test_sequential_thinking_with_external() {
    let ctx = TestContext::new().await;

    // Ce test essaie d'utiliser un serveur MCP externe (sequential-thinking)
    let params = SequentialThinkingParams {
        problem: "Quelle est la meilleure façon de gérer les erreurs en Rust ?".to_string(),
        estimated_steps: 3,
        use_external: true,
    };

    let result = ctx.server.call_sequential_thinking(params).await;

    // Peut échouer si le serveur externe n'est pas disponible
    match result {
        Ok(r) => {
            println!("External thinking result: {} steps", r.steps.len());
            assert!(!r.steps.is_empty());
        }
        Err(e) => {
            println!("External server not available (expected): {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_sequential_thinking_edge_cases() {
    let ctx = TestContext::new().await;

    // Problème très court
    let short_result = ctx.server.call_sequential_thinking(SequentialThinkingParams {
        problem: "1+1=?".to_string(),
        estimated_steps: 1,
        use_external: false,
    }).await.unwrap();

    assert!(!short_result.conclusion.is_empty());

    // Problème avec caractères spéciaux
    let special_result = ctx.server.call_sequential_thinking(SequentialThinkingParams {
        problem: "Comment gérer les émojis 🎉 et les caractères spéciaux <>&\"' ?".to_string(),
        estimated_steps: 2,
        use_external: false,
    }).await.unwrap();

    assert!(!special_result.conclusion.is_empty());
}

// =============================================================================
// EXTERNAL DOCS TESTS (Context7, MS Learn)
// =============================================================================

#[tokio::test]
async fn test_external_docs_context7() {
    let ctx = TestContext::new().await;

    let params = ExternalDocsParams {
        library: "tokio".to_string(),
        topic: Some("async runtime".to_string()),
        source: "context7".to_string(),
        max_tokens: 2000,
    };

    let result = ctx.server.call_external_docs(params).await;

    match result {
        Ok(docs) => {
            println!("Context7 docs retrieved: {} chars", docs.content.len());
            assert!(!docs.content.is_empty());
            assert_eq!(docs.source, "context7");
        }
        Err(e) => {
            // Context7 peut ne pas être disponible
            println!("Context7 not available: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_external_docs_mslearn() {
    let ctx = TestContext::new().await;

    let params = ExternalDocsParams {
        library: "Azure Blob Storage".to_string(),
        topic: Some("upload files".to_string()),
        source: "mslearn".to_string(),
        max_tokens: 3000,
    };

    let result = ctx.server.call_external_docs(params).await;

    match result {
        Ok(docs) => {
            println!("MS Learn docs retrieved: {} chars", docs.content.len());
            assert!(!docs.content.is_empty());
        }
        Err(e) => {
            println!("MS Learn not available: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_external_docs_auto_source() {
    let ctx = TestContext::new().await;

    // "auto" devrait choisir la source appropriée
    let params = ExternalDocsParams {
        library: "react".to_string(),
        topic: Some("hooks".to_string()),
        source: "auto".to_string(),
        max_tokens: 2000,
    };

    let result = ctx.server.call_external_docs(params).await;

    match result {
        Ok(docs) => {
            println!("Auto source selected: {}", docs.source);
            assert!(!docs.content.is_empty());
        }
        Err(e) => {
            println!("External docs not available: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_external_docs_various_libraries() {
    let ctx = TestContext::new().await;

    let libraries = vec![
        ("thiserror", "error handling"),
        ("serde", "serialization"),
        ("reqwest", "http client"),
        ("axum", "web framework"),
    ];

    for (lib, topic) in libraries {
        let params = ExternalDocsParams {
            library: lib.to_string(),
            topic: Some(topic.to_string()),
            source: "auto".to_string(),
            max_tokens: 1000,
        };

        let result = ctx.server.call_external_docs(params).await;

        match result {
            Ok(docs) => println!("{}: {} chars from {}", lib, docs.content.len(), docs.source),
            Err(e) => println!("{}: not available ({:?})", lib, e),
        }
    }
}

// =============================================================================
// EXTERNAL SEARCH TESTS (Tavily)
// =============================================================================

#[tokio::test]
async fn test_external_search_basic() {
    let ctx = TestContext::new().await;

    let params = ExternalSearchParams {
        query: "Rust error handling best practices 2024".to_string(),
        max_results: 5,
        search_type: "general".to_string(),
        include_domains: vec![],
        exclude_domains: vec![],
    };

    let result = ctx.server.call_external_search(params).await;

    match result {
        Ok(search) => {
            println!("Search returned {} results", search.results.len());
            for r in &search.results {
                println!("  - {}", r.title);
            }
            assert!(!search.results.is_empty());
        }
        Err(e) => {
            println!("Search not available: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_external_search_with_domain_filter() {
    let ctx = TestContext::new().await;

    let params = ExternalSearchParams {
        query: "Rust async await tutorial".to_string(),
        max_results: 5,
        search_type: "general".to_string(),
        include_domains: vec!["rust-lang.org".to_string(), "doc.rust-lang.org".to_string()],
        exclude_domains: vec![],
    };

    let result = ctx.server.call_external_search(params).await;

    match result {
        Ok(search) => {
            println!("Filtered search returned {} results", search.results.len());
            // Les résultats devraient venir des domaines spécifiés
        }
        Err(e) => {
            println!("Search not available: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_external_search_exclude_domains() {
    let ctx = TestContext::new().await;

    let params = ExternalSearchParams {
        query: "JavaScript frameworks comparison".to_string(),
        max_results: 10,
        search_type: "general".to_string(),
        include_domains: vec![],
        exclude_domains: vec!["w3schools.com".to_string()],
    };

    let result = ctx.server.call_external_search(params).await;

    match result {
        Ok(search) => {
            println!("Search with exclusion returned {} results", search.results.len());
        }
        Err(e) => {
            println!("Search not available: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_external_search_news_type() {
    let ctx = TestContext::new().await;

    let params = ExternalSearchParams {
        query: "Rust programming language news".to_string(),
        max_results: 5,
        search_type: "news".to_string(),
        include_domains: vec![],
        exclude_domains: vec![],
    };

    let result = ctx.server.call_external_search(params).await;

    match result {
        Ok(search) => {
            println!("News search returned {} results", search.results.len());
        }
        Err(e) => {
            println!("News search not available: {:?}", e);
        }
    }
}

// =============================================================================
// EXTERNAL MCP CALL TESTS
// =============================================================================

#[tokio::test]
async fn test_external_mcp_call_context7_resolve() {
    let ctx = TestContext::new().await;

    let params = ExternalMcpCallParams {
        server: "context7".to_string(),
        tool: "resolve-library-id".to_string(),
        arguments: Some(serde_json::json!({
            "libraryName": "tokio"
        })),
    };

    let result = ctx.server.call_external_mcp_call(params).await;

    match result {
        Ok(response) => {
            println!("Context7 resolve result: {:?}", response.result);
            assert!(response.success);
        }
        Err(e) => {
            println!("Context7 not available: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_external_mcp_call_context7_get_docs() {
    let ctx = TestContext::new().await;

    let params = ExternalMcpCallParams {
        server: "context7".to_string(),
        tool: "get-library-docs".to_string(),
        arguments: Some(serde_json::json!({
            "context7CompatibleLibraryID": "/tokio-rs/tokio",
            "topic": "spawn",
            "tokens": 1000
        })),
    };

    let result = ctx.server.call_external_mcp_call(params).await;

    match result {
        Ok(response) => {
            println!("Context7 get-docs success: {}", response.success);
        }
        Err(e) => {
            println!("Context7 not available: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_external_mcp_call_tavily_search() {
    let ctx = TestContext::new().await;

    let params = ExternalMcpCallParams {
        server: "tavily".to_string(),
        tool: "tavily-search".to_string(),
        arguments: Some(serde_json::json!({
            "query": "Rust MCP server implementation",
            "max_results": 3
        })),
    };

    let result = ctx.server.call_external_mcp_call(params).await;

    match result {
        Ok(response) => {
            println!("Tavily search success: {}", response.success);
        }
        Err(e) => {
            println!("Tavily not available: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_external_mcp_call_sequential_thinking() {
    let ctx = TestContext::new().await;

    let params = ExternalMcpCallParams {
        server: "sequential-thinking".to_string(),
        tool: "sequentialthinking".to_string(),
        arguments: Some(serde_json::json!({
            "thought": "Analyzing the problem of concurrent data access",
            "thoughtNumber": 1,
            "totalThoughts": 3,
            "nextThoughtNeeded": true
        })),
    };

    let result = ctx.server.call_external_mcp_call(params).await;

    match result {
        Ok(response) => {
            println!("Sequential thinking result: {:?}", response.result);
        }
        Err(e) => {
            println!("Sequential thinking server not available: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_external_mcp_call_invalid_server() {
    let ctx = TestContext::new().await;

    let params = ExternalMcpCallParams {
        server: "nonexistent-server".to_string(),
        tool: "some-tool".to_string(),
        arguments: None,
    };

    let result = ctx.server.call_external_mcp_call(params).await;

    // Devrait échouer car le serveur n'existe pas
    assert!(result.is_err());
}

#[tokio::test]
async fn test_external_mcp_call_invalid_tool() {
    let ctx = TestContext::new().await;

    let params = ExternalMcpCallParams {
        server: "context7".to_string(),
        tool: "nonexistent-tool".to_string(),
        arguments: None,
    };

    let result = ctx.server.call_external_mcp_call(params).await;

    // Peut échouer ou retourner une erreur selon l'implémentation
    match result {
        Ok(response) => assert!(!response.success),
        Err(_) => {} // Aussi acceptable
    }
}

// =============================================================================
// MCP STATUS TESTS
// =============================================================================

#[tokio::test]
async fn test_mcp_status() {
    let ctx = TestContext::new().await;

    let params = McpStatusParams {};

    let result = ctx.server.call_mcp_status(params).await.unwrap();

    println!("MCP Integrations Status:");
    for integration in &result.integrations {
        println!("  - {}: {} ({})",
                 integration.name,
                 if integration.available { "available" } else { "unavailable" },
                 integration.status);
    }
}

// =============================================================================
// INTEGRATION WORKFLOW TESTS
// =============================================================================

#[tokio::test]
async fn test_workflow_research_with_external_tools() {
    let ctx = TestContext::new().await;

    // Étape 1: Réflexion sur le problème
    let thinking = ctx.server.call_sequential_thinking(SequentialThinkingParams {
        problem: "Comment implémenter un rate limiter en Rust ?".to_string(),
        estimated_steps: 3,
        use_external: false,
    }).await.unwrap();

    println!("Step 1 - Thinking: {} steps", thinking.steps.len());

    // Étape 2: Recherche de documentation
    let docs_result = ctx.server.call_external_docs(ExternalDocsParams {
        library: "tokio".to_string(),
        topic: Some("rate limiting".to_string()),
        source: "auto".to_string(),
        max_tokens: 2000,
    }).await;

    match docs_result {
        Ok(docs) => println!("Step 2 - Docs: {} chars", docs.content.len()),
        Err(_) => println!("Step 2 - Docs: skipped (not available)"),
    }

    // Étape 3: Recherche web pour best practices
    let search_result = ctx.server.call_external_search(ExternalSearchParams {
        query: "Rust rate limiter implementation patterns".to_string(),
        max_results: 3,
        search_type: "general".to_string(),
        include_domains: vec![],
        exclude_domains: vec![],
    }).await;

    match search_result {
        Ok(search) => println!("Step 3 - Search: {} results", search.results.len()),
        Err(_) => println!("Step 3 - Search: skipped (not available)"),
    }

    // Étape 4: Stocker les résultats en mémoire
    ctx.server.call_memory_store(
        whytcard_intelligence::tools::MemoryStoreParams {
            content: format!("Research on rate limiting:\nThinking conclusion: {}", thinking.conclusion),
            title: Some("Rate Limiter Research".to_string()),
            tags: vec!["research".to_string(), "rate-limiter".to_string()],
            metadata: None,
            index: true,
            key: None,
        }
    ).await.unwrap();

    println!("Step 4 - Stored research results");
}

#[tokio::test]
async fn test_workflow_documentation_lookup() {
    let ctx = TestContext::new().await;

    let libraries = vec![
        ("serde", "derive macros"),
        ("thiserror", "custom errors"),
        ("anyhow", "error handling"),
    ];

    for (lib, topic) in libraries {
        let result = ctx.server.call_external_docs(ExternalDocsParams {
            library: lib.to_string(),
            topic: Some(topic.to_string()),
            source: "auto".to_string(),
            max_tokens: 1500,
        }).await;

        match result {
            Ok(docs) => {
                // Stocker en knowledge graph
                let _ = ctx.server.call_knowledge_add_entity(
                    whytcard_intelligence::tools::KnowledgeAddEntityParams {
                        name: lib.to_string(),
                        entity_type: "library".to_string(),
                        observations: vec![
                            format!("Topic: {}", topic),
                            format!("Source: {}", docs.source),
                        ],
                    }
                ).await;
            }
            Err(_) => {}
        }
    }

    // Vérifier le knowledge graph
    let graph = ctx.server.call_knowledge_read_graph(
        whytcard_intelligence::tools::KnowledgeReadGraphParams { limit: 10 }
    ).await.unwrap();

    println!("Knowledge graph has {} entities after doc lookup", graph.total_entities);
}
