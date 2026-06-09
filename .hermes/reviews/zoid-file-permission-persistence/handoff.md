# Feature Handoff: Zoid file permission persistence

## Original request

Page Feedback on `/` / Code workspace Edit button: "why does everytime i open the edit menu it shows me a confimation message to allow zoid 25 to access my files and folders it should allow permenantely apply the same funcitonality on all requests that asks for permissions"

## Implementation summary

- Reworked the native file-permission warmup marker from a one-time boolean-style skip into a persisted set of touched paths.
- Added native helpers to load/save `zoid-file-permissions.json`, touch/canonicalize a folder or file path, and remember newly accessed paths.
- Applied the same path-remembering behavior to repository scan roots, repository detail reads used by default-branch Edit/list/update, and clone destination roots.
- Existing touched paths are skipped on later warmups, while new repository/folder paths are added permanently to the marker so repeated Edit/list/scan/clone calls should not keep retriggering the same app-side permission warmup.

## Changed files

- `src-tauri/src/lib.rs`: file-permission marker model, path remember/touch helpers, repository command integration, and Rust regression test.

## How to test

- `npm run test:frontend`
- `npm run build`
- `npm run test:rust`
- `npm run tauri:build`
- Replace `/Applications/Zoid 25.app` with `src-tauri/target/release/bundle/macos/Zoid 25.app`, relaunch it, and confirm the Code workspace opens in the installed app.

## Tests run

- `npm run test:rust -- warm_file_permissions --nocapture`: PASS, 2 tests.
- `npm run test:frontend && npm run build`: PASS; Vite emitted only the existing chunk-size warning.
- `npm run test:rust`: PASS, 32 tests.
- `npm run tauri:build`: PASS; built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Installed with `ditto ... /Applications/Zoid 25.app`, relaunched, and confirmed process `/Applications/Zoid 25.app/Contents/MacOS/zoid` plus screenshot `/tmp/zoid-permissions-relaunch.png` showing the Code workspace.

## Git info

- Branch: current local working tree.
- Commit SHA, if committed: not committed.
- Diff base, if known: existing repo state has many unrelated dirty/untracked files; review should scope to `src-tauri/src/lib.rs` permission changes.

## Frontend/backend/database notes

- Frontend routes/components: no UI component changes needed.
- Backend endpoints/services: Tauri commands `scan_github_repositories`, `clone_github_repository`, `list_github_branches`, and `update_github_default_branch` now go through repository details/roots that remember file permission paths.
- Database tables/migrations: none.

## Reviewer focus areas

- Whether marker persistence now handles new paths after a marker already exists.
- Whether default branch Edit calls stop being a special case by remembering the repository path before Git/GitHub lookups.
- Whether failures remain truthful when a path is missing/inaccessible.

## Fix cycle notes

Initial review request.
