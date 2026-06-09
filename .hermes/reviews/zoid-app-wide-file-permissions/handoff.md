# Handoff: Zoid app-wide file permission persistence

## Feature intent
Stop repeated macOS/Tauri file/folder permission prompts when opening the Finder sidebar in Agents section 2. Permission should behave as app-wide once the app has successfully touched a broad root, instead of re-requesting for every child folder or feature.

## Changed scope
- `src-tauri/src/lib.rs`

## Implementation notes
- Added touched-root coverage helpers:
  - `raw_path_is_covered_by_touched_root`
  - `path_is_covered_by_touched_root`
- `remember_file_permission_path` now returns immediately when a path is already under a touched root, before canonicalizing/touching that child path.
- `warm_file_permissions_inner(false)` now skips child targets under an already touched root before canonicalizing them, so a successful home-root touch acts as app-wide/root-level permission for app-owned permission checks.
- Finder/sidebar listing no longer pre-counts children of common macOS protected user folders (`Desktop`, `Documents`, `Downloads`, `Library`, `Movies`, `Music`, `Pictures`). This prevents simply opening the sidebar from touching protected folders unnecessarily; those folders are only read when the user actually opens them.

## Regression coverage added/updated
- `permission_warmup_treats_touched_home_as_app_wide_root`
- `file_manager_listing_is_lazy_and_finder_sorted` now asserts protected folders get `children_count: None` instead of eager reads.

## Real verification run
- `cargo test permission_warmup_treats_touched_home_as_app_wide_root -- --nocapture` passed.
- `cargo test file_manager_listing_is_lazy_and_finder_sorted -- --nocapture` passed.
- `cargo test` passed: 61 passed.
- `npm run test -- --run && npm run build` passed. Frontend tests passed; Rust tests passed: 61 passed; Vite build completed.

## Reviewer focus
Check that the permission root coverage does not mask direct filesystem errors or overgrant arbitrary paths, and that Finder sidebar opening no longer performs eager reads that can trigger protected-folder prompts.
