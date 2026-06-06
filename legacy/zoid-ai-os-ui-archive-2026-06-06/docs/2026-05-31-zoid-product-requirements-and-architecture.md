# Zoid Product Requirements and Architecture Plan

Date: 2026-05-31
Owner: Ziad Salah
Status: Draft v0.1 from brainstorming/grill session

---

## 1. Product Definition

Zoid is a native macOS-first personal AI workspace OS.

It is not a real kernel-level operating system. It is a desktop operating layer for Ziad's work: code, agents, notes, content publishing, automations, business operations, products, files, browser-based research/verification, inbox, calendar, and history.

Zoid should feel like a first-party Apple productivity app, while internally acting as an AI command center and orchestration shell.

### One-line vision

Zoid is a local-first desktop AI workspace that turns daily work into an organized, review-gated, searchable, automatable operating system.

### Product personality

- Native-feeling
- Apple-style
- Calm but powerful
- Visual first, not terminal-first
- Conversational where execution/logs appear
- Local-first and user-owned
- Serious enough for production work
- Structured enough to become the daily driver

---

## 2. Core Product Decisions

### Locked decisions

- Name: Zoid
- Platform: macOS-first desktop app; Windows later
- UI direction: Apple-style native desktop UI
- Interface priority: native desktop UI first; terminal/TUI is hidden behind a clean conversational execution layer
- Data model: workspace-first
- Home: Today View
- Storage: local-first, hybrid visible folder + hidden app support
- Sync: private cloud sync later for mobile companion
- Profile: local profile only; no Zoid account initially
- AI execution: CLIs only; mainly Hermes, but user-configurable CLI profiles
- Onboarding: no forced onboarding wizard
- Search: simple search inside workspaces; AI-assisted search where needed; no big universal search module now
- Review: internal reviewer required for consequential work
- Permissions: per-module policies with simple UI
- Notifications: both in-app and native macOS
- Calendar: built-in calendar plus Apple Calendar integration
- Email: Gmail read/send only
- Notes: native notes replace Apple Notes; Apple Notes only import/migration source
- Content: full OmniSocials publishing pipeline included
- Files: full top-level file manager
- Browser: full work browser, not personal browser replacement
- Mobile: important future companion app, backed by private cloud sync

### Explicitly not included for now

- Real OS/kernel/distro
- Voice/audio messages
- iMessage/SMS integration
- Full analytics workspace
- Plugin/extension platform
- Zoid cloud account/login
- Proposal/document builder
- Backup/restore focus
- Global dry-run mode
- Dedicated migration module
- Full universal search workspace
- Personal time tracking
- Full personal browser replacement

---

## 3. Target User and Use Cases

### Primary user

Ziad Salah: founder/product owner/operator building and managing MaVoid, Leadra, Zoid AI, content systems, automations, and client/business workflows.

### Main work patterns Zoid must support

1. Build and ship software
   - manage repos
   - run AI agents
   - inspect diffs
   - run tests/builds
   - review work
   - deploy
   - verify production
   - record history

2. Operate AI agents
   - start runs
   - monitor progress
   - inspect logs
   - receive completion/blocker notifications
   - attach runs to tasks/products/repos/content
   - require reviewer approval where consequential

3. Manage native knowledge
   - replace Apple Notes
   - create Markdown-backed notes
   - store metadata/search/links in SQLite
   - import Apple Notes as legacy data
   - link notes to tasks/products/repos/content/business/contact records

4. Run MaVoid content operations
   - plan content
   - generate AI Intel Briefs
   - generate enterprise carousel/authority posts
   - create assets
   - review internally
   - upload to OmniSocials
   - schedule/publish
   - verify and record results

5. Run business operations
   - lightweight CRM
   - contacts/companies
   - follow-ups
   - Gmail read/send
   - calendar context
   - notes/tasks linked to clients and products

6. Manage files and assets
   - browse files
   - preview
   - move/rename/copy/delete with confirmation
   - link files to Zoid objects
   - manage content assets and repo folders

7. Browse work-related web pages
   - research
   - app verification
   - save links/screenshots
   - link pages to tasks/notes/products/content

8. Start each day from Today View
   - see priorities
   - see blockers
   - see agent/automation state
   - see calendar
   - see content queue
   - see repos needing attention
   - see inbox items

---

## 4. Information Architecture

### Top-level workspaces

1. Today
2. Code
3. Agents
4. Notes
5. Content
6. Automations
7. Business
8. Products
9. Files
10. Browser
11. Inbox

### Workspace philosophy

Workspaces are top-level operating areas. Projects/products/clients/repos/tasks are entities that can appear inside multiple workspaces through links.

Example:
Leadra may appear in:
- Code as a repo/product app
- Products as a product
- Agents as task/run history
- Notes as decisions/context
- Browser as production verification pages
- Inbox as alerts/follow-ups
- Today as a current focus item

---

## 5. Global Data Model

### Core entities

- Workspace
- Task
- Note
- Repository
- AgentRun
- Session
- Automation
- AutomationRun
- ContentPlan
- ContentPiece
- MediaAsset
- PublishRecord
- Product
- Contact
- Company
- CalendarItem
- EmailMessageRef
- FileRef
- BrowserTab
- BrowserCapture
- LaunchGate
- ReviewRecord
- Notification
- Event

### Common fields for most entities

- id
- type
- title
- workspace_id or workspace_key
- status
- source
- source_ref
- created_at
- updated_at
- archived_at
- deleted_at/tombstone flag for future sync
- tags
- metadata_json
- linked_entities
- attention_level

### Universal event/history system

Every important action creates an Event.

Event fields:
- id
- type
- timestamp
- actor_type: user | system | agent | automation | integration
- actor_id
- workspace_key
- summary
- severity: info | success | warning | error | critical
- linked_entities
- source
- metadata_json

Event examples:
- task created/completed
- note created/imported/updated
- repo scanned
- commit detected
- branch changed
- agent run started/completed/failed
- review approved/required fixes
- build/test passed/failed
- deployment created/verified/failed
- content generated/reviewed/scheduled/published/failed
- automation run failed
- Gmail message sent
- calendar event created
- file moved/deleted
- browser page captured

History views:
- Global recent history
- Today activity
- Per repo
- Per product
- Per task
- Per note
- Per content piece
- Per automation
- Per agent run
- Per contact/company

---

## 6. UI/UX Requirements

### Visual direction

Zoid must feel like a first-party macOS productivity app.

Design traits:
- Apple-style typography using system font stack
- clean sidebar
- toolbar/header per workspace
- split views
- inspector panels
- soft material/translucency where appropriate
- restrained shadows
- subtle rounded corners
- light/dark system appearance
- excellent empty states
- no flashy SaaS gradient look
- no cyberpunk/glow UI
- no raw terminal-first experience

Recommended font stack:

```css
font-family: -apple-system, BlinkMacSystemFont, "SF Pro Display", "SF Pro Text", system-ui, sans-serif;
```

### Layout model

Main desktop shell:
- Left sidebar: workspaces
- Main panel: selected workspace
- Optional right inspector: selected entity details
- Bottom/right Sessions panel: conversational terminal/agent sessions when opened
- In-app notification center/inbox
- Native macOS notifications

### Customizable widgets

Each workspace can have configurable widgets:
- add/remove
- reorder
- basic size selection: small/medium/large
- reset to default
- no full layout-builder complexity

Today widget examples:
- Daily Brief
- Needs Attention
- Active Agents
- Tasks
- Calendar
- Content Queue
- Dirty Repos
- Gmail Summary
- Automation Failures
- Recent History

### Conversational execution UI

Terminals and agents should not feel like raw terminals by default.

Sessions use:
- message bubbles
- agent icons/avatars
- command cards
- output cards
- progress/status cards
- collapsible raw logs
- summary cards
- error cards with suggested fixes

Each session can switch between:
- Clean Mode: conversational/log cards
- Raw Mode: true PTY terminal

External terminal fallback is available.

---

## 7. Storage Architecture

### Visible user-owned folder

Default visible folder:

```text
~/Zoid/
```

Suggested structure:

```text
~/Zoid/
  Notes/
  Content/
  Assets/
    Images/
    Carousels/
    BrowserCaptures/
  Exports/
  Imports/
    Apple Notes/
  Files/
  Products/
```

### Hidden app-managed folder

macOS app support folder:

```text
~/Library/Application Support/Zoid/
```

Suggested structure:

```text
~/Library/Application Support/Zoid/
  zoid.sqlite
  indexes/
  cache/
  logs/
  sessions/
  automations/
  config/
```

### Storage rules

- Markdown notes live visibly under `~/Zoid/Notes/`.
- Content assets live visibly under `~/Zoid/Content/` or `~/Zoid/Assets/`.
- SQLite stores metadata, links, status, history, indexes, and app state.
- Logs are app-managed but exportable later.
- Secrets go to macOS Keychain, not plain text files.
- Raw secrets must be redacted from logs/events.

### Security

Security target: encrypt sensitive local data at rest where practical.

Requirements:
- Store credentials in Keychain:
  - Gmail OAuth tokens
  - OmniSocials API credential
  - GitHub/Vercel tokens
  - CLI secrets references
- Redact secrets in captured command output.
- Do not store raw API keys in prompts, logs, events, or visible files.
- Use OS user permissions for visible Markdown/assets.
- Sensitive app-managed DB fields can be encrypted if feasible.

---

## 8. Technology Architecture

### Recommended stack

Desktop app:
- Tauri + React + TypeScript

Native/system layer:
- Rust sidecar/Tauri commands
- Optional Node helpers for CLI orchestration if needed

Database:
- SQLite
- SQLite FTS for notes/tasks/content metadata where useful

UI:
- Custom Apple-style React component system
- Avoid generic dashboard template styling

Terminal/PTY:
- embedded terminal emulator for Raw Mode
- PTY backend abstraction
- external terminal/iTerm/Finder open fallback

Integrations:
- CLI subprocess runner abstraction
- Git/GitHub/Vercel integration services
- Gmail OAuth/API integration
- Apple Calendar through EventKit bridge
- OmniSocials REST integration
- File system service
- Browser webview service

### High-level architecture

```text
Zoid Desktop UI (React/Tauri)
  ├─ Workspace Views
  ├─ Widget System
  ├─ Sessions UI
  ├─ Inspector Panels
  └─ Settings

Tauri Native Layer / Local Services
  ├─ SQLite Repository
  ├─ Event Bus / History Writer
  ├─ Permission Policy Engine
  ├─ Notification Service
  ├─ CLI Runner
  ├─ PTY Service
  ├─ File Service
  ├─ Git Service
  ├─ Browser Service
  ├─ Calendar Bridge
  ├─ Gmail Service
  ├─ OmniSocials Service
  └─ Keychain Service

Local Data
  ├─ ~/Zoid visible files
  └─ ~/Library/Application Support/Zoid app data
```

### Service boundaries

Do not build a plugin platform, but keep internal service boundaries clean:
- `workspace-service`
- `entity-link-service`
- `event-service`
- `task-service`
- `notes-service`
- `repo-service`
- `agent-runner-service`
- `automation-service`
- `content-service`
- `publishing-service`
- `calendar-service`
- `gmail-service`
- `file-service`
- `browser-service`
- `notification-service`
- `permission-service`
- `review-service`

---

## 9. Workspace Requirements

## 9.1 Today Workspace

Purpose:
The daily operating surface.

Must show:
- AI Daily Brief widget
- Needs Attention widget
- Active Work widget
- Today Tasks widget
- Calendar widget
- Content Queue widget
- Active Agents widget
- Dirty/attention repos widget
- Automation Failures widget
- Recent History widget
- Gmail Summary widget if connected

Attention rules:
- blocked agent
- failed automation
- failed deploy/test/build
- content generated but not approved/scheduled
- content publishing failed
- Gmail send awaiting confirmation
- calendar/task due soon
- repo dirty beyond configured threshold
- launch gate verification pending
- review required/required fixes

## 9.2 Code Workspace

Purpose:
Manage software work.

Features:
- repo discovery
- manual repo add
- managed repos
- repo profiles
- git status
- branch/commit/diff views
- changed files
- GitHub integration
- Vercel integration
- Launch Gate
- review records
- linked tasks/agents/notes/products

Repo discovery:
- scan suggestions from common folders:
  - `~/Documents/GitHub`
  - `~/Developer`
  - `~/Code`
  - `~/Projects`
  - custom folders
- user approves which repos become managed

Repo profiles:
- Production App
- Website
- Library/Package
- Experiment/Spike
- Client Project
- Content/Docs Repo

Repo profile fields:
- profile_type
- default_branch
- package_manager
- test_command
- lint_command
- build_command
- deploy_provider
- production_urls
- review_required
- deploy_verification_required
- notification_level
- linked_product/client

GitHub full integration:
- remotes
- branches
- commits
- PRs
- issues optional
- checks/CI
- create PR
- comments/reviews
- merge with confirmation
- link PRs to tasks/agent runs

Vercel full integration:
- projects
- deployments
- preview URLs
- production URLs
- aliases
- build logs
- deploy status
- redeploy/trigger with confirmation
- production verification records

## 9.3 Agents Workspace

Purpose:
Run and monitor AI/CLI workers.

AI execution rule:
Zoid runs CLIs only. It does not directly call LLM APIs initially.

Agent profile fields:
- name
- command
- args
- working directory behavior
- environment variables reference
- supported modes
- output parser/log mode
- status detection
- default permissions
- reviewer profile flag

Default expected profiles:
- Main Assistant
- Builder
- Reviewer
- Content
- Deployment Verifier
- Notes Organizer
- Automation Runner

Features:
- start agent run
- attach to task/entity
- stream logs/output
- show status
- notify on complete/fail/block
- inspect run history
- rerun/retry
- reviewer agent records

AgentRun fields:
- id
- profile_id
- task_id
- workspace
- repo_path/context
- prompt
- status
- started_at
- completed_at
- duration
- logs_path
- output_summary
- linked_entities
- blocker/error
- review_status

## 9.4 Notes Workspace

Purpose:
Replace Apple Notes with Zoid-native knowledge.

Architecture:
- Markdown bodies in visible files
- SQLite metadata, links, status, tags, search, events

Features:
- create/edit/delete note
- collections/folders
- tags
- backlinks/entity links
- workspace association
- AI summarize/organize through configured CLI where available
- import Apple Notes via `memo` CLI
- link notes to tasks/products/repos/content/contacts

Notes storage example:

```text
~/Zoid/Notes/
  Daily/
  Code/
  Content/
  Business/
  Products/
  Imported/Apple Notes/
```

Note metadata:
- id
- title
- file_path
- workspace
- collection
- tags
- source
- source_ref
- created_at
- updated_at
- last_indexed_at
- summary

Apple Notes role:
- migration/import source only
- not long-term source of truth
- no Apple Notes dependency after import

## 9.5 Content Workspace

Purpose:
Full MaVoid content planning, production, review, scheduling, and publishing.

Must support:
- content calendar
- content pillars
- MaVoid brand voice/context
- AI Intel Brief pipeline
- enterprise carousel/authority pipeline
- draft generation
- visual/carousel asset generation
- specialist design review gate
- caption/platform adaptation
- OmniSocials account/status integration
- media upload
- scheduling/publishing
- post verification
- publishing history
- failure reports

Default MaVoid content model:
- Morning AI Intel Brief visual post, around 09:00 Africa/Cairo
- Later enterprise carousel/authority visual post, around 18:00 Africa/Cairo
- Designed media posts, not text-only unless explicitly selected

OmniSocials role:
- external distribution/scheduling layer

Zoid owns:
- strategy
- calendar
- drafts
- assets
- approvals/reviews
- captions
- schedule intent
- publishing history
- verification records

Publishing pipeline:
1. Plan
2. Generate
3. Design
4. Review
5. Upload media
6. Schedule/publish
7. Verify scheduled post
8. Record event/history

Safety:
- For MaVoid recurring content: autonomous scheduling after internal reviewer approval
- Fail closed: if generation/review/upload/schedule/verification fails, do not publish; create failure report
- For one-off/client/sensitive content: human approval by policy unless configured otherwise

Platform constraints to encode:
- Instagram requires media
- Instagram carousel max around 10 media items
- X/Twitter max 4 media items; long carousels should omit X or adapt
- Verify scheduled post status, accounts, media, and schedule time

## 9.6 Automations Workspace

Purpose:
Manage first-class automations.

Automation fields:
- id
- name
- workspace
- type
- schedule
- enabled
- status
- last_run_at
- next_run_at
- last_result
- failure_policy
- notification_policy
- linked_entities
- logs

AutomationRun fields:
- id
- automation_id
- started_at
- completed_at
- duration
- status
- trigger
- inputs
- outputs
- logs
- errors
- linked_task
- linked_entities

Default automations:
- Daily MaVoid content publishing
- Morning AI Brief
- Repo scanner
- Agent monitor
- Notes indexer
- Content calendar monitor
- Deployment verifier

Actions:
- run now
- pause/resume
- inspect logs
- retry failed run
- change schedule
- disable publishing while keeping draft generation if policy supports it

## 9.7 Business Workspace

Purpose:
Lightweight CRM and business operations.

Features:
- Contacts
- Companies/Clients
- follow-ups
- linked Gmail
- linked tasks
- linked notes
- linked products
- SOP notes
- proposal/offer notes through Notes, not a dedicated document builder

Contact fields:
- name
- email
- phone
- company
- role
- tags
- notes
- source
- linked emails
- linked tasks
- linked notes
- linked products

Company fields:
- name
- status
- contacts
- notes
- follow-ups
- linked emails
- linked tasks
- linked products

Not included:
- heavy CRM pipeline
- proposal/PDF designer
- full document builder

## 9.8 Products Workspace

Purpose:
First-class product operating layer.

Product examples:
- Leadra
- Zoid OS
- Zoid AI
- MaVoid Unified Platform
- MaVoid website/internal systems
- client products

Product fields:
- name
- status
- type
- description
- owner
- repos
- notes
- tasks
- deployments
- releases
- roadmap items
- content pieces
- decisions
- clients/companies

Views:
- Overview
- Roadmap
- Tasks
- Repos
- Deployments
- Notes/Decisions
- Content
- Automations
- Timeline/History

## 9.9 Files Workspace

Purpose:
Full Zoid-aware file manager.

Scope:
- browse local folders
- pinned folders
- `~/Zoid/`
- repos
- Downloads/Desktop/Documents if user adds them
- content assets
- imports/exports

Actions:
- open
- preview
- rename
- move
- copy
- duplicate
- delete with confirmation/trash
- create folder
- reveal in Finder
- link file/folder to Zoid entity

Preview support:
- Markdown
- text/code
- images
- PDFs where feasible
- basic media metadata later

Repo-aware behavior:
- detect git repo
- show file status inside managed repo
- open in Code workspace
- link changed file to event/task/agent run

## 9.10 Browser Workspace

Purpose:
Work browser only.

Scope:
- research
- app verification
- deployed app checks
- content source collection
- saved pages
- screenshots
- links to tasks/notes/products/content

Features:
- multiple tabs
- pinned tabs
- bookmarks/work bookmarks
- history for work browsing
- save page to note
- summarize page through CLI/agent where configured
- extract key points
- save screenshot
- link page to entity
- basic console/error capture if feasible for app verification

Not a personal browser replacement:
- no password manager ambition
- no consumer browsing history
- no extension ecosystem
- no browser sync

## 9.11 Inbox Workspace

Purpose:
Unified attention center.

Includes:
- Zoid notifications
- agent completions/blockers
- automation failures
- review approvals/required fixes
- content publishing status
- calendar/task reminders
- Gmail read/send

Gmail scope:
- connect Gmail/Google Workspace
- read recent messages
- search messages
- open thread/message
- summarize email/thread through configured CLI/agent
- draft reply
- send reply after confirmation
- compose new email after confirmation
- link email to task/note/contact/company/product
- convert email to task/follow-up

Not included:
- archive
- delete
- labels
- mark read/unread
- filters/rules
- full mailbox management

Safety:
- reading/searching allowed after auth
- drafting allowed
- send always requires confirmation
- attachments require confirmation

---

## 10. Tasks

Tasks are first-class objects.

Task fields:
- id
- title
- description
- workspace
- status
- priority
- due_at
- created_at
- updated_at
- completed_at
- tags
- linked_entities
- source
- history

Statuses:
- Inbox
- Todo
- In Progress
- Waiting/Blocked
- Needs Review
- Done
- Archived

Task links:
- repo
- note
- content piece
- content plan
- agent run
- deployment
- contact/company
- automation
- decision
- file path
- URL

Rule:
Every non-trivial AI agent run should create or attach to a task.

---

## 11. Calendar

Zoid includes an internal calendar plus Apple Calendar integration.

Calendar covers:
- content publishing slots
- scheduled OmniSocials posts
- task due dates
- automation schedules
- agent/deploy reminders
- follow-ups
- product launch dates
- business deadlines

Views:
- Today
- Week
- Month
- Content calendar
- Automation schedule
- Timeline per workspace

Apple Calendar integration:
- read upcoming Apple Calendar events
- show Today/Week events in Zoid
- create event from Zoid task/content/follow-up after confirmation
- link Apple Calendar event to Zoid task/note/client/product
- conflict warnings where relevant

Implementation:
- EventKit bridge through Tauri/native code
- macOS Calendar permission required
- external event IDs stored in SQLite

Safety:
- read allowed after permission
- create requires confirmation
- edit/delete always confirm

---

## 12. Review Gate

Review is required for consequential actions.

Review required when Zoid:
- changes files
- changes code
- commits/pushes/merges
- deploys
- publishes/schedules content
- sends email
- imports/migrates data
- modifies integrations/settings
- edits business/client records
- creates/changes automations
- performs destructive/bulk file operations

Review not required for:
- reading/searching
- opening files
- viewing repo status
- viewing calendar/email
- drafting private notes
- creating simple personal tasks

Reviewer Agent:
- internal reviewer launched through configured CLI profile
- produces ReviewRecord
- verdicts:
  - APPROVED
  - REQUIRED FIXES
  - BLOCKED / INSUFFICIENT EVIDENCE

ReviewRecord fields:
- id
- reviewed_entity_type
- reviewed_entity_id
- reviewer_profile_id
- verdict
- evidence_summary
- required_fixes
- created_at
- linked_events

---

## 13. Launch Gate

Launch Gate is first-class for production work.

States:
- Not Ready
- Ready to Deploy
- Deploying
- Deployed / Verification Pending
- Verified
- Failed / Blocked
- Rolled Back

Checks:
- working tree status
- typecheck
- lint
- unit/integration tests
- E2E tests if configured
- build
- review/critique approval
- commit pushed
- GitHub checks
- Vercel deploy status
- production URL HTTP status
- browser console errors
- route smoke checks
- asset load checks
- custom verification commands

LaunchGate fields:
- id
- repo_id
- product_id
- task_id
- state
- commit_sha
- pr_url
- deployment_id
- production_url
- verification_evidence
- final_verdict
- created_at
- updated_at

UI:
- Today status card
- per-repo Launch Gate panel
- product release timeline
- Inbox attention item if pending/failed
- Run verification action
- Mark verified only if evidence exists

---

## 14. Permission Policies

Per-module permission policies with simple UI.

Modules:
- Files
- Gmail
- Content/OmniSocials
- Code/Git/GitHub
- Deployments/Vercel
- Automations
- Calendar
- CLI/Agents
- Notes import/migration
- Business/CRM

Policy values:
- allowed automatically
- ask before action
- always blocked
- require reviewer approval
- require human confirmation

Default safety:
- Reads/searches: allowed
- Draft creation: allowed
- Edits: allowed when inside a clear task/session
- Sends/publishes/deploys/deletes/credential changes: confirmation/review required
- Bulk/destructive actions: always confirm

Permission prompt examples:
- “Zoid needs confirmation because this action will send an email.”
- “Zoid needs review because this will publish content.”
- “Zoid needs approval because this will delete 12 files.”

---

## 15. Notifications

Two notification layers:

### In-app Inbox notifications

Persistent/actionable:
- grouped by workspace/type
- linked to relevant entity
- includes approvals/blockers/failures/completions/reminders

### Native macOS notifications

Used for:
- agent completed/blocked
- automation failed
- content scheduled/published/failed
- deployment verified/failed
- important calendar/task reminders
- pending confirmation if important

Notification fields:
- id
- type
- title
- message
- severity
- workspace
- linked_entities
- read_at
- created_at
- action_url/app route

---

## 16. Automations and Runtime Tracking

Zoid tracks runtime durations, not personal productivity time.

Track durations for:
- agent runs
- automation runs
- builds/tests/deployments
- terminal/sessions
- content pipeline steps
- browser verification runs

Do not track:
- personal active time
- idle/active computer behavior
- surveillance-style productivity metrics

---

## 17. Private Sync and Mobile Future

Mobile companion is important, but not desktop-first build blocker.

Future mobile should support:
- Today View
- Inbox/approvals
- tasks
- notes capture/read
- content status
- automation failures
- agent completion notifications
- Gmail read/draft maybe
- calendar view
- approve/reject review gates

Sync model:
- private cloud sync eventually
- desktop remains local-first
- mobile connects to private backend
- sync selected/safe data by policy
- raw secrets do not sync
- local files/assets sync optionally, not by default

Data model requirements now:
- stable IDs
- timestamps
- tombstones/deleted state
- source metadata
- device IDs later
- conflict policy later

---

## 18. Build Phases

The user asked for the full version, not a small V1. Still, implementation must be sequenced. These are execution phases, not product-scope reductions.

### Phase 0: Foundation

Goal:
Create app shell, data layer, design system, event model, storage directories.

Deliverables:
- Tauri + React + TypeScript desktop app
- Apple-style shell/sidebar
- SQLite setup
- `~/Zoid/` creation
- app support directory
- local profile
- workspace registry
- event/history writer
- settings shell

Acceptance:
- app launches on macOS
- sidebar shows all workspaces
- Today renders default widgets
- SQLite migrations run
- visible folders created
- events can be written/read

### Phase 1: Tasks, Notes, Files, History

Goal:
Build core local objects.

Deliverables:
- task system
- native Markdown notes
- notes metadata/indexing
- file manager basic/full actions
- entity links
- history/timeline views
- review record model

Acceptance:
- create/edit notes
- notes saved as Markdown
- tasks can link to notes/files
- file manager can browse/manage files with confirmations
- events recorded for major actions

### Phase 2: Sessions, CLI Runner, Agents, Reviewer

Goal:
Make Zoid an execution OS.

Deliverables:
- CLI profile config
- subprocess runner
- PTY/raw terminal mode
- conversational session UI
- agent run records
- reviewer agent flow
- notifications on completion/failure

Acceptance:
- user can configure a CLI profile
- run command/agent from Zoid
- output streams into clean session UI
- raw logs available
- agent run stored/history-linked
- reviewer produces review record

### Phase 3: Code, GitHub, Vercel, Launch Gate

Goal:
Support production software workflow.

Deliverables:
- repo discovery/add
- managed repos
- repo profiles
- git status/diff/commits
- GitHub integration
- Vercel integration
- Launch Gate
- production verification records

Acceptance:
- scan and approve repos
- show repo cards/status
- inspect diffs/commits
- link repo to product/tasks/agent runs
- run Launch Gate checks
- record verified/failed state

### Phase 4: Content and OmniSocials Publishing

Goal:
Full MaVoid content engine.

Deliverables:
- content calendar
- content plans/pieces
- asset storage
- content generation sessions
- design/review gate
- OmniSocials account/status integration
- media upload
- schedule/publish
- verification records
- failure reports

Acceptance:
- create content piece
- generate draft/assets through configured CLI/automation
- reviewer approves/requires fixes
- upload media to OmniSocials
- schedule post
- verify scheduled post
- record publish event/status

### Phase 5: Calendar, Gmail, Inbox, Business, Products

Goal:
Complete operations layer.

Deliverables:
- Apple Calendar EventKit integration
- Gmail read/send
- Inbox workspace
- lightweight CRM
- Products workspace
- follow-ups
- linking across contacts/products/tasks/emails

Acceptance:
- show Apple Calendar events
- create calendar event after confirmation
- read/search Gmail
- draft/send email after confirmation
- create contact/company
- link email to contact/task/product
- Products overview/timeline works

### Phase 6: Browser Workspace and Advanced Widgets

Goal:
Complete work browser and configurable workspace dashboards.

Deliverables:
- browser tabs
- bookmarks/history
- save page/screenshot
- link browser captures
- app verification helpers
- widget edit mode
- per-workspace widget layouts

Acceptance:
- open multiple work tabs
- capture screenshot/link to note/task/product
- browser verification evidence can attach to Launch Gate
- widgets can be rearranged/shown/hidden

### Phase 7: Polish, Security, Packaging

Goal:
Make it feel like a real product.

Deliverables:
- Keychain integration
- secret redaction
- native notifications polish
- permissions UI
- error states
- empty states
- macOS packaging/signing path
- performance pass
- accessibility pass

Acceptance:
- secrets do not appear in logs/events
- native notifications link into app
- destructive actions confirm
- UI feels polished and Apple-like
- packaged macOS app opens cleanly

---

## 19. Acceptance Criteria for Full Product

Zoid is “full-product complete” when:

1. It opens as a polished macOS desktop app.
2. Today View is useful as the daily operating surface.
3. All top-level workspaces exist and have real functionality.
4. Tasks are first-class and link to entities.
5. Notes replace Apple Notes for daily use.
6. Files can be browsed and managed safely.
7. Repos can be tracked, reviewed, and launch-gated.
8. AI/CLI sessions can run inside Zoid with clean conversation UI.
9. Reviewer Agent can approve/reject consequential work.
10. MaVoid content can be generated, reviewed, scheduled through OmniSocials, verified, and recorded.
11. Automations are visible, controllable, and logged.
12. Apple Calendar appears in Zoid and can create linked events with confirmation.
13. Gmail can be read and sent from Zoid with confirmation.
14. Business contacts/companies/follow-ups are usable.
15. Products connect repos, tasks, notes, launches, and content.
16. Browser workspace supports work research/capture/verification.
17. Inbox shows attention items and actionable notifications.
18. Native macOS notifications work.
19. History/events connect the entire system.
20. Credentials are stored securely and secrets are redacted.

---

## 20. Open Critical Risks

These are not feature questions; they are implementation risks to validate early.

1. Tauri embedded PTY quality
   - If terminal/PTY embedding is weak, consider Electron before deep build.

2. Apple-style UI quality in React/Tauri
   - Requires custom component system and strong design discipline.

3. CLI runner reliability
   - Long-running agent processes need robust process supervision, log streaming, cancellation, and resume strategy.

4. OmniSocials publishing reliability
   - Need retries, verification, platform constraints, media upload stability.

5. Gmail OAuth complexity
   - Need secure token handling and clear confirmation UX for sends.

6. Event/history scale
   - Avoid bloating DB/logs with raw massive outputs; store summaries + log file refs.

7. Review gate friction
   - Consequential reviews must improve trust without making simple operations annoying.

8. Files workspace safety
   - Full file manager requires careful destructive confirmation and trash behavior.

9. Browser workspace complexity
   - Keep it work-focused; do not accidentally build a full consumer browser.

10. Future mobile/private sync
   - Data model must be sync-ready without forcing cloud complexity now.

---

## 21. Immediate Next Steps

1. Create repository for Zoid.
2. Choose stack finalization after quick PTY prototype:
   - Tauri + React + TypeScript preferred
   - validate embedded terminal/PTY in a spike
3. Create design tokens/component primitives for Apple-style UI.
4. Implement Phase 0 foundation.
5. Implement core entities and event system before complex integrations.
6. Build one vertical slice before all modules:
   - Today + Task + Session + AgentRun + Review + History
7. Then expand into Code/Launch Gate and Content/OmniSocials.

Recommended first vertical slice:

“Start a CLI agent task from Today, stream it in a clean conversation session, create a task, produce a reviewer record, notify on completion, and record all events in history.”

This proves the heart of Zoid before adding every workspace.
