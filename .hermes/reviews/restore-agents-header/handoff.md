# Feature Handoff: Restore Agents header/title

## Original request

User asked: "where did the title/header that was at the top of the page in Agents page, i liked it" and then "i cant see it, rerun the app".

## Implementation summary

- Restored the Agents page top header/title block with kicker `AGENTS / HERMES TERMINAL` and title `Hermes Agent`.
- Kept the newer session controls (`New session`, `Save sessions`, save status) in a separate `topbar-session-actions` group beside the restored title.
- Added responsive CSS so the restored header remains visible and scales down on narrower windows.
- Updated scaffold regression test from requiring the title to be absent to requiring the restored title/header and compact session actions.

## Changed files

- `src/agents/AgentsHermesScreen.tsx`: restored the title/header block in the Hermes Agents topbar and grouped session action controls.
- `src/App.css`: restored title styling and topbar layout/responsive behavior.
- `src/scaffold.test.ts`: updated regression assertions to protect the restored title/header.

## How to test

- Run `npm run test:frontend`.
- Run `npm run build`.
- Run `npm run tauri:build`.
- Replace `/Applications/Zoid 25.app` with `src-tauri/target/release/bundle/macos/Zoid 25.app`, relaunch `/Applications/Zoid 25.app`, and verify the Agents page shows `AGENTS / HERMES TERMINAL` and `Hermes Agent` at the top.

## Tests run

- `npm run test:frontend && npm run build`: PASS.
- `npm run tauri:build`: PASS.
- Relaunched `/Applications/Zoid 25.app`: PASS, running process `/Applications/Zoid 25.app/Contents/MacOS/zoid` PID 69087.
- Screenshot verification: PASS, `/tmp/zoid-agents-title.png` shows restored kicker/title at top of Agents page.

## Git info

- Branch: current working tree.
- Commit SHA: not committed.
- Diff base: existing working tree with unrelated dirty files present.

## Frontend/backend/database notes

- Frontend only. No backend/database changes.
- Tauri app was rebuilt and installed into `/Applications/Zoid 25.app`.

## Reviewer focus areas

- Confirm restored title/header does not hide the session controls or status/repository controls.
- Confirm regression test aligns with the user preference to bring back the title.
- Confirm no unrelated dirty files are claimed as part of this scoped fix.

## Fix cycle notes

Initial review request.
