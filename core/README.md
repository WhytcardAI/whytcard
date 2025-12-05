# WhytCard Core

Core functionalities of the WhytCard ecosystem.

## Structure

```
core/
├── hub/              # Tauri Application + HTTP API
│   ├── backend/      # Rust (Axum)
│   └── frontend/     # React
│
├── llm/              # Local LLM inference
│   └── ...           # llama.cpp integration
│
├── rag/              # Retrieval-Augmented Generation
│   └── ...           # Vector embeddings
│
├── intelligence/     # CORTEX Memory (MCP Server)
│   └── ...           # Triple memory + knowledge graph
│
└── database/         # SurrealDB persistence
    └── migrations/
```

## Principle

Each subdirectory is an **independent feature**:

- Can be updated separately
- Has its own single responsibility
- Communicates via well-defined internal APIs

## Inter-Module Communication

```
┌─────────┐
│   HUB   │ ← Single entry point
└────┬────┘
     │
     ├──────────┬──────────┬──────────┐
     ▼          ▼          ▼          ▼
┌─────────┐┌─────────┐┌─────────┐┌─────────┐
│   LLM   ││   RAG   ││INTELLIG.││DATABASE │
└─────────┘└─────────┘└─────────┘└─────────┘
```

The Hub orchestrates, the modules execute.

## License

All core modules are licensed under GPL-3.0. See LICENSE file at repository root.
