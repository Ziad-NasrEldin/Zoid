# Feature Handoff: Sessions rail overflow cue

## Original request

Page Feedback for `/` at `tauri://localhost`: the `div.sessions-list` in the Hermes sessions rail has a very big scrollbar. Remove it and instead add a flowing animating button at the bottom of the array/list that indicates scrollable content below. If the user scrolls down, the button disappears.

## Implementation summary

- Hid the native Sessions rail scrollbar while preserving the existing scrollable list behavior.
- Added a bottom overlay button, `More sessions below`, that appears only when the sessions list has overflow, has a real positive layout height, and is still at the top.
- The sessions rail now owns the full left-side chat workspace height while the composer stays in the main chat column, preventing the list from collapsing to 0px in browser/Tauri preview layouts.
- The cue animates with a gentle vertical flow plus sheen and preserves centered placement when `prefers-reduced-motion` disables animation.
- Clicking the cue scrolls the sessions list downward and immediately hides the cue.
- The cue is intentionally accessible as a real button rather than a decorative-only element.

## Changed files

- `src/agents/AgentsHermesScreen.tsx`: added sessions-list ref/state, overflow measurement, scroll/resize listeners, cue click behavior, and rendered the cue below the sessions list.
- `src/App.css`: hid the native scrollbar, fixed sessions rail/list height ownership, kept the composer in the main chat column, and added the animated cue styles/reduced-motion fallback.
- `src/agents/AgentsHermesScreen.file-manager.test.tsx`: added focused regressions for scrollbar hiding, no false cue on zero-height/non-overflow states, and cue show/hide behavior.
- `src/scaffold.test.ts`: repaired an unrelated duplicate/stray block terminator in the already-dirty scaffold guard so TypeScript/build could run; no product behavior change.

## How to test

- Run `npx tsx src/agents/AgentsHermesScreen.file-manager.test.tsx`.
- Run `npm run build`.
- In the app, open the Hermes sessions rail with enough sessions to overflow. The native scrollbar should not be visible. A small animated button should show at the bottom while at the top, then disappear after scrolling down.

## Tests run

- `npx tsx src/agents/AgentsHermesScreen.file-manager.test.tsx`: PASS.
- `npm run build`: PASS.
- `npm run test:frontend -- --runInBand`: PASS after final rerun; prior re-review's `.blue-rail::before` scaffold failure is no longer reproducible in the current tree.
- Browser check at `http://127.0.0.1:1420/`: PASS for one-session no-cue state (`clientHeight: 129`, `scrollHeight: 129`, cue false) and forced 12-session overflow state (`clientHeight: 161`, `scrollHeight: 780`, cue true at top, cue false after `scrollTop = 40`, `scrollbar-width: none`).

## Git info

- Branch: main
- Commit SHA, if committed: not committed
- Current HEAD: 6a63013
- Dirty tree note: repository already contains unrelated modified/untracked files outside this slice; review should scope to the three changed files listed above unless evaluating aggregate gate drift.

## Frontend/backend/database notes

- Frontend route/component: Hermes/Agents chat screen, sessions rail only.
- Backend endpoints/services: none.
- Database tables/migrations: none.

## Reviewer focus areas

- Confirm the scrollbar is visually hidden without disabling keyboard/mouse scroll behavior.
- Confirm the cue only appears when overflow exists and the list is at the top.
- Confirm the cue disappears on user scroll down.
- Confirm the absolute overlay does not cover core session interactions in a way that prevents opening/archiving sessions.
- Confirm reduced-motion behavior is present.

## Fix cycle notes

Fixes after first critique report:
- R1: prevented zero-height false overflow cues, made the sessions rail span the full left workspace height, gave `.sessions-list` flex growth, and moved the composer to the main chat column so the rail list has usable height.
- R2: extended the focused regression to assert no cue before real overflow and no cue for zero-height false-overflow metrics.
- I1: added base `transform: translate(-50%, 0)` so reduced-motion mode remains centered when keyframe animation is disabled.
- Re-ran focused tests, build, aggregate frontend tests, and browser checks for both short and overflowing session lists.
- After the re-review reported aggregate scaffold failures, reran `npx tsx src/scaffold.test.ts` (exit 0) and `npm run test:frontend -- --runInBand` (exit 0, printed `contentWorkspace tests passed`) in the current tree; both pass now.
