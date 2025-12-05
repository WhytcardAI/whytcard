import * as vscode from "vscode";

/**
 * Service for secure storage of API keys and sensitive data
 * Uses VS Code's SecretStorage API which encrypts data
 */

export interface StoredSecrets {
  context7ApiKey?: string;
  tavilyApiKey?: string;
  githubToken?: string;
}

export interface StoredConfig {
  preferredModel?: string;
  language: string;
  configured: boolean;
}

const SECRET_KEYS = {
  context7: "whytcard.secrets.context7ApiKey",
  tavily: "whytcard.secrets.tavilyApiKey",
  github: "whytcard.secrets.githubToken",
} as const;

const CONFIG_KEYS = {
  preferredModel: "whytcard.config.preferredModel",
  language: "whytcard.config.language",
  configured: "whytcard.config.configured",
} as const;

export class SecretsService {
  private secrets: vscode.SecretStorage;
  private globalState: vscode.Memento;

  constructor(context: vscode.ExtensionContext) {
    this.secrets = context.secrets;
    this.globalState = context.globalState;
  }

  // ==================== SECRETS (Encrypted) ====================

  async getContext7ApiKey(): Promise<string | undefined> {
    return this.secrets.get(SECRET_KEYS.context7);
  }

  async setContext7ApiKey(key: string | undefined): Promise<void> {
    if (key) {
      await this.secrets.store(SECRET_KEYS.context7, key);
    } else {
      await this.secrets.delete(SECRET_KEYS.context7);
    }
  }

  async getTavilyApiKey(): Promise<string | undefined> {
    return this.secrets.get(SECRET_KEYS.tavily);
  }

  async setTavilyApiKey(key: string | undefined): Promise<void> {
    if (key) {
      await this.secrets.store(SECRET_KEYS.tavily, key);
    } else {
      await this.secrets.delete(SECRET_KEYS.tavily);
    }
  }

  async getGitHubToken(): Promise<string | undefined> {
    return this.secrets.get(SECRET_KEYS.github);
  }

  async setGitHubToken(token: string | undefined): Promise<void> {
    if (token) {
      await this.secrets.store(SECRET_KEYS.github, token);
    } else {
      await this.secrets.delete(SECRET_KEYS.github);
    }
  }

  async getAllSecrets(): Promise<StoredSecrets> {
    const [context7ApiKey, tavilyApiKey, githubToken] = await Promise.all([
      this.getContext7ApiKey(),
      this.getTavilyApiKey(),
      this.getGitHubToken(),
    ]);
    return { context7ApiKey, tavilyApiKey, githubToken };
  }

  async hasAnyApiKey(): Promise<boolean> {
    const secrets = await this.getAllSecrets();
    return !!(secrets.context7ApiKey || secrets.tavilyApiKey);
  }

  // ==================== CONFIG (Non-sensitive) ====================

  getPreferredModel(): string | undefined {
    return this.globalState.get<string>(CONFIG_KEYS.preferredModel);
  }

  async setPreferredModel(modelId: string | undefined): Promise<void> {
    await this.globalState.update(CONFIG_KEYS.preferredModel, modelId);
  }

  getLanguage(): string {
    return this.globalState.get<string>(CONFIG_KEYS.language) || "en";
  }

  async setLanguage(language: string): Promise<void> {
    await this.globalState.update(CONFIG_KEYS.language, language);
  }

  isConfigured(): boolean {
    return this.globalState.get<boolean>(CONFIG_KEYS.configured) || false;
  }

  async setConfigured(configured: boolean): Promise<void> {
    await this.globalState.update(CONFIG_KEYS.configured, configured);
  }

  getConfig(): StoredConfig {
    return {
      preferredModel: this.getPreferredModel(),
      language: this.getLanguage(),
      configured: this.isConfigured(),
    };
  }

  // ==================== MIGRATION ====================

  /**
   * Migrate old globalState secrets to new SecretStorage
   * Call this once during extension activation
   */
  async migrateFromGlobalState(): Promise<void> {
    // Old keys from previous implementation
    const oldKeys = {
      context7: "whytcard.context7ApiKey",
      tavily: "whytcard.tavilyApiKey",
      github: "whytcard.githubToken",
    };

    // Migrate each key
    for (const [name, oldKey] of Object.entries(oldKeys)) {
      const oldValue = this.globalState.get<string>(oldKey);
      if (oldValue) {
        // Store in secure storage
        const newKey = SECRET_KEYS[name as keyof typeof SECRET_KEYS];
        await this.secrets.store(newKey, oldValue);
        // Remove from globalState
        await this.globalState.update(oldKey, undefined);
        console.log(`Migrated ${name} API key to secure storage`);
      }
    }
  }

  // ==================== CLEAR ALL ====================

  async clearAll(): Promise<void> {
    await Promise.all([
      this.secrets.delete(SECRET_KEYS.context7),
      this.secrets.delete(SECRET_KEYS.tavily),
      this.secrets.delete(SECRET_KEYS.github),
      this.globalState.update(CONFIG_KEYS.preferredModel, undefined),
      this.globalState.update(CONFIG_KEYS.language, undefined),
      this.globalState.update(CONFIG_KEYS.configured, undefined),
    ]);
  }
}
