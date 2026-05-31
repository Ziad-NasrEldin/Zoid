# Zoid

Zoid is a desktop application built with Tauri, React, and TypeScript.

## Development

- `npm run dev` starts the Vite frontend.
- `npm run tauri:dev` starts the Tauri desktop app in development mode.
- `npm run build` builds the frontend.
- `npm run tauri:build` builds the desktop release artifacts.
- `npm run test:rust` runs the Rust test suite.

## Release / CI

CI runs frontend and Rust verification, then uploads macOS packaging artifacts for internal review. Current macOS CI and local packaging flows produce unsigned DMGs that are suitable only for internal smoke checks and artifact inspection; public macOS distribution still requires Developer ID signing and notarization.

See [Docs/release/macos-dmg-signing-notarization.md](Docs/release/macos-dmg-signing-notarization.md) for the macOS DMG signing, notarization, and verification checklist.
