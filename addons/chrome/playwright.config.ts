import { defineConfig, devices } from "@playwright/test";

/**
 * WhytCard Chrome Extension - Playwright Configuration
 *
 * This config sets up Playwright to test Chrome extensions.
 * Extensions are loaded via persistent browser context.
 */
export default defineConfig({
  testDir: "./tests/specs",
  fullyParallel: false, // Extensions require sequential testing
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1, // Single worker for extension tests
  reporter: [["list"], ["html", { outputFolder: "playwright-report" }]],
  outputDir: "test-results",

  use: {
    trace: "on-first-retry",
    screenshot: "only-on-failure",
    video: "on-first-retry",
  },

  projects: [
    {
      name: "chromium-extension",
      use: {
        ...devices["Desktop Chrome"],
        // Extension path will be set in fixtures
      },
    },
  ],

  // Global setup/teardown
  globalSetup: "./tests/global-setup.ts",
  globalTeardown: "./tests/global-teardown.ts",
});
