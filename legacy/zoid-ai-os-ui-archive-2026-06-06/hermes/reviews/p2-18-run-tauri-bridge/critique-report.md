# P2.18 Run Tauri Bridge Critique Report

## Verdict

APPROVED

The RF-1 race/evidence fix resolves the prior blocking concern. `start_agent_run_command_with_connection` now waits for the worker to register the child process in `ACTIVE_RUN_CHILDREN` before returning, so an immediate bridge cancel has a deterministic cancellable handle. The worker also preserves cancelled-run completion evidence after the bridge has terminalized the run by using `finalize_cancelled_run_evidence` for already-cancelled runs, and the strengthened cancellation regression now verifies log reference, duration, and notification linkage after cancellation.

RF-2 and RF-3 remain satisfied: streaming is offset-based with `next_offset`/`eof`/`status`, live output is served from redacted persisted safe logs, and the start response contains no raw stdout/stderr payload.

## Required changes

None.

## Re-reviewed fixes

| Prior ID | Status | Notes |
| --- | --- | --- |
| RF-1: live start/status/cancel semantics | Fixed | Start waits for worker-ready child registration before returning; cancel kills through the active-child registry; the worker recognizes a persisted cancelled run and finalizes evidence without tripping the terminal mutation guard. Tests cover immediate cancellation and evidence persistence. |
| RF-2: incremental streaming contract | Fixed | `AgentRunCommandStreamRequest` supports `offset`/`max_bytes`; `AgentRunCommandStreamChunk` includes `offset`, `next_offset`, `eof`, `status`, and `content`; partial live stdout and final redacted EOF behavior are covered. |
| RF-3: raw stdout/stderr in start response | Fixed | `AgentRunCommandOutcome` only returns `session_id`, `run`, and `log_path`; raw process output is only available through redacted log streaming. |

## Notes / optional follow-up

- Worker failure paths after successful child registration (for example later log/database errors) could still be expanded with additional stuck-running regression tests, but this is not required for P2.18 approval.
- The bridge still accepts `logs_dir` from the caller for start/stream; a future hardening pass could derive this server-side to reduce path confusion.
- Long-running outputs still rely on safe-log rotation semantics, so clients should understand that byte offsets are into the current redacted log file.

## Evidence reviewed

- Read handoff: `/Users/ziadnasreldin/Zoid/.hermes/reviews/p2-18-run-tauri-bridge/handoff.md`.
- Inspected current uncommitted diff/source for `src-tauri/src/lib.rs`, `src-tauri/src/agent_execution_service.rs`, and `src-tauri/src/tests.rs`.
- Inspected the start/status/stream/cancel helpers, worker lifecycle, active child registry insertion/removal, cancelled-run evidence finalization, safe-log path, and terminal transition guards.
- Ran `cargo fmt --manifest-path src-tauri/Cargo.toml --check && cargo test --manifest-path src-tauri/Cargo.toml p218 -- --nocapture && cargo test --manifest-path src-tauri/Cargo.toml tauri_bridge -- --nocapture` — PASS: P2.18 2 passed; Tauri bridge 8 passed.
- Ran `cargo test --manifest-path src-tauri/Cargo.toml` — PASS: 126 passed, 0 failed, doc-tests 0.
