# Feature Handoff: Responsive desktop shell sizing

## Original request

"the desktop sizing didnt make me see the right side o the screen, plese make sure its responsive"

## Implementation summary

- Removed the fixed `body` minimum width that could push the right side of the Tauri viewport offscreen.
- Made the main Zoid shell use `width: 100vw`, `min-width: 0`, and clamped sidebar columns.
- Added responsive breakpoints:
  - <=1100px: compresses sidebars/topbar padding and collapses stats to two columns.
  - <=820px: reclaims the blue rail, changes primary nav to a top horizontal row, and gives the workspace the full viewport width.
  - <=560px: stacks composer controls and hides avatars to keep content visible.
- Added regression checks that fixed desktop sizing does not return and the stats/sidebar collapse rules exist.

## Changed files

- `src/App.css`: responsive shell, sidebar, Agents topbar/composer/stats rules.
- `src/scaffold.test.ts`: source-contract checks for no fixed min width and responsive collapse rules.

## How to test

- `npm run test`
- `npm run build`
- Open Agents in the browser/native app and verify the right side of the workspace is visible without horizontal clipping.
- Resize the desktop app narrower and verify the sidebar compresses/collapses instead of pushing the Agents content offscreen.

## Tests run

- `npm run test`: PASS. Frontend scaffold test passed and Rust tests passed (`4 passed`).
- `npm run build`: PASS. TypeScript and Vite production build succeeded.
- Browser smoke at `http://127.0.0.1:1420/`: PASS. Clicked Agents; console layout probe reported `innerWidth: 1280`, document/body `scrollWidth: 1280`, and `.hermes-chat-shell` right edge `1280`, so the workspace no longer exceeds the viewport at the checked desktop size.
- Browser visual inspection: PASS. Agents page right side and repository controls are visible; no horizontal clipping observed.

## Git info

- Branch: `main`
- Commit SHA: not yet committed.
- Diff base: previous commit `ff20fb1 feat: add agents session metrics`.
- Note: repo has unrelated untracked review folders from other tasks; this handoff focuses only on `src/App.css` and `src/scaffold.test.ts`.

## Frontend/backend/database notes

- Frontend: global desktop shell and Agents screen layout CSS only.
- Backend: none.
- Database: none.

## Reviewer focus areas

- Confirm no fixed minimum body/shell width remains.
- Confirm responsive breakpoints preserve right-side visibility.
- Confirm this does not remove the Agents metrics/repository-link UI.
- Confirm verification is truthful: browser smoke was performed; native rebuild/relaunch will be done after approval/commit.

## Fix cycle notes

Initial review request.
