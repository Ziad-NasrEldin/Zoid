# Feature Handoff: Phase 3 P3.08 safe file actions

## Original request

Continue P3.08 from the prior handoff:

- Finish P3.08 preflight.
- Add RED tests for copy/rename/move/trash blocked without confirmation, approved confirmation allows safe action, destination collision rejected, symlink/path escape rejected, trash non-destructive and updates file/index states.
- Implement helpers.
- Run `npm run verify:local && git diff --check`.
- Write this handoff, run critique loop until `APPROVED`, update tracker, and commit.

## Implementation summary

- Added backend-only safe file action helpers for visible-root local files:
  - `FileActionInput` / `FileActionKind` / `FileActionRecord`.
  - `perform_file_action_service` for copy, rename, move, and trash.
- All actions run through existing action-policy clearance before filesystem writes:
  - copy/rename/move use `move_rename_copy_file` via `ActionType::File` + `LocalVisible` + `LocalWrite`.
  - trash uses `delete_trash_files` via `ActionType::Delete` + destructive consequence.
  - missing confirmation fails closed before filesystem/DB/index changes.
- Reused P3.07 visible-root path validation and symlink-component rejection for sources.
- Added safe destination resolution for new paths:
  - validates relative paths;
  - rejects parent symlink components;
  - canonicalizes parent under visible root;
  - rejects existing destinations with `fs::symlink_metadata`, including final broken symlinks;
  - rejects collisions before writes.
- Implemented non-destructive trash by moving files under `Trash/<original-parent>/...` with collision-safe suffixes.
- Updates `file_references` and `knowledge_index_entries` only after successful filesystem actions:
  - copy indexes destination and leaves source current;
  - move/rename mark source file reference `missing`, old preview index `stale`, then index destination;
  - trash marks source file reference `trashed`, old preview index `stale`, then indexes trash destination.
- Destination preview extraction failures are non-fatal after a safe filesystem action: invalid/binary/unreadable preview bytes still leave a valid destination `file_references` row and skip the preview index rather than returning a partial-action error.
- No permanent delete and no OS launch were added.

## Changed files

- `src-tauri/src/lib.rs`
  - Added file action request/record types.
  - Added safe copy/rename/move/trash service and destination safety helpers.
  - Added file reference/index status helpers and reusable file reference entity ID helper.
  - Changed destination action indexing to skip non-previewable preview bytes instead of failing after filesystem mutation.
- `src-tauri/src/tests.rs`
  - Added P3.08 RED/GREEN tests covering confirmation blocking, approved actions, collision rejection, path escape, symlinked destination parents, final broken destination symlinks, invalid preview bytes after safe copy, and non-destructive trash/index state.

## How to test

- Focused tests:
  - `cargo test --manifest-path src-tauri/Cargo.toml p308 -- --nocapture`
  - Expected: 5 P3.08 tests pass.
- Full local verification:
  - `npm run verify:local && git diff --check`
  - Expected: Rust, frontend tests, build, and diff whitespace check pass.

## Tests run

- RED before implementation:
  - `cargo test p308_file_actions -- --nocapture`: failed because `FileActionInput`, `perform_file_action_service`, and `file_reference_entity_id` did not exist yet.
- GREEN focused:
  - `cargo test p308 -- --nocapture`: PASS, 4 passed.
- Lean reviewer required fix #1:
  - Added final broken symlink destination test.
  - RED: `cargo test p308_file_actions_reject_path_escape_and_symlink_components -- --nocapture` failed because copy followed a broken final symlink and created the outside file.
  - GREEN: `cargo test p308_file_actions_reject_path_escape_and_symlink_components -- --nocapture`: PASS, 1 passed.
- Critique-agent required fix #2:
  - Added invalid preview bytes regression test for approved copy.
  - RED: `cargo test p308_file_actions_invalid_preview_bytes_do_not_turn_safe_copy_into_partial_error -- --nocapture` failed with `file is not previewable: invalid utf-8` after filesystem copy.
  - GREEN: same command PASS, 1 passed, after making destination preview extraction non-fatal.
- Full verification after final fix:
  - `cargo test --manifest-path src-tauri/Cargo.toml p308 -- --nocapture`: PASS, 5 passed.
  - `npm run verify:local && git diff --check`: PASS.
  - Rust: 157 passed, 0 failed, 1 ignored guarded P2.32 harness.
  - Frontend tests: PASS.
  - Frontend build: PASS.
  - `git diff --check`: PASS.

## Git info

- Repo: `/Users/ziadnasreldin/Zoid`
- Branch: `main`
- Latest commit before this slice: `f19966c feat: add file browse preview service`
- Commit SHA for P3.08: pending final critique approval, tracker update, and commit.

## Frontend/backend/database notes

- Frontend routes/components: not touched.
- Tauri bridge commands: not touched; P3.10 will expose commands for file actions.
- Backend services: new helper functions are backend-only and currently exercised through Rust tests.
- Database:
  - No migration added.
  - Existing `file_references` and `knowledge_index_entries` tables are updated after successful safe filesystem actions.

## Reviewer focus areas

- Confirm final destination symlink/broken-symlink rejection is sufficient before writes.
- Confirm policy clearance cannot be bypassed for copy/rename/move/trash.
- Confirm blocked/failed actions do not mutate filesystem or index state.
- Confirm trash remains non-destructive and does not permanently delete.
- Confirm DB/index status updates happen only after safe filesystem success.
- Confirm invalid preview bytes cannot make an already-successful safe action return an error with partial side effects.

## Fix cycle notes

- Lean combined reviewer initially returned `REQUEST_CHANGES` because final destination broken symlinks were not rejected.
- Fixed destination collision logic to use `fs::symlink_metadata(&destination)` so any existing final entry is rejected, including broken symlinks; lean re-review returned `APPROVED`.
- Critique-agent then returned `REQUEST_CHANGES` for post-filesystem indexing failures causing partial-action errors on invalid preview bytes.
- Fixed destination preview extraction to be non-fatal and added regression coverage; final critique re-review pending.
