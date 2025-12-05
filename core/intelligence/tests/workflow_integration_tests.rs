//! Workflow Integration Tests
//!
//! Tests complets simulant un agent qui utilise le serveur MCP
//! pour effectuer des tâches de développement en suivant le workflow ACID.
//!
//! Ces tests vérifient le comportement end-to-end du système.

mod common;

use common::TestContext;
use whytcard_intelligence::tools::{
    CortexProcessParams, ExternalDocsParams, ExternalSearchParams,
    KnowledgeAddEntityParams, KnowledgeAddRelationParams, KnowledgeSearchParams,
    MemoryStoreParams, MemorySearchParams, SequentialThinkingParams, TaskType,
};

// =============================================================================
// SCENARIO 1: Agent développe une nouvelle feature Rust
// =============================================================================

/// Simule le workflow complet d'un agent qui doit implémenter
/// une fonctionnalité d'error handling en Rust.
///
/// Workflow ACID:
/// A - Analyse: Recherche doc, vérifie existant
/// C - Code: Écrit le code
/// I - Intègre: Vérifie compilation
/// D - Documente: Stocke les décisions
#[tokio::test]
async fn test_scenario_agent_implements_rust_feature() {
    let ctx = TestContext::new().await;

    // =========================================================================
    // PHASE A: ANALYSE
    // =========================================================================

    // 1. Sequential thinking pour décomposer le problème
    let thinking = ctx.server.call_sequential_thinking(SequentialThinkingParams {
        problem: "Implement error handling with thiserror crate in a Rust library".to_string(),
        estimated_steps: 5,
        use_external: false,
    }).await.unwrap();

    assert!(thinking.steps.len() > 0);

    // 2. Vérifier si déjà documenté dans knowledge graph
    let existing = ctx.server.call_knowledge_search(KnowledgeSearchParams {
        query: "thiserror error handling".to_string(),
        limit: 5,
    }).await.unwrap();

    // 3. Recherche documentation externe (mocké en test)
    let _docs = ctx.server.call_external_docs(ExternalDocsParams {
        library: "thiserror".to_string(),
        topic: Some("derive macro".to_string()),
        max_tokens: 3000,
        source: "auto".to_string(),
    }).await;
    // Note: Peut échouer si pas de clé API, c'est OK en test

    // 4. Recherche best practices
    let _search = ctx.server.call_external_search(ExternalSearchParams {
        query: "rust error handling best practices 2025".to_string(),
        max_results: 5,
        search_type: "general".to_string(),
        include_domains: vec![],
        exclude_domains: vec![],
    }).await;
    // Note: Peut échouer si pas de clé API

    // =========================================================================
    // PHASE B: BONNES PRATIQUES (stocker avant de coder)
    // =========================================================================

    // 5. Stocker les décisions dans memory
    let decision_stored = ctx.server.call_memory_store(MemoryStoreParams {
        content: r#"
# Error Handling Decision Record

## Context
Implementing error handling for the Intelligence crate.

## Decision
Use thiserror for derive macros.

## Patterns
- Use #[error("message")] for display
- Box large error variants
- Implement From manually when needed

## Anti-patterns
- Don't use #[allow(dead_code)]
- Don't use anyhow in library code
"#.to_string(),
        title: Some("Error Handling Decision".to_string()),
        tags: vec!["decision".to_string(), "rust".to_string(), "error-handling".to_string()],
        metadata: Some(serde_json::json!({
            "phase": "analysis",
            "task": "implement-error-handling"
        })),
        index: true,
        key: Some("decision:error-handling-rust".to_string()),
    }).await.unwrap();

    assert!(!decision_stored.key.is_empty());

    // 6. Ajouter au knowledge graph
    ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "thiserror".to_string(),
        entity_type: "library".to_string(),
        observations: vec![
            "Rust error handling derive macro".to_string(),
            "Creates Display and Error implementations".to_string(),
            "Preferred for library error types".to_string(),
        ],
    }).await.unwrap();

    ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "WhytCard-Intelligence".to_string(),
        entity_type: "project".to_string(),
        observations: vec![
            "MCP server for cognitive memory".to_string(),
            "Written in Rust".to_string(),
        ],
    }).await.unwrap();

    ctx.server.call_knowledge_add_relation(KnowledgeAddRelationParams {
        from: "WhytCard-Intelligence".to_string(),
        to: "thiserror".to_string(),
        relation_type: "uses".to_string(),
    }).await.unwrap();

    // =========================================================================
    // PHASE C: CODE
    // =========================================================================

    // 7. Demander à CORTEX de générer le code
    let code_result = ctx.server.call_cortex_process(CortexProcessParams {
        query: "Generate an Error enum using thiserror with variants for Database, Config, and IO errors".to_string(),
        context: Some("Following the decision record for error handling".to_string()),
        session_id: Some("impl-error-handling".to_string()),
        auto_learn: true,
        inject_doubt: true,
        language: Some("rust".to_string()),
        task_type: Some(TaskType::Code),
    }).await.unwrap();

    assert!(code_result.success);

    // =========================================================================
    // PHASE I: INTÉGRATION
    // =========================================================================

    // 8. Vérifier que le code compile (simulation)
    // En vrai, on utiliserait cortex_execute avec cargo check
    // Ici on simule juste le passage du test

    // =========================================================================
    // PHASE D: DOCUMENTATION
    // =========================================================================

    // 9. Stocker le résultat
    ctx.server.call_memory_store(MemoryStoreParams {
        content: format!(
            "# Implementation Complete\n\nGenerated code:\n```rust\n{}\n```",
            code_result.output
        ),
        title: Some("Error Handling Implementation".to_string()),
        tags: vec!["implementation".to_string(), "rust".to_string(), "completed".to_string()],
        metadata: Some(serde_json::json!({
            "phase": "completed",
            "task": "implement-error-handling"
        })),
        index: true,
        key: Some("impl:error-handling-rust".to_string()),
    }).await.unwrap();

    // 10. Vérifier que tout est stocké
    let verification = ctx.server.call_memory_search(MemorySearchParams {
        query: "error handling rust".to_string(),
        limit: 10,
        min_score: None,
        tags: vec![],
    }).await.unwrap();

    assert!(verification.total >= 2); // Decision + Implementation
}

// =============================================================================
// SCENARIO 2: Agent debug un problème
// =============================================================================

#[tokio::test]
async fn test_scenario_agent_debugs_issue() {
    let ctx = TestContext::new().await;

    let error_message = r#"
error[E0502]: cannot borrow `data` as mutable because it is also borrowed as immutable
  --> src/main.rs:5:5
   |
4  |     let r = &data;
   |             ----- immutable borrow occurs here
5  |     data.push(4);
   |     ^^^^^^^^^^^^ mutable borrow occurs here
6  |     println!("{:?}", r);
   |                      - immutable borrow later used here
"#;

    // 1. Analyser l'erreur
    let analysis = ctx.server.call_cortex_process(CortexProcessParams {
        query: format!("Analyze this Rust compilation error and explain the issue:\n{}", error_message),
        context: None,
        session_id: Some("debug-session-1".to_string()),
        auto_learn: true,
        inject_doubt: true,
        language: Some("rust".to_string()),
        task_type: Some(TaskType::Research),
    }).await.unwrap();

    assert!(analysis.success);

    // 2. Chercher dans la mémoire des solutions passées
    let _past_solutions = ctx.server.call_memory_search(MemorySearchParams {
        query: "rust borrow checker mutable immutable".to_string(),
        limit: 5,
        min_score: Some(0.5),
        tags: vec![],
    }).await.unwrap();

    // 3. Proposer un fix
    let fix = ctx.server.call_cortex_process(CortexProcessParams {
        query: "Propose a fix for this borrow checker error".to_string(),
        context: Some(format!("Error:\n{}\n\nAnalysis:\n{}", error_message, analysis.output)),
        session_id: Some("debug-session-1".to_string()),
        auto_learn: true,
        inject_doubt: true,
        language: Some("rust".to_string()),
        task_type: Some(TaskType::Fix),
    }).await.unwrap();

    assert!(fix.success);

    // 4. Stocker la solution pour référence future
    ctx.server.call_memory_store(MemoryStoreParams {
        content: format!(
            "# Borrow Checker Fix\n\n## Problem\n{}\n\n## Solution\n{}",
            error_message, fix.output
        ),
        title: Some("Borrow Checker Fix: Mutable/Immutable Conflict".to_string()),
        tags: vec!["debug".to_string(), "rust".to_string(), "borrow-checker".to_string(), "solution".to_string()],
        metadata: None,
        index: true,
        key: None,
    }).await.unwrap();
}

// =============================================================================
// SCENARIO 3: Agent construit une base de connaissances
// =============================================================================

#[tokio::test]
async fn test_scenario_agent_builds_knowledge_base() {
    let ctx = TestContext::new().await;

    // Simuler la construction d'une base de connaissances sur l'écosystème Rust

    // 1. Créer les entités principales
    let languages = vec![
        ("Rust", "systems programming language", vec!["Memory safe", "Zero-cost abstractions", "No garbage collector"]),
        ("C", "systems programming language", vec!["Low-level", "Manual memory management"]),
        ("C++", "systems programming language", vec!["Object-oriented", "Templates"]),
    ];

    for (name, etype, observations) in &languages {
        ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
            name: name.to_string(),
            entity_type: etype.to_string(),
            observations: observations.iter().map(|s| s.to_string()).collect(),
        }).await.unwrap();
    }

    // 2. Créer les crates
    let crates = vec![
        ("tokio", "async runtime", vec!["Async runtime for Rust", "Event-driven"]),
        ("serde", "serialization", vec!["Serialization framework", "Derive macros"]),
        ("thiserror", "error handling", vec!["Error derive macro"]),
        ("anyhow", "error handling", vec!["Easy error handling", "For applications"]),
    ];

    for (name, etype, observations) in &crates {
        ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
            name: name.to_string(),
            entity_type: etype.to_string(),
            observations: observations.iter().map(|s| s.to_string()).collect(),
        }).await.unwrap();
    }

    // 3. Créer les relations
    let relations = vec![
        ("Rust", "C", "inspired_by"),
        ("Rust", "C++", "alternative_to"),
        ("tokio", "Rust", "written_in"),
        ("serde", "Rust", "written_in"),
        ("thiserror", "Rust", "written_in"),
        ("anyhow", "thiserror", "complements"),
    ];

    for (from, to, rel) in &relations {
        ctx.server.call_knowledge_add_relation(KnowledgeAddRelationParams {
            from: from.to_string(),
            to: to.to_string(),
            relation_type: rel.to_string(),
        }).await.unwrap();
    }

    // 4. Stocker des documents de référence
    let docs = vec![
        ("Rust Error Handling Guide", "error-handling", "Use thiserror for libraries, anyhow for applications."),
        ("Async Rust Best Practices", "async", "Use tokio for async runtime. Avoid blocking in async context."),
        ("Serialization Patterns", "serialization", "Use serde with derive for struct serialization."),
    ];

    for (title, tag, content) in &docs {
        ctx.server.call_memory_store(MemoryStoreParams {
            content: content.to_string(),
            title: Some(title.to_string()),
            tags: vec!["reference".to_string(), tag.to_string(), "rust".to_string()],
            metadata: None,
            index: true,
            key: None,
        }).await.unwrap();
    }

    // 5. Vérifier la base de connaissances
    let rust_entity = ctx.server.call_knowledge_get_entity(
        whytcard_intelligence::tools::KnowledgeGetEntityParams {
            name: "Rust".to_string(),
            include_relations: true,
        }
    ).await.unwrap();

    assert!(!rust_entity.outgoing.is_empty());
    assert!(!rust_entity.incoming.is_empty());

    // 6. Rechercher dans le graphe
    let search = ctx.server.call_knowledge_search(KnowledgeSearchParams {
        query: "async".to_string(),
        limit: 10,
    }).await.unwrap();

    // Devrait trouver tokio
    assert!(search.entities.iter().any(|e| e.name == "tokio"));
}

// =============================================================================
// SCENARIO 4: Agent utilise le contexte agrégé
// =============================================================================

#[tokio::test]
async fn test_scenario_agent_uses_aggregated_context() {
    let ctx = TestContext::new().await;

    // 1. Peupler les différentes sources de mémoire
    // Semantic memory (via memory_store)
    ctx.server.call_memory_store(MemoryStoreParams {
        content: "Rust Result type is used for error handling. It has Ok and Err variants.".to_string(),
        title: Some("Rust Result Type".to_string()),
        tags: vec!["rust".to_string(), "result".to_string()],
        metadata: None,
        index: true,
        key: None,
    }).await.unwrap();

    // Knowledge graph
    ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
        name: "Result".to_string(),
        entity_type: "type".to_string(),
        observations: vec![
            "Rust enum for error handling".to_string(),
            "Generic over T (success) and E (error)".to_string(),
        ],
    }).await.unwrap();

    // 2. Utiliser get_context pour agréger tout
    let context = ctx.server.call_get_context(
        whytcard_intelligence::tools::GetContextParams {
            query: "Rust Result error handling".to_string(),
            context_type: "query".to_string(),
        }
    ).await.unwrap();

    // 3. Vérifier l'agrégation
    assert!(!context.summary.is_empty());
    // Devrait avoir des résultats sémantiques
    // assert!(!context.semantic_items.is_empty());

    // 4. Utiliser ce contexte pour une requête CORTEX
    let answer = ctx.server.call_cortex_process(CortexProcessParams {
        query: "Explain how to use Result in Rust".to_string(),
        context: Some(context.summary),
        session_id: None,
        auto_learn: true,
        inject_doubt: false,
        language: Some("rust".to_string()),
        task_type: Some(TaskType::Document),
    }).await.unwrap();

    assert!(answer.success);
}

// =============================================================================
// SCENARIO 5: Agent suit le workflow ACID complet avec vérification
// =============================================================================

#[tokio::test]
async fn test_scenario_acid_workflow_with_verification() {
    let ctx = TestContext::new().await;

    // Ce test simule un agent qui suit strictement le workflow ACID
    // avec vérification à chaque étape

    let task = "Add logging to a Rust function";

    // =========================================================================
    // A - ANALYSE
    // =========================================================================

    // A.1 - Sequential thinking
    let thinking = ctx.server.call_sequential_thinking(SequentialThinkingParams {
        problem: task.to_string(),
        estimated_steps: 4,
        use_external: false,
    }).await.unwrap();

    assert!(thinking.complete || thinking.steps.len() >= 1);

    // A.2 - Vérifier l'existant
    let existing_knowledge = ctx.server.call_knowledge_search(KnowledgeSearchParams {
        query: "logging tracing rust".to_string(),
        limit: 5,
    }).await.unwrap();

    let existing_memory = ctx.server.call_memory_search(MemorySearchParams {
        query: "logging best practices rust".to_string(),
        limit: 5,
        min_score: None,
        tags: vec![],
    }).await.unwrap();

    // A.3 - Si pas d'existant, documenter
    if existing_knowledge.entities.is_empty() {
        ctx.server.call_knowledge_add_entity(KnowledgeAddEntityParams {
            name: "tracing".to_string(),
            entity_type: "crate".to_string(),
            observations: vec![
                "Rust logging/tracing framework".to_string(),
                "Structured logging".to_string(),
                "Preferred over log crate for new projects".to_string(),
            ],
        }).await.unwrap();
    }

    if existing_memory.results.is_empty() {
        ctx.server.call_memory_store(MemoryStoreParams {
            content: r#"
# Rust Logging Best Practices

1. Use `tracing` crate instead of `log`
2. Add spans for function boundaries
3. Use structured fields: `tracing::info!(user_id = %id, "action")`
4. Configure subscriber with environment filter
"#.to_string(),
            title: Some("Rust Logging Best Practices".to_string()),
            tags: vec!["logging".to_string(), "rust".to_string(), "best-practices".to_string()],
            metadata: None,
            index: true,
            key: Some("doc:rust-logging".to_string()),
        }).await.unwrap();
    }

    // =========================================================================
    // C - CODE (via CORTEX)
    // =========================================================================

    let code = ctx.server.call_cortex_process(CortexProcessParams {
        query: "Add tracing instrumentation to this function: fn process_data(input: &str) -> Result<(), Error> { ... }".to_string(),
        context: Some("Using tracing crate with spans and structured logging".to_string()),
        session_id: Some("acid-test-1".to_string()),
        auto_learn: true,
        inject_doubt: true,
        language: Some("rust".to_string()),
        task_type: Some(TaskType::Code),
    }).await.unwrap();

    assert!(code.success);

    // =========================================================================
    // I - INTÉGRATION (vérification via cortex_execute)
    // =========================================================================

    // Note: En vrai on ferait cargo check, ici on simule
    let _check = ctx.server.call_cortex_execute(
        whytcard_intelligence::tools::CortexExecuteParams {
            command: "echo 'Simulation: cargo check passed'".to_string(),
            cwd: None,
            env: None,
            timeout_secs: 30,
            separate_stderr: false,
        }
    ).await.unwrap();

    // =========================================================================
    // D - DOCUMENTATION
    // =========================================================================

    // D.1 - Stocker le résultat
    ctx.server.call_memory_store(MemoryStoreParams {
        content: format!("# Task: {}\n\n## Code:\n```rust\n{}\n```\n\n## Status: Completed", task, code.output),
        title: Some(format!("Task: {}", task)),
        tags: vec!["completed".to_string(), "logging".to_string()],
        metadata: Some(serde_json::json!({
            "task": task,
            "session": "acid-test-1",
            "status": "completed"
        })),
        index: true,
        key: None,
    }).await.unwrap();

    // D.2 - Mettre à jour knowledge graph
    ctx.server.call_knowledge_add_observation(
        whytcard_intelligence::tools::KnowledgeAddObservationParams {
            entity_name: "tracing".to_string(),
            observations: vec![format!("Used in task: {}", task)],
        }
    ).await.unwrap();

    // Vérification finale
    let final_check = ctx.server.call_memory_search(MemorySearchParams {
        query: task.to_string(),
        limit: 1,
        min_score: None,
        tags: vec!["completed".to_string()],
    }).await.unwrap();

    assert!(!final_check.results.is_empty(), "Task should be documented as completed");
}
