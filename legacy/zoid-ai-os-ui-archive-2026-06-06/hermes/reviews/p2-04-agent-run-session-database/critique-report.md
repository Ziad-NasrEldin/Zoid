# Critique Report: P2.04 AgentRun/session database

## Verdict
APPROVED

## Summary
The P2.04 final critique required fixes have been implemented and verified. The mandatory `agent_runs.session_id` relationship now uses delete semantics compatible with `NOT NULL`, and profile commands are now rejected before persistence when they contain obvious secret-like material. Regression coverage was added for both issues, and the focused, run-filter, and full backend Rust suites pass.

## Re-review findings

| # | Prior required fix | Re-review result |
|---|--------------------|------------------|
| 1 | Change `agent_runs.session_id text not null` FK from `ON DELETE SET NULL` to compatible restrict semantics and add regression coverage. | Fixed. `src-tauri/migrations/0006_phase2_agent_runs.sql` now declares `foreign key (session_id) references cli_sessions(id) on delete restrict`. The P2.04 schema test creates a referenced session/run and verifies deleting the referenced `cli_sessions` row is rejected. |
| 2 | Reject raw secret-like material in profile `command` before it can be persisted or snapshotted. | Fixed. `src-tauri/src/lib.rs` adds `validate_no_secret_command`, calls it from `upsert_agent_profile` immediately after command normalization and before SQLite persistence, and returns `RepositoryError::SecretRejected` for obvious secret flags/tokens such as `--api-key`. The P2.04 regression test verifies a secret-like command is rejected and the profile row is not stored. Because `agent_runs.command_snapshot` and `profile_snapshot_json` are sourced from persisted `agent_profiles.command`, this blocks the reviewed raw-secret persistence/snapshot path. |

## Tests performed
- Read `.hermes/reviews/p2-04-agent-run-session-database/handoff.md`.
- Read previous `.hermes/reviews/p2-04-agent-run-session-database/critique-report.md`.
- Inspected current uncommitted diff/status for the scoped changes.
- Inspected `src-tauri/migrations/0006_phase2_agent_runs.sql` and confirmed the `session_id` FK is `on delete restrict`.
- Inspected `src-tauri/src/lib.rs` and confirmed `validate_no_secret_command` is invoked from `upsert_agent_profile` before persistence.
- Inspected the new P2.04 regression tests for referenced-session delete rejection and secret-like command rejection/no persistence.
- Ran `cargo test --manifest-path src-tauri/Cargo.toml p204 -- --nocapture`:
  - Result: pass, 6 passed, 0 failed.
- Ran `cargo test --manifest-path src-tauri/Cargo.toml run`:
  - Result: pass, 4 passed, 0 failed.
- Ran `cargo test --manifest-path src-tauri/Cargo.toml`:
  - Result: pass, 100 passed, 0 failed; doc-tests 0.

## Remaining issues
No remaining P2.04 approval blockers found in this re-review.

## Notes
- This approval is scoped to the P2.04 backend/database slice and the two final critique fixes. It does not claim runner/UI completion.
- Optional future hardening remains possible, such as broader command parsing/allowlisting and additional event-target/redaction assertions, but these are not blockers for this slice.
