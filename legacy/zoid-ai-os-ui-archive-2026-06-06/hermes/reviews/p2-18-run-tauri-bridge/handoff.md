# Feature Handoff: P2.18 Run Tauri Bridge

## Original request

Continue Zoid Phase 2 with P2.18: Tauri bridge commands/events for run start, stream, cancel, and status.

## Implementation summary

- Added P2.18 run bridge commands to start agent runs, read live run status, stream redacted run output incrementally, and cancel active runs.
- `start_agent_run_command` now creates a session/run, transitions the run to `running`, returns promptly, and spawns a background worker instead of blocking on process completion.
- Active process handles are tracked by run id so `cancel_run_command` can kill the child before transitioning the run to `cancelled`.
- `stream_run_output_command` supports `offset` and `max_bytes`, returns `next_offset`, `eof`, and current `status`, and reads redacted persisted log content while a process is still running.
- Bridge-visible start response no longer exposes raw stdout/stderr; output is available only through redacted streaming/log paths.
- Registered P2.18 commands in the Tauri invoke handler and bridge command surface list.

## Changed files

- `src-tauri/src/lib.rs`
  - Added run bridge request/response structs and command handlers.
  - Registered `start_agent_run_command`, `read_run_status_command`, `stream_run_output_command`, and `cancel_run_command`.
  - Added async run worker/process registry, active child cancellation, incremental safe-log streaming, and file-backed SQLite worker reopening.
- `src-tauri/src/agent_execution_service.rs`
  - Exposed internal runner helpers needed by the bridge worker while preserving existing synchronous service tests/behavior.
- `src-tauri/src/tests.rs`
  - Added file-backed DB helper for async worker tests.
  - Added P2.18 tests proving start returns `running`, status observes `running`, partial output streams before exit, final stream is redacted/EOF, and cancel kills an active process.
  - Updated command-surface registration expectations.
- `.hermes/reviews/p2-18-run-tauri-bridge/handoff.md`
  - Current handoff.
- `.hermes/reviews/p2-18-run-tauri-bridge/critique-report.md`
  - Prior critique report requested fixes; awaiting re-review after fixes.

## How to test

- `cargo fmt --manifest-path src-tauri/Cargo.toml --check`
- `cargo test --manifest-path src-tauri/Cargo.toml p218 -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml tauri_bridge -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml`

Expected behavior:
- Start command returns a `running` run without waiting for process exit.
- Status command can observe the running state.
- Stream command returns redacted chunks with offset/cursor metadata while the run is active and EOF after terminal completion.
- Cancel command kills the active child and leaves a terminal cancelled run.
- Start response does not include raw stdout/stderr.

## Tests run

- `cargo test --manifest-path src-tauri/Cargo.toml p218 -- --nocapture`: PASS, 2 passed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml --check && cargo test --manifest-path src-tauri/Cargo.toml p218 -- --nocapture && cargo test --manifest-path src-tauri/Cargo.toml tauri_bridge -- --nocapture && cargo test --manifest-path src-tauri/Cargo.toml`: PASS; P2.18 2 passed, bridge 8 passed, full suite 126 passed, doc-tests 0.
- After RF-1 re-review fix, reran `cargo fmt --manifest-path src-tauri/Cargo.toml --check && cargo test --manifest-path src-tauri/Cargo.toml p218 -- --nocapture && cargo test --manifest-path src-tauri/Cargo.toml tauri_bridge -- --nocapture && cargo test --manifest-path src-tauri/Cargo.toml`: PASS; P2.18 2 passed, bridge 8 passed, full suite 126 passed, doc-tests 0.

## Git info

- Branch: `main`
- Base before P2.18 commit: `main...origin/main [ahead 9]`
- Latest committed base before this feature: `04ebdb8 feat: add task CRUD Tauri bridge commands`
- P2.18 state: uncommitted pending re-review/approval.

## Frontend/backend/database notes

- Frontend routes/components: no frontend files changed.
- Backend/Tauri commands:
  - `start_agent_run_command`
  - `read_run_status_command`
  - `stream_run_output_command`
  - `cancel_run_command`
- Database tables used: existing `cli_sessions`, `agent_runs`, `events`, `entity_links`, `log_references`, `notifications`; no migration added.
- Async worker requires a file-backed SQLite connection for real bridge operation; tests use file-backed DBs for P2.18 worker behavior.

## Reviewer focus areas

- Confirm all prior REQUIRED_FIXES are resolved:
  - RF-1: live start/status/cancel semantics.
  - RF-2: incremental streaming contract.
  - RF-3: no raw stdout/stderr in bridge start response.
- Inspect process lifecycle and active child registry cleanup.
- Inspect SQLite connection/thread behavior and race handling around cancellation vs worker completion.
- Inspect secret redaction path for incremental stream chunks.

## Fix cycle notes

- Initial critique verdict was `REQUIRED_FIXES`.
- Fixes made after critique:
  - Replaced synchronous `run_agent_command_service` bridge start with async run/session creation and background process worker.
  - Added active child registry and kill path for `cancel_run_command`.
  - Added offset/max-bytes cursor fields and stream response metadata.
  - Removed raw stdout/stderr from `AgentRunCommandOutcome`.
  - Added regression tests for live running status, partial streaming before exit, cancellation kill behavior, redacted final stream output, and terminal mutation guard.
  - Re-review then requested another RF-1 fix for the immediate-cancel/worker-registration race.
  - Fixed the RF-1 race by making start wait for worker child registration before returning, preserving deterministic immediate cancellation.
  - Added cancelled-run worker evidence finalization so duration/log reference/notification evidence is preserved after bridge cancellation terminalizes the run.
  - Strengthened the cancel regression to verify cancelled runs receive worker evidence and notification linkage.
- Re-review requested after this handoff update.
