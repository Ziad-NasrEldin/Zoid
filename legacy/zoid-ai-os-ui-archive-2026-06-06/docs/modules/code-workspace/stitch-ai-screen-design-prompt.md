# Stitch AI Prompt: Zoid Code Workspace Screens

Use this prompt in Stitch AI to design the complete Code Workspace screen set for Zoid.

## Product context

Zoid is a macOS desktop AI operating system for managing code, agents, content, automations, files, notes, reviews, tasks, and local/native workflows. The Code Workspace is the repo command center. It discovers and manages approved local Git repositories, shows repo health, links repos to Agents Workspace sessions, tracks checks/branches/PRs/deployments, and enforces launch gates before work is considered shipped.

The main job: help Ziad know which repos are safe, dirty, blocked, deployed, verified, or needing attention without jumping between Finder, terminal, GitHub, deployment dashboards, and Hermes sessions.

## Existing reference

Screen reference: `/Users/ziadnasreldin/Zoid/Docs/designer-screen-reference/02-code-workspace.md`

Original screen concepts:

- Code Dashboard
- Repo Discovery
- Managed Repositories
- Repository Detail
- Launch Gate

Keep these, but make them feel like a native Zoid repo command center.

## Visual style direction

Use Zoid's Apple-inspired design system:

- Native macOS desktop feel.
- Apple Finder-style project browser plus operational health dashboard.
- Clean, compact rows by default; optional card/grid view.
- Right inspector like Finder/Apple Settings detail panel.
- Low chrome, hairline borders, status chips, pill controls, Action Blue #0066cc.
- SF Pro / system typography.
- Avoid enterprise-table heaviness.
- Avoid raw terminal output unless expanded.
- Align visually with Agents Workspace.

## Core UX principles

1. Repo command center, not full code editor.
   - Show status/actions, not a full embedded IDE.
   - Link out to Finder/editor/terminal for deep browsing/editing.

2. Explicit user approval.
   - Scan only folders the user adds.
   - Discovered repos become managed only after approval.
   - Ignored repos stay recoverable in an Ignored tab.

3. Product/client grouping first.
   - Main repo list grouped by product/client.
   - Filter by profile/type/status/recent.

4. Evidence-based launch gates.
   - Mark Verified requires required evidence.
   - Manual override requires explicit reason and is recorded as override.

5. Agents are linked, not separate chaos.
   - Starting an agent from a repo prelinks workdir/context/rules.
   - Linked sessions appear in Code and Agents Workspace.

6. Safety first.
   - Git/deploy/destructive actions are confirmation-gated.
   - Secrets/env files are sensitive and excluded from default agent context.

7. Native verification matters.
   - Designs must imply real local repos, real git state, real checks, and real native app verification.

## Core data/status to represent

Entities:

- Scan Root
- Discovered Repo
- Managed Repo
- Ignored Repo
- Repo Profile
- Repo Settings
- Repo Permission
- Repo Status Snapshot
- Repo Event
- Agent Session Link
- Launch Gate
- Evidence Item
- Deployment Record
- GitHub Connection
- PR / Issue Link
- Handoff Export

Repo mission status:

- Safe
- Dirty
- Blocked
- Ready to Review
- Ready to Launch
- Verified

Attention states:

- Failed checks
- Launch gate blocked/failed
- Deployment unverified / production verification stale
- Active agent waiting for input
- Dirty/uncommitted changes
- Risky files or secrets/config changed
- Branch behind/ahead
- PR failing / awaiting review
- Repo path missing/moved
- Deploy/process crashed

## Screens to design

Design every screen below with realistic sample data for Zoid, MaVoid Site, Leadra, Kalima, and a disposable test repo.

### 01. Code Workspace — Repo Health Command Center

Purpose: Main landing screen.

Must include:

- Header: Code Workspace, Search Repos, Add Repo, Scan Folders, Refresh Status, Code Settings.
- Small summary counters: managed repos, dirty repos, open launch gates, failed checks, needs-me.
- Main repo list grouped by product/client.
- Attention queue.
- Active code agent sessions.
- Right inspector for selected repo.
- Quick actions: Open Repo, Start Agent, Run Checks, View Diff, Launch Gate.
- View toggle: compact rows / card grid.

Sample groups:

- Zoid
  - Zoid App — Dirty — Failed checks — Agent waiting
  - Zoid Disposable Test Repo — Safe
- MaVoid
  - MaVoid Site — Ready to Launch — Deployment unverified
- Leadra
  - Leadra Backend — Blocked — Failed tests
- Kalima
  - Kalima Portal — Dirty — Review required

States:

- Empty: no scan roots yet.
- Loading refresh.
- No managed repos but discovered repos available.
- Needs-me queue non-empty.
- Repo path missing.

### 02. Repo Discovery / Scan Folders

Purpose: Add scan roots and approve discovered repos.

Must include:

- Scan source panel.
- Add Scan Folder action.
- Run Scan action.
- Suggested repos list.
- Bulk actions bar: Approve Selected, Ignore Selected, Add to group/client, Set profile.
- Repo preview inspector.
- Ignored tab with Restore action.
- Warning: Zoid only scans explicitly added folders.

Repo row data:

- repo name
- path
- detected profile
- remote
- last modified
- dirty state
- duplicate/nested/monorepo warnings

States:

- Permission prompt.
- Scan running.
- No repos found.
- Duplicate repo warning.
- Ignored repo restore.

### 03. Managed Repositories List

Purpose: Browse and operate on approved repos.

Must include:

- Search/filter toolbar.
- Group by product/client.
- Compact rows default.
- Optional card/grid toggle.
- Row columns/chips: name, path, profile, branch, dirty status, checks, deployment, launch gate, active agent, activity.
- Quick row actions: Open, Details, View Status, View Diff, Run Checks, Start Agent, Launch Gate.
- Right inspector.

States:

- Safe repo.
- Dirty repo.
- Failed checks.
- Deployment unverified.
- Agent waiting for input.
- Launch gate blocked.

### 04. Repository Detail

Purpose: Answer “Is this repo safe, clean, and ready to work on or launch?”

Must include:

- Detail header: repo name, path, branch, mission status, primary actions.
- Sections/tabs: Overview, Git Status, Changed Files, Diff, Branches, Commits, PRs, Deployments, Launch Gate, Linked Items, History, Settings.
- Right inspector or detail side panel.
- Project notes/rules card.
- Danger zone panel for risky files/actions.
- Stale warning if path moved, branch behind, launch evidence old, deployment unverified.
- Compare current diff vs last verified launch.

Actions:

- Start Agent
- Run Checks
- View Diff
- Create Branch
- Commit
- Create PR Draft
- Deploy
- Verify Production
- Open Finder
- Open Editor/Terminal
- Handoff Repo State

States:

- Clean/safe.
- Dirty with risky files.
- Checks running.
- Checks failed.
- Launch gate blocked.
- Path missing.

### 05. Right Inspector

Purpose: Fast selected-repo preview/actions from dashboard/lists.

Tabs:

- Summary: repo health, branch, dirty state, last check, last deploy, launch verdict.
- Diff: changed files, diff summary, risky files.
- Checks: commands, latest output, pass/fail, duration.
- Agents: linked active/past agent sessions.
- Launch: current launch gate, checklist, evidence, verdict.
- Deployments: targets, last deploys, rollback notes.
- Linked Items: tasks, PRs, docs, reviews, handoffs.
- History: timeline of repo events.

States:

- No repo selected.
- Command output collapsed/expanded.
- Evidence missing.
- Agent waiting.
- Deployment unverified.

### 06. Repo Settings / Rules

Purpose: Configure repo profile, commands, permissions, and launch rules.

Must include:

- Display name.
- Product/client group.
- Profile/type selector.
- Local path and remote URL.
- Default branch.
- Command overrides: typecheck, lint, test, build, dev, deploy, verify.
- Deployment targets/environments.
- Launch gate checklist template.
- Production URLs.
- Allowed agent permissions.
- Sensitive file patterns.
- Preferred agent profile/model.
- Reviewer requirement.
- Evidence storage preference.
- Ignore/archive settings.
- Per-repo notes: product goal, deployment notes, test accounts, verification checklist.
- Import rules from AGENTS.md, CLAUDE.md, README, package scripts, .cursorrules with review/approval.

States:

- Auto-detected profile.
- Manual override.
- Sensitive files detected.
- Imported rules pending approval.

### 07. Run Checks Flow

Purpose: Execute real stored commands explicitly.

Must include:

- Command checklist: typecheck, lint, test, build, verify.
- Source of command: auto-detected/profile/manual/approved agent suggestion.
- Run selected / Run all.
- Live output panel collapsed by default.
- Pass/fail/duration/timestamp.
- Save as launch evidence toggle.
- Failed check action: Start Agent “Fix failed checks”.

States:

- No commands detected.
- Command running.
- Passed.
- Failed.
- Cancelled.
- Evidence saved.

### 08. Start Agent From Repo Modal

Purpose: Launch an Agents Workspace session prelinked to repo.

Must include:

- Agent profile selector.
- Model/provider selector.
- Repo/workdir already selected.
- Current diff/status attached.
- Repo profile/rules attached.
- Suggested prompts: Review this repo, Fix failed checks, Implement task, Prepare launch gate, Debug deployment.
- Permission preview scoped to repo.
- Layout choice: current Agents panel, new panel, background.
- Start button.
- Keep user in Code Workspace by default with linked run visible.
- Open in Agents Workspace/Open in panel actions after start.

States:

- No agent runner configured.
- Permission warning.
- Sensitive files excluded.
- Agent started linked run.

### 09. Launch Gate

Purpose: Evidence-based release/ship gate.

Must include:

- Gate header: state, repo, product, task, commit, deployment.
- Checklist: git state, typecheck, lint, tests, build, review, push, PR, deployment, production/native E2E, backend/API/database verification.
- Evidence panel.
- Final verdict panel.
- History timeline.
- Actions: Run Checks, Request Review, Deploy, Verify Production, Mark Verified, Mark Failed, Mark Blocked, Roll Back.
- Mark Verified disabled until required evidence exists.
- Manual override sheet with required reason and “override” label, not normal verified success.

States:

- Draft gate.
- Checks missing.
- Review required.
- Evidence missing.
- Verified.
- Failed.
- Blocked.
- Rolled back.

### 10. Evidence Attachment / Verification Record

Purpose: Add evidence to launch gate/check/deployment.

Must include:

- Evidence type selector.
- Required/supporting toggle.
- Source: command output, diff snapshot, commit SHA, reviewer verdict, PR link/status, deployment URL/provider status, E2E result, API/backend/database result, screenshot/video, logs, manual note.
- Attach file/link.
- Timestamp/source/repo/gate metadata.
- Storage explanation: small logs in Zoid app data, large files linked from repo/docs.

States:

- Required evidence missing.
- Attachment too large.
- Link unavailable.
- Evidence saved.

### 11. GitHub / PR Integration

Purpose: Remote repo/PR/CI state without blocking local management.

Must include:

- Remote URL display.
- Connect GitHub prompt if unauthenticated.
- Global GitHub setting indicator.
- Per-repo override/disable.
- Open PRs list.
- Issues/task links.
- CI/check status.
- Create PR draft flow with editable title/body and confirmation.

States:

- Unauthenticated.
- Authenticated.
- No remote.
- PR draft ready.
- CI failing.

### 12. Deployment Tracking / Actions

Purpose: Track and optionally trigger explicit deployments.

Must include:

- Deployment targets: Vercel, Hostinger VPS/SSH/Docker/Nginx, GitHub Pages, Cloudflare Pages/R2, custom/manual.
- Environment: local/staging/production.
- Production URL.
- Deploy command/provider.
- Last deploy status.
- Last verified status.
- Required E2E checklist.
- Rollback notes/command.
- Manual deployment record.
- Explicit deploy action with confirmation and evidence linkage.

States:

- No deployment target.
- Deployment unverified.
- Deploy ready.
- Deploy running.
- Deploy failed.
- Rollback warning.

### 13. Commit / Git Action Workflow

Purpose: Safe commit/PR/git operations.

Must include:

- Read-only git status/diff/history/branches.
- Stage selected files.
- Commit with generated/editable message.
- Create branch.
- Stash.
- Pull if clean.
- Strong confirmation for push, merge, rebase, reset, dirty checkout, discard, delete branch, force push, rollback-linked commit.
- Danger zone panel.
- Protected/main branch warning.

States:

- Clean.
- Dirty.
- Risky files selected.
- Protected branch.
- Strong confirmation.
- Action blocked.

### 14. Repo Handoff Export

Purpose: One-click export of repo state for agents/future sessions.

Must include:

- Summary preview: repo, branch, dirty state, changed files, checks, active agents, launch gate, deployment, stale warnings, notes.
- Include/exclude options.
- Export clean summary.
- Copy to clipboard.
- Attach to new Agents session.
- Save to repo/docs or app data.

States:

- Handoff generated.
- Missing evidence warnings.
- Copied.
- Attached to agent.

### 15. Search / History / Archive

Purpose: Find repos and past work.

Must include:

- Global search across repos by name/path/group/profile/status/file/PR/agent/evidence/history.
- Filters.
- Result rows with matched excerpt.
- Archive/ignored management.
- History timeline: repo events, checks, agents, launch gates, deployments, commits, PRs, evidence.

States:

- No results.
- Archived repo.
- Ignored repo.
- History empty.

### 16. Native Verification / Diagnostics

Purpose: Prove Code Workspace is connected to real native state.

Must include:

- Native/Tauri app indicator.
- Browser preview warning.
- Scan root status.
- Managed repo registry status.
- Disposable test repo selector/status.
- Git command availability.
- GitHub auth status.
- Deployment provider availability.
- App data/evidence paths.
- Verification checklist.

Actions:

- Create disposable test repo.
- Run discovery test.
- Run checks test.
- Run launch gate test.
- Run safe commit/PR draft test.
- Open app data.

States:

- Native app running.
- Browser preview only warning.
- Git missing.
- Test repo ready.
- Verification passed/failed.

## Cross-screen components

Design reusable components:

- Repo group header.
- Repo row.
- Repo card.
- Mission status chip.
- Attention state chip.
- Branch/dirty status badge.
- Checks status badge.
- Deployment status badge.
- Launch gate status badge.
- Agent linked-run card.
- Right inspector tabs.
- Diff summary row.
- Risky file indicator.
- Command output card.
- Evidence card.
- Launch checklist item.
- Deployment target card.
- Git action confirmation sheet.
- Danger zone panel.
- Repo settings form.
- Scan root card.
- Discovered repo row.
- Ignored repo row.
- Handoff export card.
- Needs-me queue item.
- Empty/loading/error/blocked/success states.

## Required state variants

Include component states for:

- Empty
- Loading/skeleton
- Safe
- Dirty
- Blocked
- Failed checks
- Running checks
- Agent waiting
- Review required
- Deployment unverified
- Launch evidence missing
- Verified
- Failed
- Rolled back
- Repo path missing
- Branch behind/ahead
- Sensitive file detected
- GitHub unauthenticated
- Deployment provider unavailable
- Strong confirmation required
- Native verification warning

## Accessibility and interaction requirements

- Minimum 44px action targets.
- Keyboard navigable repo list, filters, inspector tabs, modals, launch checklist, command output.
- Clear Action Blue focus ring.
- Status must not rely on color only; include labels/icons.
- Copyable command output/logs/evidence links.
- Dangerous actions explain impact and require confirmation.
- Mark Verified disabled state must explain missing evidence.
- Secrets/sensitive files warning must be explicit.

## Desktop and responsive behavior

Desktop first at 1440px macOS app width.

Responsive:

- 1024px: collapse inspector into drawer.
- 736–833px: list-first, repo detail as drill-in, inspector tabs full-width.
- 420–640px: single-column, filters as sheet, actions grouped under menus.

## Exact Stitch deliverables expected

1. High-fidelity desktop screen set for all 16 screens.
2. Component library page with all cross-screen components and state variants.
3. State board for empty/loading/safe/dirty/blocked/failed/running/agent-waiting/deployment-unverified/verified/path-missing/sensitive-file/native-warning.
4. Responsive variants for dashboard, discovery, repo detail, launch gate, right inspector, and Start Agent modal.
5. Interaction notes for discovery approval, repo settings, Run Checks, Start Agent, Launch Gate evidence, GitHub auth, deployment action, git action confirmation, handoff export, and native verification.
6. Visual token usage summary aligned with Zoid design system.
7. Handoff annotations naming each screen, primary/secondary actions, data shown, and workspace relationships.

## Sample copy/data

Use realistic sample data:

- Zoid / Zoid App
  - Profile: Zoid Tauri App
  - Branch: main
  - Status: Dirty
  - Checks: Failed
  - Agent: Agents Workspace implementation waiting for input

- Zoid / Disposable Test Repo
  - Profile: Custom
  - Branch: test/git-actions
  - Status: Safe
  - Purpose: safe git/deploy action verification

- MaVoid / MaVoid Site
  - Profile: Next.js/Vite frontend
  - Status: Ready to Launch
  - Deployment: unverified production URL

- Leadra / Leadra Backend
  - Profile: Node/Nest backend
  - Status: Blocked
  - Checks: failed tests

- Kalima / Kalima Portal
  - Profile: Full-stack web app
  - Status: Review required
  - Agent: Reviewer required fixes

Example launch gate missing evidence copy:

“Production verification is required before this launch can be marked Verified. Add browser/native E2E evidence and backend/API/database verification.”

Example danger zone copy:

“These actions can permanently change git history, deployment state, or local files. Zoid will never run them automatically.”

## Important reminders

- Do not make Code Workspace a full code editor.
- Do not hide repo ownership/grouping.
- Do not scan unapproved folders.
- Do not imply discovered repos are automatically managed.
- Do not treat build success as launch success.
- Do not make raw terminal output the main view.
- Do not make destructive actions one-click.
- Do not require GitHub auth for local repo management.
- Do not defer requested core functionality into Phase 2/3 framing.
- Do align with Zoid and Agents Workspace visual language.
