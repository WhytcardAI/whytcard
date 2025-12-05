/**
 * WhytCard Chrome Extension - Global Test Teardown
 *
 * Stops the mock Hub server after tests complete.
 */

import { Server } from "http";
import { FullConfig } from "@playwright/test";

async function globalTeardown(config: FullConfig) {
  console.log("[Test Teardown] Stopping mock Hub server...");

  const mockServer = (globalThis as Record<string, unknown>).__MOCK_SERVER__ as Server | undefined;

  if (mockServer) {
    await new Promise<void>((resolve, reject) => {
      mockServer.close((err) => {
        if (err) {
          console.error("[Test Teardown] Error closing server:", err);
          reject(err);
        } else {
          console.log("[Test Teardown] Mock Hub server stopped");
          resolve();
        }
      });
    });
  }
}

export default globalTeardown;
