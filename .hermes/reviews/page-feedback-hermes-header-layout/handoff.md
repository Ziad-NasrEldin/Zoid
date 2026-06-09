# Feature Handoff: Page feedback Hermes header/layout cleanup

## Original request

User provided 10 Page Feedback items for `/` at 1920×1018:
1. Remove `AGENTS / HERMES TERMINAL` kicker and adjust header alignment.
2. Remove sidebar `.window-controls` section.
3. Remove `macOS AI operating scaffold` brand subtitle.
4. Shrink `.connection-panel` to fit its text.
5. Remove `.topbar-session-actions` autosave status.
6. Make `.chat-composer` reclaim the left session rail/list space.
7. Make `.chat-stats-strip` inner containers flex by content instead of uniform cramped columns.
8. Replace Apple-default repository select with styled/branded dropdown.
9. Remove repository helper `<small>` path text.
10. Add a notification dot when an agent finishes a task/sends a response and waits for the user.

User requested final confirmation that all 10 items were finished and verified.

## Implementation summary

- Removed Hermes topbar kicker and autosave status from the rendered header.
- Removed sidebar faux window controls and brand subtitle from rendered markup.
- Reworked Hermes topbar alignment with compact connection status and styled repository dropdown.
- Removed repository helper/path text under the dropdown.
- Ensured chat workspace/composer stays full-width after session rail removal and composer explicitly spans width.
- Changed footer stats from uniform grid columns to flex/content-based sizing with long sections allowed to flex/truncate.
- Added `hasHermesWaitingNotification` derived from persisted Hermes sessions and renders a red notification dot on the Agents/session icon surfaces when the latest non-welcome message is an assistant sent/error response.
- Updated scaffold regression checks for the new cleanup and layout requirements.

## Changed files

- `src/App.tsx`: removed sidebar chrome/subtitle, removed saved-session prop wiring, added agent-waiting notification state and dots on Agents nav icons.
- `src/agents/AgentsHermesScreen.tsx`: removed kicker/autosave topbar/repository helper text, wrapped select in branded dropdown shell, removed unused topbar props.
- `src/App.css`: compacted connection panel/topbar, styled branded select, flexed stats strip, added notification dot CSS, removed unused session-save CSS and brand subtitle CSS.
- `src/scaffold.test.ts`: updated regression assertions for all 10 page-feedback items.

## How to test

- `npm run test`
- `npm run build`
- Browser preview at `http://127.0.0.1:1420/`, click Agents, inspect DOM/geometry.

## Tests run

- `npm run test`: PASS. Frontend scaffold passed; Rust tests passed 9/9.
- `npm run build`: PASS. TypeScript and Vite production build passed. Vite warned chunk >500 kB, pre-existing/non-blocking.
- Browser preview: PASS. DOM/geometry console check confirmed removed kicker/window controls/brand subtitle/autosave/repository small; branded select `appearance: none`; compact connection panel 142.98×30; stats display `flex`; composer spans chat workspace width.
- Browser notification-dot seed check: PASS. Injected a session whose latest message is assistant `sent`; `.session-tab-icon .session-notification-dot` and compact rail dot were present.
- Visual screenshot inspection in browser preview: PASS. Removed chrome/copy no longer visible, compact connection status and branded dropdown are visible, composer spans full available chat column, stats use content-sized segments.

## Git info

- Branch: main
- Commit SHA: not committed
- Diff base: working tree already had unrelated dirty/untracked Zoid work before this task; review should scope to files listed above.

## Frontend/backend/database notes

- Frontend only: React/CSS layout and scaffold regression checks.
- Backend/database: not applicable.

## Reviewer focus areas

- Confirm all 10 user feedback items are represented.
- Confirm removal is rendered, not only hidden behind copy changes.
- Confirm notification dot condition does not show for the initial welcome-only session.
- Confirm layout changes do not reintroduce session rail width or native select styling.

## Fix cycle notes

- Removed stale `.window-controls`, `.control`, `.close`, `.minimize`, `.zoom`, and mobile `.window-controls` CSS after initial critique requested cleanup.
