# Zoid Autonomous Content Automation Implementation Tracker

Date: 2026-06-05
Source discovery: `/Users/ziadnasreldin/brainstorms/2026-06-05-zoid-autonomous-content-automation.md`
Related docs: `2026-06-05-stitch-ai-autonomous-content-automation-screens-prompt.md`, `../../2026-06-05-phase-5-content-omnisocials-scope-plan.md`, `../../2026-06-01-zoid-implementation-tracker.md`

Purpose: concise execution tracker for the full V1 autonomous multi-brand content system. Status legend: `[ ]` pending, `[~]` in progress, `[x]` complete, `[!]` blocked.

Global gates for every phase:
- [ ] Real data or truthful empty/unconfigured states only; no fake connected, scheduled, or published state.
- [ ] Consequential actions fail closed unless policy, confirmation, reviewer confidence, credentials, fallback window, and provider verification allow them.
- [ ] Raw secrets never enter prompts, logs, SQLite records, artifacts, exports, notifications, or UI copy; macOS Keychain stores secrets, SQLite stores credential references only.
- [ ] Meaningful actions write durable events/evidence and link Content, Automation, Agent Run, Review, Notification, and OmniSocials entities.
- [ ] Routine verification passes with `npm run verify:local`, focused Rust/frontend tests, `git diff --check`, native/macOS notes where automation cannot verify, and feature critique approval.

Likely primary files:
- Backend/native: `src-tauri/src/lib.rs`, new `src-tauri/src/autonomous_content*.rs`, scheduler/job runner modules, `src-tauri/migrations/00xx_autonomous_content*.sql`, Keychain/native notification/email helpers.
- Frontend: `src/App.tsx`, `src/contentWorkspace.ts`, `src/contentWorkspace.test.ts`, new autonomous content view-model/components/tests, shared agent/automation/notification panels.
- Evidence/review: `Docs/*autonomous-content*`, `.hermes/reviews/autonomous-content-automation/*`, app-support logs/assets/evidence folders.

Verification command set:
- Focused backend: `cargo test --manifest-path src-tauri/Cargo.toml autonomous_content -- --nocapture`
- Focused frontend: `npm run test:frontend -- autonomous` or project-appropriate focused test invocation
- Full local: `npm run verify:local && git diff --check`
- Native/manual: `npm run tauri:dev`, inspect `~/Library/Application Support/Zoid/zoid.sqlite`, app-support logs/evidence, desktop notification behavior, Keychain item refs, and truthful UI states.
- Launch E2E: MaVoid dry test, then real two-slot MaVoid E2E through Zoid-owned scheduler, agents, OmniSocials, verification, notifications, and recovery.

---

## Phase 0 — PRD, Spec, and Release Gates

- [ ] A0.01 Write PRD/spec for full V1 scope: multi-brand, Zoid-owned scheduler, Agents Workspace linkage, credentials, OmniSocials, editor, notifications, recovery, autonomy controls, evidence/audit trail.
- [ ] A0.02 Define non-negotiable launch gate: real MaVoid E2E, multi-brand architecture present, no requirement for multiple live brands on day one.
- [ ] A0.03 Define done criteria for dry test, Run Now, recovery, evidence retention, native verification, tests, and critique approval.
- [ ] A0.04 Reconcile with existing Phase 5 draft-first OmniSocials scope; this tracker supersedes future autonomous publishing work but preserves fail-closed behavior.
- Likely files: `Docs/*autonomous-content*`, existing implementation tracker, `.hermes/reviews/autonomous-content-automation/`.
- Gate: spec reviewed before schema or scheduler edits.

## Phase 1 — Schema and Domain Model

- [ ] A1.01 Add schema for Brand -> Campaign -> Slots -> Content Pieces -> Platform Adaptations -> Schedules/Posts.
- [ ] A1.02 Add campaign templates, brand defaults, approval policy, notification policy, recovery policy, retention policy, and probation/trusted state.
- [ ] A1.03 Add pipeline stage records for planning, research, copy, design, review, adapt, schedule, publish, verify, recover.
- [ ] A1.04 Add evidence/artifact tables for assets, captions, review reports, OmniSocials IDs, verification records, status timelines, linked logs, and retention/archive metadata.
- [ ] A1.05 Add provider connections, platform accounts, brand-account mappings, credential references, permission status, and last verified timestamps without raw secrets.
- [ ] A1.06 Add events/entity links for campaign/slot/content/adaptation/schedule/post/agent-run/automation/notification/review/evidence relationships.
- Likely files: `src-tauri/migrations/00xx_autonomous_content.sql`, backend domain models/services/tests.
- Gate: migration idempotence, FK/check constraints, secret-field rejection, and file-backed SQLite reopen tests pass.

## Phase 2 — Zoid-Owned Scheduler and Job Runner

- [ ] A2.01 Implement scheduler model with generation time, review deadline, publish windows, latest safe fallback, timezone/language, enabled/paused/autonomous states.
- [ ] A2.02 Implement job runner and retry engine owned by Zoid, not Hermes cron; preserve current cron behavior only as reference.
- [ ] A2.03 Implement per-slot independence: successful slots continue when another slot fails.
- [ ] A2.04 Implement fallback blocking and notification when approval/scheduling misses latest safe fallback.
- [ ] A2.05 Mirror technical jobs/runs in Automations Workspace while keeping Content Workspace as source of truth.
- [ ] A2.06 Add scheduler logs, run history, pause/resume, retry failed, and Run Now entry points.
- Likely files: scheduler/job modules, automation mirror UI/components, Tauri commands.
- Gate: deterministic scheduler tests for MaVoid defaults: generate 8:00 AM, review deadline 10:00 AM, slot 1 11:00 AM-1:00 PM, slot 2 5:00 PM-7:00 PM, fallback 8:30 PM.

## Phase 3 — Agent Profiles and Pipeline Orchestration

- [ ] A3.01 Reuse Agents Workspace; do not create a separate agent runner.
- [ ] A3.02 Create/use first-class reusable profiles: Content Strategist/Planner, Researcher, Caption/Copy Agent, Social Designer, Design Reviewer, Publisher/OmniSocials Agent, Verification Agent, Recovery Agent.
- [ ] A3.03 Define handoff contracts, input/output schemas, context attachments, permission previews, linked entities, review records, and logs for each stage.
- [ ] A3.04 Orchestrate agent runs for strategy, research, copy, design, review, platform adaptation, publish, verify, recovery.
- [ ] A3.05 Surface linked Agent Run panels everywhere: dashboard, pipeline, content detail, recovery center, automation run detail.
- Likely files: agent profile seed/services, pipeline orchestrator, linked panel frontend tests.
- Gate: each pipeline stage has a linked Agent Run, status, output summary, evidence link, retry count, and review/attention state.

## Phase 4 — Brand and Campaign Templates

- [ ] A4.01 Implement hybrid Brand containers inside Content Workspace with voice, visual identity, pillars, banned claims/styles, audience, offers, accounts, default agents/templates, assets, approval, timezone/language, notifications.
- [ ] A4.02 Implement Autonomous Content Campaign template plus MaVoid Daily Two-Post template.
- [ ] A4.03 Implement wizard plus advanced editor: brand, template, slots/cadence, platforms/accounts, context, agents, schedule, approval, notifications, dry test, autonomy enablement.
- [ ] A4.04 Implement advanced freeform overrides for slot briefs, stage prompts, platform rules, retry counts, timeout budgets, pipeline chain, and context assets.
- [ ] A4.05 Add validation/diff impact preview for changes affecting future slots, pending schedules, automation jobs, or agent runs.
- Likely files: content workspace view models/components, template schema/services, wizard/editor tests.
- Gate: first runs of new brand/campaign/template require human approval until first 3 successful runs or manual trusted override.

## Phase 5 — Content Pipeline and Approval Safety

- [ ] A5.01 Implement source priority: manual slot brief, brand rules/banned claims, calendar topic, attached context, recent posts avoidance, fresh research/news, agent creative judgment.
- [ ] A5.02 Implement one core content piece per slot with platform-specific captions, formats, assets, schedule/post records.
- [ ] A5.03 Implement confidence gate: auto-publish only when reviewer verdict approved, score >= 85/100, no uncertainty/design/readability issue, probation complete, credentials verified.
- [ ] A5.04 Implement editor/override phases: pre-schedule normal edit, scheduled-not-published update/cancel/reschedule in OmniSocials + verify, published correction/repost workflow.
- [ ] A5.05 Implement approval-needed queue for low confidence, factual uncertainty, design issue, probation, credential/provider mismatch, fallback risk.
- [ ] A5.06 Implement content detail tabs: Brief, Draft, Assets, Captions, Platform Adaptations, Review, Publishing, Verification, Evidence, History.
- Likely files: content services, approval/review logic, editor UI, queue UI, tests.
- Gate: no silent publish mutation and no auto-publish unless all fail-closed gates pass.

## Phase 6 — Platform Adaptations and OmniSocials Integration

- [ ] A6.01 Implement platform adaptation records/rules for LinkedIn, Instagram, X, TikTok, Facebook, Threads; include constraints and status per platform.
- [ ] A6.02 Implement OmniSocials provider connections, account/page discovery state, permission verification, brand-account mapping, provider IDs, provider mismatch detection.
- [ ] A6.03 Implement scheduling/publishing/update/cancel flows through OmniSocials with policy enforcement and verification records.
- [ ] A6.04 Implement partial-failure behavior: publish/schedule successful adaptations, block failed ones, notify, and preserve evidence.
- [ ] A6.05 Implement account settings UI with Keychain credential reference copy and no raw secret display.
- Likely files: OmniSocials service/bridge, account settings UI, platform adapters, verification tests.
- Gate: unconfigured or unverified OmniSocials always fails closed; configured path must show real provider IDs and verification state.

## Phase 7 — Keychain Credentials and Secure Integration Storage

- [ ] A7.01 Implement macOS Keychain create/read/update/delete/test path for provider tokens/secrets, guarded by explicit user action.
- [ ] A7.02 Store only credential references, provider/account metadata, status, permission scope, and last verified timestamps in SQLite.
- [ ] A7.03 Add redaction tests proving secrets do not appear in prompts, agent inputs, logs, events, content records, notifications, exports, or evidence artifacts.
- [ ] A7.04 Add credential health checks before dry test, Run Now, scheduling, publishing, and autonomy enablement.
- [ ] A7.05 Document future external/team vault seam without implementing cloud secrets in V1.
- Likely files: Keychain service, integration status service, OmniSocials settings UI, redaction tests.
- Gate: native Keychain verification passes on macOS or state is truthfully blocked/unverified with no fake credential readiness.

## Phase 8 — Notifications, Recovery, and Evidence Retention

- [ ] A8.01 Implement per-campaign notification policy across in-app, desktop, and email.
- [ ] A8.02 MaVoid default: daily success digest plus instant alerts for failure, approval-needed, credential issues, missed fallback, provider mismatch.
- [ ] A8.03 Implement ordered Recovery Agent workflow: retry failed stage once, regenerate slot once, generate replacement once, then escalate/create task if still blocked or fallback is close.
- [ ] A8.04 Implement evidence retention defaults: final assets/captions/provider IDs/review reports forever unless deleted, raw logs/failed intermediates 90 days, summaries/timelines forever, archive to local/iCloud storage.
- [ ] A8.05 Implement evidence browser, log redaction indicators, export/reveal actions, retention warnings, and archive states.
- Likely files: notification service/native desktop/email helpers, recovery service/UI, evidence/retention services/UI.
- Gate: notification tests, desktop notification manual notes, email provider truth state, recovery tests, and retention/archive tests pass.

## Phase 9 — UI Screens and Cross-Workspace Surfaces

- [ ] A9.01 Implement Content Autonomous Campaign Dashboard with brand filter, Run Now, approval queue, summary cards, pipeline board, OmniSocials health, right inspector.
- [ ] A9.02 Implement Brand Containers, Campaign Wizard, Advanced Campaign Editor, Slot/Calendar View, Today's Pipeline, Content Piece Detail, Editor/Override Flow.
- [ ] A9.03 Implement Approval Queue, Dry Test Report, Run Now Modal, Recovery/Failure Center, OmniSocials Account Settings, Evidence/Logs/Assets View.
- [ ] A9.04 Implement Automations Mirror Detail and Agent-Run Linked Panels.
- [ ] A9.05 Implement Notifications/In-App Alerts, reusable status chips/cards/inspectors, required empty/loading/error/blocked/success/recovery/credential/unsaved states.
- [ ] A9.06 Preserve Apple-inspired Zoid design system, keyboard navigation, accessible status labels, and dangerous-action confirmations.
- Likely files: `src/App.tsx`, autonomous content components/view models/tests, CSS/design tokens.
- Gate: browser smoke has no console errors; native UI shows real/empty/blocked data truthfully.

## Phase 10 — Run Now, Dry Test, and Autonomy Enablement

- [ ] A10.01 Implement Run Now modes each time: generate missing, regenerate, catch up, selected slot/campaign; default generate missing only.
- [ ] A10.02 Run Now preview must show affected campaign/date/slots, duplicate risk, publish behavior, approval requirements, credentials/provider state, and live-publish warnings.
- [ ] A10.03 Implement full dry test: credentials/accounts/permissions, sample generation, design, review, platform adaptations, schedule simulation, notification test, secret/logging hygiene, no live publish.
- [ ] A10.04 Gate autonomy enablement on dry test pass, review approval, adaptation pass, OmniSocials verification, no secret/log issue, notification test pass.
- [ ] A10.05 Store dry test report and linked agent/automation/evidence records.
- Likely files: Run Now modal/service, dry test service/report UI, autonomy state commands/tests.
- Gate: dry test cannot live publish; autonomy toggle remains disabled until critical checks pass.

## Phase 11 — MaVoid End-to-End Launch Gate

- [ ] A11.01 Configure MaVoid brand and MaVoid Daily Autonomous Content campaign with two daily slots and selected platforms/accounts.
- [ ] A11.02 Execute dry test and retain report.
- [ ] A11.03 Execute real MaVoid E2E: Zoid scheduler, agent runs, two-post generation, designer/reviewer, platform adaptations, edit/override, OmniSocials scheduling/publishing, provider verification, notifications, and recovery path.
- [ ] A11.04 Prove one-slot failure does not block the other slot; recovery/escalation works and notifies.
- [ ] A11.05 Retain provider IDs, final assets, captions, review reports, verification reports, logs, notifications, and timeline evidence.
- [ ] A11.06 Record native/macOS verification notes and known launch limitations.
- Likely files: MaVoid campaign records, app-support evidence/logs, manual verification doc, review handoff.
- Gate: no V1 launch without real MaVoid E2E evidence.

## Phase 12 — Tests, Native Verification, and Critique Approval

- [ ] A12.01 Backend tests: schema, scheduler, pipeline, agent links, approvals, recovery, Run Now, dry test, OmniSocials, Keychain refs, notifications, retention.
- [ ] A12.02 Frontend tests: dashboard, wizard/editor, calendar, pipeline, detail/adaptations, approval queue, Run Now, dry test report, recovery center, settings, evidence, notification states.
- [ ] A12.03 Security tests: redaction, secret rejection, no raw media blobs/secrets in SQLite/logs/events/exports, fail-closed credential states.
- [ ] A12.04 Native/manual verification: Tauri app launch, app-support DB/log/evidence inspection, Keychain status, desktop notifications, OmniSocials account verification, MaVoid E2E.
- [ ] A12.05 Routine full verification: `npm run verify:local && git diff --check`.
- [ ] A12.06 Write handoff at `.hermes/reviews/autonomous-content-automation/handoff.md`.
- [ ] A12.07 Run feature critique loop until `.hermes/reviews/autonomous-content-automation/critique-report.md` says `Verdict: APPROVED`.
- Gate: phase cannot be called done until tests, native verification evidence, MaVoid E2E evidence, and critique approval are all present.
