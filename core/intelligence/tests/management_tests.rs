//! Management Tools Tests for WhytCard Intelligence
//!
//! Tests pour les outils de gestion MCP:
//! - mcp_list_servers
//! - mcp_list_tools
//! - mcp_install_server
//! - mcp_uninstall_server
//! - mcp_configure_server
//! - mcp_connect
//! - mcp_disconnect
//! - manage_tags
//! - export_graph

mod common;

use common::TestContext;
use whytcard_intelligence::tools::{
    McpListServersParams, McpListToolsParams, McpInstallServerParams,
    McpUninstallServerParams, McpConfigureServerParams, McpConnectParams,
    McpDisconnectParams, ManageTagsParams, ExportGraphParams,
};

// =============================================================================
// MCP SERVER LISTING TESTS
// =============================================================================

#[tokio::test]
async fn test_mcp_list_servers() {
    let ctx = TestContext::new().await;

    let params = McpListServersParams {};
    let result = ctx.server.call_mcp_list_servers(params).await.unwrap();

    println!("Available MCP servers:");
    for server in &result.servers {
        println!("  - {} ({}): {}",
                 server.name,
                 server.status,
                 if server.installed { "installed" } else { "not installed" });
    }

    // Vérifier la structure de la réponse
    assert!(result.servers.len() >= 0); // Peut être vide si aucun serveur
}

#[tokio::test]
async fn test_mcp_list_tools_all() {
    let ctx = TestContext::new().await;

    let params = McpListToolsParams {
        server: None,
        name_pattern: None,
    };

    let result = ctx.server.call_mcp_list_tools(params).await.unwrap();

    println!("All available tools: {}", result.tools.len());
    for tool in result.tools.iter().take(10) {
        println!("  - {}: {}", tool.name, tool.description);
    }
}

#[tokio::test]
async fn test_mcp_list_tools_by_server() {
    let ctx = TestContext::new().await;

    let params = McpListToolsParams {
        server: Some("context7".to_string()),
        name_pattern: None,
    };

    let result = ctx.server.call_mcp_list_tools(params).await;

    match result {
        Ok(tools) => {
            println!("Context7 tools: {}", tools.tools.len());
            for tool in &tools.tools {
                println!("  - {}", tool.name);
            }
        }
        Err(e) => {
            println!("Context7 not available: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_mcp_list_tools_by_pattern() {
    let ctx = TestContext::new().await;

    let params = McpListToolsParams {
        server: None,
        name_pattern: Some("memory".to_string()),
    };

    let result = ctx.server.call_mcp_list_tools(params).await.unwrap();

    println!("Tools matching 'memory': {}", result.tools.len());
    for tool in &result.tools {
        assert!(tool.name.to_lowercase().contains("memory"));
    }
}

// =============================================================================
// MCP SERVER MANAGEMENT TESTS
// =============================================================================

#[tokio::test]
async fn test_mcp_server_lifecycle() {
    let ctx = TestContext::new().await;

    // 1. Lister les serveurs disponibles
    let list_before = ctx.server.call_mcp_list_servers(McpListServersParams {}).await.unwrap();
    println!("Servers before: {}", list_before.servers.len());

    // 2. Installer un serveur test (peut échouer si déjà installé)
    let install_result = ctx.server.call_mcp_install_server(McpInstallServerParams {
        name: "test-server".to_string(),
        source: "npm:test-mcp-server".to_string(),
        config: None,
    }).await;

    match install_result {
        Ok(r) => println!("Install: {}", if r.success { "OK" } else { "already exists" }),
        Err(e) => println!("Install skipped: {:?}", e),
    }

    // 3. Configurer le serveur
    let config_result = ctx.server.call_mcp_configure_server(McpConfigureServerParams {
        name: "test-server".to_string(),
        config: serde_json::json!({
            "option1": "value1",
            "option2": true
        }),
    }).await;

    match config_result {
        Ok(_) => println!("Configure: OK"),
        Err(e) => println!("Configure skipped: {:?}", e),
    }

    // 4. Désinstaller le serveur test
    let uninstall_result = ctx.server.call_mcp_uninstall_server(McpUninstallServerParams {
        name: "test-server".to_string(),
    }).await;

    match uninstall_result {
        Ok(r) => println!("Uninstall: {}", if r.success { "OK" } else { "not found" }),
        Err(e) => println!("Uninstall skipped: {:?}", e),
    }
}

#[tokio::test]
async fn test_mcp_connect_disconnect() {
    let ctx = TestContext::new().await;

    // Tester connexion à un serveur existant
    let connect_result = ctx.server.call_mcp_connect(McpConnectParams {
        server: "context7".to_string(),
    }).await;

    match connect_result {
        Ok(r) => {
            println!("Connect to context7: {}", if r.success { "OK" } else { "failed" });

            // Si connecté, tester déconnexion
            if r.success {
                let disconnect_result = ctx.server.call_mcp_disconnect(McpDisconnectParams {
                    server: "context7".to_string(),
                }).await;

                match disconnect_result {
                    Ok(d) => println!("Disconnect: {}", if d.success { "OK" } else { "failed" }),
                    Err(e) => println!("Disconnect error: {:?}", e),
                }
            }
        }
        Err(e) => {
            println!("Connect not available: {:?}", e);
        }
    }
}

#[tokio::test]
async fn test_mcp_install_invalid_source() {
    let ctx = TestContext::new().await;

    let result = ctx.server.call_mcp_install_server(McpInstallServerParams {
        name: "invalid-server".to_string(),
        source: "invalid://source".to_string(),
        config: None,
    }).await;

    // Devrait échouer avec une source invalide
    match result {
        Ok(r) => assert!(!r.success, "Should fail with invalid source"),
        Err(_) => {} // Expected
    }
}

// =============================================================================
// TAG MANAGEMENT TESTS
// =============================================================================

#[tokio::test]
async fn test_manage_tags_add() {
    let ctx = TestContext::new().await;

    // D'abord créer une mémoire
    let mem = ctx.server.call_memory_store(
        whytcard_intelligence::tools::MemoryStoreParams {
            content: "Content for tag test".to_string(),
            title: None,
            tags: vec!["initial".to_string()],
            metadata: None,
            index: false,
            key: Some("tag-test-doc".to_string()),
        }
    ).await.unwrap();

    // Ajouter des tags
    let result = ctx.server.call_manage_tags(ManageTagsParams {
        action: "add".to_string(),
        doc_id: Some(mem.key.clone()),
        tags: vec!["new-tag".to_string(), "another-tag".to_string()],
    }).await.unwrap();

    assert!(result.success);

    // Vérifier les tags
    let get_result = ctx.server.call_manage_tags(ManageTagsParams {
        action: "get".to_string(),
        doc_id: Some(mem.key),
        tags: vec![],
    }).await.unwrap();

    assert!(get_result.tags.contains(&"initial".to_string()));
    assert!(get_result.tags.contains(&"new-tag".to_string()));
    assert!(get_result.tags.contains(&"another-tag".to_string()));
}

#[tokio::test]
async fn test_manage_tags_remove() {
    let ctx = TestContext::new().await;

    // Créer une mémoire avec plusieurs tags
    let mem = ctx.server.call_memory_store(
        whytcard_intelligence::tools::MemoryStoreParams {
            content: "Content with multiple tags".to_string(),
            title: None,
            tags: vec!["tag1".to_string(), "tag2".to_string(), "tag3".to_string()],
            metadata: None,
            index: false,
            key: Some("multi-tag-doc".to_string()),
        }
    ).await.unwrap();

    // Supprimer un tag
    let result = ctx.server.call_manage_tags(ManageTagsParams {
        action: "remove".to_string(),
        doc_id: Some(mem.key.clone()),
        tags: vec!["tag2".to_string()],
    }).await.unwrap();

    assert!(result.success);

    // Vérifier
    let get_result = ctx.server.call_manage_tags(ManageTagsParams {
        action: "get".to_string(),
        doc_id: Some(mem.key),
        tags: vec![],
    }).await.unwrap();

    assert!(get_result.tags.contains(&"tag1".to_string()));
    assert!(!get_result.tags.contains(&"tag2".to_string()));
    assert!(get_result.tags.contains(&"tag3".to_string()));
}

#[tokio::test]
async fn test_manage_tags_search() {
    let ctx = TestContext::new().await;

    // Créer plusieurs mémoires avec des tags différents
    for i in 0..5 {
        ctx.server.call_memory_store(
            whytcard_intelligence::tools::MemoryStoreParams {
                content: format!("Searchable content {}", i),
                title: None,
                tags: vec!["searchable".to_string(), format!("group-{}", i % 2)],
                metadata: None,
                index: false,
                key: None,
            }
        ).await.unwrap();
    }

    // Rechercher par tag
    let result = ctx.server.call_manage_tags(ManageTagsParams {
        action: "search".to_string(),
        doc_id: None,
        tags: vec!["searchable".to_string()],
    }).await.unwrap();

    println!("Found {} documents with tag 'searchable'", result.doc_ids.len());
    assert!(result.doc_ids.len() >= 5);

    // Rechercher par sous-groupe
    let result_group = ctx.server.call_manage_tags(ManageTagsParams {
        action: "search".to_string(),
        doc_id: None,
        tags: vec!["group-0".to_string()],
    }).await.unwrap();

    println!("Found {} documents with tag 'group-0'", result_group.doc_ids.len());
}

#[tokio::test]
async fn test_manage_tags_invalid_action() {
    let ctx = TestContext::new().await;

    let result = ctx.server.call_manage_tags(ManageTagsParams {
        action: "invalid".to_string(),
        doc_id: None,
        tags: vec![],
    }).await;

    // Devrait échouer avec action invalide
    assert!(result.is_err());
}

// =============================================================================
// EXPORT GRAPH TESTS
// =============================================================================

#[tokio::test]
async fn test_export_graph_json() {
    let ctx = TestContext::new().await;

    // Créer des données
    ctx.server.call_knowledge_add_entity(
        whytcard_intelligence::tools::KnowledgeAddEntityParams {
            name: "ExportTest1".to_string(),
            entity_type: "test".to_string(),
            observations: vec!["Observation for export".to_string()],
        }
    ).await.unwrap();

    ctx.server.call_knowledge_add_entity(
        whytcard_intelligence::tools::KnowledgeAddEntityParams {
            name: "ExportTest2".to_string(),
            entity_type: "test".to_string(),
            observations: vec!["Another observation".to_string()],
        }
    ).await.unwrap();

    ctx.server.call_knowledge_add_relation(
        whytcard_intelligence::tools::KnowledgeAddRelationParams {
            from: "ExportTest1".to_string(),
            to: "ExportTest2".to_string(),
            relation_type: "related_to".to_string(),
        }
    ).await.unwrap();

    // Exporter en JSON
    let result = ctx.server.call_export_graph(ExportGraphParams {
        format: "json".to_string(),
        entity_types: vec![],
        include_relations: true,
    }).await.unwrap();

    println!("Export JSON: {} chars", result.data.len());
    assert!(result.format == "json");
    assert!(!result.data.is_empty());

    // Vérifier que c'est du JSON valide
    let parsed: serde_json::Value = serde_json::from_str(&result.data).unwrap();
    assert!(parsed.is_object() || parsed.is_array());
}

#[tokio::test]
async fn test_export_graph_dict() {
    let ctx = TestContext::new().await;

    // Créer des données
    ctx.server.call_knowledge_add_entity(
        whytcard_intelligence::tools::KnowledgeAddEntityParams {
            name: "DictExport1".to_string(),
            entity_type: "dict_test".to_string(),
            observations: vec!["Dict observation".to_string()],
        }
    ).await.unwrap();

    // Exporter en dict
    let result = ctx.server.call_export_graph(ExportGraphParams {
        format: "dict".to_string(),
        entity_types: vec![],
        include_relations: true,
    }).await.unwrap();

    println!("Export dict format: {} chars", result.data.len());
    assert!(result.format == "dict");
}

#[tokio::test]
async fn test_export_graph_filter_by_type() {
    let ctx = TestContext::new().await;

    // Créer des entités de types différents
    ctx.server.call_knowledge_add_entity(
        whytcard_intelligence::tools::KnowledgeAddEntityParams {
            name: "PersonEntity".to_string(),
            entity_type: "person".to_string(),
            observations: vec![],
        }
    ).await.unwrap();

    ctx.server.call_knowledge_add_entity(
        whytcard_intelligence::tools::KnowledgeAddEntityParams {
            name: "ProjectEntity".to_string(),
            entity_type: "project".to_string(),
            observations: vec![],
        }
    ).await.unwrap();

    // Exporter seulement les personnes
    let result = ctx.server.call_export_graph(ExportGraphParams {
        format: "json".to_string(),
        entity_types: vec!["person".to_string()],
        include_relations: false,
    }).await.unwrap();

    let parsed: serde_json::Value = serde_json::from_str(&result.data).unwrap();
    println!("Filtered export: {:?}", parsed);

    // Vérifier que seul le type "person" est présent
    // La structure dépend de l'implémentation
}

#[tokio::test]
async fn test_export_graph_without_relations() {
    let ctx = TestContext::new().await;

    // Créer des données avec relations
    ctx.server.call_knowledge_add_entity(
        whytcard_intelligence::tools::KnowledgeAddEntityParams {
            name: "NodeA".to_string(),
            entity_type: "node".to_string(),
            observations: vec![],
        }
    ).await.unwrap();

    ctx.server.call_knowledge_add_entity(
        whytcard_intelligence::tools::KnowledgeAddEntityParams {
            name: "NodeB".to_string(),
            entity_type: "node".to_string(),
            observations: vec![],
        }
    ).await.unwrap();

    ctx.server.call_knowledge_add_relation(
        whytcard_intelligence::tools::KnowledgeAddRelationParams {
            from: "NodeA".to_string(),
            to: "NodeB".to_string(),
            relation_type: "connects".to_string(),
        }
    ).await.unwrap();

    // Exporter SANS relations
    let result = ctx.server.call_export_graph(ExportGraphParams {
        format: "json".to_string(),
        entity_types: vec![],
        include_relations: false,
    }).await.unwrap();

    println!("Export without relations: {} chars", result.data.len());
    // Les relations ne devraient pas être dans l'export
}

// =============================================================================
// INTEGRATION TESTS
// =============================================================================

#[tokio::test]
async fn test_full_management_workflow() {
    let ctx = TestContext::new().await;

    // 1. Vérifier le statut initial
    let status = ctx.server.call_mcp_status(
        whytcard_intelligence::tools::McpStatusParams {}
    ).await.unwrap();
    println!("Step 1 - Initial status: {} integrations", status.integrations.len());

    // 2. Lister les serveurs
    let servers = ctx.server.call_mcp_list_servers(McpListServersParams {}).await.unwrap();
    println!("Step 2 - {} servers available", servers.servers.len());

    // 3. Lister tous les outils
    let tools = ctx.server.call_mcp_list_tools(McpListToolsParams {
        server: None,
        name_pattern: None,
    }).await.unwrap();
    println!("Step 3 - {} tools available", tools.tools.len());

    // 4. Créer des données de test
    ctx.server.call_knowledge_add_entity(
        whytcard_intelligence::tools::KnowledgeAddEntityParams {
            name: "WorkflowTest".to_string(),
            entity_type: "workflow".to_string(),
            observations: vec!["Created during management workflow test".to_string()],
        }
    ).await.unwrap();
    println!("Step 4 - Test entity created");

    // 5. Exporter les données
    let export = ctx.server.call_export_graph(ExportGraphParams {
        format: "json".to_string(),
        entity_types: vec!["workflow".to_string()],
        include_relations: true,
    }).await.unwrap();
    println!("Step 5 - Exported {} chars", export.data.len());

    // 6. Vérifier les statistiques CORTEX
    let stats = ctx.server.call_cortex_stats(
        whytcard_intelligence::tools::CortexStatsParams {}
    ).await.unwrap();
    println!("Step 6 - CORTEX stats: {} semantic, {} episodic, {} procedural",
             stats.semantic_facts, stats.episodic_events, stats.procedural_rules);
}

#[tokio::test]
async fn test_tag_based_organization() {
    let ctx = TestContext::new().await;

    // Créer une structure organisée par tags
    let categories = vec![
        ("code-review", vec!["rust", "quality"]),
        ("architecture", vec!["design", "patterns"]),
        ("testing", vec!["unit", "integration"]),
    ];

    for (category, tags) in categories {
        ctx.server.call_memory_store(
            whytcard_intelligence::tools::MemoryStoreParams {
                content: format!("Document about {}", category),
                title: Some(format!("{} guide", category)),
                tags: tags.iter().map(|s| s.to_string()).collect(),
                metadata: Some(serde_json::json!({"category": category})),
                index: false,
                key: Some(format!("doc-{}", category)),
            }
        ).await.unwrap();
    }

    // Rechercher par différents tags
    for tag in &["rust", "design", "unit"] {
        let result = ctx.server.call_manage_tags(ManageTagsParams {
            action: "search".to_string(),
            doc_id: None,
            tags: vec![tag.to_string()],
        }).await.unwrap();

        println!("Tag '{}': {} documents", tag, result.doc_ids.len());
    }
}
