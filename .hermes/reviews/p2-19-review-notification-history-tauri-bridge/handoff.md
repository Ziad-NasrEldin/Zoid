# Feature Handoff: P2.19 Review/Notification/History Tauri Bridge

## Original request

Use the Zoid-wide subagent workflow to continue Phase 2. Active tracker item:

- P2.19 Tauri bridge: commands for review records, notifications, inbox, history.

Phase 2 vertical-slice spec requires the native bridge to expose real local data for:

- manual ReviewRecord creation/read;
- persistent Notification/Inbox create/read/query/state transitions;
- event-backed task/run/review/notification history without leaking raw logs/secrets.

## Implementation summary

Implemented P2.19 Tauri command handlers and connection-injected helper functions in `src-tauri/src/lib.rs` for:

- `create_manual_review_command`
- `read_review_record_command`
- `create_notification_command`
- `read_notification_command`
- `list_inbox_notifications_command`
- `update_notification_state_command`
- `list_task_history_command`
- `list_run_history_command`
- `list_notification_history_command`
- `list_entity_history_command`

The bridge delegates to reviewed service/repository primitives instead of duplicating business logic. It preserves:

- manual review validation/linking through `create_manual_review_service` / `read_review_record`;
- notification create/read/inbox/state behavior through notification services;
- supported notification state actions only (`delivered`, `action_required`, `failed`, `read`, `dismissed`, `resolved`), with unsupported state strings rejected;
- history-service ordering, cursor/limit handling, relation expansion, and raw-log omission;
- secret safety: bridge tests cover review secret metadata rejection and notification secret rejection without persistence.

## Changed files

- `src-tauri/src/lib.rs`
  - Added P2.19 request DTOs, Tauri command functions, connection-injected command helpers, command-name list entries, and `tauri::generate_handler!` registrations.
- `src-tauri/src/tests.rs`
  - Added focused P2.19 tests for review/notification/inbox bridge behavior, notification state actions, secret rejection, and task/run/notification/entity history bridge queries without raw logs.

## How to test

Run from `/Users/ziadnasreldin/Zoid`:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p219 -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml tauri_bridge -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected results:

- `p219`: 2 passed.
- `tauri_bridge`: 8 passed.
- full Rust suite: 128 passed, 0 failed, doc-tests 0.

## Tests run

- RED before implementation: `cargo test --manifest-path src-tauri/Cargo.toml p219 -- --nocapture` failed to compile due missing P2.19 bridge surface.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml p219 -- --nocapture`: PASS, 2 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml tauri_bridge -- --nocapture`: PASS, 8 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 128 passed, 0 failed, doc-tests 0.
- `git diff --check`: PASS.

## Git info

- Branch: `main`
- Diff base: current `HEAD` (`5c720cb feat: add run Tauri bridge commands`)
- Working tree before commit: modified `src-tauri/src/lib.rs`, `src-tauri/src/tests.rs`, plus this review artifact.

## Frontend/backend/database notes

- Frontend routes/components: none changed. P2.20+ UI remains pending.
- Backend endpoints/services: no new service modules. Bridge calls existing reviewed backend services/repository helpers.
- Database tables/migrations: no schema changes.
- Tauri/native commands: 10 new P2.19 commands registered.

## Reviewer focus areas

- Command registration completeness (`TAURI_BRIDGE_COMMAND_NAMES` and `tauri::generate_handler!`).
- Whether bridge helpers preserve existing service/repository validation rather than bypassing it.
- Notification state action mapping and unsupported-state rejection.
- Review/notification secret handling through the bridge.
- History bridge cap/cursor/entity behavior and raw-log omission.

## Fix cycle notes

Combined reviewer verdict: PASS. One improvement requested bridge-level secret coverage; added it before this handoff:

- review secret metadata through bridge is rejected before persistence;
- notification secret material through bridge is rejected before persistence.
