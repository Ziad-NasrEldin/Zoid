# P2 Agent Execution Service Batch Handoff

## Verdict requested
Final critique for Phase 2 backend agent execution batch covering P2.09 through P2.13.

## Scope
- P2.09 Backend: CLI profile config for at least one safe local command with truthful configured/unconfigured states.
- P2.10 Backend/native: command/session runner with cwd, captured stdout/stderr, stdin support, timeout kill/cancel cleanup, exit code, and duration.
- P2.11 Backend: child process cleanup and failure handling for cancelled/timed-out/crashed/nonzero runs.
- P2.12 Backend: persist redacted raw logs under app-support-style filesystem path; SQLite stores metadata/log reference only.
- P2.13 Backend: AgentRun lifecycle service writes start/progress/completion/failure/cancelled events.

## Files changed
- `src-tauri/src/lib.rs`
  - Adds `agent_execution_service` module declaration and crate-visible re-export.
- `src-tauri/src/agent_execution_service.rs`
  - Adds `AgentCommandRunRequest` and `AgentCommandRunOutcome`.
  - Adds `run_agent_command_service`.
  - Preflights task, configured profile, command availability, command shape, and cwd before creating session/run records.
  - Creates `cli_session` and `agent_run`, writes run queued/started events through existing repository primitives.
  - Executes configured command without shell interpolation by using profile command as executable and explicit argv.
  - Supports stdin when provided.
  - Supports timeout-based kill/cleanup; timed-out process is recorded as `cancelled`.
  - Captures stdout/stderr, exit code, duration.
  - Persists redacted log file via existing `write_safe_log` and links run to `log_references` metadata.
  - Completes run as `completed`, `failed`, or `cancelled` and writes lifecycle events.
  - Creates completion/failure/cancelled notifications.
- `src-tauri/src/tests.rs`
  - Adds P2.09-P2.13 tests covering preflight blockers, success output/log/event path, timeout kill/cancel cleanup, and failed process persistence.

## Safety / truthfulness notes
- Unconfigured profiles, missing commands, and missing cwd fail before any fake session/run success rows are created.
- Profile command must be executable path/name only; args are passed separately via `argv`.
- The runner does not store raw stdout/stderr in SQLite.
- Redacted filesystem logs are written through the existing safe log writer.
- Completion requires observed process result, duration, exit code where available, log reference, summary, and lifecycle event.
- Timeout kill is the implemented cancel/cleanup path for this backend slice; UI-driven live cancellation remains for P2.18 bridge/control wiring.

## Verification already passed
- `git diff --check`
- `cargo test --manifest-path src-tauri/Cargo.toml p209 -- --nocapture` — 1 passed
- `cargo test --manifest-path src-tauri/Cargo.toml p210 -- --nocapture` — 2 passed
- `cargo test --manifest-path src-tauri/Cargo.toml p211 -- --nocapture` — 1 passed
- `cargo test --manifest-path src-tauri/Cargo.toml p212 -- --nocapture` — 1 passed by matching shared P2.10/P2.12 test
- `cargo test --manifest-path src-tauri/Cargo.toml p213 -- --nocapture` — 1 passed by matching shared P2.10/P2.13 test
- Full Rust suite: `cargo test --manifest-path src-tauri/Cargo.toml` — 120 passed, 0 failed, doc-tests 0

## Review focus
- Ensure preflight does not create misleading success records.
- Ensure timeout kill/cancel handling is truthful enough for P2.10/P2.11 backend slice.
- Ensure log persistence and SQL metadata do not leak raw secret material.
- Ensure lifecycle events/notifications are written through approved repositories.
