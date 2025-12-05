# GitHub Copilot Instructions for WhytCard Chrome Extension

## Project Overview

WhytCard Chrome is a browser extension (Manifest V3) that connects to WhytCard Hub for web content capture and AI chat functionality.

## Technology Stack

- **Platform**: Chrome Extension (Manifest V3)
- **Language**: JavaScript (ES6+)
- **APIs**: Chrome Extensions API, Side Panel API, Storage API

## Code Style Guidelines

### JavaScript

- Use ES6+ features (const/let, arrow functions, async/await)
- Use meaningful variable and function names
- Add JSDoc comments for functions
- Handle errors properly with try/catch

### Extension Architecture

```
WhytCard-Chrome/
  manifest.json     # Extension configuration
  background.js     # Service worker
  content.js        # Content script
  popup.html/js     # Popup UI
  sidepanel.html/js # Side panel chat
  options.html/js   # Settings page
```

## Important Conventions

1. **No Emojis**: Never use emojis in code, comments, or UI text
2. **Error Handling**: Always handle API errors gracefully
3. **Security**: Never store sensitive data in plaintext
4. **Permissions**: Request only necessary permissions

## Chrome Extension Patterns

### Message Passing

```javascript
// Background to content
chrome.tabs.sendMessage(tabId, { action: "action_name", data: payload });

// Content to background
chrome.runtime.sendMessage({ action: "action_name", data: payload });
```

### Storage

```javascript
// Use chrome.storage.local for persistent data
chrome.storage.local.set({ key: value });
chrome.storage.local.get(['key'], (result) => { ... });
```

### API Communication

- Always use Bearer token authentication
- Handle connection errors to Hub
- Implement retry logic for failed requests

## Testing

- Test on multiple Chrome versions
- Test with Hub running and not running
- Test permission scenarios

## Commit Messages

Use conventional format:

```
feat: Add new feature
fix: Fix bug
docs: Update documentation
```
