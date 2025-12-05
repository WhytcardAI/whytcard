# WhytCard Core - Database

Couche de persistence basée sur SurrealDB.

## Description

Ce module gère le stockage unifié pour WhytCard :

- **Documents** : Stockage JSON flexible
- **Vecteurs** : Recherche sémantique (HNSW)
- **Graphe** : Entités et relations

## Structure

```
database/
├── Cargo.toml
├── src/
│   ├── lib.rs          # Point d'entrée
│   ├── config.rs       # Configuration
│   ├── database.rs     # Instance DB
│   ├── error.rs        # Erreurs
│   ├── schema.rs       # Schéma SurrealDB
│   ├── documents.rs    # Opérations Documents
│   ├── vectors.rs      # Opérations Vecteurs
│   └── graph.rs        # Opérations Graphe
└── README.md
```

## Stack

| Composant | Technologie |
| --------- | ----------- |
| Database  | SurrealDB   |
| Driver    | surrealdb   |
| Async     | tokio       |

## Usage

```rust
use whytcard_database::{Database, Config};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialisation
    let db = Database::new_persistent("./data").await?;

    // Graphe
    db.create_entity(CreateEntity::new("Alice", "person")).await?;

    // Vecteurs
    db.create_chunk(CreateChunk::new("content", vec![0.1, 0.2])).await?;

    Ok(())
}
```

## Documentation

Specs: [docs/specs/database/overview.md](../../docs/specs/database/overview.md)
