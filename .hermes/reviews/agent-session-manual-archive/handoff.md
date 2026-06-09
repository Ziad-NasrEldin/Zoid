# Agent Session Manual Persistence + Archive Review Handoff

## Scope
User requested: agent sessions should persist manually; deleting a session should move it to an archive section; archive should be accessible from Settings.

## Implemented
- Lifted Hermes chat session state from `AgentsHermesScreen` into `App`.
- Added manual `Save sessions` action that writes active sessions to `localStorage` key `zoid25:hermes-sessions`.
- Added archived session model persisted under `zoid25:hermes-archived-sessions`.
- Added per-session `Archive session` action in the Agents sessions rail.
- Archiving removes the session from active sessions, stores it in archived sessions with `archivedAt`, writes active+archive state to localStorage, and keeps at least one active session available.
- Added Settings navigation as a selectable workspace.
- Added Settings archive screen with empty state and restore action.
- Restoring moves archived session back into active sessions, persists active+archive storage, selects the restored session, and navigates back to Agents.
- Added scaffold regression checks for session storage keys, manual save button, archive action, Settings archive UI, and restore behavior.
- Added archive/settings CSS and save status styling.
- Session updates use React functional state setters so an in-flight Hermes response does not overwrite sessions created while a prompt is running.
- Save status resets to unsaved whenever active sessions change after a manual save.

## Relevant files
- `src/App.tsx`
- `src/agents/AgentsHermesScreen.tsx`
- `src/App.css`
- `src/scaffold.test.ts`

## Verification already run
- `npm run build` — passed.
- `npm test` — passed frontend scaffold + 9 Rust tests.
- `npm run tauri:build` — passed and produced `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Relaunched built app; running process: `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app/Contents/MacOS/zoid`.
- Screenshot captured at `/tmp/zoid-agent-sessions-archive.png`; app is visibly open on Agents with Save sessions and Archive session controls.

## Known context / concerns for reviewer
- Sessions intentionally do NOT auto-save normal chat changes; user must click Save sessions. Archive/restore actions persist immediately because they are explicit destructive/management actions.
- Existing codebase had many unrelated modifications before this task. Review only the relevant files above unless a related regression is discovered.
- The app currently shows prior Hermes header/title because scaffold tests require the restored visible header.

## Required review outcome
Please inspect the implementation for correctness and UX regressions. If there are Required fixes, list exact files/changes. If acceptable, return APPROVED.
