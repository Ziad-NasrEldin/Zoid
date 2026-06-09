# Feature Handoff: Finder folder picker for repository scan/clone

## Original request

i want the user to be able to select a folder from the finder on their mac, not type it mnually

## Implementation summary

- Replaced manual folder path typing for repository scan and clone destination with native macOS Finder folder selection buttons.
- Added `@tauri-apps/plugin-dialog` frontend helper `selectFolderFromFinder()` using `open({ directory: true, multiple: false })`.
- Added Tauri dialog plugin initialization and default capability permission.
- Scan folder and clone destination fields are now read-only display fields populated by Finder picker results.
- Scan/clone action buttons stay disabled until the required Finder-selected folder exists in state.
- Removed hard-coded `/Users/.../Documents/GitHub` placeholders/defaults from the repository folder UI.

## Changed files

- `package.json` / `package-lock.json`: added `@tauri-apps/plugin-dialog`.
- `src-tauri/Cargo.toml` / `src-tauri/Cargo.lock`: added `tauri-plugin-dialog`.
- `src-tauri/src/lib.rs`: initialized `tauri_plugin_dialog`.
- `src-tauri/capabilities/default.json`: added `dialog:default` permission.
- `src/code/repositoryClient.ts`: added native Finder folder picker helper.
- `src/code/CodeWorkspace.tsx`: switched scan and clone destination folder fields to read-only picker-driven controls.
- `src/App.css`: added folder picker row styling.
- `src/scaffold.test.ts`: added regression checks for native folder picker usage and no manual path typing.

## How to test

1. Launch `/Applications/Zoid 25.app`.
2. Open Code workspace.
3. Under Scan folder, click `Choose folder…`; Finder folder picker should open.
4. Select a folder; the selected path should populate the read-only field.
5. Click `Scan selected folder`; repositories under the selected folder should be detected.
6. Under Clone repo, enter a GitHub URL, click `Choose destination…`, select a folder, then click `Clone repo`.

## Tests run

- `npm run test && npm run build`: PASS.
- `npm run tauri:build`: PASS.
- Reinstalled and launched `/Applications/Zoid 25.app`: PASS.
- Native screenshot `/tmp/zoid-finder-folder-picker-code.png`: PASS, Code workspace shows read-only selected-folder fields with `Choose folder…` and `Choose destination…` buttons.

## Git info

- Branch: not checked for this focused update.
- Commit SHA: not committed.
- Diff base: current dirty working tree.

## Frontend/backend/database notes

- Frontend: `CodeWorkspace` now calls `selectFolderFromFinder()` before scan/clone folder paths are set.
- Backend/native: Tauri dialog plugin registered; no database changes.
- Security: no shell interpolation introduced; selected paths still flow into existing backend validation for scan/clone.

## Reviewer focus areas

- Verify folder path entry is no longer manually typed for scan folder or clone destination.
- Verify native Tauri dialog dependency, plugin init, and permissions are correctly wired.
- Verify browser/preview failure modes remain safe; actual picker is native-only.
- Verify tests cover the new behavior enough for this slice.
