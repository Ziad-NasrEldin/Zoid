# Phase 3 Notes, Files, and Local Knowledge Scope Plan

> **For Hermes:** Use test-driven-development for implementation slices and feature-critique-workflow before marking feature work complete.

**Goal:** Build Zoid’s local Markdown notes and safe file knowledge foundation with durable indexes, stable entity IDs, history links, and restart-safe metadata.

**Architecture:** Phase 3 stays local-first. Markdown notes live under the visible Zoid directory and are indexed into SQLite with stable IDs/frontmatter metadata. Files are indexed as local file references with path fingerprints and safety metadata; Zoid records links/events but does not become a full Finder replacement.

**Tech Stack:** Tauri + Rust + SQLite migrations, existing event/entity-link/history repositories, React frontend later in the phase.

---

## Scope guardrails

### In scope for Phase 3

- Local Markdown notes created/edited/trash-managed by Zoid.
- Stable note identity via frontmatter ID and SQLite index rows.
- Local file reference/index records for visible Zoid-managed roots.
- File preview/open/browse metadata sufficient for later UI.
- Entity links connecting notes/files to tasks, products, runs, reviews, and history.
- Non-destructive conflict states for duplicate note IDs, missing files, manual renames, and external edits.
- Confirmation-policy coverage for destructive or bulk file operations later in the phase.
- Restart persistence and filesystem inspection evidence.

### Explicitly out of scope

- Apple Notes import/sync.
- iCloud/CloudKit sync.
- Full file-manager replacement for arbitrary system paths.
- Background recursive indexing of the entire home directory.
- OCR, embeddings, semantic search, or remote knowledge sync.
- Automatic destructive conflict resolution.
- Publishing/sharing/social content workflows; those belong to later phases.

## Data model direction

P3.02 should introduce schema only, not services:

- `notes`
  - stable `id`, `title`, optional `slug`, relative path, status, conflict state, frontmatter JSON, metadata JSON, timestamps.
  - path uniqueness scoped to active rows.
  - status/conflict checks that fail closed for invalid values.
- `file_references`
  - stable `id`, root key, relative path, kind, MIME/extension, byte size, fingerprint, status, conflict state, metadata JSON, timestamps.
  - no raw absolute path requirement in ordinary rows; later services resolve inside allowed roots.
- `knowledge_index_entries`
  - normalized index rows for note/file entities, title/body excerpt/search text, source modified time, scan state, metadata JSON.
  - one current index row per entity/source type.
- Existing `entity_links` and `events` remain the relationship/history backbone.
  - P3.02 should prove note/file entity types can be linked to existing task/run entities without adding special link tables.

## P3.02 acceptance criteria

1. Migration version advances from 8 to 9.
2. Fresh migrated DB has `notes`, `file_references`, and `knowledge_index_entries` tables.
3. Notes table validates status/conflict state and JSON metadata/frontmatter.
4. File references table validates root/kind/status/conflict and JSON metadata.
5. Knowledge index rows validate entity/source/scan state and JSON metadata.
6. Useful indexes exist for status/path/entity/source lookups.
7. Existing event/entity-link services can link `note` and `file` entities to a task and list them back.
8. Existing migration upgrade path remains idempotent and foreign keys stay enabled.
9. No Apple Notes import, whole-home crawler, or full file-manager assumptions are introduced.

## Implementation order

### Task 1 — RED schema test

Add `p302_schema_version_nine_has_notes_files_and_knowledge_index_tables` in `src-tauri/src/tests.rs`.

Expected RED command:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p302_schema_version_nine_has_notes_files_and_knowledge_index_tables -- --nocapture
```

Expected failure before migration: missing table/columns or migration version < 9.

### Task 2 — GREEN migration

Create `src-tauri/migrations/0009_phase3_notes_files_knowledge.sql` and add it to `MIGRATIONS`.

Run the same test; expected pass.

### Task 3 — RED/GREEN validation tests

Add tests that invalid note/file/index statuses and invalid JSON fail closed, then rely on table constraints to pass.

### Task 4 — RED/GREEN entity link compatibility test

Add a test that creates a task, creates note/file rows, links them with existing entity-link service, and lists them back directionally.

### Task 5 — verification/review

Run:

```bash
npm run verify:local && git diff --check
```

Write `.hermes/reviews/p3-01-p3-02-notes-files-schema/handoff.md`, run lean critique, fix Required issues, then commit.
