# PRD: Zoid Code Workspace — Repo Command Center

Date: 2026-06-06
Product area: Code Workspace, Agents Workspace, Launch Gates, GitHub, Deployments
Source discovery: `/Users/ziadnasreldin/brainstorms/2026-06-06-zoid-code-workspace-implementation.md`
Screen reference: `/Users/ziadnasreldin/Zoid/Docs/designer-screen-reference/02-code-workspace.md`

## 1. Overview

The Code Workspace is Zoid's repo command center. It discovers and manages the repositories Ziad cares about, shows repo health at a glance, links repositories to real Agents Workspace sessions, tracks dirty state/checks/branches/PRs/deployments, and enforces launch gates before any work is considered shipped.

It should reduce manual switching between Finder, terminal, GitHub, deployment providers, and Hermes sessions. The day-one job is not to become a full code editor or Finder clone. It is the operational surface for knowing which repos are safe, dirty, blocked, deployed, verified, or needing Ziad.

## 2. Primary User

Primary user: Ziad, founder/operator/product owner/developer managing multiple Zoid, MaVoid, Leadra, Kalima, and client/product repositories.

The workspace must respect Ziad's delivery standard: deploy/launch success is only valid after real production/native verification proves frontend/backend/database and all stated requirements. Build success alone is not enough.

## 3. Core Problems

Code Workspace solves:

1. Losing track of repo state across many projects.
2. Not knowing which repos are dirty, blocked, failed, deployed, or unverified.
3. Manually jumping between Finder, terminal, GitHub, deployment dashboards, and Hermes sessions.
4. Starting agents without clear repo/workdir/context.
5. Calling launches complete before evidence exists.
6. Forgetting which checks, commands, deployment targets, and repo rules apply per project.

## 4. Goals

- Discover local Git repos from user-approved scan roots.
- Let Ziad explicitly approve which repos become managed.
- Group repos primarily by product/client.
- Show repo mission status: Safe, Dirty, Blocked, Ready to Review, Ready to Launch, Verified.
- Show attention states: failed checks, blocked launch gates, unverified deployments, agents waiting for input, dirty/risky changes.
- Provide repo detail that answers: “Is this repo safe, clean, and ready to work on or launch?”
- Link repos to Agents Workspace sessions with repo/workdir/context preselected.
- Store repo profiles, rules, command overrides, permissions, deployment targets, launch gates, evidence, and history.
- Enforce evidence-based launch gates.
- Support safe GitHub/PR/commit workflows with confirmations.
- Support deployment tracking and explicit, confirmation-gated deployment actions.
- Provide native/Tauri verification using real repos and disposable test repos.

## 5. Non-Goals

- Do not scan the whole home folder automatically.
- Do not auto-manage every discovered Git repo without approval.
- Do not become a full embedded code editor/file browser.
- Do not show raw terminal output by default.
- Do not trigger deployments, pushes, resets, destructive git/file actions, or protected-branch writes automatically.
- Do not require GitHub auth for local repo management.
- Do not mark launch/deploy success without required evidence.
- Do not use Phase 2/3 deferral framing for requested scope; use Build Order.

## 6. Managed Repo Definition

A managed repo is a local Git repository Zoid knows Ziad cares about and has explicitly approved after scan/discovery.

Stored metadata:

- id
- display name
- product/client group
- profile/type
- local path
- remote URL
- default branch
- current branch
- dirty state
- ahead/behind remote state
- linked GitHub remote/PRs/issues
- deployment target(s)
- linked Agents sessions
- launch gate history
- verification commands
- project-specific notes/rules
- allowed agent permissions
- sensitive file patterns
- evidence storage preference
- ignore/archive state

## 7. Repo Discovery

Zoid scans only user-approved roots such as:

- `~/Zoid`
- `~/Projects`
- `~/Desktop`
- manually added client/project folders

Discovery detects:

- `.git` directories
- package/framework markers
- remotes
- recent activity
- dirty state
- likely product/client name
- duplicate repos
- nested/monorepo relationships

Discovery list shows:

- repo name
- path
- detected profile
- remote
- last modified
- risk/attention state
- duplicate/inside-monorepo warnings

Actions:

- Approve Selected
- Ignore Selected
- Add to group/client
- Set profile
- Open in Finder
- Rescan

Ignored repos appear in an Ignored tab and can be restored later.

## 8. Repo Profiles

Each managed repo gets a detected and manually overrideable profile:

- Zoid app / native Tauri app
- Next.js/Vite frontend
- Node/Nest backend
- full-stack web app
- static website
- SaaS/product app
- client project
- docs/content repo
- infra/devops repo
- unknown/custom

Profile defaults include:

- verification commands
- build command
- test command
- deployment target type
- launch gate checklist
- dangerous actions
- preferred agent profile
- production verification steps

Detection sources:

- `package.json`
- lockfiles
- `Cargo.toml`
- `docker-compose*`
- `Makefile`
- Tauri config
- framework markers
- `AGENTS.md`, `CLAUDE.md`, README, `.cursorrules` with approval before saving extracted rules

## 9. Main Workspace UX

The Code Workspace opens to a repo health command center.

Layout:

- Top header: Search Repos, Add Repo, Scan Folders, Refresh Status, Code Settings.
- Main repo list grouped by product/client.
- Small summary counters for managed repos, dirty repos, open launch gates, failed checks, and needs-me items.
- Attention queue for dirty repos, failed checks, blocked launch gates, unverified deployments, and agents waiting for input.
- Active code agent sessions linked from Agents Workspace.
- Right inspector for the selected repo.
- Quick actions: Open Repo, Start Agent, Run Checks, View Diff, Launch Gate.

Visual style:

- Hybrid Apple Finder-style project browser plus operational health dashboard.
- Clean native rows by default, with optional card/grid view.
- Grouped sections by product/client.
- Status chips for health/checks/deploy/launch gate.
- Right inspector feels like Finder/Apple Settings detail panel.
- Align with the existing Zoid design system and Agents Workspace.

## 10. Repo Attention States

A repo needs attention if:

- dirty/uncommitted changes
- failed checks
- launch gate blocked/failed
- deployment unverified
- active agent waiting for input
- reviewer required fixes
- branch behind/ahead remote
- PR failing or awaiting review
- secrets/config changed
- production verification stale
- process/deploy crashed
- repo path missing/moved

Highest-priority dashboard states:

1. Failed checks.
2. Launch gate blocked/failed.
3. Deployment unverified / production verification stale.
4. Active agent waiting for input.
5. Dirty/uncommitted changes, especially risky files or secrets/config changes.

## 11. Repository Detail

Repo detail answers: “Is this repo safe, clean, and ready to work on or launch?”

It shows:

- current branch and dirty status
- changed files and diff
- last checks/build/test result
- active/linked agent sessions
- open tasks/issues/PRs
- deployment target and latest deployment state
- launch gate state
- recent history/events
- project notes/rules
- danger zone panel for risky files/actions
- stale repo warnings
- quick compare between current diff and last verified launch
- quick actions: Start Agent, Run Checks, View Diff, Create PR, Deploy, Verify Production

It should not be a full file tree/code editor. It shows changed/relevant files and links out to editor/Finder when deeper browsing/editing is needed.

## 12. Right Inspector

When a repo is selected, the right inspector gives fast preview/actions. Clicking Open/Details opens the full repository detail page with the same sections expanded.

Tabs:

- Summary: repo health, branch, dirty state, last check, last deploy, launch gate verdict.
- Diff: changed files, diff summary, risky files.
- Checks: commands, latest output, pass/fail, duration.
- Agents: linked active/past agent sessions.
- Launch: current launch gate, checklist, evidence, verdict.
- Deployments: targets, last deploys, rollback notes.
- Linked Items: tasks, PRs, docs, reviews, handoffs.
- History: timeline of repo events.

## 13. Checks and Commands

Layered command discovery:

1. Auto-detect from repo files: `package.json`, `Cargo.toml`, docker-compose, Makefile, pnpm/yarn/npm lockfiles, Tauri config.
2. Repo profile defaults.
3. Manual repo settings override.
4. Agent-suggested commands saved only after Ziad approval.

Stored commands:

- typecheck
- lint
- test
- build
- dev server
- deploy if safe/approved
- production verification command/checklist

Refresh should gather cheap status only: git branch, dirty state, remote/deployment metadata. Full checks run when Ziad clicks Run Checks, Launch Gate requires them, or explicit repo policy enables auto-checks.

## 14. Agents Workspace Integration

Clicking Start Agent opens the New Agent Session modal with the repo prelinked.

Pre-filled context:

- repo/workdir selected
- current diff/status attached
- repo profile/rules attached
- suggested prompt based on action
- permission preview scoped to the repo
- layout choice: current Agents panel, new panel, background

Suggested actions:

- Review this repo
- Fix failed checks
- Implement task
- Prepare launch gate
- Debug deployment

After start:

- the session appears in Agents Workspace under that repo group
- the session appears in Code Workspace active agent runs
- Code Workspace keeps user in place by default and offers Open in Agents Workspace/Open in panel actions

## 15. Git Operations Safety

Safe anytime:

- show status
- show diff
- show history
- show branches

Allow with normal confirmation:

- create branch
- stage selected files
- commit with generated/editable message
- create PR draft
- pull latest if clean
- stash changes

Require strong confirmation:

- push
- merge
- rebase
- reset
- checkout with dirty state
- discard changes
- delete branch
- force push
- rollback deployment-linked commit

Never do automatically:

- force push
- reset hard
- delete untracked files
- overwrite user changes
- push to protected/main branch

Commit and PR draft creation are in scope but must be explicit, reviewable, and confirmation-gated.

## 16. GitHub / Remote Integration

Start with GitHub:

- detect remote URL
- show repo link
- show open PRs
- show issue/task links if available
- create PR draft from selected branch
- show CI/check status if GitHub auth exists
- do not require GitHub login for local repo management
- if unauthenticated, show graceful “connect GitHub for PR/CI” state

Auth/config:

- global GitHub connection in Zoid settings
- per-repo override/disable where needed

## 17. Deployment Tracking and Actions

Deployment providers/environments:

- generic deployment records
- Vercel
- Hostinger VPS / SSH / Docker / Nginx
- GitHub Pages
- Cloudflare Pages/R2 where relevant
- custom command/manual deployment

For each repo store:

- environment: local/staging/production
- production URL
- deploy command/provider
- last deploy status
- last verified status
- required E2E checklist
- rollback notes/command if known

Zoid can track deployments and support explicit deployment actions, but triggering deployments must be confirmation-gated and evidence-linked. No surprise deployments. Manual/agent-run deployments must be recordable.

## 18. Launch Gate

Launch Gate is strict and evidence-based.

Checklist areas:

- git state reviewed: branch, diff, uncommitted changes
- typecheck/lint/tests/build pass or blockers recorded
- code review/reviewer agent approval complete when required
- commit created and pushed when required
- PR created/merged if workflow uses PRs
- deployment executed or deployment blocker recorded
- production URL/app verified with real E2E
- database/backend/API verified where relevant
- screenshots/logs/evidence attached
- final verdict recorded

Verdicts:

- Verified
- Failed
- Blocked
- Rolled back

Zoid should prevent Mark Verified unless required evidence exists. A manual forced override can exist only with explicit reason and must be recorded as an override, not normal verified success.

## 19. Evidence Model

Acceptable evidence:

- command output: typecheck/lint/tests/build
- git diff snapshot and commit SHA
- reviewer verdict
- PR link/status
- deployment URL and provider status
- browser/native E2E result
- API/backend/database verification result
- screenshots/video of verified flows
- logs for failed/blocked cases
- manual notes as supporting evidence only

Storage:

- canonical metadata and small logs in Zoid app data
- attachments/links to repo/docs folders for screenshots, videos, reports, and handoff docs
- every evidence item records source path/URL, timestamp, repo, launch gate, required/supporting status, and verification owner/source

## 20. Permissions and Sensitive Files

Permissions are folder/repo-level:

- user manually adds scan roots
- Zoid explains what it reads: paths, git metadata, package files, scripts, remotes
- approving a repo grants read/status access
- running commands requires explicit action
- destructive git/file/deploy actions require confirmation
- secrets/env files are detected as sensitive and never opened/sent to agents by default
- per-repo permissions/settings are visible and editable

Zoid must only scan folders explicitly added by the user.

## 21. Files/Finder Relationship

Code Workspace is not a Finder clone, but integrates with local filesystem:

- Open repo in Finder
- Open repo in editor/terminal
- Reveal changed file
- Show changed files/diff
- Open docs/handoff/evidence files
- Respect macOS permissions
- Let a future Files module browse deeper if needed

For this scope, show changed/relevant files rather than a full repo file tree.

## 22. Repo Settings

Repo settings include:

- display name
- product/client group
- profile/type
- local path
- remote URL
- default branch
- command overrides: typecheck/lint/test/build/dev/deploy/verify
- deployment targets/environments
- launch gate checklist template
- production URLs
- allowed agent permissions
- sensitive file patterns
- preferred agent profile/model
- reviewer requirement
- evidence storage preference
- ignore/archive settings
- per-repo notes: product goal, deployment notes, test accounts, verification checklist

Repo settings support importing rules from `AGENTS.md`, `CLAUDE.md`, README, package scripts, and `.cursorrules`, with review/approval before saving extracted rules.

## 23. Handoff, Search, History, and Needs-Me

Required:

- one-click “handoff repo state” export for agents or future sessions
- global “what needs me?” queue across Code + Agents
- stale repo warnings for moved path, branch behind, old launch evidence, unverified deployment
- quick compare between current diff and last verified launch
- search across repos by name/path/group/profile/status/file/PR/agent/evidence/history
- archive/ignored repo management
- history timeline of repo events, checks, agents, launch gates, deployments, commits, PRs, and evidence

## 24. Build Order

No Phase 2/3 deferral. Use Build Order for safe execution:

1. Repo discovery + approved managed repo registry.
2. Repo profiles/settings/permissions.
3. Repo health dashboard + right inspector.
4. Repo detail + git status/diff/checks.
5. Agents Workspace linking.
6. Launch gate + evidence model.
7. GitHub/PR integration.
8. Deployment tracking/actions.
9. Commit/PR/git action workflows.
10. Search/archive/history/polish + native verification.

## 25. Acceptance Criteria

- User can add a scan root.
- Zoid discovers real local Git repos.
- User can approve selected repos into the managed registry.
- Ignored repos appear in an Ignored tab and can be restored.
- Zoid auto-detects repo profile and allows manual override.
- Repos are grouped primarily by product/client.
- Dashboard shows mission status, attention states, summary counters, active linked agents, and right inspector.
- Repo detail shows branch, dirty state, diff, checks, linked agents, PRs, deployments, launch gates, notes/rules, danger zone, stale warnings, and current-vs-last-verified comparison.
- Run Checks executes stored real commands and persists output/evidence.
- Start Agent opens Agents Workspace session modal with repo/workdir/context prelinked.
- Launch Gate blocks Mark Verified until required evidence exists.
- Evidence records command output, git/diff/commit data, review, PR/deploy/E2E/API/backend/database results, screenshots/video/logs/notes as appropriate.
- GitHub unauthenticated/authenticated states behave gracefully.
- Commit/PR draft flow works only with explicit confirmation and safe repo/test repo.
- Deployment tracking and confirmation-gated deployment actions work.
- Restarting Zoid preserves repo registry, settings, history, evidence, launch gates, and linked sessions.
- Native/Tauri verification passes; browser preview alone is insufficient.

## 26. Native Verification Gate

Done requires real native/Tauri verification:

- add a scan root
- discover real local Git repos
- approve selected repos into managed registry
- auto-detect repo profile and allow override
- show grouped repo health dashboard
- show right inspector tabs
- open repo detail
- show real git branch/dirty state/diff
- run real checks from stored commands
- start a real linked Agents Workspace session from a repo
- create/update a real launch gate
- attach evidence
- block Mark Verified until required evidence exists
- test GitHub unauthenticated/authenticated states if possible
- test commit/PR draft flow only with safe disposable test repo or explicit approval
- test deployment tracking/action with safe manual/custom target
- restart Zoid and confirm registry/settings/history/evidence persist
- use at least one real repo plus one disposable test repo for safe git/deploy action testing
- run feature critique workflow until approved before calling done
