# GitHub Copilot Instructions for WhytCard VS Code Extension

## Project Overview

WhytCard VS Code is a Visual Studio Code extension that integrates with WhytCard Hub for AI-assisted coding and project analysis.

## Technology Stack

- **Platform**: VS Code Extension
- **Language**: TypeScript
- **Build**: Webpack
- **APIs**: VS Code Extension API

## Code Style Guidelines

### TypeScript

- Use strict TypeScript settings
- Define interfaces for data structures
- Use async/await for asynchronous code
- Handle errors with try/catch

### Extension Architecture

```
WhytCard-Vscode/
  src/
    extension.ts       # Entry point
    core/              # Core functionality
    services/          # API services
    sidebar/           # Sidebar provider
    i18n/              # Internationalization
```

## Important Conventions

1. **No Emojis**: Never use emojis in code, comments, or UI
2. **i18n**: All user-facing strings must be translatable
3. **Error Handling**: Display user-friendly error messages
4. **Security**: Use VS Code SecretStorage for sensitive data

## VS Code Extension Patterns

### Command Registration

```typescript
const disposable = vscode.commands.registerCommand(
  'whytcard.commandName',
  async () => { ... }
);
context.subscriptions.push(disposable);
```

### Webview Communication

```typescript
// Send to webview
panel.webview.postMessage({ type: 'action', data: payload });

// Receive from webview
panel.webview.onDidReceiveMessage((message) => { ... });
```

### Secret Storage

```typescript
// Store secrets securely
await context.secrets.store('apiKey', key);
const key = await context.secrets.get('apiKey');
```

## Testing

- Run `npm run test` for unit tests
- Test in VS Code Extension Development Host
- Test with and without Hub connection

## Commit Messages

```
feat: New feature
fix: Bug fix
docs: Documentation update
```
