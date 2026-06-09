# Feature Handoff: Sidebar brand inline 25

## Original request

Page Feedback for `/`: `h1 "ZOID25"` in `.brand-block`; feedback: "put the 25 beside zoid but leave a small space between".

## Implementation summary

- Changed the primary sidebar brand heading from two-line `ZOID` + `25` to one inline brand mark.
- Added a dedicated `.brand-number` span and flex/gap styling so `25` sits beside `ZOID` with a small visual gap and no wrap.
- Added a scaffold guard that fails if the brand mark returns to the split/two-line layout or loses the inline gap/no-wrap styling.

## Changed files

- `src/App.tsx`: renders `ZOID<span className="brand-number">25</span>` in the sidebar brand heading instead of inserting a `<br />`.
- `src/App.css`: makes `.brand-block h1` a single-row flex heading with `gap: 0.12em` and `white-space: nowrap`; adds `.brand-number` styling.
- `src/scaffold.test.ts`: adds a source guard for the inline brand mark and CSS contract.

## How to test

- Run `npm run test:frontend && npm run build`.
- Run `npm run tauri:build`.
- Replace and relaunch `/Applications/Zoid 25.app` from the packaged app.
- Browser/DOM check: heading should be one line, computed `.brand-block h1` display should be `flex`, direction `row`, gap about `7.68px`, no wrap.
- Native screenshot check: app sidebar should visibly show `ZOID 25` on one line.

## Tests run

- `npm run test:frontend && npm run build`: PASS. Frontend scaffold/dropdown tests passed; TypeScript and Vite production build passed.
- Browser DOM check at `http://127.0.0.1:1420/`: PASS. Heading rendered as one row; `.brand-number` x-position is beside `ZOID`, h1 height is one line, computed gap is `7.68px`, `white-space: nowrap`.
- `npm run tauri:build`: PASS. Built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Installed app refresh: PASS. Replaced `/Applications/Zoid 25.app`, relaunched it, and verified process `81944 /Applications/Zoid 25.app/Contents/MacOS/zoid`.
- Native screenshot `/tmp/zoid-brand-inline.png`: PASS. Screenshot shows the installed Zoid 25 window and sidebar brand `ZOID 25` on one line with a small gap.

## Git info

- Branch: current working tree (repo has many pre-existing unrelated dirty/untracked files).
- Commit SHA: not committed.
- Diff base: existing working tree; intended scoped change is only `src/App.tsx`, `src/App.css`, and `src/scaffold.test.ts`.

## Frontend/backend/database notes

- Frontend routes/components: primary app shell sidebar brand block only.
- Backend endpoints/services: none.
- Database tables/migrations: none.

## Reviewer focus areas

- Confirm no `<br />` remains in the brand h1.
- Confirm `.brand-block h1` is single-row/nowrap and uses a small gap between `ZOID` and `25`.
- Confirm tests/build/native relaunch evidence is sufficient for this small Page Feedback UI slice.
- Ignore unrelated dirty files in the repo; review the scoped brand change only.

## Fix cycle notes

Initial review request.
