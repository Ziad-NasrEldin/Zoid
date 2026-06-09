# Feature Handoff: Remove Hermes session rail list

## Original request

remove the session rail list from hermes desktop , check screenshot

## Implementation summary

- Removed the visible Hermes session rail/list from the Agents/Hermes chat workspace.
- Reclaimed the rail column so the chat pane uses a single full-width `minmax(0, 1fr)` workspace column.
- Removed rail resize/compact/list UI state, constants, JSX, imports, and related CSS.
- Kept the existing chat session persistence and active-session behavior so the current Hermes chat still loads and auto-saves.

## Changed files

- `src/agents/AgentsHermesScreen.tsx`: removed the sessions rail/list JSX and rail-specific state/handlers; chat now renders directly in the workspace.
- `src/App.css`: removed sessions rail/list/resize/compact styles and changed `.chat-workspace` to one full-width column.
- `src/App.tsx`: removed now-unused session rail archive callback wiring from the Agents screen props.
- `src/scaffold.test.ts`: updated scaffold assertions to require the rail/list UI to stay absent and the chat workspace to reclaim full width.

## How to test

- `PATH="$HOME/.hermes/node/bin:$PATH" npm run build`
- `PATH="$HOME/.hermes/node/bin:$PATH" npm run test:frontend`
- Open `http://127.0.0.1:1420`, navigate to Agents, verify no left Sessions rail/list or New session row is visible.
- Browser DOM check:
  - `.sessions-rail`, `.session-tab`, `.session-tab-row` count is `0`.
  - body text has no `sessions-rail`, `session-tab`, `New session`, or `Opened Hermes sessions` rail terms.
  - `.chat-workspace` grid columns resolve to one full-width column.

## Tests run

- `PATH="$HOME/.hermes/node/bin:$PATH" npm run build`: PASS; Vite built successfully, with existing chunk-size warning.
- `PATH="$HOME/.hermes/node/bin:$PATH" npm run test:frontend`: PASS.
- `PATH="$HOME/.hermes/node/bin:$PATH" npm run test`: BLOCKED at Rust step because `cargo` is not installed / not on PATH.
- `PATH="$HOME/.hermes/node/bin:$PATH" npm run tauri:build`: BLOCKED because Tauri cannot run `cargo metadata`; `cargo` missing.
- Browser DOM verification at `http://127.0.0.1:1420` Agents page: PASS — `sessionsRailCount: 0`, `bodyHasRailTerms: []`, `chatWorkspaceColumns: "908.812px"`, `chatWorkspaceWidth: 909`, `chatMainWidth: 909`.
- Browser screenshot verification: PASS — Agents page shows the main chat directly beside the primary navigation; no left Sessions rail/list/New session row.

## Git info

- Branch: current working tree has broad pre-existing dirty/untracked Zoid changes.
- Scoped intended files: `src/agents/AgentsHermesScreen.tsx`, `src/App.css`, `src/App.tsx`, `src/scaffold.test.ts`.

## Frontend/backend/database notes

- Frontend: Hermes Agents chat workspace only.
- Backend: no backend changes for this scoped removal.
- Database: not applicable.

## Reviewer focus areas

- Confirm the screenshoted rail/list is removed, not hidden behind an empty reserved column.
- Confirm rail-specific state/styles/controls do not remain in active source.
- Keep review scoped to the four intended files because the repository already has unrelated dirty files.
- Note that native packaged verification is blocked by missing `cargo`; browser/source/build checks passed.

## Fix cycle notes

Initial review request.
