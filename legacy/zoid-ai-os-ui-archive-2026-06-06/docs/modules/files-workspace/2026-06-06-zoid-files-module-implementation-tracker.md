# Zoid Files Module Implementation Tracker

Date: 2026-06-06
Source discovery: `/Users/ziadnasreldin/brainstorms/2026-06-06-zoid-files-module.md`
Related docs: `2026-06-06-zoid-files-module-prd.md`, `2026-06-06-stitch-ai-files-workspace-screens-prompt.md`, `../../designer-screen-reference/09-files-workspace.md`

Purpose: execution tracker for the native in-app Files module. Status legend: `[ ]` pending, `[~]` in progress, `[x]` complete, `[!]` blocked.

Global gates for every phase:
- [ ] No fake file data, fake permission state, fake indexing state, or fake successful file operation.
- [ ] All filesystem actions operate on real local files only after native macOS permission.
- [ ] Deletes route to macOS Trash; no silent permanent delete path in MVP.
- [ ] Agent file access is off by default and requires separate explicit enablement.
- [ ] Every user/agent/system file operation writes an audit-grade event.
- [ ] Eligible text/code/docs/config edits create pre-edit snapshots before mutation.
- [ ] Raw secrets and sensitive folder contents are not exposed to prompts/logs/exports/snapshots unexpectedly.
- [ ] Finder extensions, Finder context menus, and share sheet integration are not included in MVP.
- [ ] Routine verification passes with focused frontend/native tests, `npm run verify:local`, `git diff --check`, packaged Tauri app checks, and feature critique approval.

Likely primary files:
- Backend/native: `src-tauri/src/lib.rs`, new `src-tauri/src/files*.rs`, native macOS permission helpers, filesystem watcher modules, index/snapshot services, migrations under `src-tauri/migrations/00xx_files*.sql`.
- Frontend: `src/App.tsx`, new Files workspace view model/components/tests, shared inspector/activity/settings components.
- Storage: Zoid app data under `~/Library/Application Support/Zoid/` for SQLite/index/snapshots/logs unless existing app conventions differ.
- Review: `.hermes/reviews/files-module/handoff.md`, `.hermes/reviews/files-module/critique-report.md`.

Verification command set:
- Backend/native focused: `cargo test --manifest-path src-tauri/Cargo.toml files -- --nocapture`
- Frontend focused: `npm run test:frontend -- files` or current project-equivalent focused command
- Full local: `npm run verify:local && git diff --check`
- Native/manual: packaged Tauri app permission request, real folder browsing, real file open/reveal, real rename/move/copy/delete-to-Trash, external Finder change sync, index/snapshot/app-support inspection.
- Safety: create temp test folder/files only; clean up temp artifacts after verification.

---

## Phase 0 — PRD, Design Prompt, and Scope Lock

- [ ] F0.01 Save PRD for Files module scope, goals, non-goals, entity model, safety, indexing, agent access, and acceptance criteria.
- [ ] F0.02 Save Stitch AI screen design prompt for Dashboard, Explorer, Detail/Inspector, Settings/Permissions, Activity Log drawer, and states.
- [ ] F0.03 Confirm MVP excludes Finder extensions, Finder context menus, and share sheet integration.
- [ ] F0.04 Define done criteria: real native local files, permissions, indexing, snapshots, activity logs, agent file tools, packaged Tauri verification, and critique approval.
- [ ] F0.05 Create `.hermes/reviews/files-module/` review folder before implementation work starts.
- Likely files: `2026-06-06-zoid-files-module-prd.md`, `2026-06-06-stitch-ai-files-workspace-screens-prompt.md`, this tracker, `.hermes/reviews/files-module/`.
- Gate: scope is stable and no extension/context-menu requirement remains in MVP.

## Phase 1 — Current Codebase Discovery and Architecture Map

- [ ] F1.01 Inspect existing Zoid Tauri commands, SQLite/migration patterns, app data path conventions, and frontend workspace routing.
- [ ] F1.02 Identify existing Agents Workspace, Projects, Content Workspace, Browser Captures, Repos, Tasks, Chats, and entity-link/event models.
- [ ] F1.03 Identify existing test commands and native verification workflows.
- [ ] F1.04 Write architecture notes for Files services: permissions, filesystem bridge, watcher, indexer, snapshotter, activity log, agent tools, UI view model.
- [ ] F1.05 Decide whether to reuse existing event/entity-link tables or add Files-specific tables.
- Likely files: existing `src-tauri` modules, `src/App.tsx`, workspace modules/tests, migrations.
- Gate: implementer can name exact files to modify before schema work begins.

## Phase 2 — Schema, Migrations, and Domain Models

- [ ] F2.01 Add `file_records` or equivalent table for path, kind, size, modified time, permission/watch/index status, repo status, and last seen metadata.
- [ ] F2.02 Add folder/pinned folder model if not covered by `file_records`.
- [ ] F2.03 Add `file_index_entries` for extracted text/metadata status, last indexed time, parser result, excluded reason, stale state.
- [ ] F2.04 Add `file_snapshots` for eligible pre-edit snapshots with source path, actor, operation, timestamp, retention, restore metadata.
- [ ] F2.05 Add `file_activity_events` for actor, operation, path(s), result, source, linked entities, snapshot availability, undo/restore availability.
- [ ] F2.06 Add `file_entity_links` or extend shared entity links to support projects, agent runs, content assets, browser captures, repos, tasks, chats.
- [ ] F2.07 Add `files_settings` for permission state, indexing state, exclusions, retention, storage quota, agent access toggle.
- [ ] F2.08 Add migration tests for idempotence, SQLite reopen, constraints, and no raw secret fields.
- Likely files: `src-tauri/migrations/00xx_files.sql`, backend models/services/tests.
- Gate: migrations pass against clean and existing local DB.

## Phase 3 — Native macOS Permission and Filesystem Bridge

- [ ] F3.01 Implement native permission request flow for local filesystem access using the project’s Tauri/macOS approach.
- [ ] F3.02 Persist truthful permission status and denial/error states.
- [ ] F3.03 Implement `list`, `stat`, path normalization, path display, and path existence checks.
- [ ] F3.04 Implement open-with-default-app and reveal-in-Finder commands.
- [ ] F3.05 Implement basic UI-backed file operations: create file/folder, rename, move, copy, duplicate, delete-to-Trash, copy path.
- [ ] F3.06 Add path validation, symlink handling, race-condition handling, and locked/missing file errors.
- [ ] F3.07 Add tests for denied permission, missing path, file/folder distinction, and error payloads.
- Likely files: native Files service module, Tauri command registration, frontend command bridge.
- Gate: real temp folder can be browsed and operated on from native commands.

## Phase 4 — File Watcher and External Change Sync

- [ ] F4.01 Implement filesystem watcher for accessible/local watched roots.
- [ ] F4.02 Record external create/modify/rename/move/delete events as system/external activity events.
- [ ] F4.03 Update file records and UI state when changes happen outside Zoid/Finder.
- [ ] F4.04 Debounce burst events and avoid duplicate noisy logs.
- [ ] F4.05 Handle unavailable roots, moved folders, permission revoked, and watcher restart.
- [ ] F4.06 Add tests or manual verification notes for external Finder edits.
- Likely files: watcher service, activity event service, frontend subscriptions/state.
- Gate: changing a temp file in Finder/terminal updates Zoid state truthfully.

## Phase 5 — Indexing and Extraction Engine

- [ ] F5.01 Implement local index queue with pause/resume, status, stale markers, and retry/failed extraction state.
- [ ] F5.02 Implement default exclusion engine for system folders, app bundles, caches, `node_modules`, `.git`, build outputs, virtualenvs, vendor dirs, `.ssh`, `.gnupg`, keychains, browser profiles, mail stores, Photos libraries unless opted in, and size limits.
- [ ] F5.03 Extract text from plain text, code, Markdown, PDF, and DOCX.
- [ ] F5.04 Extract metadata from images, videos, and audio.
- [ ] F5.05 Defer OCR and audio/video transcription; represent unsupported extraction truthfully.
- [ ] F5.06 Store index in local app data only; use encryption-at-rest if feasible in current stack.
- [ ] F5.07 Add settings actions to clear index and rebuild/reindex later if exposed.
- [ ] F5.08 Add tests for exclusions, supported extraction, failed extraction, pause/resume, and no cloud sync.
- Likely files: indexer/extractor modules, settings service, SQLite tables, frontend status UI.
- Gate: supported temp files index locally and excluded folders remain excluded.

## Phase 6 — Snapshots, Undo, and Delete-to-Trash Safety

- [ ] F6.01 Implement eligibility detection for text/code/docs/config-like files under size/type rules.
- [ ] F6.02 Create pre-edit snapshot before `write_text` and `patch_text` agent operations and any eligible UI edit path.
- [ ] F6.03 Store snapshots separately in local app data with retention/storage metadata.
- [ ] F6.04 Mark non-snapshotted operations visibly in activity log.
- [ ] F6.05 Implement delete-to-Trash and verify no normal permanent delete path exists in MVP.
- [ ] F6.06 Implement undo where feasible for move, rename, create, and restore from snapshot for eligible edits.
- [ ] F6.07 Add cleanup/retention controls and destructive confirmation.
- [ ] F6.08 Add tests/manual verification for snapshot create, skipped snapshot, restore, delete-to-Trash, and undo metadata.
- Likely files: snapshot service, trash/native helper, activity log service, settings UI.
- Gate: editing a temp text file creates a restorable snapshot before mutation.

## Phase 7 — Activity Log and Entity Linking

- [ ] F7.01 Implement audit-grade activity event writer shared by UI operations, agent tools, watcher/external changes, indexer, and system events.
- [ ] F7.02 Include actor, operation, path(s), timestamp, linked entity, result, source, snapshot availability, undo/restore action.
- [ ] F7.03 Implement filters for actor, operation, result, source, linked entity, date, and path.
- [ ] F7.04 Implement Activity Log dashboard panel plus filterable drawer/modal.
- [ ] F7.05 Implement file/folder links to projects, agent runs, content assets, browser captures, repos, tasks, and chats using shared link model where possible.
- [ ] F7.06 Implement deterministic automatic links: files created by agent run link to that run/project; repo files link to repo; known browser captures/downloads link when source is known.
- [ ] F7.07 Implement suggested links where uncertain if feasible, or leave clear follow-up tasks.
- [ ] F7.08 Add tests for event creation and deterministic links.
- Likely files: activity/link services, frontend log drawer/inspector, tests.
- Gate: every meaningful operation produces a visible log event.

## Phase 8 — Agent File Tool Surface

- [ ] F8.01 Add global “Allow Zoid agents to operate on files” setting, default off.
- [ ] F8.02 Ensure macOS file permission and agent file-access toggle are separate states.
- [ ] F8.03 Implement agent tools: `list`, `read`, `search`, `create_file`, `create_folder`, `write_text`, `patch_text`, `rename`, `move`, `copy`, `delete_to_trash`, `open`, `reveal`, `stat`.
- [ ] F8.04 Block arbitrary binary writes in MVP; allow binary copy/move/delete only if implemented safely by native file operation path.
- [ ] F8.05 Enforce snapshots before eligible write/patch.
- [ ] F8.06 Log every agent operation with linked agent run/project where available.
- [ ] F8.07 Return structured errors for permission denied, toggle off, excluded path, missing file, locked file, unsupported binary edit, snapshot failure, and trash failure.
- [ ] F8.08 Add tests for toggle off/on, logging, snapshots, and blocked binary write.
- Likely files: agent tool registry/bridge, Files command service, activity/snapshot services, tests.
- Gate: agents cannot use file tools until the separate toggle is enabled.

## Phase 9 — Files Dashboard UI

- [ ] F9.01 Implement workspace route/nav entry for Files.
- [ ] F9.02 Build Files Dashboard header: search files, add/open folder or permission CTA, new folder, settings.
- [ ] F9.03 Build permission/indexing health panel with truthful states.
- [ ] F9.04 Build summary cards: pinned folders, Zoid folder, recent files, linked files, agent-touched files, index health.
- [ ] F9.05 Build folder shortcuts, recent files, content assets, browser captures, repo folders, file events.
- [ ] F9.06 Build right inspector for selected file/folder/event.
- [ ] F9.07 Build empty, permission-denied, indexing-paused, indexing-failed, and healthy states.
- [ ] F9.08 Add frontend tests and browser smoke verification.
- Likely files: Files workspace components/view model/tests, CSS/design tokens.
- Gate: dashboard renders truthful state with no fake connected/files/indexed data.

## Phase 10 — File Explorer UI

- [ ] F10.01 Build Explorer path bar with back, forward, up, breadcrumbs, search, new folder, more.
- [ ] F10.02 Build left folder tree/sidebar.
- [ ] F10.03 Build main file list with name, kind, size, modified, linked entity, repo status, index/snapshot/agent badges.
- [ ] F10.04 Add sortable table and optional grid toggle if feasible.
- [ ] F10.05 Build preview pane for supported file types and truthful unsupported states.
- [ ] F10.06 Build right inspector with metadata, linked entities, events, snapshot/restore state, path actions.
- [ ] F10.07 Wire basic Finder actions to native commands.
- [ ] F10.08 Add keyboard navigation/multi-select only if it does not expand MVP risk; otherwise document follow-up.
- [ ] F10.09 Add frontend tests and native manual verification.
- Likely files: Explorer components/view model/tests, command bridge.
- Gate: Explorer can browse and operate on real temp files in packaged app.

## Phase 11 — Detail/Inspector, Settings, and Permission States

- [ ] F11.01 Build File/Folder Detail or robust inspector state with metadata, preview, links, repo status, events, actions.
- [ ] F11.02 Build Files Settings/Permissions screen.
- [ ] F11.03 Add permission status and re-request access UI.
- [ ] F11.04 Add indexing status, pause/resume, exclusions, clear index.
- [ ] F11.05 Add snapshot retention/storage limit, clear snapshots, activity log retention.
- [ ] F11.06 Add local-only privacy note and encryption/secure-storage status if available.
- [ ] F11.07 Add global agent file-access toggle with clear warning copy and default off.
- [ ] F11.08 Add confirmation dialogs for destructive settings actions.
- [ ] F11.09 Add tests for settings state changes and confirmations.
- Likely files: settings components/view model/tests, settings backend service.
- Gate: user can understand and control local index/snapshot/agent access state.

## Phase 12 — Tests, Native Verification, and Critique Approval

- [ ] F12.01 Backend/native tests: schema, permission state, filesystem commands, watcher, indexer, exclusions, snapshots, activity log, agent tools.
- [ ] F12.02 Frontend tests: dashboard, explorer, detail/inspector, settings, activity drawer, permission-denied, indexing, empty/error states.
- [ ] F12.03 Safety tests: delete-to-Trash, no permanent delete MVP path, snapshot before eligible mutation, toggle-off agent block, raw secret redaction assumptions.
- [ ] F12.04 Local native E2E: packaged Tauri app against a temp folder; verify permission, browse, open/reveal, basic actions, external change sync, index, snapshot, activity log, agent toggle/tools.
- [ ] F12.05 Routine full verification: `npm run verify:local && git diff --check`.
- [ ] F12.06 Clean up temp files/folders and verification artifacts after E2E.
- [ ] F12.07 Write handoff at `.hermes/reviews/files-module/handoff.md` with exact verification results and known limitations.
- [ ] F12.08 Run feature critique loop until `.hermes/reviews/files-module/critique-report.md` says `Verdict: APPROVED`.
- Gate: feature cannot be called done until tests, packaged native verification, and critique approval are all present.
