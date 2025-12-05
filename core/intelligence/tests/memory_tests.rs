//! Memory Tools Integration Tests
//!
//! Tests all memory operations:
//! - memory_store, memory_get, memory_search, memory_delete, memory_list
//! - batch_store, hybrid_search, manage_tags, get_context

mod common;

use common::{random_key, test_content, TestContext};
use whytcard_intelligence::tools::{
    BatchStoreParams, GetContextParams, HybridSearchParams, ManageTagsParams,
    MemoryDeleteParams, MemoryGetParams, MemoryListParams, MemorySearchParams,
    MemoryStoreParams,
};

// =============================================================================
// MEMORY_STORE TESTS
// =============================================================================

#[tokio::test]
async fn test_memory_store_basic() {
    let ctx = TestContext::new().await;

    let params = MemoryStoreParams {
        content: "Test content for memory store".to_string(),
        title: Some("Test Title".to_string()),
        tags: vec!["test".to_string(), "memory".to_string()],
        metadata: None,
        index: true,
        key: None,
    };

    let result = ctx.server.call_memory_store(params).await;
    assert!(result.is_ok());

    let res = result.unwrap();
    assert!(!res.key.is_empty());
    assert!(res.indexed);
}

#[tokio::test]
async fn test_memory_store_with_custom_key() {
    let ctx = TestContext::new().await;
    let custom_key = random_key();

    let params = MemoryStoreParams {
        content: "Content with custom key".to_string(),
        title: None,
        tags: vec![],
        metadata: Some(serde_json::json!({"custom": "data"})),
        index: false,
        key: Some(custom_key.clone()),
    };

    let result = ctx.server.call_memory_store(params).await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap().key, custom_key);
}

#[tokio::test]
async fn test_memory_store_with_metadata() {
    let ctx = TestContext::new().await;

    let metadata = serde_json::json!({
        "source": "test",
        "version": "1.0",
        "nested": {
            "key": "value"
        }
    });

    let params = MemoryStoreParams {
        content: "Content with rich metadata".to_string(),
        title: Some("Metadata Test".to_string()),
        tags: vec!["metadata".to_string()],
        metadata: Some(metadata.clone()),
        index: true,
        key: None,
    };

    let result = ctx.server.call_memory_store(params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_memory_store_empty_content() {
    let ctx = TestContext::new().await;

    let params = MemoryStoreParams {
        content: "".to_string(),
        title: None,
        tags: vec![],
        metadata: None,
        index: true,
        key: None,
    };

    // Empty content should still work (edge case)
    let result = ctx.server.call_memory_store(params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_memory_store_unicode_content() {
    let ctx = TestContext::new().await;

    let params = MemoryStoreParams {
        content: "Contenu français avec accents: é, è, ê, à, ù, ô, î, ç 中文 日本語 🚀".to_string(),
        title: Some("Test Unicode éàç".to_string()),
        tags: vec!["unicode".to_string(), "français".to_string()],
        metadata: None,
        index: true,
        key: None,
    };

    let result = ctx.server.call_memory_store(params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_memory_store_large_content() {
    let ctx = TestContext::new().await;

    // Generate 100KB of content
    let large_content = "a".repeat(100_000);

    let params = MemoryStoreParams {
        content: large_content,
        title: Some("Large Content Test".to_string()),
        tags: vec!["large".to_string()],
        metadata: None,
        index: true,
        key: None,
    };

    let result = ctx.server.call_memory_store(params).await;
    assert!(result.is_ok());
}

// =============================================================================
// MEMORY_GET TESTS
// =============================================================================

#[tokio::test]
async fn test_memory_get_existing() {
    let ctx = TestContext::new().await;

    // Store first
    let store_params = MemoryStoreParams {
        content: "Content to retrieve".to_string(),
        title: Some("Retrieve Test".to_string()),
        tags: vec!["retrieve".to_string()],
        metadata: None,
        index: true,
        key: None,
    };

    let stored = ctx.server.call_memory_store(store_params).await.unwrap();

    // Get
    let get_params = MemoryGetParams {
        key: stored.key.clone(),
    };

    let result = ctx.server.call_memory_get(get_params).await;
    assert!(result.is_ok());

    let memory = result.unwrap();
    assert_eq!(memory.content, "Content to retrieve");
    assert_eq!(memory.title, Some("Retrieve Test".to_string()));
}

#[tokio::test]
async fn test_memory_get_nonexistent() {
    let ctx = TestContext::new().await;

    let get_params = MemoryGetParams {
        key: "nonexistent-key-12345".to_string(),
    };

    let result = ctx.server.call_memory_get(get_params).await;
    assert!(result.is_err());
}

// =============================================================================
// MEMORY_SEARCH TESTS
// =============================================================================

#[tokio::test]
async fn test_memory_search_basic() {
    let ctx = TestContext::new().await;

    // Store some content
    let store_params = MemoryStoreParams {
        content: "Machine learning is a subset of artificial intelligence".to_string(),
        title: Some("ML Definition".to_string()),
        tags: vec!["ml".to_string(), "ai".to_string()],
        metadata: None,
        index: true,
        key: None,
    };
    ctx.server.call_memory_store(store_params).await.unwrap();

    // Search
    let search_params = MemorySearchParams {
        query: "artificial intelligence".to_string(),
        limit: 10,
        min_score: Some(0.0),
        tags: vec![],
    };

    let result = ctx.server.call_memory_search(search_params).await;
    assert!(result.is_ok());

    let search_result = result.unwrap();
    assert!(search_result.total > 0);
}

#[tokio::test]
async fn test_memory_search_with_tags_filter() {
    let ctx = TestContext::new().await;

    // Store with specific tag
    let store_params = MemoryStoreParams {
        content: "Tagged content for filtering".to_string(),
        title: None,
        tags: vec!["special-tag".to_string()],
        metadata: None,
        index: true,
        key: None,
    };
    ctx.server.call_memory_store(store_params).await.unwrap();

    // Search with tag filter
    let search_params = MemorySearchParams {
        query: "tagged content".to_string(),
        limit: 10,
        min_score: None,
        tags: vec!["special-tag".to_string()],
    };

    let result = ctx.server.call_memory_search(search_params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_memory_search_min_score() {
    let ctx = TestContext::new().await;

    let store_params = MemoryStoreParams {
        content: "High relevance content about Rust programming language".to_string(),
        title: None,
        tags: vec![],
        metadata: None,
        index: true,
        key: None,
    };
    ctx.server.call_memory_store(store_params).await.unwrap();

    // Search with high min_score
    let search_params = MemorySearchParams {
        query: "Rust programming".to_string(),
        limit: 10,
        min_score: Some(0.9),
        tags: vec![],
    };

    let result = ctx.server.call_memory_search(search_params).await;
    assert!(result.is_ok());

    // All results should have score >= 0.9
    for item in result.unwrap().results {
        assert!(item.score >= 0.9);
    }
}

#[tokio::test]
async fn test_memory_search_empty_results() {
    let ctx = TestContext::new().await;

    let search_params = MemorySearchParams {
        query: "xyznonexistentquery123456".to_string(),
        limit: 10,
        min_score: None,
        tags: vec![],
    };

    let result = ctx.server.call_memory_search(search_params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().results.is_empty());
}

// =============================================================================
// MEMORY_DELETE TESTS
// =============================================================================

#[tokio::test]
async fn test_memory_delete_existing() {
    let ctx = TestContext::new().await;

    // Store
    let store_params = MemoryStoreParams {
        content: "Content to delete".to_string(),
        title: None,
        tags: vec![],
        metadata: None,
        index: true,
        key: None,
    };
    let stored = ctx.server.call_memory_store(store_params).await.unwrap();

    // Delete
    let delete_params = MemoryDeleteParams {
        key: stored.key.clone(),
    };
    let result = ctx.server.call_memory_delete(delete_params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().deleted);

    // Verify deleted
    let get_params = MemoryGetParams {
        key: stored.key,
    };
    assert!(ctx.server.call_memory_get(get_params).await.is_err());
}

#[tokio::test]
async fn test_memory_delete_nonexistent() {
    let ctx = TestContext::new().await;

    let delete_params = MemoryDeleteParams {
        key: "nonexistent-key".to_string(),
    };

    let result = ctx.server.call_memory_delete(delete_params).await;
    assert!(result.is_ok());
    assert!(!result.unwrap().deleted);
}

// =============================================================================
// MEMORY_LIST TESTS
// =============================================================================

#[tokio::test]
async fn test_memory_list_basic() {
    let ctx = TestContext::new().await;

    // Store multiple items
    for i in 0..5 {
        let store_params = MemoryStoreParams {
            content: format!("List item {}", i),
            title: Some(format!("Item {}", i)),
            tags: vec!["list-test".to_string()],
            metadata: None,
            index: false,
            key: None,
        };
        ctx.server.call_memory_store(store_params).await.unwrap();
    }

    // List
    let list_params = MemoryListParams {
        limit: 10,
        offset: 0,
        tags: vec![],
    };

    let result = ctx.server.call_memory_list(list_params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().memories.len() >= 5);
}

#[tokio::test]
async fn test_memory_list_pagination() {
    let ctx = TestContext::new().await;

    // Store 10 items
    for i in 0..10 {
        let store_params = MemoryStoreParams {
            content: format!("Paginated item {}", i),
            title: None,
            tags: vec!["pagination".to_string()],
            metadata: None,
            index: false,
            key: None,
        };
        ctx.server.call_memory_store(store_params).await.unwrap();
    }

    // First page
    let list_params = MemoryListParams {
        limit: 5,
        offset: 0,
        tags: vec!["pagination".to_string()],
    };
    let page1 = ctx.server.call_memory_list(list_params).await.unwrap();
    assert_eq!(page1.memories.len(), 5);
    assert!(page1.has_more);

    // Second page
    let list_params = MemoryListParams {
        limit: 5,
        offset: 5,
        tags: vec!["pagination".to_string()],
    };
    let page2 = ctx.server.call_memory_list(list_params).await.unwrap();
    assert_eq!(page2.memories.len(), 5);
}

#[tokio::test]
async fn test_memory_list_with_tags_filter() {
    let ctx = TestContext::new().await;

    // Store with different tags
    ctx.server.call_memory_store(MemoryStoreParams {
        content: "Item A".to_string(),
        title: None,
        tags: vec!["tag-a".to_string()],
        metadata: None,
        index: false,
        key: None,
    }).await.unwrap();

    ctx.server.call_memory_store(MemoryStoreParams {
        content: "Item B".to_string(),
        title: None,
        tags: vec!["tag-b".to_string()],
        metadata: None,
        index: false,
        key: None,
    }).await.unwrap();

    // List only tag-a
    let list_params = MemoryListParams {
        limit: 10,
        offset: 0,
        tags: vec!["tag-a".to_string()],
    };

    let result = ctx.server.call_memory_list(list_params).await.unwrap();
    assert_eq!(result.memories.len(), 1);
}

// =============================================================================
// BATCH_STORE TESTS
// =============================================================================

#[tokio::test]
async fn test_batch_store_basic() {
    let ctx = TestContext::new().await;

    let params = BatchStoreParams {
        items: vec![
            whytcard_intelligence::tools::BatchStoreItem {
                content: "Batch item 1".to_string(),
                source: "test".to_string(),
                category: "batch".to_string(),
                tags: vec!["batch".to_string()],
                metadata: None,
            },
            whytcard_intelligence::tools::BatchStoreItem {
                content: "Batch item 2".to_string(),
                source: "test".to_string(),
                category: "batch".to_string(),
                tags: vec!["batch".to_string()],
                metadata: None,
            },
            whytcard_intelligence::tools::BatchStoreItem {
                content: "Batch item 3".to_string(),
                source: "test".to_string(),
                category: "batch".to_string(),
                tags: vec!["batch".to_string()],
                metadata: None,
            },
        ],
    };

    let result = ctx.server.call_batch_store(params).await;
    assert!(result.is_ok());

    let res = result.unwrap();
    assert_eq!(res.stored, 3);
    assert_eq!(res.keys.len(), 3);
    assert!(res.errors.is_empty());
}

#[tokio::test]
async fn test_batch_store_large_batch() {
    let ctx = TestContext::new().await;

    // Create 100 items
    let items: Vec<_> = (0..100)
        .map(|i| whytcard_intelligence::tools::BatchStoreItem {
            content: format!("Large batch item {}", i),
            source: "stress-test".to_string(),
            category: "large-batch".to_string(),
            tags: vec!["large".to_string()],
            metadata: None,
        })
        .collect();

    let params = BatchStoreParams { items };

    let result = ctx.server.call_batch_store(params).await;
    assert!(result.is_ok());

    let res = result.unwrap();
    assert_eq!(res.stored, 100);
}

// =============================================================================
// HYBRID_SEARCH TESTS
// =============================================================================

#[tokio::test]
async fn test_hybrid_search_basic() {
    let ctx = TestContext::new().await;

    // Store semantic content
    ctx.server.call_memory_store(MemoryStoreParams {
        content: "Rust is a systems programming language".to_string(),
        title: None,
        tags: vec!["rust".to_string()],
        metadata: None,
        index: true,
        key: None,
    }).await.unwrap();

    let params = HybridSearchParams {
        query: "Rust programming".to_string(),
        top_k: 10,
        min_relevance: 0.3,
    };

    let result = ctx.server.call_hybrid_search(params).await;
    assert!(result.is_ok());

    let res = result.unwrap();
    // Should have results from at least one memory type
    assert!(
        !res.semantic.is_empty() || !res.episodic.is_empty() || !res.procedural.is_empty(),
        "Hybrid search should return at least one result"
    );
}

// =============================================================================
// MANAGE_TAGS TESTS
// =============================================================================

#[tokio::test]
async fn test_manage_tags_add() {
    let ctx = TestContext::new().await;

    // Store
    let stored = ctx.server.call_memory_store(MemoryStoreParams {
        content: "Content for tag management".to_string(),
        title: None,
        tags: vec!["original".to_string()],
        metadata: None,
        index: false,
        key: None,
    }).await.unwrap();

    // Add tags
    let params = ManageTagsParams {
        action: "add".to_string(),
        doc_id: Some(stored.key.clone()),
        tags: vec!["new-tag1".to_string(), "new-tag2".to_string()],
    };

    let result = ctx.server.call_manage_tags(params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}

#[tokio::test]
async fn test_manage_tags_remove() {
    let ctx = TestContext::new().await;

    // Store with multiple tags
    let stored = ctx.server.call_memory_store(MemoryStoreParams {
        content: "Content with tags to remove".to_string(),
        title: None,
        tags: vec!["keep".to_string(), "remove-me".to_string()],
        metadata: None,
        index: false,
        key: None,
    }).await.unwrap();

    // Remove tag
    let params = ManageTagsParams {
        action: "remove".to_string(),
        doc_id: Some(stored.key.clone()),
        tags: vec!["remove-me".to_string()],
    };

    let result = ctx.server.call_manage_tags(params).await;
    assert!(result.is_ok());
    assert!(result.unwrap().success);
}

#[tokio::test]
async fn test_manage_tags_get() {
    let ctx = TestContext::new().await;

    let stored = ctx.server.call_memory_store(MemoryStoreParams {
        content: "Content to get tags from".to_string(),
        title: None,
        tags: vec!["tag1".to_string(), "tag2".to_string()],
        metadata: None,
        index: false,
        key: None,
    }).await.unwrap();

    let params = ManageTagsParams {
        action: "get".to_string(),
        doc_id: Some(stored.key),
        tags: vec![],
    };

    let result = ctx.server.call_manage_tags(params).await;
    assert!(result.is_ok());

    let res = result.unwrap();
    assert!(res.tags.contains(&"tag1".to_string()));
    assert!(res.tags.contains(&"tag2".to_string()));
}

#[tokio::test]
async fn test_manage_tags_search() {
    let ctx = TestContext::new().await;

    // Store items with specific tags
    ctx.server.call_memory_store(MemoryStoreParams {
        content: "Searchable by tag".to_string(),
        title: None,
        tags: vec!["searchable-unique-tag".to_string()],
        metadata: None,
        index: false,
        key: None,
    }).await.unwrap();

    let params = ManageTagsParams {
        action: "search".to_string(),
        doc_id: None,
        tags: vec!["searchable-unique-tag".to_string()],
    };

    let result = ctx.server.call_manage_tags(params).await;
    assert!(result.is_ok());
    assert!(!result.unwrap().results.is_empty());
}

// =============================================================================
// GET_CONTEXT TESTS
// =============================================================================

#[tokio::test]
async fn test_get_context_basic() {
    let ctx = TestContext::new().await;

    // Store some context
    ctx.server.call_memory_store(MemoryStoreParams {
        content: "Context about Rust error handling with Result type".to_string(),
        title: None,
        tags: vec!["rust".to_string(), "error".to_string()],
        metadata: None,
        index: true,
        key: None,
    }).await.unwrap();

    let params = GetContextParams {
        query: "Rust error handling".to_string(),
        context_type: "query".to_string(),
    };

    let result = ctx.server.call_get_context(params).await;
    assert!(result.is_ok());

    let res = result.unwrap();
    assert!(!res.summary.is_empty());
}

#[tokio::test]
async fn test_get_context_empty_query() {
    let ctx = TestContext::new().await;

    let params = GetContextParams {
        query: "".to_string(),
        context_type: "query".to_string(),
    };

    let result = ctx.server.call_get_context(params).await;
    // Should handle empty query gracefully
    assert!(result.is_ok());
}
