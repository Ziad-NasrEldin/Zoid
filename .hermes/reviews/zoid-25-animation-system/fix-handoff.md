# Zoid 25 Animation Fix Handoff

Scope: fix the three REQUIRED_FIXES from the ruthless line-by-line review of commit 0a3c8f8.

Changed areas:
- src/agents/AgentsHermesScreen.tsx: active Hermes run cleanup is now scoped through clearActiveHermesRunIfCurrent(sessionId, assistantId), so a finally block cannot delete a newer queued run that already became active.
- src/agents/AgentMonitorPanel.tsx: focusable/clickable monitor panel now exposes role="button" and handles Enter/Space when focus is on the panel itself.
- src-tauri/src/lib.rs: file manager listing, repository scanning/details, and GitHub branch/default-branch paths tolerate permission marker persistence failures while still touching the user-selected path when possible.
- src/scaffold.test.ts: source invariants assert the run-cleanup guard and monitor-panel keyboard semantics.
- src-tauri/src/lib.rs tests: added regression coverage for marker persistence failures across file listing, repository scan, and branch lookup.

Verification already run:
- ./node_modules/.bin/tsx src/scaffold.test.ts: pass
- cargo test --manifest-path src-tauri/Cargo.toml marker_cannot_persist --lib -- --test-threads=1: 3 passed
- npm run test:frontend: pass
- npm run test:rust: 79 passed, 1 ignored
- npm run build: pass

Reviewer instructions:
- Review only the current working-tree diff against HEAD.
- Confirm the three prior blockers are resolved without introducing new required fixes.
- Pay special attention to agent queue/run lifecycle races, keyboard accessibility, and macOS/Tauri permission-marker semantics.
