# Feature Handoff: Phase 3 P3.04 Markdown Note CRUD/Trash Service

## Original request

Continue autonomous Zoid Phase 3 tracker work:

- P3.04 Backend: Markdown note create/edit/delete/trash service.
- Keep scope backend-only and non-destructive; do not claim full scanner/Tauri bridge/frontend completion.
- Run verification and lean critique before commit.

## Implementation summary

Added a backend filesystem-backed Markdown note service over the existing Phase 3 note identity/index helpers.

Implemented service behavior:

- Create Markdown note:
  - validates note title, metadata JSON, and safe `Notes/` relative path;
  - refuses to overwrite an existing visible file;
  - creates parent directories under caller-provided visible root;
  - writes Zoid frontmatter with stable `zoid_id`, title, and slug;
  - persists note identity metadata to SQLite;
  - creates/updates the `knowledge_index_entries` frontmatter row;
  - records a `note.created` event.
- Edit Markdown note:
  - rejects trashed/deleted notes;
  - rejects frontmatter ID changes;
  - preserves the existing stable note ID;
  - rewrites the Markdown file with normalized identity frontmatter;
  - updates SQLite note metadata/index;
  - records a `note.updated` event.
- Trash Markdown note:
  - moves the file to `Notes/.Trash/<note_id>.md` if it exists;
  - sets note status to `trashed`;
  - records original relative path in metadata;
  - does not hard-delete file content;
  - records a `note.trashed` event.
- Delete Markdown note:
  - soft-deletes DB row (`status = deleted`, `deleted_at` set);
  - intentionally leaves file content in place;
  - records a `note.deleted` event.
- Read helper:
  - read service returns Markdown content from disk.

## Changed files

- `src-tauri/src/lib.rs`
  - note service records/input types;
  - create/edit/trash/delete/read helpers;
  - path resolution and note lifecycle event helpers.
- `src-tauri/src/tests.rs`
  - P3.04 tests for create/edit/index/events, trash/delete non-destructive states, and rejection cases.

## Fix cycle notes

Initial critique returned `REQUEST_CHANGES` for:

1. trash/delete leaving knowledge index rows `current`;
2. double-trash corrupting `original_relative_path`;
3. trash destination collision risk;
4. string-only path containment with symlinked parents.

Fixes applied:

- Trash/delete now mark note `knowledge_index_entries` as `scan_state = 'missing'` with lifecycle metadata.
- Double-trash is idempotent and preserves the original visible path metadata.
- Trash refuses to overwrite an existing deterministic trash destination.
- Note path resolution canonicalizes the visible root, rejects symlinked existing parents, and verifies existing parents remain inside the canonical visible root.
- Added focused regression coverage for index lifecycle state, double-trash original path preservation, trash collision, and symlink escape rejection.

## Tests run

Focused:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p304_ -- --nocapture
```

Result:

- PASS, 4 passed.

Full verification:

```bash
npm run verify:local && git diff --check
```

Result:

- PASS.
- Rust: 141 passed, 0 failed, 1 ignored guarded P2.32 real-DB harness.
- Frontend tests: PASS.
- Frontend build: PASS.
- Final marker: `PASS: local push verification passed (--skip-package)`.

## Git info

- Repo: `/Users/ziadnasreldin/Zoid`
- Branch: `main`
- Latest committed baseline before this slice: `8311bbd feat: add note identity metadata`
- Current state before review: uncommitted P3.04 changes.

## Reviewer focus areas

Please review:

1. Scope discipline: this should satisfy P3.04 backend CRUD/trash service only, not P3.05 scanner or Tauri bridge/frontend.
2. Filesystem safety: all paths must remain under caller-provided visible root and `Notes/` relative path rules; no absolute or `..` paths.
3. Non-destructive behavior: trash moves file into `Notes/.Trash`, delete is DB-soft-delete only and does not remove file content.
4. Identity preservation: edit cannot change `zoid_id`; create/edit preserve stable frontmatter identity.
5. DB/index/event consistency: notes row, knowledge index, and lifecycle events remain coherent.
6. Secret handling: metadata JSON rejects secret-like material before persistence.

## Known limitations / remaining work

- No Tauri bridge commands yet; P3.10 remains pending.
- No frontend notes workspace yet; P3.11 remains pending.
- No scanner/indexer for existing files yet; P3.05 remains pending.
- No advanced conflict handling beyond existing identity/duplicate safeguards; P3.06 remains pending.
