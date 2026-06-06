# Stitch AI Prompt: Zoid Autonomous Content Automation Screens — Part 01 — Dashboard, Brand, Campaign Setup, Advanced Editor

Use this part in Stitch AI as a standalone prompt. Design only the screens listed in this part, but keep them consistent with the full Zoid Autonomous Content Automation product area.

## Shared product context

Zoid is a macOS desktop app for orchestrating agents, content, automations, publishing, logs, and review workflows. Content Workspace is the primary control surface for autonomous multi-brand social campaigns. Automations Workspace mirrors scheduler/job execution. Agents Workspace provides linked Agent Profiles and Agent Runs.

Core V1 model:
- Multi-brand hierarchy: Brand → Campaigns → Slots → Content Pieces → Platform Adaptations → Schedules/Posts.
- Daily MaVoid launch pattern: generation at 8:00 AM, two posts per day, editable publish windows, OmniSocials scheduling/publishing, verification, notifications, and recovery.
- Pipeline stages: Planning, Research, Copy, Design, Review, Adapt, Schedule, Publish, Verify, Recover.
- Safety: auto-publish only when reviewer verdict is approved, confidence >= 85/100, probation is complete, credentials verify, and no policy flag remains.
- Credentials: show Keychain-backed credential references and verification status only; never show raw secrets.
- Evidence: every autonomous action links to assets, captions, provider IDs, review reports, verification results, status timelines, and agent/automation logs.

## Existing workspace references to preserve

Agents Workspace: reuse Agent Profile selector, Active Runs, Agent Run Detail tabs, linked run panels, reviewer queue, retry/stop/send-input/log actions.
Content Workspace: reuse dashboard/calendar/content-piece/asset-library patterns, right inspector, content piece tabs, review queue, publishing history.
Automations Workspace: reuse automation list/detail/run detail patterns for schedule, enabled state, runs, logs, linked entities, failure policy, and history.

## Visual style direction

Apple-inspired Zoid style: clean, photography-first, low-chrome, desktop-first macOS UI. Primary Action Blue #0066cc. Surfaces: white #ffffff, parchment #f5f5f7, near-black tiles #272729/#2a2a2c/#252527. SF Pro/system typography. Pill CTAs/filter chips, 18px cards, 8px compact controls, 1px hairlines, minimal shadows, quiet hierarchy, precise status chips, right inspectors.

## Required global states

Use relevant variants: empty, loading/skeleton, normal/healthy, in progress/running, approval-needed, warning/low-confidence, error/failure, blocked/fail-closed, recovery/retrying, success/completed, disabled/unavailable, permission/credential issue, unsaved changes, and mobile collapsed state where applicable.

## Shared sample data

Brand: MaVoid. Campaign: MaVoid Daily Autonomous Content. Slot 1: Founder insight post, 11:00 AM-1:00 PM. Slot 2: Product/market signal post, 5:00 PM-7:00 PM. Latest safe fallback: 8:30 PM. Platforms: LinkedIn, Instagram, X, TikTok, Facebook, Threads. Agent profiles: Content Strategist/Planner, Researcher, Caption/Copy Agent, Social Designer, Design Reviewer, Publisher/OmniSocials Agent, Verification Agent, Recovery Agent.

## Screens to design in this part

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

## Component/state guidance for this part

Use the cross-screen components and state variants that apply to these screens. Do not design unrelated screens from other parts.

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

## Important design reminders

- Do not invent a separate agent runner. Show linked Agent Runs and Agent Profiles from the Agents Workspace model.
- Do not make Automations the primary content management surface. It is a technical mirror with Open in Content links.
- Do not hide failures. Surface stage, reason, evidence, owner, retry path, and next action.
- Do not show raw secrets. Show credential references and verification status only.
- Do not auto-publish low-confidence or probationary content without approval.
- Do not make platform posts completely separate concepts; show one core content piece with platform-specific adaptations.
- Do not silently mutate published posts; use correction/repost workflows.
