# Feature Handoff: Hermes message copyable responses

## Original request

"why cant i copy the text that hermes responded to me with, any response should be copyable and add a small copy icon beside each response that copies it"

## Implementation summary

- Added a small copy icon beside every visible Hermes chat message bubble.
- The copy action writes the visible, plumbing-stripped message text to the clipboard and shows a copied/failed state.
- Restored normal text selection by marking message content selectable and preventing the chat-stage click-to-focus handler from stealing focus when pointer-down starts inside a message bubble or the copy button.
- Added scaffold coverage to guard the selectable message/copy-icon behavior.

## Changed files

- `src/agents/MessageBubble.tsx`: copy button, clipboard helper, copied/failed button state, selectable content class.
- `src/agents/AgentsHermesScreen.tsx`: chat-stage pointer guard now excludes message bubbles and copy buttons so drag selection is not interrupted.
- `src/App.css`: message bubble frame, small copy icon styling, copied/failed states, explicit `user-select: text` on message content.
- `src/scaffold.test.ts`: regression guard for selectable Hermes messages and copy icon.

## How to test

- Open Zoid, go to Agents / Hermes.
- Drag-select text inside a Hermes response: selection should remain possible and should not jump focus to the composer.
- Click the small clipboard icon beside a response: it should copy the visible response text and briefly switch to a check icon.

## Tests run

- `npm run test:frontend`: PASS
- `npm run build`: PASS (Vite chunk-size warning only)
- Browser smoke at `http://127.0.0.1:1420/`: PASS — Agents/Hermes screen renders a `Copy Hermes message` button, message text is present, computed `user-select` is `text`.

## Git info

- Branch: current working tree
- Commit SHA: not committed
- Diff base: current dirty working tree includes substantial pre-existing unrelated Zoid work; this handoff concerns the scoped files listed above.

## Frontend/backend/database notes

- Frontend route/components: Hermes chat message rendering only.
- Backend endpoints/services: none.
- Database: none.

## Reviewer focus areas

- Ensure selection is not stolen by the chat-stage click-to-focus behavior.
- Ensure copy text uses the visible response content, not hidden terminal command plumbing.
- Ensure the copy affordance is small, accessible, and appears beside each visible message bubble.

## Fix cycle notes

Initial review request.
