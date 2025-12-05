# WhytCard Core - Intelligence

MCP server for persistent memory, knowledge graph, and CORTEX cognitive engine.

## Description

This module provides WhytCard's core intelligence via the MCP protocol. It combines:

- **Triple Memory**: Semantic memory (vectors), episodic memory (events), procedural memory (rules)
- **CORTEX Engine**: Perceive → Execute → Learn pipeline
- **Knowledge Graph**: Structured entities and relations
- **External Integrations**: Context7, Tavily, MS Learn
- **MCP Client**: Connection to other MCP servers (sequential-thinking)

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

Detailed technical documentation:

- **Overview**: docs/specs/intelligence/overview.md
- **Server**: docs/specs/intelligence/server.md
- **Config**: docs/specs/intelligence/config.md
- **Errors**: docs/specs/intelligence/error.md
- **Tools**: docs/specs/intelligence/tools/
- **Multi-Agent**: docs/specs/intelligence/multi-agent-architecture.md

## MCP Tools

### CORTEX (Cognitive Engine)

| Tool              | Description                                    |
| ----------------- | ---------------------------------------------- |
| `cortex_process`  | Main Perceive → Execute → Learn pipeline      |
| `cortex_feedback` | Feedback for adaptive learning                 |
| `cortex_stats`    | Engine statistics                              |
| `cortex_cleanup`  | Cleanup old data                               |
| `cortex_execute`  | Execute shell commands (npm, cargo, git)       |

### Memory (Triple Memory)

| Tool            | Description                               |
| --------------- | ----------------------------------------- |
| `memory_store`  | Store with optional semantic indexing     |
| `memory_search` | Semantic search                           |
| `memory_get`    | Retrieve by key                           |
| `memory_delete` | Delete by key                             |
| `memory_list`   | Paginated list                            |

### Knowledge (Knowledge Graph)

| Tool                        | Description                          |
| --------------------------- | ------------------------------------ |
| `knowledge_add_entity`      | Add entity                           |
| `knowledge_add_observation` | Add observations to existing entity  |
| `knowledge_add_relation`    | Create relation between entities     |
| `knowledge_search`          | Search in graph                      |
| `knowledge_get_entity`      | Retrieve entity + relations          |
| `knowledge_delete_entity`   | Delete entity + relations            |
| `knowledge_delete_relation` | Delete relations                     |
| `knowledge_read_graph`      | Export full graph                    |

## Usage

```bash
# Start MCP server (stdio)
cargo run -p whytcard-intelligence

# With custom data directory
WHYTCARD_DATA_DIR=/path/to/data cargo run -p whytcard-intelligence

# Tests
cargo test -p whytcard-intelligence

# Clippy
cargo clippy -p whytcard-intelligence
```

## Roadmap

- [x] Triple Memory System
- [x] CORTEX Engine (Perceive, Execute, Learn)
- [x] Integrations (Context7, Tavily, MS Learn)
- [x] MCP Client (sequential-thinking)
- [ ] Multi-Agent System (architecture TBD)
