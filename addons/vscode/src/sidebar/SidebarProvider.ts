import * as vscode from "vscode";
import { ProjectAnalyzer, ProjectInfo } from "../core/projectAnalyzer";
import {
  SecretsService,
  ApiService,
  ThinkingStep,
  McpService,
  McpServerConfig,
  HubService,
  ChatResponseEvent,
  SessionSyncEvent,
} from "../services";
import { i18n, SupportedLanguage } from "../i18n";
import { locales } from "../i18n/locales";

// ==================== TYPES ====================

interface ChatMessage {
  role: "user" | "assistant" | "thinking";
  content: string;
  timestamp: number;
  thinkingSteps?: ThinkingStep[];
  isCollapsed?: boolean;
}

interface AvailableModel {
  id: string;
  name: string;
  vendor: string;
  family: string;
}

// ==================== MAIN CLASS ====================

export class SidebarProvider implements vscode.WebviewViewProvider {
  public static readonly viewType = "whytcard.sidebar";

  private _view?: vscode.WebviewView;
  private _context: vscode.ExtensionContext;
  private _secretsService: SecretsService;
  private _apiService: ApiService;
  private _mcpService: McpService;
  private _hubService: HubService;
  private _projectInfo?: ProjectInfo;
  private _chatHistory: ChatMessage[] = [];
  private _checklist: string[] = [];
  private _availableModels: AvailableModel[] = [];
  private _githubSession?: vscode.AuthenticationSession;
  private _currentCancellation?: vscode.CancellationTokenSource;
  private _streamingContent = "";
  private _isStreaming = false;

  constructor(
    context: vscode.ExtensionContext,
    secretsService: SecretsService,
    mcpService: McpService,
    hubService: HubService
  ) {
    this._context = context;
    this._secretsService = secretsService;
    this._mcpService = mcpService;
    this._hubService = hubService;
    this._apiService = new ApiService(secretsService);

    // Set up thinking callback to update UI
    this._apiService.setThinkingCallback((step) => {
      this._postMessage({ type: "thinkingStep", step });
    });

    // Set up Hub SSE event listeners
    this._setupHubEventListeners();

    // Load available models and restore GitHub session
    this._loadAvailableModels();
    this._restoreGitHubSession();
  }

  // ==================== HUB SSE EVENT HANDLING ====================

  private _setupHubEventListeners(): void {
    // Connection events
    this._hubService.on("connected", () => {
      console.log("[SidebarProvider] Hub SSE connected");
      this._postMessage({ type: "hubConnected" });
    });

    this._hubService.on("disconnected", () => {
      console.log("[SidebarProvider] Hub SSE disconnected");
      this._postMessage({ type: "hubDisconnected" });
    });

    // Chat response streaming
    this._hubService.on("chat_response", (data) => {
      const event = data as ChatResponseEvent;
      this._handleChatResponseEvent(event);
    });

    // Session sync (history from other clients)
    this._hubService.on("session_sync", (data) => {
      const event = data as SessionSyncEvent;
      this._handleSessionSyncEvent(event);
    });

    // Error events
    this._hubService.on("error", (data) => {
      console.error("[SidebarProvider] Hub error:", data);
      this._postMessage({
        type: "error",
        message: "Hub connection error",
      });
    });
  }

  private _handleChatResponseEvent(event: ChatResponseEvent): void {
    if (event.is_final) {
      // Final message - add to history
      if (this._isStreaming) {
        this._chatHistory.push({
          role: "assistant",
          content: this._streamingContent || event.content,
          timestamp: Date.now(),
          isCollapsed: false,
        });
        this._streamingContent = "";
        this._isStreaming = false;
      }

      this._postMessage({
        type: "assistantMessage",
        content: event.content,
        isFinal: true,
        thinkingSteps: [],
      });
      this._postMessage({ type: "thinking", isThinking: false });
    } else {
      // Streaming chunk
      this._isStreaming = true;
      this._streamingContent += event.content;

      this._postMessage({
        type: "streamingChunk",
        content: event.content,
        fullContent: this._streamingContent,
      });
    }
  }

  private _handleSessionSyncEvent(event: SessionSyncEvent): void {
    // Sync chat history from Hub
    this._chatHistory = event.messages.map((msg) => ({
      role: msg.role as "user" | "assistant",
      content: msg.content,
      timestamp: msg.timestamp,
      isCollapsed: false,
    }));

    this._postMessage({
      type: "sessionSync",
      messages: this._chatHistory,
    });
  }

  // ==================== INITIALIZATION ====================

  private async _restoreGitHubSession(): Promise<void> {
    try {
      // Try to get existing GitHub session without prompting
      const session = await vscode.authentication.getSession(
        "github",
        ["user", "repo"],
        { createIfNone: false }
      );
      if (session) {
        this._githubSession = session;
        // Update the token in secrets
        await this._secretsService.setGitHubToken(session.accessToken);
      }
    } catch (error) {
      console.error("Failed to restore GitHub session:", error);
    }
  }

  private async _loadAvailableModels(): Promise<void> {
    try {
      const models = await vscode.lm.selectChatModels();
      this._availableModels = models.map((m) => ({
        id: m.id,
        name: `${m.name} (${m.vendor})`,
        vendor: m.vendor,
        family: m.family,
      }));

      const currentModel = this._secretsService.getPreferredModel();
      if (!currentModel && this._availableModels.length > 0) {
        await this._secretsService.setPreferredModel(
          this._availableModels[0].id
        );
      }

      if (this._view && !this._secretsService.isConfigured()) {
        this._view.webview.html = await this._getSetupHtml(this._view.webview);
      }
    } catch (error) {
      console.error("Failed to load models:", error);
      this._availableModels = [];
    }
  }

  // ==================== WEBVIEW PROVIDER ====================

  public resolveWebviewView(
    webviewView: vscode.WebviewView,
    _context: vscode.WebviewViewResolveContext,
    _token: vscode.CancellationToken
  ): void {
    this._view = webviewView;

    webviewView.webview.options = {
      enableScripts: true,
      localResourceRoots: [
        vscode.Uri.joinPath(this._context.extensionUri, "media"),
      ],
    };

    // Wait a bit for GitHub session to be restored before showing UI
    setTimeout(() => this._updateView(), 100);

    webviewView.webview.onDidReceiveMessage(async (message) => {
      await this._handleMessage(message);
    });
  }

  private async _updateView(): Promise<void> {
    if (!this._view) return;

    if (!this._secretsService.isConfigured()) {
      this._view.webview.html = await this._getSetupHtml(this._view.webview);
    } else {
      this._view.webview.html = this._getChatHtml(this._view.webview);
      this._initializeChat();
    }
  }

  // ==================== MESSAGE HANDLING ====================

  private async _handleMessage(message: unknown): Promise<void> {
    // eslint-disable-next-line @typescript-eslint/no-explicit-any
    const msg = message as any;
    switch (msg.type) {
      case "saveConfig":
        await this._saveConfig(msg.config);
        break;

      case "validateApiKey":
        await this._handleValidateApiKey(msg.keyType, msg.value);
        break;

      case "skipSetup":
        await this._secretsService.setConfigured(true);
        this._updateView();
        break;

      case "openSettings":
        await this._secretsService.setConfigured(false);
        this._updateView();
        break;

      case "connectGitHub":
        await this._handleGitHubConnect();
        break;

      case "connectWhytCard":
        await this._handleWhytCardConnect();
        break;

      case "autoConnectWhytCard":
        await this._handleAutoConnectWhytCard();
        break;

      case "setLanguage":
        await this._secretsService.setLanguage(msg.language);
        i18n.setLanguage(msg.language);
        this._updateView();
        break;

      case "sendMessage":
        await this._handleUserMessage(msg.content);
        break;

      case "cancelThinking":
        this._cancelCurrentOperation();
        break;

      case "toggleThinking":
        this._toggleThinkingCollapse(msg.messageIndex);
        break;

      case "openFile":
        await this._openFile(msg.path);
        break;

      case "newChat":
        this._chatHistory = [];
        this._streamingContent = "";
        this._isStreaming = false;
        this._hubService.setSessionId(null);
        this._updateView();
        break;

      case "openExternal":
        vscode.env.openExternal(vscode.Uri.parse(msg.url));
        break;

      case "connectHub":
        await this._handleConnectHub();
        break;

      case "getHubStatus":
        await this._handleGetHubStatus();
        break;
    }
  }

  private async _saveConfig(config: {
    preferredModel?: string;
    selectedMcps?: string[];
    mcpConfigs?: Record<string, Record<string, string>>;
  }): Promise<void> {
    if (config.preferredModel) {
      await this._secretsService.setPreferredModel(config.preferredModel);
    }

    // Install selected MCP servers
    if (config.selectedMcps) {
      const available = this._mcpService.getAvailableServers();
      const toInstall: Record<string, McpServerConfig> = {};

      for (const id of config.selectedMcps) {
        const serverDef = available.find((s) => s.id === id);
        if (serverDef) {
          // Deep copy default config
          const finalConfig = JSON.parse(
            JSON.stringify(serverDef.defaultConfig)
          );
          const userConfig = (config.mcpConfigs && config.mcpConfigs[id]) || {};

          // Helper to replace variables
          const replaceVars = (str: string) => {
            let result = str;
            // Replace variables from user config
            for (const [key, value] of Object.entries(userConfig)) {
              result = result.replace(
                new RegExp(`\\$\\{${key}\\}`, "g"),
                value as string
              );
            }
            // Replace variables from default values if not provided by user
            if (serverDef.configFields) {
              for (const field of serverDef.configFields) {
                if (!userConfig[field.key] && field.defaultValue) {
                  result = result.replace(
                    new RegExp(`\\$\\{${field.key}\\}`, "g"),
                    field.defaultValue
                  );
                }
              }
            }
            return result;
          };

          // Process args
          if (finalConfig.args) {
            finalConfig.args = finalConfig.args.map(replaceVars);
          }

          // Process env
          if (finalConfig.env) {
            for (const key in finalConfig.env) {
              finalConfig.env[key] = replaceVars(finalConfig.env[key]);
            }
          }

          toInstall[id] = finalConfig;
        }
      }

      await this._mcpService.installServers(toInstall);
    }

    await this._secretsService.setConfigured(true);
    this._updateView();
    vscode.window.showInformationMessage(i18n.t("response.done"));
  }

  private async _handleValidateApiKey(
    keyType: string,
    _value: string
  ): Promise<void> {
    this._postMessage({ type: "validating", keyType, status: "checking" });

    let result: { valid: boolean; error?: string };

    switch (keyType) {
      default:
        result = { valid: false, error: "Unknown key type" };
    }

    this._postMessage({
      type: "validationResult",
      keyType,
      valid: result.valid,
      error: result.error,
    });
  }

  // ==================== GITHUB AUTH ====================

  private async _handleGitHubConnect(): Promise<void> {
    try {
      const session = await vscode.authentication.getSession(
        "github",
        ["user", "repo"],
        {
          createIfNone: true,
        }
      );

      if (session) {
        this._githubSession = session;
        await this._secretsService.setGitHubToken(session.accessToken);
        vscode.window.showInformationMessage(
          `Connected to GitHub as ${session.account.label}`
        );
        // Notify webview about the connection
        this._postMessage({
          type: "githubConnected",
          username: session.account.label,
        });
        await this._loadAvailableModels();
      }
    } catch (error) {
      console.error("GitHub auth error:", error);
      vscode.window.showErrorMessage("Failed to connect to GitHub");
      this._postMessage({
        type: "githubError",
        error: "Failed to connect to GitHub",
      });
    }
    this._updateView();
  }

  private async _handleWhytCardConnect(): Promise<void> {
    this._postMessage({
      type: "validating",
      keyType: "whytcard",
      status: "checking",
    });

    const isConnected = await this._apiService.pingHub();

    if (isConnected) {
      // Add WhytCard Local Model to available models if not present
      const localModelId = "whytcard-local-gguf";
      const localModelName = "WhytCard Local Model (GGUF)";

      const exists = this._availableModels.some((m) => m.id === localModelId);
      if (!exists) {
        this._availableModels.unshift({
          id: localModelId,
          name: localModelName,
          vendor: "WhytCard",
          family: "Local LLM",
        });
      }

      // Set as preferred model
      await this._secretsService.setPreferredModel(localModelId);

      // Start SSE connection
      await this._hubService.connect();

      vscode.window.showInformationMessage(
        "Connected to WhytCard Hub! Local model selected."
      );
      this._postMessage({ type: "whytcardConnected", modelId: localModelId });
    } else {
      vscode.window.showErrorMessage(
        "Could not connect to WhytCard Hub. Is the app running?"
      );
      this._postMessage({ type: "whytcardError" });
    }

    this._updateView();
  }

  private async _handleAutoConnectWhytCard(): Promise<void> {
    const isConnected = await this._apiService.pingHub();

    if (isConnected) {
      // Add WhytCard Local Model to available models if not present
      const localModelId = "whytcard-local-gguf";
      const localModelName = "WhytCard Local Model (GGUF)";

      const exists = this._availableModels.some((m) => m.id === localModelId);
      if (!exists) {
        this._availableModels.unshift({
          id: localModelId,
          name: localModelName,
          vendor: "WhytCard",
          family: "Local LLM",
        });
      }

      // Set as preferred model
      await this._secretsService.setPreferredModel(localModelId);

      // Start SSE connection
      await this._hubService.connect();

      vscode.window.showInformationMessage("Auto-connected to WhytCard Hub!");
      this._postMessage({ type: "whytcardConnected", modelId: localModelId });
      this._updateView();
    }
  }

  private async _handleConnectHub(): Promise<void> {
    const isHubAvailable = await this._hubService.ping();

    if (isHubAvailable) {
      await this._hubService.connect();
      this._postMessage({
        type: "hubConnected",
        isConnected: true,
      });
    } else {
      this._postMessage({
        type: "hubDisconnected",
        isConnected: false,
        error: "Hub not available",
      });
    }
  }

  private async _handleGetHubStatus(): Promise<void> {
    const isConnected = this._hubService.getIsConnected();
    const status = await this._hubService.getSSEStatus();

    this._postMessage({
      type: "hubStatus",
      isConnected,
      clientsCount: status?.connected_clients || 0,
    });
  }

  // ==================== CHAT LOGIC ====================

  private async _initializeChat(): Promise<void> {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (workspaceFolder) {
      try {
        const analyzer = new ProjectAnalyzer();
        this._projectInfo = await analyzer.analyze(workspaceFolder.uri);
        this._postMessage({
          type: "projectAnalyzed",
          projectInfo: this._projectInfo,
        });
      } catch (error) {
        console.error("Failed to analyze project:", error);
      }
    }

    const hasApis = await this._secretsService.hasAnyApiKey();
    if (!hasApis) {
      this._postMessage({ type: "noApiWarning" });
    }
  }

  private async _handleUserMessage(content: string): Promise<void> {
    // Add user message to history
    this._chatHistory.push({ role: "user", content, timestamp: Date.now() });

    // Post user message to webview
    this._postMessage({
      type: "userMessage",
      content,
      timestamp: Date.now(),
    });

    // Create cancellation token
    this._currentCancellation = new vscode.CancellationTokenSource();

    // Show thinking state
    this._postMessage({ type: "thinking", isThinking: true });

    // Reset streaming state
    this._streamingContent = "";
    this._isStreaming = false;

    try {
      // Generate unique session ID if not set
      if (!this._hubService.getSessionId()) {
        const sessionId = `vscode-${Date.now()}-${Math.random()
          .toString(36)
          .substring(7)}`;
        this._hubService.setSessionId(sessionId);
      }

      // Use HubService to send message
      // If SSE is connected, we will receive the response via SSE events
      // Otherwise, we use the direct API call
      if (this._hubService.getIsConnected()) {
        // SSE is connected - response will come via chat_response events
        const response = await this._hubService.sendChatMessage(
          content,
          this._hubService.getSessionId() || undefined
        );

        // For non-streaming response, handle it directly
        if (!this._isStreaming) {
          this._handleChatResponse(response.reply);
        }
      } else {
        // Fallback to direct API call (no SSE)
        const response = await fetch("http://localhost:3000/api/chat", {
          method: "POST",
          headers: { "Content-Type": "application/json" },
          body: JSON.stringify({
            message: content,
            session_id: this._hubService.getSessionId() || "vscode-session",
          }),
        });

        if (!response.ok) {
          throw new Error(`Hub error: ${response.statusText}`);
        }

        const data = (await response.json()) as { reply: string };
        this._handleChatResponse(data.reply);
      }
    } catch (error) {
      const errorMessage =
        error instanceof Error ? error.message : String(error);

      this._postMessage({
        type: "assistantMessage",
        content: `${i18n.t(
          "response.error"
        )} ${errorMessage}. Is the WhytCard Hub running?`,
        thinkingSteps: [],
      });
      this._postMessage({ type: "thinking", isThinking: false });
    } finally {
      this._currentCancellation = undefined;
    }
  }

  private _handleChatResponse(reply: string): void {
    // Parse checklist updates
    let cleanReply = reply;
    const checklistMatch = reply.match(/:::checklist ({.*}) :::/);
    if (checklistMatch) {
      try {
        const updates = JSON.parse(checklistMatch[1]) as {
          add?: string[];
          remove?: string[];
        };
        if (updates.add) {
          for (const item of updates.add) {
            if (!this._checklist.includes(item)) {
              this._checklist.push(item);
            }
          }
        }
        if (updates.remove) {
          this._checklist = this._checklist.filter(
            (item) => !updates.remove?.includes(item)
          );
        }

        this._postMessage({
          type: "updateChecklist",
          checklist: this._checklist,
        });

        // Remove the block from the displayed message
        cleanReply = reply.replace(checklistMatch[0], "").trim();
      } catch (error) {
        console.error("Failed to parse checklist update:", error);
      }
    }

    // Add to history
    this._chatHistory.push({
      role: "assistant",
      content: cleanReply,
      timestamp: Date.now(),
      isCollapsed: false,
    });

    this._postMessage({
      type: "assistantMessage",
      content: cleanReply,
      thinkingSteps: [],
    });

    this._postMessage({ type: "thinking", isThinking: false });
  }

  private _cancelCurrentOperation(): void {
    if (this._currentCancellation) {
      this._currentCancellation.cancel();
    }
  }

  private _toggleThinkingCollapse(messageIndex: number): void {
    if (this._chatHistory[messageIndex]) {
      this._chatHistory[messageIndex].isCollapsed =
        !this._chatHistory[messageIndex].isCollapsed;
    }
  }

  // ==================== HELPERS ====================

  private async _openFile(relativePath: string): Promise<void> {
    const workspaceFolder = vscode.workspace.workspaceFolders?.[0];
    if (!workspaceFolder) return;

    const uri = vscode.Uri.joinPath(workspaceFolder.uri, relativePath);
    try {
      await vscode.window.showTextDocument(uri);
    } catch (error) {
      console.error(`Failed to open file ${relativePath}:`, error);
    }
  }

  // eslint-disable-next-line @typescript-eslint/no-explicit-any
  private _postMessage(message: any): void {
    this._view?.webview.postMessage(message);
  }

  private _getNonce(): string {
    let text = "";
    const chars =
      "ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789";
    for (let i = 0; i < 32; i++) {
      text += chars.charAt(Math.floor(Math.random() * chars.length));
    }
    return text;
  }

  // ==================== PUBLIC METHODS ====================

  public startInitialization(): void {
    this._updateView();
  }

  public async quickGenerateInstructions(): Promise<void> {
    // Deprecated
  }

  // ==================== HTML GENERATION ====================

  private async _getSetupHtml(webview: vscode.Webview): Promise<string> {
    const nonce = this._getNonce();
    const lang = this._secretsService.getLanguage() as SupportedLanguage;
    const t = locales[lang] || locales.en;

    // Load existing API keys and GitHub session
    const hasGitHub = !!this._githubSession;
    const githubUsername = this._githubSession?.account?.label || "";
    const currentModel = this._secretsService.getPreferredModel();
    const availableMcps = this._mcpService.getAvailableServers();
    const isHubConnected = await this._apiService.pingHub();

    // Translations
    const validateText =
      lang === "fr"
        ? "Vérifier"
        : lang === "es"
        ? "Verificar"
        : lang === "de"
        ? "Prüfen"
        : "Verify";
    const validatingText =
      lang === "fr"
        ? "Vérification..."
        : lang === "es"
        ? "Verificando..."
        : lang === "de"
        ? "Prüfen..."
        : "Checking...";
    const validText =
      lang === "fr"
        ? "Valide"
        : lang === "es"
        ? "Válida"
        : lang === "de"
        ? "Gültig"
        : "Valid";
    const invalidText =
      lang === "fr"
        ? "Invalide"
        : lang === "es"
        ? "Inválida"
        : lang === "de"
        ? "Ungültig"
        : "Invalid";
    const connectGitHubText =
      lang === "fr"
        ? "Se connecter à GitHub"
        : lang === "es"
        ? "Conectar a GitHub"
        : lang === "de"
        ? "Mit GitHub verbinden"
        : "Connect to GitHub";
    const connectWhytCardText =
      lang === "fr"
        ? "Se connecter à WhytCard"
        : lang === "es"
        ? "Conectar a WhytCard"
        : lang === "de"
        ? "Mit WhytCard verbinden"
        : "Connect to WhytCard";
    const connectedAsText =
      lang === "fr"
        ? "Connecté en tant que"
        : lang === "es"
        ? "Conectado como"
        : lang === "de"
        ? "Verbunden als"
        : "Connected as";
    const githubDesc =
      lang === "fr"
        ? "Accès aux dépôts et opérations GitHub"
        : "Access to repositories and GitHub operations";
    const mcpDesc =
      lang === "fr"
        ? "Outils pour donner plus de capacités à WhytCard"
        : "Tools to give WhytCard more capabilities";
    const toolsLabel = lang === "fr" ? "Outils inclus :" : "Included tools:";
    const configLabel = lang === "fr" ? "Configuration :" : "Configuration:";

    return `<!DOCTYPE html>
<html lang="${lang}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${
      webview.cspSource
    } 'unsafe-inline'; script-src 'nonce-${nonce}';">
    <title>WhytCard Setup</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: var(--vscode-font-family);
            color: var(--vscode-foreground);
            background: var(--vscode-sideBar-background);
            padding: 20px;
            font-size: 13px;
            line-height: 1.6;
        }
        .header { text-align: center; margin-bottom: 24px; }
        .header h1 { font-size: 18px; margin-bottom: 8px; }
        .header p { color: var(--vscode-descriptionForeground); font-size: 12px; }
        .hidden { display: none !important; }

        .lang-row { display: flex; justify-content: center; gap: 8px; margin-bottom: 20px; }
        .lang-btn {
            padding: 6px 12px;
            background: var(--vscode-button-secondaryBackground);
            color: var(--vscode-button-secondaryForeground);
            border: 1px solid var(--vscode-widget-border);
            border-radius: 4px;
            cursor: pointer;
            font-size: 12px;
        }
        .lang-btn.active {
            background: var(--vscode-button-background);
            color: var(--vscode-button-foreground);
            border-color: var(--vscode-button-background);
        }

        .form-group { margin-bottom: 20px; }
        .form-group label { display: block; font-weight: 600; margin-bottom: 4px; }
        .form-group .desc { font-size: 11px; color: var(--vscode-descriptionForeground); margin-bottom: 8px; }
        .form-group input:not([type="checkbox"]), .form-group select {
            width: 100%;
            padding: 10px;
            background: var(--vscode-input-background);
            color: var(--vscode-input-foreground);
            border: 1px solid var(--vscode-input-border);
            border-radius: 6px;
            font-size: 13px;
        }
        .input-row { display: flex; gap: 8px; align-items: center; }
        .input-row input { flex: 1; }

        /* MCP Styles */
        .mcp-list { display: flex; flex-direction: column; gap: 4px; }
        .mcp-item-container {
            border: 1px solid var(--vscode-widget-border);
            border-radius: 4px;
            background: var(--vscode-editor-background);
            overflow: hidden;
        }
        .mcp-item-header {
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 6px 8px;
            cursor: pointer;
            position: relative;
        }
        .mcp-item-header:hover {
            background: var(--vscode-list-hoverBackground);
        }
        .arrow-toggle {
            font-size: 10px;
            width: 14px;
            height: 14px;
            line-height: 14px;
            text-align: center;
            transition: transform 0.2s;
            color: var(--vscode-foreground);
            display: inline-block;
            flex-shrink: 0;
        }
        .arrow-toggle.open {
            transform: rotate(90deg);
        }
        .mcp-checkbox {
            width: 14px !important;
            height: 14px !important;
            margin: 0;
            cursor: pointer;
            flex-shrink: 0;
        }
        .mcp-info { flex: 1; min-width: 0; }
        .mcp-name { font-weight: 600; font-size: 11px; color: var(--vscode-foreground); }
        .mcp-desc { font-size: 10px; color: var(--vscode-descriptionForeground); margin-top: 1px; }

        .mcp-details {
            padding: 8px 8px 8px 28px;
            border-top: 1px solid var(--vscode-widget-border);
            background: var(--vscode-sideBar-background);
            display: none;
        }
        .mcp-details.open {
            display: block;
        }

        .tools-section { margin-bottom: 10px; }
        .section-label {
            font-size: 10px;
            text-transform: uppercase;
            color: var(--vscode-descriptionForeground);
            margin-bottom: 4px;
            font-weight: 600;
        }
        .tools-list {
            display: flex;
            flex-wrap: wrap;
            gap: 4px;
        }
        .tool-badge {
            display: inline-block;
            padding: 2px 6px;
            background: var(--vscode-badge-background);
            color: var(--vscode-badge-foreground);
            border-radius: 3px;
            font-size: 10px;
            font-family: monospace;
        }

        .config-section { margin-top: 8px; }
        .config-field { margin-bottom: 8px; }
        .config-field label {
            display: block;
            font-size: 11px;
            margin-bottom: 2px;
            font-weight: normal;
        }
        .config-field input {
            padding: 6px;
            font-size: 12px;
        }

        .optional-badge { font-size: 10px; color: var(--vscode-descriptionForeground); font-weight: normal; margin-left: 8px; }
        .configured-badge { font-size: 10px; color: var(--vscode-testing-iconPassed); font-weight: normal; margin-left: 8px; }
        .configured-input { border-color: var(--vscode-testing-iconPassed) !important; }
        .error-input { border-color: var(--vscode-inputValidation-errorBorder) !important; }

        .btn {
            width: 100%;
            padding: 12px;
            border: none;
            border-radius: 8px;
            font-size: 14px;
            font-weight: 600;
            cursor: pointer;
            margin-top: 8px;
        }
        .btn-primary { background: var(--vscode-button-background); color: var(--vscode-button-foreground); }
        .btn-primary:hover { background: var(--vscode-button-hoverBackground); }
        .btn-primary:disabled { opacity: 0.5; cursor: not-allowed; }
        .btn-secondary { background: transparent; color: var(--vscode-descriptionForeground); font-weight: normal; font-size: 12px; }

        .btn-validate {
            padding: 10px 14px;
            background: var(--vscode-button-secondaryBackground);
            color: var(--vscode-button-secondaryForeground);
            border: 1px solid var(--vscode-widget-border);
            border-radius: 6px;
            cursor: pointer;
            font-size: 11px;
            white-space: nowrap;
            min-width: 80px;
        }
        .btn-validate:hover { background: var(--vscode-button-secondaryHoverBackground); }
        .btn-validate:disabled { opacity: 0.5; cursor: not-allowed; }
        .btn-validate.valid { background: var(--vscode-testing-iconPassed); color: white; border-color: var(--vscode-testing-iconPassed); }
        .btn-validate.invalid { background: var(--vscode-inputValidation-errorBackground); color: var(--vscode-inputValidation-errorForeground); border-color: var(--vscode-inputValidation-errorBorder); }

        .btn-github {
            display: flex;
            align-items: center;
            justify-content: center;
            gap: 8px;
            padding: 10px 16px;
            background: #238636;
            color: white;
            border: none;
            border-radius: 6px;
            cursor: pointer;
            font-size: 13px;
            font-weight: 500;
        }
        .btn-github:hover { background: #2ea043; }
        .btn-github.connected { background: var(--vscode-testing-iconPassed); }
        .btn-github svg { width: 16px; height: 16px; fill: currentColor; }

        .github-status {
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 10px;
            background: var(--vscode-editor-background);
            border-radius: 6px;
            font-size: 12px;
        }
        .github-status .avatar {
            width: 24px;
            height: 24px;
            border-radius: 50%;
            background: var(--vscode-button-secondaryBackground);
            display: flex;
            align-items: center;
            justify-content: center;
            font-weight: 600;
        }

        .link-btn {
            padding: 10px 12px;
            background: var(--vscode-button-secondaryBackground);
            color: var(--vscode-textLink-foreground);
            border-radius: 6px;
            text-decoration: none;
            font-size: 11px;
            white-space: nowrap;
            cursor: pointer;
            border: none;
        }

        .divider { border-top: 1px solid var(--vscode-widget-border); margin: 24px 0; }
        .status-text { font-size: 10px; margin-top: 4px; }
        .status-text.success { color: var(--vscode-testing-iconPassed); }
        .status-text.error { color: var(--vscode-inputValidation-errorForeground); }

        .api-section { margin-bottom: 16px; }
        .api-section h3 { font-size: 12px; color: var(--vscode-descriptionForeground); margin-bottom: 12px; text-transform: uppercase; letter-spacing: 0.5px; }
    </style>
</head>
<body>
    <div class="header">
        <h1>${t["setup.title"]} (v${new Date().toLocaleTimeString()})</h1>
        <p>${t["setup.description"]}</p>
    </div>

    <div class="lang-row">
        <button class="lang-btn ${
          lang === "en" ? "active" : ""
        }" data-lang="en">EN</button>
        <button class="lang-btn ${
          lang === "fr" ? "active" : ""
        }" data-lang="fr">FR</button>
        <button class="lang-btn ${
          lang === "es" ? "active" : ""
        }" data-lang="es">ES</button>
        <button class="lang-btn ${
          lang === "de" ? "active" : ""
        }" data-lang="de">DE</button>
    </div>

    <div class="api-section">
        <h3>WhytCard Hub</h3>
        <div class="form-group">
            <label>Status
                <span class="${
                  isHubConnected ? "configured-badge" : "hidden"
                }">[OK] ${validText}</span>
                <span class="${
                  !isHubConnected ? "optional-badge" : "hidden"
                }" style="color: var(--vscode-inputValidation-errorForeground)">[OFFLINE]</span>
            </label>
            <div class="desc">Connection to local WhytCard Desktop App (Port 3000)</div>
        </div>
    </div>

    <div class="api-section">
        <h3>MCP Servers</h3>
        <div class="form-group">
            <div class="desc">${mcpDesc}</div>
            <div class="mcp-list">
                ${availableMcps
                  .map(
                    (mcp) => `
                <div class="mcp-item-container">
                    <div class="mcp-item-header" data-mcp-toggle="${mcp.id}">
                        <span class="arrow-toggle" id="arrow-${mcp.id}">▶</span>
                        <input type="checkbox" class="mcp-checkbox" id="mcp-${
                          mcp.id
                        }" value="${mcp.id}"
                            ${
                              mcp.id === "filesystem"
                                ? "checked disabled"
                                : "checked"
                            }
                            onclick="event.stopPropagation()">
                        <div class="mcp-info">
                            <div class="mcp-name">${
                              t[mcp.name] || mcp.name || mcp.id
                            }</div>
                            <div class="mcp-desc">${
                              t[mcp.description] || mcp.description || ""
                            }</div>
                        </div>
                    </div>
                    <div class="mcp-details" id="details-${mcp.id}">
                        ${
                          mcp.tools && mcp.tools.length > 0
                            ? `
                        <div class="tools-section">
                            <div class="section-label">${toolsLabel}</div>
                            <div class="tools-list">
                                ${mcp.tools
                                  .map(
                                    (tool) =>
                                      `<span class="tool-badge">${tool}</span>`
                                  )
                                  .join("")}
                            </div>
                        </div>
                        `
                            : ""
                        }

                        ${
                          mcp.configFields && mcp.configFields.length > 0
                            ? `
                        <div class="config-section">
                            <div class="section-label">${configLabel}</div>
                            ${mcp.configFields
                              .map(
                                (field) => `
                            <div class="config-field">
                                <label>${field.label}${
                                  field.required ? " *" : ""
                                }</label>
                                <input type="${field.type}"
                                       class="mcp-config-input"
                                       data-mcp="${mcp.id}"
                                       data-key="${field.key}"
                                       placeholder="${field.placeholder || ""}"
                                       ${field.required ? "required" : ""}>
                            </div>
                            `
                              )
                              .join("")}
                        </div>
                        `
                            : ""
                        }
                    </div>
                </div>
                `
                  )
                  .join("")}
            </div>
        </div>
    </div>

    <div class="api-section">
        <h3>GitHub</h3>
        <div class="form-group">
            <label>GitHub <span class="optional-badge">${
              t["setup.optional"]
            }</span>
                <span id="githubBadge" class="${
                  hasGitHub ? "configured-badge" : "hidden"
                }">[OK] ${validText}</span>
            </label>
            <div class="desc">${githubDesc}</div>
            ${
              hasGitHub
                ? `
            <div class="github-status">
                <div class="avatar">${githubUsername
                  .charAt(0)
                  .toUpperCase()}</div>
                <span>${connectedAsText} <strong>${githubUsername}</strong></span>
            </div>
            `
                : `
            <button class="btn-github" id="connectGitHub">
                <svg viewBox="0 0 16 16"><path d="M8 0C3.58 0 0 3.58 0 8c0 3.54 2.29 6.53 5.47 7.59.4.07.55-.17.55-.38 0-.19-.01-.82-.01-1.49-2.01.37-2.53-.49-2.69-.94-.09-.23-.48-.94-.82-1.13-.28-.15-.68-.52-.01-.53.63-.01 1.08.58 1.23.82.72 1.21 1.87.87 2.33.66.07-.52.28-.87.51-1.07-1.78-.2-3.64-.89-3.64-3.95 0-.87.31-1.59.82-2.15-.08-.2-.36-1.02.08-2.12 0 0 .67-.21 2.2.82.64-.18 1.32-.27 2-.27.68 0 1.36.09 2 .27 1.53-1.04 2.2-.82 2.2-.82.44 1.1.16 1.92.08 2.12.51.56.82 1.27.82 2.15 0 3.07-1.87 3.75-3.65 3.95.29.25.54.73.54 1.48 0 1.07-.01 1.93-.01 2.2 0 .21.15.46.55.38A8.013 8.013 0 0016 8c0-4.42-3.58-8-8-8z"/></svg>
                ${connectGitHubText}
            </button>
            `
            }
            <div id="githubStatus" class="status-text"></div>
        </div>

        <div class="form-group" style="margin-top: 12px;">
            ${
              isHubConnected
                ? `
            <div class="github-status" style="background: var(--vscode-button-secondaryBackground); color: var(--vscode-button-secondaryForeground);">
                <div class="avatar" style="background: var(--vscode-testing-iconPassed); color: white;">W</div>
                <span>Connected to <strong>WhytCard Hub</strong></span>
            </div>
            `
                : `
            <button class="btn-github" id="connectWhytCard" style="background: #007acc;">
                <svg viewBox="0 0 16 16"><path d="M8 0a8 8 0 100 16A8 8 0 008 0zm3.5 11.5l-3.5-2-3.5 2 1-4-3-2h4l2-4 2 4h4l-3 2 1 4z"/></svg>
                ${connectWhytCardText}
            </button>
            `
            }
        </div>
    </div>

    <div class="divider"></div>

    <div class="form-group">
        <label>${t["setup.modelLabel"]}</label>
        <div class="desc">${t["setup.modelDesc"]}</div>
        <select id="modelSelect">
            ${
              this._availableModels.length > 0
                ? this._availableModels
                    .map(
                      (m) =>
                        `<option value="${m.id}"${
                          m.id === currentModel ? " selected" : ""
                        }>${m.name}</option>`
                    )
                    .join("")
                : '<option value="">Loading...</option>'
            }
        </select>
        <div class="desc" style="margin-top: 8px;">${t[
          "setup.modelsAvailable"
        ].replace("{count}", String(this._availableModels.length))}</div>
    </div>

    <button class="btn btn-primary" id="saveBtn">${
      t["setup.startChat"]
    }</button>
    <button class="btn btn-secondary" id="skipBtn">${t["setup.skip"]}</button>

    <script nonce="${nonce}">
        (function() {
            const vscode = acquireVsCodeApi();
            console.log('WhytCard Setup View Loaded', new Date().toISOString());
            const validateText = "${validateText}";
            const validatingText = "${validatingText}";
            const validText = "${validText}";
            const invalidText = "${invalidText}";

            // MCP toggle via event delegation
            document.addEventListener('click', function(e) {
                const header = e.target.closest('[data-mcp-toggle]');
                if (header && !e.target.classList.contains('mcp-checkbox')) {
                    const id = header.getAttribute('data-mcp-toggle');
                    const details = document.getElementById('details-' + id);
                    const arrow = document.getElementById('arrow-' + id);
                    if (details && arrow) {
                        details.classList.toggle('open');
                        arrow.classList.toggle('open');
                    }
                }
            });

            // Track validation state
            const validationState = {
                github: ${hasGitHub}
            };

            document.querySelectorAll('.lang-btn').forEach(btn => {
                btn.addEventListener('click', () => {
                    vscode.postMessage({ type: 'setLanguage', language: btn.dataset.lang });
                });
            });

            document.querySelectorAll('.external-link').forEach(link => {
                link.addEventListener('click', (e) => {
                    e.preventDefault();
                    vscode.postMessage({ type: 'openExternal', url: link.dataset.url });
                });
            });

            // GitHub OAuth connection
            const connectGitHubBtn = document.getElementById('connectGitHub');
            if (connectGitHubBtn) {
                connectGitHubBtn.addEventListener('click', () => {
                    vscode.postMessage({ type: 'connectGitHub' });
                });
            }

            // WhytCard connection
            const connectWhytCardBtn = document.getElementById('connectWhytCard');
            if (connectWhytCardBtn) {
                connectWhytCardBtn.addEventListener('click', () => {
                    vscode.postMessage({ type: 'connectWhytCard' });
                });
            }

            document.getElementById('saveBtn').addEventListener('click', () => {
                const selectedMcps = Array.from(document.querySelectorAll('.mcp-checkbox:checked'))
                    .map(cb => cb.value);

                // Collect MCP configs
                const mcpConfigs = {};
                selectedMcps.forEach(mcpId => {
                    const inputs = document.querySelectorAll('.mcp-config-input[data-mcp="' + mcpId + '"]');
                    if (inputs.length > 0) {
                        mcpConfigs[mcpId] = {};
                        inputs.forEach(input => {
                            const key = input.dataset.key;
                            if (input.value.trim()) {
                                mcpConfigs[mcpId][key] = input.value.trim();
                            }
                        });
                    }
                });

                vscode.postMessage({
                    type: 'saveConfig',
                    config: {
                        preferredModel: document.getElementById('modelSelect').value,
                        selectedMcps: selectedMcps,
                        mcpConfigs: mcpConfigs
                    }
                });
            });

            document.getElementById('skipBtn').addEventListener('click', () => {
                vscode.postMessage({ type: 'skipSetup' });
            });

            // Handle messages from extension
            window.addEventListener('message', event => {
                const msg = event.data;
                if (msg.type === 'githubConnected') {
                    // Refresh the page to show connected state
                    location.reload();
                }
                if (msg.type === 'whytcardConnected') {
                    location.reload();
                }
            });

            // Auto-connect loop if not connected
            const isHubConnected = ${isHubConnected};
            if (!isHubConnected) {
                console.log('WhytCard Hub not connected, starting auto-connect polling...');
                const autoConnectInterval = setInterval(() => {
                    vscode.postMessage({ type: 'autoConnectWhytCard' });
                }, 3000);

                // Stop polling if we get a connected message
                window.addEventListener('message', event => {
                    if (event.data.type === 'whytcardConnected') {
                        clearInterval(autoConnectInterval);
                    }
                });
            }
        })();
    </script>
</body>
</html>`;
  }

  private _getChatHtml(webview: vscode.Webview): string {
    const nonce = this._getNonce();
    const lang = this._secretsService.getLanguage() as SupportedLanguage;
    const t = locales[lang] || locales.en;

    return `<!DOCTYPE html>
<html lang="${lang}">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <meta http-equiv="Content-Security-Policy" content="default-src 'none'; style-src ${
      webview.cspSource
    } 'unsafe-inline'; script-src 'nonce-${nonce}';">
    <title>WhytCard Chat</title>
    <style>
        * { box-sizing: border-box; margin: 0; padding: 0; }
        body {
            font-family: var(--vscode-font-family);
            color: var(--vscode-foreground);
            background: var(--vscode-sideBar-background);
            height: 100vh;
            display: flex;
            flex-direction: column;
            font-size: 13px;
        }

        .header {
            padding: 10px 16px;
            border-bottom: 1px solid var(--vscode-widget-border);
            display: flex;
            align-items: center;
            justify-content: space-between;
        }
        .header h1 { font-size: 14px; }
        .header-btns { display: flex; gap: 4px; }
        .header-btn {
            padding: 4px 8px;
            background: transparent;
            color: var(--vscode-descriptionForeground);
            border: none;
            border-radius: 4px;
            cursor: pointer;
            font-size: 11px;
        }
        .header-btn:hover { background: var(--vscode-button-secondaryBackground); }

        .warning-bar {
            padding: 8px 16px;
            background: var(--vscode-inputValidation-warningBackground);
            border-bottom: 1px solid var(--vscode-inputValidation-warningBorder);
            font-size: 11px;
            display: flex;
            justify-content: space-between;
        }
        .warning-bar a { color: var(--vscode-textLink-foreground); cursor: pointer; }
        .hidden { display: none !important; }

        .chat-container {
            flex: 1;
            overflow-y: auto;
            padding: 16px;
            display: flex;
            flex-direction: column;
            gap: 12px;
        }

        .message {
            max-width: 90%;
            padding: 10px 14px;
            border-radius: 12px;
            line-height: 1.5;
        }
        .message.user {
            align-self: flex-end;
            background: var(--vscode-button-background);
            color: var(--vscode-button-foreground);
            border-bottom-right-radius: 4px;
        }
        .message.assistant {
            align-self: flex-start;
            background: var(--vscode-editor-background);
            border: 1px solid var(--vscode-widget-border);
            border-bottom-left-radius: 4px;
        }

        /* Thinking bubble */
        .thinking-bubble {
            background: var(--vscode-textBlockQuote-background);
            border: 1px dashed var(--vscode-widget-border);
            border-radius: 8px;
            padding: 10px;
            margin-top: 8px;
            font-size: 11px;
        }
        .thinking-header {
            display: flex;
            justify-content: space-between;
            align-items: center;
            cursor: pointer;
            color: var(--vscode-descriptionForeground);
        }
        .thinking-header:hover { color: var(--vscode-foreground); }
        .thinking-content { margin-top: 8px; }
        .thinking-content.collapsed { display: none; }
        .thinking-step {
            padding: 4px 0;
            display: flex;
            align-items: center;
            gap: 8px;
        }
        .thinking-step .icon { font-size: 12px; font-family: monospace; }
        .thinking-step.searching .icon::before { content: "[Search]"; }
        .thinking-step.analyzing .icon::before { content: "[Analyze]"; }
        .thinking-step.reasoning .icon::before { content: "[Think]"; }
        .thinking-step.complete .icon::before { content: "[OK]"; }
        .thinking-step.error .icon::before { content: "[Error]"; }

        /* Live thinking indicator */
        .thinking-live {
            display: flex;
            align-items: center;
            gap: 8px;
            padding: 12px;
            background: var(--vscode-textBlockQuote-background);
            border-radius: 8px;
            animation: pulse 1.5s infinite;
        }
        @keyframes pulse {
            0%, 100% { opacity: 1; }
            50% { opacity: 0.6; }
        }
        .thinking-live .cancel-btn {
            margin-left: auto;
            padding: 4px 8px;
            background: var(--vscode-button-secondaryBackground);
            border: none;
            border-radius: 4px;
            color: var(--vscode-foreground);
            cursor: pointer;
            font-size: 10px;
        }

        .project-card {
            background: var(--vscode-editor-background);
            border: 1px solid var(--vscode-widget-border);
            border-radius: 8px;
            padding: 10px;
            font-size: 11px;
        }
        .project-card h3 { font-size: 11px; margin-bottom: 6px; color: var(--vscode-descriptionForeground); }
        .project-item { display: flex; justify-content: space-between; padding: 2px 0; }
        .project-item .value { color: var(--vscode-textLink-foreground); }

        .checklist-card {
            background: var(--vscode-editor-background);
            border: 1px solid var(--vscode-widget-border);
            border-radius: 8px;
            padding: 10px;
            font-size: 11px;
            margin-top: 8px;
        }
        .checklist-card h3 { font-size: 11px; margin-bottom: 6px; color: var(--vscode-descriptionForeground); }
        .checklist-item { display: flex; gap: 6px; padding: 2px 0; align-items: flex-start; }
        .checklist-item .check { color: var(--vscode-testing-iconPassed); font-weight: bold; }

        .file-list { margin-top: 10px; }
        .file-item {
            display: flex;
            justify-content: space-between;
            align-items: center;
            padding: 6px 8px;
            background: var(--vscode-editor-background);
            border-radius: 4px;
            margin-bottom: 4px;
            font-size: 11px;
        }
        .file-item .open-btn {
            padding: 2px 8px;
            background: transparent;
            color: var(--vscode-textLink-foreground);
            border: 1px solid currentColor;
            border-radius: 4px;
            cursor: pointer;
            font-size: 10px;
        }

        .input-area {
            padding: 12px 16px;
            border-top: 1px solid var(--vscode-widget-border);
        }
        .input-row {
            display: flex;
            gap: 8px;
            margin-bottom: 8px;
        }
        .input-row input {
            flex: 1;
            padding: 10px 14px;
            background: var(--vscode-input-background);
            color: var(--vscode-input-foreground);
            border: 1px solid var(--vscode-input-border);
            border-radius: 20px;
            font-size: 13px;
        }
        .input-row input:focus { border-color: var(--vscode-focusBorder); outline: none; }
        .input-row button {
            padding: 10px 16px;
            background: var(--vscode-button-background);
            color: var(--vscode-button-foreground);
            border: none;
            border-radius: 20px;
            cursor: pointer;
            font-weight: 500;
        }
        .input-row button:disabled { opacity: 0.5; }

        .action-btn {
            width: 100%;
            padding: 10px;
            background: var(--vscode-button-background);
            color: var(--vscode-button-foreground);
            border: none;
            border-radius: 8px;
            cursor: pointer;
            font-weight: 500;
        }
        .action-btn:hover { background: var(--vscode-button-hoverBackground); }
        .action-btn:disabled { opacity: 0.5; }
    </style>
</head>
<body>
    <div class="header">
        <h1>${t["chat.title"]}</h1>
        <div class="header-btns">
            <button class="header-btn" id="settingsBtn">${
              t["chat.settings"]
            }</button>
            <button class="header-btn" id="newChatBtn">${
              t["chat.newChat"]
            }</button>
        </div>
    </div>

    <div class="warning-bar hidden" id="warningBar">
        ${t["chat.noApiWarning"]} <a id="configureLink">${
      t["chat.configureNow"]
    }</a>
    </div>

    <div class="chat-container" id="chatContainer">
        <div class="message assistant">${t["chat.welcome"]}</div>
        <div class="project-card hidden" id="projectCard">
            <h3>${t["project.detected"]}</h3>
            <div id="projectInfo"></div>
        </div>
        <div class="checklist-card hidden" id="checklistCard">
            <h3>Project Checklist</h3>
            <div id="checklistItems"></div>
        </div>
        <div class="thinking-live hidden" id="thinkingLive">
            <span>[Thinking]</span>
            <span id="thinkingText">${t["thinking.title"]}</span>
            <button class="cancel-btn" id="cancelBtn">[X]</button>
        </div>
    </div>

    <div class="input-area">
        <div class="input-row">
            <input type="text" id="messageInput" placeholder="${
              t["chat.placeholder"]
            }" autocomplete="off">
            <button id="sendBtn">${t["chat.send"]}</button>
        </div>
    </div>

    <script nonce="${nonce}">
        (function() {
            const vscode = acquireVsCodeApi();
            const chatContainer = document.getElementById('chatContainer');
            const messageInput = document.getElementById('messageInput');
            const sendBtn = document.getElementById('sendBtn');
            const thinkingLive = document.getElementById('thinkingLive');
            const thinkingText = document.getElementById('thinkingText');
            const projectCard = document.getElementById('projectCard');
            const projectInfo = document.getElementById('projectInfo');
            const checklistCard = document.getElementById('checklistCard');
            const checklistItems = document.getElementById('checklistItems');

            const t = ${JSON.stringify(t)};

            function sendMessage() {
                const content = messageInput.value.trim();
                if (!content) return;
                addMessage(content, 'user');
                messageInput.value = '';
                vscode.postMessage({ type: 'sendMessage', content });
            }

            function addMessage(content, role, thinkingSteps) {
                const msg = document.createElement('div');
                msg.className = 'message ' + role;
                msg.textContent = content;

                // Add thinking bubble if there are steps
                if (thinkingSteps && thinkingSteps.length > 0) {
                    const bubble = document.createElement('div');
                    bubble.className = 'thinking-bubble';

                    const header = document.createElement('div');
                    header.className = 'thinking-header';
                    header.innerHTML = '<span>[Thinking] ' + t["thinking.expand"] + '</span><span>[v]</span>';

                    const contentDiv = document.createElement('div');
                    contentDiv.className = 'thinking-content collapsed';

                    thinkingSteps.forEach(step => {
                        const stepDiv = document.createElement('div');
                        stepDiv.className = 'thinking-step ' + step.type;
                        stepDiv.innerHTML = '<span class="icon"></span><span>' + step.message + '</span>';
                        contentDiv.appendChild(stepDiv);
                    });

                    header.addEventListener('click', () => {
                        contentDiv.classList.toggle('collapsed');
                        header.querySelector('span:last-child').textContent = contentDiv.classList.contains('collapsed') ? '[v]' : '[^]';
                        header.querySelector('span:first-child').innerHTML = '[Thinking] ' + (contentDiv.classList.contains('collapsed') ? t["thinking.expand"] : t["thinking.collapse"]);
                    });

                    bubble.appendChild(header);
                    bubble.appendChild(contentDiv);
                    msg.appendChild(bubble);
                }

                chatContainer.insertBefore(msg, thinkingLive);
                chatContainer.scrollTop = chatContainer.scrollHeight;
            }

            function addFileList(files) {
                const container = document.createElement('div');
                container.className = 'file-list';
                files.forEach(f => {
                    const item = document.createElement('div');
                    item.className = 'file-item';
                    item.innerHTML = '<span>' + f + '</span><button class="open-btn" data-file="' + f + '">' + t["chat.openFile"] + '</button>';
                    container.appendChild(item);
                });
                chatContainer.insertBefore(container, thinkingLive);
                chatContainer.scrollTop = chatContainer.scrollHeight;
            }

            // Events
            sendBtn.addEventListener('click', sendMessage);
            messageInput.addEventListener('keypress', e => { if (e.key === 'Enter') sendMessage(); });
            document.getElementById('settingsBtn').addEventListener('click', () => vscode.postMessage({ type: 'openSettings' }));
            document.getElementById('newChatBtn').addEventListener('click', () => vscode.postMessage({ type: 'newChat' }));
            document.getElementById('configureLink')?.addEventListener('click', () => vscode.postMessage({ type: 'openSettings' }));
            document.getElementById('cancelBtn').addEventListener('click', () => vscode.postMessage({ type: 'cancelThinking' }));

            chatContainer.addEventListener('click', e => {
                if (e.target.classList.contains('open-btn')) {
                    vscode.postMessage({ type: 'openFile', path: e.target.dataset.file });
                }
            });

            // Messages from extension
            window.addEventListener('message', event => {
                const msg = event.data;
                switch (msg.type) {
                    case 'projectAnalyzed':
                        if (msg.projectInfo) {
                            let html = '';
                            const info = msg.projectInfo;
                            if (info.framework) html += '<div class="project-item"><span>' + t["project.framework"] + '</span><span class="value">' + info.framework + '</span></div>';
                            if (info.language) html += '<div class="project-item"><span>' + t["project.language"] + '</span><span class="value">' + info.language + '</span></div>';
                            if (info.styling) html += '<div class="project-item"><span>' + t["project.styling"] + '</span><span class="value">' + info.styling + '</span></div>';
                            if (html) {
                                projectInfo.innerHTML = html;
                                projectCard.classList.remove('hidden');
                            }
                        }
                        break;
                    case 'thinking':
                        thinkingLive.classList.toggle('hidden', !msg.isThinking);
                        sendBtn.disabled = msg.isThinking;
                        chatContainer.scrollTop = chatContainer.scrollHeight;
                        break;
                    case 'thinkingStep':
                        thinkingText.textContent = msg.step.message;
                        break;
                    case 'assistantMessage':
                        addMessage(msg.content, 'assistant', msg.thinkingSteps);
                        break;
                    case 'error':
                        addMessage(t["response.error"] + ' ' + msg.message, 'assistant');
                        break;
                    case 'noApiWarning':
                        document.getElementById('warningBar').classList.remove('hidden');
                        break;
                    case 'updateChecklist':
                        if (msg.checklist && msg.checklist.length > 0) {
                            checklistItems.innerHTML = msg.checklist.map(item =>
                                '<div class="checklist-item"><span class="check">[OK]</span><span>' + item + '</span></div>'
                            ).join('');
                            checklistCard.classList.remove('hidden');
                        } else {
                            checklistCard.classList.add('hidden');
                        }
                        break;
                }
            });

            messageInput.focus();
        })();
    </script>
</body>
</html>`;
  }
}
