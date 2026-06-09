# Feature Handoff: Zoid Hermes sessions UI

## Original request

User said they still cannot see these features in Zoid 25:

- Added a compact New session button in the top chat header.
- Added a separated left Sessions rail for opened sessions.
- Switching sessions now uses that rail without deleting old sessions.
- Composer sizing aligned: attach/context button, input, and send button now share the same height token.
- Fixed the desktop lint blocker and the TUI import-order lint issue from the existing working diff.

## Implementation summary

- Root cause: those session UI changes existed in the Hermes Desktop repo, not in the active Zoid 25 source/bundle the user opens.
- Implemented the missing Zoid 25 Agents/Hermes UI directly in `/Users/ziadnasreldin/Zoid`.
- Added an in-chat session model with multiple opened sessions, active-session selection, and per-session messages.
- Added compact topbar `New session` button.
- Added left `Sessions` rail inside the Hermes chat workspace.
- Session switching sets `activeSessionId` and leaves old sessions in `sessions`.
- Composer controls now share `--composer-control-size` across attach, textarea, and send button.
- Preserved managed repository linking dropdown path and Code workspace wiring in the current Zoid source.
- Rebuilt, packaged, replaced `/Applications/Zoid 25.app`, relaunched the installed binary, and screenshot-verified native pixels.

## Changed files

- `src/agents/AgentsHermesScreen.tsx`: session state/model, New session button, sessions rail, active-session switching, repository dropdown props.
- `src/agents/ChatComposer.tsx`: one-row composer input aligned to shared control height.
- `src/App.css`: sessions rail layout, topbar button styling, shared composer height token, responsive behavior.
- `src/App.tsx`: repository/linking state passed to Code and Agents workspaces.
- `src/scaffold.test.ts`: regression checks for New session/session rail/session switching/composer height token.

## How to test

1. `cd /Users/ziadnasreldin/Zoid`
2. `npm run test:frontend`
3. `npm run build`
4. `npm run tauri:build`
5. Replace installed app with `src-tauri/target/release/bundle/macos/Zoid 25.app`.
6. Launch `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
7. Open Agents workspace and verify:
   - compact `New session` button in topbar;
   - left `Sessions` rail;
   - clicking session tabs switches active session without removing others;
   - attach button, text input, and Send button share the same visual height.

## Tests run

- `npm run test:frontend && npm run build`: PASS
- `npm run test:frontend && npm run build && npm run tauri:build`: PASS
- Installed app replacement/relaunch: PASS, process path `/Applications/Zoid 25.app/Contents/MacOS/zoid`
- Native screenshot: PASS, `/tmp/zoid25-sessions.png` shows the New session button, Sessions rail, and aligned composer controls.

## Git info

- Branch: current working tree in `/Users/ziadnasreldin/Zoid`
- Commit SHA: not committed
- Diff base: existing dirty working tree had prior Zoid changes/review folders before this fix.

## Frontend/backend/database notes

- Frontend only for this feature.
- Backend unchanged for sessions rail; repository/Hermes CLI bridge remains existing Tauri command path.
- Database not applicable.

## Reviewer focus areas

- Confirm the requested features are in Zoid 25, not only Hermes Desktop.
- Verify session switching does not delete old sessions.
- Verify composer controls truly share `--composer-control-size`.
- Verify installed-app/native evidence is not only browser preview.

## Fix cycle notes

Initial handoff for critique.
