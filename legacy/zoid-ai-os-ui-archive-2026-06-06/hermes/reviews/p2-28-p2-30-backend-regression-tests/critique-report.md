# Critique Report: P2.28-P2.30 backend regression tests

## Verdict

APPROVED

## Summary

Re-reviewed the current on-disk `src-tauri/src/tests.rs` after the fix cycle. The previously requested fixes RF-1 through RF-6 are now resolved in test code, and the focused backend regression filters pass locally. The added coverage now asserts real persisted behavior for P2.28 task/event durability, stricter P2.29 failed/cancelled run evidence and redaction, and dedicated P2.30 review-linked notification history through bridge history commands.

No backend/source implementation edits were made during this re-review; only this critique report was updated.

## Scope reviewed

- Handoff/fix notes: `Zoid/.hermes/reviews/p2-28-p2-30-backend-regression-tests/handoff.md`
- Tests: `src-tauri/src/tests.rs`, focused on:
  - `p228_task_service_persists_tasks_and_task_events_after_reopen`
  - `p229_run_bridge_records_failed_exit_code_log_notification_and_redacted_stream`
  - `p229_run_bridge_cancel_kills_active_process_writes_log_and_rejects_terminal_mutation`
  - `p230_review_notification_history_bridge_records_state_transitions_and_targets`
  - existing `p219_p230_*` bridge/history regression tests

## Required fixes status

| ID | Area | Previous issue | Re-review finding | Status |
|---|---|---|---|---|
| RF-1 | P2.30 review/notification/history | P2.30 coverage was mostly renamed P2.19 tests and did not prove review-linked notification state/history behavior. | A dedicated `p230_review_notification_history_bridge_records_state_transitions_and_targets` test now creates a review-linked notification through bridge helpers, drives delivered/action_required/read/resolved transitions, and queries notification/run/entity history through bridge history commands. It asserts expected action types, workspace/outcome/source, state metadata, and notification/task/run/review targets plus entity links. | RESOLVED |
| RF-2 | P2.29 cancellation log evidence | Cancellation test did not read the persisted log or validate log reference fields. | The cancellation test now reads `logs_dir/<run_id>.log`, asserts it exists, contains the deterministic pre-cancel marker, excludes post-sleep output, and validates `log_references` scope/path/redaction/byte/metadata fields. | RESOLVED |
| RF-3 | P2.29 cancellation process-kill assertion | Stream assertion allowed empty content, weakening kill/evidence proof. | The test now waits until deterministic pre-cancel output appears before cancelling, then requires final stream/log content to contain that marker and exclude `should-not-finish`; the empty-content escape hatch is gone. | RESOLVED |
| RF-4 | P2.29 cancellation notification/event specificity | Cancellation only checked for some notification and did not verify `run.cancelled` history. | The test now verifies an `Attention` notification titled `Agent run cancelled` with warning severity and asserts run history includes `run.cancelled` targeted at the run as primary. | RESOLVED |
| RF-5 | P2.29 redaction persistence breadth | Failed-run redaction checks did not broadly scan notifications/events/SQLite fields. | The failed-run test now checks stream/log/history/notifications for absence of the raw secret and runs a SQLite aggregate over `agent_runs`, `events`, `event_targets`, and `notifications` text/metadata fields. | RESOLVED |
| RF-6 | P2.28 persistence/event assertions | P2.28 omitted updated field assertions and event semantics after reopen. | The P2.28 test now verifies reopened task detail/priority/status/workspace/metadata, archived/deleted timestamps, active list filtering, and task history event source/outcome/workspace/primary-target semantics after database reopen/migration rerun. | RESOLVED |

## Verification performed

From `/Users/ziadnasreldin/Zoid`:

- `cargo test --manifest-path src-tauri/Cargo.toml p228 -- --nocapture` → PASS (`1 passed`)
- `cargo test --manifest-path src-tauri/Cargo.toml p229 -- --nocapture` → PASS (`2 passed`)
- `cargo test --manifest-path src-tauri/Cargo.toml p230 -- --nocapture` → PASS (`3 passed`)
- `git diff --check` → PASS

## Notes

- The repository currently has unrelated modified/untracked frontend and review files visible in `git status`; this re-review only evaluated the P2.28-P2.30 backend regression tests and did not change source files.
- No remaining required fixes for this review scope.
