# Feature Handoff: Hermes chat stage focuses composer

## Original request

Page Feedback on `/`: “i want whenever i click on the chat window for my curost to be ready in the chat/message field so i immediately start typeing whenever i want”

## Implementation summary

- Added an imperative `ChatComposer` ref API (`focusMessageField`) that focuses the message textarea without scrolling.
- Added a `composerRef` in `AgentsHermesScreen` and a `handleChatStagePointerDown` handler on `.chat-stage`.
- Clicking non-interactive empty/message space in the chat stage now focuses the composer textarea on the next animation frame so typing can start immediately.
- Guarded interactive descendants (`button`, `a`, `input`, `textarea`, `select`, `[role='button']`, contenteditable) so existing controls are not hijacked.
- Added scaffold regression coverage for the stage click -> composer focus wiring.

## Changed files

- `src/agents/AgentsHermesScreen.tsx`: wire stage pointer-down to focus the composer through a ref.
- `src/agents/ChatComposer.tsx`: expose `focusMessageField` via `forwardRef` and attach a textarea ref.
- `src/scaffold.test.ts`: add source guard for this Page Feedback behavior and update the existing composer sibling assertion for the new ref prop.

## How to test

- `npm run test:frontend`
- `npm run build`
- `npm run tauri:build`
- Relaunch `/Applications/Zoid 25.app`, open Hermes Agent, click the chat stage, then type; the message textarea should receive focus.

## Tests run

- `npm run test:frontend`: PASS
- `npm run build`: PASS
- `npm run tauri:build`: PASS; built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`
- Reinstalled/relaunched `/Applications/Zoid 25.app`: PASS; running process `/Applications/Zoid 25.app/Contents/MacOS/zoid` (pid 98682)
- Browser functional probe on `http://127.0.0.1:1420`: PASS; dispatching `pointerdown` on `.chat-stage` produced `document.activeElement.tagName === "TEXTAREA"` and the active placeholder was the Hermes message composer.
- Native screenshot captured at `/tmp/zoid-chat-focus-after-relaunch.png`: PASS; app visible on Hermes Agent page with composer at bottom.

## Git info

- Branch: current working tree (not committed by this task)
- Commit SHA, if committed: not committed
- Diff base: current repository working tree contains many pre-existing unrelated dirty/untracked files; review this handoff as a scoped fix for the three files listed above.

## Frontend/backend/database notes

- Frontend routes/components: Hermes chat screen and composer only.
- Backend endpoints/services: none.
- Database tables/migrations: none.

## Reviewer focus areas

- Confirm clicking chat-stage focuses the actual textarea, not a fake proxy.
- Confirm interactive descendants are ignored so controls inside messages/stage remain usable.
- Confirm `ChatComposer` ref typing is safe and build passes.
- Confirm this scoped change does not depend on Hermes CLI being online; focus behavior should work even when Send is locked.

## Fix cycle notes

Initial handoff.
