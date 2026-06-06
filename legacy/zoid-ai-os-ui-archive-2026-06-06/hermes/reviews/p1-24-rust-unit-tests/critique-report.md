# Critique Report: P1.24 Rust unit tests

## Verdict

APPROVED

## Summary

The feature adds the requested Rust-only unit test coverage for P1.24 without changing production code or drifting into the explicitly excluded P1.25 file-backed SQLite integration-test scope. The new tests are meaningful regressions around policy preview parsing, event NotFound behavior, entity-link filter validation, invalid-JSON redaction fallback, safe log scope fallback, and safe log truncation metadata. Existing tests in the same Rust suite continue to cover Keychain readiness truthfulness and path creation/idempotence/symlink guards. Local Rust and full local verification passed.

## What was changed

- `src-tauri/src/lib.rs`: added six unit tests inside the existing `#[cfg(test)]` module:
  - `tauri_bridge_policy_preview_allows_low_risk_and_rejects_invalid_parse_inputs`
  - `read_event_missing_id_returns_not_found`
  - `entity_link_list_filter_rejects_invalid_or_empty_filter_fields`
  - `redact_metadata_json_invalid_json_returns_redacted_notice_and_no_raw_secret`
  - `safe_log_scope_falls_back_to_app_for_empty_or_all_unsafe_scope`
  - `safe_log_writer_truncates_oversized_line_and_records_truncated_metadata`
- `git diff --stat` shows only `src-tauri/src/lib.rs | 187 insertions(+)`; no production logic, frontend, database schema, or migration files were changed.
- Existing Rust tests observed for the remaining requested P1.24 areas include Keychain readiness tests at `src-tauri/src/lib.rs:6481` and `src-tauri/src/lib.rs:6497`, plus path creation/idempotence/symlink guard tests at `src-tauri/src/lib.rs:4234`, `src-tauri/src/lib.rs:4255`, `src-tauri/src/lib.rs:4277`, `src-tauri/src/lib.rs:4296`, `src-tauri/src/lib.rs:4321`, `src-tauri/src/lib.rs:4344`, `src-tauri/src/lib.rs:4377`, `src-tauri/src/lib.rs:4404`, `src-tauri/src/lib.rs:4420`, `src-tauri/src/lib.rs:4442`, and `src-tauri/src/lib.rs:4463`.

## Required fixes

| ID | Severity | Area | Issue | Evidence | Required fix |
|----|----------|------|-------|----------|--------------|
| None | - | - | No blocking issues found. | `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `cargo test --manifest-path src-tauri/Cargo.toml --lib`, and `npm run verify:local` all passed. Diff is tests-only. | None. |

## Improvements

| ID | Priority | Area | Suggestion | Why it matters |
|----|----------|------|------------|----------------|
| I1 | Low | Test maintainability | Consider moving the large Rust test module into one or more dedicated test modules/files when the Rust suite grows further. | `src-tauri/src/lib.rs` is already large, and continued test growth in the same file may make future review and navigation harder. This is non-blocking for P1.24. |
| I2 | Low | Test coverage | Consider adding an explicit target-direction invalid-filter test for `list_entity_links_by_target` in a future cleanup, even though the shared `validate_entity_link_list_filter` path is covered via source-direction listing. | It would protect against accidental divergence if source and target listing validation are later separated. Non-blocking because the current shared validation function is exercised. |

## Tests performed

- Read handoff: `/Users/ziadnasreldin/Zoid/.hermes/reviews/p1-24-rust-unit-tests/handoff.md`.
- Inspected working tree and diff:
  - `git status --short`: `M src-tauri/src/lib.rs` plus untracked review directory.
  - `git diff --stat`: one source file changed, 187 insertions.
  - `git diff -- src-tauri/src/lib.rs`: confirmed added code is unit tests only.
- Inspected relevant implementation/test areas in `src-tauri/src/lib.rs` for policy preview parsing, event reads, entity-link validation, redaction fallback, safe log writing/scope handling, Keychain readiness, and path creation guards.
- Ran formatting check:
  - `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`: PASS.
- Ran Rust library tests:
  - `cargo test --manifest-path src-tauri/Cargo.toml --lib`: PASS, 88 passed, 0 failed.
- Ran full local verification:
  - `npm run verify:local`: PASS.
  - Included `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 88 lib tests passed, 0 main/doc tests.
  - Included frontend tests: PASS.
  - Included TypeScript/Vite production build: PASS.
- Ran whitespace/security spot checks:
  - `git diff --check`: PASS/no output.
  - Added-line scan for hardcoded assignment-style secrets and common dangerous constructs: no matches.

## Tests still needed

- None required for this Rust-unit-test-only P1.24 slice.
- P1.25 SQLite file-backed integration/reopen lifecycle tests remain intentionally out of scope, as stated in the handoff.
- No deployed/prod E2E was performed because this feature is tests-only and not a deployment request.

## Dev-agent instructions

1. No required fixes are needed for P1.24.
2. Keep P1.25 file-backed SQLite integration/reopen lifecycle tests separate from this approved Rust unit-test slice.
3. Optionally address I1/I2 in a future test-maintenance cleanup, but do not block this feature on them.
4. If you make any further changes before commit, re-run `cargo fmt --manifest-path src-tauri/Cargo.toml -- --check`, `cargo test --manifest-path src-tauri/Cargo.toml --lib`, and `npm run verify:local`, then request re-review if the handoff changes.
