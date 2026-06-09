# Feature Handoff: Page feedback composer, welcome copy, sidebar Agents icon

## Original request

in zoid 25
## Page Feedback: /
**Viewport:** 1920×1018

1. textarea `.chat-composer > .composer-input-column > .composer-input-wrap > textarea`: Text being written is not aligned in the middle of the box, and second-line text is very close to touching the line beneath. Please fix.
2. paragraph default message: change the default message to be something cooler.
3. span `.editorial-sidebar > .nav-list > .nav-row > .nav-icon` with classes `nav-icon, session-tab-icon` next to Agents: icon is bugged and not like the rest of the sidebar icons.

## Implementation summary

- Adjusted composer textarea vertical rhythm: kept the shared 44px control height, changed padding to `10px 14px`, and increased line-height to `1.45` so single-line text is vertically centered and multiline text has more breathing room.
- Replaced the default Hermes welcome message with cooler Zoid-local command-deck copy.
- Added `refreshHermesWelcomeCopy` so existing persisted sessions that still contain the old welcome copy are migrated on load, not only new sessions.
- Stopped the primary sidebar Agents icon from inheriting `.session-tab-icon` boxed session styling by changing it to `nav-icon nav-icon--agent-session`, while keeping notification-dot anchoring through a scoped CSS rule.
- Added scaffold regression checks for the textarea metrics, welcome-copy migration, and sidebar icon class separation.

## Changed files

- `src/App.css`: textarea padding/line-height; primary nav icon sizing; scoped Agents notification-dot positioning.
- `src/App.tsx`: primary sidebar Agents icon now uses `nav-icon--agent-session` instead of `session-tab-icon`.
- `src/agents/AgentsHermesScreen.tsx`: updated default Hermes welcome message and exports a persisted-session welcome-copy migration helper.
- `src/scaffold.test.ts`: added regression checks for the three page-feedback items, including persisted legacy welcome migration.

## How to test

- `npm run test:frontend`
- `npm run build`
- Browser preview at `http://127.0.0.1:1420/`, open Agents and inspect:
  - first assistant message text;
  - textarea computed height/padding/line-height;
  - Agents nav icon class and computed border/background.

## Tests run

- `npm run test:frontend && npm run build`: PASS after the persisted-session welcome migration. Vite emitted only the pre-existing chunk-size warning.
- Browser DOM check on `http://127.0.0.1:1420/`: PASS.
  - message: `Hermes is awake. Drop the mission, the repo, or the mess — Zoid will route it through your local command deck.`
  - textarea: height `44`, padding top/bottom `10px`, line-height `21.75px`.
  - Agents nav icon: class `nav-icon nav-icon--agent-session`, border `0px none`, transparent background, 30×30 box.
- Native screenshot initially showed an old persisted session with the legacy welcome copy; this drove the added `refreshHermesWelcomeCopy` migration and rerun build.

## Git info

- Branch: current working tree in `/Users/ziadnasreldin/Zoid`.
- Commit SHA: not committed.
- Diff base: current repo index; note the repo already had broader unrelated dirty files before this scoped fix.

## Frontend/backend/database notes

- Frontend-only visual/copy fix.
- No backend, Tauri command, or database changes required.

## Reviewer focus areas

- The primary sidebar Agents icon must no longer carry `.session-tab-icon` styling.
- The textarea change must preserve the shared 44px composer control height while improving vertical centering and multiline spacing.
- The old default-message copy must be absent.

## Fix cycle notes

Initial handoff.
