# P3.10 Tauri Notes/Files Bridge Handoff

## Scope
Backend/Tauri bridge slice for tracker P3.10: commands for notes CRUD, scan/index, conflicts, file browse/open/preview/actions.

## Changed files
- `src-tauri/src/lib.rs`
- `src-tauri/src/tests.rs`

## Implementation summary
- Added Tauri command request structs for:
  - `NoteCommandCreateRequest`
  - `NoteCommandEditRequest`
  - `NoteCommandListRequest`
  - `FileBrowseCommandRequest`
  - `FileReferenceCommandRequest`
  - `FileActionCommandRequest`
- Added registered command names and `generate_handler!` entries for 13 new bridge commands:
  - `create_markdown_note_command`
  - `read_note_command`
  - `list_notes_command`
  - `edit_markdown_note_command`
  - `trash_markdown_note_command`
  - `delete_markdown_note_command`
  - `scan_markdown_notes_command`
  - `list_note_conflicts_command`
  - `accept_note_conflict_command`
  - `browse_files_command`
  - `open_file_reference_command`
  - `preview_file_command`
  - `perform_file_action_command`
- Added with-connection helpers for tests and bridge wrappers.
- Added `open_ready_connection_and_visible_root()` so bridge commands consistently use the initialized DB and visible root.
- File action bridge loads persisted confirmation decisions by `confirmation_id`; no frontend-only boolean confirmation is accepted.
- Unsupported file action strings fail closed.
- Note read/list markdown paths now propagate missing/unreadable markdown file errors instead of silently returning empty markdown.

## Test coverage
Added P3.10 tests for:
- command surface registration and generate handler coverage;
- notes create/read/list/edit/scan/trash/delete/conflict commands;
- missing markdown truthfulness for read/list include-markdown paths;
- file browse/open/preview/action commands;
- file actions blocked without persisted confirmation and allowed with persisted confirmation;
- unsupported file actions fail closed.

## Verification
Passed:
- `cargo test --manifest-path src-tauri/Cargo.toml p310 -- --nocapture`
  - 4 passed, 0 failed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml && npm run verify:local && git diff --check`
  - Rust: 165 passed, 0 failed, 1 ignored.
  - Frontend tests: passed.
  - Frontend build: passed.
  - `git diff --check`: passed.

## Lean review status
- First lean review found one REQUIRED blocker: note read/list bridge silently returned empty markdown on missing/unreadable files.
- Fixed by propagating IO errors and adding `p310_note_bridge_reports_missing_markdown_files_truthfully`.
- Re-review found no remaining REQUIRED blockers.

## Notes for critique
Focus on whether bridge-exposed commands truthfully preserve the underlying service guarantees: filesystem safety, persisted confirmations for file actions, no silent fake note content, registration coverage, and validation/error propagation.
