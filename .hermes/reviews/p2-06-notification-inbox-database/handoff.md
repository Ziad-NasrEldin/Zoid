# Feature Handoff: P2.06 Notification/Inbox database

## Original request

Continue Phase 2 implementation from the tracker. Next slice: `P2.06 Database: Notification/Inbox model for completion/blocker/attention records`.

Source tracker: `Docs/2026-06-01-zoid-implementation-tracker.md`
Phase 2 spec: `Docs/2026-06-02-phase-2-first-vertical-slice-spec.md`

Relevant spec requirements:
- Persistent `notifications`/Inbox records for completion, blocker, failure, review-required, and attention.
- Store type/title/message/severity/state/action route/read/dismiss/resolution timestamps.
- Link notifications to task/run/review through direct fields and durable links where useful.
- Read/dismiss/resolve transitions affect notification only, not underlying task/run/review.
- Creation writes `notification.created` event.
- Delivery/failure/read/resolved event taxonomy exists.
- Inbox query sorts actionable unread items by severity and time.
- No raw secrets/logs in SQLite/events.

## Implementation summary

- Added migration version 8: `phase2_notifications`.
- Added `notifications` table with type, title, message, severity, state, action route, direct task/run/review links, transition timestamps, metadata, indexes, and FK constraints.
- Added DB triggers to enforce task/run/review ownership consistency, including the reviewer-found edge case where a task-level review from task A cannot be paired with an unrelated run from task B when `task_id` is omitted.
- Added backend/repository primitives:
  - `NotificationType`
  - `NotificationSeverity`
  - `NotificationState`
  - `NotificationRecord`
  - `NotificationCreateInput`
  - `create_notification`
  - `read_notification`
  - `list_inbox_notifications`
  - `mark_notification_delivered`
  - `require_notification_action`
  - `mark_notification_failed`
  - `mark_notification_read`
  - `dismiss_notification`
  - `resolve_notification`
- Notification creation writes `notification.created` events and durable entity links:
  - task → notification
  - agent_run → notification
  - review_record → notification
- Notification transitions write events:
  - `notification.delivered`
  - `notification.action_required`
  - `notification.failed`
  - `notification.read`
  - `notification.dismissed`
  - `notification.resolved`
- Notification fields reject obvious secret material before persistence.
- Inbox active query excludes read/dismissed/resolved items and sorts by severity (`critical`, `error`, `warning`, `success`, `info`) then newest first.

## Changed files

- `src-tauri/migrations/0008_phase2_notifications.sql`: new migration for `notifications`, FK/index definitions, and consistency triggers.
- `src-tauri/src/lib.rs`: migration registration, notification enums/models/repository functions/events/entity links/tests.
- `.hermes/reviews/p2-06-notification-inbox-database/handoff.md`: this handoff.

## How to test

From `/Users/ziadnasreldin/Zoid`:

```bash
cargo fmt --manifest-path src-tauri/Cargo.toml --check
cargo test --manifest-path src-tauri/Cargo.toml p206 -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml notification
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected:
- P2.06 focused tests pass.
- Full Rust suite passes.
- No raw secret material persists in notification rows/events.
- DB rejects mismatched task/run/review direct-link ownership.

## Tests run

- `cargo fmt --manifest-path src-tauri/Cargo.toml`: PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml p206 -- --nocapture`: PASS — 6 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml notification`: PASS — 5 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS — 112 passed, 0 failed; doc-tests 0.
- `git diff --check`: PASS.

## Review evidence

Lean combined backend/database review:
- Initial verdict: `REQUEST_CHANGES`.
- Required fix: close task/run/review ownership gaps at DB and repository layers for task-level review + unrelated run with omitted `task_id`.
- Fix applied with DB trigger and repository validation changes plus direct-SQL and repository regression tests.
- Combined re-review verdict: `PASS`.

## Git info

- Branch: `main`
- Commit SHA: not committed yet.
- Current status before final critique: uncommitted P2.06 changes only.

## Frontend/backend/database notes

- Frontend routes/components: not touched.
- Backend/native: repository-only primitives in `src-tauri/src/lib.rs`; no new Tauri commands yet.
- Database: migration version 8, `notifications` table, indexes, ownership triggers.
- Native macOS notifications: intentionally not implemented in P2.06; native delivery remains optional/gated by verified native support per spec.

## Reviewer focus areas

- Spec compliance for P2.06 notification/inbox persistence.
- SQLite schema integrity: FK enforcement, trigger coverage, state/timestamp constraints.
- Repository/schema contract alignment.
- Event and entity-link correctness.
- Secret/log safety.
- Inbox active query semantics and deterministic ordering.

## Fix cycle notes

Combined review found one required fix: inconsistent direct links were possible when a task-level review and unrelated run were supplied without `task_id`. Fixed at both DB and repository layers, with regression tests. Combined re-review passed.
