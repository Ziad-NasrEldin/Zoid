# Critique Report: Secure Foundation Services v1

## Verdict

APPROVED

## Summary

The R1 blocker from the prior review has been addressed. `write_event` now uses JSON-aware metadata redaction, preserving valid JSON for stored `events.metadata_json` while removing raw obvious secret values. The fix includes regression coverage that parses stored metadata and checks SQLite `json_valid(...) = 1`. Focused Rust tests, frontend build, and the local verification script all pass. I did not run `npm run verify:release`, per the handoff/request.

Overall, the Secure Foundation Services v1 slice is acceptable for this stage: centralized native helpers exist for redaction, safe app-support logging, action-policy decisions, and event writing; `foundation.ready` is routed through the generic writer; and the UI remains truthful that Keychain is unverified.

## What was changed

- `src-tauri/src/lib.rs`: `write_event` now calls `redact_metadata_json(...)` instead of plain text redaction for metadata.
- `src-tauri/src/lib.rs`: added JSON parsing and recursive redaction via `serde_json::Value`; invalid metadata falls back to a valid JSON envelope with redacted text.
- `src-tauri/src/lib.rs`: added/expanded tests for valid redacted event metadata, action policy matrix coverage, and safe log path sanitization.
- `src/App.tsx`: added secure foundation status types/card showing redaction, logging, action policy, event writer, and Keychain status.
- `src/App.css`: added styling for the secure foundation status list and policy note.
- `.hermes/reviews/secure-foundation-services-v1/handoff.md`: updated with fix-cycle notes and verification results.

## Required fixes

| ID | Severity | Area | Issue | Evidence | Required fix |
|----|----------|------|-------|----------|--------------|
| — | — | — | No required fixes remain. | R1 validated by source inspection and passing focused/full checks. | — |

## Improvements

| ID | Priority | Area | Suggestion | Why it matters |
|----|----------|------|------------|----------------|
| I1 | Low | Security/Test | Consider adding metadata redaction cases for nested objects/arrays, non-string secret-key values, invalid JSON fallback, and bearer/token-like values inside generic JSON strings. | The current regression proves the original invalid-JSON bug is fixed; broader cases would better lock down future redaction behavior. |
| I2 | Low | Backend/UX truthfulness | Consider backing `action_policy_ready` and `event_writer_ready` with lightweight runtime/self-test checks, or label them as configured/available rather than ready. | Current behavior is acceptable for this slice, but stronger runtime signals would make status semantics more precise. |
| I3 | Low | Security/Docs | Align handoff/docs wording with implementation: secret-looking JSON values under secret-looking keys are redacted, but object key names such as `api_key` are currently retained. | Retaining field names is usually useful and not a raw secret leak, but documentation should avoid implying keys are renamed if that is not intended. |

## Tests performed

- Read handoff: `/Users/ziadnasreldin/Zoid/.hermes/reviews/secure-foundation-services-v1/handoff.md`.
- Read prior report: `/Users/ziadnasreldin/Zoid/.hermes/reviews/secure-foundation-services-v1/critique-report.md`.
- Inspected current git status/diff for `src-tauri/src/lib.rs`, `src/App.tsx`, and `src/App.css`.
- Reviewed relevant Secure Foundation/action-policy requirements in `Docs/2026-05-31-zoid-implementation-plan-v1.md`.
- Source inspection confirmed `write_event` uses `redact_metadata_json(input.metadata_json)` at `src-tauri/src/lib.rs:424-425`.
- Source inspection confirmed `redact_metadata_json` parses JSON, recursively redacts values, serializes valid JSON, and falls back to a valid JSON envelope for invalid input at `src-tauri/src/lib.rs:481-495`.
- Source inspection confirmed the regression test parses stored metadata and checks SQLite JSON validity at `src-tauri/src/lib.rs:1169-1175`.
- `cargo test --manifest-path src-tauri/Cargo.toml generic_event_writer_redacts_and_links_targets -- --nocapture`: PASS; 1 test passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS; 7 tests passed.
- `npm run build`: PASS; TypeScript and Vite production build completed successfully.
- `npm run verify:local`: PASS; dependencies present, Tauri CLI found, Rust tests passed, frontend build passed, local push verification passed with `--skip-package`.
- Did not run `npm run verify:release`; release/DMG verification was intentionally not needed for this re-review.

## Tests still needed

- Native packaged-app visual verification of the Secure foundation card remains useful before a release build, but is not blocking for this source-level slice.
- If this feature is later exposed beyond the current internal helper surface, add API/integration tests for callers that submit event metadata.

## Dev-agent instructions

1. No required fixes remain for this review cycle.
2. Optionally address I1-I3 in a follow-up hardening pass.
3. Do not mark a release/DMG path verified unless `npm run verify:release` or equivalent package verification is intentionally run later.
