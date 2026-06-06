# PRD: Zoid Agents Workspace — Hermes Session OS

Date: 2026-06-06
Product area: Agents Workspace, Code Workspace, Reviews
Source discovery: `/Users/ziadnasreldin/brainstorms/2026-06-06-zoid-agents-workspace-implementation.md`
Screen reference: `/Users/ziadnasreldin/Zoid/Docs/designer-screen-reference/03-agents-workspace.md`

## 1. Overview

The Agents Workspace is Zoid's visual operating system for Hermes Agent sessions. It replaces the pain of managing multiple raw TUI conversations with a persistent, repo-aware, multi-panel, chat-first workspace that lets Ziad create, monitor, talk to, resume, review, and compare real live Hermes-backed sessions.

This is not an analytics dashboard first. The default surface is a clean chat/session workspace inspired by Apple iMessage message clarity, aligned with the existing Zoid Apple-inspired design system, and backed by real 1:1 Hermes TUI/session processes. Raw terminal output is preserved, but translated into clean chat, status, activity, file, repo, and review views by default.

## 2. Core Product Intent

Agents Workspace must function as the command center for autonomous and conversational agent work in Zoid:

- Start and talk to real Hermes Agent sessions.
- Keep multiple sessions persistent, visible, grouped, and instantly continuable.
- Link sessions to Code Workspace repositories so workdir, branch, git state, tasks, plans, diffs, and reviews are obvious.
- Show clean chat instead of ugly raw terminal output.
- Support up to 4 live sessions in one viewing area.
- Support quick model switching through seamless model handoff inside the same Zoid session.
- Support reviewer workflows that match Ziad's delivery rule.
- Preserve transcript, metadata, repo context, status, logs, reviews, and handoff artifacts.

## 3. Primary User

Primary user: Ziad, founder/operator/product owner/developer using Hermes to manage multiple real workstreams across Zoid, MaVoid, Leadra, Kalima, and client/product repos.

Primary pain points to solve:

1. Losing track of multiple sessions.
2. Not seeing which repo/project a session belongs to.
3. Ugly/raw terminal output when clean decisions, output, and state are needed.

Secondary pain points:

- Not knowing which agent needs input.
- Switching models without breaking continuity.
- Comparing 2–4 agents side by side.
- Seeing tools/files/commands used without reading raw logs.
- Turning completed work into tasks, PRs, reviews, or follow-ups.
- Preventing context from rotting through persistence, health warnings, and handoff/compact actions.

## 4. Goals

- Make Hermes sessions visually pleasing, persistent, and operationally manageable.
- Keep sessions grouped primarily by repo/project.
- Preserve real live process identity: Zoid session maps 1:1 to a Hermes session/process while Hermes is the runner.
- Allow fast model/provider switching with same-session continuity.
- Provide a clean chat UI with custom avatars for the user and AI/agent.
- Provide multi-panel support for up to 4 live sessions.
- Provide a right activity inspector for operational details.
- Provide a repo-aware left rail and global needs-me queue.
- Provide reviewer workflow and review history for coding sessions.
- Provide native/Tauri verification with real Hermes sessions, not fake records.

## 5. Non-Goals

- Do not build a fake/demo chat system that pretends to run agents.
- Do not make browser preview verification sufficient for done.
- Do not default to raw terminal/TUI as the main UX.
- Do not make analytics dashboard cards the main Agents landing screen.
- Do not support Codex CLI, Claude Code, OpenHands/OpenClaw, generic terminal bots, or remote/cloud agents as day-one runners. Keep structure future-ready, but MVP implementation is Hermes-first.
- Do not silently destroy transcript/context when processes stop, restart, switch model, or archive.

## 6. Key Decisions

- The Agents tab opens to the last active session or last active multi-panel layout.
- The left rail groups sessions primarily by repo/project.
- Sessions are persistent Hermes conversations with metadata, transcript, process state, linked repo, context, logs, and health.
- Zoid session maps 1:1 to a live Hermes session/process for Hermes-backed sessions.
- Clicking a session in the left rail instantly continues it; it should not reload in a way that risks context loss.
- Closing a panel only removes it from the current layout; it does not stop/archive the live session.
- Model switching defaults to same-session handoff, not fork.
- The chat remains clean; operational details live in the right activity inspector.
- Critical blocked/permission/approval events appear both in the inspector and as small inline alert cards.
- All requested capability belongs in the implementation tracker. Use Build Order, not Phase 2/3 deferral.

## 7. Core Entities

### AgentSession

A persistent conversation/work unit.

Fields/metadata:

- id
- title
- type: chat, background_run, reviewer_run, watcher_run
- agent_profile_id
- runner: Hermes for initial implementation
- live_process_id / pty/session handle
- model/provider
- linked_repo_id
- working_directory
- transcript messages
- status history/events
- health state
- context attachments
- layout/panel state
- pinned/archive state
- created_at/updated_at/last_active_at

### AgentProfile

Defines default behavior and identity.

- avatar image/icon
- generated/themed avatar option
- display name
- role label, e.g. Hermes Agent, Reviewer, Coding Agent, Designer
- default model/provider
- command/runner
- default repo/workdir behavior
- permissions
- visual accent/theme

### AgentProcess

The real process/session backing the Zoid session.

- process id/session handle
- runner command
- model/provider args
- workdir
- live/idle/running/stopped/crashed state
- current tool/command
- elapsed time
- raw logs/TUI buffer
- resume capability

### RepoLink

Connects a session to a Code Workspace repo.

- repo id
- repo path
- branch
- git status summary
- current diff summary
- task/plan/review links
- selected files/context
- last repo snapshot/hash for stale warnings

### ReviewRecord

- linked original session
- reviewer session/run
- verdict: Approved, Required fixes, Blocked
- required fixes
- evidence/attachments
- review history

## 8. Main Workspace Layout

Default Agents Workspace should be chat-first:

- Left rail: repo/project grouped sessions.
- Center: live chat panel or multi-panel grid.
- Top header: New Session, layout switcher, model/profile controls, search, needs-me indicator.
- Right inspector: Activity, Files, Repo, Context, Review, Raw tabs.
- Small summary counters only in header/rail, not large dashboard-first cards.

### Left Rail

Session cards show:

- custom agent/avatar icon
- session title
- repo/project group
- model badge
- status: active, waiting for input, running tool, blocked, crashed, idle, completed, approval needed, context warning, permission needed, review required
- unread/needs-attention indicator
- last message preview
- hover quick actions: open in panel, pin, rename, stop, duplicate/fork, archive

Filters/search:

- Active
- Blocked
- By repo
- By agent profile
- Pinned
- Recent
- Needs me
- Reviewer runs
- Search transcript/repo/model/status/file path

### Multi-Panel Layout

Support up to 4 live sessions in one viewing area:

- 1 panel: focused chat
- 2 panels: side-by-side
- 3 panels: one large primary + two stacked secondary
- 4 panels: 2x2 grid

Each panel has:

- session header
- avatar/profile/model/repo/status
- independent chat
- independent composer
- clean process strip
- panel close/remove from view

Drag/drop a session from the left rail into a panel. Closing a panel does not stop the session.

## 9. Chat Interface Requirements

Visual direction:

- Existing Zoid Apple-inspired design system.
- Apple iMessage-inspired message clarity.
- Clean practical work console, not heavy cyber dashboard.
- User and agent messages as polished readable bubbles/cards.
- Avatars visible and customizable.
- Operational events mostly kept out of the main chat and placed in the right inspector.
- Critical alerts appear inline.

Message types:

- user message
- agent reply
- permission request alert
- blocked state alert
- model switch marker
- repo link/change marker
- review result alert/card
- system note

Tool/file/command details should be shown in the right inspector, not mixed densely into chat.

## 10. Right Activity Inspector

Tabs:

1. Activity
   - live tools
   - commands
   - file reads/writes
   - status timeline
   - current process state

2. Files
   - changed files
   - read files
   - diffs
   - open buttons

3. Repo
   - linked repo
   - branch
   - git status
   - tests
   - PR/task links
   - workdir

4. Context
   - attached docs
   - screenshots
   - files
   - plans
   - issue/task/context pack

5. Review
   - reviewer verdict
   - required fixes
   - approvals
   - review history

6. Raw
   - raw Hermes TUI/logs fallback
   - expandable, not default

Inspector is visible in roomy layouts and collapsible/responsive in multi-panel or smaller spaces.

## 11. Process State Requirements

Do not show raw terminal UI by default.

Show a clean process strip/collapsible panel with:

- live/idle/running/stopped/crashed status
- current command/tool name
- elapsed time
- last important event
- token/context meter
- working directory
- model/provider
- buttons: View Raw TUI, View Logs, Stop, Restart, Send Control Input

Raw TUI/logs remain available in the Raw inspector tab.

## 12. New Agent Session Flow

Fast modal fields:

- agent profile: Hermes / Reviewer / Coding / Designer / Custom
- model/provider selector
- linked repo selector from Code Workspace
- session title
- prompt / initial instruction
- context attachments: files, docs, plan, screenshot, issue, task
- permissions preview
- review requirement toggle: none / reviewer required / approval required before changes
- layout choice: open in current panel / new panel / background

One-click presets:

- Start coding session
- Review this repo
- Debug issue
- Design screen
- Continue previous work
- Create PR

Default: preselect Hermes Agent with current/default profile and model, while allowing quick changes.

## 13. Composer Requirements

Composer supports:

- normal text prompt
- slash commands: `/review`, `/run-tests`, `/link-repo`, `/switch-model`, `/handoff`, `/plan`
- attach files/screenshots/docs
- paste image
- drop repo file/context
- select target if in multi-panel mode
- send to one session
- broadcast to selected sessions
- prompt templates/presets
- approval response buttons when agent is waiting
- send as instruction vs send as note/context

## 14. Model Switching / Handoff

Default behavior: same Zoid session continuity.

If hot switching is not supported:

1. Stop/suspend old process.
2. Start new 1:1 process with selected model/profile.
3. Inject preserved transcript/context using the best supported Hermes resume/handoff mechanism.
4. Show timeline marker: “Switched from GPT-5.5 to Claude” or equivalent.
5. Preserve old raw logs and process history.

Optional action: fork with new model for comparison.

## 15. Repo Linking / Code Workspace Integration

Linking a repo should:

- set the session working directory
- show repo name/branch/status in chat header
- allow tools/commands inside that repo
- attach selected files, task tracker, plans, reviews, and current git diff
- expose quick actions: open repo, open terminal, view git diff, create task, create PR, run tests
- warn before destructive git/file operations
- require explicit confirmation before switching repo

Default: one primary repo per session. Multi-repo can be handled later as one primary repo plus attachments/context.

## 16. Persistence / Liveness

Separate conversation persistence from process liveness:

- transcript, metadata, linked repo, context, and status history always persist
- live process is kept alive while active if possible
- if process dies/restarts, session remains visible as “process stopped, resume available”
- one-click Resume starts a new 1:1 process with preserved context
- autosave every message/event
- stale warning only if repo/context changed since last active
- never silently discard context
- pinned/active sessions stay alive until stopped/archived when possible

## 17. Permissions and Safety

Require approval for:

- deleting files/folders
- overwriting large files
- git commit/push/merge/rebase/reset
- installing packages
- changing environment/secrets/config files
- running destructive shell commands
- deployments
- sending external messages/posts/emails
- modifying production data

Allow without approval when session mode permits:

- reading files in linked repo
- searching code
- running safe tests/lints/builds
- editing normal source files in Coding/Autonomous mode
- creating drafts/plans/reviews

Modes:

- Ask-first
- Coding
- Review-only
- Autonomous with approval gates

## 18. Reviewer Workflow

Include lightweight reviewer workflow:

- any coding session can request review
- reviewer agent opens as linked session/run
- reviewer produces Approved / Required fixes / Blocked verdict
- required fixes appear in right inspector
- required fixes can be sent back to original session
- original session can mark fixes complete and re-request review
- review history is stored per session/repo/task
- reviewer runs appear in left rail with type badge and are linked/nested under original session in inspector

## 19. Handoff / Needs-Me / Health / Search / Export

Required:

- quick handoff/compact session action when context gets long
- visible needs-me queue across all sessions
- health warnings: crashed, stale repo, context near limit, permission blocked
- pin mission-critical sessions
- global search across agent transcripts
- quick copy/export clean output, raw logs, and handoff summary

## 20. Build Order

Everything above is in scope. Use Build Order, not deferred phases:

1. Core session/runtime foundation
2. Repo-linked left rail + chat UI
3. Multi-panel layout
4. Right activity inspector
5. Model switching/handoff
6. Avatars/themes
7. Composer power features
8. Reviewer workflow
9. Search/archive/tags/broadcast
10. Final polish + native E2E verification

## 21. Acceptance Criteria

- User can create a real Hermes-backed session from Zoid.
- User can link a session to a real Code Workspace repo.
- Session header shows repo/project, branch/status, model/provider, agent profile, and health.
- User can send/receive real messages through Hermes.
- Chat transcript is clean, readable, iMessage-inspired, and persisted.
- Raw Hermes/TUI output is available but hidden by default.
- User can switch between multiple preserved sessions from a repo-grouped left rail.
- User can open 2, 3, or 4 live sessions in multi-panel layouts.
- Each panel has its own composer and process state.
- Right inspector shows Activity, Files, Repo, Context, Review, and Raw tabs.
- Model handoff preserves Zoid session continuity.
- Agent/user avatars inherit from profile and can be manually overridden per session.
- Composer supports slash commands, attachments, paste image, repo context, target selection, broadcast, templates, approval responses, and instruction vs context modes.
- Reviewer workflow produces stored verdicts and required fixes.
- Needs-me queue, health warnings, pins, search, archive, and exports work.
- Restarting Zoid preserves sessions, transcripts, metadata, repo links, status history, and layout.
- Stopping/archiving does not destroy saved context.
- Native/Tauri verification passes; browser preview alone is insufficient.

## 22. Native Verification Gate

Done requires real native/local verification:

- run inside Tauri/native Zoid app
- create real Hermes-backed session
- link real Code Workspace repo
- send/receive real messages
- show clean chat transcript
- show process state and raw fallback
- switch between preserved sessions
- open 2–4 live sessions in multi-panel
- verify repo grouping and repo badge/status
- test model handoff behavior
- test right inspector tabs
- run reviewer workflow
- restart Zoid and confirm persistence
- confirm stop/archive keeps saved context
- collect screenshots/video evidence
- run project verification commands and feature critique workflow until approved
