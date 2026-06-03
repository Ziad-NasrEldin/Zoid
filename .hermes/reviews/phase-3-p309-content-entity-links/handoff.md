# P3.09 Content Entity Links Handoff

## Scope
Backend-only P3.09: entity links from notes/files to tasks/products/runs.

## Original tracker item
- `P3.09 Backend: entity links from notes/files to tasks/products/runs.`

## Changed files
- `src-tauri/src/lib.rs`
- `src-tauri/src/tests.rs`

## Implementation summary
- Added backend content-link request/list filter types:
  - `ContentEntityLinkCreateRequest`
  - `ContentEntityLinkListFilter`
- Added `create_content_entity_link_service` wrapper over the existing approved generic entity-link service.
- Added deterministic link IDs derived from `(source_type, source_id, target_type, target_id, relation_type)` so repeated logical requests are idempotent through the existing entity-link service.
- Restricted P3.09 content-link direction:
  - source types: `note`, `file`
  - target types: `task`, `product`, `run`
- Added create-time existence/linkability validation:
  - notes must exist and be `active` or `draft` at create time;
  - files must exist in `file_references` and be `indexed` at create time;
  - tasks must exist and not be deleted;
  - runs must exist via existing `agent_runs` lookup;
  - products are accepted as opaque future IDs while no `products` table exists; if a future `products` table exists, the helper checks row existence.
- Added source/target list wrappers:
  - `list_content_entity_links_by_source`
  - `list_content_entity_links_by_target`
- Split create-time linkability from query-time row existence after lean review:
  - source listing now requires only that note/file source row exists, not that it remains active/indexed;
  - this keeps existing links queryable after later note/file lifecycle changes such as trash/missing/stale states.
- Metadata validation/redaction remains delegated to the existing generic entity-link service.

## Tests added
- `p309_note_links_to_tasks_products_and_runs_with_directional_queries`
  - note -> task/product/run links;
  - idempotent duplicate logical link;
  - deterministic source and target listing.
- `p309_file_links_to_tasks_products_and_runs_after_file_reference_exists`
  - file -> task/product/run links only after a `file_references` row exists.
- `p309_content_links_reject_invalid_direction_missing_entities_and_secret_metadata`
  - rejects task -> note direction;
  - rejects missing note/file/task;
  - verifies secret-like metadata is redacted by the generic entity-link service.
- `p309_content_link_source_queries_survive_later_note_and_file_state_changes`
  - regression for lean-review blocker;
  - existing note/file links remain queryable after source rows are marked `trashed`.

## Verification
Passed:
- `cargo test --manifest-path src-tauri/Cargo.toml p309 -- --nocapture`
  - result: 4 passed, 0 failed.
- `npm run verify:local && git diff --check`
  - Rust suite: 161 passed, 1 ignored.
  - Frontend tests: passed.
  - Frontend build: passed.
  - `git diff --check`: passed.

## Review notes
- Lean backend/database review found one REQUIRED blocker: source-side list wrapper reused create-time linkability checks, making existing links unqueryable after note/file source status changed.
- Fixed by introducing `ensure_content_source_row_exists` and using it only for source-side listing.
- Lean re-review confirmed the blocker is resolved and found no remaining REQUIRED blockers.

## Product target note
There is currently no `products` table in the repo. P3.09 therefore treats product IDs as opaque future IDs, but the helper is future-aware: if a `products` table appears later, it validates target row existence.

## No frontend or bridge scope
No Tauri command bridge or frontend UI was changed; those are later tracker items.
