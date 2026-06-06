# Feature Handoff: P2.28-P2.30 backend regression tests

## Original request

Use the Zoid-wide subagent workflow to finish multiple things at once. Current tracker items:

- `P2.28 Tests: backend tests for task persistence and event writing.`
- `P2.29 Tests: backend tests for run lifecycle, cancellation, exit codes, log persistence, redaction.`
- `P2.30 Tests: backend tests for review records, notifications, history queries.`

## Implementation summary

- Added P2.28 regression coverage that creates/updates/archives/deletes tasks, reopens the file-backed SQLite database, reruns migrations, verifies task persistence, and verifies persisted task events through history queries.
- Added P2.29 regression coverage for failed agent-run bridge execution: failed status, exit code, log reference, redacted stream/log output, run failed history event, and failure notification.
- Added/renamed P2.29 cancellation regression coverage for active process cancellation, cancellation evidence, notification linkage, stream behavior, and terminal mutation rejection.
- Extended existing P2.19 review/notification/history bridge tests with `p230` names so focused P2.30 checks cover review records, notifications/inbox, and history queries.

## Changed files

- `src-tauri/src/tests.rs`: added/updated P2.28, P2.29, and P2.30 backend regression tests.

## How to test

- `cargo test --manifest-path src-tauri/Cargo.toml p228 -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml p229 -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml p230 -- --nocapture`
- `git diff --check`

Expected behavior:

- P2.28: task records and task events survive SQLite reopen/migration rerun.
- P2.29: failed/cancelled runs preserve truthful evidence and redact secret-looking output.
- P2.30: review, notification/inbox, and history bridge regressions pass under focused filter.

## Tests run

- `cargo test --manifest-path src-tauri/Cargo.toml p228 -- --nocapture`: PASS (1 passed)
- `cargo test --manifest-path src-tauri/Cargo.toml p229 -- --nocapture`: PASS (2 passed)
- `cargo test --manifest-path src-tauri/Cargo.toml p230 -- --nocapture`: PASS (2 passed)
- `git diff --check`: PASS

## Git info

- Branch: `main`
- Commit SHA: not committed yet
- Current base before commit: `bd03152 feat: add review notification history Tauri bridge`

## Frontend/backend/database notes

- Frontend routes/components: none.
- Backend endpoints/services: tests cover existing task/run/review/notification/history repository/service/Tauri bridge helpers.
- Database tables/migrations: no schema changes; P2.28 verifies file-backed SQLite persistence after reopen and migration rerun.

## Reviewer focus areas

- Ensure tests assert real behavior rather than only helper existence.
- Ensure P2.29 redaction tests do not persist secret-looking output in stream/log/history.
- Ensure P2.30 focused filter genuinely exercises review/notification/history query coverage.
- Ensure no implementation code was changed for these test items.

## Fix cycle notes

Initial critique returned `REQUEST_CHANGES`; required fixes addressed in tests only:

- RF-1: added dedicated `p230_review_notification_history_bridge_records_state_transitions_and_targets` covering review-linked notification creation, delivered/action_required/read/resolved transitions, and task/run/review/notification targets through bridge history commands.
- RF-2/RF-3: tightened P2.29 cancellation to wait for deterministic pre-cancel output, remove empty-stream acceptance, read the persisted cancellation log, validate log reference path/metadata/byte fields, and assert post-sleep output is absent.
- RF-4: asserted cancellation notification type/title/severity and `run.cancelled` history target.
- RF-5: broadened failed-run redaction checks over serialized notifications and SQLite text fields for `agent_runs`, `events`, `event_targets`, and `notifications`.
- RF-6: strengthened P2.28 reopened field assertions and task event semantics for source/outcome/workspace/primary target relation.

Post-fix verification:

- `cargo fmt --manifest-path src-tauri/Cargo.toml`: PASS
- `cargo test --manifest-path src-tauri/Cargo.toml p228 -- --nocapture`: PASS (1 passed)
- `cargo test --manifest-path src-tauri/Cargo.toml p229 -- --nocapture`: PASS (2 passed)
- `cargo test --manifest-path src-tauri/Cargo.toml p230 -- --nocapture`: PASS (3 passed)
- `npm run test:frontend`: PASS
- `npm run build`: PASS (`✓ 36 modules transformed`, built in 290ms)
- `git diff --check`: PASS
