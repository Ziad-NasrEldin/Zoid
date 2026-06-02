# P2.03 Task Database Critique Report

## Verdict

APPROVED

## Summary

The P2.03 task database slice satisfies the Phase 2 first vertical slice requirements for a backend-only `tasks` schema/model/repository implementation with task events. The migration is registered as version 5, creates an idempotent SQLite table with appropriate CHECK constraints and indexes, and the repository covers create/read/list/status/archive/delete behavior with validation and event targets. Security handling is acceptable for this slice: task metadata JSON is rejected when it contains secret-like keys/values, and task events pass through the existing event redaction path.

No required fixes are blocking approval.

## What was changed

- Added migration `src-tauri/migrations/0005_phase2_tasks.sql`:
  - `tasks` table with id, title, optional detail, status, priority, workspace key, timestamps, archive/delete timestamps, and metadata JSON.
  - SQLite CHECK constraints for title/detail lengths, valid status/priority enum values, and valid metadata JSON.
  - Active/status/workspace/archive/delete indexes.
- Registered migration version 5 in `MIGRATIONS`.
- Added backend Rust task types and repository helpers in `src-tauri/src/lib.rs`:
  - `TaskStatus`, `TaskPriority`, `TaskRecord`, `TaskCreateInput`.
  - `create_task_record`, `read_task_record`, `list_active_tasks`, `update_task_status`, `archive_task`, `soft_delete_task`.
  - normalization/validation helpers and task ID generation.
- Added task event writing:
  - `task.created` with a task event target.
  - `task.status_changed` with old/new status metadata and task target.
  - `task.archived` and `task.deleted` lifecycle events with task targets.
- Added focused tests for P2.03 task creation/defaults/events, validation/security, active-list filtering/order, status/archive/delete events, and migration assertions.

## Required fixes table

| ID | Severity | Area | Required fix | Status |
| --- | --- | --- | --- | --- |
| None | - | - | No required fixes. | Approved |

## Improvements table

| ID | Priority | Area | Suggested improvement | Rationale |
| --- | --- | --- | --- | --- |
| I-1 | Low | Repository semantics | Consider setting `archived_at`/`deleted_at` during creation if a caller explicitly creates a task with initial `Archived` or `Deleted` status, or reject terminal initial statuses if those should only happen through lifecycle helpers. | Active-list filtering is currently safe because it excludes both timestamps and terminal statuses, but timestamp/status consistency would be clearer for future history and UI logic. |
| I-2 | Low | Events/API completeness | When a non-status update helper is introduced, add `task.updated` event coverage from the Phase 2 event taxonomy. | P2.03 currently has no general task update operation, so this is not blocking; it should be covered when update behavior exists. |
| I-3 | Low | Tests | Add assertions that task event `metadata_json` for titles containing secret-like assignments is redacted, not just summaries. | The shared event writer appears to handle this, but an explicit P2.03 assertion would protect against regressions in task event formatting. |
| I-4 | Low | Transactionality | Consider making archive/delete status mutation and lifecycle event creation one atomic repository operation. | `update_task_status` is atomic for the status-change event; `task.archived`/`task.deleted` are written after that call. This is acceptable for current tests but stronger atomicity would be cleaner. |

## Tests performed

Commands run from `/Users/ziadnasreldin/Zoid`:

- `cargo test --manifest-path src-tauri/Cargo.toml p203 -- --nocapture`
  - Result: PASS. 4 tests passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml task`
  - Result: PASS. 7 tests passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml`
  - Result: PASS. 94 lib tests passed, 0 failed; 0 main tests; 0 doc-tests.

## Tests still needed

- No additional tests are required for P2.03 approval.
- Future Phase 2 slices should add integration coverage once tasks are connected to CLI sessions, agent runs, reviews, notifications, and history queries.
- Future UI/Tauri command exposure, if added, should get bridge and frontend smoke/build coverage.

## Dev-agent instructions

- No blocking changes are required before committing P2.03.
- Optional follow-ups may be handled in later slices or a cleanup pass:
  - Normalize initial archived/deleted timestamp semantics or restrict initial terminal statuses.
  - Add `task.updated` when general task update behavior is implemented.
  - Add explicit task event metadata redaction assertions for secret-like task titles.
  - Consider single-operation transactionality for archive/delete lifecycle event writes.

