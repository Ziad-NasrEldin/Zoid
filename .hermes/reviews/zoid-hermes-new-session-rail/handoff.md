# Feature Handoff: Hermes new session moved into Sessions rail

## Original request

remove the "new session button" and add it to the sessions rail list
- also make sure the number of button square is alligned with the expand/minimize button as you can see from the screenshot

## Implementation summary

- Removed the topbar `new-session-button` from `AgentsHermesScreen` so the topbar only shows the auto-save status beside the restored title/status area.
- Added a `New session` row as the first item inside the `sessions-list`, wired to the existing `handleNewSession` behavior.
- Added compact-mode styling so the rail count square and expand/minimize square use the same `--session-rail-control-size`, same `justify-self: center`, and verified equal x/width in browser compact mode.
- Kept the new-session rail item visible in expanded mode and icon-only/NS-labeled in compact mode.
- Added scaffold regression checks that the topbar class is gone and the new rail-list action exists.

## Changed files

- `src/agents/AgentsHermesScreen.tsx`: removed topbar button, imported `Plus`, added first `sessions-list` row for New session.
- `src/App.css`: removed `.new-session-button`, styled `.session-new-button` / `.session-new-icon`, aligned count and morph controls via shared sizing/centering.
- `src/scaffold.test.ts`: added regression checks for new placement and removed topbar class.

## How to test

- `npm run test:frontend`
- `npm run build`
- Browser visual check on `http://127.0.0.1:1420/` with `zoid25:hermes-sessions-rail-compact=true`.
- Native app check after `npm run tauri:build`, replacing `/Applications/Zoid 25.app`, and launching it.

## Tests run

- `npm run test:frontend`: PASS.
- `npm run build`: PASS.
- Browser compact DOM geometry: PASS. `.sessions-rail-count` and `.sessions-rail-morph-button` both `w=32`, `h=32`, `left=388.6875`, `right=420.6875`; `.new-session-button` topbar selector absent.
- Browser console after visual check: PASS, no console messages or JS errors.
- `npm run tauri:build`: PASS. Built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Installed native app replaced and relaunched from `/Applications/Zoid 25.app`; running process verified: `/Applications/Zoid 25.app/Contents/MacOS/zoid` PID 8223.
- Native screenshot `/tmp/zoid-sessions-rail.png`: PASS for expected Zoid 25 app and no topbar New session; New session appears inside Sessions rail list. Native screenshot was expanded rail, not compact.

## Git info

- Branch: `main`.
- Commit SHA: not committed.
- Diff base: working tree has pre-existing unrelated dirty/untracked files; review should scope only the three changed files above and this handoff.

## Frontend/backend/database notes

- Frontend route/component: Hermes Agents screen session rail only.
- Backend endpoints/services: none.
- Database: none.

## Reviewer focus areas

- Confirm topbar no longer renders a New session button/class.
- Confirm New session is inside `sessions-list` and calls `handleNewSession`.
- Confirm compact count and expand/minimize controls share identical dimensions/alignment.
- Confirm tests are meaningful and no unrelated source changes were introduced by this task.

## Fix cycle notes

Initial review request.
