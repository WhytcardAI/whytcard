import * as vscode from "vscode";
import { LocaleKey } from "../i18n/locales";

export interface McpServerConfig {
  command: string;
  args: string[];
  env?: Record<string, string>;
}

export interface McpServerDefinition {
  id: string;
  name: LocaleKey;
  description: LocaleKey;
  defaultConfig: McpServerConfig;
  tools: string[];
  configFields?: {
    key: string;
    label: string;
    placeholder?: string;
    defaultValue?: string;
    description?: string;
    type: "text" | "password" | "path";
    required?: boolean;
  }[];
}

export class McpService {
  constructor(private context: vscode.ExtensionContext) {}

  getAvailableServers(): McpServerDefinition[] {
    return [
      {
        id: "filesystem",
        name: "mcp.filesystem.name",
        description: "mcp.filesystem.desc",
        tools: ["read_file", "write_file", "list_directory", "move_file"],
        defaultConfig: {
          command: "npx",
          args: [
            "-y",
            "@modelcontextprotocol/server-filesystem",
            "${rootPath}",
          ],
        },
        configFields: [
          {
            key: "rootPath",
            label: "Root Path",
            defaultValue: "${workspaceFolder}",
            type: "path",
            description: "Directory to expose to the model",
          },
        ],
      },
      {
        id: "tavily",
        name: "mcp.tavily.name",
        description: "mcp.tavily.desc",
        tools: ["tavily_search", "tavily_extract"],
        defaultConfig: {
          command: "npx",
          args: ["-y", "@tavily/mcp-server"],
          env: {
            TAVILY_API_KEY: "${apiKey}",
          },
        },
        configFields: [
          {
            key: "apiKey",
            label: "API Key",
            type: "password",
            required: true,
            description: "Tavily API Key",
          },
        ],
      },
      {
        id: "sequential-thinking",
        name: "mcp.sequential.name",
        description: "mcp.sequential.desc",
        tools: ["sequentialthinking"],
        defaultConfig: {
          command: "npx",
          args: ["-y", "@modelcontextprotocol/server-sequential-thinking"],
        },
      },
      {
        id: "context7",
        name: "mcp.context7.name",
        description: "mcp.context7.desc",
        tools: ["resolve-library-id", "get-library-docs"],
        defaultConfig: {
          command: "npx",
          args: ["-y", "@upstash/context7-mcp"],
        },
      },
      {
        id: "memory",
        name: "mcp.memory.name",
        description: "mcp.memory.desc",
        tools: ["create_entities", "create_relations", "read_graph"],
        defaultConfig: {
          command: "npx",
          args: ["-y", "@modelcontextprotocol/server-memory"],
        },
      },
      {
        id: "puppeteer",
        name: "mcp.puppeteer.name",
        description: "mcp.puppeteer.desc",
        tools: ["navigate", "screenshot", "click", "fill"],
        defaultConfig: {
          command: "npx",
          args: ["-y", "@modelcontextprotocol/server-puppeteer"],
        },
      },
      {
        id: "github",
        name: "mcp.github.name",
        description: "mcp.github.desc",
        tools: [
          "search_repositories",
          "create_issue",
          "get_pull_request",
          "push_files",
        ],
        defaultConfig: {
          command: "npx",
          args: ["-y", "@modelcontextprotocol/server-github"],
          env: {
            GITHUB_PERSONAL_ACCESS_TOKEN: "${env:GITHUB_TOKEN}",
          },
        },
      },
      {
        id: "sqlite",
        name: "mcp.sqlite.name",
        description: "mcp.sqlite.desc",
        tools: ["query", "list_tables", "describe_table"],
        defaultConfig: {
          command: "npx",
          args: ["-y", "@modelcontextprotocol/server-sqlite", "${dbPath}"],
        },
        configFields: [
          {
            key: "dbPath",
            label: "Database Path",
            defaultValue: "${workspaceFolder}/data/db/whytcard.db",
            type: "path",
            description: "Path to the SQLite database file",
          },
        ],
      },
    ];
  }

  async installServers(servers: Record<string, McpServerConfig>) {
    const config = vscode.workspace.getConfiguration("github.copilot");
    const currentServers = config.get<Record<string, any>>("mcpServers") || {};

    const newServers = { ...currentServers, ...servers };

    await config.update(
      "mcpServers",
      newServers,
      vscode.ConfigurationTarget.Workspace
    );
  }

  async validateServer(config: McpServerConfig): Promise<boolean> {
    // Basic validation: ensure command is not empty
    if (!config.command) return false;
    return true;
  }
}
