# Content Workspace Autonomous Social Automation Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Do not mark complete until feature-critique-workflow approves `.hermes/reviews/content-autonomous-social-automation/critique-report.md`.

**Goal:** Integrate the currently working MaVoid daily Hermes cron automation into Zoid’s Content and Automations workspaces so Zoid can run, observe, approve by policy, verify, and recover a fully autonomous two-post-per-day OmniSocials workflow.

**Architecture:** Keep Hermes cron as the first execution engine because it already works: `MaVoid Daily Social Creator Scheduler` runs at `0 8 * * *`, loads `mavoid-social-design-workflow`, creates reviewed designs, and schedules through OmniSocials; `MaVoid OmniSocials Publish Monitor` checks publishing every 15 minutes. Zoid should not duplicate the scheduler initially. Zoid becomes the product control plane: campaign policy, content queue, run history, review evidence, OmniSocials schedule/publish verification, failure recovery, and manual override.

**Tech Stack:** Tauri + Rust SQLite services, React/TypeScript UI, existing Content Workspace Phase 5 models, new Automation/Provider service layer, Hermes cron metadata/run sync, OmniSocials verified API state.

---

## Current Context / Verified Facts

- Existing Hermes cron jobs:
  - `12fd35ec77e2` — `MaVoid Daily Social Creator Scheduler`, schedule `0 8 * * *`, next run `2026-06-06T08:00:00+03:00`, enabled, last status `ok`, skills include `mavoid-social-design-workflow`.
  - `9562e7cb93b6` — `MaVoid OmniSocials Publish Monitor`, schedule `every 15m`, enabled, script-only, last status `ok`.
- Existing Zoid Phase 5 deliberately excludes autonomous publishing and keeps OmniSocials fail-closed.
- Existing Content Workspace models support local plans, pieces, media assets, review gates, schedules, verification records, and OmniSocials status.
- Existing UI screen references already include:
  - Content: “Generate Today’s Content”, review queue, asset queue, publishing history.
  - Automations: automation list, upcoming schedule, run logs, pause/resume, retry, failure policy.

## Product Decision: Best Integration

### Recommended integration model

Use a three-layer model:

1. **Content Workspace = campaign and output control plane**
   - Defines the daily social program: cadence, platforms, post count, content pillars, brand/design policy, review policy, publish windows.
   - Shows today’s planned/generated/scheduled/published/failed posts.
   - Stores durable evidence for every generated piece, asset, review result, schedule, publish attempt, and provider verification.

2. **Automations Workspace = recurring job control plane**
   - Shows the Hermes cron jobs as first-class Zoid automations.
   - Supports run now, pause/resume, schedule view, linked content plan, recent runs, logs, and failure policy.
   - Does not expose secrets.

3. **Hermes cron + OmniSocials = execution providers**
   - Hermes creates the content and calls the specialist designer/reviewer agents.
   - OmniSocials owns real social scheduling/publishing.
   - Zoid syncs from providers and records verification, rather than pretending local state equals provider success.

### Why this is best

- It preserves the working cron path instead of rebuilding it prematurely.
- It gives you full autonomy without losing auditability.
- It keeps Phase 5’s fail-closed truthfulness while adding a separate, explicit “autonomous mode enabled” policy boundary.
- It lets Zoid eventually swap Hermes cron for an internal scheduler without changing the Content UI or data model.

## Autonomy Policy

Full autonomy should mean:

- No human approval required between daily generation and scheduling/publishing once the automation policy is enabled.
- Specialist designer and independent reviewer approval are still mandatory, but they are agent gates, not human gates.
- Publishing is allowed only when all gates pass:
  - automation enabled;
  - campaign policy active;
  - OmniSocials credentials verified;
  - designer output exists;
  - reviewer verdict is `APPROVED`;
  - platform constraints pass;
  - OmniSocials create/schedule response verified by direct provider read-back;
  - publish monitor verifies final posted/published state.
- Fail closed on missing assets, reviewer rejection, unverifiable OmniSocials response, stale credentials, duplicate post risk, or policy disabled.

## Data Model Changes

### Task 1: Add automation provider tables

**Objective:** Represent Hermes cron jobs in Zoid without storing secrets.

**Files:**
- Modify: `src-tauri/src/phase5_service.rs` or new `src-tauri/src/automation_service.rs`
- Modify: `src-tauri/src/lib.rs`
- Test: `src-tauri/src/tests.rs`

**Schema to add:**

- `automation_providers`
  - `id`
  - `provider_type` (`hermes_cron`)
  - `display_name`
  - `state` (`connected`, `needs_permission`, `disabled`, `error`)
  - `credential_ref` nullable, never raw secret
  - `metadata_json`
  - `created_at`, `updated_at`

- `automation_jobs`
  - `id`
  - `provider_id`
  - `external_job_id` (`12fd35ec77e2`, `9562e7cb93b6`)
  - `workspace_id` (`content`)
  - `name`
  - `job_kind` (`daily_creator_scheduler`, `publish_monitor`)
  - `schedule_expr`
  - `timezone`
  - `enabled`
  - `state`
  - `last_run_at`
  - `next_run_at`
  - `last_status`
  - `prompt_preview`
  - `metadata_json`

- `automation_runs`
  - `id`
  - `job_id`
  - `external_run_id` nullable
  - `started_at`
  - `finished_at` nullable
  - `trigger_type` (`scheduled`, `manual`, `retry`, `catchup`)
  - `status` (`running`, `ok`, `failed`, `blocked`, `partial`)
  - `summary`
  - `log_ref` nullable
  - `metadata_json`

- `automation_run_links`
  - `id`
  - `run_id`
  - `entity_type` (`content_plan`, `content_piece`, `media_asset`, `content_schedule`, `content_verification`)
  - `entity_id`

**Test first:** Assert tables and indexes exist.

### Task 2: Add campaign automation policy table

**Objective:** Make autonomy explicit and reversible.

**Files:**
- Modify: `src-tauri/src/phase5_service.rs` or new `src-tauri/src/content_automation_service.rs`
- Test: `src-tauri/src/tests.rs`

**Schema:**

- `content_automation_policies`
  - `id`
  - `content_plan_id`
  - `name`
  - `state` (`draft`, `enabled`, `paused`, `disabled_by_policy`)
  - `daily_post_count` default `2`
  - `creator_job_id`
  - `publish_monitor_job_id`
  - `timezone` default `Africa/Cairo`
  - `generation_time_local` default `08:00`
  - `publish_windows_json`
  - `platforms_json`
  - `requires_agent_review` boolean default `true`
  - `requires_human_approval` boolean default `false` for autonomous mode
  - `failure_policy_json`
  - `created_at`, `updated_at`

**Acceptance:** Zoid can display “Autonomous MaVoid Daily Social: enabled, 2 posts/day, 08:00 generation, publish monitor every 15m.”

### Task 3: Add provider verification fields to existing content records

**Objective:** Separate local intent from real provider success.

**Files:**
- Modify: `src-tauri/src/phase5_service.rs`
- Modify: `src/contentWorkspace.ts`
- Test: `src-tauri/src/tests.rs`, `src/contentWorkspace.test.ts`

**Add/extend:**

- `content_schedules.provider_schedule_id`
- `content_schedules.provider_post_id`
- `content_schedules.provider_permalink`
- `content_schedules.provider_checked_at`
- `content_schedules.provider_state`
- `content_verification_records.provider_response_ref`
- `media_assets.provider_media_id`
- `media_assets.rendered_asset_path`
- `media_assets.review_report_path`

**Acceptance:** UI can say “local scheduled” vs “OmniSocials scheduled and verified” vs “published and verified.”

## Service/API Plan

### Task 4: Create automation list/read commands

**Objective:** Feed Automations workspace with real recurring jobs.

**Commands:**

- `list_automation_jobs_command({ workspace_id?: string })`
- `read_automation_job_command({ job_id })`
- `list_automation_runs_command({ job_id, limit })`
- `read_automation_run_command({ run_id })`

**Files:**
- Create/modify: `src-tauri/src/automation_service.rs`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/App.tsx`
- Create: `src/automationWorkspace.ts`
- Create: `src/automationWorkspace.test.ts`

**Tests:**
- Default seeded Hermes jobs appear.
- Jobs link to Content workspace.
- No credential/raw prompt secrets exposed.

### Task 5: Add Hermes cron provider sync

**Objective:** Import live Hermes cron metadata into Zoid.

**Implementation options:**

- Preferred: local provider adapter that reads Hermes cron state through a safe command/API exposed by Hermes if available.
- Fallback: explicit “manual import/update” command that stores the known current job IDs and allows refresh later.

**Important:** Do not scrape secrets or raw full prompts into Zoid. Store only job ID, name, schedule, enabled, state, timestamps, status, skill list, and sanitized prompt preview.

**Commands:**

- `sync_hermes_cron_jobs_command()`
- `sync_hermes_cron_job_command({ external_job_id })`

**Acceptance:** After sync, Zoid shows the current 8 AM creator job and 15-minute monitor with accurate next/last run fields.

### Task 6: Add controlled run/pause/resume commands

**Objective:** Allow Zoid to operate the external automation safely.

**Commands:**

- `run_automation_job_now_command({ job_id })`
- `pause_automation_job_command({ job_id, reason })`
- `resume_automation_job_command({ job_id })`
- `retry_automation_run_command({ run_id })`

**Policy:**

- Mutating commands require explicit local confirmation in UI.
- “Run now” must mark trigger type as `manual` or `catchup`.
- Commands must record local run intent before provider call and final status after provider call.
- If provider call fails, record `failed` with failure report.

## Content Workflow Plan

### Task 7: Add autonomous daily social plan entity

**Objective:** Represent the MaVoid daily social program inside Content Workspace.

**Seed/default content plan:**

- Title: `MaVoid Daily Social Automation`
- Pillar: `enterprise AI automation + operational systems`
- Owner actor type: `autonomous_agent`
- Status: `active`
- Metadata:
  - `daily_post_count: 2`
  - `creator_job_external_id: 12fd35ec77e2`
  - `publish_monitor_external_id: 9562e7cb93b6`
  - `design_workflow: mavoid-social-design-workflow`
  - `review_required: agent_reviewer_approved`

**Acceptance:** The Content dashboard shows this as an active automation-backed plan.

### Task 8: Model generated daily outputs

**Objective:** Each daily run creates or syncs two `content_pieces` with assets, review gates, schedules, and verifications.

**Rules:**

- Each post gets one `content_piece`.
- Each visual/carousel file gets one or more `media_assets`.
- Designer/reviewer evidence maps to `content_review_gates`.
- OmniSocials schedule response maps to `content_schedules`.
- Provider read-back maps to `content_verification_records`.
- Duplicate prevention key: date + platform + slot + content hash + provider post id.

**Statuses:**

- `generated` after designer output exists.
- `review_ready` when sent to reviewer.
- `approved` when reviewer verdict is approved.
- `scheduled` only after provider schedule read-back passes.
- `published` only after publish monitor read-back passes.
- `blocked` or `failed` on any gate failure.

### Task 9: Keep design/review gate autonomous but mandatory

**Objective:** Full autonomy must not bypass quality.

**Rules from MaVoid workflow:**

- Designer agent must create real visual assets, not text-only placeholders.
- Reviewer must inspect rendered PNG/screenshot pixels, not only SVG/source.
- Reviewer verdict must be `APPROVED` before OmniSocials scheduling.
- If reviewer says `REQUEST_CHANGES`, the automation may self-fix and re-review up to a configured retry limit.
- If retry limit exceeded, block the post and surface failure in Zoid.

**Acceptance:** UI clearly says “agent-approved” not “human-approved.”

### Task 10: Add publish monitor sync

**Objective:** Convert OmniSocials publish monitor output into Zoid verification state.

**Rules:**

- The publish monitor must verify exact provider IDs through direct read-back.
- Local state must never infer published from time passing.
- Each published post should include provider status, platform, post ID, permalink if available, and checked timestamp.
- Failures should create action items or tasks.

## UI Plan

### Task 11: Update Content Dashboard

**Objective:** Make autonomy visible and controllable from Content.

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/contentWorkspace.ts`
- Modify CSS as needed in existing stylesheet

**Add sections:**

- “Autonomous Daily Social” card:
  - Enabled/paused state
  - Next generation: 08:00 Africa/Cairo
  - Today’s target: 2 posts
  - Designer/reviewer gate status
  - OmniSocials status
  - Last run status
  - Buttons: Run today now, Pause, View automation

- “Today’s Posts”:
  - Slot 1 and Slot 2
  - status: planned/generated/reviewed/scheduled/published/failed
  - media thumbnail/path
  - caption excerpt
  - provider IDs

- “Failure / Intervention Needed”:
  - reviewer rejection
  - missing credentials
  - OmniSocials verification mismatch
  - duplicate risk
  - publish monitor failure

### Task 12: Update Automations Workspace

**Objective:** Show Hermes cron as first-class automation records.

**Files:**
- Modify: `src/App.tsx`
- Create/modify: `src/automationWorkspace.ts`

**Add:**

- List row for `MaVoid Daily Social Creator Scheduler`.
- List row for `MaVoid OmniSocials Publish Monitor`.
- Detail tabs: Overview, Schedule, Runs, Logs, Linked Content, Failure Policy.
- Actions: Run Now, Pause/Resume, Retry Failed, View Linked Content.

### Task 13: Add settings/policy UI

**Objective:** Make full autonomy intentional.

**UI fields:**

- Enable autonomous posting toggle.
- Daily post count, default 2.
- Generation time, default 08:00.
- Publish windows.
- Platforms.
- Agent review required toggle locked on by default.
- Human approval required toggle off only after explicit user enablement.
- Failure behavior:
  - stop day on first failure;
  - continue other slot;
  - retry design fixes N times;
  - notify only vs create task.

**Safety copy:** “Autonomous mode can schedule/publish externally through OmniSocials after agent review approval and provider read-back verification.”

## Testing Plan

### Rust service tests

Add tests in `src-tauri/src/tests.rs`:

1. `automation_schema_has_provider_jobs_runs_and_links`
2. `default_mavoid_social_automation_jobs_are_seeded_or_imported`
3. `content_automation_policy_requires_explicit_enabled_state_for_external_publish`
4. `automation_run_links_to_generated_content_piece_assets_review_schedule`
5. `provider_verified_schedule_can_mark_content_scheduled`
6. `published_state_requires_provider_readback_verification`
7. `duplicate_provider_post_is_rejected_or_linked_idempotently`
8. `pause_policy_blocks_run_now_and_records_blocked_run`
9. `secrets_are_not_returned_by_automation_commands`

### TypeScript tests

Add/update:

- `src/contentWorkspace.test.ts`
- `src/automationWorkspace.test.ts`

Cover:

- Status copy distinguishes local intent, agent approved, provider scheduled, provider published.
- Full autonomy copy is explicit.
- Blocked/failed verification records surface correctly.
- Job rows render next/last run status.

### Manual/native verification

Run:

- `npm run test`
- `npm run verify:local`
- Tauri app smoke: Content workspace loads from real backend.
- Tauri app smoke: Automations workspace shows both cron jobs.
- Trigger a dry-run/safe run if supported.
- Confirm no fake publish state appears without OmniSocials provider read-back.

### Production-like E2E for autonomy

Before calling it fully autonomous:

1. Use a test/sandbox OmniSocials account or a clearly tagged test campaign if available.
2. Run creator job manually from Zoid.
3. Verify two content pieces appear in Content workspace.
4. Verify media files exist and reviewer reports are approved.
5. Verify OmniSocials has scheduled posts by provider GET/list read-back.
6. Let publish monitor run or trigger it safely.
7. Verify published state only after provider confirms.
8. Cleanup test posts if they are not intended to remain public.

## Implementation Order

1. Create failing Rust tests for automation schema and seeded/imported jobs.
2. Add automation provider/job/run/link tables.
3. Add default/import support for the two existing Hermes cron jobs.
4. Add list/read commands and TS types/tests.
5. Render Automations workspace rows and detail view.
6. Add content automation policy table and command surface.
7. Link MaVoid Daily Social plan to the two automation jobs.
8. Add content status fields for provider verified scheduled/published state.
9. Update Content Dashboard with autonomous daily card and today’s post slots.
10. Add run/pause/resume/retry commands with confirmation and failure records.
11. Add Hermes cron sync/run provider adapter.
12. Add OmniSocials verification sync mapping into content records.
13. Run local tests and native app verification.
14. Create `.hermes/reviews/content-autonomous-social-automation/handoff.md`.
15. Run feature critique workflow.
16. Fix all Required fixes.
17. Re-review until APPROVED.
18. Only then call the feature complete.

## Files Likely To Change

- `src-tauri/src/lib.rs`
- `src-tauri/src/phase5_service.rs`
- `src-tauri/src/automation_service.rs` (new, recommended)
- `src-tauri/src/tests.rs`
- `src/App.tsx`
- `src/contentWorkspace.ts`
- `src/contentWorkspace.test.ts`
- `src/automationWorkspace.ts` (new)
- `src/automationWorkspace.test.ts` (new)
- CSS/style files used by `App.tsx`
- `Docs/2026-06-05-phase-5-content-omnisocials-scope-plan.md` or a new Phase 9/autonomy scope doc
- `.hermes/reviews/content-autonomous-social-automation/handoff.md`

## Risks / Tradeoffs

- **Risk:** Directly embedding Hermes cron operation in Zoid may couple Zoid to one local Hermes profile.
  - **Mitigation:** Use provider abstraction and store external IDs, not raw config/secrets.

- **Risk:** “Full autonomy” can accidentally bypass quality or publish unverified posts.
  - **Mitigation:** Keep agent reviewer approval and provider read-back mandatory.

- **Risk:** Duplicate posts on retries/catchup.
  - **Mitigation:** Add idempotency keys and provider post ID linking.

- **Risk:** OmniSocials API shape or credentials are not fully known in Zoid.
  - **Mitigation:** Treat provider integration as adapter with blocked/needs_permission states until verified.

- **Risk:** Current Phase 5 docs say autonomous recurring publishing is excluded.
  - **Mitigation:** Implement as a new autonomy phase/slice, not as a silent Phase 5 behavior change.

## Open Questions

1. Should Zoid operate Hermes cron directly, or only observe/sync it in the first slice?
   - Recommended: observe/sync first, then add run/pause/resume once display is verified.

2. Should autonomous mode default to live publishing or scheduled drafts?
   - Recommended for this specific workflow: live scheduling is allowed because you explicitly requested full autonomy, but require agent reviewer approval and OmniSocials read-back.

3. Which platforms are in scope for the first autonomous run?
   - Recommended: mirror the current working cron/OmniSocials configuration exactly; do not expand platforms during integration.

4. Where should run logs live?
   - Recommended: store compact summaries in SQLite and log file refs/paths, not full raw agent transcripts.

## Definition of Done

- Zoid shows both current recurring jobs with accurate schedule/enabled/last/next status.
- Content Workspace shows the MaVoid Daily Social automation plan and today’s two-post pipeline.
- Autonomous mode policy is explicit and can be paused.
- Agent design/review approval is mandatory before scheduling/publishing.
- OmniSocials scheduled/published states require provider read-back verification.
- Failures create durable verification/failure records and are visible in Content and Automations.
- `npm run test` passes.
- `npm run verify:local` passes.
- Tauri native app smoke verifies Content and Automations UI against real backend state.
- Feature critique workflow returns `APPROVED`.
