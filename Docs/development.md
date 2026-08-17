# Zoid development

Engineer runbook for the Zoid 25 desktop app. The landing page is [README.md](../README.md).

## Stack

- Desktop shell: Tauri 2
- Frontend: React + TypeScript + Vite
- Native layer: Rust Tauri commands
- Local persistence: files and app state through the desktop bridge
- External operations: GitHub CLI, Hermes CLI, and provider APIs with explicit read-back

## Layout

```text
src/                 React workspaces (agents, code, content, social, automations, brain, vps)
src-tauri/           Tauri/Rust shell and command bridge
Docs/                Product, design, and module documentation
public/              Static assets
scripts/             Local verification and helper scripts
PRODUCT.md           Product intent
DESIGN.md            Visual language
```

## Commands

```bash
npm install
npm run dev              # Vite preview on 127.0.0.1:1420
npm run tauri:dev        # native desktop app
npm run build            # frontend production build
npm run tauri:build      # desktop release artifacts
npm run test:frontend
npm run test:rust
npm test                 # frontend + Rust
```

## Local verification

GitHub Actions is not the release gate for this repo. Use the local scripts:

```bash
scripts/verify-local.sh --skip-package
scripts/verify-local.sh
scripts/install-git-hooks.sh
```

Full package verification needs macOS tools (hdiutil, ditto, PlistBuddy) and produces unsigned internal artifacts. Public macOS distribution still needs Developer ID signing and notarization.

## Operating rules

1. No fake success states. Distinguish requested, queued, verified, scheduled, failed, and unavailable.
2. No raw implementation leakage in product copy (provider IDs, tokens, internal paths) unless debugging.
3. Local artifacts stay local. Planning notes and proof screenshots belong in ignored paths unless they are product docs or app assets.
4. Visual changes need a Visual proof section on the PR. Do not commit proof files unless they are app assets.
5. Never commit secrets, tokens, credentials, local databases, or environment files.

## Pull request standard

Each PR should include What changed, Why it changed, Proof/tests, Risks/notes, and Visual proof.

## Documentation

Durable product, release, and module docs belong in Docs/. Local-only notes, decisions, and generated reports stay under ignored paths such as .local-workdocs/ or .hermes/.
