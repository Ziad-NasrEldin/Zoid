# Phase 6 Calendar/Gmail/Inbox/Business/Products critique

Verdict: APPROVED

Date: 2026-06-05
Repo: `/Users/ziadnasreldin/Zoid`

## Evidence reviewed

- Tracker: `Docs/2026-06-01-zoid-implementation-tracker.md` P6.01-P6.26.
- Handoff: `.hermes/reviews/phase-6-calendar-gmail-business-products/handoff.md`.
- Verification doc: `Docs/2026-06-05-phase-6-calendar-gmail-business-products-verification.md`.
- Backend/source:
  - `src-tauri/migrations/0012_phase6_calendar_gmail_business_products.sql`
  - `src-tauri/src/phase6_service.rs`
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/tests.rs`
- Frontend/source:
  - `src/phase6Workspace.ts`
  - `src/phase6WorkspaceView.tsx`
  - `src/phase6Workspace.test.ts`
  - relevant `src/App.tsx` routing/search evidence
- Git/worktree checks:
  - `git status --short`
  - `git diff --stat`
  - `git diff --check`
- Verification rerun:
  - `npm run verify:local && git diff --check`

## Verification rerun result

Command run during critique:

```sh
npm run verify:local && git diff --check
```

Result: passed.

Observed output included:

- Rust: 182 passed, 0 failed, 1 ignored.
- Frontend tests: passed, including `src/phase6Workspace.test.ts`.
- Frontend production build: passed (`tsc && vite build`).
- Final marker: `PASS: local push verification passed (--skip-package)`.
- `git diff --check`: no whitespace errors.

Note: the command output line from Cargo reports `running 183 tests`, with final result `182 passed; 0 failed; 1 ignored`, matching the handoff/verification doc.

## Findings

### Tracker P6.01-P6.26

Approved.

The tracker entries for P6.01-P6.26 are marked complete and the wording is now honest about scope:

- P6.03/P6.04 define EventKit as a truthful blocker/local calendar reference surface, not external Apple Calendar access.
- P6.05/P6.06 define Gmail as not configured/local mail refs only, not OAuth/API/send integration.
- P6.18/P6.19 confirmation-gate tests are backed by source tests.
- P6.20 safe provider-copy test is backed by source tests.
- P6.21 persistence/cross-link test is backed by file-backed SQLite test.
- P6.22/P6.23 document blockers instead of overclaiming native integrations.
- P6.24 full local verification evidence matches the rerun.
- P6.25 handoff exists.
- P6.26 says critique completed/approved. This report is the approving critique artifact for the current state.

### Truthfulness of EventKit and Gmail blockers

Approved.

`src-tauri/src/phase6_service.rs` implements `phase6_integration_states()` with:

- `eventkit`: `needs_permission`
- `gmail`: `not_configured`

No EventKit permission prompt, external Apple Calendar write, Gmail OAuth/token storage/API read, or external Gmail send path was found in the Phase 6 source. The verification doc and handoff both state these are blocked/unconfigured boundaries.

Frontend copy is also safe:

- Integration card heading says `No fake connected data`.
- Gmail send result copy says `Draft marked sent locally. No external Gmail send was performed.`
- Browser/Tauri bridge failures are sanitized by `phase6SafeBridgeError()` rather than exposing raw `undefined` / `Cannot read properties...` details.

### Confirmation gates and fail-closed mutation behavior

Approved.

Backend checks:

- `require_approved_confirmation()` requires a persisted confirmation decision, matching action category, and `Approved` state.
- `create_calendar_event()` requires `create_calendar_event` confirmation.
- `update_calendar_event()` and `delete_calendar_event()` require `edit_delete_calendar_event` confirmation.
- `send_email_draft()` requires `send_email` confirmation.
- Calendar update/delete and email send verify affected row count and return a constraint error when no active/draft row was mutated, avoiding false success.

Frontend checks:

- `assertPhase6CalendarConfirmation()` fails closed for calendar create/update/delete without `confirmation_id`.
- `assertPhase6NoSilentSend()` fails closed for email send without `confirmation_id`.
- Forms require confirmation IDs before calendar writes and local send-state transition.

Test evidence:

- `p618_p619_calendar_and_email_writes_require_approved_confirmation` covers no-confirmation failures, approved success, repeat-send failure, deleted-calendar update failure, and secret-like calendar/email rejection.
- Frontend test covers no-silent-send and calendar confirmation guards.

### No fake data / safe user-facing copy

Approved.

Phase 6 browser fallback/empty overview contains empty arrays and blocker states, not fake records. `loadPhase6OverviewFromBridge()` returns an error state on failed native bridge load and safe copy says browser preview stays UI-only without simulated records.

Reviewed copy avoids claiming actual Gmail or EventKit execution. The only potentially broad phrase is the hero copy “connects attention, calendar, mail...” but it is immediately scoped by “native commands and persisted local records” and reinforced by blocker/status copy, so it is not blocking.

### Persistence, events, and cross-links

Approved.

Schema v12 adds persistent SQLite tables for:

- `companies`
- `contacts`
- `follow_ups`
- `products`
- `email_refs`
- `calendar_refs`

Backend create/link/calendar/email operations write Phase 6 events through `write_phase6_event()` with source `phase6_service` and redacted event handling delegated to the existing event writer.

Product links use the existing `entity_links` table with source type `product`. The persistence test `p621_business_product_cross_links_and_overview_persist` creates company/contact/product/follow-up/link records, verifies Phase 6 event emission, reopens file-backed SQLite, reruns migrations, and verifies overview records and product links persist.

### Secret handling

Approved.

Phase 6 persisted JSON fields use `validate_safe_json()` and freeform text fields use `validate_safe_text()` before persistence. Reviewed coverage includes company/contact/follow-up/product/calendar/email/link fields. Tests exercise calendar title, email subject, product summary, follow-up subject/priority, product-link relation, and company metadata secret rejection.

### E2E/manual evidence

Approved with caveat.

The verification doc records:

- Browser preview E2E for Phase 6 Business safe UI/no fake records/safe command failure copy.
- Native Tauri launch/process/window/schema evidence: `target/debug/zoid`, window `Zoid`, schema version 12, and Phase 6 tables present.

The caveat is truthful and non-blocking: no full visual native click-through was performed because GUI automation was unavailable. Given the local-first implementation is covered by backend persistence tests, frontend tests, build, browser preview, and native launch/schema evidence, this is acceptable for this phase.

## Non-blocking caveats

- EventKit and Gmail remain intentionally blocked/unconfigured. Phase 6 is approved only as a truthful local-first surface, not as production external Calendar/Gmail integration.
- Native GUI click-through remains limited by unavailable automation; evidence is process/window/schema plus automated tests and browser preview.
- Product link targets are validated by the existing entity-link allowed-type framework, but some target IDs can still represent future/opaque entities depending on broader app schema maturity. This is acceptable for current Phase 6 scope because the source product existence is enforced and persistence is tested.

## Final verdict

Verdict: APPROVED

No required fixes found for the current Phase 6 implementation.