# WhytCard Core

Core modules of the WhytCard AI infrastructure.

## Structure

```
core/
├── intelligence/     # MCP Server + CORTEX Engine
│   └── ...           # Triple memory + knowledge graph + MCP Gateway
│
├── database/         # SurrealDB persistence
│   └── migrations/
│
├── rag/              # Retrieval-Augmented Generation
│   └── ...           # FastEmbed embeddings
│
├── llm/              # Local LLM inference
│   └── ...           # llama.cpp/GGUF integration
│
└── mcp/              # MCP Gateway configuration
    └── servers.json
```

## Principle

Each subdirectory is an **independent module**:

- Can be updated separately
- Has its own single responsibility
- Communicates via MCP Protocol

## Inter-Module Communication

```
┌──────────────┐
│   VS Code    │ ← Entry point (VSIX)
│    (MCP)     │
└──────┬───────┘
       │ MCP Protocol
       ▼
┌──────────────┐
│ INTELLIGENCE │ ← Orchestrator
└──────┬───────┘
       │
       ├──────────┬──────────┐
       ▼          ▼          ▼
┌─────────┐┌─────────┐┌─────────┐
│   RAG   ││   LLM   ││DATABASE │
└─────────┘└─────────┘└─────────┘
```

## License

All core modules are licensed under GPL-3.0. See LICENSE file at repository root.
