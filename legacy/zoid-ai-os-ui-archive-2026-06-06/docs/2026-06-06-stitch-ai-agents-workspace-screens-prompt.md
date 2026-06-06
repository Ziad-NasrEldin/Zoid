# Stitch AI Prompt: Zoid Agents Workspace Screens

Use this prompt in Stitch AI to design the complete Agents Workspace screen set for Zoid.

## Product context

Zoid is a macOS desktop AI operating system for managing code, agents, content, automations, files, notes, reviews, tasks, and local/native workflows. The Agents Workspace is the visual operating system for Hermes Agent sessions. It turns raw Hermes TUI conversations into a persistent, repo-aware, multi-session, multi-panel, clean chat workspace.

The main job: help Ziad manage many live Hermes agent sessions without losing track of context, repo ownership, process state, outputs, reviews, or follow-ups.

This design must prioritize real work, not decorative dashboards. It should feel like a native macOS productivity surface: calm, readable, fast, precise.

## Existing workspace references to preserve

### Agents Workspace reference

Existing screen reference: `/Users/ziadnasreldin/Zoid/Docs/designer-screen-reference/03-agents-workspace.md`

Original concepts included:

- Agents Dashboard
- New Agent Run Modal
- Active Runs
- Agent Run Detail
- Agent Profiles
- Reviewer Agent

For the new design, reinterpret these around a chat-first session OS:

- Main screen is not an analytics dashboard first.
- Main screen is a persistent multi-session chat workspace.
- Agent Run Detail concepts become right inspector tabs and session metadata.
- Agent Profiles power avatars, model defaults, permissions, commands, workdirs, and themes.
- Reviewer Agent becomes a linked session/workflow visible in the rail and inspector.

### Content Workspace reference

Existing screen reference: `/Users/ziadnasreldin/Zoid/Docs/designer-screen-reference/05-content-workspace.md`

Reuse the pattern of:

- workspace header with search, new item/action, settings
- right inspector pattern
- status chips and queues
- detail tabs
- active pipeline/status visibility

### Code Workspace relationship

Agents Workspace must link sessions to repositories created/managed in the Code Workspace.

Repo linking should show:

- repo/project name
- branch
- git status
- working directory
- selected files/context
- current diff/plans/reviews/tasks where available

## Visual style direction

Use Zoid's existing Apple-inspired design system:

- Clean, low-chrome, native macOS feel.
- Do not copy the dark cyber aesthetic from the screenshot literally.
- Take inspiration from Apple iMessage for message clarity, spacing, and readable chat bubbles.
- Use Zoid surfaces: white, parchment/light gray, quiet near-black only for high-emphasis previews/status where appropriate.
- Primary action color: Action Blue #0066cc.
- Typography: SF Pro Display / SF Pro Text or system equivalent.
- Use pill CTAs, filter chips, 18px utility cards, 8px compact controls, 1px hairline borders.
- Avoid heavy SaaS chrome, heavy shadows, decorative gradients, and dense terminal output.
- Desktop-first macOS app at 1440px wide, with responsive collapsed rail/inspector behavior.

## UX principles

1. Chat-first, not dashboard-first.
   - Opening Agents should restore the last active session or multi-panel layout.
   - Summary counters are small in the header/rail, not big dashboard cards.

2. Repo/project grouping first.
   - The left rail groups sessions by repo/project by default.
   - Recent-first sorting can exist inside each repo group.

3. Persistent sessions, not fragile tabs.
   - Selecting a session instantly continues it.
   - Closing a panel does not stop/archive the session.
   - Sessions preserve transcript, repo link, process state, logs, layout, reviews, and context.

4. Clean chat, operational inspector.
   - Chat stays readable and iMessage-inspired.
   - Tool/file/command events live primarily in the right activity inspector.
   - Critical alerts also appear inline as compact cards.

5. Multi-agent work must be visible.
   - Support 1, 2, 3, and 4 panel layouts.
   - Each panel is a live session with independent composer and status.

6. Real Hermes process state must be present but minimal.
   - Show clean status/process strip.
   - Raw TUI/logs are hidden by default and available in Raw tab.

7. Model switching should preserve continuity.
   - Show a model selector.
   - Same-session model handoff is the default.
   - Optional fork/compare action exists.

8. Reviews are first-class.
   - Reviewer sessions are linked runs with verdicts and required fixes.
   - Review state appears in the right inspector and left rail.

## Core data and status model to reflect visually

Entities:

- Agent Session
- Agent Profile
- Agent Process
- Repo Link
- Message
- Process Event
- File Event
- Command Event
- Context Attachment
- Review Record
- Handoff Summary

Session types:

- Chat session
- Background run
- Reviewer run
- Watcher/cron run

Session statuses:

- Active
- Idle
- Running tool
- Waiting for input
- Needs me
- Blocked
- Permission needed
- Approval needed
- Review required
- Completed
- Stopped/resume available
- Crashed
- Archived

Health warnings:

- Context near limit
- Stale repo/context changed
- Process crashed
- Permission blocked
- Raw logs unavailable

## Screens to design

Design every screen below. Include realistic sample data for Zoid, MaVoid Site, Leadra, Kalima, and a generic client repo to prove repo grouping scales.

### 01. Agents Workspace — Chat-First Main Screen

Purpose: Main working surface for real Hermes sessions.

Must include:

- Top header: Agents, New Session, layout switcher, global search, needs-me queue, profile/model controls, settings.
- Left rail grouped by repo/project.
- Center selected live chat session.
- Right activity inspector.
- Clean process strip above or below chat.
- Composer at bottom.
- Current session header with avatar, title, agent profile, model badge, repo badge, branch/status, health.

Sample left rail groups:

- Zoid
  - Agents Workspace PRD — Hermes — GPT-5.5 — Running tool
  - Reviewer: Agents Workspace — Reviewer — Needs fixes
- MaVoid Site
  - Review deployment — Hermes — Waiting for input
- Leadra
  - Fix Redis cache bug — Hermes — Idle
- Kalima
  - Debug auth flow — Hermes — Blocked

States:

- Empty: no sessions yet, create first Hermes session.
- Loading/skeleton while sessions load.
- Process stopped/resume available.
- Needs me queue has items.
- No repo linked yet.
- Repo stale warning.

### 02. New Agent Session Modal

Purpose: Fast creation flow for a real Hermes-backed session.

Fields:

- Agent profile: Hermes, Reviewer, Coding, Designer, Custom
- Model/provider selector
- Linked repo selector from Code Workspace
- Session title
- Prompt/initial instruction
- Context attachments: files, docs, plan, screenshot, issue, task
- Permissions preview
- Review requirement: none, reviewer required, approval required before changes
- Layout choice: current panel, new panel, background

One-click presets:

- Start coding session
- Review this repo
- Debug issue
- Design screen
- Continue previous work
- Create PR

States:

- Hermes not configured.
- Repo missing/not selected.
- Permission preview warning.
- Review required enabled.
- Start disabled until required fields valid.

### 03. Multi-Panel Session Workspace

Purpose: View and work with up to 4 live sessions at once.

Layouts to design:

- 1 focused panel
- 2 side-by-side
- 3: one large primary + two stacked secondary
- 4: 2x2 grid

Each panel includes:

- session header
- avatar/profile/model/repo/status
- compact chat
- independent composer
- process strip
- panel actions: focus, swap, close/remove, raw, inspector target

Interactions:

- Drag/drop session from left rail into panel.
- Close panel without stopping session.
- Select panel to drive right inspector.
- Broadcast prompt to selected panels from composer/target selector.

States:

- Empty panel slot.
- One panel waiting for input while another runs.
- One panel crashed/resume available.
- Inspector collapsed due to 4-panel density.

### 04. Repo-Grouped Session Rail

Purpose: Manage and find sessions primarily by repo/project.

Must include:

- Repo/project groups with counts and health indicators.
- Session cards with avatar, title, model, status, last preview, attention dot.
- Pinned sessions section.
- Needs-me section.
- Filters: Active, Blocked, Needs Me, Pinned, Reviewer, By repo, By profile, Recent.
- Hover actions: open in panel, pin, rename, stop, duplicate/fork, archive.
- Search input across transcript/repo/model/status/file path.

States:

- Repo group collapsed/expanded.
- No sessions in repo.
- Session archived.
- Session pinned.
- Session waiting for input.

### 05. Right Activity Inspector

Purpose: Keep chat clean while surfacing all operational detail.

Tabs:

- Activity: tools, commands, file reads/writes, status timeline, current process.
- Files: changed/read files, diffs, open/reveal buttons.
- Repo: repo path, branch, git status, tests, PR/task links, workdir.
- Context: attached docs, screenshots, files, plans, issue/task/context pack.
- Review: reviewer verdict, required fixes, approvals, review history.
- Raw: raw Hermes TUI/logs fallback.

States:

- Empty tab.
- Running command.
- File diff available.
- Permission request.
- Blocked state.
- Reviewer Required fixes.
- Raw logs collapsed/expanded.

### 06. Agent Profile Manager

Purpose: Configure profiles that define identity, runner, model, permissions, and avatars.

Must include:

- Profile list: Hermes, Reviewer, Coding, Designer, Custom.
- Profile detail form:
  - avatar image/icon
  - generated avatar option
  - display name
  - role label
  - default model/provider
  - command/runner
  - default repo/workdir behavior
  - permissions
  - visual accent/theme
- Recent sessions using profile.
- Actions: Save, Test Profile, Duplicate, Disable, Delete, Set as Reviewer.

States:

- Missing runner command.
- Model unavailable.
- Avatar overridden by session.
- Profile disabled.

### 07. Model Switch / Handoff Flow

Purpose: Change model quickly without destroying session continuity.

Must include:

- Model dropdown in session header.
- Current model/provider badge.
- Handoff confirmation sheet when hot switch unsupported.
- Timeline marker: switched from model A to model B.
- Optional “Fork with new model” action.
- Handoff progress: preserving transcript, stopping old process, starting new process, injecting context, ready.

States:

- Hot switch supported.
- Handoff required.
- Handoff failed with rollback/resume options.
- Fork created.

### 08. Composer and Slash Command States

Purpose: Power input surface for sessions and panels.

Must include:

- Text input.
- Attach file/doc/screenshot/image.
- Paste image preview.
- Repo file/context drop zone.
- Slash command menu: `/review`, `/run-tests`, `/link-repo`, `/switch-model`, `/handoff`, `/plan`.
- Send target selector for multi-panel mode.
- Broadcast to selected sessions.
- Prompt templates/presets.
- Approval response buttons.
- Toggle: send as instruction vs note/context.

States:

- Attachment queued.
- Invalid file/context.
- Broadcast selected sessions.
- Agent waiting for approval.
- Slash command autocomplete.

### 09. Reviewer Workflow Screen/Inspector State

Purpose: Run reviewer agents and close Required fixes loops.

Must include:

- Request Review action from a coding session.
- Linked reviewer session card.
- Review verdict card: Approved, Required fixes, Blocked.
- Required fixes list.
- Send fixes back to original session.
- Mark fix complete.
- Re-request review.
- Review history timeline.

States:

- Review running.
- Required fixes.
- Approved.
- Blocked.
- Re-review requested.

### 10. Handoff / Compact Session Flow

Purpose: Preserve context when long sessions approach limits.

Must include:

- Context meter warning.
- Quick Handoff/Compact action.
- Generated handoff summary preview.
- Include/exclude options: clean output, decisions, files changed, open tasks, raw logs link, review status, repo status.
- Save/export/copy handoff.
- Resume with compacted context.

States:

- Context near limit.
- Handoff generated.
- Handoff export copied.
- Resume from handoff.

### 11. Search / Archive / Export

Purpose: Find and preserve past work.

Must include:

- Global search across all transcripts.
- Filters by repo, model, profile, status, file path, tag, archived.
- Result rows with session title, repo, date, matched excerpt, status.
- Archive management.
- Export options: clean output, raw logs, handoff summary.

States:

- No results.
- Archived result.
- Export in progress.
- Export complete.

### 12. Native Verification / Diagnostics Screen

Purpose: Internal/test surface or diagnostic panel for proving the feature works.

Must include:

- Hermes runner status.
- Active process list.
- App-support DB/log paths.
- Last verification checklist.
- Native/Tauri app indicator.
- Browser preview warning if not native.
- Actions: Create test session, Restart session, Open logs, Reveal app data, Run verification checklist.

States:

- Native app running.
- Browser preview only warning.
- Hermes command unavailable.
- DB/log unavailable.

## Cross-screen components to design

Create reusable components with variants:

- Repo-grouped session rail.
- Session card.
- Agent avatar/profile badge.
- Model/provider selector.
- Repo badge with branch/status.
- Process strip.
- Health warning chip.
- Needs-me indicator.
- Multi-panel layout switcher.
- Chat bubble: user, agent, system, alert.
- Composer with slash menu/attachments/target selector.
- Right inspector tabs.
- Activity timeline item.
- File diff row.
- Review verdict card.
- Permission request card.
- Handoff summary card.
- Raw logs viewer.
- Empty/loading/error/blocked/success states.
- Modal/sheet/confirmation patterns.

## Required state variants

For relevant screens/components include:

- Empty
- Loading/skeleton
- Normal/healthy
- Running
- Waiting for input
- Needs me
- Permission needed
- Approval needed
- Blocked
- Crashed
- Stopped/resume available
- Context near limit
- Repo stale
- Review required
- Required fixes
- Approved
- Archived
- Broadcast selected
- Raw logs expanded
- Native verification warning

## Accessibility and interaction requirements

- Minimum click/touch target 44x44px.
- Keyboard navigable rail, panels, composer, slash menu, tabs, modals, inspector.
- Clear focus ring using Action Blue.
- Status cannot rely on color alone; use labels/icons.
- High readability for long coding/debugging sessions.
- Dangerous actions require confirmation and explain impact.
- Model handoff and repo switching must disclose context impact.
- Raw logs must be copyable/selectable.

## Desktop and responsive requirements

Design desktop first for a macOS app at 1440px wide.

Responsive behavior:

- 1024px: collapse inspector into drawer; keep left rail compact.
- 736–833px: list-first sessions, one panel at a time, inspector as drill-in.
- 420–640px: single-column chat, session rail as sheet, inspector as full-screen tabs.

## Exact deliverables expected from Stitch

Stitch AI should output:

1. High-fidelity desktop screen set for all 12 screens listed above.
2. Multi-panel desktop variants: 1, 2, 3, and 4 panels.
3. Component library page with all cross-screen components and variants.
4. State board covering empty, loading, running, waiting, needs-me, blocked, permission, approval, crashed, stale repo, context warning, review required, archived, and success.
5. Responsive variants for main chat workspace, multi-panel workspace, new session modal, right inspector, and search/archive.
6. Interaction notes for new session, repo link, model handoff, multi-panel drag/drop, broadcast prompt, reviewer workflow, handoff/compact, and raw logs fallback.
7. Visual token usage summary matching Zoid's Apple-inspired design system.
8. Clear handoff annotations naming each screen, primary actions, secondary actions, data shown, and workspace relationships.

## Sample copy/data to use

Use these examples in mockups:

- Repo/project: Zoid
  - Session: Agents Workspace PRD
  - Agent: Hermes Agent
  - Model: GPT-5.5
  - Status: Running tool
  - Branch: main

- Repo/project: MaVoid Site
  - Session: Review deployment
  - Agent: Reviewer
  - Model: Claude Sonnet
  - Status: Waiting for input

- Repo/project: Leadra
  - Session: Fix Redis cache bug
  - Agent: Hermes Agent
  - Status: Idle

- Repo/project: Kalima
  - Session: Debug auth flow
  - Agent: Coding Agent
  - Status: Blocked — needs API key

Example chat:

- User: “Start a focused implementation plan for the Agents Workspace.”
- Agent: “I’ll inspect the repo, identify the existing workspace structure, and create the tracker before editing.”
- Activity inspector: “Read Docs/designer-screen-reference/03-agents-workspace.md”
- File event: “Created Docs/2026-06-06-zoid-agents-workspace-prd.md”
- Review: “Required fixes: add native verification gate.”

## Important design reminders

- Do not make the main screen a dashboard-first analytics page.
- Do not show raw terminal UI by default.
- Do not hide repo/project ownership.
- Do not make sessions fragile browser tabs; they are persistent process-backed work units.
- Do not stop sessions when closing a panel.
- Do not defer multi-panel, right inspector, model handoff, avatars, composer power features, reviewer workflow, search/archive/export, or native verification from the requested scope.
- Do not create fake runtime records. Designs should imply real Hermes-backed state and truthful empty/blocked states.
- Do not break Zoid's existing design language; use iMessage as inspiration for chat readability, not as a full visual clone.
