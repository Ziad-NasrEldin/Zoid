# Stitch AI Prompt: Zoid Autonomous Content Automation Screens

Use this prompt in Stitch AI to design a complete screen set for Zoid's Autonomous Content Automation product area.

## Product context

Zoid is a macOS desktop app for orchestrating agents, content, automations, publishing, logs, and review workflows. This feature makes the Content Workspace the primary control surface for autonomous multi-brand content campaigns while mirroring technical execution in Automations and linking every stage to Agents Workspace runs.

The system must support a full V1 architecture immediately:
- Multi-brand content automation.
- Zoid-owned scheduler, job runner, logs, retry engine, credentials, and direct agent invocation.
- Agents Workspace linkage through reusable Agent Profiles and Agent Runs.
- OmniSocials provider/account settings and brand-account mappings.
- Autonomous generation, review, scheduling, publishing, verification, notifications, recovery, evidence retention, and human override.
- Real MaVoid E2E launch gate: two daily posts, editable publish windows, platform adaptations, editor/override, OmniSocials scheduling, verification, notifications, and recovery path.

## Existing workspace references to preserve

The design must feel like an extension of these existing workspace screen models:

### Agents Workspace reference
Existing screens include Agents Dashboard, New Agent Run modal, Active Runs, Agent Run Detail, Agent Profiles, and Reviewer Agent. Reuse these ideas:
- Agent profile selector, mode selector, working directory, prompt, context attachments, permission preview, review requirement preview.
- Active run list with status, attention state, stop/send input/retry/log actions.
- Agent Run Detail tabs: Summary, Prompt, Clean Output, Raw Logs, Status Timeline, Linked Entities, Review, Events.
- Reviewer queue with Approve, Require Fixes, Block, Attach Evidence.

### Content Workspace reference
Existing screens include Content Dashboard, Content Calendar, Content Plans, Content Pieces, and Asset Library. Reuse these ideas:
- Header with search, new content/campaign, generate today's content, settings.
- Summary cards for planned today, drafts, needs review, scheduled, failures.
- Content calendar preview, active pipeline items, review queue, OmniSocials status, asset queue, publishing history, right inspector.
- Content Piece tabs: Brief, Draft, Assets, Captions, Platform Adaptations, Review, Publishing, Verification, History.

### Automations Workspace reference
Existing screens include Automations Dashboard, All Automations, Automation Detail, and Automation Run Detail. Reuse these ideas:
- Automation list with schedule, enabled state, last run, next run, result.
- Detail tabs: Overview, Schedule, Runs, Logs, Linked Entities, Failure Policy, History.
- Run detail areas: Trigger, Inputs, Outputs, Errors, Logs, Linked entities, Events, metadata inspector.

## Visual style direction

Use Zoid's Apple-inspired design system:
- Clean, photography-first, low-chrome interface; let content previews, generated assets, and campaign artifacts be the hero.
- Primary action color: Action Blue #0066cc. Use it for links, pill CTAs, focus rings, selected states, and main actions only.
- Surfaces: white #ffffff, parchment #f5f5f7, near-black tiles #272729/#2a2a2c/#252527 for high-emphasis status/preview sections.
- Typography: SF Pro Display/SF Pro Text or system font equivalent. Headlines 600 weight with tight tracking; body 17px regular; captions 14px.
- Geometry: pill CTAs and filter chips, 18px utility cards, 8px compact utility controls, 1px hairline borders, no decorative gradients.
- Avoid heavy SaaS chrome. Prefer spacious sections, quiet hairlines, precise status chips, and right inspectors.
- Do not use card shadows except if representing actual generated content/product imagery. Use surface alternation and hairlines for hierarchy.
- Desktop-first macOS app layouts, but include responsive tablet/mobile adaptations where relevant.

## UX principles

1. Content Workspace is the source of truth.
   - Campaigns, brands, slots, posts, approvals, edits, and Run Now should be managed from Content.
   - Automations only mirrors the technical job/run/scheduler layer.

2. Show every pipeline stage.
   - Strategy/planning, research, copy, design, review, adaptation, scheduling, publishing, verification, notifications, and recovery should all be visible.
   - Each stage should expose status, confidence, owning agent profile, linked run, evidence, retry count, and next action.

3. Fail closed around publishing.
   - Auto-publish only if reviewer verdict is approved, confidence >= 85/100, probation is complete, credentials verify, and no policy flag remains.
   - Low confidence, first-run probation, factual uncertainty, design/readability issues, credential mismatches, missed fallback, or provider mismatch must stop and request approval.

4. Human override must be obvious and safe.
   - Before scheduled: normal edits.
   - Scheduled but not published: update/cancel/reschedule in OmniSocials and re-verify provider state.
   - Published: correction/repost workflow, not silent mutation.

5. Multi-brand from day one.
   - Hierarchy: Brand → Campaigns → Slots → Content Pieces → Platform Adaptations → Schedules/Posts.
   - Brands carry voice, visual identity, content pillars, banned claims, accounts, default agents/templates, assets, approval policy, timezone/language, and notifications.

6. Credentials are secure and never shown as secrets.
   - macOS Keychain stores tokens/secrets.
   - SQLite/app records store credential references, provider/account metadata, status, and verification timestamps only.
   - No raw secrets in prompts, logs, evidence, exports, or screen copy.

7. Recovery should preserve successful work.
   - If one slot fails, successful slots continue.
   - Recovery order: retry failed stage once, regenerate slot once, generate replacement once, then escalate if still blocked or fallback is close.

8. Every autonomous action needs evidence.
   - Store final assets, captions, provider IDs, review reports, verification results, status timelines, and linked agent/automation logs.

## Core data and status model to reflect visually

Show these entities and statuses throughout the UI:
- Brand status: configured, missing accounts, credentials unverified, in probation, trusted, disabled.
- Campaign status: draft, dry-test required, ready, active/autonomous, paused, approval-needed, failed, recovering, archived.
- Slot status: planned, generating, reviewing, approval-needed, scheduled, published, failed, missed fallback, skipped.
- Content piece status: brief, draft, designed, adapted, reviewed, scheduled, published, correction-needed.
- Platform adaptation status: generated, needs edit, scheduled, published, provider mismatch, failed.
- Agent run status: queued, running, blocked, requires input, retrying, completed, failed, reviewed.
- Confidence states: approved 85+, low-confidence, factual uncertainty, design/readability warning, manual forced approval.
- Publish safety states: safe to publish, blocked by approval, blocked by credential, blocked by fallback, duplicate risk, provider mismatch.

## Screens to design

Design every screen below. Include desktop layout and responsive behavior. For each screen, show realistic sample data for MaVoid plus enough generic multi-brand examples to prove the model scales.

### 01. Content Autonomous Campaign Dashboard

Purpose: Main command center in Content Workspace for autonomous campaigns.

Must include:
- Top workspace header: search, brand filter, New Campaign, Run Now, Approval Queue, Settings.
- Summary cards: active campaigns, today's planned slots, needs approval, scheduled, published today, failures/recovering, credential/account health.
- Brand/campaign switcher with MaVoid selected and other brand examples.
- Today's timeline: generation start, review deadline, publish windows, fallback time.
- Active campaign list with status, autonomy toggle, probation/trusted badge, confidence threshold, selected platforms, next slot, next run.
- Pipeline board grouped by stage: Planning, Research, Copy, Design, Review, Adapt, Schedule, Publish, Verify, Recover.
- Approval-needed strip with count, reason, deadline, primary action.
- OmniSocials health panel: connection status, mapped accounts, last verified, provider mismatch alerts.
- Right inspector for selected campaign/slot with linked automation and linked agent runs.

States:
- Empty: no brands/campaigns yet, with Create Brand and Create Campaign CTAs.
- Loading/skeleton while campaigns and scheduler load.
- Error: scheduler unavailable, account verification failed, logs unavailable.
- Blocked: autonomy disabled until dry test passes.
- Success: all slots scheduled/published, daily digest ready.

### 02. Brand Containers

Purpose: Manage brand-level settings inside Content Workspace.

Must include:
- Brand list/grid with logo/mark, status, active campaigns, account mappings, timezone, approval policy.
- Brand detail tabs: Overview, Voice & Rules, Visual Identity, Content Pillars, Accounts, Agent Defaults, Templates, Assets, Approval Policy, Notifications, Retention, History.
- Voice/rules editor: tone, banned claims/words/styles, target audience, offers/products/services, language.
- Visual identity area: colors, typography, logo, image style, example assets.
- Account mapping table: platform, OmniSocials account/page, credential reference, status, permissions, last verified.
- Default agent profile assignments: strategist, researcher, copy, designer, reviewer, publisher, verifier, recovery.
- Notification recipients and channels: in-app, desktop, email.

States:
- Missing account mapping.
- Credential reference present but unverified.
- Brand in first-run probation.
- Trusted brand/campaign override.
- Delete/archive confirmation.

### 03. Campaign Wizard

Purpose: Guided setup for a new autonomous campaign.

Wizard steps:
1. Pick Brand.
2. Pick Campaign Template.
3. Define slots/cadence.
4. Select platforms/accounts.
5. Attach brand/context docs/assets.
6. Choose agent profiles.
7. Set schedule/windows/fallbacks.
8. Set approval/confidence policy.
9. Set notification policy.
10. Run dry test.
11. Enable autonomy.

Must include:
- Stepper with progress, validation status, Save Draft, Exit, Continue.
- Template card for Autonomous Content Campaign and a MaVoid Daily Two-Post template.
- Slots/cadence editor with default MaVoid schedule: generation 8:00 AM, review deadline 10:00 AM, slot 1 publish window 11:00 AM-1:00 PM, slot 2 publish window 5:00 PM-7:00 PM, latest safe fallback 8:30 PM.
- Platform/account selector showing LinkedIn, Instagram, X, TikTok, Facebook, Threads examples.
- Context attachment area with source priority explanation: manual slot brief, brand rules, calendar topic, attached docs, recent posts avoidance, fresh research, creative judgment.
- Agent profile chain preview.
- Approval policy defaults: threshold 85/100, first 3 successful runs require approval, manual trusted override.
- Notification defaults: daily success digest plus instant failure/approval-needed/credential/missed fallback/provider mismatch alerts.
- Dry test gate before autonomy enablement.

States:
- Incomplete required fields.
- Duplicate campaign/template name.
- Account permission failure.
- Dry test not run.
- Dry test failed.
- Ready to enable.

### 04. Advanced Campaign Editor

Purpose: Power-user editor for campaign templates, pipeline overrides, slots, agents, and policies.

Must include:
- Split layout: left navigation tree, center editor, right validation/impact inspector.
- Editable areas: General, Brand, Template, Slots, Platforms, Pipeline, Agent Profiles, Context, Schedule, Approval, Notifications, Recovery, Retention, Advanced Overrides.
- Visual pipeline builder with agent stages and handoff contracts.
- Freeform override fields for slot briefs, stage prompts, platform rules, retry counts, timeout budgets.
- Diff/impact preview before saving: affected future slots, pending schedules, automation jobs, agent runs.
- Save Draft, Validate, Publish Changes, Revert, Pause Campaign.

States:
- Unsaved changes.
- Validation warning.
- Breaking change requiring reschedule.
- Locked field inherited from template with unlock/override affordance.
- Success after publishing changes.

### 05. Slot / Calendar View

Purpose: Calendar and slot-level planning/scheduling surface.

Must include:
- Month/week/day toggle, brand/campaign filters, platform filters.
- Daily slots with stage/status chips, confidence, approval markers, platform icons, publish windows, fallback time.
- Drag/reschedule behavior with safe publish window feedback.
- Slot inspector showing brief, sources, content piece, platform adaptations, linked agent runs, automation job, evidence, retry/recovery history.
- Visual distinction between generation time, review deadline, publish window, and latest safe fallback.

States:
- Empty day with Add Slot / Generate Missing.
- Overbooked or conflicting schedule.
- Missed fallback.
- Scheduled provider mismatch.
- Published success.

### 06. Today's Pipeline

Purpose: Operational day view for generation through verification.

Must include:
- Timeline for today with MaVoid's two slots.
- Stage swimlanes for planner, researcher, copy, designer, reviewer, publisher, verifier, recovery.
- Live progress indicators and ETA.
- Per-stage cards with agent profile, run status, output summary, confidence, evidence count, retry count.
- Attention drawer for blocked steps and required approvals.
- Controls: Pause Today, Run Missing, Regenerate Slot, Open Logs, Notify Me, Mark Trusted.

States:
- Before generation starts.
- Running.
- Waiting for review deadline.
- Approval-needed.
- One slot failed while another proceeds.
- All published/verified.

### 07. Content Piece Detail with Platform Adaptations

Purpose: Deep detail screen for one content piece and its platform-specific outputs.

Must include:
- Header: title, brand, campaign, slot, status, confidence, publish state, primary action.
- Pipeline status bar across all stages.
- Tabs: Brief, Draft, Assets, Captions, Platform Adaptations, Review, Publishing, Verification, Evidence, History.
- Core content idea/brief and source priority trace.
- Asset preview gallery and generated creative variants.
- Platform adaptation cards for LinkedIn, Instagram, X, TikTok, Facebook, Threads with caption, format, media, constraints, schedule/post ID, provider status.
- Review report with score, confidence, reasons, factual/source uncertainty, design/readability flags, required fixes.
- Publishing/verification panel with OmniSocials IDs and provider state.

States:
- Draft only, no assets yet.
- Assets generated but not reviewed.
- Platform adaptation failed for one platform.
- Scheduled but editable.
- Published with correction/repost options.

### 08. Editor / Override Flow

Purpose: Safe editing flow before schedule, after schedule, and after publish.

Must include:
- Compare layout: generated version vs edited override.
- Editable caption, asset, brief, platform rules, schedule, approval note.
- Platform-specific editor with constraint validation.
- Safety banner explaining phase:
  - Before scheduled: edit normally.
  - Scheduled not published: update/cancel/reschedule in OmniSocials and verify.
  - Published: correction/repost workflow only.
- Actions: Save Draft, Re-run Review, Apply to All Platforms, Update OmniSocials Schedule, Cancel Schedule, Create Correction, Repost, Discard.
- Audit note requirement for forced override of warnings.

States:
- Unsaved edits.
- Validation errors by platform.
- Requires re-review.
- Provider update pending.
- Provider update failed.
- Override saved and verified.

### 09. Approval-Needed Queue

Purpose: Central queue for human decisions.

Must include:
- Queue filters: brand, campaign, reason, deadline, platform, confidence, stage.
- Items grouped by urgency: missed/near fallback, low confidence, factual uncertainty, design issue, first-run probation, credential/provider issue.
- Item row with preview, slot time, reason chips, confidence score, reviewer verdict, platform icons, deadline.
- Inspector with generated output, review report, evidence, logs, linked agent runs, recommended fixes.
- Batch actions where safe.
- Actions: Approve, Edit, Require Fixes, Regenerate, Generate Replacement, Block, Mark Campaign Trusted.

States:
- Empty queue.
- Approval deadline missed.
- Batch action disabled due to mixed reasons.
- Approved success.
- Require fixes started recovery run.

### 10. Dry Test Report

Purpose: Pre-autonomy gate validating the full campaign pipeline without live publishing.

Must include:
- Report header with pass/fail, campaign, brand, run date, dry test ID.
- Checklist sections: credentials/accounts/permissions, sample generation, design, review, platform adaptations, schedule simulation, notifications, secret/logging hygiene.
- Sample slot preview and platform adaptation previews.
- Agent run links for every profile.
- Failure details with suggested fixes.
- Gate decision: Enable Autonomy disabled until all critical checks pass.
- Actions: Re-run Dry Test, Fix Settings, Open Logs, Export Report, Enable Autonomy.

States:
- Running dry test.
- Passed.
- Failed critical credential check.
- Failed review confidence.
- Notification test failed.
- Secret leakage warning.

### 11. Run Now Modal

Purpose: Manual execution entry point with duplicate protection.

Must include:
- Mode selector each time: Generate Missing, Regenerate, Catch Up, Selected Slot/Campaign.
- Default selected mode: Generate Missing Only.
- Campaign/date/slot picker.
- Preview of affected slots, platforms, current statuses, duplicate risk, publish behavior, approval requirements.
- Publish behavior choices: stop at draft, schedule if approved, publish now if safe, require manual approval.
- Risk warnings: duplicates, provider mismatch, missed fallback, credentials unverified, live publish.
- Actions: Cancel, Preview Runs, Run Now.

States:
- No missing content.
- Duplicate risk detected.
- Credentials blocked.
- Approval required.
- Run started with linked automation and agent runs.

### 12. Recovery / Failure Center

Purpose: Unified troubleshooting and recovery surface for failed autonomous content slots.

Must include:
- Failure summary cards: active failures, recovering, escalated, missed fallback, provider mismatches.
- Failure list grouped by brand/campaign/slot/stage.
- Recovery plan visualization: retry failed stage, regenerate slot, generate replacement, escalate.
- Detail inspector with error, logs, evidence, failed output, affected platforms, time remaining before fallback.
- Recovery Agent run card and history.
- Actions: Retry Stage, Regenerate Slot, Generate Replacement, Escalate to Human, Skip Slot, Mark Resolved, Create Task.

States:
- One slot failed while another published.
- Recovery running.
- Recovery succeeded.
- Recovery exhausted and escalated.
- Fallback time too close; human approval required.

### 13. OmniSocials / Account Settings

Purpose: Provider connections, platform accounts, and brand mappings.

Must include:
- OmniSocials connection list with credential reference, status, last verified, permissions.
- Platform account/page table: platform, account name, handle/page, provider ID, permissions, mapped brand(s), status.
- Brand-account mapping editor.
- Credential storage explanation: secrets stored in macOS Keychain; app stores references only.
- Test connection and verify permissions workflow.
- Notification if raw secrets are detected in logs/prompts.

States:
- Not connected.
- Connected but no accounts mapped.
- Permission missing for scheduling/publishing.
- Token expired.
- Verification success.
- Provider mismatch after publish.

### 14. Evidence / Logs / Assets View

Purpose: Audit and artifact repository for autonomous content.

Must include:
- Unified evidence browser with filters for brand, campaign, slot, platform, stage, artifact type, date, retention.
- Artifact cards/list: final assets, captions, review reports, OmniSocials IDs, verification reports, raw logs, failed intermediate assets, DB summaries, status timelines.
- Asset preview and log viewer with redaction indicators.
- Retention policy panel: final published artifacts kept forever unless deleted; raw logs and failed intermediates 90 days; summaries/timelines forever; archive older artifacts to local/iCloud storage.
- Export and reveal-in-storage actions.

States:
- Empty/no evidence yet.
- Redacted sensitive value.
- Artifact archived.
- Retention warning before deletion.
- Log unavailable.

### 15. Automations Mirror Detail

Purpose: Technical mirror of a Content campaign in Automations Workspace.

Must include:
- Automation Detail for an Autonomous Content Campaign scheduler job.
- Header with linked brand/campaign, enabled state, next run, last result.
- Tabs: Overview, Schedule, Runs, Logs, Linked Entities, Failure Policy, History.
- Schedule model showing generation time, review deadline, publish windows, fallback.
- Run list with job stages and linked Content pieces/slots.
- Failure policy matching ordered recovery flow.
- Actions: Run Now, Pause/Resume, Retry Failed, Change Schedule, View Failures, Open in Content.

States:
- Enabled healthy.
- Paused from Content.
- Scheduler unavailable.
- Failed run with retry available.
- Schedule changed requiring Content confirmation.

### 16. Agent-Run Linked Panels

Purpose: Cross-workspace panels that reveal agent execution from Content and Automations screens.

Must include:
- Compact linked Agent Run panel embeddable in Content detail, Today's Pipeline, Recovery Center, and Automation Run Detail.
- Shows profile, role, status, duration, confidence/review status, prompt summary, output summary, permission preview, linked entity, latest log event.
- Expand to mini-tabs: Summary, Prompt, Output, Logs, Review, Events.
- Actions: Open Full Agent Run, Retry, Send Input, Stop, Request Review, Export Logs.
- Show first-class profiles: Content Strategist/Planner, Researcher, Caption/Copy Agent, Social Designer, Design Reviewer, Publisher/OmniSocials Agent, Verification Agent, Recovery Agent.

States:
- Queued/running.
- Requires input.
- Blocked by permissions.
- Completed and reviewed.
- Failed with retry.

### 17. Notifications and In-App Alerts

Purpose: Alert patterns for approval, failures, credentials, daily digest, and provider mismatches.

Must include:
- In-app notification center entries.
- Desktop notification examples.
- Email digest preview.
- Per-campaign notification settings preview.
- Alert severities: info, approval-needed, failure, credential issue, missed fallback, provider mismatch, success digest.
- Actions: Open Queue, Open Failure, Snooze, Mark Read, Configure Policy.

States:
- Daily success digest for MaVoid.
- Instant approval-needed alert.
- Instant failure alert.
- Credential issue alert.
- Provider mismatch alert.

## Cross-screen components to design

Create reusable components with variants:
- Workspace header with search, filters, primary CTA, settings.
- Brand selector and brand status badge.
- Campaign card/list row.
- Pipeline stage chip and stage timeline.
- Confidence meter and reviewer score badge.
- Autonomy toggle with safe/blocked states.
- Approval reason chips.
- Platform icon row and platform adaptation card.
- Schedule window chip and fallback warning chip.
- Agent Run linked panel.
- Automation mirror link chip.
- OmniSocials account health card.
- Evidence artifact card.
- Review report card.
- Recovery plan stepper.
- Run Now mode selector.
- Dry test checklist.
- Empty/loading/error/blocked/success state panels.
- Right inspector pattern.
- Modal, sheet, and confirmation dialog patterns.

## Required state variants

For every relevant screen/component, include:
- Empty.
- Loading/skeleton.
- Normal/healthy.
- In progress/running.
- Approval-needed.
- Warning/low confidence.
- Error/failure.
- Blocked/fail-closed.
- Recovery/retrying.
- Success/completed.
- Disabled/unavailable.
- Permission/credential issue.
- Unsaved changes.
- Mobile collapsed state where applicable.

## Accessibility and interaction requirements

- Minimum touch/click target: 44x44px.
- Keyboard navigable modals, tables, filters, tabs, and inspectors.
- Clear focus ring using Action Blue.
- Status cannot rely on color alone; use text labels and icons.
- Provide readable contrast on dark and light surfaces.
- Tables/lists must support sorting, filtering, and row selection.
- Dangerous actions require confirmation and explain impact.
- Publishing actions must disclose whether they are live, scheduled, draft-only, or correction/repost.

## Desktop and responsive requirements

Design desktop first for a macOS app at 1440px wide.
Also provide responsive adaptations for:
- 1024px small desktop/tablet landscape: collapse right inspector into a drawer where needed.
- 736-833px tablet portrait: list-first layout, tabs compress, calendar becomes agenda.
- 420-640px phone: single-column operational views, sticky bottom action bar, filters in sheet, inspectors as full-screen drill-ins.

## Exact deliverables expected from Stitch

Stitch AI should output:
1. A complete high-fidelity desktop screen set for all 17 screens listed above.
2. Mobile/responsive variants for the dashboard, calendar, content piece detail, approval queue, Run Now modal, and failure center.
3. Component library page with all cross-screen components and required variants.
4. State board covering empty, loading, error, blocked, success, approval-needed, recovery, credential issue, and unsaved changes.
5. Interaction notes for Run Now, approval flow, editor/override flow, dry test gate, recovery flow, and OmniSocials verification.
6. Visual token usage summary matching the Apple-inspired Zoid design style.
7. Clear handoff annotations naming each screen, primary actions, secondary actions, data shown, and linked workspace relationships.

## Sample copy/data to use

Use these examples in mockups:
- Brand: MaVoid.
- Campaign: MaVoid Daily Autonomous Content.
- Slot 1: Founder insight post, publish window 11:00 AM-1:00 PM.
- Slot 2: Product/market signal post, publish window 5:00 PM-7:00 PM.
- Generation start: 8:00 AM.
- Review deadline: 10:00 AM.
- Latest safe fallback: 8:30 PM.
- Confidence threshold: 85/100.
- First-run probation: first 3 successful runs require human approval.
- Platforms: LinkedIn, Instagram, X, TikTok, Facebook, Threads.
- Agent profiles: Content Strategist/Planner, Researcher, Caption/Copy Agent, Social Designer, Design Reviewer, Publisher/OmniSocials Agent, Verification Agent, Recovery Agent.
- Notification default: daily success digest plus instant failure, approval-needed, credential issue, missed fallback, and provider mismatch alerts.

## Important design reminders

- Do not invent a separate agent runner. Show linked Agent Runs and Agent Profiles from the Agents Workspace model.
- Do not make Automations the primary content management surface. It is a technical mirror with Open in Content links.
- Do not hide failures. Surface stage, reason, evidence, owner, retry path, and next action.
- Do not show raw secrets. Show credential references and verification status only.
- Do not auto-publish low-confidence or probationary content without approval.
- Do not make platform posts completely separate concepts; show one core content piece with platform-specific adaptations.
- Do not silently mutate published posts; use correction/repost workflows.
