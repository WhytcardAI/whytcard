/**
 * WhytCard Chrome Extension - Playwright Test Fixtures
 *
 * Provides fixtures for testing Chrome extensions with Playwright.
 * Loads the extension into a persistent browser context.
 */

import { test as base, chromium, BrowserContext, Page } from "@playwright/test";
import path from "path";

// Re-export expect
export { expect } from "@playwright/test";

/**
 * Custom test fixtures for Chrome extension testing
 */
export const test = base.extend<{
  extensionContext: BrowserContext;
  extensionId: string;
  backgroundPage: Page;
  popupPage: Page;
  sidepanelPage: Page;
  optionsPage: Page;
}>({
  // Persistent browser context with extension loaded
  extensionContext: async ({}, use) => {
    const extensionPath = path.resolve(__dirname, "..", "..");

    const context = await chromium.launchPersistentContext("", {
      headless: false, // Extensions require headed mode
      args: [
        `--disable-extensions-except=${extensionPath}`,
        `--load-extension=${extensionPath}`,
        "--no-sandbox",
        "--disable-setuid-sandbox",
      ],
    });

    await use(context);
    await context.close();
  },

  // Get extension ID from service worker
  extensionId: async ({ extensionContext }, use) => {
    // Wait for service worker to register
    let extensionId = "";

    // Get extension ID from background service worker URL
    const serviceWorkers = extensionContext.serviceWorkers();

    if (serviceWorkers.length > 0) {
      const swUrl = serviceWorkers[0].url();
      const match = swUrl.match(/chrome-extension:\/\/([^/]+)/);
      if (match) {
        extensionId = match[1];
      }
    } else {
      // Wait for service worker to appear
      const sw = await extensionContext.waitForEvent("serviceworker", {
        timeout: 10000,
      });
      const swUrl = sw.url();
      const match = swUrl.match(/chrome-extension:\/\/([^/]+)/);
      if (match) {
        extensionId = match[1];
      }
    }

    await use(extensionId);
  },

  // Background service worker page
  backgroundPage: async ({ extensionContext }, use) => {
    let bgPage: Page | null = null;

    // Try to get existing service worker
    const workers = extensionContext.serviceWorkers();
    if (workers.length > 0) {
      // Service workers don't have a page, but we can access them
      // For testing, we'll use the context's first page or create one
    }

    // Wait for service worker
    const sw = await extensionContext
      .waitForEvent("serviceworker", {
        timeout: 10000,
      })
      .catch(() => null);

    // Create a test page to interact with the extension
    bgPage = await extensionContext.newPage();
    await use(bgPage);
    await bgPage.close();
  },

  // Popup page
  popupPage: async ({ extensionContext, extensionId }, use) => {
    const popupUrl = `chrome-extension://${extensionId}/popup.html`;
    const page = await extensionContext.newPage();
    await page.goto(popupUrl);
    await use(page);
    await page.close();
  },

  // Sidepanel page
  sidepanelPage: async ({ extensionContext, extensionId }, use) => {
    const sidepanelUrl = `chrome-extension://${extensionId}/sidepanel.html`;
    const page = await extensionContext.newPage();
    await page.goto(sidepanelUrl);
    await use(page);
    await page.close();
  },

  // Options page
  optionsPage: async ({ extensionContext, extensionId }, use) => {
    const optionsUrl = `chrome-extension://${extensionId}/options.html`;
    const page = await extensionContext.newPage();
    await page.goto(optionsUrl);
    await use(page);
    await page.close();
  },
});

/**
 * Wait for mock Hub server to be ready
 */
export async function waitForHubServer(timeout = 5000): Promise<boolean> {
  const start = Date.now();

  while (Date.now() - start < timeout) {
    try {
      const response = await fetch("http://localhost:3000/api/health");
      if (response.ok) {
        return true;
      }
    } catch {
      // Server not ready yet
    }
    await new Promise((r) => setTimeout(r, 100));
  }

  return false;
}
