# Plan: Zoid Content Workspace exact user-flow map before implementation

## Goal

Correct the previous mistake: do not collapse the 16 Stitch Content Workspace screens into one catalog page.

Use the 16 Stitch screens as references to define the exact Content Workspace user-flow map first, then implement each screen as a distinct navigable screen, modal state, editor state, detail state, or mirror state inside the actual Zoid frontend.

This plan is planning-only. No implementation should continue until this flow map is accepted.

## Current context

- Repo: `/Users/ziadnasreldin/Zoid`
- Stitch project: `Zoid macOS Desktop Sitemap`, project ID `2534809720873389640`
- Core Content Workspace screens: 16
- Scope for next implementation: frontend/design only
- Explicit backend boundary: do not wire backend yet; use sample/design-copy data and preserve fail-closed/live-state truthfulness
- User correction: the prior implementation was wrong because it placed the 16 screens inside one singular page. Future implementation must treat the screens as distinct flow surfaces.

## Design principle

The Content Workspace is not a single dashboard with 16 cards.

It is a workspace with:
- a top-level command dashboard
- campaign setup and editing flows
- production/calendar/pipeline flows
- detail/editor/override flows
- approval/review flows
- run/recovery flows
- account/evidence/agent/automation support surfaces

Each Stitch screen should map to one user job and one navigable state.

## Proposed information architecture

Inside Zoid, the sidebar still has one primary workspace entry:

- `Content`

Inside `Content`, use internal tabs/routes/states:

1. `Dashboard`
2. `Campaigns`
3. `Calendar`
4. `Pipeline`
5. `Approvals`
6. `Runs`
7. `Library`
8. `Settings`

These tabs are not the 16 screens themselves. They are the navigation frame that hosts the 16 screen states.

## Exact 16-screen flow map

### 1. Autonomous Campaign Dashboard

Type: primary landing screen

Route/state:
- `Content > Dashboard`
- suggested route/state key: `content.dashboard`

Purpose:
- user sees active campaigns, automation confidence, today’s content status, risk queue, next run, blocked approvals

Primary entry points:
- sidebar `Content`
- breadcrumb back from any Content sub-screen

Primary exits:
- click campaign -> `Advanced Campaign Editor`
- click `New Campaign` -> `New Campaign Wizard`
- click pipeline item -> `Today’s Content Pipeline` or `Content Piece Detail`
- click risk/approval -> `Approval-Needed Queue`
- click failed run -> `Recovery / Failure Center`
- click next run -> `Run Now Modal`

Implementation note:
- This should be the default Content screen, not a card inside another screen.

---

### 2. Brand Management - MaVoid

Type: settings/profile screen

Route/state:
- `Content > Settings > Brands > MaVoid`
- suggested route/state key: `content.settings.brand.maVoid`

Purpose:
- manage brand voice, content pillars, allowed claims, account defaults, markets, platform constraints

Primary entry points:
- Dashboard brand selector
- Campaign Wizard brand step
- Advanced Campaign Editor brand constraints panel
- Settings tab

Primary exits:
- save brand constraints -> back to prior campaign/editor context
- switch brand -> same screen with different brand
- account mappings -> `OmniSocials & Account Mappings`

Implementation note:
- This is not a dashboard card. It is a form/settings screen with panels for voice rules, pillars, reusable constraints, and platform defaults.

---

### 3. New Campaign Wizard

Type: multi-step creation flow

Route/state:
- `Content > Campaigns > New`
- suggested route/state key: `content.campaigns.new`

Purpose:
- create a campaign through sequential steps before entering editor

Expected steps:
1. campaign goal and audience
2. brand selection / brand constraints
3. platforms and cadence
4. automation mode / approval rules
5. dry-test before activation

Primary entry points:
- Dashboard `New Campaign`
- Campaigns tab `Create`

Primary exits:
- cancel -> Dashboard or Campaigns list
- complete draft setup -> `Advanced Campaign Editor`
- run dry test -> `Dry Test Report - MaVoid Daily`

Implementation note:
- Must render as a wizard state, not as a modal card unless the Stitch reference specifically indicates compact modal behavior.

---

### 4. Advanced Campaign Editor

Type: campaign configuration/editor screen

Route/state:
- `Content > Campaigns > :campaignId > Editor`
- suggested route/state key: `content.campaigns.editor`

Purpose:
- edit campaign rules, cadence, templates, agent assignments, platform adaptation rules, approval thresholds

Primary entry points:
- Dashboard active campaign row
- Campaign Wizard completion
- Campaign Automation Mirror campaign link

Primary exits:
- open calendar -> `Content Slot Calendar`
- open pipeline -> `Today’s Content Pipeline`
- open dry test -> `Dry Test Report`
- save changes -> remain in editor with saved state

Implementation note:
- Main screen should include a left campaign structure pane, central settings/editor area, and right risk/preview/automation summary inspector.

---

### 5. Content Slot Calendar

Type: calendar/scheduling screen

Route/state:
- `Content > Calendar`
- optional campaign-filtered state: `Content > Campaigns > :campaignId > Calendar`
- suggested route/state key: `content.calendar`

Purpose:
- show content slots by day/week/month, platform, campaign, approval state, conflicts

Primary entry points:
- Content tab `Calendar`
- Campaign Editor cadence/calendar link
- Dashboard next scheduled slot

Primary exits:
- click slot -> `Content Piece Detail & Adaptations`
- click empty slot -> `New Campaign Wizard` or quick slot draft state
- click blocked slot -> `Approval-Needed Queue` or `Recovery Center`

Implementation note:
- Needs actual calendar grid/slot layout, not an item list.

---

### 6. Today’s Content Pipeline

Type: kanban/production board screen

Route/state:
- `Content > Pipeline > Today`
- suggested route/state key: `content.pipeline.today`

Purpose:
- show today’s content moving through brief, draft, design, review, adaptation, scheduled, published/blocked

Primary entry points:
- Dashboard today pipeline summary
- Content tab `Pipeline`
- Calendar day header

Primary exits:
- click content card -> `Content Piece Detail & Adaptations`
- click blocked/review card -> `Approval-Needed Queue`
- click failed execution -> `Recovery / Failure Center`

Implementation note:
- This should be a real board with columns and cards.

---

### 7. Content Piece Detail & Adaptations

Type: detail screen

Route/state:
- `Content > Pieces > :pieceId`
- suggested route/state key: `content.pieces.detail`

Purpose:
- inspect one content piece, source brief, status, platform adaptations, artifacts, run history, approvals

Primary entry points:
- Pipeline card
- Calendar slot
- Evidence library artifact link
- Approval queue item

Primary exits:
- edit copy -> `Content Editor / Override Flow`
- approve/reject -> `Approval-Needed Queue` outcome or return to detail
- open artifacts -> `Evidence & Artifact Library`
- schedule/run -> `Run Now Modal`

Implementation note:
- Should be a full detail page with tabs/side panels, not just an inspector card.

---

### 8. Content Editor / Override Flow

Type: editor state / focused editing screen

Route/state:
- `Content > Pieces > :pieceId > Edit`
- suggested route/state key: `content.pieces.edit`

Purpose:
- edit original copy, override AI output, compare versions, adapt per platform, record reason for manual change

Primary entry points:
- Piece Detail `Edit`
- Approval Queue `Override`
- Pipeline card quick action

Primary exits:
- save draft -> back to Piece Detail
- submit review -> `Approval-Needed Queue`
- discard -> back to Piece Detail

Implementation note:
- Needs editor layout: central text editor, platform adaptation tabs, version/diff rail, reviewer notes.

---

### 9. Approval-Needed Queue

Type: review queue screen

Route/state:
- `Content > Approvals`
- suggested route/state key: `content.approvals.queue`

Purpose:
- list all content requiring human decision: low-confidence, risky claims, blocked special categories, account mismatch, failed dry test

Primary entry points:
- Dashboard risk queue
- Pipeline blocked column
- Piece Detail approval banner
- Dry Test Report failure

Primary exits:
- open item -> `Content Piece Detail`
- override -> `Content Editor / Override Flow`
- approve/reject -> stays in queue with item removed/updated

Implementation note:
- Must be a queue screen with filters, severity, reasons, and batch actions.

---

### 10. Dry Test Report - MaVoid Daily

Type: report/results screen

Route/state:
- `Content > Runs > Dry Test > :runId`
- suggested route/state key: `content.runs.dryTestReport`

Purpose:
- preview what the autonomous system would generate/schedule/publish without external writes

Primary entry points:
- New Campaign Wizard final step
- Campaign Editor `Run dry test`
- Run Now Modal dry-run option

Primary exits:
- approve campaign activation -> Campaign Editor or Dashboard
- fix issues -> Approval Queue / Editor / Recovery Center
- view generated content -> Piece Detail

Implementation note:
- Needs report sections: generated outputs, platform adaptations, policy checks, blocked writes, confidence, recommendations.

---

### 11. Run Now Modal

Type: modal/confirmation state

Route/state:
- overlay on current screen
- suggested state key: `content.runs.runNowModal`

Purpose:
- confirm a controlled run/dry-run/retry before any automation starts

Primary entry points:
- Dashboard next run
- Campaign Editor run button
- Piece Detail run/schedule button
- Recovery Center retry button

Primary exits:
- cancel -> previous screen
- confirm dry run -> `Dry Test Report`
- confirm controlled run -> `Agent Execution & Notifications`

Implementation note:
- This is correctly a modal, not a standalone page.
- Must explicitly say no external publish unless backend integration/approval rules later allow it.

---

### 12. Recovery / Failure Center

Type: operations recovery screen

Route/state:
- `Content > Runs > Recovery`
- suggested route/state key: `content.runs.recovery`

Purpose:
- diagnose failed content automation runs, blocked integrations, failed agent steps, missing approvals, invalid account mappings

Primary entry points:
- Dashboard failed run/risk queue
- Agent Execution failure row
- Dry Test Report failure section
- Automation Mirror failed schedule

Primary exits:
- retry controlled run -> `Run Now Modal`
- edit mapping -> `OmniSocials & Account Mappings`
- edit content -> `Content Editor / Override Flow`
- view artifacts/logs -> `Evidence & Artifact Library`

Implementation note:
- Should be a timeline + failure list + recovery actions screen.

---

### 13. OmniSocials & Account Mappings

Type: integration/settings screen

Route/state:
- `Content > Settings > OmniSocials`
- suggested route/state key: `content.settings.omnisocials`

Purpose:
- map brands/campaigns/platforms to social accounts, validate availability, show fail-closed status

Primary entry points:
- Settings tab
- Brand Management platform defaults
- Recovery Center integration failure
- Campaign Editor platform mapping warning

Primary exits:
- fix mapping -> back to prior context
- view affected campaigns -> Campaign Editor / Dashboard

Implementation note:
- Frontend-only pass must not expose credentials or imply live connected publishing.
- Show mappings/status, not secret fields.

---

### 14. Evidence & Artifact Library

Type: searchable library screen

Route/state:
- `Content > Library`
- suggested route/state key: `content.library.evidence`

Purpose:
- store and inspect generated drafts, screenshots/designs, run logs, review decisions, dry-test evidence, approval artifacts

Primary entry points:
- Content tab `Library`
- Piece Detail artifacts
- Dry Test Report output evidence
- Recovery Center logs

Primary exits:
- open artifact -> artifact detail drawer/state
- linked content -> Content Piece Detail
- linked run -> Agent Execution / Dry Test Report

Implementation note:
- Needs grid/list, filters, preview panel, linked entity context.

---

### 15. Agent Execution & Notifications

Type: run execution monitor screen

Route/state:
- `Content > Runs > :runId > Execution`
- suggested route/state key: `content.runs.execution`

Purpose:
- show planner/research/copy/designer/reviewer/publisher/verifier/recovery agent steps, statuses, notifications, blockers

Primary entry points:
- Run Now Modal confirm
- Dashboard active run
- Automation Mirror run history
- Recovery Center failed run

Primary exits:
- failed step -> Recovery Center
- generated piece -> Content Piece Detail
- artifacts -> Evidence Library
- automation schedule -> Campaign Automation Mirror

Implementation note:
- This should be a run timeline/agent-step monitor, distinct from the broader Agents Workspace.

---

### 16. Campaign Automation Mirror

Type: automation mirror/support screen

Route/state:
- `Content > Runs > Automation Mirror`
- or mirrored from `Automations > Content Campaigns`
- suggested route/state key: `content.runs.automationMirror`

Purpose:
- show recurring campaign automation schedules, last run, next run, manual override, pause/probation status

Primary entry points:
- Dashboard automation summary
- Campaign Editor automation section
- Automations workspace mirror link

Primary exits:
- open campaign -> Advanced Campaign Editor
- open run -> Agent Execution
- failed automation -> Recovery Center
- run now -> Run Now Modal

Implementation note:
- This screen bridges Content and Automations but should remain accessible inside Content.

## End-to-end user journeys

### Journey A: Create and activate a new autonomous campaign

1. Dashboard
2. New Campaign Wizard
3. Brand Management, if brand rules need editing
4. Advanced Campaign Editor
5. Dry Test Report
6. Approval-Needed Queue, only if issues exist
7. Campaign Automation Mirror
8. Dashboard

### Journey B: Review today’s content and override one piece

1. Dashboard
2. Today’s Content Pipeline
3. Content Piece Detail & Adaptations
4. Content Editor / Override Flow
5. Approval-Needed Queue
6. Content Slot Calendar or Pipeline

### Journey C: Diagnose a failed automation run

1. Dashboard failure card
2. Recovery / Failure Center
3. Agent Execution & Notifications
4. Evidence & Artifact Library, if logs/artifacts needed
5. OmniSocials & Account Mappings, if account issue
6. Run Now Modal for controlled retry
7. Dry Test Report or Agent Execution

### Journey D: Manage brand/account setup

1. Dashboard or Settings
2. Brand Management - MaVoid
3. OmniSocials & Account Mappings
4. Campaign Editor affected campaign
5. Dry Test Report

### Journey E: Inspect published/scheduled evidence without backend writes

1. Dashboard
2. Evidence & Artifact Library
3. Content Piece Detail & Adaptations
4. Dry Test Report or Agent Execution
5. Recovery Center, if evidence links to failure

## Navigation model to implement after approval

Recommended frontend state model:

```ts
type ContentScreen =
  | "dashboard"
  | "brand-management"
  | "new-campaign"
  | "campaign-editor"
  | "slot-calendar"
  | "today-pipeline"
  | "piece-detail"
  | "piece-editor"
  | "approval-queue"
  | "dry-test-report"
  | "run-now-modal"
  | "recovery-center"
  | "omnisocials-mappings"
  | "evidence-library"
  | "agent-execution"
  | "automation-mirror";
```

Recommended local UI state:

```ts
type ContentWorkspaceUiState = {
  screen: ContentScreen;
  previousScreen?: ContentScreen;
  selectedCampaignId?: string;
  selectedPieceId?: string;
  selectedRunId?: string;
  selectedBrandId?: string;
  modal?: "run-now" | null;
};
```

The first implementation can use local React state instead of URL routes if Zoid does not currently have a router. But each screen must render as its own full screen/state, not as one dashboard containing all screens.

## Proposed implementation sequence after this plan is accepted

### Step 0: Undo/replace wrong one-page catalog behavior

- Remove or stop using the previous `content-screen-grid` catalog as the main implementation.
- Keep useful data only if it helps populate individual screens.
- Do not leave all 16 screens visible at once as the primary UX.

### Step 1: Build Content workspace navigation shell

Likely files:
- `/Users/ziadnasreldin/Zoid/src/App.tsx`
- `/Users/ziadnasreldin/Zoid/src/App.css`
- `/Users/ziadnasreldin/Zoid/src/contentWorkspace.ts`

Tasks:
- add internal Content nav/tabs
- add breadcrumb/back behavior
- add `ContentWorkspaceUiState`
- render exactly one primary Content screen at a time, except modal overlays

### Step 2: Implement top-level operational screens

Implement:
1. Autonomous Campaign Dashboard
2. Today’s Content Pipeline
3. Content Slot Calendar
4. Approval-Needed Queue

Reason:
- These define the main user flow and navigation behavior.

### Step 3: Implement campaign setup/editing screens

Implement:
5. New Campaign Wizard
6. Advanced Campaign Editor
7. Brand Management - MaVoid
8. OmniSocials & Account Mappings

Reason:
- These define how campaigns and constraints are created and configured.

### Step 4: Implement content detail/edit/review screens

Implement:
9. Content Piece Detail & Adaptations
10. Content Editor / Override Flow
11. Evidence & Artifact Library

Reason:
- These define how individual content is inspected, edited, and evidenced.

### Step 5: Implement run/recovery/automation screens

Implement:
12. Run Now Modal
13. Dry Test Report - MaVoid Daily
14. Agent Execution & Notifications
15. Recovery / Failure Center
16. Campaign Automation Mirror

Reason:
- These define automation control and failure handling.

### Step 6: Integrate flow transitions

Add buttons/links matching the flow map:
- Dashboard -> New Campaign Wizard
- Dashboard -> Campaign Editor
- Dashboard -> Pipeline
- Dashboard -> Approval Queue
- Dashboard -> Recovery Center
- Wizard -> Editor -> Dry Test Report
- Pipeline -> Piece Detail -> Editor/Override
- Approval Queue -> Piece Detail/Editor
- Run Now -> Agent Execution or Dry Test Report
- Agent Execution -> Recovery/Evidence/Piece Detail
- Automation Mirror -> Run Now/Execution/Recovery

### Step 7: Preserve frontend-only truthfulness

Every screen that references automation/publishing must keep clear copy:
- design/sample data only
- no backend connected yet
- no external publishing implied
- fail-closed until integrations are wired
- credentials/secrets never displayed

## Files likely to change after approval

Likely direct changes:
- `/Users/ziadnasreldin/Zoid/src/App.tsx`
- `/Users/ziadnasreldin/Zoid/src/App.css`
- `/Users/ziadnasreldin/Zoid/src/contentWorkspace.ts`
- `/Users/ziadnasreldin/Zoid/src/contentWorkspace.test.ts`

Possible better structure if we split components:
- `/Users/ziadnasreldin/Zoid/src/content/ContentWorkspace.tsx`
- `/Users/ziadnasreldin/Zoid/src/content/contentWorkspaceFlow.ts`
- `/Users/ziadnasreldin/Zoid/src/content/contentWorkspaceScreens.tsx`
- `/Users/ziadnasreldin/Zoid/src/content/contentWorkspaceSampleData.ts`
- `/Users/ziadnasreldin/Zoid/src/content/contentWorkspace.test.ts`

Recommendation:
- Split into `src/content/` if the current `App.tsx` is already too large.
- Keep `App.tsx` as shell/routing only.

## Tests and validation after implementation

### Unit/view-model tests

Add tests for:
- all 16 screen IDs exist
- only one primary screen renders at a time
- `Run Now Modal` overlays previous screen instead of replacing it
- each screen has required outgoing transitions
- no transition leads to a missing screen
- backend/publishing truthfulness copy exists on relevant screens

### Browser smoke tests

Verify in browser:
- open Content -> Dashboard appears
- navigate through each of the 16 screens/states
- assert each screen has a unique heading and unique layout
- assert not all 16 are visible at once
- assert modal state overlays correctly
- assert console has no JS errors

### Build checks

Run:
- `npm run test:frontend`
- `npm run build`

### Critique workflow

Before final handoff:
- create `.hermes/reviews/content-workspace-user-flow-frontend/handoff.md`
- run separate critique review
- require `APPROVED`

Critique must specifically check:
- screens are not collapsed into one catalog page
- user flow map is implemented
- all 16 screens/states are reachable
- frontend-only truthfulness is preserved
- no backend/live publishing implication

## Risks / tradeoffs

1. App may not currently use URL routing.
   - Mitigation: use internal state first, but structure it so URL routes can be added later.

2. Too many components in `App.tsx`.
   - Mitigation: split Content workspace screens into `src/content/` components.

3. Sample data may feel fake if not labelled.
   - Mitigation: keep a compact persistent “frontend preview / backend not linked yet” disclosure.

4. Automations and Content overlap.
   - Mitigation: Content owns campaign context; Automations mirror shows recurring run schedule and run history.

5. Agents Workspace overlap.
   - Mitigation: Agent Execution inside Content shows content-run agent steps only; full agent management remains in Agents Workspace.

## Open questions before implementation

1. Should the 16 screens be URL-addressable now, or is internal React state acceptable for this frontend pass?
   - Recommendation: internal state for this pass unless Zoid already has routing conventions.

2. Should `Campaign Automation Mirror` live under Content only, or also add a cross-link from Automations?
   - Recommendation: implement inside Content now, add Automations cross-link later if needed.

3. Should `Run Now Modal` support both dry-run and real-run modes visually now?
   - Recommendation: yes visually, but label real-run as disabled/not connected until backend is wired.

4. Should the wrong previous one-page screen be removed entirely or kept as an internal sitemap/debug view?
   - Recommendation: remove from user-facing Content. If kept, hide behind a dev-only/reference state.

## Definition of done for the next implementation pass

- Content workspace no longer presents all 16 screens as one singular screen.
- Each of the 16 Stitch screens has a distinct screen/state/modal implementation.
- User can navigate the major journeys from Dashboard through creation, editing, review, run, recovery, and evidence flows.
- Only one primary screen is visible at a time, except modal overlays.
- Frontend-only/backend-not-linked truthfulness is visible.
- `npm run test:frontend` passes.
- `npm run build` passes.
- Browser smoke proves all 16 screens/states are reachable and not collapsed.
- Critique workflow returns `APPROVED`.
