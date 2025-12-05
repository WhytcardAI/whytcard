# WhytCard MCP Servers

Local installation directory for external MCP servers.

## Structure

```
mcp/
  package.json          # npm dependencies
  node_modules/         # npm packages (gitignored)
  venv/                 # Python virtual environment (gitignored)
  mcp_servers.json      # Server configuration
```

## Local-First Philosophy

All MCP servers are installed locally in this directory rather than globally.
This ensures:

1. **Reproducibility** - Same versions across all installations
2. **Isolation** - No conflicts with global packages
3. **Portability** - Project can be moved/cloned with all dependencies
4. **Offline capability** - Once installed, no network needed to run

## Adding a Server

Use the MCP management tools:

```
mcp_install(name="context7", package="@upstash/context7-mcp", package_type="npm")
```

## Manual Installation

For npm packages:
```bash
cd core/mcp
npm install @upstash/context7-mcp
```

For pip packages:
```bash
cd core/mcp
python -m venv venv
./venv/Scripts/pip install mcp-server-fetch
```
