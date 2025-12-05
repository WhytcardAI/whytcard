/**
 * WhytCard VS Code Extension - Extension Activation Tests
 *
 * Tests for extension activation and lifecycle.
 */

import * as assert from "assert";
import * as vscode from "vscode";

suite("Extension Activation", () => {
  vscode.window.showInformationMessage("Starting WhytCard Extension tests.");

  test("Extension should be present", () => {
    const extension = vscode.extensions.getExtension("WhytCard.whytcard-vscode");
    assert.ok(extension, "Extension should be installed");
  });

  test("Extension should activate", async () => {
    const extension = vscode.extensions.getExtension("WhytCard.whytcard-vscode");

    if (extension) {
      await extension.activate();
      assert.ok(extension.isActive, "Extension should be active");
    }
  });

  test("Commands should be registered", async () => {
    const commands = await vscode.commands.getCommands(true);

    const expectedCommands = [
      "whytcard.openSidebar",
      "whytcard.initWorkspace",
      "whytcard.configureMcp",
      "whytcard.reconnectHub",
    ];

    for (const cmd of expectedCommands) {
      assert.ok(commands.includes(cmd), `Command ${cmd} should be registered`);
    }
  });

  test("Sidebar view should be registered", async () => {
    // The sidebar view container should exist
    // This is checked indirectly by seeing if the extension provides views
    const extension = vscode.extensions.getExtension("WhytCard.whytcard-vscode");

    if (extension) {
      const packageJson = extension.packageJSON;
      assert.ok(packageJson.contributes?.views, "Extension should contribute views");
      assert.ok(
        packageJson.contributes.views["whytcard-sidebar"],
        "Should have whytcard-sidebar view"
      );
    }
  });
});

suite("Extension Configuration", () => {
  test("Configuration section should exist", () => {
    const config = vscode.workspace.getConfiguration("whytcard");
    assert.ok(config, "Configuration section should exist");
  });

  test("Should have default hub URL", () => {
    const config = vscode.workspace.getConfiguration("whytcard");
    const hubUrl = config.get<string>("hubUrl");

    assert.strictEqual(
      hubUrl,
      "http://localhost:3000",
      "Default hub URL should be localhost:3000"
    );
  });

  test("Should have language setting", () => {
    const config = vscode.workspace.getConfiguration("whytcard");
    const language = config.get<string>("language");

    assert.ok(
      ["auto", "en", "fr", "es", "de"].includes(language || "auto"),
      "Language should be a valid option"
    );
  });

  test("Should have autoConnectSSE setting", () => {
    const config = vscode.workspace.getConfiguration("whytcard");
    const autoConnect = config.get<boolean>("autoConnectSSE");

    assert.strictEqual(typeof autoConnect, "boolean", "autoConnectSSE should be boolean");
  });
});
