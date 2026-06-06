# Critique Report: Phase 3 P3.08 safe file actions

## Verdict

APPROVED

## Summary

P3.08 adds backend-only safe file action helpers for copy, rename, move, and trash under the visible root. The prior required fix is resolved: destination preview/index extraction failures caused by invalid preview bytes no longer turn an already-successful filesystem action into a returned error with partial side effects. The implementation now satisfies the requested confirmation gate, destination collision rejection, symlink/path escape rejection, non-destructive trash behavior, and file/index state updates for this backend-only phase. Focused and full local verification pass.

## What was changed

- `src-tauri/src/lib.rs`: added file action request/record types, `perform_file_action_service`, policy clearance gating, source/destination path safety helpers, trash destination selection, DB/index status helpers, and non-fatal destination preview indexing for unreadable/non-previewable bytes after a successful safe file action.
- `src-tauri/src/tests.rs`: added P3.08 tests for confirmation blocking, approved copy/rename/move, collision rejection, path escape rejection, symlinked destination parent rejection, broken final destination symlink rejection, invalid preview bytes after approved copy, and non-destructive trash/index state updates.
- `.hermes/reviews/phase-3-p308-file-actions/handoff.md`: updated fix-cycle notes and test evidence after the re-review fix.

## Required fixes

| ID | Severity | Area | Issue | Evidence | Required fix |
|----|----------|------|-------|----------|--------------|
| None | - | - | No blocking issues found in this re-review. | `cargo test --manifest-path src-tauri/Cargo.toml p308 -- --nocapture` passed 5 P3.08 tests; `npm run verify:local && git diff --check` passed. | None. |

## Improvements

| ID | Priority | Area | Suggestion | Why it matters |
|----|----------|------|------------|----------------|
| I1 | Medium | Backend/Test | Add a trash collision test that uses an existing broken symlink at the first trash destination. | `next_trash_relative_path` still uses `Path::exists()`, which treats broken symlinks as non-existent; `resolve_file_service_new_path` then rejects the symlink and the trash action fails instead of trying the suffix. This is non-blocking because it fails safe, but it weakens the advertised collision-safe trash behavior. |
| I2 | Low | Maintainability | De-duplicate `file_reference_entity_id` usage inside existing upsert/index helpers. | Reduces risk of future drift in file-reference entity IDs. |
| I3 | Low | Test | When P3.10 exposes these helpers through Tauri commands, add frontend/backend bridge and UX confirmation-flow E2E coverage. | P3.08 is backend-only; full-stack behavior is intentionally deferred until the command surface exists. |

## Tests performed

- Read `/Users/ziadnasreldin/Zoid/.hermes/reviews/phase-3-p308-file-actions/handoff.md`.
- Read the previous critique report and verified the prior R1 fix target.
- Inspected `git status --short`, branch, HEAD, and `git diff -- src-tauri/src/lib.rs src-tauri/src/tests.rs`.
- Inspected relevant source around `perform_file_action_service`, policy clearance, destination resolution, destination indexing, symlink/collision handling, and trash destination logic.
- Inspected P3.08 test coverage in `src-tauri/src/tests.rs`.
- Ran `cargo test --manifest-path src-tauri/Cargo.toml p308 -- --nocapture`: PASS, 5 passed, 0 failed.
- Ran `npm run verify:local && git diff --check`: PASS. Rust tests: 157 passed, 0 failed, 1 ignored. Frontend tests: PASS. Frontend build: PASS. Diff whitespace check: PASS by command exit status.

## Tests still needed

- Non-blocking: trash destination suffix behavior when the first trash destination is an existing broken symlink.
- Future phase: frontend/backend bridge and UX confirmation-flow E2E coverage once P3.10 exposes file action commands.

## Dev-agent instructions

1. No required fixes remain for P3.08.
2. Optionally add I1/I2 improvements in a follow-up or future phase.
3. Update the tracker and commit the P3.08 changes.
