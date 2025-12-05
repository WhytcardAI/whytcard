//! CORTEX Engine - Main Orchestrator
//!
//! The CortexEngine is the main entry point that orchestrates:
//! 1. Perception - Analyze and understand input
//! 2. Cognition - Retrieve memory and plan
//! 3. Action - Execute with adaptation
//! 4. Reflection - Learn and improve

use crate::error::Result;
use crate::memory::TripleMemory;
use crate::paths::DataPaths;
use super::{
    CortexConfig,
    perceiver::{Perceiver, PerceptionResult},
    executor::{Executor, ExecutionPlan},
    learner::Learner,
    context::{ContextManager, ActiveContext},
    instructions::InstructionsManager,
};
use serde::{Deserialize, Serialize};
use std::path::Path;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Result of a CORTEX process call
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CortexResult {
    /// Whether the operation succeeded
    pub success: bool,

    /// Main result/output
    pub result: serde_json::Value,

    /// Perception analysis
    pub perception: PerceptionResult,

    /// Execution metrics
    pub execution: ExecutionMetrics,

    /// Learning insights
    pub insights: Vec<String>,

    /// Confidence score
    pub confidence: f32,

    /// Suggested next actions
    pub next_actions: Vec<String>,
}

/// Execution metrics summary
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionMetrics {
    /// Total duration in milliseconds
    pub duration_ms: u64,

    /// Number of steps executed
    pub steps_executed: usize,

    /// Success rate
    pub success_rate: f32,

    /// Research was performed
    pub research_performed: bool,

    /// Adjustments made
    pub adjustments: usize,
}

/// The main CORTEX Engine
pub struct CortexEngine {
    /// Configuration (reserved for future use)
    #[allow(dead_code)]
    config: CortexConfig,

    /// Triple memory system
    memory: Arc<RwLock<TripleMemory>>,

    /// Perceiver module
    perceiver: Perceiver,

    /// Executor module
    executor: Executor,

    /// Learner module
    learner: Learner,

    /// Context manager
    context: RwLock<ContextManager>,

    /// Instructions manager for loading .instructions.md files
    instructions: RwLock<InstructionsManager>,

    /// Whether initialized
    initialized: bool,
}

impl CortexEngine {
    /// Create a new CORTEX engine
    pub async fn new(data_path: &Path, config: CortexConfig) -> Result<Self> {
        // Create DataPaths from the path
        let paths = DataPaths::from_root(data_path.to_path_buf());

        // Initialize triple memory
        let memory = TripleMemory::new(&paths).await?;
        let memory = Arc::new(RwLock::new(memory));

        // Create modules
        let perceiver = Perceiver::new(config.research_threshold);
        let executor = Executor::new(config.max_execution_steps);
        let learner = Learner::new(config.auto_learn).with_memory(Arc::clone(&memory));
        let context = RwLock::new(ContextManager::new());

        // Initialize instructions manager
        let mut instructions_mgr = InstructionsManager::new();

        // Try to load instructions from workspace
        // Look for .github/instructions relative to data_path's parent
        if let Some(workspace) = data_path.parent() {
            if let Err(e) = instructions_mgr.load_from_workspace(workspace) {
                tracing::warn!("Failed to load instructions: {}", e);
            } else {
                tracing::info!("Loaded {} instruction files", instructions_mgr.count());
            }
        }

        Ok(Self {
            config,
            memory,
            perceiver,
            executor,
            learner,
            context,
            instructions: RwLock::new(instructions_mgr),
            initialized: true,
        })
    }

    /// Process a query through the full CORTEX pipeline
    pub async fn process(&self, query: &str, _context: Option<serde_json::Value>) -> Result<CortexResult> {
        let start_time = std::time::Instant::now();

        tracing::info!("CORTEX processing: {}", query);

        // 1. PERCEPTION - Analyze and understand (without aggregated context initially)
        let perception = self.perceiver.analyze_simple(query);
        tracing::debug!("Perception: intent={:?}, confidence={}", perception.intent, perception.confidence);

        // 2. COGNITION - Memory retrieval and planning
        let plan = self.cognition(&perception).await?;
        let research_performed = perception.needs_research;
        tracing::debug!("Plan created: {} steps", plan.steps.len());

        // 3. ACTION - Execute with OODA
        let execution = self.executor.execute(plan).await?;
        tracing::debug!("Execution: success={}, steps={}", execution.success, execution.successful_steps);

        // 4. REFLECTION - Learn and improve
        let learning = self.learner.reflect(&execution, &perception).await?;
        tracing::debug!("Learning: {} insights, {} memory updates", learning.insights.len(), learning.memory_updates.len());

        // Record in context
        {
            let mut ctx = self.context.write().await;
            ctx.record_query(query, perception.intent.as_str(), execution.success);
        }

        // Build result
        let execution_output = execution.output.clone();
        let result = CortexResult {
            success: execution.success,
            result: execution_output.unwrap_or(serde_json::json!({
                "message": if execution.success { "Task completed successfully" } else { "Task encountered issues" },
                "steps_completed": execution.successful_steps,
            })),
            perception,
            execution: ExecutionMetrics {
                duration_ms: start_time.elapsed().as_millis() as u64,
                steps_executed: execution.successful_steps + execution.failed_steps,
                success_rate: execution.success_rate(),
                research_performed,
                adjustments: execution.adjustments.len(),
            },
            insights: learning.insights.iter().map(|i| i.description.clone()).collect(),
            confidence: learning.success_rate,
            next_actions: learning.recommendations,
        };

        Ok(result)
    }

    /// Cognition phase - retrieve memory and create plan
    async fn cognition(&self, perception: &PerceptionResult) -> Result<ExecutionPlan> {
        let memory = self.memory.read().await;

        // Search semantic memory for relevant knowledge
        let mut semantic = memory.semantic.write().await;
        let relevant = semantic.search(&perception.query, 5, Some(0.5)).await?;
        drop(semantic);
        tracing::debug!("Found {} relevant semantic memories", relevant.len());

        // Get applicable procedural rules
        let context_json = serde_json::json!({
            "intent": perception.intent.as_str(),
            "labels": perception.labels.iter().map(|l| l.as_str()).collect::<Vec<_>>(),
        });
        let procedural = memory.procedural.read().await;
        let rules = procedural.get_applicable_rules(&context_json);
        tracing::debug!("Found {} applicable rules", rules.len());

        // Get routing recommendation
        let routing = procedural.get_routing(&perception.query);
        if let Some(ref r) = routing {
            tracing::debug!("Routing recommendation: {} (confidence: {})", r.target_agent, r.confidence);
        }
        drop(procedural);

        // Drop read lock before creating plan
        drop(memory);

        // Create plan based on perception and memory
        let mut plan = self.executor.create_plan_from_perception(perception);

        // Enrich plan with memory context
        plan = plan.with_metadata("relevant_facts", serde_json::json!(relevant.len()));
        plan = plan.with_metadata("rules_applied", serde_json::json!(rules.len()));
        if let Some(r) = routing {
            plan = plan.with_metadata("routing", serde_json::json!(r));
        }

        Ok(plan)
    }

    /// Get the current context
    pub async fn get_context(&self) -> ActiveContext {
        self.context.read().await.get_context().clone()
    }

    /// Start a new session
    pub async fn start_session(&self, workspace: Option<std::path::PathBuf>) -> Result<String> {
        let mut ctx = self.context.write().await;
        let session_id = ctx.start_session();

        if let Some(ws) = workspace {
            ctx.set_workspace(ws);
        }

        // Also start episodic memory session
        let memory = self.memory.read().await;
        let mut episodic = memory.episodic.write().await;
        episodic.start_session(ctx.get_workspace().map(|p| p.to_string_lossy().to_string())).await?;

        Ok(session_id)
    }

    /// End the current session
    pub async fn end_session(&self) -> Result<()> {
        let mut ctx = self.context.write().await;
        ctx.end_session();

        let memory = self.memory.read().await;
        let mut episodic = memory.episodic.write().await;
        episodic.end_session().await?;

        Ok(())
    }

    /// Get memory statistics
    pub async fn get_stats(&self) -> serde_json::Value {
        let mem = self.memory.read().await;
        let stats = mem.get_stats().await;

        serde_json::json!({
            "semantic": {
                "total_facts": stats.semantic.total_facts,
                "initialized": stats.semantic.initialized,
            },
            "episodic": {
                "total_episodes": stats.episodic.total_episodes,
                "current_session": stats.episodic.current_session,
                "by_type": stats.episodic.episodes_by_type,
            },
            "procedural": {
                "total_rules": stats.procedural.total_rules,
                "total_patterns": stats.procedural.total_patterns,
                "total_routing": stats.procedural.total_routing,
                "avg_confidence": stats.procedural.average_confidence,
            },
            "initialized": self.initialized,
        })
    }

    /// Provide feedback for learning
    pub async fn provide_feedback(&self, rule_id: &str, success: bool) -> Result<f32> {
        self.learner.provide_feedback(rule_id, success).await
    }

    /// Cleanup old data
    pub async fn cleanup(&self, retention_days: i64) -> Result<usize> {
        let memory = self.memory.read().await;
        let episodic = memory.episodic.read().await;
        episodic.cleanup_old(retention_days).await
    }

    /// Search episodic memory
    pub async fn search_episodic(&self, query: &str, limit: usize) -> Result<Vec<crate::memory::episodic::StoredEpisode>> {
        let memory = self.memory.read().await;
        let episodic = memory.episodic.read().await;
        episodic.search(query, None, limit).await
    }

    /// Search procedural memory (rules)
    pub async fn search_procedural(&self, query: &str, limit: usize) -> Result<Vec<ProceduralRuleResult>> {
        let memory = self.memory.read().await;
        let procedural = memory.procedural.read().await;

        // Get applicable rules matching the query
        let context = serde_json::json!({ "query": query });
        let rules = procedural.get_applicable_rules(&context);

        // Convert to result format
        let results: Vec<ProceduralRuleResult> = rules
            .into_iter()
            .take(limit)
            .map(|r| ProceduralRuleResult {
                id: r.id,
                trigger: r.condition,
                action: r.action,
                confidence: r.confidence,
                usage_count: r.success_count + r.failure_count,
            })
            .collect();

        Ok(results)
    }

    // ========================================================================
    // INSTRUCTIONS MANAGEMENT
    // ========================================================================

    /// Load instructions from a workspace directory
    pub async fn load_instructions(&self, workspace: &Path) -> Result<usize> {
        let mut instructions = self.instructions.write().await;
        instructions.load_from_workspace(workspace)
    }

    /// Reload instructions from the configured directory
    pub async fn reload_instructions(&self) -> Result<usize> {
        let mut instructions = self.instructions.write().await;
        instructions.reload()
    }

    /// Add a user instruction (persisted separately, takes priority over file instructions)
    pub async fn add_user_instruction(&self, instruction: super::instructions::UserInstruction) {
        let mut instructions = self.instructions.write().await;
        instructions.add_user_instruction(instruction);
    }

    /// Set current user ID for instruction filtering
    pub async fn set_user(&self, user_id: impl Into<String>) {
        let mut instructions = self.instructions.write().await;
        instructions.set_user(user_id);
    }

    /// Load user instructions from DB (call at session start)
    pub async fn load_user_instructions(&self, user_instructions: Vec<super::instructions::UserInstruction>) {
        let mut instructions = self.instructions.write().await;
        instructions.add_user_instructions(user_instructions);
    }

    /// Get user instructions for export/save
    pub async fn get_user_instructions(&self) -> Vec<super::instructions::UserInstruction> {
        let instructions = self.instructions.read().await;
        instructions.get_user_instructions().to_vec()
    }

    /// Get instructions that apply to a specific file
    pub async fn get_instructions_for_file(&self, file_path: &str) -> Vec<super::instructions::Instruction> {
        let instructions = self.instructions.read().await;
        instructions.for_file(file_path).into_iter().cloned().collect()
    }

    /// Get all global instructions (applyTo: **)
    pub async fn get_global_instructions(&self) -> Vec<super::instructions::Instruction> {
        let instructions = self.instructions.read().await;
        instructions.global().into_iter().cloned().collect()
    }

    /// Get instructions as prompt context
    ///
    /// This generates a formatted string suitable for injection into LLM prompts.
    /// If a file_path is provided, only instructions that apply to that file are included.
    pub async fn get_instructions_prompt(&self, file_path: Option<&str>) -> String {
        let instructions = self.instructions.read().await;
        instructions.to_prompt_context(file_path)
    }

    /// Get instruction content by name
    pub async fn get_instruction_content(&self, name: &str) -> Option<String> {
        let instructions = self.instructions.read().await;
        instructions.get_content(name).map(|s| s.to_string())
    }

    /// Get instructions statistics
    pub async fn get_instructions_stats(&self) -> serde_json::Value {
        let instructions = self.instructions.read().await;
        let stats = instructions.stats();

        serde_json::json!({
            "total": stats.total,
            "from_files": stats.from_files,
            "from_user": stats.from_user,
            "current_user": stats.current_user,
            "loaded": instructions.is_loaded(),
            "instructions": instructions.all().iter().map(|i| serde_json::json!({
                "name": i.name,
                "description": i.description,
                "apply_to": i.apply_to,
                "source": format!("{:?}", i.source),
            })).collect::<Vec<_>>(),
        })
    }
}

/// Result of a procedural rule search
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProceduralRuleResult {
    pub id: String,
    pub trigger: String,
    pub action: String,
    pub confidence: f32,
    pub usage_count: i32,
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[tokio::test]
    async fn test_cortex_engine_creation() {
        let temp = tempdir().unwrap();
        let engine = CortexEngine::new(temp.path(), CortexConfig::default()).await;
        assert!(engine.is_ok());
    }
}
