# P3.04 Markdown Note CRUD/Trash Backend Re-review

Verdict: APPROVED

## Scope and verification

Re-reviewed the P3.04 backend-only slice after the requested fix cycle, focusing on the four previously blocking issues in:

- `src-tauri/src/lib.rs`
- `src-tauri/src/tests.rs`
- `.hermes/reviews/phase-3-p304-note-crud/handoff.md`

Verification run:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p304_ -- --nocapture
```

Result: PASS, 4 passed.

Full local verification run:

```bash
npm run verify:local && git diff --check
```

Result: PASS. Rust tests reported 141 passed, 0 failed, 1 ignored; frontend tests/build passed; `git diff --check` passed.

## Required-fix review

### 1. Trash/delete index lifecycle state

Satisfied. `trash_markdown_note_service` and `delete_markdown_note_service` now call `mark_note_index_lifecycle(...)`, setting matching note `knowledge_index_entries` rows to `scan_state = 'missing'` with lifecycle metadata (`trashed` or `deleted`). Regression assertions cover both trash and delete states.

### 2. Double-trash original path preservation

Satisfied. Already-trashed notes are handled idempotently before recomputing trash metadata, and `merge_note_original_path_metadata(...)` preserves an existing `original_relative_path` rather than overwriting it. Regression coverage verifies repeat trash does not replace the original visible path with `Notes/.Trash/...`.

### 3. Trash destination overwrite risk

Satisfied. Trash now checks the deterministic `Notes/.Trash/<note_id>.md` destination before rename and returns a `RepositoryError::Constraint` if a different recoverable trash artifact already exists. Regression coverage verifies the existing trash file content is preserved and the source note remains in place.

### 4. Symlink-parent path escape

Satisfied. `resolve_note_service_path(...)` canonicalizes the visible root, rejects existing symlink parents in note paths, and verifies existing parents canonicalize under the visible root. Regression coverage verifies a `Notes/` child symlink to an outside directory is rejected and no escaped file is created.

## Non-blocking observations

- `read_note_service` still returns empty Markdown on file-read failure via `unwrap_or_default()`. That remains acceptable for this approval because it was non-blocking in the prior review and is not part of the required fix cycle.
- Handoff wording was reconciled after re-review: P3.04 does not add a v10 migration, does not implement a list helper, and intentionally uses the existing Phase 3 notes/index/event schema.

## Conclusion

All four previously required changes are satisfied with focused regression coverage and passing full local verification. No remaining blockers for P3.04 backend note CRUD/trash approval.
