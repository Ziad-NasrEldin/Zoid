# Feature Handoff: P2.17 Tauri bridge task CRUD commands

## Original request

Continue Zoid Phase 2 from tracker. Active task: **P2.17 Tauri bridge: commands/events for task CRUD**.

## Implementation summary

- Added Tauri command request structs for task create/update/status changes.
- Added Tauri command handlers:
  - `create_task_command`
  - `read_task_command`
  - `list_tasks_command`
  - `update_task_command`
  - `update_task_status_command`
  - `archive_task_command`
  - `delete_task_command`
- Added connection-injected helpers for testability and to match existing bridge patterns.
- Commands delegate to the already-reviewed `task_service` APIs, preserving validation, secret metadata guards, active-list filtering, and task event writing.
- Registered the commands in `TAURI_BRIDGE_COMMAND_NAMES` and the Tauri `generate_handler!` block.

## Changed files

- `src-tauri/src/lib.rs`: added request structs, command handlers, connection-injected helpers, command registration markers, and `generate_handler!` entries.
- `src-tauri/src/tests.rs`: added P2.17 tests for CRUD bridge behavior, event evidence, command registration, and validation/secret guard preservation.

## How to test

From `/Users/ziadnasreldin/Zoid`:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p217 -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml tauri_bridge -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected behavior:
- P2.17 focused tests pass.
- Tauri bridge command registration test sees all 19 commands, including task CRUD commands.
- Full Rust suite passes.

## Tests run

- RED before implementation: `cargo test --manifest-path src-tauri/Cargo.toml p217 -- --nocapture` — failed as expected with missing `TaskCommandCreateRequest`, `TaskCommandUpdateRequest`, `TaskCommandStatusRequest`, and task command helper functions.
- GREEN: `cargo fmt --manifest-path src-tauri/Cargo.toml && cargo test --manifest-path src-tauri/Cargo.toml p217 -- --nocapture` — PASS, 2 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml tauri_bridge -- --nocapture` — PASS, 8 passed.
- Full Rust suite: `cargo test --manifest-path src-tauri/Cargo.toml` — PASS, 124 passed, 0 failed, doc-tests 0.

## Git info

- Branch: `main`
- Current base before commit: `06572d7 feat: add manual review service`
- Working tree currently has unstaged P2.17 changes in `src-tauri/src/lib.rs` and `src-tauri/src/tests.rs` plus this review handoff.

## Frontend/backend/database notes

- Frontend routes/components: not touched.
- Tauri bridge: task CRUD commands added and registered.
- Backend services: commands delegate to `task_service` wrappers.
- Database: no schema changes. Events are written by existing task repository/service paths (`task.created`, `task.updated`, `task.status_changed`, archive/delete status events).

## Reviewer focus areas

- Confirm task CRUD commands are present in both command marker list and `generate_handler!`.
- Confirm bridge helpers delegate to `task_service` rather than bypassing repository/service validation.
- Confirm create/update priority and status string parsing fails closed for unsupported values.
- Confirm secret-like task metadata is rejected before persistence.
- Confirm tests check task events remain written through the bridge path.

## Fix cycle notes

Critique review completed via `critique-agent` on 2026-06-02 with verdict `APPROVED`; no required fixes.
