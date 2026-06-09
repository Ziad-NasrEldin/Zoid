# Feature Handoff: Hermes chat feedback polish 2026-06-07

## Original request

Page Feedback for `/` asked to:

1. Hide the empty assistant message bubble when Hermes is already showing the HERMES WRITING animation.
2. Replace the boring `HA` chat avatar with something more unique/creative.
3. Add a clear creative separation between the sessions rail and chat window without hurting hierarchy.
4. Stop displaying `Terminal command used: $ hermes ...` in the visible chat transcript because it looks ugly.

## Implementation summary

- Empty/whitespace-only messages no longer render `.message-bubble`; status animation remains visible for streaming/sending/error states.
- Hermes avatar now has a layered conic/radial mark, inner diamond, and small spark details while preserving initials and accessible label.
- Sessions rail now uses a subtle vertical separator spine with tinted background, dotted/rule texture, and preserved resize handle behavior.
- Hermes CLI output still executes through the backend bridge, but terminal command plumbing is stripped from user-visible assistant responses.
- Composer helper copy was updated to say CLI plumbing stays out of the conversation.
- Regression guards were added to `src/scaffold.test.ts` and Rust tests were updated for the non-visible terminal command behavior.

## Changed files

Scoped intentional changes:

- `src/agents/MessageBubble.tsx`: conditional bubble rendering for empty streaming/sending content.
- `src/agents/ChatComposer.tsx`: updated CLI-mode helper copy.
- `src/App.css`: sessions rail separator and Hermes avatar visual treatment.
- `src/scaffold.test.ts`: source guards for the visual feedback fixes and terminal-command non-display.
- `src-tauri/src/lib.rs`: stops prefixing visible Hermes responses with terminal command usage; tests updated.

Note: repository already contains many unrelated dirty/untracked Zoid changes from prior work. Review should focus only the scoped files above and the relevant hunks.

## How to test

- `npm run test:frontend`
- `npm run test:rust`
- `npm run build`
- `npm run tauri:build`
- Replace `/Applications/Zoid 25.app`, relaunch `/Applications/Zoid 25.app/Contents/MacOS/zoid`, and inspect the Hermes chat screen.

Expected behavior:

- A streaming empty assistant row shows only metadata/status animation, not an empty bubble.
- Hermes avatar is visually more distinctive than plain `HA`.
- Sessions rail has a clear vertical separator spine before the chat pane.
- Assistant chat bubbles do not contain `Terminal command used:` or raw `$ hermes ...` plumbing.

## Tests run

- `npm run test:frontend`: PASS.
- `npm run test:rust`: PASS, 25 tests passed.
- `npm run build`: PASS.
- `npm run tauri:build`: PASS, built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Installed bundle copied to `/Applications/Zoid 25.app` and launched via `/Applications/Zoid 25.app/Contents/MacOS/zoid`; process observed at `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- Browser DOM smoke on `http://127.0.0.1:1420/` after selecting Agents: rail separator pseudo-element width `14px`, Hermes avatar pseudo-element present, no visible `Terminal command used` in bubbles.
- Native screenshot captured at `/tmp/zoid25-chat-polish.png`; it shows Zoid 25 Hermes Agent screen with sessions rail and chat area visible after relaunch.

## Git info

- Branch: current local working tree (not committed by this task).
- Commit SHA: not committed.
- Diff base: working tree contains unrelated pre-existing changes; review scoped files listed above.

## Frontend/backend/database notes

- Frontend components: `MessageBubble`, `ChatComposer`, global chat CSS.
- Backend: Hermes CLI response formatting only; CLI invocation/workdir behavior remains intact.
- Database: not applicable.

## Reviewer focus areas

- Confirm the visible transcript no longer prints terminal command plumbing while CLI execution tests still prove the bridge works.
- Confirm hiding empty bubbles does not hide real errors or non-empty assistant content.
- Confirm CSS separator/avatar changes do not block interactions or disrupt hierarchy/resizing.

## Fix cycle notes

Initial review request.
