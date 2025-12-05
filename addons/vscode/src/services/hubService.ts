import * as vscode from "vscode";

/**
 * Hub Service for WhytCard VS Code Extension
 * Handles SSE connection and real-time communication with WhytCard Hub
 */

// ==================== TYPES ====================

export interface HubEvent {
  type: string;
  data: unknown;
  timestamp: number;
}

export interface ChatResponseEvent {
  session_id: string;
  content: string;
  is_final: boolean;
}

export interface SessionSyncEvent {
  session_id: string;
  messages: Array<{
    role: string;
    content: string;
    timestamp: number;
  }>;
}

export interface ConnectionStatusEvent {
  connected: boolean;
  clients_count: number;
}

export type HubEventType =
  | "connected"
  | "disconnected"
  | "chat_response"
  | "session_sync"
  | "status"
  | "error"
  | "heartbeat";

// ==================== CONSTANTS ====================

const HUB_BASE_URL = "http://localhost:3000";
const SSE_ENDPOINT = "/api/events";
const RECONNECT_DELAY_MS = 3000;
const MAX_RECONNECT_ATTEMPTS = 10;
const HEARTBEAT_TIMEOUT_MS = 45000;

// ==================== HUB SERVICE CLASS ====================

export class HubService {
  private static instance: HubService;

  private abortController: AbortController | null = null;
  private reconnectTimer: ReturnType<typeof setTimeout> | null = null;
  private heartbeatTimer: ReturnType<typeof setTimeout> | null = null;
  private reconnectAttempts = 0;
  private isConnected = false;
  private currentSessionId: string | null = null;

  // Event emitters
  private eventListeners: Map<HubEventType, Set<(data: unknown) => void>> =
    new Map();

  private constructor() {
    // Private constructor for singleton
  }

  /**
   * Get singleton instance
   */
  static getInstance(): HubService {
    if (!HubService.instance) {
      HubService.instance = new HubService();
    }
    return HubService.instance;
  }

  // ==================== PUBLIC API ====================

  /**
   * Connect to Hub SSE endpoint
   */
  async connect(token?: string): Promise<void> {
    if (this.isConnected && this.abortController) {
      console.log("[HubService] Already connected");
      return;
    }

    this.abortController = new AbortController();
    const headers: Record<string, string> = {
      Accept: "text/event-stream",
      "Cache-Control": "no-cache",
    };

    if (token) {
      headers["Authorization"] = `Bearer ${token}`;
    }

    try {
      console.log("[HubService] Connecting to SSE...");

      const response = await fetch(`${HUB_BASE_URL}${SSE_ENDPOINT}`, {
        method: "GET",
        headers,
        signal: this.abortController.signal,
      });

      if (!response.ok) {
        throw new Error(`HTTP error: ${response.status}`);
      }

      if (!response.body) {
        throw new Error("No response body");
      }

      this.isConnected = true;
      this.reconnectAttempts = 0;
      this.emit("connected", { timestamp: Date.now() });
      this.startHeartbeatMonitor();

      // Process SSE stream
      await this.processSSEStream(response.body);
    } catch (error) {
      if ((error as Error).name === "AbortError") {
        console.log("[HubService] Connection aborted");
        return;
      }

      console.error("[HubService] Connection error:", error);
      this.handleDisconnect();
    }
  }

  /**
   * Disconnect from Hub
   */
  disconnect(): void {
    console.log("[HubService] Disconnecting...");

    if (this.abortController) {
      this.abortController.abort();
      this.abortController = null;
    }

    if (this.reconnectTimer) {
      clearTimeout(this.reconnectTimer);
      this.reconnectTimer = null;
    }

    if (this.heartbeatTimer) {
      clearTimeout(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }

    this.isConnected = false;
    this.reconnectAttempts = 0;
    this.emit("disconnected", { timestamp: Date.now() });
  }

  /**
   * Check if connected to Hub
   */
  getIsConnected(): boolean {
    return this.isConnected;
  }

  /**
   * Set current session ID for filtering events
   */
  setSessionId(sessionId: string | null): void {
    this.currentSessionId = sessionId;
  }

  /**
   * Get current session ID
   */
  getSessionId(): string | null {
    return this.currentSessionId;
  }

  /**
   * Subscribe to events
   */
  on(eventType: HubEventType, callback: (data: unknown) => void): void {
    if (!this.eventListeners.has(eventType)) {
      this.eventListeners.set(eventType, new Set());
    }
    this.eventListeners.get(eventType)!.add(callback);
  }

  /**
   * Unsubscribe from events
   */
  off(eventType: HubEventType, callback: (data: unknown) => void): void {
    const listeners = this.eventListeners.get(eventType);
    if (listeners) {
      listeners.delete(callback);
    }
  }

  /**
   * Send a chat message to Hub
   */
  async sendChatMessage(
    message: string,
    sessionId?: string
  ): Promise<{ reply: string; session_id: string }> {
    const response = await fetch(`${HUB_BASE_URL}/api/chat`, {
      method: "POST",
      headers: {
        "Content-Type": "application/json",
      },
      body: JSON.stringify({
        message,
        session_id: sessionId || this.currentSessionId || "vscode-session",
      }),
    });

    if (!response.ok) {
      throw new Error(`Chat API error: ${response.status}`);
    }

    const data = (await response.json()) as {
      reply: string;
      session_id: string;
    };
    return data;
  }

  /**
   * Validate API token with Hub
   */
  async validateToken(token: string): Promise<boolean> {
    try {
      const response = await fetch(`${HUB_BASE_URL}/api/tokens/validate`, {
        method: "POST",
        headers: {
          "Content-Type": "application/json",
        },
        body: JSON.stringify({ token }),
      });

      return response.ok;
    } catch (error) {
      console.error("[HubService] Token validation error:", error);
      return false;
    }
  }

  /**
   * Check Hub availability
   */
  async ping(): Promise<boolean> {
    try {
      const response = await fetch(`${HUB_BASE_URL}/api/ping?source=vscode`);
      return response.ok;
    } catch (error) {
      return false;
    }
  }

  /**
   * Get SSE connection status
   */
  async getSSEStatus(): Promise<{ connected_clients: number } | null> {
    try {
      const response = await fetch(`${HUB_BASE_URL}/api/events/status`);
      if (response.ok) {
        return (await response.json()) as { connected_clients: number };
      }
      return null;
    } catch (error) {
      return null;
    }
  }

  // ==================== PRIVATE METHODS ====================

  /**
   * Process SSE stream
   */
  private async processSSEStream(
    body: ReadableStream<Uint8Array>
  ): Promise<void> {
    const reader = body.getReader();
    const decoder = new TextDecoder();
    let buffer = "";
    let isReading = true;

    try {
      while (isReading) {
        const { done, value } = await reader.read();

        if (done) {
          isReading = false;
        }

        if (done) {
          console.log("[HubService] Stream ended");
          break;
        }

        buffer += decoder.decode(value, { stream: true });
        const lines = buffer.split("\n");
        buffer = lines.pop() || "";

        let currentEvent = "";
        let currentData = "";

        for (const line of lines) {
          if (line.startsWith("event:")) {
            currentEvent = line.slice(6).trim();
          } else if (line.startsWith("data:")) {
            currentData = line.slice(5).trim();
          } else if (line === "" && currentData) {
            // End of event
            this.handleSSEEvent(currentEvent || "message", currentData);
            currentEvent = "";
            currentData = "";
          }
        }
      }
    } catch (error) {
      if ((error as Error).name !== "AbortError") {
        console.error("[HubService] Stream error:", error);
      }
    } finally {
      reader.releaseLock();
      this.handleDisconnect();
    }
  }

  /**
   * Handle incoming SSE event
   */
  private handleSSEEvent(eventType: string, data: string): void {
    this.resetHeartbeatMonitor();

    try {
      const parsedData = JSON.parse(data);

      switch (eventType) {
        case "heartbeat":
        case "ping":
          // Heartbeat received, connection is alive
          this.emit("heartbeat", parsedData);
          break;

        case "chat_response": {
          const chatData = parsedData as ChatResponseEvent;
          // Only emit if it is for our session or we have no session filter
          if (
            !this.currentSessionId ||
            chatData.session_id === this.currentSessionId
          ) {
            this.emit("chat_response", chatData);
          }
          break;
        }

        case "session_sync": {
          const syncData = parsedData as SessionSyncEvent;
          if (
            !this.currentSessionId ||
            syncData.session_id === this.currentSessionId
          ) {
            this.emit("session_sync", syncData);
          }
          break;
        }

        case "status":
          this.emit("status", parsedData as ConnectionStatusEvent);
          break;

        case "error":
          this.emit("error", parsedData);
          break;

        default:
          console.log(
            `[HubService] Unknown event type: ${eventType}`,
            parsedData
          );
      }
    } catch (error) {
      console.error("[HubService] Failed to parse event data:", error, data);
    }
  }

  /**
   * Handle disconnection
   */
  private handleDisconnect(): void {
    if (!this.isConnected) return;

    this.isConnected = false;
    this.stopHeartbeatMonitor();
    this.emit("disconnected", { timestamp: Date.now() });

    // Schedule reconnection
    if (this.reconnectAttempts < MAX_RECONNECT_ATTEMPTS) {
      const delay = RECONNECT_DELAY_MS * Math.pow(1.5, this.reconnectAttempts);
      console.log(
        `[HubService] Reconnecting in ${Math.round(delay / 1000)}s (attempt ${
          this.reconnectAttempts + 1
        }/${MAX_RECONNECT_ATTEMPTS})`
      );

      this.reconnectTimer = setTimeout(() => {
        this.reconnectAttempts++;
        this.connect();
      }, delay);
    } else {
      console.error("[HubService] Max reconnection attempts reached");
      vscode.window.showWarningMessage(
        "WhytCard Hub connection lost. Please check if the Hub is running."
      );
    }
  }

  /**
   * Start heartbeat monitor
   */
  private startHeartbeatMonitor(): void {
    this.stopHeartbeatMonitor();
    this.heartbeatTimer = setTimeout(() => {
      console.warn("[HubService] Heartbeat timeout, reconnecting...");
      this.handleDisconnect();
    }, HEARTBEAT_TIMEOUT_MS);
  }

  /**
   * Reset heartbeat monitor
   */
  private resetHeartbeatMonitor(): void {
    this.startHeartbeatMonitor();
  }

  /**
   * Stop heartbeat monitor
   */
  private stopHeartbeatMonitor(): void {
    if (this.heartbeatTimer) {
      clearTimeout(this.heartbeatTimer);
      this.heartbeatTimer = null;
    }
  }

  /**
   * Emit event to listeners
   */
  private emit(eventType: HubEventType, data: unknown): void {
    const listeners = this.eventListeners.get(eventType);
    if (listeners) {
      listeners.forEach((callback) => {
        try {
          callback(data);
        } catch (error) {
          console.error(
            `[HubService] Event listener error (${eventType}):`,
            error
          );
        }
      });
    }
  }

  // ==================== DISPOSE ====================

  /**
   * Dispose service
   */
  dispose(): void {
    this.disconnect();
    this.eventListeners.clear();
  }
}
