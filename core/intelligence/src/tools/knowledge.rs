//! Knowledge graph tools for MCP
//!
//! Tools for managing entities and relations in the knowledge graph.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Parameters for knowledge_add_entity tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeAddEntityParams {
    /// Entity name (unique identifier)
    pub name: String,

    /// Entity type (e.g., "person", "concept", "project")
    pub entity_type: String,

    /// Initial observations about this entity
    #[serde(default)]
    pub observations: Vec<String>,
}

/// Result from knowledge_add_entity
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeAddEntityResult {
    /// Entity name
    pub name: String,

    /// Entity type
    pub entity_type: String,

    /// Whether this was a new entity or existing
    pub created: bool,

    /// Number of observations added
    pub observations_added: usize,
}

/// Parameters for knowledge_add_observation tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeAddObservationParams {
    /// Entity name to add observation to
    pub entity_name: String,

    /// Observations to add
    pub observations: Vec<String>,
}

/// Result from knowledge_add_observation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeAddObservationResult {
    /// Entity name
    pub entity_name: String,

    /// Number of observations added (excluding duplicates)
    pub added: usize,
}

/// Parameters for knowledge_add_relation tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeAddRelationParams {
    /// Source entity name
    pub from: String,

    /// Target entity name
    pub to: String,

    /// Relation type (e.g., "works_with", "knows", "depends_on")
    pub relation_type: String,
}

/// Result from knowledge_add_relation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeAddRelationResult {
    /// Source entity
    pub from: String,

    /// Target entity
    pub to: String,

    /// Relation type
    pub relation_type: String,

    /// Whether this was a new relation
    pub created: bool,
}

/// Parameters for knowledge_search tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeSearchParams {
    /// Search query
    pub query: String,

    /// Maximum entities to return
    #[serde(default = "default_limit")]
    pub limit: usize,
}

/// Result from knowledge_search
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeSearchResult {
    /// Matching entities
    pub entities: Vec<EntityInfo>,

    /// Relations between matching entities
    pub relations: Vec<RelationInfo>,
}

/// Entity information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EntityInfo {
    /// Entity name
    pub name: String,

    /// Entity type
    pub entity_type: String,

    /// Observations
    pub observations: Vec<String>,
}

/// Relation information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RelationInfo {
    /// Source entity
    pub from: String,

    /// Target entity
    pub to: String,

    /// Relation type
    pub relation_type: String,
}

/// Parameters for knowledge_get_entity tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeGetEntityParams {
    /// Entity name to retrieve
    pub name: String,

    /// Include related entities
    #[serde(default)]
    pub include_relations: bool,
}

/// Result from knowledge_get_entity
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeGetEntityResult {
    /// Entity information
    pub entity: EntityInfo,

    /// Outgoing relations (if requested)
    pub outgoing: Vec<RelationInfo>,

    /// Incoming relations (if requested)
    pub incoming: Vec<RelationInfo>,
}

/// Parameters for knowledge_delete_entity tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeDeleteEntityParams {
    /// Entity names to delete
    pub names: Vec<String>,
}

/// Result from knowledge_delete_entity
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeDeleteEntityResult {
    /// Names that were deleted
    pub deleted: Vec<String>,

    /// Number of relations removed
    pub relations_removed: usize,
}

/// Parameters for knowledge_delete_relation tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeDeleteRelationParams {
    /// Relations to delete
    pub relations: Vec<RelationDeleteSpec>,
}

/// Specification for a relation to delete
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RelationDeleteSpec {
    /// Source entity
    pub from: String,

    /// Target entity
    pub to: String,

    /// Relation type
    pub relation_type: String,
}

/// Result from knowledge_delete_relation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeDeleteRelationResult {
    /// Number of relations deleted
    pub deleted: usize,
}

/// Parameters for knowledge_read_graph tool (read entire graph)
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeReadGraphParams {
    /// Maximum entities to return (0 = all)
    #[serde(default)]
    pub limit: usize,
}

/// Result from knowledge_read_graph
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeReadGraphResult {
    /// All entities
    pub entities: Vec<EntityInfo>,

    /// All relations
    pub relations: Vec<RelationInfo>,

    /// Total entity count
    pub total_entities: usize,

    /// Total relation count
    pub total_relations: usize,
}

// ============================================================================
// DELETE OBSERVATIONS (from Python v2.0)
// ============================================================================

/// Parameters for knowledge_delete_observation tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeDeleteObservationParams {
    /// Entity name
    pub entity_name: String,

    /// Observations to remove (exact match)
    pub observations: Vec<String>,
}

/// Result from knowledge_delete_observation
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeDeleteObservationResult {
    /// Entity name
    pub entity_name: String,

    /// Number of observations removed
    pub removed: usize,
}

// ============================================================================
// EXPORT GRAPH (from Python v2.0)
// ============================================================================

/// Parameters for export_graph tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportGraphParams {
    /// Include relations in export (default: true)
    #[serde(default = "default_true")]
    pub include_relations: bool,

    /// Export format: "dict" or "json"
    #[serde(default = "default_format")]
    pub format: String,

    /// Filter by entity types (empty = all)
    #[serde(default)]
    pub entity_types: Vec<String>,
}

/// Result from export_graph
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ExportGraphResult {
    /// All entities
    pub entities: Vec<EntityInfo>,

    /// Entity count
    pub entity_count: usize,

    /// All relations (if requested)
    #[serde(default)]
    pub relations: Vec<RelationInfo>,

    /// Relation count
    pub relation_count: usize,

    /// Export format used
    pub format: String,

    /// Export timestamp
    pub exported_at: i64,
}

// ============================================================================
// GRAPH TRAVERSAL (from Python v2.0)
// ============================================================================

/// Parameters for knowledge_get_neighbors tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeGetNeighborsParams {
    /// Entity name to start from
    pub entity_name: String,

    /// Maximum hop distance (default: 1)
    #[serde(default = "default_depth")]
    pub max_depth: usize,

    /// Filter by relation types (empty = all)
    #[serde(default)]
    pub relation_types: Vec<String>,
}

/// A neighbor with path information
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct NeighborInfo {
    /// Entity information
    pub entity: EntityInfo,

    /// Distance from start entity
    pub distance: usize,

    /// Path of relation IDs to reach this entity
    pub path: Vec<String>,
}

/// Result from knowledge_get_neighbors
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeGetNeighborsResult {
    /// Starting entity
    pub start_entity: String,

    /// Neighboring entities with distance
    pub neighbors: Vec<NeighborInfo>,

    /// Total neighbors found
    pub total: usize,
}

/// Parameters for knowledge_find_path tool
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeFindPathParams {
    /// Source entity name
    pub from: String,

    /// Target entity name
    pub to: String,

    /// Maximum path length (default: 5)
    #[serde(default = "default_max_depth")]
    pub max_depth: usize,
}

/// Result from knowledge_find_path
#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct KnowledgeFindPathResult {
    /// Whether a path was found
    pub found: bool,

    /// Path of relations (if found)
    pub path: Vec<RelationInfo>,

    /// Path length
    pub length: usize,
}

// Default helpers
fn default_limit() -> usize {
    10
}

fn default_true() -> bool {
    true
}

fn default_format() -> String {
    "dict".to_string()
}

fn default_depth() -> usize {
    1
}

fn default_max_depth() -> usize {
    5
}

#[cfg(test)]
mod tests {
    use super::*;

    impl KnowledgeAddEntityParams {
        fn new(name: impl Into<String>, entity_type: impl Into<String>) -> Self {
            Self {
                name: name.into(),
                entity_type: entity_type.into(),
                observations: Vec::new(),
            }
        }

        fn with_observation(mut self, obs: impl Into<String>) -> Self {
            self.observations.push(obs.into());
            self
        }
    }

    impl KnowledgeAddRelationParams {
        fn new(
            from: impl Into<String>,
            to: impl Into<String>,
            relation_type: impl Into<String>,
        ) -> Self {
            Self {
                from: from.into(),
                to: to.into(),
                relation_type: relation_type.into(),
            }
        }
    }

    impl KnowledgeSearchParams {
        fn new(query: impl Into<String>) -> Self {
            Self {
                query: query.into(),
                limit: super::default_limit(),
            }
        }

        fn with_limit(mut self, limit: usize) -> Self {
            self.limit = limit;
            self
        }
    }

    #[test]
    fn test_entity_params_builder() {
        let params = KnowledgeAddEntityParams::new("John Doe", "person")
            .with_observation("Software engineer")
            .with_observation("Works at WhytCard");

        assert_eq!(params.name, "John Doe");
        assert_eq!(params.entity_type, "person");
        assert_eq!(params.observations.len(), 2);
    }

    #[test]
    fn test_relation_params() {
        let params = KnowledgeAddRelationParams::new("John", "Jane", "works_with");

        assert_eq!(params.from, "John");
        assert_eq!(params.to, "Jane");
        assert_eq!(params.relation_type, "works_with");
    }

    #[test]
    fn test_search_params_builder() {
        let params = KnowledgeSearchParams::new("software").with_limit(20);

        assert_eq!(params.query, "software");
        assert_eq!(params.limit, 20);
    }

    #[test]
    fn test_serialization() {
        let params = KnowledgeAddEntityParams::new("Test", "concept");
        let json = serde_json::to_string(&params).unwrap();
        assert!(json.contains("Test"));
        assert!(json.contains("concept"));

        let parsed: KnowledgeAddEntityParams = serde_json::from_str(&json).unwrap();
        assert_eq!(parsed.name, "Test");
    }
}
