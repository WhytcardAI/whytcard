# WhytCard Chrome Extension

Browser companion for WhytCard Hub - Your local, private AI assistant.

## Features

### Core Features

- **Hub Connection**: Automatically connects to WhytCard Hub running locally
- **Real-time Sync**: SSE-based synchronization with Hub and other clients
- **Side Panel Chat**: Full chat interface in the browser side panel
- **Floating Action Button**: Quick access to WhytCard on any page
- **Page Capture**: Save web page content to your local knowledge base
- **Token Authentication**: Secure connection with Bearer token

### Context Menu Actions

Right-click on any page or selected text to:

- **Ask about selection**: Query your AI about selected text
- **Explain this**: Get explanations for complex content
- **Translate this**: Translate selected text
- **Save page**: Save the entire page to WhytCard
- **Summarize**: Get a quick summary of the page

### Keyboard Shortcuts

| Shortcut       | Action               |
| -------------- | -------------------- |
| `Ctrl+Shift+W` | Open Side Panel      |
| `Ctrl+Shift+C` | Capture current page |

## Installation

### Development Mode

1. Open Chrome and navigate to `chrome://extensions/`
2. Enable "Developer mode" (toggle in top right)
3. Click "Load unpacked"
4. Select the `WhytCard-Chrome` folder

### Configuration

1. After installation, right-click the extension icon then "Options"
2. Generate an API token from WhytCard Desktop app (Settings or via command)
3. Paste the token in the extension options
4. Click "Save"

### Requirements

- WhytCard Hub must be running on `localhost:3000`
- Chrome browser (Manifest V3 compatible)
- API token generated from WhytCard Desktop app

## Architecture

```
WhytCard-Chrome/
├── manifest.json      # Extension configuration
├── background.js      # Service worker (event handling)
├── content.js         # Page content script
├── content.css        # Content script styles
├── popup.html/js      # Popup UI
├── sidepanel.html/js  # Side panel chat UI
├── options.html/js    # Settings page (API key)
└── icons/             # Extension icons
```

### Communication Flow

```
[Web Page] <---> [Content Script] <---> [Service Worker] <---> [WhytCard Hub]
                       ↑                       ↑
                  [Popup UI]            [Side Panel]
                                             ↑
                                      [Options Page]
```

## API Endpoints Used

| Endpoint               | Method | Description                       |
| ---------------------- | ------ | --------------------------------- |
| `/api/ping`            | GET    | Health check with source tracking |
| `/api/chat`            | POST   | Send chat messages                |
| `/api/ingest`          | POST   | Save page content                 |
| `/api/events`          | GET    | SSE stream for real-time sync     |
| `/api/tokens/validate` | GET    | Validate stored token             |

## Real-time Sync (Coming Soon)

The extension will connect to `/api/events` SSE endpoint to:

- Receive chat messages from other clients (Desktop, VS Code)
- Display the same conversation across all devices
- Get notified of new content ingestion
- Sync action results from other extensions

## Development

### File Descriptions

- **manifest.json**: Extension manifest (Manifest V3)
- **background.js**: Handles context menus, alarms, Hub communication, and auth
- **content.js**: Injects floating button, extracts page content
- **content.css**: Styles for injected UI elements
- **popup.html/js**: Quick access popup with actions
- **sidepanel.html/js**: Full chat interface in side panel
- **options.html/js**: API key configuration page

### Adding New Features

1. Add permissions to `manifest.json` if needed
2. Implement in appropriate script (background/content/popup/sidepanel)
3. Add message handlers for cross-script communication

## Troubleshooting

### Extension not connecting to Hub

1. Ensure WhytCard Hub is running
2. Check that Hub is on port 1420 (primary) or 3000 (fallback)
3. Look for connection status indicator in popup
4. Verify API key is configured in Options

### "Unauthorized" errors

1. Open extension Options (right-click icon → Options)
2. Generate a new API key from WhytCard Desktop
3. Save the key and try again

### Content script not loading

1. Refresh the page
2. Check console for errors
3. Some pages (chrome://, edge://) block extensions

### Side panel not opening

1. Make sure you're on a regular web page
2. Try the keyboard shortcut `Ctrl+Shift+W`
3. Check browser version supports Side Panel API

## License

GPL-3.0 License - See LICENSE file for details.
