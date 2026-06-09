# Feature Handoff: Hermes message rollback button

## Original request

"also add a button to quickly roll back the conversation to that message point, use hermes default command that does that"

## Implementation summary

- Added per-message rollback action beside the existing copy action in `MessageBubble` using the same hidden-until-hover action rail.
- Rollback appears only when there are later user turns to undo and is disabled while Hermes is sending.
- Clicking rollback computes the number of later user turns and runs Hermes’ default `/undo N` command through the existing slash command bridge with confirmation bypass for this explicit UI action.
- After Hermes returns, Zoid trims the local visible transcript to the clicked message point and preserves the Hermes CLI session id from the command result.
- Added failure handling that appends a visible Hermes error message if rollback fails.
- Fixed the Settings provider mount to pass `availableModels`, unblocking the production build in the current tree.

## Changed files

- `src/agents/MessageBubble.tsx`: rollback action prop/icon/button beside copy.
- `src/agents/AgentsHermesScreen.tsx`: `handleRollbackToMessage`, `/undo N` execution, local transcript trim, hover/action focus guard, per-message rollback visibility.
- `src/App.css`: rollback button styling shares the smooth hover/focus actions rail.
- `src/scaffold.test.ts`: source guard for rollback command wiring and action visibility.
- `src/App.tsx`: passes `availableModels` to `ProvidersSettings` to satisfy the existing provider-management contract.

## How to test

- Open Agents / Hermes chat.
- Hover an older message that has later user turns after it.
- Confirm the action rail reveals copy + rollback.
- Click rollback; Zoid should run `/undo <count>` against the linked Hermes session and trim messages after the clicked point.

## Tests run

- `npm run test:frontend`: PASS
- `npm run build`: PASS
- `npm run tauri:build`: PASS
- Reinstalled/relaunched `/Applications/Zoid 25.app`: PASS, running process `/Applications/Zoid 25.app/Contents/MacOS/zoid`
- Browser DOM smoke with synthetic transcript: PASS, older messages expose `Roll back conversation...` plus `Copy...`; latest messages expose copy only.

## Git info

- Branch: main working tree
- Commit SHA: not committed
- Diff base: existing dirty Zoid working tree with many unrelated changes/untracked review artifacts

## Frontend/backend/database notes

- Frontend: rollback uses the existing Tauri `executeHermesSlashCommand` helper.
- Backend: no new Rust command; relies on current slash command bridge to run Hermes CLI query `/undo N`.
- Database: none.

## Reviewer focus areas

- Verify `/undo N` count maps correctly to clicked message point.
- Verify the action rail stays hidden until hover/focus and does not interfere with text selection/copy.
- Verify failure state is visible if the slash command cannot run.
- Verify scope is limited to Hermes chat message actions.
