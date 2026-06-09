# Ruthless fix re-review: zoid-25-animation-system

Verdict: REQUIRED_FIXES

Scope reviewed: current working-tree diff against HEAD only.

Required fixes:
- `src/agents/AgentMonitorPanel.tsx:50` still makes the whole monitor panel an ARIA `role="button"` while the panel contains nested interactive controls at `src/agents/AgentMonitorPanel.tsx:73-78` (`textarea`, Send, Stop, Continue). This is not a clean accessibility fix: a button-role container must not wrap other focusable controls. Keep keyboard activation for focusing the panel, but do not expose the entire composite panel as one button containing nested buttons/form fields. Use a non-button composite/group plus a separate explicit focus/select button, or another ARIA pattern that does not nest interactive controls inside a button.

Prior required fixes:
- AgentsHermesScreen active run queue cleanup race: fixed. Cleanup now goes through `clearActiveHermesRunIfCurrent`, which deletes only when the completed `assistantId` still matches the active run; completion paths call it instead of unconditional session deletes.
- AgentMonitorPanel keyboard/button accessibility: partially fixed but still blocking. Enter/Space activation exists, but the chosen `role="button"` wrapper introduces invalid nested-interactive semantics.
- src-tauri permission marker persistence failures: fixed. File manager listing, repo scan, default branch update, and branch listing now use best-effort marker persistence; Rust tests cover marker-write failure for file manager, repo scan, and GitHub branch lookup.

Verification:
- `npx tsx src/agents/AgentsHermesScreen.file-manager.test.tsx && npx tsx src/agents/dashboardLayoutState.test.ts && npx tsx src/scaffold.test.ts`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib -- --test-threads=1`: passed, 79 passed, 1 ignored.
- `cargo check --manifest-path src-tauri/Cargo.toml`: passed.
- `npm run test:frontend -- --runInBand`: first run failed in `SocialDashboard.behavior.test.tsx` with a safe-media validation assertion, but focused rerun and full rerun passed. I am not treating that as the blocker.
- `npm run build`: passed.
