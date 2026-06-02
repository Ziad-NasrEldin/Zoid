# P2 Backend Query/Service Batch Handoff

## Verdict requested
Final critique for the Phase 2 backend query/service batch covering P2.07, P2.08, P2.15, and P2.16.

## Scope
- P2.07 History/Event query model optimized for task/run/entity timelines.
- P2.08 task create/list/detail/update service with event writing.
- P2.15 notification creation/query service basics over the already-approved P2.06 repository primitives.
- P2.16 history query service combining events/entity links without leaking raw logs/secrets.
- Adjacent monolith reduction: moved the large `#[cfg(test)]` module out of `src-tauri/src/lib.rs` into `src-tauri/src/tests.rs`; new query/service code lives in dedicated modules.
- Added `Docs/2026-06-02-phase-2-velocity-operating-model.md` to document grouped Phase 2 review/commit boundaries.

## Files changed
- `src-tauri/src/lib.rs`
  - Added module declarations/re-exports for `history_service`, `notification_service`, and `task_service`.
  - Moved test module to `src-tauri/src/tests.rs` behind `#[cfg(test)] mod tests;`.
- `src-tauri/src/history_service.rs`
  - Adds task, run, notification, and generic entity history query helpers.
  - Uses event/event_target records and entity_links to compose timelines.
  - Adds deterministic ordering by `(timestamp desc, id desc)` and cursor pagination.
  - Caps limits and rejects invalid history entities.
  - Run history uses a relation-aware entity set to avoid leaking sibling run/review/notification events through broad task ownership.
- `src-tauri/src/task_service.rs`
  - Adds task service create/list/read/update/status/archive/delete wrappers.
  - Adds field update service for title/detail/priority/workspace/metadata with validation and `task.updated` event writing.
- `src-tauri/src/notification_service.rs`
  - Adds notification service create/read/inbox and state transition wrappers.
- `src-tauri/src/tests.rs`
  - Extracted existing tests from `lib.rs`.
  - Adds P2.07/P2.08/P2.15/P2.16 focused coverage.
- `Docs/2026-06-02-phase-2-velocity-operating-model.md`
  - Documents batching/risk-band review model.

## Review history
- Combined read-only review returned `REQUEST_CHANGES` with two required fixes:
  1. P2.08 task update service was incomplete.
  2. Run history could include sibling/unrelated run events through task-owner expansion.
- Fixes applied:
  1. Added `TaskServiceUpdateInput` and `update_task_service` with validation, atomic update, and `task.updated` event coverage.
  2. Changed `list_run_history` to use `run_history_entity_set`, including only the run plus directly linked review/notification entities and review-linked notifications; it no longer adds the owning task as a broad event target.
  3. Added regression tests for field-update persistence/event writing and sibling-run exclusion.
- Combined re-review returned `PASS`.

## Verification
All commands were run from `/Users/ziadnasreldin/Zoid` after fixes:

- `git diff --check` — passed.
- `cargo test --manifest-path src-tauri/Cargo.toml p207 -- --nocapture` — 3 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml p208 -- --nocapture` — 1 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml history -- --nocapture` — 3 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml notification -- --nocapture` — 6 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml` — 116 passed, 0 failed, doc-tests 0.

## Security/redaction notes
- History queries read existing redacted event records and event targets; raw logs are not introduced or queried.
- P2.07 pagination/redaction tests verify no raw log body metadata is exposed in the history timeline.
- Task update service validates metadata JSON with the existing no-secret guard before persistence.
- Notification service is a thin wrapper over P2.06 repository logic, preserving existing secret-material rejection and transition guards.

## Known boundaries / non-goals
- No new migration in this batch; it builds on P2.03-P2.06 schema and repositories.
- No Tauri command bridge changes for these services yet; those remain P2.17-P2.19.
- No process runner work; P2.09-P2.13 remain pending.
- No UI work; P2.20-P2.27 remain pending.
