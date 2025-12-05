/**
 * WhytCard Chrome Extension - Global Test Setup
 *
 * Starts a mock Hub server for testing extension connectivity.
 */

import { FullConfig } from "@playwright/test";
import { createServer, IncomingMessage, Server, ServerResponse } from "http";

let mockServer: Server | null = null;

async function globalSetup(config: FullConfig) {
  console.log("[Test Setup] Starting mock Hub server...");

  mockServer = createServer((req: IncomingMessage, res: ServerResponse) => {
    // CORS headers
    res.setHeader("Access-Control-Allow-Origin", "*");
    res.setHeader("Access-Control-Allow-Methods", "GET, POST, OPTIONS");
    res.setHeader("Access-Control-Allow-Headers", "Content-Type, Authorization");

    if (req.method === "OPTIONS") {
      res.writeHead(200);
      res.end();
      return;
    }

    const url = req.url || "";

    // Health check endpoint
    if (url === "/api/health" || url === "/health") {
      res.writeHead(200, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ status: "ok", version: "1.0.0-test" }));
      return;
    }

    // SSE Events endpoint
    if (url.startsWith("/api/events")) {
      res.writeHead(200, {
        "Content-Type": "text/event-stream",
        "Cache-Control": "no-cache",
        Connection: "keep-alive",
        "Access-Control-Allow-Origin": "*",
      });

      // Send connected event immediately
      res.write(`event: connected\ndata: ${JSON.stringify({ session_id: "test-session" })}\n\n`);

      // Send first heartbeat immediately for tests
      res.write(`event: heartbeat\ndata: ${JSON.stringify({ timestamp: Date.now() })}\n\n`);

      // Send heartbeat every 2 seconds (faster for tests)
      const heartbeatInterval = setInterval(() => {
        try {
          res.write(`event: heartbeat\ndata: ${JSON.stringify({ timestamp: Date.now() })}\n\n`);
        } catch {
          // Client disconnected
          clearInterval(heartbeatInterval);
        }
      }, 2000);

      req.on("close", () => {
        clearInterval(heartbeatInterval);
      });

      req.on("error", () => {
        clearInterval(heartbeatInterval);
      });

      return;
    }

    // Chat endpoint - returns format matching real API
    if (url === "/api/chat" && req.method === "POST") {
      let body = "";
      req.on("data", (chunk) => {
        body += chunk;
      });
      req.on("end", () => {
        res.writeHead(200, { "Content-Type": "application/json" });
        res.end(
          JSON.stringify({
            reply: "This is a mock response from the test Hub server.",
          })
        );
      });
      return;
    }

    // Ingest endpoint
    if (url === "/api/ingest" && req.method === "POST") {
      let body = "";
      req.on("data", (chunk) => {
        body += chunk;
      });
      req.on("end", () => {
        res.writeHead(200, { "Content-Type": "application/json" });
        res.end(
          JSON.stringify({
            success: true,
            job_id: `job-${Date.now()}`,
          })
        );
      });
      return;
    }

    // Token validation - GET method as per API spec
    if (url === "/api/tokens/validate" && (req.method === "GET" || req.method === "POST")) {
      const authHeader = req.headers.authorization;
      const isValid = authHeader && authHeader.startsWith("Bearer ");
      res.writeHead(isValid ? 200 : 401, { "Content-Type": "application/json" });
      res.end(JSON.stringify({ valid: isValid ? true : false }));
      return;
    }

    // Default 404
    res.writeHead(404, { "Content-Type": "application/json" });
    res.end(JSON.stringify({ error: "Not found" }));
  });

  await new Promise<void>((resolve) => {
    mockServer!.listen(3000, "127.0.0.1", () => {
      console.log("[Test Setup] Mock Hub server running on http://localhost:3000");
      resolve();
    });
  });

  // Store server reference for teardown
  (globalThis as Record<string, unknown>).__MOCK_SERVER__ = mockServer;
}

export default globalSetup;
