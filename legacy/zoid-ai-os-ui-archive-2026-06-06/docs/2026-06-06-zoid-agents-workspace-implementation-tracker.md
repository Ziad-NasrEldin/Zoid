# Zoid Agents Workspace Implementation Tracker

Date: 2026-06-06
Source discovery: `/Users/ziadnasreldin/brainstorms/2026-06-06-zoid-agents-workspace-implementation.md`
Related docs: `Docs/2026-06-06-zoid-agents-workspace-prd.md`, `Docs/2026-06-06-stitch-ai-agents-workspace-screens-prompt.md`, `Docs/designer-screen-reference/03-agents-workspace.md`

Purpose: execution tracker for the full Agents Workspace feature set. Status legend: `[ ]` pending, `[~]` in progress, `[x]` complete, `[!]` blocked.

Important: there is no Phase 2/3 deferral for the requested feature set. Use Build Order to implement safely, but all major functionality belongs in scope.

## Global Gates

- [ ] Real Hermes-backed sessions only for runtime claims; no fake agent/session records used as proof.
- [ ] Browser preview is not sufficient for done; final verification must run inside the Tauri/native app.
- [ ] Sessions persist transcript, metadata, repo link, context, status history, logs, review links, and layout state.
- [ ] Raw terminal/TUI output is available but not the default user experience.
- [ ] Consequential actions require approval gates: destructive files, git commit/push/reset/rebase/merge, installs, secrets/config, deployments, external messages, production data.
- [ ] Normal source edits are allowed only when session mode permits Coding/Autonomous work and repo permissions are granted.
- [ ] Feature critique workflow must run before calling implementation complete: create `.hermes/reviews/agents-workspace/handoff.md`, receive critique report, fix Required items, and re-review until approved.

## Likely Primary Files / Areas to Inspect

- Existing docs: `Docs/designer-screen-reference/03-agents-workspace.md`, `Docs/designer-screen-reference/02-code-workspace.md`, `Docs/designer-screen-reference/12-shared-tasks-calendar-history-reviews.md`
- Frontend shell/workspace routing: `src/App.tsx` and current workspace/view-model files
- Agents workspace components/view-model/tests: existing or new `src/agents*`
- Code Workspace repo model/components: existing or new `src/code*`
- Native/Tauri process runtime: `src-tauri/src/*`, PTY/CLI process modules, migrations
- Persistence: app SQLite migrations/services, app-support logs/artifacts
- Reviews: `.hermes/reviews/agents-workspace/*`

## Verification Command Set

Adjust commands to current repo scripts after inspection.

- [ ] Frontend focused tests: `npm run test:frontend -- agents` or project equivalent
- [ ] Backend/native focused tests: `cargo test --manifest-path src-tauri/Cargo.toml agents -- --nocapture` or project equivalent
- [ ] Full local verification: `npm run verify:local && git diff --check`
- [ ] Native manual: `npm run tauri:dev`, create real Hermes session, inspect app-support DB/logs, restart app, confirm persistence
- [ ] Release/native check if needed: Tauri `.app` + DMG verification path already used by Zoid

---

## Build Order 1 — Core Session / Runtime Foundation

- [ ] A1.01 Inspect current Zoid app structure, Agents Workspace placeholder, Code Workspace repo model, Tauri command patterns, SQLite migrations, and PTY/runtime spike docs.
- [ ] A1.02 Define `AgentSession`, `AgentProfile`, `AgentProcess`, `RepoLink`, `AgentEvent`, `AgentMessage`, `ReviewRecord`, `ContextAttachment`, and `LayoutState` domain models.
- [ ] A1.03 Add persistence for sessions, messages, events, profiles, process metadata, repo links, context attachments, layout state, pins/archive, and review links.
- [ ] A1.04 Implement Hermes runner service that starts a real Hermes-backed process/session with configured model/provider and working directory.
- [ ] A1.05 Implement process lifecycle states: queued, starting, live, idle, running, waiting_for_input, blocked, stopped, crashed, resume_available, archived.
- [ ] A1.06 Capture raw TUI/log output and normalize important events for the clean UI.
- [ ] A1.07 Autosave every user message, agent message, process event, file event, command event, repo event, and review event.
- [ ] A1.08 Implement Resume after stopped/crashed process using preserved transcript/context and Hermes-supported resume/handoff mechanics.
- [ ] A1.09 Add health warnings: crashed, stale repo, context near limit, permission blocked, process stopped, raw logs unavailable.
- [ ] A1.10 Add unit/integration tests for persistence reopen, process state transitions, raw log capture, and resume metadata.
- [ ] Gate: create a real Hermes session from native Zoid, send one message, receive output, stop/restart app, and confirm session metadata/transcript persist.

## Build Order 2 — Repo-Grouped Left Rail + Chat UI

- [ ] A2.01 Integrate Code Workspace repository list as the source for repo linking.
- [ ] A2.02 Implement repo link action on a session: set working directory, store repo id/path, show branch/status, attach current diff/task/plan/review metadata.
- [ ] A2.03 Implement left rail grouped primarily by repo/project.
- [ ] A2.04 Add session cards with avatar, title, repo badge, model badge, live state, needs-attention indicator, last message preview, and hover actions.
- [ ] A2.05 Add rail filters: Active, Blocked, Needs Me, Pinned, Reviewer, By repo, By profile, Recent.
- [ ] A2.06 Add search across transcript, repo, model, status, and file path.
- [ ] A2.07 Implement chat-first landing: open last active session/multi-panel layout, not dashboard-first analytics.
- [ ] A2.08 Implement iMessage-inspired user/agent bubbles aligned with Zoid design system.
- [ ] A2.09 Add inline alert cards for permission requests, blocked states, approvals, model switch, repo switch, and review verdicts.
- [ ] A2.10 Implement clean process strip: status, command/tool, elapsed, last event, token/context meter, workdir, model/provider, View Raw, Stop, Restart, Send Control Input.
- [ ] Gate: user can switch between repo-grouped sessions instantly without losing context; raw TUI is not the default view.

## Build Order 3 — Multi-Panel Layout

- [ ] A3.01 Add layout switcher for 1 panel, 2 side-by-side, 3 primary+stacked, and 4 grid.
- [ ] A3.02 Store layout state persistently per workspace/user.
- [ ] A3.03 Allow drag/drop or action-menu placement of a session from left rail into a panel.
- [ ] A3.04 Give every panel its own header, chat transcript, composer, process strip, model/profile/avatar/repo/status controls.
- [ ] A3.05 Closing a panel removes it from view only; it does not stop/archive the session.
- [ ] A3.06 Ensure panels show live preserved process state and continue output updates independently.
- [ ] A3.07 Add responsive/collapsed behavior for inspector/rail in 3–4 panel layouts.
- [ ] Gate: open 4 real/preserved Hermes sessions in one native viewing area and send messages independently.

## Build Order 4 — Right Activity Inspector

- [ ] A4.01 Add collapsible right inspector connected to the selected panel/session.
- [ ] A4.02 Activity tab: tools, commands, file reads/writes, status timeline, current process state.
- [ ] A4.03 Files tab: changed/read files, diffs, open buttons, reveal/open actions.
- [ ] A4.04 Repo tab: repo path, branch, git status, tests, PR/task links, workdir.
- [ ] A4.05 Context tab: attached docs, screenshots, files, plans, issue/task/context pack.
- [ ] A4.06 Review tab: reviewer verdict, required fixes, approvals, review history.
- [ ] A4.07 Raw tab: raw Hermes TUI/logs fallback with copy/export.
- [ ] A4.08 Mirror critical alerts from inspector into chat as compact inline cards.
- [ ] Gate: tool/file/command details do not clutter chat but are available in inspector.

## Build Order 5 — Model Switching / Handoff

- [ ] A5.01 Add visible model/provider selector in session/panel header.
- [ ] A5.02 Detect whether hot switch is supported by current runner/config.
- [ ] A5.03 If hot switch unsupported, implement same-session handoff: stop/suspend old process, start new 1:1 Hermes process with selected model/provider, inject preserved transcript/context, preserve old logs.
- [ ] A5.04 Add model switch event marker to transcript/status timeline.
- [ ] A5.05 Add optional fork-with-new-model action for comparisons.
- [ ] A5.06 Add tests for session continuity, process replacement, metadata preservation, and failure rollback.
- [ ] Gate: model switch keeps the same Zoid session visible and preserves transcript/context.

## Build Order 6 — Profiles, Avatars, Themes

- [ ] A6.01 Implement Agent Profiles: Hermes, Reviewer, Coding, Designer, Custom.
- [ ] A6.02 Profile fields: avatar image/icon, generated avatar option, display name, role label, default model/provider, command/runner, default repo/workdir behavior, permissions, accent/theme.
- [ ] A6.03 Session inherits profile avatar/theme by default.
- [ ] A6.04 Allow per-session override for AI avatar, user avatar, session background/art style, display name/nickname.
- [ ] A6.05 Add upload/select avatar support.
- [ ] A6.06 Add generated/themed avatar support where practical without blocking core functionality.
- [ ] Gate: changing profile changes default AI avatar/theme; manual session override persists.

## Build Order 7 — Composer Power Features

- [ ] A7.01 Implement slash command parser and menu: `/review`, `/run-tests`, `/link-repo`, `/switch-model`, `/handoff`, `/plan`.
- [ ] A7.02 Add attachments: files, screenshots, docs, pasted images, repo files/context.
- [ ] A7.03 Add target selector for multi-panel mode.
- [ ] A7.04 Add send to one session and broadcast to selected sessions.
- [ ] A7.05 Add prompt templates/presets.
- [ ] A7.06 Add approval response buttons when an agent waits for permission/input.
- [ ] A7.07 Add mode: send as instruction vs send as note/context.
- [ ] A7.08 Add quick handoff/compact action for long context.
- [ ] Gate: user can send targeted and broadcast prompts, attach context, and compact/handoff from the composer.

## Build Order 8 — Reviewer Workflow

- [ ] A8.01 Add Request Review action to coding sessions.
- [ ] A8.02 Launch reviewer as linked Hermes-backed reviewer session/run.
- [ ] A8.03 Store review verdict: Approved, Required fixes, Blocked.
- [ ] A8.04 Show required fixes in Review inspector tab and linked original session.
- [ ] A8.05 Allow sending required fixes back to original session.
- [ ] A8.06 Allow mark fixes complete and re-request review.
- [ ] A8.07 Show reviewer runs in left rail with type badge and nested/linked under original session.
- [ ] A8.08 Add `.hermes/reviews/<feature-slug>/handoff.md` creation guidance/action where appropriate.
- [ ] Gate: a real coding session can request review, receive Required fixes, apply fixes, and re-request until Approved.

## Build Order 9 — Search, Archive, Tags, Needs-Me, Export

- [ ] A9.01 Add global transcript search across all agent sessions.
- [ ] A9.02 Add pin/favorite mission-critical sessions.
- [ ] A9.03 Add archive without deleting transcript/context.
- [ ] A9.04 Add tags and repo/profile/status filters.
- [ ] A9.05 Add visible needs-me queue across all sessions.
- [ ] A9.06 Add quick copy/export for clean output, raw logs, and handoff summary.
- [ ] A9.07 Add health queue for crashed, stale repo, context near limit, permission blocked, approval needed.
- [ ] Gate: user can find any prior session by repo/project, transcript, file path, model, or status.

## Build Order 10 — Final Polish + Native E2E Verification

- [ ] A10.01 Align visual system with Zoid design: Apple-inspired, low chrome, SF/system typography, Action Blue, hairline borders, pill controls, no heavy SaaS chrome.
- [ ] A10.02 Ensure iMessage-inspired message readability and long-session comfort.
- [ ] A10.03 Add empty/loading/error/blocked/success states for sessions, rail, panels, inspector tabs, model handoff, repo link, reviewer, broadcast, raw logs.
- [ ] A10.04 Add keyboard navigation, accessible labels, status text not color-only, 44px targets, focus rings.
- [ ] A10.05 Run focused frontend and backend/native tests.
- [ ] A10.06 Run `npm run verify:local && git diff --check` or current full verification command.
- [ ] A10.07 Run inside Tauri/native app, not just browser preview.
- [ ] A10.08 Create real Hermes-backed session from Zoid.
- [ ] A10.09 Link real Code Workspace repo and verify branch/status/workdir.
- [ ] A10.10 Send/receive real messages and inspect clean chat/raw fallback.
- [ ] A10.11 Open 2–4 live sessions in multi-panel.
- [ ] A10.12 Test model handoff behavior.
- [ ] A10.13 Test right inspector Activity/Files/Repo/Context/Review/Raw.
- [ ] A10.14 Run reviewer workflow.
- [ ] A10.15 Restart Zoid and confirm sessions/transcripts/metadata/layout persist.
- [ ] A10.16 Confirm stopping/archiving does not destroy context.
- [ ] A10.17 Capture screenshots/video evidence for tracker.
- [ ] A10.18 Create `.hermes/reviews/agents-workspace/handoff.md`.
- [ ] A10.19 Run critique-agent review and fix Required items until critique report says approved.
- [ ] Gate: feature cannot be called done until native E2E evidence and critique approval exist.
