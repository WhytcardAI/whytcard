# WhytCard Core - Intelligence

Serveur MCP pour la mémoire persistante, le graphe de connaissances et le moteur cognitif CORTEX.

## Description

Ce module fournit l'intelligence centrale de WhytCard via le protocole MCP. Il combine :

- **Triple Memory** : Mémoire sémantique (vecteurs), épisodique (événements), procédurale (règles)
- **CORTEX Engine** : Pipeline Perceive → Execute → Learn
- **Knowledge Graph** : Entités et relations structurées
- **Intégrations externes** : Context7, Tavily, MS Learn
- **MCP Client** : Connexion à d'autres serveurs MCP (sequential-thinking)

## Structure

```
intelligence/
├── Cargo.toml
├── README.md
├── data/                    # Données runtime (cortex/, logs/, etc.)
├── mcp/                     # Configuration MCP (servers.json)
└── src/
    ├── lib.rs               # Point d'entrée + exports
    ├── main.rs              # Binaire serveur
    ├── server.rs            # Serveur MCP (rmcp)
    ├── config.rs            # IntelligenceConfig
    ├── error.rs             # Gestion erreurs
    ├── paths.rs             # DataPaths (répertoires)
    │
    ├── cortex/              # Moteur cognitif CORTEX
    │   ├── mod.rs           # CortexEngine, CortexConfig
    │   ├── engine.rs        # Orchestrateur principal
    │   ├── perceiver.rs     # Analyse intents, labels, complexity
    │   ├── executor.rs      # OODA loops, ExecutionPlan
    │   ├── learner.rs       # Feedback, apprentissage
    │   └── context.rs       # ContextManager, historique
    │
    ├── memory/              # Triple Memory System
    │   ├── mod.rs           # TripleMemory
    │   ├── semantic.rs      # Vecteurs/embeddings (via RAG)
    │   ├── episodic.rs      # Événements/historique
    │   └── procedural.rs    # Règles/patterns
    │
    ├── integrations/        # Clients externes
    │   ├── mod.rs           # IntegrationHub
    │   ├── context7.rs      # Documentation librairies
    │   ├── tavily.rs        # Recherche web
    │   └── mslearn.rs       # Documentation Microsoft
    │
    ├── mcp_client/          # Client MCP vers autres serveurs
    │   ├── mod.rs           # McpClientManager
    │   ├── manager.rs       # Gestion connexions
    │   ├── sequential_thinking.rs  # Client sequential-thinking
    │   └── types.rs         # Types partagés
    │
    ├── tools/               # Outils MCP exposés
    │   ├── mod.rs           # Exports
    │   ├── cortex.rs        # cortex_process, cortex_feedback, cortex_stats
    │   ├── memory.rs        # memory_store, memory_search, etc.
    │   ├── knowledge.rs     # knowledge_add_entity, etc.
    │   ├── integrations.rs  # Outils pour intégrations
    │   ├── external.rs      # Outils externes
    │   └── ...
```

## Documentation

Documentation technique détaillée :

- **Overview** : [docs/specs/intelligence/overview.md](../../docs/specs/intelligence/overview.md)
- **Server** : [docs/specs/intelligence/server.md](../../docs/specs/intelligence/server.md)
- **Config** : [docs/specs/intelligence/config.md](../../docs/specs/intelligence/config.md)
- **Errors** : [docs/specs/intelligence/error.md](../../docs/specs/intelligence/error.md)
- **Tools** : [docs/specs/intelligence/tools/](../../docs/specs/intelligence/tools/)
- **Multi-Agent** : [docs/specs/intelligence/multi-agent-architecture.md](../../docs/specs/intelligence/multi-agent-architecture.md)

## Outils MCP

### CORTEX (Moteur cognitif)

| Outil             | Description                                   |
| ----------------- | --------------------------------------------- |
| `cortex_process`  | Pipeline principal Perceive → Execute → Learn |
| `cortex_feedback` | Feedback pour apprentissage adaptatif         |
| `cortex_stats`    | Statistiques du moteur                        |
| `cortex_cleanup`  | Nettoyage données anciennes                   |

### Memory (Triple Memory)

| Outil           | Description                                   |
| --------------- | --------------------------------------------- |
| `memory_store`  | Stocke avec indexation sémantique optionnelle |
| `memory_search` | Recherche sémantique                          |
| `memory_get`    | Récupère par clé                              |
| `memory_delete` | Supprime par clé                              |
| `memory_list`   | Liste paginée                                 |

### Knowledge (Graphe de connaissances)

| Outil                       | Description                            |
| --------------------------- | -------------------------------------- |
| `knowledge_add_entity`      | Ajoute une entité                      |
| `knowledge_add_observation` | Ajoute observations à entité existante |
| `knowledge_add_relation`    | Crée relation entre entités            |
| `knowledge_search`          | Recherche dans le graphe               |
| `knowledge_get_entity`      | Récupère entité + relations            |
| `knowledge_delete_entity`   | Supprime entité + relations            |
| `knowledge_delete_relation` | Supprime relations                     |
| `knowledge_read_graph`      | Export graphe complet                  |

## Usage

```bash
# Lancer le serveur MCP (stdio)
cargo run -p whytcard-intelligence

# Avec répertoire de données custom
WHYTCARD_DATA_DIR=/path/to/data cargo run -p whytcard-intelligence

# Tests
cargo test -p whytcard-intelligence

# Clippy
cargo clippy -p whytcard-intelligence
```

## Roadmap

- [x] Triple Memory System
- [x] CORTEX Engine (Perceive, Execute, Learn)
- [x] Intégrations (Context7, Tavily, MS Learn)
- [x] MCP Client (sequential-thinking)
- [ ] Multi-Agent System (architecture à définir)
