# Feature Handoff: P1.11 Backend safe logging

## Original request

Phase 1 / Backend-Security: Implement safe log writer under app-support logs with filename/path sanitization, redaction, size/rotation basics, and no raw secret leakage.

Current user instruction: continue as orchestrator and delegate implementation/review tasks to subagents.

## Implementation summary

- Completed the safe log writer in `src-tauri/src/lib.rs`.
- Added deterministic log size bounds:
  - `SAFE_LOG_MAX_BYTES = 4096`
  - single oversized writes are truncated with `[TRUNCATED]`
  - truncation respects UTF-8 character boundaries
  - active log rotates to `.log.1` before append when the next write would exceed the limit
- Hardened log file path handling:
  - user-provided log scope is sanitized into a single safe filename segment
  - generated log paths are direct children of the app-support logs directory
  - active log symlinks are rejected before append
  - rotated log targets are validated/rejected before replacement
- Redaction happens before disk write.
- Safe log writer appends only redacted content to disk.
- Safe log writer upserts `log_references` rows into SQLite:
  - sanitized `log_scope`
  - safe `relative_path`
  - cumulative `redaction_count`
  - cumulative `byte_count`
  - safe metadata JSON with counts/flags/max byte limit only
- SQLite `log_references.metadata_json` does not store raw log content.
- `ensure_foundation` now passes the foundation DB connection into `write_safe_log`, so the foundation safe-log probe also persists a log reference row.
- No frontend changes.
- No new Tauri commands.
- No external calls or probes.

## Changed files

- `src-tauri/src/lib.rs`
  - Added safe log constants and rotation/truncation helpers.
  - Updated `write_safe_log` signature to accept a SQLite connection and upsert `log_references`.
  - Added safe metadata persistence for log references.
  - Updated `ensure_foundation` safe-log probe call.
  - Added/expanded safe-log tests.

## How to test

From `/Users/ziadnasreldin/Zoid`:

- `cargo test safe_log_writer --manifest-path src-tauri/Cargo.toml --lib -- --nocapture`
- `cargo test secure_redaction --manifest-path src-tauri/Cargo.toml --lib -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- `npm run verify:local`

Expected behavior:

- Unsafe scope/path input is sanitized to a direct child log path under the logs directory.
- Symlinked active log paths are rejected before append.
- Symlinked rotated paths are rejected before replacement.
- Secret-like content is redacted before disk write.
- Raw secrets do not appear in the log file or `log_references.metadata_json`.
- Log files rotate/truncate at the fixed threshold and do not grow unbounded under the writer.
- `log_references` is upserted idempotently for repeated writes and accumulates redaction/byte counts.

## Tests run

Implementation subagent TDD evidence:

- RED:
  - `cargo test safe_log_writer --lib`
  - Failed before implementation with expected missing behavior/symbols, including:
    - missing `SAFE_LOG_MAX_BYTES`
    - old 3-argument `write_safe_log` signature
    - DB persistence/rotation expectations not implemented yet
  - Exit code: 101.
- GREEN focused:
  - `cargo test safe_log_writer --lib`: PASS, 4 passed.
- Full Rust:
  - `cargo test`: PASS, 40 passed.
- Clippy:
  - `cargo clippy --all-targets --all-features -- -D warnings`: PASS.

Parent/orchestrator verification:

- `npm run verify:local`: PASS.
  - Rust tests: 40 passed, 0 failed.
  - Frontend build: PASS.

Independent reviews:

- Spec compliance review: PASS.
  - Reviewer ran:
    - `cargo test safe_log_writer -- --nocapture`: PASS, 4 passed.
    - `cargo test secure_redaction_masks_obvious_secrets -- --nocapture`: PASS.
    - exact secure redaction test: PASS.
    - `git diff --check HEAD^ HEAD -- src-tauri/src/lib.rs`: clean.
- Code quality/security review: APPROVED.
  - Reviewer ran:
    - `cargo test safe_log_writer -- --nocapture`: PASS, 4 passed.
    - `cargo test secure_redaction -- --nocapture`: PASS.
    - `cargo clippy --tests -- -D warnings`: PASS.

## Git info

- Branch: main
- Commit: `a72ca9b Implement safe log writer persistence and rotation`
- Diff base: `5414825 Record P1.08 settings service review`

## Frontend/backend/database notes

- Frontend routes/components:
  - No frontend changes.
  - Existing TypeScript build still passes.
- Backend services:
  - Safe log writer now performs redaction, sanitization, rotation/truncation, symlink checks, and DB reference persistence.
  - Existing foundation status uses the writer as a probe.
- Database:
  - Uses existing `log_references` table from P1.05.
  - No migration/schema change.
  - SQLite stores only references/counts/metadata flags, not raw log content.
- Security/privacy:
  - Raw secret-like content is redacted before disk write.
  - Raw log content is not stored in SQLite metadata.
  - Current symlink protection is pre-open/pre-rename validation; it is appropriate for this local app phase, but not an O_NOFOLLOW/openat race-proof design.

## Reviewer focus areas

- Confirm redaction occurs before file writes and DB metadata persistence.
- Confirm path sanitization prevents traversal/escape and direct-child logs only.
- Confirm active and rotated symlink handling rejects unsafe paths.
- Confirm rotation/truncation bounds log size deterministically.
- Confirm `log_references` upsert accumulates counts and never stores raw log content.
- Confirm no frontend/Tauri command/external call scope creep.

## Fix cycle notes

- Initial implementation passed spec review and quality/security review.
- No required fixes are currently open.
- Non-blocking reviewer notes:
  - Add explicit future tests for oversized single-write truncation and multibyte char-boundary truncation.
  - Consider O_NOFOLLOW/openat-style hardening if same-user filesystem race resistance becomes an explicit requirement.
