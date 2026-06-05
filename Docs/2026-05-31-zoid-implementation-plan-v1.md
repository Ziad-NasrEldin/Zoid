# Zoid Implementation Plan v1

> For Hermes: Use `subagent-driven-development` before executing coding tasks. This plan is the engineering guardrail for implementing the Zoid PRD. Do not treat the PRD as a direct ticket list.

Date: 2026-05-31
Owner: Ziad Salah
Status: Implementation planning draft v1
Source PRD: `/Users/ziadnasreldin/Zoid/Docs/2026-05-31-zoid-product-requirements-and-architecture.md`

Internal/private: This document contains internal product, workflow, business, and operating-system design details. Do not use its internal implementation details, infrastructure choices, private business workflows, or third-party service details in public UI/copy unless explicitly approved.

---

## 1. Goal

Build Zoid as a polished macOS-first, local-first desktop AI workspace OS: a native-feeling Apple-style desktop app for Today, Tasks, Notes, Agents, Code, Content, Automations, Business, Products, Files, Browser, Inbox, Calendar, and History.

The first implementation objective is not to build every workspace shallowly. The first objective is to prove the core operating loop:

Start a task from Today -> run a CLI/session -> stream output into a clean conversational UI -> record AgentRun, ReviewRecord, Task, Notification, and Event history -> surface completion/blockers back in Today/Inbox.

---

## 2. Non-Negotiable Engineering Principles

1. Local-first by default.
2. Apple-style native-feeling UI; no generic SaaS dashboard, cyberpunk, or raw terminal-first interface.
3. Security, Keychain, redaction, permissions, and safe logging are foundation work, not late polish.
4. Visible user files stay user-owned under `~/Zoid/`.
5. App state, indexes, logs, and config live under `~/Library/Application Support/Zoid/`.
6. Secrets never go into prompts, logs, events, Markdown files, or visible folders.
7. CLIs are the initial AI execution layer; Zoid does not directly call LLM APIs in the first implementation path.
8. All consequential actions require an explicit action policy decision before execution.
9. Every meaningful action creates an Event.
10. No fake integration success states. If GitHub, Vercel, Gmail, OmniSocials, EventKit, Browser, or CLI execution cannot be verified, the UI must show blocked/unconfigured/failure truthfully.
11. Do not ship a feature as complete until it has real verification output.
12. Feature implementation must pass the `feature-critique-workflow` gate before being considered done.

---

## 3. Architecture Decision: Preferred Stack, Pending Spikes

Preferred stack:

- Desktop shell: Tauri + React + TypeScript
- Native layer: Rust/Tauri commands
- Optional helper layer: Node helper only if required for CLI/PTY compatibility
- Database: SQLite with migrations
- Search/indexing: SQLite FTS where appropriate
- UI: custom Apple-style React component system
- Secrets: macOS Keychain
- Notifications: native macOS notifications + in-app Inbox
- Calendar: EventKit bridge
- AI/agent execution: configured local CLI profiles

Important: Tauri remains the preferred choice only if the technical spikes below pass. Do not deeply implement the app shell before validating PTY/session, Browser/WebView, and native macOS integration feasibility.

Fallback decision point:

If embedded PTY, browser capture, OAuth/native integration, or process supervision are fundamentally weak in Tauri, stop and evaluate Electron or a native Swift helper before continuing.

---

## 4. Required Technical Spikes Before Main Build

### Spike A: PTY / CLI Runtime Feasibility

Goal: prove Zoid can run real local CLI/session workflows reliably.

Must prove:

- Start a shell command in a chosen working directory.
- Start an interactive PTY command.
- Stream stdout/stderr incrementally.
- Send stdin to a running process.
- Cancel/kill a process and its child process tree.
- Persist raw logs to app support.
- Render clean output cards from streamed logs.
- Reopen app/session view and show previous session history.
- Detect exit code, duration, and failure state.
- Enforce max log size or rotation.
- Redact obvious secrets from logs.

Acceptance:

- A test command and one interactive command both run from the prototype UI.
- Output appears live.
- Cancellation works.
- Logs are persisted outside SQLite as files.
- SQLite stores only metadata and log references.
- No secrets are visible in stored logs/events during redaction tests.

### Spike B: Browser / WebView Feasibility

Goal: prove the Browser workspace can be a work webview/capture workspace, not an over-promised full browser.

Must prove:

- Open multiple work tabs or a credible first tab abstraction.
- Persist tab URL/title/history metadata.
- Capture screenshot if feasible.
- Save page URL/title/screenshot reference to an entity.
- Test login-heavy websites enough to understand cookie/session behavior.
- Determine whether console/error capture is feasible in Tauri WebView.
- Determine whether app verification evidence can include screenshot + URL + HTTP status even if console capture is unavailable.

Acceptance:

- Browser spike document states supported/unsupported capabilities.
- If console capture is not robust, Launch Gate browser verification must not depend on it.
- Browser workspace wording remains “work webview/capture workspace” until capabilities are proven.

### Spike C: Native macOS Services Feasibility

Goal: prove key native integrations before building dependent features.

Must prove:

- Keychain write/read/delete for a test credential.
- Native notification with click/open route if feasible.
- EventKit permission prompt and read upcoming events.
- EventKit create event after explicit confirmation.
- Reveal file in Finder/open file behavior.
- App support and visible folder creation.

Acceptance:

- All native integration results are documented.
- Permissions failure states are handled cleanly.
- Keychain abstraction is viable before any OAuth/API credential storage is implemented.

---

## 5. Foundation Security Must Move Earlier

The PRD placed several security items in late polish. That is not safe. Move these to foundation before real integrations:

- Keychain service abstraction.
- Secret redaction utility.
- Permission/action policy engine.
- Confirmation framework.
- Safe logging rules.
- Event/audit writer.
- Log file storage and rotation.
- Credential reference model.
- “Unconfigured / needs auth / blocked” integration states.

Security acceptance:

- No raw token/API key can be stored in SQLite, Markdown, visible files, prompts, events, or logs.
- Logs are redacted before persistence and before rendering.
- CLI environment variables are passed by reference from secure config/Keychain where possible, not copied into visible text.
- Credential changes require confirmation.
- External send/publish/deploy actions require human confirmation and applicable review policy.

---

## 6. Action Policy Matrix

Replace vague “review required when changes files” language with a concrete action policy matrix.

| Action Category | Default Policy | Reviewer Required | Human Confirmation | Notes |
|---|---:|---:|---:|---|
| Read/search local app data | Allow | No | No | Events optional for high-value reads only |
| Read/search Gmail/Calendar after auth | Allow | No | No | Must respect integration permissions |
| Create simple local task | Allow | No | No | Event required |
| Create/edit private Markdown note | Allow inside user/session context | No by default | No by default | It is a file but not automatically consequential |
| Import/migrate notes/data | Ask before action | Usually yes | Yes | Must preview count/source/destination |
| Modify visible non-code file | Ask if outside current task | Maybe | Maybe | Depends on scope/destructiveness |
| Move/rename/copy one file | Ask before action | No by default | Yes for move/rename/delete | Use Trash for delete |
| Bulk file operations | Block until confirmed | Maybe/Yes | Yes | Preview exact affected paths |
| Delete/trash files | Ask before action | Maybe | Yes | Prefer Trash over permanent delete |
| Modify code/repo files | Require clear task/session | Yes for consequential work | No unless destructive | Must show diff |
| Commit/push/merge | Ask before action | Yes | Yes | Must show branch/diff/remote |
| Deploy/redeploy/rollback | Ask before action | Yes | Yes | Launch Gate evidence required |
| Send email | Ask before action | Policy-dependent | Always | Show recipients/body/attachments |
| Publish/schedule content | Ask before action | Yes | Policy-dependent; yes for sensitive/one-off | MaVoid recurring may allow autonomous scheduling after internal approval |
| Change automation schedule/enabled state | Ask before action | Maybe | Yes | Show before/after |
| Run existing automation manually | Ask if consequential | Maybe | Maybe | Depends on automation action type |
| Change credentials/settings/integrations | Ask before action | Maybe | Always | Never show raw secret |
| Create calendar event | Ask before action | No by default | Yes | Show title/time/calendar |
| Edit/delete calendar event | Ask before action | Maybe | Always | Show before/after |

Action-policy implementation requirements:

- Every action passes through a centralized policy evaluator.
- UI must show why confirmation/review is required.
- Consequential action cards must include preview/diff/evidence.
- No global dry-run engine is required, but every consequential action needs a local preview/diff before confirmation.

---

## 7. SQLite and Data Model Foundation

The PRD entity list is not enough to build from. Implement a concrete schema before broad workspace work.

### Core schema concepts

Use stable IDs for all durable entities. Prefer ULID or UUIDv7-style sortable IDs.

Required base tables:

- `schema_migrations`
- `app_settings`
- `workspaces`
- `tasks`
- `notes`
- `files`
- `entity_links`
- `events`
- `event_targets`
- `sessions`
- `agent_profiles`
- `agent_runs`
- `review_records`
- `notifications`
- `automation_definitions`
- `automation_runs`
- `repositories`
- `products`
- `content_plans`
- `content_pieces`
- `media_assets`
- `publish_records`
- `contacts`
- `companies`
- `calendar_item_refs`
- `email_message_refs`
- `browser_tabs`
- `browser_captures`
- `launch_gates`
- `credential_refs`

### Entity link model

Do not use an undefined JSON blob as the only relationship model.

Implement `entity_links`:

- `id`
- `source_type`
- `source_id`
- `target_type`
- `target_id`
- `relation_type`
- `created_at`
- `created_by_actor_type`
- `metadata_json`

Examples:

- task -> note: `references`
- task -> agent_run: `executed_by`
- repo -> product: `belongs_to`
- content_piece -> media_asset: `uses_asset`
- browser_capture -> launch_gate: `evidence_for`
- email_message_ref -> contact: `from_contact`

### Event model

Implement `events`:

- `id`
- `type`
- `timestamp`
- `actor_type`
- `actor_id`
- `workspace_key`
- `summary`
- `severity`
- `source`
- `metadata_json`

Implement `event_targets`:

- `event_id`
- `entity_type`
- `entity_id`
- `relation_type`

Rules:

- SQLite stores event summaries and metadata, not massive raw logs.
- Raw logs live as files in app support and are referenced by path/id.
- Events must never contain raw secrets.
- Important UI state should be reconstructable enough from current tables + events.

### Migration rules

- Every schema change uses a migration file.
- App startup runs pending migrations.
- Migration failure blocks startup into a safe error screen, not silent data corruption.
- Backups are not a major product focus yet, but destructive migrations must create a local DB copy first.

---

## 8. Markdown Notes Architecture Rules

Notes are user-owned Markdown files plus SQLite metadata/indexes.

### Source of truth

- Markdown file body is source of truth for user-authored note content.
- Frontmatter stores stable visible metadata.
- SQLite stores index/search/status/summaries/entity links/history/app state.

Recommended frontmatter:

```yaml
---
zoid_id: note_...
title: "Note title"
created_at: "2026-05-31T00:00:00Z"
updated_at: "2026-05-31T00:00:00Z"
tags: []
workspace: notes
---
```

Rules:

- File rename/move outside Zoid must be detected by scanner where possible.
- If a file has a known `zoid_id`, preserve identity after rename/move.
- If two files share the same `zoid_id`, mark conflict and do not auto-delete either.
- If SQLite metadata conflicts with frontmatter, preserve file content and surface a metadata conflict.
- Deleted notes use tombstones in SQLite for future sync readiness.
- Imports from Apple Notes go under `~/Zoid/Notes/Imported/Apple Notes/` unless user chooses otherwise.
- Import collisions must create unique filenames and preserve original source reference.
- Index rebuild must be possible from Markdown files.

Acceptance:

- Create/edit note writes Markdown.
- App restart reloads note from disk.
- Manual rename still preserves note identity if frontmatter exists.
- Duplicate ID conflict is visible and non-destructive.

---

## 9. CLI Runtime Contract

Zoid runs local CLIs through configured profiles. This needs a strict runtime contract.

### Agent profile fields

- `id`
- `name`
- `command`
- `args_template`
- `default_workdir`
- `workdir_policy`
- `env_refs`
- `uses_pty`
- `supports_stdin`
- `supports_cancel`
- `parser_mode`
- `default_permissions`
- `is_reviewer_profile`
- `created_at`
- `updated_at`

### AgentRun lifecycle

Statuses:

- `queued`
- `starting`
- `running`
- `waiting_for_input`
- `completed`
- `failed`
- `cancelled`
- `blocked`
- `review_required`

Fields:

- `id`
- `profile_id`
- `task_id`
- `workspace_key`
- `repo_id`
- `workdir`
- `prompt_ref_or_inline_prompt`
- `status`
- `started_at`
- `completed_at`
- `duration_ms`
- `exit_code`
- `logs_path`
- `output_summary`
- `error_summary`
- `review_status`
- `metadata_json`

### Runtime requirements

- Support PTY and non-PTY modes.
- Stream stdout/stderr incrementally.
- Support stdin where the profile allows it.
- Cleanup process tree on cancel.
- Store raw redacted logs in app support.
- Store summaries/metadata in SQLite.
- Do not store secrets in prompts/logs/events.
- Provide max runtime policy and timeout warning.
- Provide max log size and rotation/truncation policy.
- App restart should show completed/failed/cancelled historical runs; live process reattachment is optional unless proven feasible.
- UI must distinguish “failed”, “blocked”, “needs input”, and “completed”.

### Clean Session UI requirements

Default Clean Mode:

- user prompt bubble
- command card
- progress/status card
- streamed output cards
- collapsible raw log card
- summary card
- error/blocker card with suggested next action

Raw Mode:

- true PTY terminal when supported
- explicit mode switch
- raw mode must still respect logging/redaction policies

---

## 10. Browser Workspace Scope Correction

Until Spike B proves full browser capabilities, implement this as a work webview/capture workspace, not a personal or full developer browser.

Allowed initial scope:

- open work URL
- tabs if feasible
- persist URL/title
- save page link to entity
- screenshot capture if feasible
- create BrowserCapture records
- attach browser evidence to LaunchGate/Task/Note/Product/ContentPiece

Do not promise initially:

- extension ecosystem
- password manager
- personal browser sync
- full consumer browser history
- robust DevTools-level console/network capture unless spike proves it

Launch Gate browser verification fallback:

If console capture is weak, verification evidence can still include:

- production URL
- HTTP status from a backend check
- screenshot
- route smoke result
- asset-load check from external command where feasible
- manual observation note

---

## 11. First Vertical Slice: Implement This Before Broad Workspace Expansion

Name: Today -> Task -> CLI Session -> AgentRun -> ReviewRecord -> Notification -> History

Goal:

Prove the heart of Zoid before building every workspace.

Included:

- macOS app shell
- sidebar with all workspace names
- Today workspace with real widgets backed by SQLite
- task create/list/detail
- local SQLite migrations
- event writer
- CLI profile config for at least one local command
- command/session runner
- Clean Session UI
- raw log capture
- AgentRun records
- ReviewRecord model and manual reviewer stub first
- notification records
- in-app Inbox/attention card basics
- native notification if Spike C passed
- permission/action policy basics
- redaction and Keychain foundation

Excluded from first slice:

- Gmail
- OmniSocials
- GitHub/Vercel full integration
- Apple Notes import
- full file manager
- full browser workspace
- full calendar integration beyond spike result
- autonomous content publishing
- mobile/private sync

Acceptance:

1. App launches locally on macOS.
2. `~/Zoid/` folder exists with expected starter directories.
3. app support directory exists with SQLite DB and logs directory.
4. SQLite migrations run successfully.
5. Today renders widgets from real data, not hardcoded fake completion states.
6. User can create a task.
7. User can start a configured CLI run attached to that task.
8. Output streams into Clean Session UI.
9. Raw redacted log is persisted to app support.
10. AgentRun record stores status, duration, exit code, log reference, and summary.
11. Events are written for task creation, run start, run completion/failure, review record creation, and notification.
12. ReviewRecord can be created manually or by a simple configured reviewer profile if available.
13. Today/Inbox shows completion/blocker notification.
14. App restart preserves task/session/history data.
15. Tests verify data persistence, event writing, redaction, and action policy behavior.
16. Feature critique workflow approves the slice before marking it complete.

---

## 12. Implementation Phases Revised

### Phase 0: Spikes and Architecture Decision

Deliverables:

- PTY/CLI spike result
- Browser/WebView spike result
- Native macOS services spike result
- final stack decision record

Gate:

Do not continue to deep product implementation until these are documented.

### Phase 1: Secure Foundation

Deliverables:

- Tauri/React/TypeScript app shell
- Apple-style design tokens and base components
- SQLite migration system
- workspace registry
- `~/Zoid/` visible folder creation
- app support directory creation
- Keychain service
- secret redaction service
- safe logging service
- action policy evaluator
- confirmation framework
- event writer
- entity link service
- settings shell

Acceptance:

- App launches.
- Schema migrates.
- folders are created.
- secrets can be stored/retrieved from Keychain test path.
- redaction test passes.
- event write/read works.
- action policy tests pass.

### Phase 2: First Vertical Slice

Deliver the Today -> Task -> CLI Session -> AgentRun -> ReviewRecord -> Notification -> History loop described above.

Gate:

Must pass tests, local verification, and feature critique approval.

### Phase 3: Notes, Files, and Local Knowledge

Deliverables:

- Markdown note create/edit/delete/trash
- frontmatter stable note ID
- note scanner/indexer
- note conflict handling
- file manager basic browse/open/preview
- safe rename/move/copy/trash with confirmation
- entity links from notes/files to tasks/products/runs
- history views per note/file/task

Acceptance:

- Markdown notes survive restart.
- manual file rename preserves identity.
- duplicate ID conflict is non-destructive.
- file destructive operations require confirmation.
- events are recorded.

### Phase 4: Code, Repos, Launch Gate

Scope note:

Phase 4 is intentionally narrowed to a lightweight native repo registry, truthful local integration states, Launch Gate evidence records, and protected-action policy previews. Full git status/diff/read operations, external GitHub/Vercel automation, and deploy execution are deferred to a later dedicated slice.

Deliverables:

- lightweight repo profile model and manual add/list/read primitives
- repo links to product/task via the existing entity-link model
- truthful GitHub/Vercel integration states where credentials are not configured; no fake connected data
- Launch Gate checks and verification evidence model
- deploy/push/merge policy previews that require review/confirmation before any future execution path
- Code workspace read-only surface for repo registry, truthful integration state, and Launch Gate policy preview

Acceptance:

- repo can be added, listed, read, and linked to product/task through native backend/bridge primitives.
- GitHub/Vercel/git/deploy status is truthful and does not claim connected/executed behavior without credentials or implementation.
- Launch Gate cannot be marked verified without at least one persisted evidence record.
- deploy/push/merge actions are policy previews only and require review + confirmation before any future execution path.

### Phase 5: Content and OmniSocials

Deliverables:

- content calendar
- content plan/piece model
- asset storage
- MaVoid content generation sessions
- specialist design/review gate records
- OmniSocials account/status integration
- media upload
- schedule/publish
- verification records
- failure reports

Acceptance:

- content piece can move through plan -> draft -> asset -> review -> schedule.
- failed generation/review/upload/schedule/verification fails closed.
- no post is scheduled/published without required review/confirmation policy.
- platform media constraints are enforced.

### Phase 6: Calendar, Gmail, Inbox, Business, Products

Deliverables:

- EventKit calendar read/create/edit confirmation flows
- Gmail OAuth/read/search/draft/send confirmation flows
- Inbox workspace
- contacts/companies/follow-ups
- Products workspace
- links across products, contacts, tasks, repos, notes, emails, events

Acceptance:

- Gmail send always requires confirmation.
- calendar create/edit/delete requires confirmation.
- internal implementation details are not leaked into public/user-facing copy.

### Phase 7: Browser Workspace and Advanced Widgets

Deliverables:

- work webview/capture workspace
- tab or saved page model
- browser captures
- screenshot/link to entities
- app verification evidence attachment
- configurable widgets per workspace

Acceptance:

- browser evidence can attach to Launch Gate.
- widgets can be shown/hidden/reordered/resized within simple limits.
- no full personal browser ambition is introduced.

### Phase 8: Packaging, Performance, Accessibility, Hardening

Deliverables:

- macOS packaging/signing path
- native notification polish
- error/empty/loading states
- accessibility pass
- performance pass
- log retention settings
- migration hardening
- documentation

Acceptance:

- packaged macOS app opens cleanly.
- secrets remain redacted.
- destructive actions confirm.
- UI feels polished and Apple-like.
- accessibility baseline passes.

---

## 13. Testing and Verification Requirements

Every implementation slice must include:

- unit tests for pure services
- integration tests for SQLite migrations/repositories
- policy tests for action confirmations/review requirements
- redaction tests for logs/events
- UI smoke tests for core flows where feasible
- local run/build verification
- manual verification notes for native/macOS behaviors that automated tests cannot cover
- feature critique workflow report with APPROVED verdict before marking done

Required verification report format:

- Commit status
- Build status
- Test status
- Manual verification performed
- Known blockers
- Screens/routes/pages checked where applicable
- Exact command output summary
- Feature critique verdict

Do not report “done” with fabricated output or unverified integration states.

---

## 14. UI Quality Requirements

Zoid must not drift into a generic dashboard.

Base requirements:

- system font stack
- clean sidebar
- toolbar/header per workspace
- split views where useful
- inspector panel pattern
- subtle materials/translucency where appropriate
- restrained shadows
- excellent empty states
- light/dark appearance
- command/session cards rather than raw terminal by default
- no noisy gradients/glow/cyberpunk styling
- no dense shortcut-first TUI UX as the main interface

Each new workspace must include:

- empty state
- loading state only for real data waits
- error/blocker state
- useful first real action
- real data source or explicitly marked unconfigured state

---

## 15. Integration State Rules

Every integration must represent truthfully one of these states:

- `not_configured`
- `needs_permission`
- `connecting`
- `connected`
- `auth_expired`
- `unavailable`
- `error`
- `disabled_by_policy`

Do not show fake connected data.
Do not silently ignore auth failures.
Do not store raw credentials outside Keychain.
Do not expose internal provider/infra details in public-facing copy unless approved.

---

## 16. Immediate Task Breakdown

### Task 1: Create spike documents

Files:

- Create: `Docs/spikes/2026-05-31-pty-cli-runtime-spike.md`
- Create: `Docs/spikes/2026-05-31-browser-webview-spike.md`
- Create: `Docs/spikes/2026-05-31-native-macos-services-spike.md`

Objective:

Document spike goals, commands/prototypes, results, constraints, and stack decision impact.

Verification:

- Each spike doc has Result: Pass / Partial / Fail.
- Each unsupported capability has fallback recommendation.

### Task 2: Create architecture decision record for stack

Files:

- Create: `Docs/adr/0001-desktop-stack-tauri-vs-electron.md`

Objective:

Record why Tauri remains preferred or why fallback is needed after spikes.

Verification:

- ADR references actual spike findings.
- ADR states decision, consequences, and revisit triggers.

### Task 3: Initialize app foundation

Files:

- Create/modify repository app files according to chosen stack.

Objective:

Create app shell, sidebar, Today placeholder backed by real workspace registry.

Verification:

- App launches locally.
- Sidebar lists all workspaces.
- No fake integration success states.

### Task 4: Add SQLite migrations and repositories

Objective:

Implement migration runner and core tables from this plan.

Verification:

- Fresh DB migrates successfully.
- Migration version is recorded.
- Basic insert/read tests pass.

### Task 5: Add security foundation

Objective:

Implement Keychain test path, redaction service, safe logging service, and action policy evaluator.

Verification:

- Keychain test credential roundtrip passes.
- redaction test proves secrets are removed from sample CLI output.
- policy tests prove send/publish/deploy/delete require confirmation.

### Task 6: Implement first vertical slice

Objective:

Build Today -> Task -> CLI Session -> AgentRun -> ReviewRecord -> Notification -> History.

Verification:

- Real CLI command runs from UI.
- output streams.
- task and agent run persist after restart.
- events are visible in history.
- notification appears in Today/Inbox.
- feature critique workflow returns APPROVED.

---

## 17. Open Decisions / Blockers to Resolve During Spikes

1. Tauri PTY quality: acceptable or fallback required?
2. Tauri WebView browser capture/console capability: what is actually supported?
3. EventKit bridge approach: direct Rust/Swift helper/plugin?
4. Keychain crate/plugin choice.
5. SQLite migration library choice.
6. Whether Node helper is needed for CLI/PTY orchestration.
7. Whether live process reattachment after app restart is required now or deferred.
8. Exact Apple-style component strategy.
9. Packaging/signing approach and whether sandboxing affects CLI/file access.

---

## 18. Explicit Not-Yet List

Do not build these in the first vertical slice:

- full Gmail integration
- full OmniSocials publishing
- autonomous recurring content publishing
- GitHub PR/merge flows
- Vercel deployment controls
- Apple Notes import
- full file manager
- full browser replacement
- mobile/private sync backend
- plugin/extension platform
- heavy analytics workspace
- personal productivity surveillance/time tracking
- voice/audio messages
- iMessage/SMS
- proposal/PDF builder
- global dry-run simulation engine

---

## 19. Definition of Done for Any Feature

A Zoid feature is done only when:

1. It uses real data or truthful unconfigured/empty states.
2. It has persistence where required.
3. It writes Events for meaningful actions.
4. It respects the action policy matrix.
5. It redacts secrets from logs/events/UI.
6. It has tests for core service logic.
7. It has local build/test verification output.
8. It has manual verification notes for native/UI behavior.
9. It does not leak private implementation details into public-facing copy.
10. It passes feature critique workflow with APPROVED verdict.

---

## 20. Recommended First Execution Command for Implementer

Before writing product code, the implementer should run repository discovery and create the spike docs:

```bash
mkdir -p Docs/spikes Docs/adr
```

Then complete Spike A, Spike B, and Spike C with real prototype output before finalizing the app architecture.

Do not skip the spikes. They decide whether Tauri is viable for Zoid’s most important workflows.
