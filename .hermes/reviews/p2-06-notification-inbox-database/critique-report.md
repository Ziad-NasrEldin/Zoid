# Critique Report: P2.06 Notification/Inbox Database

## Verdict

APPROVED

## Summary

The P2.06 Notification/Inbox database slice satisfies the stated acceptance criteria for persistent in-app notification records. The implementation adds the version 8 migration, a `notifications` table with direct task/run/review links, repository primitives for creation/query/transitions, notification event writing, durable entity links, secret-material rejection/redaction paths, and focused regression tests. I found no required fixes blocking approval.

## What was changed

- Added migration registration for version 8 (`phase2_notifications`) in `src-tauri/src/lib.rs`.
- Added `src-tauri/migrations/0008_phase2_notifications.sql` with:
  - `notifications` table.
  - Type/severity/state CHECK constraints.
  - `task_id`, `run_id`, and `review_record_id` foreign keys with `ON DELETE RESTRICT`.
  - Timestamp consistency CHECK constraints for read/dismissed/resolved states.
  - Triggers enforcing task/run/review ownership consistency, including the task-level-review plus unrelated-run case when `task_id` is omitted.
  - Notification indexes.
- Added Rust notification model/repository primitives in `src-tauri/src/lib.rs`:
  - `NotificationType`, `NotificationSeverity`, `NotificationState`.
  - `NotificationRecord`, `NotificationCreateInput`.
  - `create_notification`, `read_notification`, `list_inbox_notifications`.
  - State transitions for delivered/action-required/failed/read/dismissed/resolved.
- Added notification event creation for `notification.created`, `notification.delivered`, `notification.action_required`, `notification.failed`, `notification.read`, `notification.dismissed`, and `notification.resolved`.
- Added durable entity links from task/run/review to notification where those direct links exist.
- Added P2.06-focused test coverage for schema constraints, creation/read/list/events/links, inbox severity sorting, ownership mismatch rejection, secret-material rejection, and non-mutating state transitions.

## Required fixes

None.

## Improvements

- Consider wrapping notification state update plus transition-event creation in a savepoint/transaction, mirroring `create_notification`, so a future event-write failure cannot leave a state transition without its corresponding event.
- Consider adding a covering/partial inbox index optimized for the active inbox query (`state`, computed severity ordering is not directly indexable as written, but a persisted severity rank or partial active-state index could help later if notification volume grows).
- Consider documenting transition policy explicitly if certain states should become terminal in a future UI/service layer. Current repository permits read -> dismissed -> resolved transitions, which is acceptable for this slice but may need product semantics later.

## Tests performed

From `/Users/ziadnasreldin/Zoid`:

- `cargo fmt --manifest-path src-tauri/Cargo.toml --check` — PASS.
- `git diff --check` — PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml p206 -- --nocapture` — PASS: 6 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml notification` — PASS: 5 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml` — PASS: 112 passed, 0 failed; doc-tests 0.

Evidence highlights:

- Schema version reaches 8 and includes all required notification columns (`p206_schema_version_eight_has_notifications_table_and_constraints`).
- DB-level constraints reject mismatched task/run, task/review, and task-level-review/unrelated-run combinations.
- Repository-level validation rejects the task-level-review/unrelated-run mismatch before persistence.
- Notification creation writes a `notification.created` event targeted at the notification plus linked task/run/review, and creates durable `entity_links` for those relationships.
- Inbox active query excludes a read notification and sorts remaining items by severity (`critical` before `warning` before `success`) and time/id ordering.
- Secret-like notification message/action-route/metadata input is rejected before persistence.
- Delivered/action-required/failed/read/dismissed/resolved transitions write events and do not mutate the linked task, run, or review records.

## Tests still needed

- No additional tests are required for this P2.06 database slice approval.
- Later phase/service work should add Tauri bridge and frontend Inbox tests once commands/UI are introduced.
- Later phase/history work should verify notification events appear correctly in task/run/entity timelines through the P2.07 history query model.

## Dev-agent instructions

No required implementation changes. You may proceed to the next slice (P2.07 History/Event query model) after recording this approval. If you do touch notification code later, prefer making state transition + event creation atomic with a savepoint and keep the existing P2.06 regression tests passing.
