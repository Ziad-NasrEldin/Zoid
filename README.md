# Zoid 25

Zoid 25 is MaVoid’s local-first operating desk for AI-assisted work: a native desktop surface for founder-led execution, repository control, Hermes agent operations, content/social publishing, and automation oversight.

It is designed for complex operations where the product must stay readable, auditable, and fail-closed. The application favors local runtime truth, visible provider read-back, and clear operator control over hidden automations or generic AI-chat abstractions.

## Business fundamentals

### Mission

Give one operator a command room for turning strategy into shipped work: capture context, operate agents, move code, run content rhythms, and supervise automations from one coherent desktop system.

### Product promise

- **Local-first control:** Zoid runs as a desktop app and treats local runtime state as the source of operational truth.
- **Fail-closed execution:** actions that depend on external providers, files, repositories, or automations must show read-back before the UI claims success.
- **Operator clarity:** every workspace should expose status, blockers, and next actions without leaking implementation noise.
- **Provider/tool agnostic UX:** the product should present capabilities and outcomes, not lock the user’s mental model to one backend provider.
- **Sumi-e operating language:** the interface uses ink, paper, red-seal, and Japanese-inspired spatial rhythm to feel tactile, calm, and deliberate.

### Primary users

- Founder/operator running multiple business workstreams.
- Product and engineering lead coordinating AI agents, repositories, and delivery gates.
- Content operator managing daily enterprise-facing publishing rhythms.
- Automation owner supervising scheduled jobs, local watchers, and provider state.

## Product pillars

### 1. Hermes agent command room

The Agents workspace is the operator’s Hermes cockpit. It manages chat sessions, local CLI reachability, repository connection context, file attachments, queued work, and session-level focus without presenting Hermes internals as product copy.

### 2. Code and repository operations

The Code workspace connects local repositories to the agent workflow. It supports repository discovery, GitHub cloning, branch context, dirty-state awareness, and controlled handoff into implementation sessions.

### 3. Content and social operations

The Content workspace coordinates MaVoid social operations. It is built around daily publishing rhythm, media validation, provider API checks, proof/read-back, and schedule gates. The UI should never claim a post is live, scheduled, or verified without runtime/provider evidence.

### 4. Automation supervision

The Automations workspace presents local Hermes cron/watchers and operational routines as supervised jobs. Protected/system jobs are treated carefully, and provider state is read-only unless an explicit safe action is available.

### 5. Local app settings and provider configuration

Settings own local profile configuration, enabled tools/providers, model/runtime preferences, and app-level controls. Secrets and provider implementation details must stay local and out of public UI copy.

## Architecture at a glance

- **Desktop shell:** Tauri
- **Frontend:** React + TypeScript + Vite
- **Native/backend layer:** Rust Tauri commands
- **Styling:** shared app CSS with workspace-scoped design systems
- **Local persistence:** local files/app state through the desktop bridge
- **External operations:** GitHub CLI, Hermes CLI, and provider APIs through explicit native/runtime read-back

## Repository layout

```text
src/                 React/TypeScript frontend workspaces
src-tauri/           Tauri/Rust native shell and command bridge
Docs/                Product, release, and module documentation
public/              Static assets served by the frontend
scripts/             Local verification, packaging, and helper scripts
```

## Operating principles

1. **No fake success states.** The UI must distinguish requested, queued, verified, scheduled, failed, and unavailable states.
2. **No raw implementation leakage.** Provider IDs, raw timestamps, internal paths, stack traces, and tokens should not appear in user-facing copy unless explicitly needed for debugging.
3. **Local artifacts stay local.** Planning notes, decisions, proof screenshots, generated reports, and agent workdocs belong in ignored local paths unless they are intentionally product documentation or app assets.
4. **Visual changes need visual proof.** PRs should include a `Visual proof` section. Proof files should not be committed to the repository unless they are actual app assets; link externally only when explicitly approved.
5. **Every meaningful change is verified.** Build/test output, native/browser smoke checks, and feature critique gates are part of delivery, not afterthoughts.

## Development

Install dependencies:

```bash
npm install
```

Run the browser preview:

```bash
npm run dev
```

Run the native desktop app:

```bash
npm run tauri:dev
```

Build the frontend:

```bash
npm run build
```

Build desktop release artifacts:

```bash
npm run tauri:build
```

Run Rust tests:

```bash
npm run test:rust
```

## Verification and release gates

GitHub Actions/CI/CD is unavailable for this GitHub account because workflows fail before jobs start with `startup_failure`. The active release gate is therefore local-first verification.

Use these scripts as the source of truth:

- `npm run verify:push` — fast local push gate: Rust tests and frontend build.
- `npm run verify:local` — normal local development gate: Rust tests and frontend build.
- `npm run verify:release` — full local release gate: dependency check/install, Rust tests, frontend build, Tauri `.app` packaging, deterministic DMG creation, and DMG/app inspection.
- `npm run hooks:install` — installs a local-only pre-push hook that runs the push gate before pushing from this checkout.

`npm run verify:release` produces unsigned internal macOS artifacts only. Public macOS distribution requires Developer ID signing and notarization.

See [Docs/release/macos-dmg-signing-notarization.md](Docs/release/macos-dmg-signing-notarization.md) for the macOS DMG signing, notarization, and verification checklist.

## Pull request standard

Each PR should be readable without local context and include:

- **What changed**
- **Why it changed**
- **Proof/tests**
- **Risks/notes**
- **Visual proof**

Do not commit screenshots, generated proof files, local planning docs, or agent decision records unless they are used by the app or are intentional product documentation.

## Security and privacy

- Never commit secrets, tokens, passwords, provider credentials, local database files, or environment files.
- Keep provider-specific implementation details out of product-facing copy unless the operator explicitly needs them.
- Treat pasted credentials as exposed and rotate them outside the repository.
- Prefer local read-back and explicit bridge errors over silent failure.

## Documentation policy

Durable product, release, and module documentation belongs in `Docs/`.

Local-only implementation notes, plans, decisions, proof screenshots, review artifacts, and generated reports should stay under ignored paths such as `.local-workdocs/`, `.hermes/`, `Docs/proofs/`, `Docs/plans/`, or `Docs/decisions/`.
