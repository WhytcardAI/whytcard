//! Stress Tests for WhytCard Intelligence
//!
//! Tests de charge et de performance:
//! - Opérations parallèles massives
//! - Limites du système
//! - Comportement sous pression

mod common;

use common::{test_content, TestContext, wait_ms};
use std::time::Instant;
use whytcard_intelligence::tools::{
    BatchStoreParams, KnowledgeAddEntityParams, KnowledgeSearchParams,
    MemorySearchParams, MemoryStoreParams,
};

// =============================================================================
// MEMORY STRESS TESTS
// =============================================================================

#[tokio::test]
async fn stress_test_memory_store_sequential() {
    let ctx = TestContext::new().await;
    let count = 100;

    let start = Instant::now();

    for i in 0..count {
        let params = MemoryStoreParams {
            content: format!("Stress test content item {}", i),
            title: Some(format!("Item {}", i)),
            tags: vec!["stress".to_string()],
            metadata: None,
            index: true,
            key: None,
        };

        ctx.server.call_memory_store(params).await.unwrap();
    }

    let elapsed = start.elapsed();
    println!("Stored {} items sequentially in {:?}", count, elapsed);
    println!("Average: {:?} per item", elapsed / count);

    // Vérifier que tout est stocké
    let list = ctx.server.call_memory_list(
        whytcard_intelligence::tools::MemoryListParams {
            limit: count + 10,
            offset: 0,
            tags: vec!["stress".to_string()],
        }
    ).await.unwrap();

    assert!(list.memories.len() >= count as usize);
}

#[tokio::test]
async fn stress_test_memory_store_parallel() {
    let ctx = TestContext::new().await;
    let count = 50;

    let start = Instant::now();

    // Créer les futures
    let futures: Vec<_> = (0..count)
        .map(|i| {
            let server = ctx.server.clone();
            async move {
                let params = MemoryStoreParams {
                    content: format!("Parallel stress test content item {}", i),
                    title: Some(format!("Parallel Item {}", i)),
                    tags: vec!["stress-parallel".to_string()],
                    metadata: None,
                    index: false, // Disable indexing for parallel test
                    key: None,
                };
                server.call_memory_store(params).await
            }
        })
        .collect();

    // Exécuter en parallèle
    let results = futures::future::join_all(futures).await;

    let elapsed = start.elapsed();
    println!("Stored {} items in parallel in {:?}", count, elapsed);

    // Vérifier les résultats
    let success_count = results.iter().filter(|r| r.is_ok()).count();
    println!("Success: {}/{}", success_count, count);

    assert!(success_count as u32 >= count * 90 / 100, "At least 90% should succeed");
}

#[tokio::test]
async fn stress_test_batch_store_large() {
    let ctx = TestContext::new().await;
    let batch_size = 500;

    let items: Vec<_> = (0..batch_size)
        .map(|i| whytcard_intelligence::tools::BatchStoreItem {
            content: format!("Batch stress item {} with some additional content to make it larger", i),
            source: "stress-test".to_string(),
            category: "batch".to_string(),
            tags: vec!["stress-batch".to_string()],
            metadata: Some(serde_json::json!({"index": i})),
        })
        .collect();

    let params = BatchStoreParams { items };

    let start = Instant::now();
    let result = ctx.server.call_batch_store(params).await.unwrap();
    let elapsed = start.elapsed();

    println!("Batch stored {} items in {:?}", batch_size, elapsed);
    println!("Rate: {:.2} items/sec", batch_size as f64 / elapsed.as_secs_f64());

    assert_eq!(result.stored, batch_size);
    assert!(result.errors.is_empty());
}

#[tokio::test]
async fn stress_test_memory_search_under_load() {
    let ctx = TestContext::new().await;

    // Peupler avec des données
    for i in 0..50 {
        ctx.server.call_memory_store(MemoryStoreParams {
            content: format!("Searchable content about topic {} with keywords alpha beta gamma", i),
            title: None,
            tags: vec![],
            metadata: None,
            index: true,
            key: None,
        }).await.unwrap();
    }

    // Attendre l'indexation
    wait_ms(100).await;

    // Faire de nombreuses recherches
    let search_count = 20;
    let start = Instant::now();

    for i in 0..search_count {
        let query = match i % 3 {
            0 => "alpha",
            1 => "beta",
            _ => "gamma",
        };

        let _result = ctx.server.call_memory_search(MemorySearchParams {
            query: query.to_string(),
            limit: 10,
            min_score: None,
            tags: vec![],
        }).await.unwrap();
    }

    let elapsed = start.elapsed();
    println!("{} searches completed in {:?}", search_count, elapsed);
    println!("Average search time: {:?}", elapsed / search_count);
}

// =============================================================================
// KNOWLEDGE GRAPH STRESS TESTS
// =============================================================================

#[tokio::test]
async fn stress_test_knowledge_graph_build() {
    let ctx = TestContext::new().await;
    let entity_count = 100;

    let start = Instant::now();

    // Créer de nombreuses entités
    for i in 0..entity_count {
        ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
            name: format!("Entity_{}", i),
            entity_type: "stress_node".to_string(),
            observations: vec![
                format!("Observation 1 for entity {}", i),
                format!("Observation 2 for entity {}", i),
            ],
        }).await.unwrap();
    }

    let entity_elapsed = start.elapsed();
    println!("Created {} entities in {:?}", entity_count, entity_elapsed);

    // Créer des relations (graphe connecté)
    let relation_start = Instant::now();
    for i in 0..entity_count - 1 {
        ctx.server.call_knowledge_add_relation(
            whytcard_intelligence::tools::KnowledgeAddRelationParams {
                from: format!("Entity_{}", i),
                to: format!("Entity_{}", i + 1),
                relation_type: "next".to_string(),
            }
        ).await.unwrap();
    }

    let relation_elapsed = relation_start.elapsed();
    println!("Created {} relations in {:?}", entity_count - 1, relation_elapsed);

    // Test de lecture du graphe
    let read_start = Instant::now();
    let graph = ctx.server.call_knowledge_read_graph(
        whytcard_intelligence::tools::KnowledgeReadGraphParams { limit: 0 }
    ).await.unwrap();
    let read_elapsed = read_start.elapsed();

    println!("Read graph ({} entities, {} relations) in {:?}",
             graph.total_entities, graph.total_relations, read_elapsed);

    assert!(graph.total_entities >= entity_count as usize);
}

#[tokio::test]
async fn stress_test_knowledge_search_performance() {
    let ctx = TestContext::new().await;

    // Créer des entités avec des noms variés
    for i in 0..50 {
        ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
            name: format!("SearchableEntity_{}_{}", i % 10, i),
            entity_type: "searchable".to_string(),
            observations: vec![format!("Content for searchable entity {}", i)],
        }).await.unwrap();
    }

    // Mesurer les recherches
    let queries = vec!["Searchable", "Entity_5", "Entity_1", "nonexistent"];

    for query in queries {
        let start = Instant::now();
        let result = ctx.server.call_knowledge_search(KnowledgeSearchParams {
            query: query.to_string(),
            limit: 100,
        }).await.unwrap();
        let elapsed = start.elapsed();

        println!("Search '{}': {} results in {:?}", query, result.entities.len(), elapsed);
    }
}

#[tokio::test]
async fn stress_test_path_finding_deep_graph() {
    let ctx = TestContext::new().await;
    let depth = 20;

    // Créer une chaîne longue
    for i in 0..depth {
        ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
            name: format!("ChainNode_{}", i),
            entity_type: "chain".to_string(),
            observations: vec![],
        }).await.unwrap();

        if i > 0 {
            ctx.server.call_knowledge_add_relation(
                whytcard_intelligence::tools::KnowledgeAddRelationParams {
                    from: format!("ChainNode_{}", i - 1),
                    to: format!("ChainNode_{}", i),
                    relation_type: "next".to_string(),
                }
            ).await.unwrap();
        }
    }

    // Test path finding sur différentes distances
    let test_cases = vec![
        (0, 5, 5),
        (0, 10, 10),
        (0, depth - 1, depth - 1),
    ];

    for (from, to, expected_length) in test_cases {
        let start = Instant::now();
        let result = ctx.server.call_knowledge_find_path(
            whytcard_intelligence::tools::KnowledgeFindPathParams {
                from: format!("ChainNode_{}", from),
                to: format!("ChainNode_{}", to),
                max_depth: depth,
            }
        ).await.unwrap();
        let elapsed = start.elapsed();

        println!("Path {} -> {}: length {} in {:?}", from, to, result.length, elapsed);
        assert!(result.found);
        assert_eq!(result.length, expected_length);
    }
}

// =============================================================================
// RAG STRESS TESTS
// =============================================================================

#[tokio::test]
async fn stress_test_rag_indexing_large_documents() {
    let ctx = TestContext::new().await;

    // Indexer des documents de différentes tailles
    let sizes = vec![
        ("small", 100),
        ("medium", 1000),
        ("large", 10000),
        ("xlarge", 50000),
    ];

    for (label, size) in sizes {
        let content = "word ".repeat(size);

        let start = Instant::now();
        let result = ctx.server.call_memory_store(MemoryStoreParams {
            content,
            title: Some(format!("{} document", label)),
            tags: vec!["size-test".to_string()],
            metadata: None,
            index: true,
            key: None,
        }).await;
        let elapsed = start.elapsed();

        match result {
            Ok(_) => println!("{} document ({} words) indexed in {:?}", label, size, elapsed),
            Err(e) => println!("{} document failed: {:?}", label, e),
        }
    }
}

#[tokio::test]
async fn stress_test_hybrid_search_performance() {
    let ctx = TestContext::new().await;

    // Peupler toutes les sources de mémoire
    // Semantic
    for i in 0..20 {
        ctx.server.call_memory_store(MemoryStoreParams {
            content: format!("Semantic content about machine learning topic {}", i),
            title: None,
            tags: vec![],
            metadata: None,
            index: true,
            key: None,
        }).await.unwrap();
    }

    // Knowledge
    for i in 0..10 {
        ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
            name: format!("MLConcept_{}", i),
            entity_type: "concept".to_string(),
            observations: vec![format!("Machine learning concept {}", i)],
        }).await.unwrap();
    }

    wait_ms(100).await;

    // Mesurer hybrid search
    let iterations = 10;
    let start = Instant::now();

    for _ in 0..iterations {
        let _result = ctx.server.call_hybrid_search(
            whytcard_intelligence::tools::HybridSearchParams {
                query: "machine learning".to_string(),
                top_k: 10,
                min_relevance: 0.3,
            }
        ).await.unwrap();
    }

    let elapsed = start.elapsed();
    println!("{} hybrid searches in {:?}", iterations, elapsed);
    println!("Average: {:?} per search", elapsed / iterations);
}

// =============================================================================
// CORTEX STRESS TESTS
// =============================================================================

#[tokio::test]
async fn stress_test_cortex_process_sequential() {
    let ctx = TestContext::new().await;
    let count = 10;

    let start = Instant::now();

    for i in 0..count {
        let _result = ctx.server.call_cortex_process(
            whytcard_intelligence::tools::CortexProcessParams {
                query: format!("Process query number {}", i),
                context: None,
                session_id: None,
                auto_learn: false,
                inject_doubt: false,
                language: None,
                task_type: None,
            }
        ).await.unwrap();
    }

    let elapsed = start.elapsed();
    println!("{} CORTEX processes in {:?}", count, elapsed);
    println!("Average: {:?} per process", elapsed / count);
}

#[tokio::test]
async fn stress_test_cortex_execute_commands() {
    let ctx = TestContext::new().await;
    let count = 20;

    let start = Instant::now();

    for i in 0..count {
        let _result = ctx.server.call_cortex_execute(
            whytcard_intelligence::tools::CortexExecuteParams {
                command: format!("echo 'Command {}'", i),
                cwd: None,
                env: None,
                timeout_secs: 5,
                separate_stderr: false,
            }
        ).await.unwrap();
    }

    let elapsed = start.elapsed();
    println!("{} shell commands in {:?}", count, elapsed);
    println!("Average: {:?} per command", elapsed / count);
}

// =============================================================================
// CONCURRENCY STRESS TESTS
// =============================================================================

#[tokio::test]
async fn stress_test_concurrent_mixed_operations() {
    let ctx = TestContext::new().await;

    let iterations = 20;
    let start = Instant::now();

    // Créer des opérations mixtes en parallèle
    let mut handles = vec![];

    for i in 0..iterations {
        let server = ctx.server.clone();
        let handle = tokio::spawn(async move {
            match i % 4 {
                0 => {
                    // Memory store
                    let _ = server.call_memory_store(MemoryStoreParams {
                        content: format!("Concurrent content {}", i),
                        title: None,
                        tags: vec![],
                        metadata: None,
                        index: false,
                        key: None,
                    }).await;
                }
                1 => {
                    // Knowledge entity
                    let _ = server.call_knowledge_add_entity(KnowledgeAddEntityParams {
                        name: format!("ConcurrentEntity_{}", i),
                        entity_type: "concurrent".to_string(),
                        observations: vec![],
                    }).await;
                }
                2 => {
                    // Memory search
                    let _ = server.call_memory_search(MemorySearchParams {
                        query: "concurrent".to_string(),
                        limit: 5,
                        min_score: None,
                        tags: vec![],
                    }).await;
                }
                _ => {
                    // Knowledge search
                    let _ = server.call_knowledge_search(KnowledgeSearchParams {
                        query: "Concurrent".to_string(),
                        limit: 5,
                    }).await;
                }
            }
        });
        handles.push(handle);
    }

    // Attendre tous les handles
    for handle in handles {
        let _ = handle.await;
    }

    let elapsed = start.elapsed();
    println!("{} concurrent mixed operations in {:?}", iterations, elapsed);
}

// =============================================================================
// CLEANUP & RECOVERY TESTS
// =============================================================================

#[tokio::test]
async fn stress_test_create_delete_cycle() {
    let ctx = TestContext::new().await;
    let cycles = 10;
    let items_per_cycle = 20;

    let start = Instant::now();

    for cycle in 0..cycles {
        // Create phase
        let mut keys = vec![];
        for i in 0..items_per_cycle {
            let result = ctx.server.call_memory_store(MemoryStoreParams {
                content: format!("Cycle {} item {}", cycle, i),
                title: None,
                tags: vec![],
                metadata: None,
                index: false,
                key: None,
            }).await.unwrap();
            keys.push(result.key);
        }

        // Delete phase
        for key in keys {
            ctx.server.call_memory_delete(
                whytcard_intelligence::tools::MemoryDeleteParams { key }
            ).await.unwrap();
        }
    }

    let elapsed = start.elapsed();
    println!("{} create/delete cycles ({} items each) in {:?}",
             cycles, items_per_cycle, elapsed);
}

// =============================================================================
// LIMITES DU SYSTÈME
// =============================================================================

#[tokio::test]
async fn stress_test_max_content_size() {
    let ctx = TestContext::new().await;

    // Tester différentes tailles pour trouver les limites
    let sizes_mb = vec![0.1, 0.5, 1.0, 2.0];

    for size_mb in sizes_mb {
        let size_bytes = (size_mb * 1024.0 * 1024.0) as usize;
        let content = "x".repeat(size_bytes);

        let start = Instant::now();
        let result = ctx.server.call_memory_store(MemoryStoreParams {
            content,
            title: Some(format!("{} MB document", size_mb)),
            tags: vec![],
            metadata: None,
            index: false, // Skip indexing for size test
            key: None,
        }).await;
        let elapsed = start.elapsed();

        match result {
            Ok(_) => println!("{} MB: OK in {:?}", size_mb, elapsed),
            Err(e) => println!("{} MB: FAILED - {:?}", size_mb, e),
        }
    }
}

#[tokio::test]
async fn stress_test_max_tags() {
    let ctx = TestContext::new().await;

    let tag_counts = vec![10, 50, 100, 500];

    for count in tag_counts {
        let tags: Vec<String> = (0..count).map(|i| format!("tag_{}", i)).collect();

        let result = ctx.server.call_memory_store(MemoryStoreParams {
            content: "Content with many tags".to_string(),
            title: None,
            tags,
            metadata: None,
            index: false,
            key: None,
        }).await;

        match result {
            Ok(_) => println!("{} tags: OK", count),
            Err(e) => println!("{} tags: FAILED - {:?}", count, e),
        }
    }
}

#[tokio::test]
async fn stress_test_max_observations() {
    let ctx = TestContext::new().await;

    let obs_counts = vec![10, 50, 100, 200];

    for count in obs_counts {
        let observations: Vec<String> = (0..count)
            .map(|i| format!("Observation number {} with some text content", i))
            .collect();

        let result = ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
            name: format!("EntityWith{}Obs", count),
            entity_type: "test".to_string(),
            observations,
        }).await;

        match result {
            Ok(_) => println!("{} observations: OK", count),
            Err(e) => println!("{} observations: FAILED - {:?}", count, e),
        }
    }
}
