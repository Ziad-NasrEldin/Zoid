# Feature Handoff: Hermes message hover copy icon

## Original request

"ok i want you to only reveal that icon whenever i hover over a message bubble or hover closely to it , dont make it buggy , make it smooth and dependable"

## Implementation summary

- Adjusted the existing Hermes message copy affordance so the copy icon is hidden by default.
- The icon now fades/slides in smoothly when the message bubble frame is hovered, when focus is inside the frame, or while copied/failed feedback is active.
- Kept the surrounding frame as the hover target so hovering close beside the bubble reliably reveals the icon.
- Preserved message text selection and the existing copy-to-visible-text behavior.
- Added reduced-motion handling so the control remains dependable for users who prefer less motion.

## Changed files

- `src/App.css`: changed `.message-copy-button` default/hover/focus/copied/failed states to hidden-by-default with smooth reveal.
- `src/scaffold.test.ts`: strengthened regression guard to require hover reveal, default hidden state, transition, text selectability, and chat-stage focus guard.

## How to test

- Open Hermes chat in Zoid.
- Confirm the copy icon is hidden when the pointer is away from the message.
- Hover the message bubble or the area immediately beside it; the icon should fade/slide in.
- Click the icon; it should copy the visible message text and stay available during copied feedback.
- Confirm response text remains selectable.

## Tests run

- `npm run test:frontend`: PASS.
- `npm run build`: PASS. Vite emitted only the existing chunk-size warning.
- Browser smoke on `http://127.0.0.1:1420/`: PASS. Confirmed default opacity `0` / `pointer-events: none` away from the message, opacity `1` / `pointer-events: auto` on hover/click target, `userSelect: text`, and successful copy-button click.

## Git info

- Branch: not changed by this task.
- Commit SHA: not committed.
- Diff base: current working tree with many unrelated pre-existing dirty/untracked files.

## Frontend/backend/database notes

- Frontend routes/components: Hermes chat message transcript styling only.
- Backend endpoints/services: not applicable.
- Database tables/migrations: not applicable.

## Reviewer focus areas

- The copy button should not be visually always-on anymore.
- Hover target should be reliable around the bubble, not a tiny finicky target.
- Keyboard focus/copy feedback should still reveal the icon.
- Text selection and chat-stage focus guard should remain intact.
- No unrelated dirty-tree cleanup should be required for this scoped change.
