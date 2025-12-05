//! Sequential Thinking MCP Client
//!
//! Client for the sequential-thinking MCP server that helps with
//! complex problem decomposition and multi-step reasoning.

use crate::error::Result;
use serde::{Deserialize, Serialize};

/// Parameters for sequential thinking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingParams {
    /// Current thinking step
    pub thought: String,

    /// Whether another thought step is needed
    pub next_thought_needed: bool,

    /// Current thought number
    pub thought_number: u32,

    /// Estimated total thoughts needed
    pub total_thoughts: u32,

    /// Whether this revises previous thinking
    #[serde(default)]
    pub is_revision: bool,

    /// Which thought is being reconsidered
    #[serde(skip_serializing_if = "Option::is_none")]
    pub revises_thought: Option<u32>,

    /// Branching point thought number
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_from_thought: Option<u32>,

    /// Branch identifier
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch_id: Option<String>,

    /// If more thoughts are needed
    #[serde(default)]
    pub needs_more_thoughts: bool,
}

/// Result from sequential thinking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThinkingResult {
    /// All thinking steps
    pub thoughts: Vec<ThoughtStep>,

    /// Final conclusion/answer
    pub conclusion: Option<String>,

    /// Whether reasoning is complete
    pub complete: bool,

    /// Total steps taken
    pub total_steps: u32,
}

/// A single thought step
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThoughtStep {
    /// Step number
    pub number: u32,

    /// The thought content
    pub content: String,

    /// Whether this was a revision
    pub is_revision: bool,

    /// Branch ID if branched
    pub branch_id: Option<String>,
}

/// Sequential Thinking Client
///
/// Provides structured thinking capabilities for complex problems.
/// This client implements the sequential-thinking pattern without
/// requiring an external MCP server - it can work standalone or
/// be used to format calls to the MCP server.
pub struct SequentialThinkingClient {
    /// Current session thoughts
    thoughts: Vec<ThoughtStep>,

    /// Current thought number
    current_number: u32,

    /// Whether thinking is complete
    complete: bool,
}

impl SequentialThinkingClient {
    /// Create a new client
    pub fn new() -> Self {
        Self {
            thoughts: Vec::new(),
            current_number: 0,
            complete: false,
        }
    }

    /// Start a new thinking session
    pub fn start_session(&mut self) {
        self.thoughts.clear();
        self.current_number = 0;
        self.complete = false;
    }

    /// Add a thinking step
    pub fn add_thought(&mut self, params: ThinkingParams) -> Result<ThoughtStep> {
        let step = ThoughtStep {
            number: params.thought_number,
            content: params.thought,
            is_revision: params.is_revision,
            branch_id: params.branch_id,
        };

        self.thoughts.push(step.clone());
        self.current_number = params.thought_number;
        self.complete = !params.next_thought_needed;

        Ok(step)
    }

    /// Get all thoughts in the current session
    pub fn get_thoughts(&self) -> &[ThoughtStep] {
        &self.thoughts
    }

    /// Check if thinking is complete
    pub fn is_complete(&self) -> bool {
        self.complete
    }

    /// Get the final result
    pub fn get_result(&self) -> ThinkingResult {
        let conclusion = if self.complete && !self.thoughts.is_empty() {
            Some(self.thoughts.last().map(|t| t.content.clone()).unwrap_or_default())
        } else {
            None
        };

        ThinkingResult {
            thoughts: self.thoughts.clone(),
            conclusion,
            complete: self.complete,
            total_steps: self.current_number,
        }
    }

    /// Create initial thinking params for a problem
    pub fn create_initial_params(problem: &str, estimated_steps: u32) -> ThinkingParams {
        ThinkingParams {
            thought: format!("Analyzing problem: {}", problem),
            next_thought_needed: true,
            thought_number: 1,
            total_thoughts: estimated_steps,
            is_revision: false,
            revises_thought: None,
            branch_from_thought: None,
            branch_id: None,
            needs_more_thoughts: false,
        }
    }

    /// Create a continuation params
    pub fn create_continuation(
        &self,
        thought: impl Into<String>,
        needs_more: bool,
    ) -> ThinkingParams {
        ThinkingParams {
            thought: thought.into(),
            next_thought_needed: needs_more,
            thought_number: self.current_number + 1,
            total_thoughts: self.current_number + if needs_more { 2 } else { 1 },
            is_revision: false,
            revises_thought: None,
            branch_from_thought: None,
            branch_id: None,
            needs_more_thoughts: false,
        }
    }

    /// Create a revision params
    pub fn create_revision(
        &self,
        thought: impl Into<String>,
        revises: u32,
    ) -> ThinkingParams {
        ThinkingParams {
            thought: thought.into(),
            next_thought_needed: true,
            thought_number: self.current_number + 1,
            total_thoughts: self.current_number + 2,
            is_revision: true,
            revises_thought: Some(revises),
            branch_from_thought: None,
            branch_id: None,
            needs_more_thoughts: false,
        }
    }

    /// Format params for MCP call
    pub fn format_for_mcp(params: &ThinkingParams) -> serde_json::Value {
        serde_json::json!({
            "thought": params.thought,
            "nextThoughtNeeded": params.next_thought_needed,
            "thoughtNumber": params.thought_number,
            "totalThoughts": params.total_thoughts,
            "isRevision": params.is_revision,
            "revisesThought": params.revises_thought,
            "branchFromThought": params.branch_from_thought,
            "branchId": params.branch_id,
            "needsMoreThoughts": params.needs_more_thoughts
        })
    }

    /// Decompose a problem into thinking steps (helper method)
    pub async fn decompose_problem(&mut self, problem: &str) -> Result<ThinkingResult> {
        self.start_session();

        // Step 1: Understand the problem
        let params1 = Self::create_initial_params(problem, 5);
        self.add_thought(params1)?;

        // Step 2: Identify key components
        let params2 = self.create_continuation(
            format!("Identifying key components in: {}", problem),
            true,
        );
        self.add_thought(params2)?;

        // Step 3: Plan approach
        let params3 = self.create_continuation(
            "Planning solution approach based on identified components",
            true,
        );
        self.add_thought(params3)?;

        // Step 4: Consider edge cases
        let params4 = self.create_continuation(
            "Considering edge cases and potential issues",
            true,
        );
        self.add_thought(params4)?;

        // Step 5: Synthesize solution
        let params5 = self.create_continuation(
            "Synthesizing final solution from analysis",
            false,
        );
        self.add_thought(params5)?;

        Ok(self.get_result())
    }
}

impl Default for SequentialThinkingClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_initial_params() {
        let params = SequentialThinkingClient::create_initial_params("Test problem", 5);
        assert_eq!(params.thought_number, 1);
        assert_eq!(params.total_thoughts, 5);
        assert!(params.next_thought_needed);
    }

    #[test]
    fn test_thinking_session() {
        let mut client = SequentialThinkingClient::new();
        client.start_session();

        let params1 = SequentialThinkingClient::create_initial_params("Test", 3);
        client.add_thought(params1).unwrap();

        let params2 = client.create_continuation("Step 2", true);
        client.add_thought(params2).unwrap();

        let params3 = client.create_continuation("Final step", false);
        client.add_thought(params3).unwrap();

        assert!(client.is_complete());
        assert_eq!(client.get_thoughts().len(), 3);
    }

    #[test]
    fn test_format_for_mcp() {
        let params = ThinkingParams {
            thought: "Test thought".into(),
            next_thought_needed: true,
            thought_number: 1,
            total_thoughts: 5,
            is_revision: false,
            revises_thought: None,
            branch_from_thought: None,
            branch_id: None,
            needs_more_thoughts: false,
        };

        let json = SequentialThinkingClient::format_for_mcp(&params);
        assert_eq!(json["thoughtNumber"], 1);
        assert_eq!(json["nextThoughtNeeded"], true);
    }
}
