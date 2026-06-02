# Feature Handoff: Phase 3 Notes/Files/Knowledge P3.01–P3.02

## Original request

Continue autonomous Zoid tracker work from the active task list:

- P3.01 Planning: define Notes/Files scope; explicitly exclude Apple Notes import and full file-manager overreach.
- P3.02 Database: notes/files/index/entity-link schema and migrations.
- Run lean critique, update tracker, commit clean slice.

## Implementation summary

- Added a Phase 3 scope plan that narrows this phase to local Markdown notes, safe local file references, knowledge index metadata, and entity links.
- Explicitly excluded Apple Notes import/sync, iCloud/CloudKit, whole-home crawling, full Finder/file-manager replacement, semantic/vector search, OCR, and destructive automatic conflict resolution.
- Added SQLite migration version 9 for the Phase 3 notes/files/knowledge foundation.
- Added `notes` table with stable local path, status, conflict state, frontmatter JSON, body digest, soft-delete/trash fields, and metadata JSON constraints.
- Upgraded the existing foundation `file_references` table into a Phase 3 shape with `root_key`, safe relative paths, display metadata, kind/status/conflict state, size/fingerprint, and soft-delete fields.
- Added `knowledge_index_entries` for narrow metadata/search indexing over notes/files only.
- Preserved upgrade compatibility for old databases where `file_references` already existed and for simulated older v3 databases where the table did not exist yet.
- Added tests for migration registration, schema columns/indexes, fail-closed constraints, and task links to note/file entities through the existing entity-link service.

Known limitation: this slice is schema/planning only. It does not implement note CRUD, file browsing/opening, note scanning, frontmatter writer, conflict resolver, or UI. Those remain P3.03+.

## Changed files

- `Docs/2026-06-03-phase-3-notes-files-scope-plan.md`: new P3.01 scope plan and guardrails.
- `src-tauri/migrations/0009_phase3_notes_files_knowledge.sql`: new P3.02 migration for notes/files/knowledge index schema and upgrade-compatible `file_references` rebuild.
- `src-tauri/src/lib.rs`: registered migration version 9.
- `src-tauri/src/tests.rs`: added P3.02 RED/GREEN schema/constraint/link tests and adjusted older version/shape assertions to remain valid after later migrations.

## How to test

Run from repo root `/Users/ziadnasreldin/Zoid`:

```bash
npm run verify:local && git diff --check
```

Focused P3.02 tests:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p302_ -- --nocapture
```

Expected:

- P3.02 targeted tests pass.
- Full Rust/frontend/build verification passes.
- Migration version is at least 9.
- Existing P2/P1 schema tests still pass after `file_references` upgrade.

## Tests run

- `cargo test --manifest-path src-tauri/Cargo.toml p302_ -- --nocapture`: PASS, 3 P3.02 tests passed.
- `cargo test --manifest-path src-tauri/Cargo.toml p229_run_bridge_cancel_kills_active_process_writes_log_and_rejects_terminal_mutation -- --nocapture`: PASS after rerun; prior full-suite occurrence was timing-flaky.
- `npm run verify:local && git diff --check`: PASS.
  - Rust: 134 passed, 0 failed, 1 ignored guarded real-DB P2.32 harness.
  - Frontend tests: PASS.
  - Frontend build: PASS.
  - Final marker: `PASS: local push verification passed (--skip-package)`.

## Git info

- Branch: `main`
- Current state before review: `main...origin/main [ahead 20]` with uncommitted P3.01/P3.02 changes.
- Last committed Phase 2 closeout: `6fca42e test: verify phase 2 native closeout`.

## Frontend/backend/database notes

- Frontend routes/components: none changed.
- Backend services/commands: none added in this slice.
- Database tables/migrations:
  - `notes`
  - upgraded `file_references`
  - `knowledge_index_entries`
  - migration registration: version 9, `phase3_notes_files_knowledge`
- Existing entity-link service now has executable coverage for task ↔ note/file links.

## Reviewer focus areas

Please perform a lean backend/database review focused on:

1. SQLite migration correctness for fresh, upgrade, and older simulated DB paths.
2. Whether rebuilding `file_references` is safe and preserves legacy columns' meaningful data (`workspace_key`, `content_hash`) into new fields.
3. Fail-closed constraints for notes/files/index entries.
4. Whether this slice stays within P3.01/P3.02 and does not overbuild Apple Notes import or a full file manager.
5. Whether tests are sufficient for schema-level P3.02 without falsely claiming CRUD/UI completion.
6. Any security/path traversal risk in the schema guardrails.

## Fix cycle notes

Initial review request for P3.01–P3.02.
