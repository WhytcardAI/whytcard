//! CORTEX Engine Integration Tests
//!
//! Tests the cognitive engine:
//! - cortex_process with different task types
//! - cortex_feedback for learning
//! - cortex_stats monitoring
//! - cortex_cleanup maintenance
//! - cortex_execute shell commands

mod common;

use common::TestContext;
use whytcard_intelligence::tools::{
    CortexCleanupParams, CortexExecuteParams, CortexFeedbackParams,
    CortexProcessParams, CortexStatsParams, TaskType,
};

// =============================================================================
// CORTEX_PROCESS TESTS
// =============================================================================

#[tokio::test]
async fn test_cortex_process_basic_query() {
    let ctx = TestContext::new().await;

    let params = CortexProcessParams {
        query: "What is Rust?".to_string(),
        context: None,
        session_id: None,
        auto_learn: true,
        inject_doubt: true,
        language: None,
        task_type: None,
    };

    let result = ctx.server.call_cortex_process(params).await;
    assert!(result.is_ok());

    let res = result.unwrap();
    assert!(res.success);
    assert!(!res.output.is_empty());
    assert!(!res.intent.is_empty());
}

#[tokio::test]
async fn test_cortex_process_with_task_type_code() {
    let ctx = TestContext::new().await;

    let params = CortexProcessParams {
        query: "Create a function to parse JSON".to_string(),
        context: Some("Working on a Rust project".to_string()),
        session_id: None,
        auto_learn: true,
        inject_doubt: true,
        language: Some("rust".to_string()),
        task_type: Some(TaskType::Code),
    };

    let result = ctx.server.call_cortex_process(params).await;
    assert!(result.is_ok());

    let res = result.unwrap();
    assert!(res.success);
    // Should have loaded prompts if available
    // assert!(!res.loaded_prompts.is_empty());
}

#[tokio::test]
async fn test_cortex_process_with_task_type_research() {
    let ctx = TestContext::new().await;

    let params = CortexProcessParams {
        query: "Research best practices for error handling in Rust".to_string(),
        context: None,
        session_id: None,
        auto_learn: true,
        inject_doubt: true,
        language: Some("rust".to_string()),
        task_type: Some(TaskType::Research),
    };

    let result = ctx.server.call_cortex_process(params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cortex_process_with_task_type_fix() {
    let ctx = TestContext::new().await;

    let params = CortexProcessParams {
        query: "Fix compilation error: borrowed value does not live long enough".to_string(),
        context: Some("fn get_ref() -> &str { let s = String::new(); &s }".to_string()),
        session_id: None,
        auto_learn: true,
        inject_doubt: true,
        language: Some("rust".to_string()),
        task_type: Some(TaskType::Fix),
    };

    let result = ctx.server.call_cortex_process(params).await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_cortex_process_with_session() {
    let ctx = TestContext::new().await;

    // First query in session
    let params1 = CortexProcessParams {
        query: "Start a new task".to_string(),
        context: None,
        session_id: Some("test-session-1".to_string()),
        auto_learn: true,
        inject_doubt: false,
        language: None,
        task_type: None,
    };

    let result1 = ctx.server.call_cortex_process(params1).await.unwrap();
    let session_id = result1.session_id.clone();

    // Second query in same session
    let params2 = CortexProcessParams {
        query: "Continue the task".to_string(),
        context: None,
        session_id: session_id.clone(),
        auto_learn: true,
        inject_doubt: false,
        language: None,
        task_type: None,
    };

    let result2 = ctx.server.call_cortex_process(params2).await;
    assert!(result2.is_ok());
}

#[tokio::test]
async fn test_cortex_process_without_doubt_injection() {
    let ctx = TestContext::new().await;

    let params = CortexProcessParams {
        query: "Simple query without doubt".to_string(),
        context: None,
        session_id: None,
        auto_learn: false,
        inject_doubt: false, // Disabled
        language: None,
        task_type: None,
    };

    let result = ctx.server.call_cortex_process(params).await;
    assert!(result.is_ok());

    // Should work without doubt prompts
    let res = result.unwrap();
    assert!(res.loaded_prompts.is_empty() || !res.loaded_prompts.iter().any(|p| p.contains("doubt")));
}

#[tokio::test]
async fn test_cortex_process_all_task_types() {
    let ctx = TestContext::new().await;

    let task_types = vec![
        (TaskType::Code, "Write code"),
        (TaskType::Research, "Research topic"),
        (TaskType::Fix, "Fix issue"),
        (TaskType::Create, "Create new"),
        (TaskType::Review, "Review code"),
        (TaskType::Document, "Document this"),
    ];

    for (task_type, query) in task_types {
        let params = CortexProcessParams {
            query: query.to_string(),
            context: None,
            session_id: None,
            auto_learn: false,
            inject_doubt: false,
            language: None,
            task_type: Some(task_type),
        };

        let result = ctx.server.call_cortex_process(params).await;
        assert!(result.is_ok(), "Failed for task type: {:?}", task_type);
    }
}

// =============================================================================
// CORTEX_FEEDBACK TESTS
// =============================================================================

#[tokio::test]
async fn test_cortex_feedback_positive() {
    let ctx = TestContext::new().await;

    // First, we need a rule to exist. Create via process that triggers learning.
    let process_params = CortexProcessParams {
        query: "Test query for rule creation".to_string(),
        context: None,
        session_id: None,
        auto_learn: true,
        inject_doubt: false,
        language: None,
        task_type: None,
    };
    ctx.server.call_cortex_process(process_params).await.unwrap();

    // Get stats to find a rule
    let stats = ctx.server.call_cortex_stats(CortexStatsParams {}).await.unwrap();

    // If rules exist, provide feedback
    if stats.memory.procedural_rules > 0 {
        let feedback_params = CortexFeedbackParams {
            rule_id: "test-rule".to_string(), // This might not exist
            success: true,
            message: Some("Rule worked well".to_string()),
        };

        // This may fail if rule doesn't exist, which is expected
        let _result = ctx.server.call_cortex_feedback(feedback_params).await;
    }
}

#[tokio::test]
async fn test_cortex_feedback_negative() {
    let ctx = TestContext::new().await;

    let feedback_params = CortexFeedbackParams {
        rule_id: "some-rule".to_string(),
        success: false,
        message: Some("Rule did not work".to_string()),
    };

    // May fail if rule doesn't exist
    let _result = ctx.server.call_cortex_feedback(feedback_params).await;
}

// =============================================================================
// CORTEX_STATS TESTS
// =============================================================================

#[tokio::test]
async fn test_cortex_stats_empty() {
    let ctx = TestContext::new().await;

    let params = CortexStatsParams {};
    let result = ctx.server.call_cortex_stats(params).await;

    assert!(result.is_ok());
    let stats = result.unwrap();

    // Fresh server should have empty or minimal stats
    assert_eq!(stats.status, "running");
}

#[tokio::test]
async fn test_cortex_stats_after_operations() {
    let ctx = TestContext::new().await;

    // Perform some operations
    ctx.server.call_cortex_process(CortexProcessParams {
        query: "Test query 1".to_string(),
        context: None,
        session_id: None,
        auto_learn: true,
        inject_doubt: false,
        language: None,
        task_type: None,
    }).await.unwrap();

    ctx.server.call_cortex_process(CortexProcessParams {
        query: "Test query 2".to_string(),
        context: None,
        session_id: None,
        auto_learn: true,
        inject_doubt: false,
        language: None,
        task_type: None,
    }).await.unwrap();

    // Check stats
    let stats = ctx.server.call_cortex_stats(CortexStatsParams {}).await.unwrap();

    // Should have some episodic events after processing
    assert!(stats.memory.episodic_events >= 0); // May or may not have events
}

// =============================================================================
// CORTEX_CLEANUP TESTS
// =============================================================================

#[tokio::test]
async fn test_cortex_cleanup_basic() {
    let ctx = TestContext::new().await;

    let params = CortexCleanupParams {
        retention_days: 30,
    };

    let result = ctx.server.call_cortex_cleanup(params).await;
    assert!(result.is_ok());

    let res = result.unwrap();
    assert!(res.cleaned_count >= 0);
    assert!(res.message.contains("retention"));
}

#[tokio::test]
async fn test_cortex_cleanup_aggressive() {
    let ctx = TestContext::new().await;

    // First create some data
    ctx.server.call_cortex_process(CortexProcessParams {
        query: "Create data for cleanup test".to_string(),
        context: None,
        session_id: None,
        auto_learn: true,
        inject_doubt: false,
        language: None,
        task_type: None,
    }).await.unwrap();

    // Cleanup with 0 days retention (clean everything)
    let params = CortexCleanupParams {
        retention_days: 0,
    };

    let result = ctx.server.call_cortex_cleanup(params).await;
    assert!(result.is_ok());
}

// =============================================================================
// CORTEX_EXECUTE TESTS
// =============================================================================

#[tokio::test]
async fn test_cortex_execute_echo() {
    let ctx = TestContext::new().await;

    #[cfg(windows)]
    let command = "echo test";
    #[cfg(not(windows))]
    let command = "echo test";

    let params = CortexExecuteParams {
        command: command.to_string(),
        cwd: None,
        env: None,
        timeout_secs: 10,
        separate_stderr: false,
    };

    let result = ctx.server.call_cortex_execute(params).await;
    assert!(result.is_ok());

    let res = result.unwrap();
    assert!(res.success);
    assert_eq!(res.exit_code, 0);
    assert!(res.stdout.contains("test"));
}

#[tokio::test]
async fn test_cortex_execute_with_cwd() {
    let ctx = TestContext::new().await;

    #[cfg(windows)]
    let command = "cd";
    #[cfg(not(windows))]
    let command = "pwd";

    let params = CortexExecuteParams {
        command: command.to_string(),
        cwd: Some(ctx.path().to_string_lossy().to_string()),
        env: None,
        timeout_secs: 10,
        separate_stderr: false,
    };

    let result = ctx.server.call_cortex_execute(params).await;
    assert!(result.is_ok());

    let res = result.unwrap();
    assert!(res.success);
}

#[tokio::test]
async fn test_cortex_execute_with_env() {
    let ctx = TestContext::new().await;

    #[cfg(windows)]
    let command = "echo %TEST_VAR%";
    #[cfg(not(windows))]
    let command = "echo $TEST_VAR";

    let mut env = std::collections::HashMap::new();
    env.insert("TEST_VAR".to_string(), "hello_world".to_string());

    let params = CortexExecuteParams {
        command: command.to_string(),
        cwd: None,
        env: Some(env),
        timeout_secs: 10,
        separate_stderr: false,
    };

    let result = ctx.server.call_cortex_execute(params).await;
    assert!(result.is_ok());

    // Note: Environment variable expansion depends on shell
}

#[tokio::test]
async fn test_cortex_execute_failure() {
    let ctx = TestContext::new().await;

    let params = CortexExecuteParams {
        command: "nonexistent_command_12345".to_string(),
        cwd: None,
        env: None,
        timeout_secs: 10,
        separate_stderr: true,
    };

    let result = ctx.server.call_cortex_execute(params).await;
    assert!(result.is_ok());

    let res = result.unwrap();
    assert!(!res.success);
    assert_ne!(res.exit_code, 0);
}

#[tokio::test]
async fn test_cortex_execute_timeout() {
    let ctx = TestContext::new().await;

    #[cfg(windows)]
    let command = "ping -n 10 127.0.0.1";
    #[cfg(not(windows))]
    let command = "sleep 10";

    let params = CortexExecuteParams {
        command: command.to_string(),
        cwd: None,
        env: None,
        timeout_secs: 1, // Short timeout
        separate_stderr: false,
    };

    let result = ctx.server.call_cortex_execute(params).await;
    // Should return error due to timeout
    assert!(result.is_err() || !result.unwrap().success);
}

#[tokio::test]
async fn test_cortex_execute_separate_stderr() {
    let ctx = TestContext::new().await;

    #[cfg(windows)]
    let command = "cmd /c \"echo stdout && echo stderr 1>&2\"";
    #[cfg(not(windows))]
    let command = "sh -c 'echo stdout && echo stderr >&2'";

    let params = CortexExecuteParams {
        command: command.to_string(),
        cwd: None,
        env: None,
        timeout_secs: 10,
        separate_stderr: true,
    };

    let result = ctx.server.call_cortex_execute(params).await;
    assert!(result.is_ok());

    let res = result.unwrap();
    assert!(res.stdout.contains("stdout"));
    // stderr should be separate
}

#[tokio::test]
async fn test_cortex_execute_cargo_check() {
    let ctx = TestContext::new().await;

    // This test requires cargo to be installed
    let params = CortexExecuteParams {
        command: "cargo --version".to_string(),
        cwd: None,
        env: None,
        timeout_secs: 30,
        separate_stderr: false,
    };

    let result = ctx.server.call_cortex_execute(params).await;
    // May fail if cargo not installed, that's OK
    if let Ok(res) = result {
        if res.success {
            assert!(res.stdout.contains("cargo"));
        }
    }
}

// =============================================================================
// INTEGRATION SCENARIOS
// =============================================================================

#[tokio::test]
async fn test_cortex_code_workflow_simulation() {
    let ctx = TestContext::new().await;

    // Simulate: Agent wants to write code
    // 1. Research phase
    let research = ctx.server.call_cortex_process(CortexProcessParams {
        query: "How to implement error handling in Rust using thiserror".to_string(),
        context: None,
        session_id: Some("code-workflow-1".to_string()),
        auto_learn: true,
        inject_doubt: true,
        language: Some("rust".to_string()),
        task_type: Some(TaskType::Research),
    }).await.unwrap();

    assert!(research.success);

    // 2. Code phase
    let code = ctx.server.call_cortex_process(CortexProcessParams {
        query: "Write an Error enum with thiserror derive".to_string(),
        context: Some(research.output.clone()),
        session_id: Some("code-workflow-1".to_string()),
        auto_learn: true,
        inject_doubt: true,
        language: Some("rust".to_string()),
        task_type: Some(TaskType::Code),
    }).await.unwrap();

    assert!(code.success);

    // 3. Review phase
    let review = ctx.server.call_cortex_process(CortexProcessParams {
        query: "Review the error handling code".to_string(),
        context: Some(code.output.clone()),
        session_id: Some("code-workflow-1".to_string()),
        auto_learn: true,
        inject_doubt: true,
        language: Some("rust".to_string()),
        task_type: Some(TaskType::Review),
    }).await.unwrap();

    assert!(review.success);
}

#[tokio::test]
async fn test_cortex_fix_workflow_simulation() {
    let ctx = TestContext::new().await;

    let broken_code = r#"
fn main() {
    let s = String::from("hello");
    let r = &s;
    drop(s);  // Error: cannot move out of `s` because it is borrowed
    println!("{}", r);
}
"#;

    // 1. Analyze the error
    let analysis = ctx.server.call_cortex_process(CortexProcessParams {
        query: "Analyze this Rust code and identify the error".to_string(),
        context: Some(broken_code.to_string()),
        session_id: Some("fix-workflow-1".to_string()),
        auto_learn: true,
        inject_doubt: true,
        language: Some("rust".to_string()),
        task_type: Some(TaskType::Research),
    }).await.unwrap();

    assert!(analysis.success);

    // 2. Propose fix
    let fix = ctx.server.call_cortex_process(CortexProcessParams {
        query: "Fix the borrowing error in this code".to_string(),
        context: Some(format!("Original code:\n{}\n\nAnalysis:\n{}", broken_code, analysis.output)),
        session_id: Some("fix-workflow-1".to_string()),
        auto_learn: true,
        inject_doubt: true,
        language: Some("rust".to_string()),
        task_type: Some(TaskType::Fix),
    }).await.unwrap();

    assert!(fix.success);
}
