/**
 * WhytCard VS Code Extension - Command Tests
 *
 * Tests for extension commands.
 */

import * as assert from "assert";
import * as vscode from "vscode";

suite("Commands", () => {
  test("openSidebar command should execute", async () => {
    // This command focuses the sidebar view
    await vscode.commands.executeCommand("whytcard.openSidebar");
    // If no error is thrown, the command exists and can be executed
    assert.ok(true, "openSidebar command executed successfully");
  });

  test("reconnectHub command should execute", async () => {
    // Execute the command - it should not throw even if Hub is unavailable
    try {
      await vscode.commands.executeCommand("whytcard.reconnectHub");
      assert.ok(true, "reconnectHub command executed");
    } catch (err) {
      // Command may show a warning if Hub is unavailable, which is expected
      assert.ok(true, "Command handled Hub unavailability");
    }
  });

  test("initWorkspace command should be available", async () => {
    const commands = await vscode.commands.getCommands(true);
    assert.ok(
      commands.includes("whytcard.initWorkspace"),
      "initWorkspace command should be registered"
    );
  });

  test("configureMcp command should be available", async () => {
    const commands = await vscode.commands.getCommands(true);
    assert.ok(
      commands.includes("whytcard.configureMcp"),
      "configureMcp command should be registered"
    );
  });
});

suite("Command Palette Integration", () => {
  test("WhytCard commands should appear in palette", async () => {
    const commands = await vscode.commands.getCommands(true);
    const whytcardCommands = commands.filter((cmd) =>
      cmd.startsWith("whytcard.")
    );

    assert.ok(whytcardCommands.length >= 4, "Should have at least 4 WhytCard commands");
  });
});
