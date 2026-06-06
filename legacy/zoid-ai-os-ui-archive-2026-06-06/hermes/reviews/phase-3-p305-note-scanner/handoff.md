# Feature Handoff: Phase 3 P3.05 Markdown Note Scanner/Indexer

## Original request

Continue autonomous Zoid Phase 3 tracker work with autopilot always on.

Tracker item:

- P3.05 Backend: frontmatter stable ID writer/reader and scanner/indexer.

## Implementation summary

Added a backend-only Markdown note scanner/indexer that builds on the existing P3.03 stable frontmatter reader/writer and P3.04 note CRUD path safety.

Implemented behavior:

- Scans Markdown files recursively under `Notes/` from the caller-provided visible root.
- Skips `Notes/.Trash/` and non-Markdown files.
- Skips symlink entries during scan traversal.
- Sorts scan paths for deterministic indexing.
- Uses `derive_note_identity_from_markdown` to read existing `zoid_id`, title, slug, relative path, and digest.
- Writes missing Zoid frontmatter back to discovered Markdown files using `write_note_identity_frontmatter`.
- Upserts discovered notes into SQLite `notes` and `knowledge_index_entries` through `upsert_note_identity_metadata`.
- Preserves existing frontmatter IDs.
- Detects duplicate `zoid_id` conflicts non-destructively:
  - first deterministic path remains the indexed note row;
  - existing note is marked `duplicate_id` through the existing duplicate guard;
  - duplicate file is not rewritten or deleted;
  - scan result increments `conflicted_notes` and continues.
- Marks previously indexed active/conflicted notes as missing when their files disappear:
  - `notes.status = 'conflicted'`;
  - `notes.conflict_state = 'path_missing'`;
  - matching knowledge index rows get `scan_state = 'missing'` with lifecycle metadata.

## Changed files

- `src-tauri/src/lib.rs`
  - added `NoteScanResult`;
  - added `scan_markdown_notes_service`;
  - added recursive Markdown collection helpers;
  - added relative-path rendering and note-directory containment helper;
  - added missing-file lifecycle marking.
- `src-tauri/src/tests.rs`
  - added P3.05 scanner/indexer tests for missing frontmatter writes, indexing, `.Trash`/non-Markdown skipping, missing-file marking, duplicate ID conflict handling, and non-destructive duplicate behavior.

## Fix cycle notes

Initial critique returned `REQUEST_CHANGES` for:

1. existing YAML frontmatter without `zoid_id` being indexed without persisted identity;
2. scanner DB/index upsert happening before direct file writes, risking incoherence on write failure.

Fixes applied:

- Scanner now detects missing required Zoid identity scalars (`zoid_id`, `title`, `slug`) instead of only missing YAML blocks.
- Existing YAML frontmatter without `zoid_id` is rewritten while preserving unrelated custom keys and body content.
- Scanner frontmatter writes use temp-file + rename through `write_note_frontmatter_atomically`.
- Scanner writes identity frontmatter before DB/index upsert for files needing persistence, and duplicate-ID paths are checked before rewrite so duplicate files remain non-destructive.
- Added regression coverage for existing YAML without `zoid_id` and for write failure not leaving active/current DB/index rows.

## Tests run

RED:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p305_ -- --nocapture
```

Result:

- Failed as expected because `scan_markdown_notes_service` did not exist.
- Required-fix RED later failed on existing-YAML-without-`zoid_id` and write-failure coherence before fixes.

Focused GREEN:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p305_ -- --nocapture
```

Result:

- PASS, 4 passed.

Full verification:

```bash
npm run verify:local && git diff --check
```

Result:

- PASS.
- Rust: 145 passed, 0 failed, 1 ignored guarded P2.32 real-DB harness.
- Frontend tests: PASS.
- Frontend build: PASS.
- Final marker: `PASS: local push verification passed (--skip-package)`.

## Git info

- Repo: `/Users/ziadnasreldin/Zoid`
- Branch: `main`
- Baseline commit before this slice: `4ba8117 feat: add markdown note crud service`
- Current state before review: uncommitted P3.05 changes.

## Reviewer focus areas

Please review:

1. Scope discipline: backend scanner/indexer only; no Tauri bridge/frontend/file-manager overreach.
2. Filesystem safety: scanner stays under `Notes/`, skips `.Trash`, skips symlink entries, and uses existing path validation/resolution for file reads/writes.
3. Non-destructive defaults: duplicate files are not rewritten/deleted; missing files only mark DB/index lifecycle state.
4. Identity stability: existing `zoid_id` is preserved; missing frontmatter gets stable deterministic IDs.
5. DB/index coherence: discovered notes are active/current; missing files are conflicted/missing; duplicate IDs are flagged.
6. Test sufficiency for RED/GREEN and regression behavior.

## Known limitations / remaining work

- P3.06 still owns deeper conflict handling for manual renames and external edits.
- P3.07/P3.08 still own generic file browse/open/preview and safe file actions.
- P3.10 still owns Tauri bridge commands.
- P3.11+ frontend notes workspace remains pending.
