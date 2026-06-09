# Feature Handoff: Zoid Hermes Finder file manager sidebar

## Original request

Page Feedback for `/` at `tauri://localhost`: Add a button beside the Link repository section in the Hermes topbar. The button opens a sidebar on the right side of the chat window, beside the main chat area. The sidebar should act as a file manager on macOS, showing the Finder-style folder structure that Zoid can access, with basic folder minimize/maximize behavior to see nested folder contents.

## Implementation summary

- Added a topbar `Files` button beside the repository linker.
- Added a right-side Finder sidebar inside the Hermes chat workspace.
- Added lazy folder expansion/collapse: folders fetch children on first expand and can be minimized/maximized afterward.
- Added native Tauri/Rust directory listing command that defaults to the macOS home folder, allows moving upward, skips hidden dotfiles, sorts directories first like Finder, and returns file/folder metadata.
- Added frontend client types/invoke helper and scaffold/Rust regression coverage.
- Known limitation: this is a browser-safe Finder-style tree and metadata browser; it does not yet implement file open/rename/delete/copy/drag/drop or Finder icon previews.

## Changed files

- `src/agents/AgentsHermesScreen.tsx`: topbar button, sidebar state, tree rendering, lazy folder expansion.
- `src/agents/hermesClient.ts`: file-manager response types and Tauri invoke helper.
- `src-tauri/src/lib.rs`: native directory listing models, helper, Tauri command registration, tests.
- `src/App.css`: topbar button and right sidebar/tree styling.
- `src/scaffold.test.ts`: regression checks for the new UI/bridge strings.

## How to test

- `npm run build`
- `npm run test:frontend`
- `cargo test` from `src-tauri/` or `npm run test:rust`
- Launch/relaunch `/Applications/Zoid 25.app`, open Hermes, click `Files`, verify the right sidebar appears, shows the home folder, and folders expand/collapse to nested contents.

## Tests run

- `npm run build`: PASS. Vite build completed; only standard large-chunk warning.
- `npm run test:frontend`: PASS. Scaffold, GlobalDropdown behavior, and ChatComposer slash tests passed.
- `cargo test` in `src-tauri/`: PASS. 56 Rust tests passed; existing dead-code warnings for profile helpers remain.

## Git info

- Branch: `main`
- Commit SHA, if committed: not committed
- Diff base: working tree, already contains many unrelated dirty/untracked Zoid changes; review should focus on the files listed above and the file-manager diff only.

## Frontend/backend/database notes

- Frontend: `AgentsHermesScreen` renders `file-manager-toggle-button` and `file-manager-sidebar` within the Hermes chat workspace.
- Backend/native: Tauri command `list_file_manager_directory(path: Option<String>)` lists real filesystem folders via Rust `fs::read_dir`.
- Database: not applicable.

## Reviewer focus areas

- Spec fit: topbar placement beside Link repository; right-side chat sidebar; Finder-like expandable folder tree.
- Native filesystem truthfulness: command reads real directories, does not fabricate entries, handles unreadable folders gracefully.
- UX/layout: right sidebar does not replace the chat and remains scrollable.
- Tests: Rust listing behavior and frontend scaffold coverage.

## Fix cycle notes

Initial critique returned REQUEST_CHANGES for: Up navigation not updating root, missing responsive single-column layout, and missing behavioral frontend coverage.

Fixes made:
- Updated `loadFileManagerPath` with an explicit `{ makeRoot: true }` option for Up/Refresh root navigation while keeping nested folder expansion lazy.
- Added single-column responsive CSS so the file manager moves into row 3 below chat on narrow layouts instead of implicit grid column 3.
- Added `src/agents/AgentsHermesScreen.file-manager.test.tsx` with mocked Tauri IPC covering open, initial root rendering, folder expand/collapse, Up root navigation, and the narrow-layout CSS guard.
- Added that test to `npm run test:frontend`.

Latest checks:
- `npm run test:frontend`: PASS.
- `npm run build`: PASS; standard Vite chunk-size warning only.
- `npm run test:rust`: PASS; 60 tests passed with existing dead-code warnings.
