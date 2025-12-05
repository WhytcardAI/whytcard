# WhytCard

**Local AI Ecosystem** — Secure, Local, Sovereign.

A high-performance local AI system built with Rust & Tauri. Stop renting your intelligence.

[![License: GPL-3.0](https://img.shields.io/badge/License-GPL--3.0-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/Rust-1.75+-orange.svg)](https://www.rust-lang.org/)

## Overview

WhytCard is a complete local AI ecosystem featuring:

- **CORTEX Engine** — Cognitive orchestration with Perceive → Execute → Learn pipeline
- **Triple Memory System** — Semantic (vectors), Episodic (events), Procedural (rules)
- **Knowledge Graph** — Structured entities and relations via SurrealDB
- **MCP Server** — Model Context Protocol for AI tool integration
- **Local LLM** — Run quantized models (Llama 3, Mistral) on your hardware

```
┌─────────────────────────────────────────────────────────┐
│                    CORTEX ENGINE                         │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │ PERCEIVE │─▶│ EXECUTE  │─▶│  LEARN   │              │
│  └──────────┘  └──────────┘  └──────────┘              │
└─────────────────────────────────────────────────────────┘
┌─────────────────────────────────────────────────────────┐
│                  TRIPLE MEMORY                           │
│  ┌──────────┐  ┌──────────┐  ┌──────────┐              │
│  │ SEMANTIC │  │ EPISODIC │  │PROCEDURAL│              │
│  │ (Vectors)│  │ (Events) │  │ (Rules)  │              │
│  └──────────┘  └──────────┘  └──────────┘              │
└─────────────────────────────────────────────────────────┘
```

## Architecture

```
whytcard/
├── core/                    # Rust modules
│   ├── intelligence/        # MCP Server + CORTEX Engine
│   ├── database/            # SurrealDB (documents, vectors, graph)
│   ├── rag/                 # Retrieval-Augmented Generation
│   ├── llm/                 # Local LLM inference
│   └── hub/                 # Tauri app + HTTP API (WIP)
│
├── addons/                  # Extensions
│   ├── chrome/              # Browser extension
│   ├── vscode/              # VS Code extension
│   ├── ears/                # STT service (Whisper)
│   └── voice/               # TTS service (XTTS)
│
└── data/                    # Runtime data
    ├── cortex/              # Memory storage
    ├── vectors/             # Vector embeddings
    └── models/              # LLM models
```

## Quick Start

### Prerequisites

- **Rust** 1.75+
- **Node.js** 20+ (for addons)
- **Python** 3.10+ (for voice/ears services)

### Run the MCP Server

```bash
cd core/intelligence
cargo run --release
```

The server starts on stdio for MCP protocol communication.

### With custom data directory

```bash
WHYTCARD_DATA_DIR=/path/to/data cargo run -p whytcard-intelligence
```

### With namespace isolation

```bash
cargo run -p whytcard-intelligence -- --namespace copilot
```

## MCP Tools

### CORTEX (Cognitive Engine)

| Tool | Description |
|------|-------------|
| `cortex_process` | Main Perceive → Execute → Learn pipeline |
| `cortex_feedback` | Feedback for adaptive learning |
| `cortex_stats` | Engine statistics |
| `cortex_cleanup` | Cleanup old data |
| `cortex_execute` | Execute shell commands |

### Memory

| Tool | Description |
|------|-------------|
| `memory_store` | Store with semantic indexing |
| `memory_search` | Semantic search |
| `memory_get` | Retrieve by key |
| `memory_delete` | Delete by key |
| `hybrid_search` | Search across all memory types |
| `get_context` | Aggregated context for queries |

### Knowledge Graph

| Tool | Description |
|------|-------------|
| `knowledge_add_entity` | Add entity |
| `knowledge_add_relation` | Create relation |
| `knowledge_search` | Search graph |
| `knowledge_get_entity` | Get entity + relations |
| `knowledge_find_path` | Find path between entities |
| `export_graph` | Export full graph |

### External Integrations

| Tool | Description |
|------|-------------|
| `sequential_thinking` | Problem decomposition |
| `external_docs` | Library documentation (Context7) |
| `external_search` | Web search (Tavily) |

### MCP Server Management

| Tool | Description |
|------|-------------|
| `mcp_available_servers` | List predefined MCP servers |
| `mcp_list_installed` | List installed servers |
| `mcp_install` | Install a server |
| `mcp_uninstall` | Uninstall a server |
| `mcp_connect` | Connect to a server |
| `mcp_disconnect` | Disconnect from a server |
| `mcp_list_tools` | List tools of a server |
| `mcp_call` | Call a tool on external server |
| `mcp_status` | Get connection status |
| `mcp_configure` | Configure server settings |

## Configuration

### Environment Variables

```bash
# Data directory
WHYTCARD_DATA_DIR=/path/to/data

# Namespace for isolation
WHYTCARD_NAMESPACE=copilot

# External APIs (optional)
TAVILY_API_KEY=your-key
```

### Predefined MCP Servers

The following MCP servers are pre-configured and can be connected on demand:

| Server | Description | Requires API Key |
|--------|-------------|------------------|
| `sequential-thinking` | Problem decomposition & analysis | No |
| `context7` | Library documentation lookup | No |
| `playwright` | Browser automation & testing | No |
| `memory` | Persistent memory storage | No |
| `microsoft-learn` | Microsoft/Azure documentation | No |
| `markitdown` | Document conversion to markdown | No |
| `chrome-devtools` | Chrome DevTools Protocol | No |
| `tavily` | Web search & research | Yes (`TAVILY_API_KEY`) |

### MCP Server Configuration

Servers are managed via `core/mcp/servers.json`:

```json
{
  "sequential-thinking": {
    "command": "npx",
    "args": ["-y", "@anthropic/mcp-sequential-thinking"],
    "enabled": true
  },
  "context7": {
    "command": "npx",
    "args": ["-y", "@anthropic/context7-mcp"],
    "enabled": true
  },
  "playwright": {
    "command": "npx",
    "args": ["-y", "@anthropic/mcp-playwright"],
    "enabled": true
  },
  "memory": {
    "command": "npx",
    "args": ["-y", "@anthropic/mcp-memory"],
    "enabled": true
  },
  "microsoft-learn": {
    "command": "npx",
    "args": ["-y", "@anthropic/mcp-microsoft-learn"],
    "enabled": true
  },
  "markitdown": {
    "command": "npx",
    "args": ["-y", "@anthropic/mcp-markitdown"],
    "enabled": true
  },
  "chrome-devtools": {
    "command": "npx",
    "args": ["-y", "@anthropic/mcp-chrome-devtools"],
    "enabled": true
  },
  "tavily": {
    "command": "npx",
    "args": ["-y", "tavily-mcp"],
    "env": {
      "TAVILY_API_KEY": "${TAVILY_API_KEY}"
    },
    "enabled": true
  }
}
```

## Development

### Build all modules

```bash
# Database
cd core/database && cargo build

# RAG
cd core/rag && cargo build

# Intelligence
cd core/intelligence && cargo build

# LLM
cd core/llm && cargo build
```

### Run tests

```bash
cd core/intelligence
cargo test
```

### Clippy

```bash
cargo clippy -p whytcard-intelligence
```

## Addons

### Chrome Extension

```bash
cd addons/chrome
npm install
# Load unpacked extension in Chrome
```

### VS Code Extension

```bash
cd addons/vscode
npm install
npm run compile
# Press F5 in VS Code to debug
```

### Ears (STT)

```bash
cd addons/ears
pip install -r requirements.txt
python -m ears
```

### Voice (TTS)

```bash
cd addons/voice
pip install -r requirements.txt
python -m voice
```

## Tech Stack

| Component | Technology |
|-----------|------------|
| Core Engine | Rust |
| Database | SurrealDB (embedded) |
| Embeddings | FastEmbed (ONNX) |
| MCP Protocol | rmcp SDK |
| Desktop App | Tauri (planned) |
| Browser Extension | Manifest V3 |
| IDE Extension | VS Code Extension API |

## Roadmap

- [x] Triple Memory System
- [x] CORTEX Engine (Perceive, Execute, Learn)
- [x] Knowledge Graph
- [x] MCP Server
- [x] External Integrations (Context7, Tavily, MS Learn)
- [x] Chrome Extension
- [x] VS Code Extension
- [ ] Tauri Desktop App (Hub)
- [ ] Multi-Agent System
- [ ] Voice Interface

## License

GPL-3.0 — See [LICENSE](LICENSE) for details.

## Links

- **Website**: [whytcard.ai](https://whytcard.ai)
- **Documentation**: Coming soon
- **Issues**: [GitHub Issues](https://github.com/WhytcardAI/whytcard/issues)
