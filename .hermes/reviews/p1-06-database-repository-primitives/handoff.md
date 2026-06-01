# Feature Handoff: P1.06 Database repository primitives

## Original request

Phase 1 / Database: Add repository/helper primitives for insert/read/list/update patterns with typed errors and tests.

Current user instruction: continue phase-by-phase as orchestrator and delegate implementation/review tasks to subagents.

## Implementation summary

- Added private/internal SQLite repository helper primitives for Phase 1 foundation database tables.
- Added typed repository error classification for:
  - `NotFound`
  - `Constraint`
  - `InvalidJson`
  - `Database`
- Added app settings primitives:
  - `upsert_app_setting`
  - `read_app_setting`
  - `list_app_settings`
  - `list_app_settings_by_scope`
  - `update_app_setting`
- Added entity link primitives:
  - `insert_entity_link`
  - `insert_or_get_entity_link`
  - `read_entity_link`
  - `read_entity_link_by_unique`
  - `list_entity_links_for_source`
- Added JSON validation before writing JSON-backed columns.
- Kept the primitives private/internal for now; P1.16 or later service tasks can expose/use them.
- Fixed review-required edge case: `insert_or_get_entity_link` now only treats a constraint as idempotent when the logical unique tuple exists; primary-key id collisions or other non-idempotent constraints preserve `RepositoryError::Constraint`.

## Changed files

- `src-tauri/src/lib.rs`: repository error types, app_settings/entity_links primitives, and tests.

## How to test

From `/Users/ziadnasreldin/Zoid`:

- `cargo test --manifest-path src-tauri/Cargo.toml repository -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml insert_or_get_entity_link_preserves_constraint_for_id_collision -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `npm run verify:local`
- `npm run verify:release` at stop/reporting checkpoints per current user request.

Expected behavior:

- App settings can be upserted, read, listed globally, listed by scope, and updated.
- Missing app setting update returns typed `RepositoryError::NotFound`.
- Invalid JSON is rejected before SQLite write as typed `RepositoryError::InvalidJson`.
- SQLite check/unique constraint failures are typed as `RepositoryError::Constraint`.
- Entity links can be inserted, read, listed by source, and idempotently returned for logical duplicate tuples.
- Entity link primary-key id collision with a different logical tuple remains a typed `RepositoryError::Constraint`, not `NotFound`.

## Tests run

Implementation subagent TDD evidence:

- RED for initial repository primitives:
  - `cargo test --manifest-path src-tauri/Cargo.toml repository -- --nocapture`
  - Failed before implementation with missing types/functions: `AppSettingInput`, `upsert_app_setting`, `read_app_setting`, `list_app_settings_by_scope`, `update_app_setting`, `EntityLinkInput`, `insert_entity_link`, `RepositoryError`, etc.
- GREEN focused for initial repository primitives:
  - `cargo test --manifest-path src-tauri/Cargo.toml repository -- --nocapture`: PASS, 3 passed.
- Full Rust after initial implementation:
  - `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 27 passed.
- Clippy after initial implementation:
  - `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`: PASS.

Review fix TDD evidence:

- RED:
  - Added `insert_or_get_entity_link_preserves_constraint_for_id_collision`; focused run failed before fix because the returned error was not `RepositoryError::Constraint`.
- GREEN:
  - Focused regression test: PASS.
  - Full Rust tests: PASS, 28 passed.
  - Clippy: PASS with `-D warnings`.

Parent/orchestrator verification:

- `cargo test --manifest-path src-tauri/Cargo.toml repository -- --nocapture`: PASS, 3 passed.
- `npm run verify:local`: PASS.
  - Rust tests: 28 passed, 0 failed.
  - Frontend build: PASS.

Independent reviews:

- Spec review after initial implementation: PASS.
- Quality review after initial implementation: REQUEST_CHANGES for entity-link primary-key collision misclassification.
- Spec re-review after fix: PASS.
- Quality re-review after fix: APPROVED.

## Git info

- Branch: main
- Commits:
  - `f327ba8 Add database repository primitives`
  - `f9cb79a fix: preserve entity link constraint collisions`
- Diff base: `4f8c81f Record P1.05 database schema review`

## Frontend/backend/database notes

- Frontend routes/components: no frontend code changed.
- Backend/native: private Rust repository primitives added.
- Database: no new schema migration; uses existing P1.05 `app_settings` and `entity_links` tables.
- Security/privacy:
  - No secret columns added.
  - JSON writes are validated.
  - Helpers are not exposed to the Tauri command surface yet.

## Reviewer focus areas

- Typed error classification is correct and does not hide constraint/integrity failures.
- `insert_or_get_entity_link` handles idempotent logical duplicates without masking primary-key collisions.
- List/read ordering is deterministic.
- Scope remains P1.06 repository-helper primitives only, without building P1.07-P1.16 services early.
- Tests prove red/green behavior and cover critical edge cases.

## Fix cycle notes

- Initial quality review requested one required fix: do not convert all entity-link constraints into idempotent duplicate handling.
- Fix commit `f9cb79a` preserves non-idempotent `RepositoryError::Constraint` values when the requested logical tuple does not exist.
- Re-review verdicts after `f9cb79a`:
  - Spec: PASS.
  - Quality: APPROVED.
- Final critique then found a deeper combined edge case: supplied id collides with row A while requested logical tuple already exists on row B.
- Fix commit `098acec` checks the existing id row before logical-tuple fallback and preserves `RepositoryError::Constraint` when the id row belongs to a different logical tuple.
- Added regression test `insert_or_get_entity_link_preserves_constraint_for_id_collision_with_existing_logical_tuple`.
- Parent verification after `098acec`:
  - `cargo test --manifest-path src-tauri/Cargo.toml insert_or_get_entity_link_preserves_constraint_for_id_collision_with_existing_logical_tuple -- --nocapture`: PASS, 1 passed.
  - `npm run verify:local`: PASS; Rust tests 29 passed, frontend build passed.
