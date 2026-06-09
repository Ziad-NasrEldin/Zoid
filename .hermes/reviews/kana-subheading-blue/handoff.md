# Feature Handoff: Kana subheading blue color

## Original request

Page Feedback: / — Make all this subheading text in all of the pages blue, just like you did in the Asians page. in zoid 25

## Implementation summary

- Updated the shared `.kana-line` style to use `var(--kujo-blue)` globally.
- Added a stronger `.zoid25-shell .kana-line` rule so page-specific paragraph/header rules cannot override the blue subheading color.
- Kept the Settings-specific `p.kana-line` override because its header paragraph rule previously rendered `設定` as muted gray.
- Added a scaffold guard for the global and Settings-specific blue subheading rules.

## Changed files

- `src/App.css`: global and Settings kana-line color rules.
- `src/scaffold.test.ts`: source guard for all-page blue kana subheadings.

## How to test

- `npm run test:frontend`
- `npm run build`
- `npm run tauri:build`
- Browser computed-style check at `http://127.0.0.1:1420/` for `.kana-line` elements should return `rgb(53, 88, 162)`.
- Installed app should run from `/Applications/Zoid 25.app/Contents/MacOS/zoid`.

## Tests run

- `npm run test:frontend && npm run build`: PASS.
- `npm run tauri:build`: PASS, with existing Rust dead-code warnings for `apply_profile_runtime_args` and `prompt_with_enabled_profile_context`.
- Browser computed-style check on Code and Settings kana headings: PASS, both returned `rgb(53, 88, 162)`.
- Reinstalled and relaunched `/Applications/Zoid 25.app`: PASS, process path `/Applications/Zoid 25.app/Contents/MacOS/zoid`.

## Git info

- Branch: main
- Commit SHA, if committed: not committed
- Diff base, if known: working tree has substantial pre-existing unrelated dirty/untracked Zoid work; intended change is scoped to `src/App.css`, `src/scaffold.test.ts`, and this review folder.

## Frontend/backend/database notes

- Frontend routes/components: shared page `.kana-line` styling.
- Backend endpoints/services: none.
- Database tables/migrations: none.

## Reviewer focus areas

- Confirm the stronger shared rule covers all page subheadings, including Settings where `.settings-archive-header p` used to override color.
- Confirm the guard does not alter non-subheading body paragraph color.
- Confirm no backend/native changes were needed for this visual CSS slice.

## Fix cycle notes

Initial review request.
