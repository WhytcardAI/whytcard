//! Learner Module - Reflect and Improve
//!
//! The Learner analyzes execution results and:
//! - Extracts patterns from successes
//! - Extracts lessons from failures
//! - Updates memory with new knowledge
//! - Optimizes routes for future use
//! - Processes user feedback with quality metrics (from Python v2.0)

use crate::memory::TripleMemory;
use crate::error::Result;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

// ============================================================================
// WORK METRICS (ported from Python v2.0)
// Reserved for multi-agent system integration
// ============================================================================

/// Structured metrics for work quality assessment.
/// These objective metrics help evaluate and improve future work quality.
#[allow(dead_code)]
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct WorkMetrics {
    // Documentation quality
    /// Sources used: ["context7:lib", "microsoft_learn:topic"]
    #[serde(default)]
    pub sources_used: Vec<String>,

    /// Only official documentation sources used (no random StackOverflow)
    #[serde(default = "default_true")]
    pub official_docs_only: bool,

    // Implementation quality
    /// Implementation complete (no TODO/FIXME left)
    #[serde(default = "default_true")]
    pub complete: bool,

    /// Number of workarounds/hacks used
    #[serde(default)]
    pub workarounds_count: u32,

    /// Patterns followed: ["error_handling", "i18n"]
    #[serde(default)]
    pub patterns_followed: Vec<String>,

    // Validation
    /// Tests were written
    #[serde(default)]
    pub tests_written: bool,

    /// Tests are passing
    #[serde(default)]
    pub tests_passing: bool,

    /// Number of errors encountered
    #[serde(default)]
    pub errors_encountered: u32,

    /// Number of errors resolved
    #[serde(default)]
    pub errors_resolved: u32,

    // Process quality
    /// Sequential thinking used before action
    #[serde(default)]
    pub thinking_used: bool,

    /// Research done before implementing
    #[serde(default)]
    pub research_done: bool,
}

#[allow(dead_code)]
fn default_true() -> bool {
    true
}

#[allow(dead_code)]
impl WorkMetrics {
    /// Calculate objective quality score based on work metrics (0.0 to 1.0)
    ///
    /// Score breakdown:
    /// - Official documentation used: 0.25
    /// - Implementation complete (no TODOs): 0.20
    /// - Zero workarounds: 0.15
    /// - Tests passing: 0.20
    /// - All errors resolved: 0.10
    /// - Thinking/research done: 0.10
    pub fn quality_score(&self, outcome: WorkOutcome) -> f32 {
        let mut score = 0.0f32;

        // Documentation quality (0.25)
        if self.official_docs_only && !self.sources_used.is_empty() {
            score += 0.25;
        } else if !self.sources_used.is_empty() {
            score += 0.15;
        }

        // Implementation completeness (0.20)
        if self.complete {
            score += 0.20;
        }

        // No workarounds (0.15)
        match self.workarounds_count {
            0 => score += 0.15,
            1 => score += 0.05,
            _ => {}
        }

        // Tests (0.20)
        if self.tests_passing {
            score += 0.20;
        } else if self.tests_written {
            score += 0.10;
        }

        // Errors resolved (0.10)
        if self.errors_encountered > 0 {
            let resolution_rate = self.errors_resolved as f32 / self.errors_encountered as f32;
            score += 0.10 * resolution_rate;
        } else {
            score += 0.10;
        }

        // Process quality (0.10)
        if self.thinking_used {
            score += 0.05;
        }
        if self.research_done {
            score += 0.05;
        }

        // Outcome modifier
        match outcome {
            WorkOutcome::Failure => score *= 0.5,
            WorkOutcome::Partial => score *= 0.8,
            WorkOutcome::Success => {}
        }

        score.clamp(0.0, 1.0)
    }

    /// Convert quality score to human-readable grade
    pub fn score_to_grade(score: f32) -> &'static str {
        match score {
            s if s >= 0.9 => "excellent",
            s if s >= 0.75 => "good",
            s if s >= 0.5 => "acceptable",
            s if s >= 0.3 => "needs_improvement",
            _ => "poor",
        }
    }

    /// Generate actionable insights based on objective metrics
    pub fn generate_insights(&self, _outcome: WorkOutcome, quality_score: f32) -> Vec<String> {
        let mut insights = Vec::new();

        // Positive insights
        if quality_score >= 0.8 {
            insights.push("Excellent work quality - patterns reinforced for future tasks".to_string());
        }

        if self.official_docs_only && !self.sources_used.is_empty() {
            insights.push(format!(
                "Good practice: Used official documentation ({})",
                self.sources_used.iter().take(3).cloned().collect::<Vec<_>>().join(", ")
            ));
        }

        if self.complete && self.workarounds_count == 0 {
            insights.push("Solid implementation: Complete with no workarounds".to_string());
        }

        if self.tests_passing {
            insights.push("Validated: Tests are passing".to_string());
        }

        if self.errors_encountered > 0 && self.errors_resolved == self.errors_encountered {
            insights.push(format!(
                "All {} error(s) resolved - good problem-solving",
                self.errors_encountered
            ));
        }

        // Improvement suggestions
        if !self.official_docs_only {
            insights.push(
                "Improvement: Prefer official documentation sources (Context7, Microsoft Learn)"
                    .to_string(),
            );
        }

        if !self.complete {
            insights.push(
                "Improvement: Avoid leaving TODOs - complete implementation fully".to_string(),
            );
        }

        if self.workarounds_count > 0 {
            insights.push(format!(
                "Improvement: {} workaround(s) used - aim for proper solutions",
                self.workarounds_count
            ));
        }

        if self.tests_written && !self.tests_passing {
            insights.push("Improvement: Tests written but failing - fix before completion".to_string());
        }

        if !self.tests_written {
            insights.push("Improvement: Consider adding tests for validation".to_string());
        }

        if self.errors_encountered > self.errors_resolved {
            let unresolved = self.errors_encountered - self.errors_resolved;
            insights.push(format!("Warning: {} error(s) remain unresolved", unresolved));
        }

        if !self.thinking_used {
            insights.push("Process: Consider using structured thinking before implementation".to_string());
        }

        if !self.research_done {
            insights.push("Process: Research before implementing can improve quality".to_string());
        }

        insights
    }
}

/// Work outcome status
#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum WorkOutcome {
    Success,
    Partial,
    Failure,
}

impl Default for WorkOutcome {
    fn default() -> Self {
        Self::Success
    }
}

impl std::str::FromStr for WorkOutcome {
    type Err = ();

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "success" => Ok(Self::Success),
            "partial" => Ok(Self::Partial),
            "failure" | "failed" => Ok(Self::Failure),
            _ => Err(()),
        }
    }
}

// ============================================================================
// LEARNING TYPES (enhanced from Python v2.0)
// Reserved for multi-agent system integration
// ============================================================================

/// Types of learning events
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum LearningType {
    /// Successful execution
    Success,
    /// Failed execution
    Failure,
    /// Partial success
    Partial,
    /// Work completed with metrics
    WorkCompleted,
    /// Error was resolved
    ErrorResolved,
    /// Pattern discovered
    PatternDiscovered,
    /// Rule created
    RuleCreated,
    /// Anti-pattern identified
    AntiPatternCreated,
    /// Knowledge stored
    KnowledgeStored,
}

/// A learning event record
#[allow(dead_code)]
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningEvent {
    /// Unique ID
    pub id: String,

    /// Type of learning
    pub learning_type: LearningType,

    /// Original query
    pub query: String,

    /// Source of the learning
    pub source: String,

    /// Content/details
    pub content: serde_json::Value,

    /// Confidence delta (positive or negative)
    pub confidence_delta: f32,

    /// Timestamp
    pub timestamp: chrono::DateTime<chrono::Utc>,
}

#[allow(dead_code)]
impl LearningEvent {
    /// Create a new learning event
    pub fn new(
        learning_type: LearningType,
        query: impl Into<String>,
        source: impl Into<String>,
    ) -> Self {
        Self {
            id: uuid::Uuid::new_v4().to_string(),
            learning_type,
            query: query.into(),
            source: source.into(),
            content: serde_json::Value::Null,
            confidence_delta: 0.0,
            timestamp: chrono::Utc::now(),
        }
    }

    /// Add content to the event
    pub fn with_content(mut self, content: serde_json::Value) -> Self {
        self.content = content;
        self
    }

    /// Set confidence delta
    pub fn with_confidence_delta(mut self, delta: f32) -> Self {
        self.confidence_delta = delta;
        self
    }
}

// ============================================================================
// EXISTING CODE (enhanced)
// ============================================================================

/// Insight from reflection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReflectionInsight {
    /// Type of insight
    pub insight_type: InsightType,

    /// Description of the insight
    pub description: String,

    /// Confidence in this insight
    pub confidence: f32,

    /// Source of the insight
    pub source: String,

    /// Whether this should be memorized
    pub should_memorize: bool,

    /// Tags for categorization
    pub tags: Vec<String>,
}

/// Types of insights
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InsightType {
    /// A successful pattern
    SuccessPattern,
    /// A failure lesson
    FailureLesson,
    /// A workflow optimization
    WorkflowOptimization,
    /// A new fact learned
    NewFact,
    /// A route improvement
    RouteImprovement,
}

/// Outcome of the learning process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LearningOutcome {
    /// Insights extracted
    pub insights: Vec<ReflectionInsight>,

    /// Memory updates made
    pub memory_updates: Vec<MemoryUpdate>,

    /// Overall success of the execution
    pub execution_success: bool,

    /// Success rate
    pub success_rate: f32,

    /// Recommendations for next time
    pub recommendations: Vec<String>,
}

/// A memory update record
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUpdate {
    /// Memory type updated
    pub memory_type: MemoryType,

    /// Operation performed
    pub operation: UpdateOperation,

    /// Key/ID updated
    pub key: String,

    /// Description
    pub description: String,
}

/// Memory types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MemoryType {
    Semantic,
    Episodic,
    Procedural,
}

/// Update operations
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum UpdateOperation {
    Create,
    Update,
    Delete,
}

/// The Learner module
pub struct Learner {
    /// Reference to triple memory
    memory: Option<Arc<RwLock<TripleMemory>>>,

    /// Minimum confidence to memorize
    min_confidence: f32,

    /// Auto-learn enabled
    auto_learn: bool,
}

impl Learner {
    /// Create a new Learner
    pub fn new(auto_learn: bool) -> Self {
        Self {
            memory: None,
            min_confidence: 0.7,
            auto_learn,
        }
    }

    /// Set the memory reference
    pub fn with_memory(mut self, memory: Arc<RwLock<TripleMemory>>) -> Self {
        self.memory = Some(memory);
        self
    }

    /// Reflect on an execution result
    pub async fn reflect(
        &self,
        execution: &super::executor::ExecutionResult,
        perception: &super::perceiver::PerceptionResult,
    ) -> Result<LearningOutcome> {
        let mut insights = Vec::new();
        let mut memory_updates = Vec::new();
        let mut recommendations = Vec::new();

        // Analyze outcome
        let success_rate = execution.success_rate();
        let execution_success = execution.success;

        // Extract success patterns
        if execution_success {
            let pattern = self.extract_success_pattern(execution, perception);
            insights.push(pattern);

            // Recommend continuing with this approach
            recommendations.push(format!(
                "Pattern effective for {:?} tasks - reuse in similar contexts",
                perception.intent
            ));
        }

        // Extract failure lessons
        for step_result in &execution.step_results {
            if !step_result.success {
                let lesson = self.extract_failure_lesson(step_result);
                insights.push(lesson);
            }
        }

        // Extract adjustments as insights
        for adjustment in &execution.adjustments {
            insights.push(ReflectionInsight {
                insight_type: InsightType::WorkflowOptimization,
                description: adjustment.clone(),
                confidence: 0.8,
                source: execution.plan_id.clone(),
                should_memorize: false,
                tags: vec!["adjustment".to_string()],
            });
        }

        // Update memory if auto_learn is enabled
        if self.auto_learn {
            if let Some(memory) = &self.memory {
                memory_updates = self.update_memory(memory, &insights, execution, perception).await?;
            }
        }

        // Generate recommendations
        if !execution_success {
            recommendations.push("Consider adding more research steps for this task type".to_string());
            if execution.failed_steps > execution.successful_steps {
                recommendations.push("Task complexity may require breaking into smaller steps".to_string());
            }
        }

        if execution.adjustments.len() > 2 {
            recommendations.push("Multiple adjustments suggest need for better initial planning".to_string());
        }

        Ok(LearningOutcome {
            insights,
            memory_updates,
            execution_success,
            success_rate,
            recommendations,
        })
    }

    /// Extract a success pattern
    fn extract_success_pattern(
        &self,
        execution: &super::executor::ExecutionResult,
        perception: &super::perceiver::PerceptionResult,
    ) -> ReflectionInsight {
        let labels: Vec<String> = perception.labels.iter().map(|l| l.as_str().to_string()).collect();

        ReflectionInsight {
            insight_type: InsightType::SuccessPattern,
            description: format!(
                "Successfully completed {:?} task with {} steps in {}ms",
                perception.intent,
                execution.successful_steps,
                execution.total_duration_ms
            ),
            confidence: execution.success_rate(),
            source: execution.plan_id.clone(),
            should_memorize: execution.success_rate() >= self.min_confidence,
            tags: labels,
        }
    }

    /// Extract a failure lesson
    fn extract_failure_lesson(&self, step_result: &super::executor::StepResult) -> ReflectionInsight {
        ReflectionInsight {
            insight_type: InsightType::FailureLesson,
            description: format!(
                "Step {} failed: {}",
                step_result.step_id,
                step_result.error.as_deref().unwrap_or("Unknown error")
            ),
            confidence: 0.9, // Failures are usually reliable data
            source: step_result.step_id.clone(),
            should_memorize: step_result.retries_used > 0, // Memorize persistent failures
            tags: vec!["failure".to_string(), "lesson".to_string()],
        }
    }

    /// Update memory with insights
    async fn update_memory(
        &self,
        memory: &Arc<RwLock<TripleMemory>>,
        insights: &[ReflectionInsight],
        execution: &super::executor::ExecutionResult,
        perception: &super::perceiver::PerceptionResult,
    ) -> Result<Vec<MemoryUpdate>> {
        let mut updates = Vec::new();
        let mem = memory.read().await;

        // Record episodic event
        {
            use crate::memory::episodic::{Episode, EpisodeType};

            let episode_type = if execution.success {
                EpisodeType::Decision
            } else {
                EpisodeType::Error
            };

            let episode = Episode::new(episode_type, format!(
                "{:?} task: {} (success_rate: {:.1}%)",
                perception.intent,
                perception.query,
                execution.success_rate() * 100.0
            )).with_context(serde_json::json!({
                "labels": perception.labels.iter().map(|l| l.as_str()).collect::<Vec<_>>(),
                "duration_ms": execution.total_duration_ms,
                "steps": execution.successful_steps + execution.failed_steps,
            }));

            let episodic = mem.episodic.write().await;
            let episode_id = episodic.record(episode).await?;
            drop(episodic);

            updates.push(MemoryUpdate {
                memory_type: MemoryType::Episodic,
                operation: UpdateOperation::Create,
                key: episode_id,
                description: "Recorded execution event".to_string(),
            });
        }

        // Store semantic facts from high-confidence insights
        for insight in insights.iter().filter(|i| i.should_memorize && i.confidence >= self.min_confidence) {
            use crate::memory::semantic::SemanticFact;

            let fact = SemanticFact {
                id: None,
                content: insight.description.clone(),
                source: Some(insight.source.clone()),
                category: format!("{:?}", insight.insight_type),
                tags: insight.tags.clone(),
                relevance_score: insight.confidence,
            };

            let mut semantic = mem.semantic.write().await;
            let fact_id = semantic.store(fact).await?;
            drop(semantic);

            updates.push(MemoryUpdate {
                memory_type: MemoryType::Semantic,
                operation: UpdateOperation::Create,
                key: fact_id,
                description: format!("Stored {:?} insight", insight.insight_type),
            });
        }

        // Update procedural memory for successful patterns
        if execution.success && execution.success_rate() >= 0.8 {
            let labels: Vec<String> = perception.labels.iter().map(|l| l.as_str().to_string()).collect();
            let condition = labels.join("|");

            let mut procedural = mem.procedural.write().await;
            let rule_id = procedural.add_rule(
                format!("{:?}_pattern", perception.intent),
                condition,
                format!("apply_{:?}_workflow", perception.intent),
                execution.success_rate(),
            )?;
            drop(procedural);

            updates.push(MemoryUpdate {
                memory_type: MemoryType::Procedural,
                operation: UpdateOperation::Create,
                key: rule_id,
                description: "Created successful workflow rule".to_string(),
            });
        }

        Ok(updates)
    }

    /// Provide feedback to update confidence
    pub async fn provide_feedback(
        &self,
        rule_id: &str,
        success: bool,
    ) -> Result<f32> {
        if let Some(memory) = &self.memory {
            let mem = memory.read().await;
            let mut procedural = mem.procedural.write().await;
            return procedural.update_confidence(rule_id, success);
        }

        Ok(0.0)
    }

    /// Process work feedback with objective metrics (ported from Python v2.0)
    ///
    /// This method evaluates work quality based on objective metrics rather
    /// than subjective sentiment. It updates memory and generates insights.
    #[allow(dead_code)]
    pub async fn process_work_feedback(
        &self,
        query: &str,
        outcome: WorkOutcome,
        metrics: WorkMetrics,
        notes: Option<&str>,
    ) -> Result<LearningEvent> {
        // Calculate objective quality score
        let quality_score = metrics.quality_score(outcome);
        let grade = WorkMetrics::score_to_grade(quality_score);

        // Determine learning type based on outcome and score
        let learning_type = if outcome == WorkOutcome::Failure || quality_score < 0.3 {
            LearningType::Failure
        } else if outcome == WorkOutcome::Success && quality_score >= 0.7 {
            LearningType::Success
        } else {
            LearningType::Partial
        };

        // Calculate confidence delta based on quality score
        let confidence_delta = if quality_score >= 0.5 {
            0.05 * quality_score  // Increment
        } else {
            -0.03 * (1.0 - quality_score)  // Decrement
        };

        // Generate insights
        let insights = metrics.generate_insights(outcome, quality_score);

        // Create learning event
        let event = LearningEvent::new(learning_type.clone(), query, "work_feedback")
            .with_content(serde_json::json!({
                "outcome": format!("{:?}", outcome),
                "quality_score": quality_score,
                "grade": grade,
                "metrics": {
                    "sources_used": metrics.sources_used,
                    "official_docs_only": metrics.official_docs_only,
                    "complete": metrics.complete,
                    "workarounds_count": metrics.workarounds_count,
                    "patterns_followed": metrics.patterns_followed,
                    "tests_written": metrics.tests_written,
                    "tests_passing": metrics.tests_passing,
                    "errors_encountered": metrics.errors_encountered,
                    "errors_resolved": metrics.errors_resolved,
                    "thinking_used": metrics.thinking_used,
                    "research_done": metrics.research_done,
                },
                "notes": notes,
                "insights": insights,
            }))
            .with_confidence_delta(confidence_delta);

        // Update memory if available
        if self.auto_learn {
            if let Some(memory) = &self.memory {
                self.update_memory_from_work_feedback(memory, query, &metrics, quality_score, outcome).await?;
            }
        }

        tracing::info!(
            "work_feedback_processed: outcome={:?}, quality_score={:.2}, grade={}, insights={}",
            outcome,
            quality_score,
            grade,
            insights.len()
        );

        Ok(event)
    }

    /// Update memory based on work feedback
    #[allow(dead_code)]
    async fn update_memory_from_work_feedback(
        &self,
        memory: &Arc<RwLock<TripleMemory>>,
        query: &str,
        metrics: &WorkMetrics,
        quality_score: f32,
        outcome: WorkOutcome,
    ) -> Result<()> {
        let mem = memory.read().await;

        // Record episodic event
        {
            use crate::memory::episodic::{Episode, EpisodeType};

            let episode = Episode::new(
                EpisodeType::Feedback,
                format!("Work completed: {}", &query[..query.len().min(100)]),
            )
            .with_context(serde_json::json!({
                "outcome": format!("{:?}", outcome),
                "quality_score": quality_score,
                "official_docs_used": metrics.official_docs_only,
                "complete": metrics.complete,
                "workarounds": metrics.workarounds_count,
                "tests_passing": metrics.tests_passing,
                "errors_resolved": metrics.errors_resolved,
            }));

            let episodic = mem.episodic.write().await;
            let _ = episodic.record(episode).await;
        }

        // Store semantic fact if high quality
        if quality_score >= 0.7 {
            use crate::memory::semantic::SemanticFact;

            let fact = SemanticFact {
                id: None,
                content: format!(
                    "High-quality work pattern: {} (score: {:.2})",
                    &query[..query.len().min(100)],
                    quality_score
                ),
                source: Some("work_feedback".to_string()),
                category: "quality_pattern".to_string(),
                tags: vec!["high_quality".to_string(), "validated".to_string()],
                relevance_score: quality_score,
            };

            let mut semantic = mem.semantic.write().await;
            let _ = semantic.store(fact).await;
        }

        // Create anti-pattern if workarounds were used
        if metrics.workarounds_count > 0 && outcome != WorkOutcome::Success {
            let mut procedural = mem.procedural.write().await;
            let _ = procedural.add_rule(
                format!("anti_pattern_{}", &uuid::Uuid::new_v4().to_string()[..8]),
                format!("workaround|hack|bypass|{}", &query[..query.len().min(20)]),
                "avoid_workarounds".to_string(),
                0.3, // Low confidence for anti-patterns
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_reflection_insight() {
        let insight = ReflectionInsight {
            insight_type: InsightType::SuccessPattern,
            description: "Test pattern".to_string(),
            confidence: 0.95,
            source: "test".to_string(),
            should_memorize: true,
            tags: vec!["test".to_string()],
        };

        assert!(insight.should_memorize);
        assert!(insight.confidence > 0.9);
    }

    #[test]
    fn test_learner_new() {
        let learner = Learner::new(true);
        assert!(learner.auto_learn);
        assert!(learner.memory.is_none());
    }
}
