# Security Policy

## Reporting Vulnerabilities

Report security issues privately via GitHub security advisories.

Do NOT open public issues for security vulnerabilities.

## Security Measures

- API keys stored in chrome.storage.local
- No sensitive data in console logs
- HTTPS-only communication (localhost exception for Hub)

## Permissions

This extension requests only necessary permissions:

- `storage`: Save settings
- `sidePanel`: Chat interface
- `contextMenus`: Right-click actions
- `activeTab`: Access current page content
