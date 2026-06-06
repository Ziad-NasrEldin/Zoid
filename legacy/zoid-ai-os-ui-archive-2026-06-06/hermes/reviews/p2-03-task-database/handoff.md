# Feature Handoff: P2.03 Task Database

## Original request

Proceed to P2.03 database TDD after Phase 2 planning/review approval.

## Implementation summary

- Added Phase 2 `tasks` SQLite migration and registered it as migration version 5.
- Added backend-only Rust task data model and repository helpers for create/read/list/status/archive/delete.
- Added task lifecycle events for `task.created`, `task.status_changed`, `task.archived`, and `task.deleted` with task event targets.
- Added validation for required title, optional detail, status/priority enums, JSON metadata, and secret-like metadata rejection.
- Added active-task query behavior that excludes archived/deleted tasks by timestamp and status.
- No frontend, Tauri bridge, CLI-runner, or UI changes in this slice.

## Changed files

- `src-tauri/src/lib.rs`: task enums/models, repository functions, migration registration, task event writing, and P2.03 tests.
- `src-tauri/migrations/0005_phase2_tasks.sql`: `tasks` table and indexes.

## How to test

From `/Users/ziadnasreldin/Zoid`:

- `cargo test --manifest-path src-tauri/Cargo.toml p203 -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml task`
- `cargo test --manifest-path src-tauri/Cargo.toml`

Expected: focused P2.03 tests, task-filter tests, and full backend tests pass.

## Tests run

- RED before implementation: `cargo test --manifest-path src-tauri/Cargo.toml p203_task -- --nocapture` failed with missing task symbols before implementation.
- RED after review fixes: `cargo test --manifest-path src-tauri/Cargo.toml p203 -- --nocapture` failed on missing detail validation, archived/deleted active-list filtering, and missing archive/delete events.
- GREEN focused: `cargo test --manifest-path src-tauri/Cargo.toml p203 -- --nocapture`: PASS, 4 passed / 0 failed.
- Broader task filter: `cargo test --manifest-path src-tauri/Cargo.toml task`: PASS, 7 passed / 0 failed.
- Full backend: `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 94 passed / 0 failed; doc-tests 0 passed / 0 failed.

## Git info

- Branch: `main`
- Base state: `main...origin/main [ahead 1]` before P2.03 commit; docs commit already exists from P2.01/P2.02.
- P2.03 commit: pending critique approval.
- Current uncommitted P2.03 files: `src-tauri/src/lib.rs`, `src-tauri/migrations/0005_phase2_tasks.sql`, and this handoff.

## Frontend/backend/database notes

- Frontend routes/components: not affected.
- Backend endpoints/services: no public Tauri commands added; repository layer only.
- Database tables/migrations: new `tasks` table with title/detail/status/priority/workspace/timestamps/archive/delete/metadata fields and active/status/workspace/archive/delete indexes.
- Security: `tasks.metadata_json` rejects secret-like keys/values through existing no-secret JSON validation; event summaries/metadata use existing redaction via `create_event_record`.

## Reviewer focus areas

- Verify P2.03 scope matches Phase 2 spec and does not prematurely add frontend/Tauri bridge scope.
- Verify SQLite migration registration, schema constraints, indexes, idempotence, and active list query semantics.
- Verify metadata secret rejection prevents raw secret-like JSON persistence in `tasks`.
- Verify event writing for task create/status/archive/delete is adequate for later History slice.
- Verify tests cover required title/detail validation, enum rejection, archive/delete active filtering, task event targets, and redaction/security behavior.

## Fix cycle notes

Initial read-only spec and quality reviews requested changes for:

- Missing detail validation.
- Raw secret-like task metadata persistence.
- Initial archived/deleted status leaking into active list.
- Missing `task.archived` and `task.deleted` events.

Fixes applied:

- Added `TASK_DETAIL_MAX_BYTES`, `normalize_task_detail`, Rust detail validation, and SQLite detail CHECK constraint.
- Switched task metadata validation to `validate_no_secret_json` and added secret metadata rejection test.
- Updated active-list query to exclude archived/deleted statuses as well as archive/delete timestamps.
- Added task lifecycle events for archive/delete helpers and tests for both event types.
- Re-ran focused, task-filter, and full backend tests successfully.

## Current review status

- Read-only spec re-review: PASS.
- Read-only code quality/security re-review: APPROVED.
- Final critique-agent verdict: pending.
