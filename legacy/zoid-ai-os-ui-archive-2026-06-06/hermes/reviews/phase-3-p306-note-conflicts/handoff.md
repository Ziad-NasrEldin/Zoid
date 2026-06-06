# Feature Handoff: Phase 3 P3.06 Note Conflict Handling

## Original request

Continue autonomous Zoid Phase 3 tracker work with autopilot always on.

Tracker item:

- P3.06 Backend: conflict handling for duplicate IDs, manual renames, and external edits; non-destructive defaults.

## Scope delivered

Backend-only conflict handling for existing Phase 3 Markdown note scanner/indexer:

- manual rename detection when a scanned file has an existing `zoid_id` at a new path and the previous path is missing;
- external edit detection when a scanned file keeps the same path/id but the body digest changed from the stored DB digest;
- conflict listing helper for notes in conflicted states;
- conflict accept helper for manual rename and external edit conflicts;
- duplicate ID conflicts remain non-destructive and require manual file/frontmatter edit before acceptance.

No frontend, Tauri command bridge, destructive auto-resolution, OCR, embeddings, Apple Notes/iCloud import, or broad file-manager behavior was added.

## Changed files

- `src-tauri/src/lib.rs`
  - added `NoteConflictRecord`.
  - scanner now distinguishes duplicate ID vs manual rename vs external edit.
  - added `list_note_conflicts_service`.
  - added `accept_note_conflict_service`.
  - added metadata-backed `mark_note_conflict` and path-update persistence helper for accepted manual renames.
  - duplicate conflicts still refuse automatic acceptance.
- `src-tauri/src/tests.rs`
  - added P3.06 tests for manual rename conflict detection/acceptance and external edit conflict detection/acceptance.

## Behavior details

Manual rename:

1. Existing note row has `zoid_id` stored at `Notes/old.md`.
2. Disk file moves externally to `Notes/archive/new.md` while preserving frontmatter `zoid_id`.
3. Scanner finds same ID at new path.
4. If old path no longer exists, scanner marks existing row `status='conflicted'`, `conflict_state='manual_rename'` and records `detected_relative_path` in metadata.
5. It does not mutate the stored original path until the conflict is explicitly accepted.
6. Accepting the conflict updates the row/index to the detected path and restores `status='active'`, `conflict_state='none'`.

External edit:

1. Existing note row and disk file keep same path/id.
2. Disk body changes outside the service.
3. Scanner compares stored digest vs disk-derived digest.
4. It marks `external_edit` and records stored/disk digests without overwriting DB digest immediately.
5. Accepting the conflict updates DB/index digest/content metadata and returns the externally edited Markdown.

Duplicate ID:

- If the previous indexed path still exists, a second file with the same ID is treated as `duplicate_id`.
- The duplicate file is not rewritten or deleted.
- Accept helper rejects `duplicate_id` with a constraint because it requires manual file/frontmatter edit.

## Fix cycle notes

Initial critique returned `REQUEST_CHANGES` because conflict marking and acceptance overwrote unrelated note `metadata_json`, violating non-destructive defaults.

Fixes applied:

- Conflict bookkeeping now merges into a nested `_zoid_conflict` object inside existing note metadata instead of replacing the whole JSON document.
- Conflict listing reads `_zoid_conflict` while returning the original metadata JSON.
- Conflict acceptance removes only `_zoid_conflict`, preserving unrelated metadata keys.
- Added tests proving manual rename and external edit metadata survives mark + accept.
- Added explicit duplicate-ID acceptance rejection test proving files/metadata remain unchanged.

## Tests run

RED:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p306_ -- --nocapture
```

Result:

- Failed as expected because `list_note_conflicts_service` and `accept_note_conflict_service` did not exist.
- Required-fix RED later failed on metadata preservation for manual rename/external edit before fixes.

Focused GREEN:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p306_ -- --nocapture
```

Result:

- PASS, 3 passed.

Full verification:

```bash
npm run verify:local && git diff --check
```

Result:

- PASS.
- Rust: 148 passed, 0 failed, 1 ignored guarded P2.32 real-DB harness.
- Frontend tests: PASS.
- Frontend build: PASS.
- Final marker: `PASS: local push verification passed (--skip-package)`.

## Review focus requested

Please verify:

- conflict classifications are correct and non-destructive;
- manual rename does not mutate stored path before acceptance;
- external edit does not overwrite stored digest before acceptance;
- accept helper is safe for manual rename and external edit;
- duplicate IDs still require manual intervention;
- scope stays backend-only and aligned with P3.06 tracker wording.
