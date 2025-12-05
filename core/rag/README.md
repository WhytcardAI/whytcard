# WhytCard Core - RAG

Systeme de Retrieval-Augmented Generation.

## Responsabilite

- Indexation de documents
- Recherche semantique
- Chunking et embeddings

## Stack

| Composant  | Technologie           |
| ---------- | --------------------- |
| Vectors    | LanceDB               |
| Embeddings | fastembed (MiniLM-L6) |
| Chunking   | Custom Rust           |

## API Interne

| Fonction        | Description          |
| --------------- | -------------------- |
| `index(doc)`    | Indexer un document  |
| `search(query)` | Recherche semantique |
| `delete(id)`    | Supprimer de l'index |

## Donnees

- Vecteurs: `data/vectors/`
- Modele embeddings: `data/models/embeddings/`

## Source

Migration depuis: `old/Persona/Body/apps/core/src/brain/`
