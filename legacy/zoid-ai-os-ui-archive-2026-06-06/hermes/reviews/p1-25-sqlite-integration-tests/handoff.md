# Feature Handoff: P1.25 SQLite integration tests

## Original request

Continue Zoid tracker item:

`P1.25 Tests: SQLite integration tests for migrations, version, repositories, event read/write, entity links.`

Scope boundary: SQLite/file-backed integration coverage only. P1.26 frontend smoke/build checks are not in scope.

## Implementation summary

- Added file-backed SQLite integration coverage inside the existing Rust test module.
- Tests use a temporary app-support-style database path with `AppSupportPaths::for_home`, `open_foundation_database`, and real file-backed reopen behavior.
- Added a test-only `count_rows` helper for direct SQL row counts without changing production `count_table` behavior.
- Verified migration/version/seed persistence before rerun/reseed after reopen, then rerun/reseed idempotence.
- Verified repository/event/entity-link records persist across file-backed database close/reopen.
- No production behavior changes, schema changes, frontend changes, or migrations were added.

## Changed files

- `src-tauri/src/lib.rs`:
  - Added test-only helper `count_rows`.
  - Added `file_backed_sqlite_migrations_seed_counts_and_foreign_keys_are_reenabled_after_reopen`.
  - Added `file_backed_repository_event_and_entity_links_persist_across_reopen`.

## How to test

From `/Users/ziadnasreldin/Zoid`:

- `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`
- `cargo test --manifest-path src-tauri/Cargo.toml file_backed_ -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run verify:local`

Expected result:

- Targeted file-backed tests pass: 2 passed.
- Full Rust tests pass: 90 passed.
- Full local verification passes.

## Tests run

- Implementer ran `cargo fmt`: PASS.
- Implementer ran `cargo test`: PASS, 90 passed, 0 failed.
- Parent ran `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 90 passed, 0 failed.
- Spec review ran `cargo test --manifest-path /Users/ziadnasreldin/Zoid/src-tauri/Cargo.toml file_backed_ -- --nocapture`: PASS, 2 passed, 0 failed.
- Initial quality review ran targeted tests: PASS, 2 passed, then requested two fixes.
- Parent applied quality fixes and ran:
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: PASS.
  - `cargo test --manifest-path src-tauri/Cargo.toml file_backed_ -- --nocapture`: PASS, 2 passed, 0 failed.
  - `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 90 passed, 0 failed.
- Re-review spec: PASS.
- Re-review quality: APPROVED.

## Git info

- Branch: `main`
- Diff base: `5f16daf Add P1.24 Rust unit coverage`
- Commit SHA: not committed yet at handoff creation.

## Frontend/backend/database notes

- Frontend: not changed.
- Backend/Rust: tests only; no production behavior changed.
- Database: no migration/schema changes. Tests exercise the current migration runner and repository/event/entity-link behavior against a real file-backed SQLite DB path.
- Reopen behavior verifies `open_foundation_database` re-enables SQLite foreign keys on reopened connections; it does not imply foreign key PRAGMA is persisted in the DB file.

## Reviewer focus areas

- Confirm P1.25 coverage includes migrations/version/repositories/event read-write/entity links.
- Confirm file-backed reopen assertions are meaningful and happen before rerun/reseed.
- Confirm expected migration version is derived from the migration registry, not hardcoded.
- Confirm tests remain scoped to P1.25 and do not change production behavior.

## Fix cycle notes

- First quality review requested:
  1. Assert persisted migration/seed rows immediately after reopen before rerun/reseed.
  2. Avoid hardcoded migration version `4`.
- Fixes made:
  - Derived expected migration version from `MIGRATIONS.last().version`.
  - Reordered reopen assertions to check persisted rows before rerun/reseed.
  - Kept a second assertion pass after rerun/reseed to prove idempotence.
  - Renamed the foreign-key test to clarify that foreign keys are re-enabled on each reopened connection.
- Re-review spec passed and quality re-review approved.
