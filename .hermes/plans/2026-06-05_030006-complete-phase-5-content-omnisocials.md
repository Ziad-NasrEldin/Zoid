# Complete Phase 5 Content and OmniSocials Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan as one controlled Phase 5 bulk session, then run one combined feature-critique-workflow loop before calling Phase 5 complete.

**Goal:** Complete every Phase 5 tracker item P5.01-P5.20 for Zoid AI OS in one session, building the Content + OmniSocials slice as draft-first, fail-closed, review-gated, and locally verified.

**Architecture:** Preserve Zoid’s local-first Tauri + React + Rust + SQLite architecture. Treat the already-present Phase 5 source as partial/unapproved scratch: reconcile it against the tracker, improve it where needed, then formally verify and review the whole phase. OmniSocials external writes must remain fail-closed unless real credentials/integration are explicitly configured and verified; this session should not publish anything.

**Tech Stack:** Tauri, Rust, SQLite migrations, React/TypeScript, existing Zoid action policy / confirmation / redaction / event / entity-link services.

---

## Current context

Source references:
- Tracker: `Docs/2026-06-01-zoid-implementation-tracker.md`, lines P5.01-P5.20.
- Plan: `Docs/2026-05-31-zoid-implementation-plan-v1.md`, Phase 5 section.
- Current Phase 5 partial files:
  - `src-tauri/migrations/0011_phase5_content_omnisocials.sql`
  - `src-tauri/src/phase5_service.rs`
  - Phase 5 command wiring in `src-tauri/src/lib.rs`
  - Basic Content workspace UI in `src/App.tsx`
  - Phase 5 tests in `src-tauri/src/tests.rs`

Current verified state before writing this plan:
- `npm run verify:local` passed.
- Rust: 176 passed / 0 failed / 1 ignored.
- Frontend tests passed.
- Frontend build passed.
- Phase 5 has partial tests: schema, draft/asset/review/schedule flow, OmniSocials fail-closed verification.
- Phase 5 tracker remains fully unchecked and has no handoff/critique approval.

Non-negotiables:
- No real publish/schedule/upload should happen by default.
- No fake OmniSocials connected state.
- No schedule/publish without required specialist review + human confirmation.
- Media/platform constraints must be enforced before schedule/publish attempts.
- All meaningful actions should write events or verification/failure records.
- Secrets must be rejected/redacted from metadata, logs, events, UI errors, and failure reports.
- Phase 5 is not complete until `.hermes/reviews/phase-5-content-omnisocials/critique-report.md` says `Verdict: APPROVED`.

---

## One-session execution strategy

Use a single bulk implementation session with strict sequencing:

1. Reconcile current source and tracker.
2. Finish backend/database/service gaps.
3. Finish Tauri bridge gaps.
4. Finish frontend Content workspace workflow UI.
5. Add/strengthen tests.
6. Run local/manual verification.
7. Update tracker P5.01-P5.20 truthfully.
8. Write Phase 5 handoff.
9. Trigger critique, fix Required fixes, re-review until approved.
10. Commit only after approval and final verification.

Because `src/App.tsx`, `src-tauri/src/lib.rs`, and `src-tauri/src/tests.rs` are shared high-conflict files, implementation agents must be serialized for edit lanes. Parallel work is allowed only for read-only discovery/review lanes.

---

## Batch lanes for one-session completion

### Lane A — Source reconciliation and exact scope lock

**Objective:** Establish live state and prevent stale Phase 4/5 contamination before implementation.

**Files:**
- Read: `Docs/2026-06-01-zoid-implementation-tracker.md`
- Read: `Docs/2026-05-31-zoid-implementation-plan-v1.md`
- Read: `.hermes/reviews/phase-4-code-repos-launch-gate/critique-report.md`
- Inspect: `src-tauri/migrations/0011_phase5_content_omnisocials.sql`
- Inspect: `src-tauri/src/phase5_service.rs`
- Inspect: `src-tauri/src/lib.rs`
- Inspect: `src-tauri/src/tests.rs`
- Inspect: `src/App.tsx`

**Steps:**
1. Run `git status --short --branch`.
2. Confirm P4.22 is stale because Phase 4 critique is already APPROVED; do not mix P4 tracker fix into Phase 5 commit unless it is a tiny docs-only reconciliation.
3. Build a current Phase 5 inventory:
   - database tables present
   - backend services present
   - bridge commands present
   - frontend UI present
   - tests present
   - missing tracker items
4. Decide final Phase 5 scope: complete exactly P5.01-P5.20, no external publish execution.
5. Record the inventory in the Phase 5 handoff later.

**Verification:**
- `npm run verify:local` baseline is known passing before edits.
- No code changed in this lane.

---

### Lane B — P5.01 planning/spec document

**Objective:** Create the formal draft-first/fail-closed publishing scope and review-gate spec required by P5.01.

**Files:**
- Create: `Docs/2026-06-05-phase-5-content-omnisocials-scope-plan.md`

**Spec contents:**
- Content lifecycle states: `plan`, `draft`, `review_ready`, `approved`, `schedule_intent`, `blocked`, `failed`, `cancelled`, and explicitly no automatic publish.
- Content entities: plan, piece, media asset, review gate, schedule intent, verification record, platform status.
- Review gates: specialist design/review required by default for MaVoid social content.
- Confirmation gates: schedule/publish/upload require human confirmation and action-policy preview.
- OmniSocials truth states: `not_configured`, `needs_permission`, `connected`, `error`, `disabled_by_policy` where supported; current default is `not_configured`.
- Explicit exclusions: autonomous publishing, credential setup, real platform upload/publish, external account mutation, broad analytics, recurring content automation.
- Manual verification acceptance: create content piece, add asset ref, approve review gate, create schedule intent with confirmation, verify fail-closed OmniSocials action record without publishing.

**Verification:**
- Spec exists and matches tracker P5.01.

---

### Lane C — Database and backend hardening for P5.02-P5.08

**Objective:** Complete and harden Phase 5 SQLite schema/services beyond the current partial implementation.

**Files:**
- Modify: `src-tauri/migrations/0011_phase5_content_omnisocials.sql`
- Modify: `src-tauri/src/phase5_service.rs`
- Modify: `src-tauri/src/tests.rs`

**Required improvements to verify/add:**
1. P5.02 schema:
   - `content_plans`
   - `content_pieces`
   - `media_assets`
   - `content_review_gates`
   - `content_schedules`
   - `content_verification_records`
   - `omnisocials_accounts` or platform statuses
   - indexes for plan/piece/status/schedule/platform lookup
   - valid FK constraints and state CHECK constraints
2. P5.03 workflow:
   - plan -> draft -> asset -> review -> schedule intent.
   - schedule intent, not external platform schedule, unless explicitly connected and verified later.
3. P5.04 media constraints:
   - store references only, not raw media blobs.
   - enforce basic platform constraints: Instagram/TikTok need media; LinkedIn can allow text-only; byte/dim metadata bounds where possible.
4. P5.05 review gates:
   - specialist review gate required before schedule when `required_gate = specialist_review`.
   - rejected/pending gate blocks schedule.
5. P5.06 integration status:
   - truthful OmniSocials status with default `not_configured`.
   - no fake connected state.
   - no credential value in SQLite.
6. P5.07 fail-closed external surfaces:
   - upload/schedule/publish commands create blocked verification records when not configured.
   - connected-but-not-execution-ready should still avoid external side effects unless implementation is real and reviewed.
7. P5.08 verification/failure records:
   - every blocked/failed upload/schedule/publish attempt records outcome, platform, action type, failure report, and optional schedule/piece link.
8. Events/entity links:
   - create meaningful events for plan created, piece created/updated, asset referenced, gate created/approved/rejected, schedule intent created/cancelled, OmniSocials action blocked/failed.
   - link content pieces/assets/schedules to tasks/products/files where available only if validated.
9. Secret safety:
   - reject/redact secret-like metadata/body/failure details where applicable.
   - at minimum reject obvious secret metadata and avoid raw credentials in status notes.

**Focused tests to add/strengthen:**
- `p502_phase5_schema_constraints_indexes_and_fk_behavior`
- `p503_phase5_content_draft_asset_review_schedule_flow_is_draft_first`
- `p504_phase5_omnisocials_fails_closed_and_records_failure`
- `p505_phase5_schedule_blocks_pending_or_rejected_review_gate`
- `p506_phase5_secret_metadata_is_rejected_or_redacted`
- `p507_phase5_platform_constraints_enforced`
- `p508_phase5_events_and_verification_records_are_written`

**Focused commands:**
- `cargo test --manifest-path src-tauri/Cargo.toml p50 -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml phase5 -- --nocapture` if naming supports it.

---

### Lane D — Tauri bridge completion for P5.09-P5.10

**Objective:** Ensure native commands expose the completed backend safely and truthfully.

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src-tauri/src/tests.rs`

**Commands that should exist and be tested:**
- `create_content_plan_command`
- `list_content_plans_command`
- `create_content_piece_command`
- `read_content_piece_command` if missing and needed by UI
- `list_content_pieces_command`
- `update_content_piece_draft_command`
- `add_media_asset_reference_command`
- `list_media_asset_references_command` if missing and needed by UI
- `create_content_review_gate_command`
- `approve_content_review_gate_command`
- `reject_content_review_gate_command`
- `list_content_review_gates_command`
- `create_content_schedule_command`
- `list_content_schedules_command`
- `cancel_content_schedule_command`
- `get_omnisocials_status_command`
- `omnisocials_upload_media_command`
- `omnisocials_schedule_content_command`
- `omnisocials_publish_content_command`
- `list_content_verification_records_command`
- Optional if needed: `preview_content_action_policy_command`

**Bridge rules:**
- Preserve backend validation.
- Return structured, redacted errors.
- Do not expose any command that performs real external write without reviewed credential/config path.
- Command names must be registered in `TAURI_BRIDGE_COMMAND_NAMES` and `generate_handler!`.

**Tests:**
- command-surface registration test includes every P5 command.
- bridge command test for plan -> draft -> gate -> confirmed schedule intent.
- bridge command test for OmniSocials upload/schedule/publish fail-closed records.

**Focused command:**
- `cargo test --manifest-path src-tauri/Cargo.toml p5 -- --nocapture`

---

### Lane E — Frontend Content workspace completion for P5.11-P5.12

**Objective:** Replace the current mostly read-only Content workspace with a usable, truthful workflow surface.

**Files:**
- Prefer create/extract to reduce `App.tsx` risk:
  - Create: `src/contentWorkspace.ts`
  - Create: `src/contentWorkspaceView.tsx`
  - Create: `src/contentWorkspace.test.ts`
  - Modify: `src/App.tsx`
  - Modify: `src/App.css` if needed

**UI requirements:**
1. Content calendar/workspace:
   - list plans
   - list pieces
   - selected piece detail
   - empty/loading/error states
   - clear native bridge failure state
2. Draft workflow:
   - create plan form or at least native-backed create controls
   - create content piece/draft form
   - update draft body/status
   - no fake drafts
3. Assets:
   - add media asset reference form
   - show asset refs and platform constraints
4. Review gates:
   - create specialist review gate
   - approve/reject gate with evidence summary
   - show pending/approved/rejected state
5. Scheduling:
   - create schedule intent only after review + confirmation ID.
   - if confirmation UI primitive is not enough for full native confirmation creation, provide truthful blocked state and use existing confirmation decision command if available.
   - never claim actual external platform schedule happened.
6. OmniSocials status/actions:
   - show `not_configured` truthfully.
   - upload/schedule/publish buttons should either be disabled with reason or perform fail-closed native verification record creation, not external writes.
   - show failure reports and verification records.
7. Copy constraints:
   - Draft-first.
   - No publish-by-default.
   - No fake connected state.
   - No internal provider/secret details.

**Frontend tests:**
- `contentWorkspace.test.ts` should cover view-model state:
  - loading/empty/error
  - draft pieces render from native data
  - review pending blocks schedule CTA copy
  - approved review + confirmation shows schedule-intent-ready state
  - OmniSocials `not_configured` disables or fail-closes external action copy
  - no copy claims publish succeeded unless verification says passed/real
- Add bridge integration test if needed:
  - content workspace invokes the right command names and handles errors.

**Commands:**
- `npx tsx src/contentWorkspace.test.ts`
- `npm run test:frontend`
- `npm run build`

---

### Lane F — Full Phase 5 regression and manual/native verification P5.13-P5.18

**Objective:** Prove Phase 5 works locally without publishing.

**Automated tests:**
- P5.13 workflow progression plan -> draft -> asset -> review -> schedule intent.
- P5.14 failed generation/review/upload/schedule fails closed.
- P5.15 no publish/schedule without required review/confirmation.
- P5.16 platform media constraints enforced.

**Manual/native verification doc:**
- Create: `Docs/2026-06-05-phase-5-content-omnisocials-manual-verification.md`

**Manual/native verification steps:**
1. Launch native app with `npm run tauri:dev` only if needed for UI/native invoke evidence.
2. Create a content plan and draft through native commands or UI.
3. Add a media asset reference.
4. Create a specialist review gate.
5. Verify schedule is blocked before approval/confirmation.
6. Approve review gate.
7. Create/persist confirmation decision.
8. Create schedule intent.
9. Attempt OmniSocials upload/schedule/publish in unconfigured state and verify blocked verification/failure records, not external publish.
10. Restart/reopen SQLite connection or app and verify rows persist.
11. Browser preview smoke if native UI automation is blocked; document limitation truthfully.

**Required commands:**
- `cargo test --manifest-path src-tauri/Cargo.toml p5 -- --nocapture`
- `npm run test:frontend`
- `npm run build`
- `npm run verify:local`
- `git diff --check`
- `git status --short --branch`

---

### Lane G — Tracker and handoff closeout P5.19-P5.20

**Objective:** Close Phase 5 only after local verification, then run critique loop.

**Files:**
- Modify: `Docs/2026-06-01-zoid-implementation-tracker.md`
- Create: `.hermes/reviews/phase-5-content-omnisocials/handoff.md`
- Critique creates/updates: `.hermes/reviews/phase-5-content-omnisocials/critique-report.md`

**Tracker update rules:**
- Mark P5.01-P5.18 complete only when exact evidence exists.
- Mark P5.19 complete after handoff is written.
- Leave P5.20 pending until critique report says `Verdict: APPROVED`.
- After approval, mark P5.20 complete with report path and final verification command output.

**Handoff must include:**
- Original request: complete Phase 5 Content and OmniSocials.
- Implementation summary.
- Changed files.
- Tests run with exact pass counts.
- Manual verification notes.
- Known limitations: no real external publishing, no configured OmniSocials credentials unless explicitly added, fail-closed only.
- Reviewer focus areas:
  - draft-first lifecycle
  - review + confirmation gates
  - platform constraints
  - no fake connected/published states
  - verification/failure record behavior
  - frontend truthfulness
  - secrets/redaction

**Critique loop:**
1. Trigger/wait for critique.
2. If `REQUEST_CHANGES`, fix every Required fix.
3. Update handoff with fix cycle notes and actual rerun commands.
4. Re-trigger critique.
5. Repeat until `APPROVED`.
6. Run final `npm run verify:local && git diff --check && git status --short --branch` after any final tracker/handoff edits.

---

## Proposed in-session todo list

1. P5-live-state: Reconcile git/source/tracker/review state.
2. P5-spec: Write Phase 5 scope/spec doc.
3. P5-backend-db: Harden schema/services/events/verification records.
4. P5-bridge: Complete Tauri command surface and bridge tests.
5. P5-frontend: Complete Content workspace workflow UI and tests.
6. P5-regression: Run focused backend/frontend tests and full local verification.
7. P5-manual: Write manual/native verification doc.
8. P5-tracker-handoff: Update tracker and write handoff.
9. P5-critique: Run critique loop until APPROVED.
10. P5-final: Final verification and commit/report.

---

## Files likely to change

Docs/review:
- `Docs/2026-06-05-phase-5-content-omnisocials-scope-plan.md`
- `Docs/2026-06-05-phase-5-content-omnisocials-manual-verification.md`
- `Docs/2026-06-01-zoid-implementation-tracker.md`
- `.hermes/reviews/phase-5-content-omnisocials/handoff.md`
- `.hermes/reviews/phase-5-content-omnisocials/critique-report.md`

Backend/native:
- `src-tauri/migrations/0011_phase5_content_omnisocials.sql`
- `src-tauri/src/phase5_service.rs`
- `src-tauri/src/lib.rs`
- `src-tauri/src/tests.rs`

Frontend:
- `src/App.tsx`
- `src/App.css`
- `src/contentWorkspace.ts`
- `src/contentWorkspaceView.tsx`
- `src/contentWorkspace.test.ts`

---

## Risk controls

1. One-session scope risk
   - Complete as one bulk, but do not skip critique or verification.
   - If context/tool limit hits, stop with exact current lane, commands run, required fixes, and pending tracker items.

2. Real publishing risk
   - Keep OmniSocials external actions fail-closed unless real credentials/config and reviewed execution path exist.
   - Do not attempt to publish/schedule externally in this phase.

3. Shared-file conflict risk
   - Serialize edits to `App.tsx`, `lib.rs`, and `tests.rs`.
   - Prefer extracted frontend modules.

4. Stale tracker/review risk
   - Re-read tracker and critique report before final claim.
   - Do not mark P5.20 complete until critique says APPROVED.

5. Fake success risk
   - UI must say schedule intent / blocked verification record, not “published” or “scheduled on platform” unless actual provider success exists.

---

## Definition of done for Phase 5

Phase 5 is complete only when all are true:

- P5.01-P5.20 are checked in `Docs/2026-06-01-zoid-implementation-tracker.md` with evidence.
- Phase 5 scope/manual verification docs exist.
- Backend schema/services/bridge/frontend are wired.
- Content plan -> draft -> asset -> specialist review -> confirmed schedule intent works locally.
- OmniSocials unconfigured upload/schedule/publish fails closed and records verification/failure evidence.
- No schedule/publish without review + confirmation.
- Platform constraints are enforced.
- Relevant events/verification records are persisted.
- Frontend has truthful empty/loading/error/blocked states.
- `npm run verify:local` passes.
- `git diff --check` passes.
- `.hermes/reviews/phase-5-content-omnisocials/critique-report.md` says `Verdict: APPROVED`.
- Final report separates: implemented, verified, deferred, blockers, and commit status.

---

## Recommended execution command

When ready to execute, start with:

```bash
git status --short --branch
npm run verify:local
```

Then proceed through the lanes above. Do not call Phase 5 done until the critique approval gate passes.
