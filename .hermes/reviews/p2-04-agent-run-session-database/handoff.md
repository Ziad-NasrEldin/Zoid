# Feature Handoff: P2.04 AgentRun/session database

## Original request

Continue Zoid Phase 2 and implement P2.04 Database: AgentRun/session tables with task link, command/profile, cwd, status, duration, exit code, log reference, summary.

## Implementation summary

- Added Phase 2 migration `0006_phase2_agent_runs.sql`.
- Registered migration version 6 as `phase2_agent_runs`.
- Added backend/database-only repository primitives for:
  - `agent_profiles`
  - `cli_sessions`
  - `agent_runs`
- Added profile configured/unconfigured truthfulness:
  - configured profiles require a command;
  - unconfigured/missing-command profiles block run/session creation before fake success.
- Added CLI session persistence linked to task/profile with mode, cwd, status, status summary, metadata, timestamps.
- Added AgentRun persistence linked to task/session/profile with cwd, command snapshot, profile snapshot JSON, status, timestamps, duration, exit code, log reference id, output/error summaries, review state, metadata.
- Added lifecycle events for meaningful run states:
  - `run.queued`
  - `run.started`
  - `run.waiting_for_input`
  - `run.completed`
  - `run.failed`
  - `run.cancelled`
  - `run.blocked`
- Added entity links from task to session/run.
- Kept raw stdout/stderr/log body out of SQLite/events; SQLite stores log reference ids/paths and redacted summaries/metadata only.
- Applied required fixes from lean review:
  - `agent_runs.session_id` is now non-null.
  - `AgentRunCreateInput.session_id` is mandatory and validated against an existing session for the same task/profile.
  - completed runs require non-null `exit_code` and existing `log_reference_id`.
  - terminal runs are immutable, including same-terminal status rewrites.

## Changed files

- `src-tauri/migrations/0006_phase2_agent_runs.sql`
  - Creates `agent_profiles`, `cli_sessions`, and `agent_runs` tables and indexes.
- `src-tauri/src/lib.rs`
  - Registers migration version 6.
  - Adds P2.04 enums/models/repository functions.
  - Adds P2.04 tests.
- `.hermes/reviews/p2-04-agent-run-session-database/handoff.md`
  - This handoff.

## How to test

Run from `/Users/ziadnasreldin/Zoid`:

- `cargo test --manifest-path src-tauri/Cargo.toml p204 -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml run`
- `cargo test --manifest-path src-tauri/Cargo.toml`

Expected:

- P2.04 focused tests pass.
- Run-filter tests pass.
- Full backend test suite passes.

## Tests run

- RED after adding review-fix regression tests:
  - `cargo test --manifest-path src-tauri/Cargo.toml p204 -- --nocapture`
  - Result: FAILED as expected before fixes; 3 passed, 2 failed.
  - Failing tests:
    - `p204_run_creation_rejects_missing_deleted_task_and_unconfigured_profile`
    - `p204_terminal_transitions_are_immutable_and_completion_stores_evidence`
  - Failures proved missing mandatory session handling and missing completed-run evidence enforcement.
- GREEN focused:
  - `cargo test --manifest-path src-tauri/Cargo.toml p204 -- --nocapture`
  - Result: 5 passed, 0 failed.
- Broader run-filter:
  - `cargo test --manifest-path src-tauri/Cargo.toml run`
  - Result: 4 passed, 0 failed.
- Full backend:
  - `cargo test --manifest-path src-tauri/Cargo.toml`
  - Result: 99 passed, 0 failed; doc-tests 0.

## Git info

- Branch: `main`
- Commit SHA: pending; uncommitted at handoff time.
- Diff base: current `HEAD` after P2.03 commit `5ffb7b5 feat: add Phase 2 task repository`.

## Frontend/backend/database notes

- Frontend routes/components: not touched.
- Tauri bridge commands: not touched.
- Backend/database:
  - Adds internal repository/model layer only.
  - No real process runner yet.
  - No raw log persistence implementation changes in this slice; existing log reference table is used for references.
- Migration notes:
  - Fresh migrated in-memory DB now reports version 6.
  - Foreign keys link runs/sessions to tasks/profiles/log references.

## Reviewer focus areas

- Confirm P2.04 scope is backend/database-only and does not claim execution runner/UI completion.
- Confirm mandatory run/session/task/profile linkage.
- Confirm completed run evidence requirements.
- Confirm terminal run immutability.
- Confirm no raw stdout/stderr/log bodies are persisted in SQLite/events.
- Confirm secret-like metadata/config is rejected or redacted consistently.
- Confirm tests are sufficient for P2.04 acceptance targets.

## Fix cycle notes

Lean combined review initially returned REQUEST_CHANGES with three required fixes:

1. `agent_runs` allowed missing sessions.
2. completed runs did not require exit code/log reference.
3. terminal runs could be mutated if the target terminal status was the same.

All three were fixed and re-reviewed. Lean combined re-review returned PASS.

Final critique initially returned REQUEST_CHANGES with two required fixes:

1. `agent_runs.session_id` was `NOT NULL` but FK used `ON DELETE SET NULL`.
2. Profile `command` values could contain secret-like material and be persisted/snapshotted.

Both were fixed:

- `agent_runs.session_id` now uses `ON DELETE RESTRICT`, and the P2.04 schema test verifies deleting a referenced session is rejected.
- `upsert_agent_profile` now validates command strings for secret-like flags/tokens before persistence, and `p204_agent_profile_rejects_secret_like_command_before_persistence` verifies a raw API-key command is rejected and not stored.

Post-fix verification:

- `cargo test --manifest-path src-tauri/Cargo.toml p204 -- --nocapture`: 6 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml run`: 4 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 100 passed, 0 failed; doc-tests 0.