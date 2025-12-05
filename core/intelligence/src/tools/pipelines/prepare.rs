//! Phase B - Prepare Pipeline
//!
//! Document decisions BEFORE coding.
//! Combines: memory_store + knowledge_add_entity + knowledge_add_relation + knowledge_add_observation + user_instructions
//!
//! Workflow from instructions:
//! 1. Create .instructions.md file with DO/DON'T
//! 2. memory_store: save notes for the project
//! 3. knowledge_add_entity: if new concept
//! 4. knowledge_add_relation: if new relationship
//! 5. user_instructions: save user preferences to DB

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use crate::cortex::instructions::{InstructionCategory, UserInstruction};



/// A single item to remember
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RememberItem {
    /// Content to store
    pub content: String,
    /// Optional title/summary
    #[serde(default)]
    pub title: Option<String>,
    /// Tags for categorization
    #[serde(default)]
    pub tags: Vec<String>,
    /// Source of this information
    #[serde(default)]
    pub source: Option<String>,
    /// Category (e.g., "decision", "pattern", "warning")
    #[serde(default = "default_category")]
    pub category: String,
    /// Optional metadata
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
}

fn default_category() -> String {
    "general".to_string()
}

/// Entity definition for knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityDef {
    /// Unique name for the entity
    pub name: String,
    /// Entity type (e.g., "concept", "project", "technology", "person")
    pub entity_type: String,
    /// Initial observations about this entity
    #[serde(default)]
    pub observations: Vec<String>,
}

/// Relation definition for knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RelationDef {
    /// Source entity name
    pub from: String,
    /// Target entity name
    pub to: String,
    /// Relation type (e.g., "uses", "depends_on", "works_with", "knows")
    pub relation_type: String,
}

/// Observation to add to existing entity
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ObservationDef {
    /// Entity name to add observation to
    pub entity_name: String,
    /// Observations to add
    pub observations: Vec<String>,
}

/// User instruction to save (persisted to DB)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserInstructionDef {
    /// Instruction key (e.g., "language", "style", "workflow")
    pub key: String,
    /// Instruction value/content
    pub value: String,
    /// Category: communication, workflow, domain, coding, or custom
    #[serde(default = "default_instruction_category")]
    pub category: String,
    /// Priority (higher = applied first)
    #[serde(default)]
    pub priority: i32,
}

fn default_instruction_category() -> String {
    "communication".to_string()
}

impl UserInstructionDef {
    /// Convert to UserInstruction with user_id
    pub fn to_user_instruction(&self, user_id: &str) -> UserInstruction {
        let category = match self.category.as_str() {
            "communication" => InstructionCategory::Communication,
            "workflow" => InstructionCategory::Workflow,
            "domain" => InstructionCategory::Domain,
            "coding" => InstructionCategory::Coding,
            other => InstructionCategory::Custom(other.to_string()),
        };

        UserInstruction::new(user_id, &self.key, &self.value)
            .with_category(category)
            .with_priority(self.priority)
    }
}

/// Parameters for the prepare pipeline
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrepareParams {
    /// Items to remember (stored in semantic memory)
    #[serde(default)]
    pub remember: Vec<RememberItem>,

    /// Entities to add to knowledge graph
    #[serde(default)]
    pub entities: Vec<EntityDef>,

    /// Relations to add to knowledge graph
    #[serde(default)]
    pub relations: Vec<RelationDef>,

    /// Observations to add to existing entities
    #[serde(default)]
    pub observations: Vec<ObservationDef>,

    /// User instructions to save (persisted to DB for future sessions)
    #[serde(default)]
    pub user_instructions: Vec<UserInstructionDef>,

    /// User/session ID for user instructions (default: "default")
    #[serde(default = "default_user_id")]
    pub user_id: String,

    /// Whether to index for semantic search (default: true)
    #[serde(default = "default_true")]
    pub index: bool,

    /// Context about what this preparation is for
    #[serde(default)]
    pub context: Option<String>,
}

fn default_user_id() -> String {
    "default".to_string()
}

fn default_true() -> bool {
    true
}

/// Result of a single remember operation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RememberResult {
    /// Assigned key
    pub key: String,
    /// Whether it was indexed
    pub indexed: bool,
    /// Timestamp
    pub stored_at: i64,
}

/// Result of entity creation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityResult {
    /// Entity name
    pub name: String,
    /// Entity type
    pub entity_type: String,
    /// Whether this was a new entity (false = updated existing)
    pub created: bool,
    /// Number of observations added
    pub observations_added: usize,
}

/// Result of relation creation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RelationResult {
    /// Source entity
    pub from: String,
    /// Target entity
    pub to: String,
    /// Relation type
    pub relation_type: String,
    /// Whether this was a new relation
    pub created: bool,
}

/// Result of observation addition
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ObservationResult {
    /// Entity name
    pub entity_name: String,
    /// Number of observations added (excluding duplicates)
    pub added: usize,
}

/// Result of user instruction save
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct UserInstructionResult {
    /// Instruction key
    pub key: String,
    /// Category
    pub category: String,
    /// Whether it was saved successfully
    pub saved: bool,
    /// Whether it replaced an existing instruction
    pub replaced: bool,
}

/// Result from the prepare pipeline
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct PrepareResult {
    /// Results from remember operations
    pub remembered: Vec<RememberResult>,

    /// Results from entity operations
    pub entities_created: Vec<EntityResult>,

    /// Results from relation operations
    pub relations_created: Vec<RelationResult>,

    /// Results from observation operations
    pub observations_added: Vec<ObservationResult>,

    /// Results from user instruction operations
    #[serde(default)]
    pub user_instructions_saved: Vec<UserInstructionResult>,

    /// Total items processed
    pub total_processed: usize,

    /// Total items successfully stored
    pub total_stored: usize,

    /// Any errors encountered
    #[serde(default)]
    pub errors: Vec<String>,

    /// Summary message
    pub summary: String,
}

impl Default for PrepareParams {
    fn default() -> Self {
        Self {
            remember: Vec::new(),
            entities: Vec::new(),
            relations: Vec::new(),
            observations: Vec::new(),
            user_instructions: Vec::new(),
            user_id: "default".to_string(),
            index: true,
            context: None,
        }
    }
}

impl PrepareParams {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_prepare_params_serialization() {
        let params = PrepareParams {
            remember: vec![RememberItem {
                content: "test content".to_string(),
                title: Some("Test".to_string()),
                tags: vec!["rust".to_string()],
                source: Some("analysis".to_string()),
                category: "decision".to_string(),
                metadata: None,
            }],
            entities: vec![EntityDef {
                name: "TestEntity".to_string(),
                entity_type: "concept".to_string(),
                observations: vec!["observation 1".to_string()],
            }],
            relations: vec![RelationDef {
                from: "A".to_string(),
                to: "B".to_string(),
                relation_type: "uses".to_string(),
            }],
            observations: vec![],
            user_instructions: vec![],
            user_id: "default".to_string(),
            index: true,
            context: Some("Testing".to_string()),
        };

        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("test content"));
        assert!(json.contains("TestEntity"));

        let parsed: PrepareParams = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.remember.len(), 1);
        assert_eq!(parsed.entities.len(), 1);
    }

    #[test]
    fn test_prepare_result() {
        let result = PrepareResult {
            remembered: vec![RememberResult {
                key: "key-1".to_string(),
                indexed: true,
                stored_at: 1234567890,
            }],
            entities_created: vec![],
            relations_created: vec![],
            observations_added: vec![],
            user_instructions_saved: vec![],
            total_processed: 1,
            total_stored: 1,
            errors: vec![],
            summary: "1 item stored".to_string(),
        };

        assert!(result.errors.is_empty());
        assert_eq!(result.total_stored, 1);
    }
}
