# Zoid Implementation Tracker

Source plan: `/Users/ziadnasreldin/Zoid/Docs/2026-05-31-zoid-implementation-plan-v1.md`

Purpose: durable execution tracker for building the actual Zoid app. Each phase includes planning, database, backend/native, Tauri bridge, frontend, tests, manual verification, local verification, and critique/review gates.

Status legend: `[ ]` pending, `[~]` in progress, `[x]` complete, `[!]` blocked.

Non-negotiable gates for every implementation slice:
- Real data or truthful empty/unconfigured states only.
- SQLite/app-support persistence where required.
- Events for meaningful actions.
- Action policy for consequential actions.
- Secret redaction for logs/events/UI/errors.
- Local verification output recorded.
- Native/macOS manual notes where automation cannot verify.
- Feature critique workflow approved before calling the slice done.
- Use `npm run verify:local` for routine gates; do not run `npm run verify:release` unless intentionally checking a release/DMG.

---

## Phase 0 — Spikes and Architecture Decision

- [x] P0.01 Read implementation plan and PRD; confirm first objective and non-negotiable rules.
- [x] P0.02 PTY/CLI runtime spike doc exists with result, prototype evidence, constraints, and fallback recommendation.
- [x] P0.03 Browser/WebView spike doc exists with result, supported/unsupported capture behavior, and fallback recommendation.
- [x] P0.04 Native macOS services spike doc exists with Keychain, notifications, EventKit, app-support findings, and fallback recommendation.
- [x] P0.05 ADR for Tauri vs Electron exists and references actual spike findings, consequences, and revisit triggers.
- [x] P0.06 Re-check spike docs + ADR before relying on them for deep product implementation.
  - 2026-06-01 re-check: all three spikes and ADR are Partial/Accepted-with-constraints, not a blank check. Proceed with Tauri, but keep PTY UI streaming, Browser screenshot/data-store, Keychain, EventKit, and notification click routing as staged verification gates.

---

## Phase 1 — Secure Foundation

- [x] P1.00 Git hygiene: preserve and commit already-approved Secure Foundation Services v1 working-tree changes.
  - 2026-06-01 preserved in commit after inspecting diff/review artifacts and rerunning `npm run verify:local` successfully.
- [x] P1.01 Discovery: inspect current app foundation, Tauri commands, UI state, package scripts, tests, review artifacts, DB/app-support behavior.
  - 2026-06-01 inventory: `src-tauri/src/lib.rs` owns current foundation command, app-support/visible-folder creation, migrations, workspace seed, event writer, redaction, safe logging, action policy, and Rust tests. `src/App.tsx` is a single-shell React UI with fallback workspace registry and truthful native-status preview behavior. `scripts/verify-local.sh --skip-package` is the routine gate behind `npm run verify:local`; release/DMG gate remains `npm run verify:release`. Review artifacts include approved `secure-foundation-services-v1` plus earlier app/packaging/local-verification reviews. Current DB schema is only foundation workspaces/events/event_targets; deeper Phase 1 repository/settings/entity-link services remain pending.
- [ ] P1.02 Backend/native: app-support path service for `~/Library/Application Support/Zoid` with logs, database, config, safe directory creation.
- [ ] P1.03 Backend/native: visible user folder creation at `~/Zoid` with starter directories and non-destructive behavior.
- [ ] P1.04 Database: SQLite connection management, migration runner, migration version tracking, fresh DB creation, idempotent re-run behavior.
- [ ] P1.05 Database: core schema for workspaces, settings, events, entity_links, log/file references, action confirmations/policies where needed.
- [ ] P1.06 Database: repository/helper primitives for insert/read/list/update with typed errors and tests.
- [ ] P1.07 Backend: canonical workspace registry service listing all 14 workspaces with truthful availability/integration states.
- [ ] P1.08 Backend: settings service for local app preferences and integration statuses without secrets outside Keychain.
- [ ] P1.09 Backend/security: Keychain test path/read-write-delete service or truthful blocked/unverified native status.
- [ ] P1.10 Backend/security: secret redaction for logs, events, metadata JSON, obvious tokens/keys, nested values; JSON remains valid.
- [ ] P1.11 Backend/security: safe log writer under app-support logs with filename/path sanitization, redaction, size/rotation basics.
- [ ] P1.12 Backend/security: action policy evaluator for read/create/update/delete/send/publish/deploy/file/process actions.
- [ ] P1.13 Backend: confirmation decision records/framework for consequential actions.
- [ ] P1.14 Backend: generic event writer/reader with entity links, metadata redaction, timestamps, action type, outcome, source.
- [ ] P1.15 Backend: entity link service for tasks, notes, products, files, repos, runs, emails, events, browser captures.
- [ ] P1.16 Tauri bridge: commands for foundation status, workspace registry, settings, event read/write, policy preview.
- [ ] P1.17 Frontend: polished macOS-first app shell with sidebar, toolbar/header, split-view content, no SaaS/cyberpunk styling.
- [ ] P1.18 Frontend: Apple-style design tokens for system font, spacing, materials, shadows, light/dark, empty/error/loading states.
- [ ] P1.19 Frontend: reusable base components: sidebar item, workspace header, cards, badges, empty/blocker states, inspector/panel.
- [ ] P1.20 Frontend integration: render all workspace names from real workspace registry; no hardcoded fake connected states.
- [ ] P1.21 Frontend integration: Today foundation/widgets from real local state or truthful empty/unconfigured states.
- [ ] P1.22 Frontend: settings/status shell for paths, DB, Keychain, redaction, logging, policy, events, integrations.
- [ ] P1.23 Frontend: confirmation UI primitives showing policy reason and required confirmation/review.
- [ ] P1.24 Tests: Rust unit tests for redaction, logging, Keychain status, policy, events, entity links, path creation.
- [ ] P1.25 Tests: SQLite integration tests for migrations, version, repositories, event read/write, entity links.
- [ ] P1.26 Tests: frontend build/smoke checks for registry rendering, settings status, empty states, no fake success copy.
- [ ] P1.27 Manual verification: launch app locally, verify folders/DB/logs/status/settings on macOS.
- [ ] P1.28 Verification: run `npm run verify:local`.
- [ ] P1.29 Review: write `.hermes/reviews/phase-1-secure-foundation/handoff.md`.
- [ ] P1.30 Review: critique loop until `Verdict: APPROVED`.

---

## Phase 2 — First Vertical Slice

Goal: Today → Task → CLI Session → AgentRun → ReviewRecord → Notification → History.

- [ ] P2.01 Planning: convert vertical-slice acceptance into task-level spec and data-flow diagram.
- [ ] P2.02 Planning: define run lifecycle states, task states, review states, notification states, and failure/blocker states.
- [ ] P2.03 Database: tasks table/model with title, detail, status, priority, timestamps, archived/deleted handling.
- [ ] P2.04 Database: AgentRun/session tables with task link, command/profile, cwd, status, duration, exit code, log reference, summary.
- [ ] P2.05 Database: ReviewRecord table/model with manual reviewer stub fields and links to task/run.
- [ ] P2.06 Database: Notification/Inbox model for completion/blocker/attention records.
- [ ] P2.07 Database: History/Event query model optimized for task/run/entity timelines.
- [ ] P2.08 Backend: task create/list/detail/update service with event writing.
- [ ] P2.09 Backend: CLI profile config for at least one safe local command with truthful configured/unconfigured states.
- [ ] P2.10 Backend/native: command/session runner with cwd, streaming stdout/stderr, stdin if needed, cancel/kill, exit code, duration.
- [ ] P2.11 Backend: child process cleanup and failure handling for cancelled/crashed runs.
- [ ] P2.12 Backend: persist redacted raw logs to app-support; SQLite stores metadata/log reference only.
- [ ] P2.13 Backend: AgentRun lifecycle service writes start/progress/completion/failure events.
- [ ] P2.14 Backend: manual ReviewRecord creation service and reviewer-profile placeholder if available.
- [ ] P2.15 Backend: notification creation/query service for Today/Inbox.
- [ ] P2.16 Backend: history query service combining events/entity links without leaking raw logs/secrets.
- [ ] P2.17 Tauri bridge: commands/events for task CRUD.
- [ ] P2.18 Tauri bridge: commands/events for run start/stream/cancel and run status.
- [ ] P2.19 Tauri bridge: commands for review records, notifications, inbox, history.
- [ ] P2.20 Frontend: Today widgets showing real tasks, active runs, blockers, completions, empty states.
- [ ] P2.21 Frontend: task create/list/detail UI with validation and persistence.
- [ ] P2.22 Frontend: linked run/review/history panels inside task detail.
- [ ] P2.23 Frontend: Clean Session UI that streams output as clean cards/status, not raw terminal-first UI.
- [ ] P2.24 Frontend: run controls for start/cancel and clear status/error handling.
- [ ] P2.25 Frontend: manual review stub UI.
- [ ] P2.26 Frontend: notification/Inbox attention card basics.
- [ ] P2.27 Frontend: History view for task/run events.
- [ ] P2.28 Tests: backend tests for task persistence and event writing.
- [ ] P2.29 Tests: backend tests for run lifecycle, cancellation, exit codes, log persistence, redaction.
- [ ] P2.30 Tests: backend tests for review records, notifications, history queries.
- [ ] P2.31 Tests: UI smoke/E2E where feasible for create task → start CLI run → see output → notification/history.
- [ ] P2.32 Manual verification: launch app, create task, run command, restart app, verify persistence.
- [ ] P2.33 Verification: run `npm run verify:local`.
- [ ] P2.34 Review: write `.hermes/reviews/phase-2-first-vertical-slice/handoff.md`.
- [ ] P2.35 Review: critique loop until `Verdict: APPROVED`.

---

## Phase 3 — Notes, Files, and Local Knowledge

- [ ] P3.01 Planning: define Notes/Files scope; explicitly exclude Apple Notes import and full file-manager overreach.
- [ ] P3.02 Database: notes/files/index/entity-link schema and migrations.
- [ ] P3.03 Database: note identity/index metadata with stable frontmatter ID and conflict state.
- [ ] P3.04 Backend: Markdown note create/edit/delete/trash service.
- [ ] P3.05 Backend: frontmatter stable ID writer/reader and scanner/indexer.
- [ ] P3.06 Backend: conflict handling for duplicate IDs, manual renames, and external edits; non-destructive defaults.
- [ ] P3.07 Backend: basic file browse/open/preview service.
- [ ] P3.08 Backend: safe rename/move/copy/trash with action policy confirmation.
- [ ] P3.09 Backend: entity links from notes/files to tasks/products/runs.
- [ ] P3.10 Tauri bridge: commands for notes CRUD, scan/index, conflicts, file browse/open/preview/actions.
- [ ] P3.11 Frontend: Notes workspace list/editor/preview/trash/conflict states.
- [ ] P3.12 Frontend: Files workspace browse/open/preview/actions/confirmation.
- [ ] P3.13 Frontend: history/links panels for notes and files.
- [ ] P3.14 Tests: note persistence after restart.
- [ ] P3.15 Tests: manual file rename preserves note identity.
- [ ] P3.16 Tests: duplicate ID conflict is non-destructive.
- [ ] P3.17 Tests: destructive file operations require confirmation.
- [ ] P3.18 Tests: note/file events and entity links are recorded.
- [ ] P3.19 Manual verification: create/edit/delete/trash note; restart; inspect files on disk.
- [ ] P3.20 Manual verification: browse/open/preview file and perform confirmed safe operation.
- [ ] P3.21 Verification: run `npm run verify:local`.
- [ ] P3.22 Review: write `.hermes/reviews/phase-3-notes-files-knowledge/handoff.md`.
- [ ] P3.23 Review: critique loop until `Verdict: APPROVED`.

---

## Phase 4 — Code, Repos, Launch Gate

- [ ] P4.01 Planning: define scope; avoid overbuilding full GitHub/Vercel automation.
- [ ] P4.02 Database: repo profiles, product/task links, launch gate checks, verification evidence, integration states.
- [ ] P4.03 Backend: repo discovery/manual add service.
- [ ] P4.04 Backend: git status/diff/read operations.
- [ ] P4.05 Backend: git commit/push/merge/deploy action policy; protected actions require confirmation/review.
- [ ] P4.06 Backend integration: GitHub state detection where credentials configured; truthful blocked/unconfigured otherwise.
- [ ] P4.07 Backend integration: Vercel state detection where credentials configured; truthful blocked/unconfigured otherwise.
- [ ] P4.08 Backend: Launch Gate check/evidence model; cannot mark verified without real evidence.
- [ ] P4.09 Backend: production verification evidence records with screenshot/URL/status/manual note where available.
- [ ] P4.10 Tauri bridge: repo add/list/status/diff commands.
- [ ] P4.11 Tauri bridge: launch gate/evidence commands.
- [ ] P4.12 Frontend: Code/Repos UI for repo add, status, diff, linked tasks/products.
- [ ] P4.13 Frontend: GitHub/Vercel blocked/unconfigured/connected/error states; no fake connected data.
- [ ] P4.14 Frontend: Launch Gate UI showing checks, evidence, failures, confirmation requirements.
- [ ] P4.15 Tests: repo add/link/status/diff.
- [ ] P4.16 Tests: Launch Gate evidence requirements and no fake verification.
- [ ] P4.17 Tests: deploy/push/merge confirmation policy.
- [ ] P4.18 Manual verification: add a local repo, inspect status/diff, link to task/product.
- [ ] P4.19 Manual verification: try Launch Gate without evidence and confirm it fails closed.
- [ ] P4.20 Verification: run `npm run verify:local`.
- [ ] P4.21 Review: write `.hermes/reviews/phase-4-code-repos-launch-gate/handoff.md`.
- [ ] P4.22 Review: critique loop until `Verdict: APPROVED`.

---

## Phase 5 — Content and OmniSocials

- [ ] P5.01 Planning: define draft-first/fail-closed publishing scope and specialist design/review gates.
- [ ] P5.02 Database: content plans, pieces, assets, review gates, platform statuses, schedules, verification records.
- [ ] P5.03 Backend: content plan → draft → asset → review → schedule workflow.
- [ ] P5.04 Backend: media asset storage/references and platform media constraints.
- [ ] P5.05 Backend: specialist design/review gate records; no schedule/publish without required review/confirmation.
- [ ] P5.06 Backend integration: OmniSocials account/status detection with truthful states.
- [ ] P5.07 Backend integration: OmniSocials upload/schedule/publish surfaces; fail closed on missing credentials/errors.
- [ ] P5.08 Backend: verification records and failure reports for platform actions.
- [ ] P5.09 Tauri bridge: content CRUD/workflow commands.
- [ ] P5.10 Tauri bridge: OmniSocials status/upload/schedule/publish commands with policy enforcement.
- [ ] P5.11 Frontend: content calendar/workspace with plans, drafts, assets, review gates.
- [ ] P5.12 Frontend: platform constraints, schedule/publish confirmation surfaces, failure reports.
- [ ] P5.13 Tests: workflow progression plan → draft → asset → review → schedule.
- [ ] P5.14 Tests: failed generation/review/upload/schedule fails closed.
- [ ] P5.15 Tests: no publish/schedule without required review/confirmation.
- [ ] P5.16 Tests: platform media constraints enforced.
- [ ] P5.17 Manual verification: create content piece through draft/review/schedule path without publishing by default.
- [ ] P5.18 Verification: run `npm run verify:local`.
- [ ] P5.19 Review: write `.hermes/reviews/phase-5-content-omnisocials/handoff.md`.
- [ ] P5.20 Review: critique loop until `Verdict: APPROVED`.

---

## Phase 6 — Calendar, Gmail, Inbox, Business, Products

- [ ] P6.01 Planning: define Calendar/Gmail/Inbox/Business/Products scope and confirmation/privacy boundaries.
- [ ] P6.02 Database: contacts, companies, follow-ups, products, email/calendar references, cross-entity links, integration statuses.
- [ ] P6.03 Backend/native: EventKit permission/status service.
- [ ] P6.04 Backend/native: EventKit calendar read/create/edit/delete with confirmation requirements.
- [ ] P6.05 Backend integration: Gmail OAuth/status service with truthful states.
- [ ] P6.06 Backend integration: Gmail read/search/draft/send; send always requires confirmation.
- [ ] P6.07 Backend: Inbox aggregation across notifications, emails, calendar, follow-ups, blockers.
- [ ] P6.08 Backend: contacts/companies/follow-ups services.
- [ ] P6.09 Backend: Products workspace services and links to tasks/repos/notes/emails/events.
- [ ] P6.10 Tauri bridge: calendar commands.
- [ ] P6.11 Tauri bridge: Gmail commands.
- [ ] P6.12 Tauri bridge: inbox/business/products commands.
- [ ] P6.13 Frontend: Inbox workspace with attention items, blockers, follow-ups, integration states.
- [ ] P6.14 Frontend: Calendar workspace with read/create/edit/delete confirmation flows.
- [ ] P6.15 Frontend: Gmail surfaces with read/search/draft/send confirmation and safe unconfigured/auth states.
- [ ] P6.16 Frontend: Business workspace for contacts, companies, follow-ups.
- [ ] P6.17 Frontend: Products workspace with cross-links and related history.
- [ ] P6.18 Tests: Gmail send confirmation and no silent send.
- [ ] P6.19 Tests: calendar create/edit/delete confirmation.
- [ ] P6.20 Tests: permission/auth/integration states do not leak internal provider details.
- [ ] P6.21 Tests: cross-link integrity and persistence.
- [ ] P6.22 Manual verification: EventKit native checks or documented blocker.
- [ ] P6.23 Manual verification: Gmail OAuth/native checks or documented blocker.
- [ ] P6.24 Verification: run `npm run verify:local`.
- [ ] P6.25 Review: write `.hermes/reviews/phase-6-calendar-gmail-business-products/handoff.md`.
- [ ] P6.26 Review: critique loop until `Verdict: APPROVED`.

---

## Phase 7 — Browser Workspace and Advanced Widgets

- [ ] P7.01 Planning: review Phase 7 scope against Browser/WebView spike findings.
- [ ] P7.02 Planning: define “work webview/capture workspace” only; exclude full personal browser, extensions, browser sync, password manager, unproven console capture.
- [ ] P7.03 Planning: define Browser flows: open URL, saved page/tab, capture screenshot/fallback metadata, save link, attach to entities, view history.
- [ ] P7.04 Planning: define evidence fields: URL, title, timestamp, screenshot ref, HTTP status if available, manual note, entity links.
- [ ] P7.05 Planning: define widget customization requirements: visibility, order, simple size, persistence, reset.
- [ ] P7.06 Database: browser_tabs model/migration.
- [ ] P7.07 Database: browser_captures model/migration.
- [ ] P7.08 Database: entity links for browser_capture → launch_gate/task/note/product/content_piece.
- [ ] P7.09 Database: events for browser open/update/close, capture created/attached, widget config changed/reset.
- [ ] P7.10 Database: widget configuration persistence keyed by workspace/profile.
- [ ] P7.11 Backend: browser tab/saved-page repository/service.
- [ ] P7.12 Backend/native: WebView integration for opening work URLs within proven Tauri limits.
- [ ] P7.13 Backend/native: tab abstraction or saved-page fallback depending on feasibility.
- [ ] P7.14 Backend/native: screenshot capture path if feasible; files in app support, metadata in SQLite.
- [ ] P7.15 Backend/native: fallback capture path when screenshot unavailable: URL/title/timestamp/status/manual note.
- [ ] P7.16 Backend: HTTP status helper where feasible for verification evidence.
- [ ] P7.17 Backend: attachment service linking browser captures to Launch Gate/Task/Note/Product/ContentPiece.
- [ ] P7.18 Backend: Launch Gate evidence validation using browser capture only when required fields exist.
- [ ] P7.19 Backend: widget configuration service with validation against allowed widgets/sizes.
- [ ] P7.20 Backend/security: ensure no raw cookies/auth headers/tokens/secrets in logs/events/SQLite.
- [ ] P7.21 Tauri bridge: tab/saved-page commands.
- [ ] P7.22 Tauri bridge: capture creation/screenshot/attachment commands.
- [ ] P7.23 Tauri bridge: HTTP status/route smoke verification command where applicable.
- [ ] P7.24 Tauri bridge: widget config read/update/reset commands.
- [ ] P7.25 Frontend: Browser workspace shell with toolbar, URL input, content area, tab/saved-page strip, inspector/captures sidebar.
- [ ] P7.26 Frontend: truthful Browser empty/loading/error/blocked/unsupported states.
- [ ] P7.27 Frontend: tab/saved-page list backed by real data.
- [ ] P7.28 Frontend: capture action UI previewing saved fields and target entity.
- [ ] P7.29 Frontend: capture detail view with screenshot/link/evidence metadata and linked entities.
- [ ] P7.30 Frontend: attachment picker for Launch Gate, Task, Note, Product, ContentPiece.
- [ ] P7.31 Frontend: Launch Gate evidence integration for browser evidence.
- [ ] P7.32 Frontend: widget customization controls: show/hide, reorder, resize, reset, persisted state.
- [ ] P7.33 Frontend/copy: never claim full browser/personal browser capability.
- [ ] P7.34 Tests: browser capture validation, metadata, attachment rules, evidence eligibility.
- [ ] P7.35 Tests: widget validation, ordering, sizing, show/hide, reset defaults.
- [ ] P7.36 Tests: SQLite integration for tabs, captures, links, events, widget config.
- [ ] P7.37 Tests: redaction for URLs/metadata/logs/events; no cookies/auth headers/tokens/secrets.
- [ ] P7.38 Tests: UI smoke for Browser workspace, URL, capture metadata, evidence attachment, widget customization.
- [ ] P7.39 Manual verification: open normal work URL and persist URL/title after restart.
- [ ] P7.40 Manual verification: login-heavy website behavior documented as supported/blocked/partial.
- [ ] P7.41 Manual verification: screenshot capture or fallback capture verified.
- [ ] P7.42 Manual verification: evidence attaches to Launch Gate and renders there.
- [ ] P7.43 Manual verification: widget changes survive restart.
- [ ] P7.44 Accessibility: keyboard and screen-reader labels for URL, tab list, capture, attachment picker, widget controls.
- [ ] P7.45 Performance: saved tabs/captures/widgets do not degrade workspace rendering or bloat SQLite.
- [ ] P7.46 Docs: document Browser capabilities/unsupported features and widget behavior.
- [ ] P7.47 Verification: run `npm run verify:local` and fresh/existing DB migration checks.
- [ ] P7.48 Review: write `.hermes/reviews/phase-7-browser-widgets/handoff.md`.
- [ ] P7.49 Review: critique loop until `Verdict: APPROVED`.

---

## Phase 8 — Packaging, Performance, Accessibility, Hardening

- [ ] P8.01 Planning: write hardening spec covering packaging/signing, notification polish, states, accessibility, performance, log retention, migrations, docs.
- [ ] P8.02 Planning: define release readiness report format: commit, build, tests, manual verification, blockers, command output, critique verdict.
- [ ] P8.03 Planning: define macOS packaging/signing/notarization path, identity, entitlements, sandbox/file/CLI implications.
- [ ] P8.04 Planning: define app-wide hardening matrix for actions, integrations, secrets, migration failures.
- [ ] P8.05 Database/backend: log retention settings and cleanup service.
- [ ] P8.06 Database/backend: migration hardening, failed migration safe state, destructive migration backup.
- [ ] P8.07 Database/backend: performance indexes for events, tasks, runs, notifications, captures, links, history.
- [ ] P8.08 Backend/security: audit action policy enforcement across destructive actions, credentials, send/publish/deploy, calendar, Gmail, files.
- [ ] P8.09 Backend/security: audit redaction coverage for logs, events, prompts, summaries, errors, crash reports, diagnostic UI.
- [ ] P8.10 Backend: structured safe error mapping for native/backend failures.
- [ ] P8.11 Native: configure macOS packaging for Tauri.
- [ ] P8.12 Native: configure signing flow and document required certificates/env vars without committing secrets.
- [ ] P8.13 Native: configure notarization/stapling path if applicable.
- [ ] P8.14 Native: review entitlements for notifications, file access, Keychain, EventKit, WebView, CLI execution.
- [ ] P8.15 Native: polish native notifications and click/open-route behavior where feasible.
- [ ] P8.16 Native: verify packaged app can access `~/Zoid`, app support, Keychain, SQLite, notifications, CLI constraints.
- [ ] P8.17 Tauri bridge: harden command error responses with structured redacted payloads.
- [ ] P8.18 Tauri bridge: audit command allowlist/permissions and remove unused exposed surface.
- [ ] P8.19 Tauri bridge: commands for log retention settings/manual cleanup with confirmation.
- [ ] P8.20 Frontend: app-wide empty/loading/error/blocker state pass.
- [ ] P8.21 Frontend: migration failure safe screen with recovery guidance.
- [ ] P8.22 Frontend: log retention settings UI.
- [ ] P8.23 Frontend: notification/inbox route polish.
- [ ] P8.24 Frontend: Apple-style polish pass across all workspaces.
- [ ] P8.25 Frontend/copy: remove internal/private implementation details from user-facing UI.
- [ ] P8.26 Frontend: release/about/settings surface with safe version/build info.
- [ ] P8.27 Tests: log retention policy and cleanup.
- [ ] P8.28 Tests: migration hardening/failure/destructive backup.
- [ ] P8.29 Tests: redaction regressions across logs/events/errors/summaries/prompts/notifications.
- [ ] P8.30 Tests: policy regression tests for all consequential/destructive actions.
- [ ] P8.31 Tests: integration-state tests prevent fake connected/success data.
- [ ] P8.32 Tests: UI smoke tests for critical routes/workspaces.
- [ ] P8.33 Accessibility: keyboard navigation across workspaces, dialogs, menus, inspectors, confirmations, settings.
- [ ] P8.34 Accessibility: focus management, screen-reader labels, contrast, reduced motion.
- [ ] P8.35 Performance: cold/warm startup and workspace switching measurements.
- [ ] P8.36 Performance: large history/log rendering, DB query/index measurements, memory during Browser/CLI streaming.
- [ ] P8.37 Manual verification: packaged macOS app opens cleanly.
- [ ] P8.38 Manual verification: first launch creates folders; restart preserves DB/settings/log metadata/workspace state.
- [ ] P8.39 Manual verification: secrets remain redacted in logs/events/UI errors/notifications using known test secret strings.
- [ ] P8.40 Manual verification: destructive actions require confirmation after packaging.
- [ ] P8.41 Manual verification: native notification click/open behavior and migration failure safe screen.
- [ ] P8.42 Verification: clean install local verification from empty app data dir.
- [ ] P8.43 Verification: upgrade verification from populated local DB/app-support state.
- [ ] P8.44 Verification: full build/typecheck/test suite.
- [ ] P8.45 Release verification: run `npm run verify:release` only intentionally; inspect packaged app/DMG, bundle metadata, launch binary.
- [ ] P8.46 Release verification: signing/notarization/stapling if configured.
- [ ] P8.47 Release verification: artifact contains no raw secrets/private config/development-only paths.
- [ ] P8.48 Docs: local dev/build/test/package commands.
- [ ] P8.49 Docs: signing/notarization/release process and required non-committed secrets/certificates.
- [ ] P8.50 Docs: app data locations, security model, log retention, migration behavior, unsupported/partial features, manual native checklist.
- [ ] P8.51 Review: security review for secrets/redaction/Keychain/logs/events/Tauri surface/artifact contents.
- [ ] P8.52 Review: UX/accessibility/performance/release readiness reviews.
- [ ] P8.53 Review: write `.hermes/reviews/phase-8-hardening-release-readiness/handoff.md`.
- [ ] P8.54 Review: critique loop until `Verdict: APPROVED`.
