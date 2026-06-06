# Phase 5 Content and OmniSocials Manual / Native Verification

Date: 2026-06-05

Scope: Phase 5 Content + OmniSocials, draft-first and fail-closed. This verification intentionally does not publish, upload, schedule on an external platform, or configure real OmniSocials credentials.

## Verification method

Automated native-command verification was run through the Rust/Tauri backend and SQLite migration suite, plus frontend view-model/build checks. Visual macOS UI automation was not used in this pass; the Content workspace UI is covered by frontend tests/build and native bridge command tests, while the durable app-support/native behavior is covered by SQLite-backed Rust tests.

## Evidence commands

- `cargo test --manifest-path src-tauri/Cargo.toml p50 -- --nocapture`
  - Result: PASS, 6 Phase 5 Rust tests passed.
  - Covered schema, indexes/status states, plan -> draft -> asset -> review -> schedule-intent flow, denied confirmation blocking, OmniSocials fail-closed records, media constraints, event writing, and secret-safe failure reports.

- `cargo test --manifest-path src-tauri/Cargo.toml tauri_bridge_command_surface -- --nocapture`
  - Result: PASS, 2 bridge command-surface tests passed.
  - Covered command registry/generate_handler parity including Phase 5 content commands and media listing/read command additions.

- `npm run test:frontend && npm run build`
  - Result: PASS.
  - Covered Content workspace view-model helpers and full frontend suite; production build succeeded.

- `npm run verify:local`
  - Result: PASS.
  - Rust tests: 179 passed / 0 failed / 1 ignored.
  - Frontend tests: passed.
  - Frontend build: passed.
  - Final marker: `PASS: local push verification passed (--skip-package)`.

## Manual acceptance checklist

- [x] Create content plan/draft path is implemented through native command `create_content_plan_command` and `create_content_piece_command`.
- [x] Media asset references are local references only, not raw blobs or external uploads.
- [x] Specialist review gates are durable records and can be approved/rejected with evidence summary.
- [x] Schedule intent is blocked before required specialist approval and human confirmation.
- [x] Denied/cancelled/missing confirmation blocks schedule and records verification evidence.
- [x] Approved review + approved confirmation creates a local schedule intent only; it does not claim external platform scheduling.
- [x] OmniSocials default state is truthful `not_configured`.
- [x] OmniSocials upload/schedule/publish surfaces fail closed and record verification/failure rows when not configured.
- [x] Platform constraints are enforced before schedule/publish attempts where media is required.
- [x] Verification/failure reports are redacted before persistence.
- [x] Content workspace UI shows draft-first copy, review gate state, fail-closed OmniSocials copy, schedule intents, and blocked/failed verification reports.

## Deferred / explicit non-goals

- Real OmniSocials credential setup is not implemented in Phase 5.
- No real external upload, platform schedule, or publish operation was attempted.
- Full visual native UI click-through was not recorded in this pass; no Tauri-driver/AX UI harness is currently available for this workflow.
- Future real provider execution must be implemented as a separate reviewed slice with credentials, provider API error handling, and production E2E/provider verification.
