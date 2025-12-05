# WhytCard VS Code Extension

Connect your VS Code to WhytCard Hub - Your local, private AI assistant.

## Features

- **Local AI Chat**: Chat with your codebase using local LLM models
- **Real-time Sync**: SSE connection for live updates across all clients
- **Project Analysis**: Automatic detection of project framework and technologies
- **MCP Integration**: Support for Model Context Protocol servers

## Requirements

- VS Code 1.100.0 or higher
- WhytCard Hub desktop application running (default: `http://localhost:3000`)

## Installation

1. Install the extension from VS Code Marketplace
2. Start WhytCard Hub desktop application
3. Open the WhytCard sidebar from the activity bar
4. The extension will auto-connect to the Hub

## Settings

| Setting                        | Description                   | Default                 |
| ------------------------------ | ----------------------------- | ----------------------- |
| `whytcard.language`            | Interface language            | `auto`                  |
| `whytcard.hubUrl`              | WhytCard Hub URL              | `http://localhost:3000` |
| `whytcard.autoConnectSSE`      | Auto-connect to Hub SSE       | `true`                  |
| `whytcard.autoDetectFramework` | Auto-detect project framework | `true`                  |

## Commands

- `WhytCard: Open WhytCard` - Open the sidebar
- `WhytCard: Initialize Workspace for Copilot` - Initialize workspace
- `WhytCard: Generate MCP Configuration` - Configure MCP servers
- `WhytCard: Reconnect to WhytCard Hub` - Manually reconnect SSE

## Hub Integration

The extension connects to WhytCard Hub via Server-Sent Events (SSE) for real-time communication:

### SSE Events

| Event           | Description                      |
| --------------- | -------------------------------- |
| `chat_response` | Streaming chat responses from AI |
| `session_sync`  | Sync chat history across clients |
| `heartbeat`     | Connection keep-alive            |

### Connection Features

- Automatic reconnection with exponential backoff
- Session ID filtering for multi-client support
- Heartbeat timeout detection (45s)
- Max 10 reconnection attempts

## Development

```bash
# Install dependencies
npm install

# Compile
npm run compile

# Watch mode
npm run watch

# Package
npm run package
```

## Architecture

```text
src/
  extension.ts          # Extension entry point
  services/
    hubService.ts       # SSE client and Hub communication
    apiService.ts       # REST API calls
    secretsService.ts   # Secure storage
    mcpService.ts       # MCP configuration
  sidebar/
    SidebarProvider.ts  # Webview sidebar
  core/
    projectAnalyzer.ts  # Project detection
  i18n/
    index.ts           # Internationalization
```

## License

GPL-3.0
