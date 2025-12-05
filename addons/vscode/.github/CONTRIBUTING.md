# Contributing to WhytCard VS Code Extension

Thank you for your interest in contributing!

## Getting Started

### Prerequisites

- Node.js 18+
- VS Code
- WhytCard Hub (for testing)

### Development Setup

1. Clone the repository
2. Run `npm install`
3. Open in VS Code
4. Press F5 to launch Extension Development Host

## How to Contribute

### Reporting Bugs

- Use GitHub issues
- Include VS Code version
- Include extension version
- Describe steps to reproduce

### Pull Requests

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Run `npm run lint`
5. Test in Development Host
6. Submit a pull request

## Code Guidelines

- TypeScript strict mode
- No emojis anywhere
- All strings use i18n
- Handle all errors gracefully
- Document public APIs

## Building

```bash
npm run compile   # Compile TypeScript
npm run watch     # Watch mode
npm run package   # Create VSIX
```

## License

Contributions are licensed under GPL-3.0.
