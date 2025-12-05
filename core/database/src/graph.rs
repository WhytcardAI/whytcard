//! Knowledge graph operations with entities and relations

use crate::{Database, DatabaseError, Result};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use surrealdb::RecordId;

/// Entity in the knowledge graph
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Record ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,

    /// Entity name
    pub name: String,

    /// Entity type (person, concept, tool, etc.)
    pub entity_type: String,

    /// Observations/facts about this entity
    #[serde(default)]
    pub observations: Vec<String>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,

    /// Update timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub updated_at: Option<DateTime<Utc>>,
}

/// Input for creating an entity
#[derive(Debug, Clone, Serialize)]
pub struct CreateEntity {
    /// Entity name
    pub name: String,

    /// Entity type
    pub entity_type: String,

    /// Initial observations
    #[serde(default)]
    pub observations: Vec<String>,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
}

impl CreateEntity {
    /// Create a new entity input
    pub fn new(name: impl Into<String>, entity_type: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            entity_type: entity_type.into(),
            observations: Vec::new(),
            metadata: None,
        }
    }

    /// Add initial observations
    pub fn with_observations(mut self, observations: Vec<String>) -> Self {
        self.observations = observations;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Relation between two entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Relation {
    /// Record ID
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<RecordId>,

    /// Source entity (in)
    #[serde(rename = "in")]
    pub from: RecordId,

    /// Target entity (out)
    #[serde(rename = "out")]
    pub to: RecordId,

    /// Type of relation
    pub relation_type: String,

    /// Relation weight/strength
    #[serde(default = "default_weight")]
    pub weight: f32,

    /// Additional metadata
    #[serde(skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,

    /// Creation timestamp
    #[serde(skip_serializing_if = "Option::is_none")]
    pub created_at: Option<DateTime<Utc>>,
}

fn default_weight() -> f32 {
    1.0
}

/// Input for creating a relation
#[derive(Debug, Clone)]
pub struct CreateRelation {
    /// Source entity
    pub from: RecordId,

    /// Target entity
    pub to: RecordId,

    /// Relation type
    pub relation_type: String,

    /// Relation weight
    pub weight: f32,

    /// Additional metadata
    pub metadata: Option<serde_json::Value>,
}

impl CreateRelation {
    /// Create a new relation input
    pub fn new(from: RecordId, to: RecordId, relation_type: impl Into<String>) -> Self {
        Self {
            from,
            to,
            relation_type: relation_type.into(),
            weight: 1.0,
            metadata: None,
        }
    }

    /// Set weight
    pub fn with_weight(mut self, weight: f32) -> Self {
        self.weight = weight;
        self
    }

    /// Set metadata
    pub fn with_metadata(mut self, metadata: serde_json::Value) -> Self {
        self.metadata = Some(metadata);
        self
    }
}

/// Entity with its relations for graph traversal results
#[derive(Debug, Clone, Deserialize)]
pub struct EntityWithRelations {
    /// The entity
    #[serde(flatten)]
    pub entity: Entity,

    /// Outgoing relations
    #[serde(default)]
    pub outgoing: Vec<RelatedEntity>,

    /// Incoming relations
    #[serde(default)]
    pub incoming: Vec<RelatedEntity>,
}

/// Related entity from graph traversal
#[derive(Debug, Clone, Deserialize)]
pub struct RelatedEntity {
    /// Related entity
    pub entity: Entity,

    /// Relation type
    pub relation_type: String,

    /// Relation weight
    pub weight: f32,
}

/// Direction for relation queries
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RelationDirection {
    /// From this entity to others
    Outgoing,
    /// From others to this entity
    Incoming,
    /// Both directions
    Both,
}

/// Graph operations
impl Database {
    // ============ Entity Operations ============

    /// Create a new entity
    pub async fn create_entity(&self, input: CreateEntity) -> Result<Entity> {
        let entity: Option<Entity> = self.inner().create("entity").content(input).await?;
        entity.ok_or_else(|| DatabaseError::Schema("Failed to create entity".into()))
    }

    /// Get entity by ID
    pub async fn get_entity(&self, id: &str) -> Result<Entity> {
        let entity: Option<Entity> = self.inner().select(("entity", id)).await?;
        entity.ok_or_else(|| DatabaseError::NotFound {
            table: "entity".into(),
            id: id.into(),
        })
    }

    /// Get entity by name
    pub async fn get_entity_by_name(&self, name: &str) -> Result<Option<Entity>> {
        let name_owned = name.to_string();
        let mut result = self
            .inner()
            .query("SELECT * FROM entity WHERE name = $name LIMIT 1")
            .bind(("name", name_owned))
            .await?;

        let entities: Vec<Entity> = result.take(0)?;
        Ok(entities.into_iter().next())
    }

    /// Update entity
    pub async fn update_entity(&self, id: &str, updates: serde_json::Value) -> Result<Entity> {
        let entity: Option<Entity> = self.inner().update(("entity", id)).merge(updates).await?;
        entity.ok_or_else(|| DatabaseError::NotFound {
            table: "entity".into(),
            id: id.into(),
        })
    }

    /// Add observation to entity
    pub async fn add_observation(&self, id: &str, observation: &str) -> Result<Entity> {
        let obs_owned = observation.to_string();
        let mut result = self
            .inner()
            .query("UPDATE type::thing('entity', $id) SET observations += $obs, updated_at = time::now() RETURN AFTER")
            .bind(("id", id.to_string()))
            .bind(("obs", obs_owned))
            .await?;

        let entities: Vec<Entity> = result.take(0)?;
        entities.into_iter().next().ok_or_else(|| DatabaseError::NotFound {
            table: "entity".into(),
            id: id.into(),
        })
    }

    /// Delete entity and its relations
    pub async fn delete_entity(&self, id: &str) -> Result<()> {
        // Delete all relations involving this entity
        self.inner()
            .query("DELETE relates_to WHERE in = type::thing('entity', $id) OR out = type::thing('entity', $id)")
            .bind(("id", id.to_string()))
            .await?;

        // Delete the entity
        let _: Option<Entity> = self.inner().delete(("entity", id)).await?;
        Ok(())
    }

    /// List entities by type
    pub async fn list_entities_by_type(&self, entity_type: &str) -> Result<Vec<Entity>> {
        let type_owned = entity_type.to_string();
        let mut result = self
            .inner()
            .query("SELECT * FROM entity WHERE entity_type = $type ORDER BY name")
            .bind(("type", type_owned))
            .await?;

        let entities: Vec<Entity> = result.take(0)?;
        Ok(entities)
    }

    /// Search entities by name pattern
    pub async fn search_entities(&self, pattern: &str) -> Result<Vec<Entity>> {
        let pattern_owned = pattern.to_string();
        let mut result = self
            .inner()
            .query("SELECT * FROM entity WHERE name CONTAINS $pattern ORDER BY name")
            .bind(("pattern", pattern_owned))
            .await?;

        let entities: Vec<Entity> = result.take(0)?;
        Ok(entities)
    }

    // ============ Relation Operations ============

    /// Create a relation between entities
    pub async fn create_relation(&self, input: CreateRelation) -> Result<Relation> {
        let query = r#"
            RELATE $from->relates_to->$to SET
                relation_type = $rel_type,
                weight = $weight,
                metadata = $metadata,
                created_at = time::now()
        "#;

        let mut result = self
            .inner()
            .query(query)
            .bind(("from", input.from))
            .bind(("to", input.to))
            .bind(("rel_type", input.relation_type))
            .bind(("weight", input.weight))
            .bind(("metadata", input.metadata))
            .await?;

        let relations: Vec<Relation> = result.take(0)?;
        relations
            .into_iter()
            .next()
            .ok_or_else(|| DatabaseError::Relation("Failed to create relation".into()))
    }

    /// Get all relations from an entity
    pub async fn get_outgoing_relations(&self, entity_id: &str) -> Result<Vec<Relation>> {
        let mut result = self
            .inner()
            .query("SELECT * FROM relates_to WHERE in = type::thing('entity', $id)")
            .bind(("id", entity_id.to_string()))
            .await?;

        let relations: Vec<Relation> = result.take(0)?;
        Ok(relations)
    }

    /// Get all relations to an entity
    pub async fn get_incoming_relations(&self, entity_id: &str) -> Result<Vec<Relation>> {
        let mut result = self
            .inner()
            .query("SELECT * FROM relates_to WHERE out = type::thing('entity', $id)")
            .bind(("id", entity_id.to_string()))
            .await?;

        let relations: Vec<Relation> = result.take(0)?;
        Ok(relations)
    }

    /// Delete a specific relation
    pub async fn delete_relation(&self, id: &str) -> Result<()> {
        let _: Option<Relation> = self.inner().delete(("relates_to", id)).await?;
        Ok(())
    }

    /// Delete relations between two entities
    pub async fn delete_relations_between(
        &self,
        from_id: &str,
        to_id: &str,
        relation_type: Option<&str>,
    ) -> Result<usize> {
        let query = match relation_type {
            Some(_) => {
                "DELETE relates_to WHERE in = type::thing('entity', $from) AND out = type::thing('entity', $to) AND relation_type = $rel_type RETURN BEFORE"
            }
            None => {
                "DELETE relates_to WHERE in = type::thing('entity', $from) AND out = type::thing('entity', $to) RETURN BEFORE"
            }
        };

        let mut builder = self
            .inner()
            .query(query)
            .bind(("from", from_id.to_string()))
            .bind(("to", to_id.to_string()));

        if let Some(rel_type) = relation_type {
            builder = builder.bind(("rel_type", rel_type.to_string()));
        }

        let mut result = builder.await?;
        let deleted: Vec<Relation> = result.take(0)?;
        Ok(deleted.len())
    }

    // ============ Graph Traversal ============

    /// Get entity with all its direct relations
    pub async fn get_entity_graph(&self, id: &str, _depth: u32) -> Result<EntityWithRelations> {
        let entity = self.get_entity(id).await?;

        // Get outgoing relations with target entities
        let outgoing_query = r#"
            SELECT
                out.* AS entity,
                relation_type,
                weight
            FROM relates_to
            WHERE in = type::thing('entity', $id)
        "#;

        // Get incoming relations with source entities
        let incoming_query = r#"
            SELECT
                in.* AS entity,
                relation_type,
                weight
            FROM relates_to
            WHERE out = type::thing('entity', $id)
        "#;

        let mut out_result = self
            .inner()
            .query(outgoing_query)
            .bind(("id", id.to_string()))
            .await?;

        let mut in_result = self
            .inner()
            .query(incoming_query)
            .bind(("id", id.to_string()))
            .await?;

        let outgoing: Vec<RelatedEntity> = out_result.take(0).unwrap_or_default();
        let incoming: Vec<RelatedEntity> = in_result.take(0).unwrap_or_default();

        Ok(EntityWithRelations {
            entity,
            outgoing,
            incoming,
        })
    }

    /// Find path between two entities
    pub async fn find_path(
        &self,
        from_id: &str,
        to_id: &str,
        _max_depth: u32,
    ) -> Result<Vec<Entity>> {
        // Simple BFS-style path finding using SurrealDB graph traversal
        let query = r#"
            SELECT VALUE path FROM (
                SELECT ->relates_to->(entity WHERE id = type::thing('entity', $to))<-relates_to<-* AS path
                FROM type::thing('entity', $from)
            ) LIMIT 1
        "#;

        let mut result = self
            .inner()
            .query(query)
            .bind(("from", from_id.to_string()))
            .bind(("to", to_id.to_string()))
            .await?;

        let paths: Vec<Vec<Entity>> = result.take(0).unwrap_or_default();
        Ok(paths.into_iter().next().unwrap_or_default())
    }

    /// Get related entities by relation type
    pub async fn get_related(
        &self,
        entity_id: &str,
        relation_type: &str,
        direction: RelationDirection,
    ) -> Result<Vec<Entity>> {
        let query = match direction {
            RelationDirection::Outgoing => {
                "SELECT out.* FROM relates_to WHERE in = type::thing('entity', $id) AND relation_type = $rel_type"
            }
            RelationDirection::Incoming => {
                "SELECT in.* FROM relates_to WHERE out = type::thing('entity', $id) AND relation_type = $rel_type"
            }
            RelationDirection::Both => {
                r#"
                SELECT * FROM (
                    SELECT out.* FROM relates_to WHERE in = type::thing('entity', $id) AND relation_type = $rel_type
                    UNION
                    SELECT in.* FROM relates_to WHERE out = type::thing('entity', $id) AND relation_type = $rel_type
                )
                "#
            }
        };

        let mut result = self
            .inner()
            .query(query)
            .bind(("id", entity_id.to_string()))
            .bind(("rel_type", relation_type.to_string()))
            .await?;

        let entities: Vec<Entity> = result.take(0).unwrap_or_default();
        Ok(entities)
    }

    /// Count entities
    pub async fn count_entities(&self) -> Result<usize> {
        let mut result = self
            .inner()
            .query("SELECT count() FROM entity GROUP ALL")
            .await?;

        #[derive(Deserialize)]
        struct CountResult {
            count: usize,
        }

        let counts: Vec<CountResult> = result.take(0)?;
        Ok(counts.first().map(|c| c.count).unwrap_or(0))
    }

    /// Count relations
    pub async fn count_relations(&self) -> Result<usize> {
        let mut result = self
            .inner()
            .query("SELECT count() FROM relates_to GROUP ALL")
            .await?;

        #[derive(Deserialize)]
        struct CountResult {
            count: usize,
        }

        let counts: Vec<CountResult> = result.take(0)?;
        Ok(counts.first().map(|c| c.count).unwrap_or(0))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_create_and_get_entity() {
        let db = Database::new_memory().await.unwrap();

        let input = CreateEntity::new("Rust", "programming_language")
            .with_observations(vec!["Systems programming".into(), "Memory safe".into()]);

        let entity = db.create_entity(input).await.unwrap();
        assert_eq!(entity.name, "Rust");
        assert_eq!(entity.entity_type, "programming_language");
        assert_eq!(entity.observations.len(), 2);

        // Get by name
        let found = db.get_entity_by_name("Rust").await.unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Rust");
    }

    #[tokio::test]
    async fn test_add_observation() {
        let db = Database::new_memory().await.unwrap();

        let entity = db
            .create_entity(CreateEntity::new("Python", "language"))
            .await
            .unwrap();
        let id = entity.id.unwrap().key().to_string();

        let updated = db.add_observation(&id, "Dynamic typing").await.unwrap();
        assert_eq!(updated.observations.len(), 1);
        assert_eq!(updated.observations[0], "Dynamic typing");

        let updated = db.add_observation(&id, "Duck typing").await.unwrap();
        assert_eq!(updated.observations.len(), 2);
    }

    #[tokio::test]
    async fn test_create_relation() {
        let db = Database::new_memory().await.unwrap();

        let rust = db
            .create_entity(CreateEntity::new("Rust", "language"))
            .await
            .unwrap();
        let cargo = db
            .create_entity(CreateEntity::new("Cargo", "tool"))
            .await
            .unwrap();

        let relation = db
            .create_relation(CreateRelation::new(
                rust.id.clone().unwrap(),
                cargo.id.clone().unwrap(),
                "uses",
            ))
            .await
            .unwrap();

        assert_eq!(relation.relation_type, "uses");
        assert_eq!(relation.weight, 1.0);
    }

    #[tokio::test]
    async fn test_get_relations() {
        let db = Database::new_memory().await.unwrap();

        let lang = db
            .create_entity(CreateEntity::new("Rust", "language"))
            .await
            .unwrap();
        let tool1 = db
            .create_entity(CreateEntity::new("Cargo", "tool"))
            .await
            .unwrap();
        let tool2 = db
            .create_entity(CreateEntity::new("Rustfmt", "tool"))
            .await
            .unwrap();

        let lang_id = lang.id.unwrap().key().to_string();

        db.create_relation(CreateRelation::new(
            RecordId::from(("entity", lang_id.as_str())),
            tool1.id.unwrap(),
            "uses",
        ))
        .await
        .unwrap();
        db.create_relation(CreateRelation::new(
            RecordId::from(("entity", lang_id.as_str())),
            tool2.id.unwrap(),
            "uses",
        ))
        .await
        .unwrap();

        let outgoing = db.get_outgoing_relations(&lang_id).await.unwrap();
        assert_eq!(outgoing.len(), 2);

        let incoming = db.get_incoming_relations(&lang_id).await.unwrap();
        assert!(incoming.is_empty());
    }

    #[tokio::test]
    async fn test_delete_entity_cascades() {
        let db = Database::new_memory().await.unwrap();

        let e1 = db
            .create_entity(CreateEntity::new("A", "test"))
            .await
            .unwrap();
        let e2 = db
            .create_entity(CreateEntity::new("B", "test"))
            .await
            .unwrap();

        let e1_id = e1.id.unwrap().key().to_string();
        let e2_id = e2.id.clone().unwrap().key().to_string();

        db.create_relation(CreateRelation::new(
            RecordId::from(("entity", e1_id.as_str())),
            e2.id.unwrap(),
            "linked",
        ))
        .await
        .unwrap();

        // Delete entity A
        db.delete_entity(&e1_id).await.unwrap();

        // Relation should be gone
        let relations = db.get_incoming_relations(&e2_id).await.unwrap();
        assert!(relations.is_empty());
    }

    #[tokio::test]
    async fn test_search_entities() {
        let db = Database::new_memory().await.unwrap();

        db.create_entity(CreateEntity::new("Rust Programming", "language"))
            .await
            .unwrap();
        db.create_entity(CreateEntity::new("Rustfmt", "tool"))
            .await
            .unwrap();
        db.create_entity(CreateEntity::new("Python", "language"))
            .await
            .unwrap();

        let results = db.search_entities("Rust").await.unwrap();
        assert_eq!(results.len(), 2);
    }

    #[tokio::test]
    async fn test_list_by_type() {
        let db = Database::new_memory().await.unwrap();

        db.create_entity(CreateEntity::new("Rust", "language"))
            .await
            .unwrap();
        db.create_entity(CreateEntity::new("Python", "language"))
            .await
            .unwrap();
        db.create_entity(CreateEntity::new("Cargo", "tool"))
            .await
            .unwrap();

        let languages = db.list_entities_by_type("language").await.unwrap();
        assert_eq!(languages.len(), 2);

        let tools = db.list_entities_by_type("tool").await.unwrap();
        assert_eq!(tools.len(), 1);
    }
}
