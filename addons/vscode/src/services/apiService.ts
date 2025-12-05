import * as vscode from "vscode";
import { SecretsService } from "./secretsService";

/**
 * API Service for WhytCard
 * Handles external API calls and MCP tool execution
 */

// ==================== TYPES ====================

export interface ThinkingStep {
  type: "analyzing" | "searching" | "reasoning" | "complete" | "error" | "tool";
  message: string;
  detail?: string;
  source?: string;
}

export interface SearchResult {
  content: string;
  source: string;
  title?: string;
}

export interface ThinkingProcess {
  steps: ThinkingStep[];
  tavilyResults: SearchResult[];
  reasoning: string;
}

// ==================== CONSTANTS ====================

const API_URLS = {
  tavily: "https://api.tavily.com/search",
} as const;

const TECHNOLOGY_PATTERNS: Record<string, string> = {
  react: "react",
  vue: "vue",
  angular: "angular",
  next: "nextjs",
  nextjs: "nextjs",
  nuxt: "nuxt",
  svelte: "svelte",
  express: "express",
  fastify: "fastify",
  nest: "nestjs",
  nestjs: "nestjs",
  django: "django",
  flask: "flask",
  fastapi: "fastapi",
  tailwind: "tailwindcss",
  typescript: "typescript",
  prisma: "prisma",
  mongodb: "mongodb",
  postgres: "postgresql",
  postgresql: "postgresql",
  supabase: "supabase",
  firebase: "firebase",
  "chrome extension": "chrome-extension",
  vscode: "vscode-api",
  electron: "electron",
};

// ==================== API SERVICE CLASS ====================

export class ApiService {
  private secretsService: SecretsService;
  private onThinkingStep?: (step: ThinkingStep) => void;

  constructor(secretsService: SecretsService) {
    this.secretsService = secretsService;
  }

  /**
   * Set callback for thinking steps (for UI updates)
   */
  setThinkingCallback(callback: (step: ThinkingStep) => void): void {
    this.onThinkingStep = callback;
  }

  /**
   * Emit a thinking step to the UI
   */
  private emitStep(step: ThinkingStep): void {
    if (this.onThinkingStep) {
      this.onThinkingStep(step);
    }
  }

  /**
   * Ping the WhytCard Hub to notify connection
   */
  async pingHub(): Promise<boolean> {
    try {
      const response = await fetch(
        "http://localhost:3000/api/ping?source=vscode"
      );
      return response.ok;
    } catch (error) {
      // Silent fail - hub might not be running
      console.debug("Failed to ping WhytCard Hub", error);
      return false;
    }
  }

  // ==================== MAIN SEARCH METHOD ====================

  /**
   * Perform a full thinking process with available sources
   */
  async think(
    query: string,
    projectContext: { framework?: string; language?: string },
    token: vscode.CancellationToken
  ): Promise<ThinkingProcess> {
    const process: ThinkingProcess = {
      steps: [],
      tavilyResults: [],
      reasoning: "",
    };

    // Check cancellation
    if (token.isCancellationRequested) {
      throw new Error("cancelled");
    }

    // Step 1: Analyze the query
    this.emitStep({
      type: "analyzing",
      message: "Understanding your request...",
      detail: query.substring(0, 100),
    });
    process.steps.push({
      type: "analyzing",
      message: "Analyzed the request",
    });

    // Step 2: Build reasoning
    this.emitStep({
      type: "reasoning",
      message: "Putting it all together...",
    });

    process.reasoning = this.buildReasoning(query, process, projectContext);
    process.steps.push({
      type: "complete",
      message: "Ready to respond",
    });

    this.emitStep({
      type: "complete",
      message:
        process.tavilyResults.length > 0
          ? "Found helpful documentation"
          : "Ready to respond",
    });

    return process;
  }

  // ==================== HELPERS ====================

  /**
   * Extract technology names from query and project context
   */
  extractTechnologies(
    query: string,
    projectContext: { framework?: string; language?: string }
  ): string[] {
    const technologies: string[] = [];
    const lowerQuery = query.toLowerCase();

    // Check for known technology patterns
    for (const [pattern, tech] of Object.entries(TECHNOLOGY_PATTERNS)) {
      if (lowerQuery.includes(pattern) && !technologies.includes(tech)) {
        technologies.push(tech);
      }
    }

    // Add from project context
    if (projectContext.framework) {
      const framework = projectContext.framework.toLowerCase();
      const tech = TECHNOLOGY_PATTERNS[framework] || framework;
      if (!technologies.includes(tech)) {
        technologies.push(tech);
      }
    }

    // Default fallback if nothing detected
    if (technologies.length === 0) {
      if (projectContext.language === "typescript") {
        technologies.push("typescript");
      } else if (projectContext.language === "python") {
        technologies.push("python");
      }
    }

    return technologies;
  }

  /**
   * Build a reasoning summary from search results
   */
  private buildReasoning(
    query: string,
    process: ThinkingProcess,
    projectContext: { framework?: string; language?: string }
  ): string {
    const parts: string[] = [];

    // Context summary
    if (projectContext.framework || projectContext.language) {
      parts.push(
        `Project context: ${projectContext.framework || "unknown"} / ${
          projectContext.language || "unknown"
        }`
      );
    }

    // What was found
    if (process.tavilyResults.length > 0) {
      parts.push(`Web results: ${process.tavilyResults.length} relevant pages`);
    } else {
      parts.push("No external documentation found - using internal knowledge");
    }

    return parts.join("\n");
  }

  /**
   * Format all found documentation into a context string for the LLM
   */
  formatDocumentationContext(process: ThinkingProcess): string {
    const sections: string[] = [];

    for (const result of process.tavilyResults) {
      sections.push(
        `### ${result.title || "Web result"}\nSource: ${result.source}\n${
          result.content
        }`
      );
    }

    return sections.join("\n\n---\n\n");
  }

  // ==================== API VALIDATION ====================

  /**
   * Validate Tavily API key
   */
  async validateTavilyKey(
    apiKey: string
  ): Promise<{ valid: boolean; error?: string }> {
    if (!apiKey || apiKey.trim() === "") {
      return { valid: false, error: "API key is empty" };
    }

    try {
      const response = await fetch(API_URLS.tavily, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({
          api_key: apiKey.trim(),
          query: "test query",
          search_depth: "basic",
          max_results: 1,
        }),
      });

      if (response.ok) {
        return { valid: true };
      } else if (response.status === 401 || response.status === 403) {
        return { valid: false, error: "Invalid API key" };
      } else {
        const data = await response.json().catch(() => ({}));
        return {
          valid: false,
          error:
            (data as { detail?: string }).detail ||
            `API error: ${response.status}`,
        };
      }
    } catch (error) {
      return {
        valid: false,
        error: error instanceof Error ? error.message : "Connection error",
      };
    }
  }

  /**
   * Validate GitHub token by checking user info
   */
  async validateGitHubToken(
    token: string
  ): Promise<{ valid: boolean; error?: string; username?: string }> {
    if (!token || token.trim() === "") {
      return { valid: false, error: "Token is empty" };
    }

    try {
      const response = await fetch("https://api.github.com/user", {
        headers: {
          Authorization: `Bearer ${token.trim()}`,
          Accept: "application/vnd.github.v3+json",
        },
      });

      if (response.ok) {
        const data = (await response.json()) as { login?: string };
        return { valid: true, username: data.login };
      } else if (response.status === 401) {
        return { valid: false, error: "Invalid or expired token" };
      } else {
        return { valid: false, error: `GitHub API error: ${response.status}` };
      }
    } catch (error) {
      return {
        valid: false,
        error: error instanceof Error ? error.message : "Connection error",
      };
    }
  }
}
