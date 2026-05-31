# Zoid

Zoid is a desktop application built with Tauri, React, and TypeScript.

## Development

- `npm run dev` starts the Vite frontend.
- `npm run tauri:dev` starts the Tauri desktop app in development mode.
- `npm run build` builds the frontend.
- `npm run tauri:build` builds the desktop release artifacts.
- `npm run test:rust` runs the Rust test suite.

## Local verification and release gate

GitHub Actions/CI/CD is unavailable for this GitHub account: workflows fail before jobs start with `startup_failure`. The active release gate for this repository is therefore local-first verification, not GitHub Actions.

Use these scripts as the source of truth:

- `npm run verify:push` runs the faster local push gate: Rust tests and frontend build. It skips macOS packaging.
- `npm run verify:local` runs the same non-packaging local gate for normal development: Rust tests and frontend build. It intentionally does not build or mount the DMG, so it will not open the macOS drag-to-Applications DMG window.
- `npm run verify:release` runs the full local release gate: dependency check/install, Rust tests, frontend build, Tauri macOS packaging, and DMG/app inspection. Use this only when intentionally producing a new DMG.
- `npm run hooks:install` installs a local-only `.git/hooks/pre-push` hook that runs `npm run verify:push` equivalent checks before pushing from this checkout.

`npm run verify:release` produces unsigned internal macOS artifacts only. Unsigned and non-notarized DMGs are acceptable for internal smoke checks and artifact inspection, but public macOS distribution requires Developer ID signing and notarization.

See [Docs/release/macos-dmg-signing-notarization.md](Docs/release/macos-dmg-signing-notarization.md) for the macOS DMG signing, notarization, and verification checklist.
