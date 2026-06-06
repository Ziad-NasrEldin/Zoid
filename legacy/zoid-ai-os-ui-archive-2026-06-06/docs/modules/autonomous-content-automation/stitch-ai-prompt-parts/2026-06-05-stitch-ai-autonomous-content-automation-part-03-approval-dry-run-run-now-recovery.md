# Stitch AI Prompt: Zoid Autonomous Content Automation Screens — Part 03 — Approval, Dry Test, Run Now, Recovery

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
