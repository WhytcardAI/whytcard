import * as vscode from "vscode";

export interface ProjectInfo {
  name?: string;
  framework?: string;
  language?: string;
  styling?: string;
  database?: string;
  auth?: string;
  deploy?: string;
  testing?: string;
  packageManager?: string;
  dependencies?: string[];
  devDependencies?: string[];
  userDescription?: string; // Description from chat conversation
}

export class ProjectAnalyzer {
  async analyze(workspaceUri: vscode.Uri): Promise<ProjectInfo> {
    const info: ProjectInfo = {};

    // Try to read package.json
    try {
      const packageJsonUri = vscode.Uri.joinPath(workspaceUri, "package.json");
      const content = await vscode.workspace.fs.readFile(packageJsonUri);
      const packageJson = JSON.parse(new TextDecoder().decode(content));

      info.name = packageJson.name;
      info.dependencies = Object.keys(packageJson.dependencies || {});
      info.devDependencies = Object.keys(packageJson.devDependencies || {});

      const allDeps = [
        ...(info.dependencies || []),
        ...(info.devDependencies || []),
      ];

      // Detect framework
      info.framework = this._detectFramework(allDeps);
      info.language = this._detectLanguage(allDeps);
      info.styling = this._detectStyling(allDeps);
      info.database = this._detectDatabase(allDeps);
      info.auth = this._detectAuth(allDeps);
      info.testing = this._detectTesting(allDeps);
      info.packageManager = await this._detectPackageManager(workspaceUri);
    } catch {
      // Expected: package.json not found, will try other detection methods below
    }

    // Try to read requirements.txt (Python)
    if (!info.framework) {
      try {
        const requirementsUri = vscode.Uri.joinPath(
          workspaceUri,
          "requirements.txt"
        );
        const content = await vscode.workspace.fs.readFile(requirementsUri);
        const requirements = new TextDecoder().decode(content);

        info.language = "python";
        info.framework = this._detectPythonFramework(requirements);
        info.database = this._detectPythonDatabase(requirements);
      } catch {
        // Expected: requirements.txt not found, will try other detection methods
      }
    }

    // Try pyproject.toml
    if (!info.framework) {
      try {
        const pyprojectUri = vscode.Uri.joinPath(
          workspaceUri,
          "pyproject.toml"
        );
        const content = await vscode.workspace.fs.readFile(pyprojectUri);
        const pyproject = new TextDecoder().decode(content);

        info.language = "python";
        info.framework = this._detectPythonFramework(pyproject);
      } catch {
        // Expected: pyproject.toml not found, will try other detection methods
      }
    }

    // Try Cargo.toml (Rust)
    if (!info.framework) {
      try {
        const cargoUri = vscode.Uri.joinPath(workspaceUri, "Cargo.toml");
        await vscode.workspace.fs.readFile(cargoUri);
        info.language = "rust";
      } catch {
        // Expected: Cargo.toml not found, will try other detection methods
      }
    }

    // Try go.mod (Go)
    if (!info.framework) {
      try {
        const goModUri = vscode.Uri.joinPath(workspaceUri, "go.mod");
        await vscode.workspace.fs.readFile(goModUri);
        info.language = "go";
      } catch {
        // Expected: go.mod not found, no Go project detected
      }
    }

    return info;
  }

  private _detectFramework(deps: string[]): string | undefined {
    // React-based
    if (deps.includes("next")) return "nextjs";
    if (deps.includes("gatsby")) return "gatsby";
    if (deps.includes("remix")) return "remix";
    if (deps.includes("react")) return "react";

    // Vue-based
    if (deps.includes("nuxt")) return "nuxt";
    if (deps.includes("vue")) return "vue";

    // Angular
    if (deps.includes("@angular/core")) return "angular";

    // Svelte
    if (deps.includes("svelte") || deps.includes("@sveltejs/kit"))
      return "svelte";

    // Solid
    if (deps.includes("solid-js")) return "solid";

    // Node.js backends
    if (deps.includes("express")) return "express";
    if (deps.includes("fastify")) return "fastify";
    if (deps.includes("@nestjs/core")) return "nestjs";
    if (deps.includes("koa")) return "koa";
    if (deps.includes("hono")) return "hono";

    return undefined;
  }

  private _detectLanguage(deps: string[]): string {
    if (deps.includes("typescript") || deps.includes("@types/node")) {
      return "typescript";
    }
    return "javascript";
  }

  private _detectStyling(deps: string[]): string | undefined {
    if (deps.includes("tailwindcss")) return "tailwind";
    if (deps.includes("styled-components")) return "styled-components";
    if (deps.includes("@emotion/react")) return "emotion";
    if (deps.includes("sass") || deps.includes("node-sass")) return "sass";
    if (deps.includes("@mui/material")) return "material-ui";
    if (deps.includes("bootstrap")) return "bootstrap";
    if (deps.includes("chakra-ui") || deps.includes("@chakra-ui/react"))
      return "chakra-ui";

    return undefined;
  }

  private _detectDatabase(deps: string[]): string | undefined {
    if (deps.includes("prisma") || deps.includes("@prisma/client"))
      return "prisma";
    if (deps.includes("drizzle-orm")) return "drizzle";
    if (deps.includes("typeorm")) return "typeorm";
    if (deps.includes("mongoose")) return "mongodb";
    if (deps.includes("pg") || deps.includes("postgres")) return "postgresql";
    if (deps.includes("mysql") || deps.includes("mysql2")) return "mysql";
    if (deps.includes("better-sqlite3") || deps.includes("sqlite3"))
      return "sqlite";
    if (deps.includes("redis") || deps.includes("ioredis")) return "redis";

    return undefined;
  }

  private _detectAuth(deps: string[]): string | undefined {
    if (deps.includes("next-auth") || deps.includes("@auth/core"))
      return "nextauth";
    if (deps.includes("@clerk/nextjs") || deps.includes("@clerk/clerk-react"))
      return "clerk";
    if (deps.includes("auth0")) return "auth0";
    if (deps.includes("passport")) return "passport";
    if (deps.includes("@supabase/supabase-js")) return "supabase";
    if (deps.includes("firebase")) return "firebase";

    return undefined;
  }

  private _detectTesting(deps: string[]): string | undefined {
    if (deps.includes("vitest")) return "vitest";
    if (deps.includes("jest")) return "jest";
    if (deps.includes("mocha")) return "mocha";
    if (deps.includes("@playwright/test")) return "playwright";
    if (deps.includes("cypress")) return "cypress";
    if (deps.includes("@testing-library/react")) return "testing-library";

    return undefined;
  }

  private _detectPythonFramework(content: string): string | undefined {
    const lower = content.toLowerCase();
    if (lower.includes("django")) return "django";
    if (lower.includes("fastapi")) return "fastapi";
    if (lower.includes("flask")) return "flask";
    if (lower.includes("starlette")) return "starlette";
    if (lower.includes("streamlit")) return "streamlit";

    return undefined;
  }

  private _detectPythonDatabase(content: string): string | undefined {
    const lower = content.toLowerCase();
    if (lower.includes("sqlalchemy")) return "sqlalchemy";
    if (lower.includes("django")) return "django-orm";
    if (lower.includes("prisma")) return "prisma";
    if (lower.includes("pymongo")) return "mongodb";
    if (lower.includes("psycopg")) return "postgresql";

    return undefined;
  }

  private async _detectPackageManager(
    workspaceUri: vscode.Uri
  ): Promise<string> {
    // Check for lock files
    try {
      await vscode.workspace.fs.stat(
        vscode.Uri.joinPath(workspaceUri, "bun.lockb")
      );
      return "bun";
    } catch {
      // bun.lockb not found, try next
    }

    try {
      await vscode.workspace.fs.stat(
        vscode.Uri.joinPath(workspaceUri, "pnpm-lock.yaml")
      );
      return "pnpm";
    } catch {
      // pnpm-lock.yaml not found, try next
    }

    try {
      await vscode.workspace.fs.stat(
        vscode.Uri.joinPath(workspaceUri, "yarn.lock")
      );
      return "yarn";
    } catch {
      // yarn.lock not found, try next
    }

    try {
      await vscode.workspace.fs.stat(
        vscode.Uri.joinPath(workspaceUri, "package-lock.json")
      );
      return "npm";
    } catch {
      // package-lock.json not found, default to npm
    }

    return "npm";
  }
}
