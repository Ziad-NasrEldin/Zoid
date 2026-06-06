# ADR 0001: Desktop Stack After Pre-PRD Spikes

Date: 2026-05-31
Status: Accepted with constraints
Related plan: `/Users/ziadnasreldin/Zoid/Docs/2026-05-31-zoid-implementation-plan-v1.md`
Related spikes:
- `/Users/ziadnasreldin/Zoid/Docs/spikes/2026-05-31-pty-cli-runtime-spike.md`
- `/Users/ziadnasreldin/Zoid/Docs/spikes/2026-05-31-browser-webview-spike.md`
- `/Users/ziadnasreldin/Zoid/Docs/spikes/2026-05-31-native-macos-services-spike.md`

## Context

Zoid needs a macOS-first native-feeling desktop shell with:

- Apple-style UI
- local-first storage
- CLI/PTY execution
- clean conversational session UI
- work browser/webview capture
- native notifications
- Keychain-backed credential storage
- EventKit calendar integration
- file reveal/open integration
- future Windows path

The implementation plan required technical spikes before deep product implementation. The purpose was to decide whether Tauri remains viable or whether Zoid should fall back to Electron/native Swift helper before the architecture becomes expensive to change.

## Decision

Proceed with Tauri + React + TypeScript as the preferred desktop stack for the first implementation path.

Use:

- Desktop shell: Tauri + React + TypeScript
- Native layer: Rust/Tauri commands
- UI: custom Apple-style React component system
- Database: SQLite with migrations
- Logs: app-support file logs, referenced by SQLite metadata
- CLI runtime: native subprocess/PTY service, initially in Rust/Tauri if feasible
- Secrets: macOS Keychain, but not used for real credentials until the native-app-context Keychain check passes
- Browser: scoped work webview/capture workspace, not a full personal browser

Do not treat Tauri as fully de-risked yet. It remains accepted only with the constraints and follow-up validation below.

## Spike Findings

### Spike A: PTY / CLI Runtime

Result: Partial.

Findings:
- Runtime primitives for shell/PTY execution are feasible.
- Shell command execution in cwd worked.
- Interactive PTY execution worked.
- stdout/stderr streaming worked.
- stdin worked.
- cancellation by process group worked.
- logs persisted under app support.
- SQLite stored metadata/log references.
- exit code, duration, and failure state were detected.
- basic log rotation worked.
- obvious secrets were redacted.
- A real Tauri/React prototype UI was not built; Clean Session UI rendering remains unverified beyond a text/card parser prototype.

Impact:
- Runtime primitives are feasible, but the first real app-shell task must prove UI streaming/card rendering from the Tauri runner.
- Node helper is not required by evidence yet; keep optional.
- Production implementation still needs real Rust/Tauri PTY validation and terminal-output cleanup.

### Spike B: Browser / WebView

Result: Partial.

Findings:
- Work-tab abstraction and URL/title/history metadata are feasible.
- Capture records and screenshot references are feasible through fallback/prototype paths.
- HTTP status evidence should come from separate HTTP checks, not WebView internals.
- Tauri APIs support WebView/WebviewWindow, navigation/title/load hooks, data-store concepts, eval/init scripts, and cookies.
- No strong first-class Tauri screenshot API was found.
- Robust production console/error capture is not proven.
- Google/OAuth-style embedded webview login is not safe to promise.

Impact:
- Tauri remains viable for a work webview/capture workspace.
- Do not position Browser as a personal full browser.
- Launch Gate must not depend on webview console capture.
- Follow-up native Tauri mini-spike is required for WKWebView screenshot/data-store behavior after Rust/Tauri setup.

### Spike C: Native macOS Services

Result: Partial.

Findings:
- Swift/macOS tools are available.
- app support and visible folder creation worked.
- Finder reveal/open file worked.
- basic notification via `osascript` worked.
- custom `zoid://` route/click handling is not configured/proven.
- EventKit compiled but permission/read/create did not complete in CLI/temp bundle context.
- Keychain write/read/delete was not verified because the guarded command was denied and must not be retried without explicit approval/native-app context.

Impact:
- Tauri remains plausible for native macOS services.
- Keychain, EventKit, and notification click routing remain blocking follow-up validations before real credentials/calendar/click-routing features.

## Consequences

### Positive

- Keeps future Windows path open.
- Supports polished React UI and custom Apple-style design system.
- Avoids Electron overhead unless proven necessary.
- CLI runtime appears feasible without switching stacks.
- Browser workspace can be scoped safely around work capture and verification evidence.

### Negative / Risks

- Rust/Cargo was not installed during Browser spike, so no real Tauri app was compiled yet.
- Keychain is not yet proven in this environment.
- EventKit is not yet proven from an actual app bundle.
- Native notification click routing is not proven.
- Tauri screenshot/console capture may require custom native bridge or fallback tooling.
- PTY output cleanup will require careful parsing to avoid raw-terminal noise in Clean Mode.

## Required Follow-Up Gates

Before implementing real credentials/OAuth/API integrations:

1. Prove Keychain write/read/delete from the actual app/native command path.
2. Confirm secrets never appear in SQLite, logs, events, prompts, or visible files.

Before implementing Apple Calendar features:

1. Prove EventKit permission prompt from a real bundled app context.
2. Read upcoming events after permission.
3. Create a test event only after explicit confirmation.

Before relying on native notification click behavior:

1. Register app URL/deep-link route or equivalent Tauri route.
2. Send a notification that opens the relevant Zoid route when clicked.

Before implementing Browser as more than work capture:

1. Compile/run a native Tauri WebView mini-app.
2. Verify persistent data store behavior on macOS.
3. Choose screenshot strategy.
4. Keep console capture best-effort unless robustly proven.

Before deep product build:

1. Install/confirm Rust/Cargo toolchain.
2. Initialize the Tauri app shell.
3. Implement a minimal real PTY/session runner inside the chosen stack.

## Revisit Triggers

Reopen this ADR and evaluate Electron or native Swift helper if:

- Tauri/Rust PTY cannot support stable interactive CLI sessions.
- Tauri process supervision cannot safely cancel process trees.
- Browser screenshot/capture cannot be implemented acceptably.
- Keychain/EventKit cannot be integrated cleanly through Tauri/native commands.
- Apple-style UI or native packaging quality is materially worse than expected.
- Required macOS permissions/sandboxing make Zoid's core local workflow unreliable.

## Final Decision Statement

Tauri + React + TypeScript remains the correct first implementation path for Zoid, but the architecture must treat native macOS integrations as staged gates, not assumed solved features.

Proceed to Phase 1 Secure Foundation only after installing/confirming Rust/Cargo and proving the minimal Tauri app shell can run locally.
