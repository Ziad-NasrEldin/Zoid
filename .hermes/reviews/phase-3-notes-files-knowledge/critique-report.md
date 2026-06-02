# Critique Report: Phase 3 Notes/Files/Knowledge P3.01–P3.02

Verdict: APPROVED

## Scope reviewed

Reviewed the P3.01 scope plan and P3.02 backend/database schema slice:

- `Docs/2026-06-03-phase-3-notes-files-scope-plan.md`
- `src-tauri/migrations/0009_phase3_notes_files_knowledge.sql`
- `src-tauri/src/lib.rs`
- `src-tauri/src/tests.rs`

This was a review-only pass; no source files were modified.

## Verification performed

Commands run from `/Users/ziadnasreldin/Zoid`:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p302_ -- --nocapture
```

Result: PASS — 3 P3.02 tests passed.

```bash
npm run verify:local
```

Result: PASS — Rust suite reported 134 passed, 0 failed, 1 ignored; frontend tests and frontend build passed; final marker `PASS: local push verification passed (--skip-package)`.

```bash
git diff --check
```

Result: PASS — no whitespace errors reported.

## Findings

### Migration/versioning

- Migration version 9 is registered in `MIGRATIONS` as `phase3_notes_files_knowledge` and points to `0009_phase3_notes_files_knowledge.sql`.
- Fresh migration path creates the expected `notes`, upgraded `file_references`, and `knowledge_index_entries` tables.
- The migration includes a compatibility `create table if not exists file_references` step before rebuilding into the P3 shape, which supports older/simulated DBs where the foundation table is absent.
- Existing `workspace_key` and `content_hash` are meaningfully carried forward into `root_key` and `content_fingerprint` respectively.

### `file_references` rebuild compatibility

- The rebuild pattern preserves legacy rows into the new P3 table shape and then renames the rebuilt table.
- Legacy `workspace_key` values in the known visible-root set are preserved; unknown/null values are safely normalized to `zoid_visible`.
- Existing `content_hash` is preserved as `content_fingerprint`.
- No foreign keys appear to reference `file_references`, so the drop/rename rebuild does not appear to break dependent constraints.

### Constraints and guardrails

- `notes`, `file_references`, and `knowledge_index_entries` use fail-closed enum checks for statuses/kinds/source types/scan states.
- JSON fields are guarded with `json_valid(...)` checks.
- Path guardrails prevent empty, oversized, absolute, and `..`-containing relative paths for notes/files. This is intentionally conservative and appropriate for a schema foundation; later filesystem services should still canonicalize resolved paths inside allowed roots.
- `knowledge_index_entries` limits indexing to `note` and `file` entity types and constrains note/file source-type compatibility.

### Entity-link compatibility

- P3.02 tests show note and file entities can link to existing tasks through the existing entity-link service without adding special-purpose link tables.
- This preserves the existing event/entity-link/history architecture.

### Scope control

- The scope plan explicitly excludes Apple Notes import/sync, iCloud/CloudKit, whole-home crawling, full file-manager replacement, OCR, embeddings/semantic search, remote sync, and destructive automatic conflict resolution.
- The implementation stays within schema/planning. It does not add note CRUD, file-manager UI, Apple Notes import, background crawlers, or overbuilt knowledge services.

## Non-blocking notes

- The schema-level path checks are conservative (`not like '%..%'`) and may reject benign filenames containing `..`; acceptable for this foundation slice, but later service-layer path validation should be more precise and canonicalization-based.
- Migration 0009 is not written with an explicit transaction wrapper. The existing migration runner applies SQL via `execute_batch`; this matches current project style, but future rebuild-style migrations may benefit from explicit transactional handling if feasible.
- `knowledge_index_entries.entity_id` intentionally has no FK to `notes`/`file_references`; this keeps the index generic but means service-layer cleanup/upsert discipline will matter in later slices.

## Conclusion

The P3.01–P3.02 slice satisfies the requested lean backend/database scope. Migration registration, fresh/upgrade compatibility, `file_references` data preservation, fail-closed constraints, entity-link compatibility, and scope boundaries all look acceptable for this schema-only foundation.
