import * as vscode from "vscode";
import { SidebarProvider } from "./sidebar/SidebarProvider";
import { i18n, detectLanguage } from "./i18n";
import { SecretsService, McpService, ApiService, HubService } from "./services";

let secretsService: SecretsService;
let mcpService: McpService;
let hubService: HubService;

export async function activate(context: vscode.ExtensionContext) {
  console.log("WhytCard is now active!");

  // Initialize services
  secretsService = new SecretsService(context);
  mcpService = new McpService(context);
  hubService = HubService.getInstance();

  // Register HubService for disposal
  context.subscriptions.push({
    dispose: () => hubService.dispose(),
  });

  // Notify Hub of connection and start SSE
  const apiService = new ApiService(secretsService);
  const isHubAvailable = await apiService.pingHub();

  if (isHubAvailable) {
    console.log("[Extension] Hub is available, connecting SSE...");
    hubService.connect().catch((err) => {
      console.warn("[Extension] SSE connection failed:", err);
    });
  }

  // Migrate old secrets to secure storage (one-time)
  await secretsService.migrateFromGlobalState();

  // Initialize i18n
  const language = detectLanguage();
  i18n.setLanguage(language);

  // Register Sidebar Webview Provider
  const sidebarProvider = new SidebarProvider(
    context,
    secretsService,
    mcpService,
    hubService
  );
  context.subscriptions.push(
    vscode.window.registerWebviewViewProvider(
      "whytcard.sidebar",
      sidebarProvider,
      {
        webviewOptions: {
          retainContextWhenHidden: true,
        },
      }
    )
  );

  // ==================== COMMANDS ====================

  // Open Sidebar
  context.subscriptions.push(
    vscode.commands.registerCommand("whytcard.openSidebar", () => {
      vscode.commands.executeCommand("whytcard.sidebar.focus");
    })
  );

  // Initialize Workspace
  context.subscriptions.push(
    vscode.commands.registerCommand("whytcard.initWorkspace", async () => {
      await vscode.commands.executeCommand("whytcard.sidebar.focus");
      sidebarProvider.startInitialization();
    })
  );

  // Configure MCP
  context.subscriptions.push(
    vscode.commands.registerCommand("whytcard.configureMcp", async () => {
      vscode.commands.executeCommand("whytcard.sidebar.focus");
    })
  );

  // Reconnect to Hub
  context.subscriptions.push(
    vscode.commands.registerCommand("whytcard.reconnectHub", async () => {
      const isAvailable = await hubService.ping();
      if (isAvailable) {
        hubService.disconnect();
        await hubService.connect();
        vscode.window.showInformationMessage("Reconnected to WhytCard Hub!");
      } else {
        vscode.window.showWarningMessage(
          "WhytCard Hub is not available. Make sure the desktop app is running."
        );
      }
    })
  );

  // ==================== WELCOME MESSAGE ====================

  const hasShownWelcome = context.globalState.get("whytcard.hasShownWelcome");
  if (!hasShownWelcome) {
    showWelcomeMessage(context);
    context.globalState.update("whytcard.hasShownWelcome", true);
  }
}

async function showWelcomeMessage(_context: vscode.ExtensionContext) {
  const message = i18n.t("welcome.message");
  const openSidebar = i18n.t("welcome.openSidebar");
  const later = i18n.t("welcome.later");

  const result = await vscode.window.showInformationMessage(
    message,
    openSidebar,
    later
  );

  if (result === openSidebar) {
    vscode.commands.executeCommand("whytcard.sidebar.focus");
  }
}

export function deactivate() {
  console.log("WhytCard is now deactivated");
}
