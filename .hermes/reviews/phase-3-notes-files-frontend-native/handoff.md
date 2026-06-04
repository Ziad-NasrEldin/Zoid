# Feature Handoff: Phase 3 Notes/Files frontend native integration

## Original request

Continue the preserved task from `/private/tmp/zoid-hermes-handoff-2026-06-04.md`: diagnose the native UI mismatch before coding, then complete the remaining Phase 3 Notes/Files frontend/native integration in a small verified slice. Do not claim Phase 3 complete until the installed `/Applications/Zoid.app` native UI visibly shows the real Notes/Files workspace controls from the current source/build.

## Implementation summary

- Diagnosed that the installed app was running the current bundle and changing active workspace state, but the generic workspace inspector/details column visually covered or displaced the real Notes/Files workspace content in the native window.
- Wired `src/App.tsx` to render the Notes and Files native bridge workspaces for `active?.id === "notes"` and `active?.id === "files"`.
- Added a native-editor layout mode that hides the generic inspector pane for Notes/Files so their real controls are visible inside the native app.
- Added panel/card styling for the Notes/Files workspaces so forms, lists, detail panels, and policy panels render as first-class native workspace UI instead of unstyled raw sections.
- Rebuilt, reinstalled `/Applications/Zoid.app`, cleared app/WebKit caches, relaunched, and verified the installed native process `/Applications/Zoid.app/Contents/MacOS/zoid`.
- Native screenshots verified visible real controls:
  - Notes: `Refresh real notes`, `Scan Markdown notes`, `Create Markdown note`, note editor fields.
  - Files: `Browse real files`, `Perform file action`, root/relative path inputs, file action area.

## Changed files

- `package.json`: frontend test script includes Notes/Files bridge/view-model tests.
- `src/App.tsx`: imports Notes/Files bridge integrations and workspaces; manages bridge state; dispatches native bridge actions; conditionally renders real Notes/Files workspaces; applies `native-editor-active` layout mode.
- `src/App.css`: adds Notes/Files native workspace layout/card/form styling; hides generic inspector pane for Notes/Files editor workspaces.
- `src/noteBridgeIntegration.ts`: Notes UI bridge state/actions around native commands.
- `src/noteBridgeIntegration.test.ts`: Notes bridge integration behavior tests.
- `src/noteViewModel.ts`: Notes view-model state/validation mapping.
- `src/noteViewModel.test.ts`: Notes view-model tests.
- `src/noteWorkspace.tsx`: Notes workspace UI with list/detail/conflicts/create/edit controls.
- `src/fileBridgeIntegration.ts`: Files UI bridge state/actions around native commands.
- `src/fileBridgeIntegration.test.ts`: Files bridge integration behavior tests.
- `src/fileViewModel.ts`: Files view-model state/validation/action mapping.
- `src/fileViewModel.test.ts`: Files view-model tests.
- `src/fileWorkspace.tsx`: Files workspace UI with browse/preview/policy/action controls.

## How to test

1. `npm run verify:local`
2. `npm run test:frontend && npm run build && npm run tauri:build -- --bundles app`
3. Reinstall native app from `src-tauri/target/release/bundle/macos/Zoid.app` to `/Applications/Zoid.app`, clear app/WebKit caches, launch it.
4. Use macOS accessibility or UI interaction to click `Notes`, capture a screenshot, and confirm real Notes controls are visible.
5. Use macOS accessibility or UI interaction to click the exact sidebar `Files Local file manager and Zoid-aware attachments.` button, capture a screenshot, and confirm real Files controls are visible.

## Tests run

- `npm run test:frontend && npm run build`: PASS. All frontend tests passed and Vite build produced `dist/assets/index-DY2vaSQz.css` and `dist/assets/index-CR4HAcWD.js`.
- `npm run tauri:build -- --bundles app`: PASS. Built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid.app`.
- Reinstall/launch `/Applications/Zoid.app`: PASS. Process verified as `/Applications/Zoid.app/Contents/MacOS/zoid` with PID `93987` during verification.
- Native Notes screenshot `/tmp/zoid-native-notes-editor-active.png`: PASS. Visible native controls include `Refresh real notes`, `Scan Markdown notes`, and `Create Markdown note`.
- Native Files screenshot `/tmp/zoid-native-files-exact.png`: PASS. Visible native controls include root key/relative path fields, `Browse real files`, and `Perform file action`.
- `npm run verify:local`: PASS. Rust tests: `165 passed; 0 failed; 1 ignored`; frontend tests passed; frontend build passed; local verification passed with `--skip-package`.

## Git info

- Branch: `main`.
- Commit SHA: not committed.
- Working tree is not clean; source/test files are modified/untracked and must be included before commit.

## Frontend/backend/database notes

- Frontend routes/components: single Tauri React shell; Notes/Files workspaces are rendered inside `App.tsx` based on active workspace ID.
- Backend endpoints/services: no new backend commands in this slice; the UI uses existing native bridge command names from Notes/Files bridge integration modules.
- Database: no schema migration in this slice. Native backend tests for Phase 3 Notes/Files database/service/bridge commands remain covered by `npm run verify:local` Rust tests.

## Reviewer focus areas

- Confirm native UI mismatch root cause and fix: the installed app must visibly show real Notes/Files controls, not generic details/confirmation/settings cards.
- Confirm `native-editor-active` does not regress non-Notes/Files workspace inspector behavior.
- Confirm Notes/Files bridge state handlers do not fabricate data and fail truthfully when native commands return empty/error states.
- Confirm the new CSS/layout is maintainable and does not hide required review/policy surfaces for consequential actions.
- Confirm all new untracked files are included in the implementation set.

## Fix cycle notes

Initial critique request after native GUI verification succeeded for Notes and Files.
