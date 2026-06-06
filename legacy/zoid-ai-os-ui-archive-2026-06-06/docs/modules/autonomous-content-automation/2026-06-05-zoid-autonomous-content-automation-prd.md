# PRD: Zoid Autonomous Content Automation

Date: 2026-06-05
Product area: Content Workspace, Agents Workspace, Automations Workspace
Launch gate brand: MaVoid

## 1. Overview
Zoid will own autonomous multi-brand social content generation, review, scheduling, publishing, verification, and recovery. The Content Workspace becomes the primary control surface for campaigns, daily slots, drafts, approvals, calendars, edits, publishing state, and history. The Automations Workspace mirrors scheduler/jobs/retries/logs for operational debugging. The Agents Workspace executes the planner, researcher, copy, designer, reviewer, publisher, verifier, and recovery runs using first-class Agent Profiles and Agent Runs.

The v1 product scope is the full architecture: multi-brand containers, campaign templates, Zoid-owned scheduler, agent orchestration, credentials, OmniSocials integration, editor, notifications, recovery, Run Now, dry run, and evidence/audit trail. Release readiness is gated by a real MaVoid end-to-end run, while multi-brand support must exist structurally even if only MaVoid is live on day one.

## 2. Goals and Non-Goals
### Goals
- Replace Hermes cron with a Zoid-owned scheduler, job runner, logs, retry engine, credential model, and direct agent invocation path.
- Let users configure autonomous campaigns that generate, review, adapt, schedule, publish, verify, and recover content with minimal intervention.
- Support auto-generation and auto-scheduling while allowing users to stop, edit, approve, reschedule, or cancel before publish time.
- Require human approval only for first-run probation or low-confidence/flagged content.
- Show every pipeline stage and preserve evidence for generated assets, captions, reviews, provider IDs, logs, and status timelines.
- Support multiple brands, campaigns, slots, platforms, accounts, and notification policies from the beginning.

### Non-Goals
- Continuing to rely on Hermes cron as the runtime scheduler.
- Creating a separate content-specific agent runner outside the existing Agents Workspace model.
- Requiring multiple real brands to be live for the first launch.
- Silent post-publish mutation. Published content changes use correction/repost workflows.
- Storing raw provider secrets in campaign records, prompts, logs, exports, or content artifacts.

## 3. Key Decisions
- Content Workspace is the main management surface; Automations Workspace mirrors technical internals.
- Campaigns are template-based by default with advanced freeform pipeline overrides.
- Agent execution uses existing Agent Profiles and Agent Runs.
- Required reusable profiles: Content Strategist/Planner, Researcher, Caption/Copy Agent, Social Designer, Design Reviewer, Publisher/OmniSocials Agent, Verification Agent, Recovery Agent.
- Default reviewer confidence threshold is 85/100.
- First 3 successful runs of a new campaign/template require human approval unless manually marked trusted earlier.
- MaVoid defaults: generation 8:00 AM, review deadline 10:00 AM, slot 1 publish window 11:00 AM-1:00 PM, slot 2 publish window 5:00 PM-7:00 PM, latest safe fallback 8:30 PM.
- One core content piece produces platform-specific captions, assets, formats, and schedule records.
- Credentials use macOS Keychain for secrets and SQLite for references/metadata/mappings only.
- MaVoid default notifications: daily success digest plus instant failure, approval-needed, credential, missed-fallback, and publish/provider mismatch alerts.

## 4. Users
- Founder/operator: configures brands and campaigns, reviews exceptions, edits content, monitors launches.
- Content manager: manages calendar, slots, drafts, approvals, platform adaptations, and publishing history.
- Technical operator: inspects automations, agent runs, retries, logs, scheduler state, provider errors, and recovery.
- Reviewer/approver: resolves low-confidence items and required-fix queues.

## 5. Core Entities
- Brand: voice, visual identity, pillars, banned claims/styles, target audience, offers, social accounts, credential mappings, default agents/templates, assets, approval policy, timezone/language, notification recipients.
- Campaign: brand-scoped automation template instance with objective, cadence, slots, platforms/accounts, schedules, context, agents, approval policy, notification policy, autonomy state.
- Slot: recurring or one-off content opportunity with brief, date, generation time, review deadline, publish window, fallback time, status, and manual overrides.
- Content Piece: core idea/draft/assets for a slot.
- Platform Adaptation: platform-specific caption, media format, constraints, schedule/post state, provider IDs.
- Agent Run: linked execution record for each pipeline stage with prompt, attachments, output, logs, review, and events.
- Review Record: verdict, score, confidence, flags, required fixes, evidence.
- Automation Job/Run: scheduler trigger, inputs, outputs, errors, retries, linked entities, logs.
- Provider Connection/Platform Account/Brand Mapping: credential reference and account authorization metadata.
- Evidence Artifact: final assets, captions, IDs, review reports, logs, failed intermediates, timelines, archives.

## 6. Workspace Architecture
### Content Workspace
Primary UI for campaign setup, calendar, daily slots, active pipeline items, review queue, drafts, asset library, OmniSocials status, publishing history, edit/approve/pause controls, and right-side inspectors.

### Agents Workspace
Executes all pipeline profiles through Agent Runs. Run detail must expose prompt, clean output, raw logs, status timeline, linked entities, review, and events. Reviewer Agent queue handles approvals, required fixes, blocked states, and evidence attachment.

### Automations Workspace
Mirrors the technical scheduler layer: automation definitions, upcoming runs, failed runs, retry failed, change schedule, run now, logs, linked entities, failure policy, and run metadata.

### Shared Surfaces
Calendar shows publishing slots, automation schedules, reminders, due dates, and follow-ups. Reviews centralize approval records. History provides exportable event timelines. Tasks capture manual recovery/fix actions.

## 7. End-to-End Workflows
### Campaign Setup
1. Wizard: pick brand.
2. Pick campaign template.
3. Define slots/cadence.
4. Select platforms/accounts.
5. Attach brand docs, context, creative assets, and calendar topics.
6. Choose agent profiles.
7. Set generation time, review deadline, publish windows, fallback.
8. Set approval/confidence policy.
9. Set notification policy.
10. Run dry test.
11. Enable autonomy.
Advanced editor allows freeform overrides for slots, platforms, schedules, profile chain, approval rules, and context assets.

### Daily Autonomous Run
1. Scheduler starts campaign/slot at generation time.
2. Planner resolves slot brief using source priority.
3. Researcher fills gaps or fresh context.
4. Copy agent creates draft/captions.
5. Designer creates assets.
6. Reviewer scores factuality, design, readability, brand fit, compliance, and confidence.
7. If approved above threshold and campaign is trusted, Zoid schedules/publishes via OmniSocials.
8. If approval is required, item enters review queue before deadline/fallback.
9. Publisher creates OmniSocials schedules/posts and stores provider IDs.
10. Verifier checks provider state and records result.
11. Notifications and history update.

### Source Priority
1. Campaign slot brief/manual override.
2. Brand rules and banned claims/styles.
3. Content calendar topic.
4. Attached docs/context.
5. Recent posts and repetition avoidance.
6. Fresh research/news.
7. Agent creative judgment.
Brand rules veto everything. Manual override may override topic, but not compliance/banned claims unless explicitly forced.

## 8. Autonomy and Approval Policy
Auto-publish is allowed only when:
- Reviewer verdict is approved.
- Reviewer score is at least 85/100 by default.
- No factual/source uncertainty is present.
- No design/readability issue remains after retries.
- Campaign/template has completed first 3 successful approved runs or has been manually marked trusted.

Human approval is required for reviewer low confidence, score below threshold, factual/source uncertainty, unresolved design/readability issue, first-run probation, credential/provider mismatch, missed fallback, or explicit user/campaign rule.

Users can pause campaign autonomy, pause a slot, stop an Agent Run, approve and continue, require fixes, regenerate, reschedule, cancel, or create a follow-up task.

## 9. Scheduler and Agent Architecture
- Zoid scheduler owns campaign triggers, slot generation, review deadlines, publish windows, fallback checks, retry budgets, and Run Now triggers.
- Scheduler creates Automation Runs that link to Content entities and Agent Runs.
- Each stage invokes a configured Agent Profile with scoped permissions, context attachments, parser/output contract, and linked entity IDs.
- Handoffs are structured artifacts, not untracked text blobs.
- Logs are visible in Automations and Agent Run Detail; user-facing summaries are visible in Content.
- Failed stages are fail-closed and route to Recovery Agent or human queue according to policy.

## 10. Multi-Brand and Campaign Model
Hierarchy: Brand -> Campaigns -> Slots -> Content Pieces -> Platform Adaptations -> Schedules/Posts.

Brands are hybrid containers inside Content Workspace, not separate apps. Each campaign can select different platforms/accounts, schedules, agent chains, approval policies, and notification recipients. One OmniSocials provider connection can expose many accounts/pages; brands map to the appropriate platform accounts.

Campaign templates define default slots, cadence, pipeline, approval rules, and contexts. Advanced overrides are allowed while preserving auditability.

## 11. Credentials and Security
- macOS Keychain stores raw provider tokens/secrets.
- SQLite stores credential references, provider/account metadata, status, last verified timestamps, and brand-account mappings.
- No raw secrets in prompts, logs, content records, exports, review reports, or evidence archives.
- Dry run and autonomy enablement must verify account permissions for selected platforms.
- Credential errors fail closed, notify immediately, and block live publishing for affected accounts only.
- Future team/cloud versions may replace or augment Keychain with a team vault.

## 12. Notifications
Notification policy is per campaign and supports in-app Zoid, desktop notifications, and email.

MaVoid defaults:
- Daily success digest.
- Instant alert for failure, approval needed, credential/account issue, missed fallback, provider mismatch, recovery escalation, or publish verification failure.

Notifications must link to the affected campaign, slot, content piece, Agent Run, Automation Run, review record, and suggested next action.

## 13. Failure and Recovery
Failures are isolated per slot. Successful slots proceed independently.

Default Recovery Agent sequence:
1. Retry failed stage once.
2. If still failing, regenerate the slot once.
3. If still failing, generate a replacement once.
4. If still blocked or fallback is close, notify and create an approval/task.

If one post fails, publish/schedule successful posts and notify about the failed post. If fallback time passes without approval/safe schedule, block the slot and notify.

## 14. Dry Run, Run Now, and Editing
### Dry Run
Required before autonomy enablement. It validates credentials/accounts/permissions, generates a sample slot, designs assets, reviews, creates platform adaptations, simulates scheduling, and tests notifications without live publishing. Autonomy can only be enabled if the dry run passes agent, reviewer, platform, credential, logging/security, and notification checks.

### Run Now
Run Now prompts for mode each time:
- Generate missing only (default).
- Regenerate.
- Catch up.
- Selected campaign/slot.

The modal previews campaign/date/slots affected, duplicate risk, publish behavior, whether output stops at draft or schedules/publishes, and required approvals.

### Editing
- Before scheduled: edit content, captions, assets, platforms, or schedule normally.
- Scheduled but not published: edits must update/cancel/reschedule in OmniSocials and verify provider state.
- Published: use correction/repost workflow, such as platform-supported edit, delete/repost, correction comment, or follow-up post. Do not claim silent mutation when unsupported.

## 15. Evidence Retention
Per-campaign retention policy with defaults:
- Final published assets, captions, provider IDs, and review reports: keep forever unless deleted.
- DB summaries and status timelines: keep forever.
- Raw logs and failed intermediate assets: keep 90 days.
- Older artifacts may archive to local or iCloud file storage.

Evidence must be linked to content pieces, platform adaptations, agent runs, automation runs, review records, provider IDs, notifications, and history events.

## 16. MaVoid Launch Gate
Launch is not complete until MaVoid proves a real end-to-end autonomous flow using Zoid-owned infrastructure:
- Scheduler starts the daily run without Hermes cron.
- Two daily MaVoid slots are generated from campaign context.
- Designer and reviewer agents produce assets and review reports.
- Platform adaptations are created for selected accounts.
- User can edit/override before publish.
- OmniSocials schedule/post records are created.
- Provider IDs and verification state are stored.
- Success/failure/approval notifications work.
- One recovery path is exercised and recorded.
- Evidence and history are retained.

Multiple real brands are not required for launch, but the data model and UI must support them.

## 17. Acceptance Criteria
- A user can create a Brand with identity, rules, assets, accounts, mappings, defaults, timezone/language, approval policy, and notification recipients.
- A user can create an autonomous campaign through the wizard and advanced editor.
- Campaigns support configurable slots, platforms/accounts, schedules, profile chains, approvals, notifications, and context assets.
- Zoid scheduler can run generation, review deadline, publish window, fallback, retry, and Run Now flows without Hermes cron.
- Content Workspace shows campaign dashboard, calendar slots, active pipeline, review queue, OmniSocials status, assets, publishing history, and inspectors.
- Agents Workspace shows linked Agent Runs for every pipeline stage with prompts, outputs, logs, timelines, reviews, events, and linked entities.
- Automations Workspace shows automation definitions, upcoming runs, failed runs, logs, retries, failure policy, and linked entities.
- Approval logic enforces threshold, first-run probation, low-confidence categories, and manual trusted override.
- Dry run blocks autonomy enablement until credentials, generation, design, review, adaptations, scheduling simulation, logging/security, and notifications pass.
- Run Now supports generate missing, regenerate, catch up, and selected slot/campaign with duplicate-risk preview.
- Editing works before schedule, scheduled pre-publish with OmniSocials update/cancel/reschedule, and published correction/repost flow.
- Credential storage keeps raw secrets in Keychain only and never exposes them to prompts/logs/artifacts/exports.
- Notifications work in-app, desktop, and email according to per-campaign policy.
- Failure recovery follows retry, regenerate, replacement, escalation, while successful slots proceed independently.
- Evidence retention stores final assets/captions/provider IDs/reviews and status timelines permanently by default, logs/intermediates for 90 days, with archive support.
- MaVoid E2E launch gate passes with real two-post daily automation, publishing verification, notifications, and recorded recovery evidence.
