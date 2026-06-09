# Feature Handoff: Hermes chat feedback fixes 2026-06-07

## Original request

Page Feedback for `/` at `tauri://localhost`:
1. Empty Hermes writing bubble: do not show the empty message bubble when Hermes is writing and the writing animation already displays.
2. Hermes avatar: current `HA` icon is boring; make it more unique and creative.
3. Sessions rail separation: add clear, creative separation between the sessions rail and chat window without hurting visual hierarchy.
4. Terminal command plumbing: do not display terminal commands used in chat responses because it looks ugly.

## Implementation summary

- Message bubbles now sanitize visible assistant content through `stripTerminalCommandPlumbing`, removing `Terminal command used:` and `$ hermes...` command lines from rendered bubbles.
- Added `shouldShowBubble` gate so a streaming assistant with no visible content does not render an empty bubble; only the writing status animation remains.
- Replaced the Hermes initials fallback with a custom sigil made from a diamond core plus orbital rings.
- Strengthened the sessions rail/chat separation with an 18px branded perforated spine and subtle dot-rhythm pseudo-element, avoiding duplicate readable labels in the rail.
- Updated scaffold regression checks for these visible chat polish requirements.

## Changed files

- `src/agents/MessageBubble.tsx`: visible content sanitizer and empty-streaming-bubble suppression.
- `src/agents/Avatar.tsx`: custom Hermes sigil markup instead of `HA` initials.
- `src/App.css`: Hermes sigil styling and sessions rail separator spine.
- `src/scaffold.test.ts`: regression checks for terminal-plumbing hiding, bubble gate, separator, and sigil.

## How to test

- `npm run test:frontend`
- `npm run build`
- `npm run test:rust`
- Browser preview at `http://127.0.0.1:1420`, open Agents and inspect the chat rail/avatar/bubbles.
- Installed app: rebuild bundle, replace `/Applications/Zoid 25.app`, relaunch using bundle id `com.mavoid.zoid25`, verify process path.

## Tests run

- `npm run test:frontend && npm run build`: PASS. Vite emitted only existing chunk-size warning.
- `npm run test:rust`: PASS, 27 Rust tests passed.
- Browser DOM check: PASS. Agents rendered Hermes sigil markup, no `HA` text, no terminal-command text in message bubbles, sessions rail pseudo separator content is empty (decorative only).
- `npm run tauri -- build --bundles app`: PASS. Built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Installed app relaunch: PASS. `/Applications/Zoid 25.app/Contents/MacOS/zoid` running with PID 66146.

## Git info

- Branch: current working tree, dirty with broad pre-existing Zoid changes outside this focused fix.
- Commit SHA: not committed.
- Diff base: current working tree before this focused edit.

## Frontend/backend/database notes

- Frontend: `MessageBubble`, `Avatar`, `App.css` only.
- Backend: no backend behavior changed; existing Rust tests still prove terminal usage is not added by the bridge.
- Database: not applicable.

## Reviewer focus areas

- Verify all four Page Feedback items are satisfied in the focused diff.
- Confirm the sanitizer does not remove normal Hermes response text.
- Confirm the sessions rail separator is decorative and does not duplicate accessible/visible labels.
- Confirm the custom Hermes avatar works when no `avatarUrl` is provided.

## Fix cycle notes

Initial handoff after implementation and local/native verification.
