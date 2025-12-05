# WhytCard Addons

Plug-and-play extensions and modules.

## Principle

Each addon:

- Is **independent** (own build, versioning)
- **Registers** with the Hub via API
- Can be **added/removed** without modifying core

## Structure

```
addons/
├── chrome/           # Browser extension
├── vscode/           # IDE extension
├── ears/             # STT service (Whisper)
├── voice/            # TTS service (XTTS)
└── web/              # Web sites
```

## Addon Types

| Type                | Communication | Example     |
| ------------------- | ------------- | ----------- |
| `browser_extension` | HTTP + SSE    | chrome      |
| `ide_extension`     | HTTP + SSE    | vscode      |
| `service`           | HTTP API      | ears, voice |
| `static`            | None          | web         |

## Registration

Each addon registers with the Hub:

```json
POST /api/addons/register
{
  "type": "service",
  "name": "WhytCard Ears",
  "version": "1.0.0",
  "capabilities": ["transcribe_audio"],
  "endpoint": "http://localhost:3001"
}
```

## Adding a New Addon

1. Create `addons/my-addon/`
2. Implement the API or extension
3. Add `/health` endpoint (services)
4. Register with Hub on startup
5. Document capabilities

## License

All addons are licensed under GPL-3.0. See LICENSE file at repository root.
