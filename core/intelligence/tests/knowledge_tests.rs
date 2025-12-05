//! Knowledge Graph Integration Tests
//!
//! Tests all knowledge graph operations:
//! - Entity CRUD (add, get, delete)
//! - Observations management
//! - Relations (add, delete, find path)
//! - Graph traversal (neighbors, export)

mod common;

use common::TestContext;
use whytcard_intelligence::tools::{
    ExportGraphParams, KnowledgeAddEntityParams, KnowledgeAddObservationParams,
    KnowledgeAddRelationParams, KnowledgeDeleteEntityParams, KnowledgeDeleteObservationParams,
    KnowledgeDeleteRelationParams, KnowledgeFindPathParams, KnowledgeGetEntityParams,
    KnowledgeGetNeighborsParams, KnowledgeReadGraphParams, KnowledgeSearchParams,
};

// =============================================================================
// ENTITY LIFECYCLE TESTS
// =============================================================================

#[tokio::test]
async fn test_entity_full_lifecycle() {
    let ctx = TestContext::new().await;

    // 1. Create entity
    let create_params = KnowledgeAddEntityParams {
        name: "Rust".to_string(),
        entity_type: "programming_language".to_string(),
        observations: vec![
            "Systems programming language".to_string(),
            "Memory safe without garbage collector".to_string(),
        ],
    };

    let create_result = ctx.server.call_knowledge_add_entity(create_params).await;
    assert!(create_result.is_ok());
    let created = create_result.unwrap();
    assert!(created.created);
    assert_eq!(created.observations_added, 2);

    // 2. Get entity
    let get_params = KnowledgeGetEntityParams {
        name: "Rust".to_string(),
        include_relations: true,
    };

    let get_result = ctx.server.call_knowledge_get_entity(get_params).await;
    assert!(get_result.is_ok());
    let entity = get_result.unwrap();
    assert_eq!(entity.entity.name, "Rust");
    assert_eq!(entity.entity.entity_type, "programming_language");
    assert_eq!(entity.entity.observations.len(), 2);

    // 3. Add observation
    let obs_params = KnowledgeAddObservationParams {
        entity_name: "Rust".to_string(),
        observations: vec!["Created by Mozilla".to_string()],
    };

    let obs_result = ctx.server.call_knowledge_add_observation(obs_params).await;
    assert!(obs_result.is_ok());
    assert_eq!(obs_result.unwrap().added, 1);

    // 4. Verify observation added
    let get_result = ctx.server.call_knowledge_get_entity(KnowledgeGetEntityParams {
        name: "Rust".to_string(),
        include_relations: false,
    }).await.unwrap();
    assert_eq!(get_result.entity.observations.len(), 3);

    // 5. Delete entity
    let delete_params = KnowledgeDeleteEntityParams {
        names: vec!["Rust".to_string()],
    };

    let delete_result = ctx.server.call_knowledge_delete_entity(delete_params).await;
    assert!(delete_result.is_ok());
    assert!(delete_result.unwrap().deleted.contains(&"Rust".to_string()));

    // 6. Verify deleted
    let get_result = ctx.server.call_knowledge_get_entity(KnowledgeGetEntityParams {
        name: "Rust".to_string(),
        include_relations: false,
    }).await;
    assert!(get_result.is_err());
}

#[tokio::test]
async fn test_entity_duplicate_handling() {
    let ctx = TestContext::new().await;

    // Create entity
    let params = KnowledgeAddEntityParams {
        name: "Python".to_string(),
        entity_type: "language".to_string(),
        observations: vec!["Interpreted language".to_string()],
    };
    ctx.server.call_knowledge_add_entity(params.clone()).await.unwrap();

    // Create again - should add observations, not create new
    let result = ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "Python".to_string(),
        entity_type: "language".to_string(),
        observations: vec!["Dynamic typing".to_string()],
    }).await.unwrap();

    assert!(!result.created); // Not created because exists
    assert_eq!(result.observations_added, 1); // But observation added

    // Verify total observations
    let entity = ctx.server.call_knowledge_get_entity(KnowledgeGetEntityParams {
        name: "Python".to_string(),
        include_relations: false,
    }).await.unwrap();

    assert_eq!(entity.entity.observations.len(), 2);
}

// =============================================================================
// RELATIONS TESTS
// =============================================================================

#[tokio::test]
async fn test_relation_create_and_query() {
    let ctx = TestContext::new().await;

    // Create entities
    ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "TypeScript".to_string(),
        entity_type: "language".to_string(),
        observations: vec!["Typed superset of JavaScript".to_string()],
    }).await.unwrap();

    ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "JavaScript".to_string(),
        entity_type: "language".to_string(),
        observations: vec!["Web scripting language".to_string()],
    }).await.unwrap();

    // Create relation
    let rel_params = KnowledgeAddRelationParams {
        from: "TypeScript".to_string(),
        to: "JavaScript".to_string(),
        relation_type: "extends".to_string(),
    };

    let result = ctx.server.call_knowledge_add_relation(rel_params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().created);

    // Query with relations
    let ts = ctx.server.call_knowledge_get_entity(KnowledgeGetEntityParams {
        name: "TypeScript".to_string(),
        include_relations: true,
    }).await.unwrap();

    assert!(!ts.outgoing.is_empty());
    assert_eq!(ts.outgoing[0].to, "JavaScript");
    assert_eq!(ts.outgoing[0].relation_type, "extends");

    // Query reverse
    let js = ctx.server.call_knowledge_get_entity(KnowledgeGetEntityParams {
        name: "JavaScript".to_string(),
        include_relations: true,
    }).await.unwrap();

    assert!(!js.incoming.is_empty());
    assert_eq!(js.incoming[0].from, "TypeScript");
}

#[tokio::test]
async fn test_relation_delete() {
    let ctx = TestContext::new().await;

    // Setup
    ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "A".to_string(),
        entity_type: "test".to_string(),
        observations: vec![],
    }).await.unwrap();

    ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "B".to_string(),
        entity_type: "test".to_string(),
        observations: vec![],
    }).await.unwrap();

    ctx.server.call_knowledge_add_relation(KnowledgeAddRelationParams {
        from: "A".to_string(),
        to: "B".to_string(),
        relation_type: "links_to".to_string(),
    }).await.unwrap();

    // Delete relation
    let delete_params = KnowledgeDeleteRelationParams {
        relations: vec![
            whytcard_intelligence::tools::RelationSpec {
                from: "A".to_string(),
                to: "B".to_string(),
                relation_type: "links_to".to_string(),
            },
        ],
    };

    let result = ctx.server.call_knowledge_delete_relation(delete_params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().deleted > 0);

    // Verify deleted
    let a = ctx.server.call_knowledge_get_entity(KnowledgeGetEntityParams {
        name: "A".to_string(),
        include_relations: true,
    }).await.unwrap();

    assert!(a.outgoing.is_empty());
}

#[tokio::test]
async fn test_relation_cascade_on_entity_delete() {
    let ctx = TestContext::new().await;

    // Create chain: A -> B -> C
    for name in ["A", "B", "C"] {
        ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
            name: name.to_string(),
            entity_type: "node".to_string(),
            observations: vec![],
        }).await.unwrap();
    }

    ctx.server.call_knowledge_add_relation(KnowledgeAddRelationParams {
        from: "A".to_string(),
        to: "B".to_string(),
        relation_type: "connects".to_string(),
    }).await.unwrap();

    ctx.server.call_knowledge_add_relation(KnowledgeAddRelationParams {
        from: "B".to_string(),
        to: "C".to_string(),
        relation_type: "connects".to_string(),
    }).await.unwrap();

    // Delete B (middle node)
    let result = ctx.server.call_knowledge_delete_entity(KnowledgeDeleteEntityParams {
        names: vec!["B".to_string()],
    }).await.unwrap();

    // Should have removed relations
    assert!(result.relations_removed >= 2);

    // A should have no outgoing relations now
    let a = ctx.server.call_knowledge_get_entity(KnowledgeGetEntityParams {
        name: "A".to_string(),
        include_relations: true,
    }).await.unwrap();
    assert!(a.outgoing.is_empty());
}

// =============================================================================
// SEARCH TESTS
// =============================================================================

#[tokio::test]
async fn test_knowledge_search_basic() {
    let ctx = TestContext::new().await;

    ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "SearchableEntity".to_string(),
        entity_type: "test".to_string(),
        observations: vec!["Unique observation for search".to_string()],
    }).await.unwrap();

    let params = KnowledgeSearchParams {
        query: "Searchable".to_string(),
        limit: 10,
    };

    let result = ctx.server.call_knowledge_search(params).await;
    assert!(result.is_ok());

    let entities = result.unwrap().entities;
    assert!(!entities.is_empty());
    assert!(entities.iter().any(|e| e.name == "SearchableEntity"));
}

#[tokio::test]
async fn test_knowledge_search_partial_match() {
    let ctx = TestContext::new().await;

    ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "VeryLongEntityName".to_string(),
        entity_type: "test".to_string(),
        observations: vec![],
    }).await.unwrap();

    let result = ctx.server.call_knowledge_search(KnowledgeSearchParams {
        query: "Long".to_string(),
        limit: 10,
    }).await.unwrap();

    assert!(result.entities.iter().any(|e| e.name.contains("Long")));
}

// =============================================================================
// GRAPH TRAVERSAL TESTS
// =============================================================================

#[tokio::test]
async fn test_read_graph() {
    let ctx = TestContext::new().await;

    // Create some entities
    for i in 0..5 {
        ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
            name: format!("Node{}", i),
            entity_type: "graph_test".to_string(),
            observations: vec![format!("Observation {}", i)],
        }).await.unwrap();
    }

    // Create relations
    ctx.server.call_knowledge_add_relation(KnowledgeAddRelationParams {
        from: "Node0".to_string(),
        to: "Node1".to_string(),
        relation_type: "next".to_string(),
    }).await.unwrap();

    // Read graph
    let params = KnowledgeReadGraphParams { limit: 0 };
    let result = ctx.server.call_knowledge_read_graph(params).await;

    assert!(result.is_ok());
    let graph = result.unwrap();
    assert!(graph.total_entities >= 5);
    assert!(graph.total_relations >= 1);
}

#[tokio::test]
async fn test_get_neighbors() {
    let ctx = TestContext::new().await;

    // Create star topology: Center -> A, B, C, D
    ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "Center".to_string(),
        entity_type: "hub".to_string(),
        observations: vec![],
    }).await.unwrap();

    for name in ["A", "B", "C", "D"] {
        ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
            name: name.to_string(),
            entity_type: "leaf".to_string(),
            observations: vec![],
        }).await.unwrap();

        ctx.server.call_knowledge_add_relation(KnowledgeAddRelationParams {
            from: "Center".to_string(),
            to: name.to_string(),
            relation_type: "connects".to_string(),
        }).await.unwrap();
    }

    // Get neighbors
    let params = KnowledgeGetNeighborsParams {
        entity_name: "Center".to_string(),
        max_depth: 1,
        relation_types: vec![],
    };

    let result = ctx.server.call_knowledge_get_neighbors(params).await;
    assert!(result.is_ok());

    let neighbors = result.unwrap();
    assert_eq!(neighbors.total, 4);
    assert!(neighbors.neighbors.iter().all(|n| n.distance == 1));
}

#[tokio::test]
async fn test_find_path() {
    let ctx = TestContext::new().await;

    // Create chain: Start -> Middle -> End
    for name in ["Start", "Middle", "End"] {
        ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
            name: name.to_string(),
            entity_type: "path_node".to_string(),
            observations: vec![],
        }).await.unwrap();
    }

    ctx.server.call_knowledge_add_relation(KnowledgeAddRelationParams {
        from: "Start".to_string(),
        to: "Middle".to_string(),
        relation_type: "next".to_string(),
    }).await.unwrap();

    ctx.server.call_knowledge_add_relation(KnowledgeAddRelationParams {
        from: "Middle".to_string(),
        to: "End".to_string(),
        relation_type: "next".to_string(),
    }).await.unwrap();

    // Find path
    let params = KnowledgeFindPathParams {
        from: "Start".to_string(),
        to: "End".to_string(),
        max_depth: 5,
    };

    let result = ctx.server.call_knowledge_find_path(params).await;
    assert!(result.is_ok());

    let path = result.unwrap();
    assert!(path.found);
    assert_eq!(path.length, 2); // Start -> Middle -> End = 2 hops
}

#[tokio::test]
async fn test_find_path_no_connection() {
    let ctx = TestContext::new().await;

    // Create disconnected nodes
    ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "Island1".to_string(),
        entity_type: "isolated".to_string(),
        observations: vec![],
    }).await.unwrap();

    ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "Island2".to_string(),
        entity_type: "isolated".to_string(),
        observations: vec![],
    }).await.unwrap();

    let result = ctx.server.call_knowledge_find_path(KnowledgeFindPathParams {
        from: "Island1".to_string(),
        to: "Island2".to_string(),
        max_depth: 10,
    }).await.unwrap();

    assert!(!result.found);
    assert!(result.path.is_empty());
}

// =============================================================================
// OBSERVATION TESTS
// =============================================================================

#[tokio::test]
async fn test_delete_observation() {
    let ctx = TestContext::new().await;

    ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "ObsTest".to_string(),
        entity_type: "test".to_string(),
        observations: vec![
            "Keep this".to_string(),
            "Delete this".to_string(),
            "Also keep".to_string(),
        ],
    }).await.unwrap();

    let params = KnowledgeDeleteObservationParams {
        entity_name: "ObsTest".to_string(),
        observations: vec!["Delete this".to_string()],
    };

    let result = ctx.server.call_knowledge_delete_observation(params).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().removed, 1);
}

// =============================================================================
// EXPORT TESTS
// =============================================================================

#[tokio::test]
async fn test_export_graph_basic() {
    let ctx = TestContext::new().await;

    // Create some data
    ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "ExportTest".to_string(),
        entity_type: "exportable".to_string(),
        observations: vec!["Test observation".to_string()],
    }).await.unwrap();

    let params = ExportGraphParams {
        format: "json".to_string(),
        entity_types: vec![],
        include_relations: true,
    };

    let result = ctx.server.call_export_graph(params).await;
    assert!(result.is_ok());

    let export = result.unwrap();
    assert!(export.entity_count > 0);
}

#[tokio::test]
async fn test_export_graph_filtered_by_type() {
    let ctx = TestContext::new().await;

    // Create entities of different types
    ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "Lang1".to_string(),
        entity_type: "language".to_string(),
        observations: vec![],
    }).await.unwrap();

    ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "Lib1".to_string(),
        entity_type: "library".to_string(),
        observations: vec![],
    }).await.unwrap();

    // Export only languages
    let params = ExportGraphParams {
        format: "json".to_string(),
        entity_types: vec!["language".to_string()],
        include_relations: false,
    };

    let result = ctx.server.call_export_graph(params).await.unwrap();

    // Should only have language entities
    assert!(result.entities.iter().all(|e| e.entity_type == "language"));
}

// =============================================================================
// COMPLEX GRAPH TESTS
// =============================================================================

#[tokio::test]
async fn test_complex_graph_structure() {
    let ctx = TestContext::new().await;

    // Create a more complex structure:
    // Project -> uses -> Framework -> extends -> BaseFramework
    // Project -> uses -> Database
    // Framework -> requires -> Database

    let entities = vec![
        ("MyProject", "project"),
        ("React", "framework"),
        ("BaseJS", "framework"),
        ("PostgreSQL", "database"),
    ];

    for (name, etype) in entities {
        ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
            name: name.to_string(),
            entity_type: etype.to_string(),
            observations: vec![],
        }).await.unwrap();
    }

    let relations = vec![
        ("MyProject", "React", "uses"),
        ("MyProject", "PostgreSQL", "uses"),
        ("React", "BaseJS", "extends"),
        ("React", "PostgreSQL", "requires"),
    ];

    for (from, to, rel) in relations {
        ctx.server.call_knowledge_add_relation(KnowledgeAddRelationParams {
            from: from.to_string(),
            to: to.to_string(),
            relation_type: rel.to_string(),
        }).await.unwrap();
    }

    // Test traversal from project
    let neighbors = ctx.server.call_knowledge_get_neighbors(KnowledgeGetNeighborsParams {
        entity_name: "MyProject".to_string(),
        max_depth: 2,
        relation_types: vec![],
    }).await.unwrap();

    // Should find all connected entities
    assert!(neighbors.total >= 3);

    // Test specific relation filter
    let uses_only = ctx.server.call_knowledge_get_neighbors(KnowledgeGetNeighborsParams {
        entity_name: "MyProject".to_string(),
        max_depth: 1,
        relation_types: vec!["uses".to_string()],
    }).await.unwrap();

    assert_eq!(uses_only.total, 2); // React and PostgreSQL
}
