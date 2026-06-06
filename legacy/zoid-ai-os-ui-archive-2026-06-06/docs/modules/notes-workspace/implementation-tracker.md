# Zoid Notes Workspace Implementation Tracker

Date: 2026-06-06
Source discovery: `/Users/ziadnasreldin/brainstorms/2026-06-06-zoid-notes-workspace.md`
Related docs: `prd.md`, `/Users/ziadnasreldin/Zoid/Docs/designer-screen-reference/04-notes-workspace.md`

Purpose: execution tracker for the full Notes Workspace product scope. Status legend: `[ ]` pending, `[~]` in progress, `[x]` complete, `[!]` blocked.

Important: this is not an MVP or V1 tracker. Build Order sequences implementation safely, but the PRD defines the full product scope. No Stitch AI prompt should be generated for this request.

## Global Gates

- [ ] Do not call Notes complete until desktop Zoid works natively in Tauri, not only browser preview.
- [ ] Do not allow autonomous agent note operations unless the off-by-default Notes agent toggle is enabled.
- [ ] Do not write Zoid changes back into Apple Notes; Apple Notes is import/one-way refresh only.
- [ ] Do not permanently delete notes by default; delete routes to Trash/Archive first.
- [ ] Do not send full notes to remote models by default for semantic search.
- [ ] Do not include sharing/collaboration/public publishing; user removed it from scope.
- [ ] Document Apple Notes/iCloud platform limitations honestly after implementation spikes.
- [ ] Feature critique workflow must run before implementation is considered complete: create `.hermes/reviews/notes-workspace/handoff.md`, receive critique report, fix Required items, and re-review until approved.

## Likely Primary Files / Areas to Inspect

Adjust after repo inspection:

- Existing reference: `Docs/designer-screen-reference/04-notes-workspace.md`
- Module docs: `Docs/modules/notes-workspace/prd.md`
- Frontend workspace routes/components: inspect current `src/` structure.
- Native/Tauri commands: inspect `src-tauri/src/`.
- Persistence/migrations: inspect current app database layer.
- Files Workspace integration: inspect files module surfaces and permission model.
- Agents Workspace integration: inspect agent session/run and permission models.
- Search/indexing: inspect current local search/embedding capabilities, if any.
- iOS/mobile codebase decision: inspect whether one exists or create separately.

## Verification Command Set

Adjust after repo inspection:

- [ ] Frontend focused tests: `npm run test -- notes-workspace` or project equivalent.
- [ ] Native/backend tests: `cargo test --manifest-path src-tauri/Cargo.toml notes_workspace -- --nocapture` or project equivalent.
- [ ] Full local verification: `npm run verify:local && git diff --check` or current equivalent.
- [ ] Native manual: `npm run tauri:dev`, exercise vault, editor, search, Apple Notes import/refresh, activity, settings.
- [ ] iCloud/local sync manual: two device/folder simulation where feasible.
- [ ] Apple Notes spike/manual: verify import/one-way refresh feasibility on macOS with permissions.
- [ ] iOS companion verification: capture/read/search/light edit/offline/sync with real synced vault.

---

## Build Order 1 — Product Shell + Domain Model

- [ ] N1.01 Inspect current Zoid app structure, Notes placeholder/routes, persistence, Tauri command conventions, and shared UI patterns.
- [ ] N1.02 Define domain models: Note, NoteBlock, CanvasBlock, Collection, Tag, Backlink, EntityLink, Attachment, NoteVersion, NoteActivity, Vault, SyncState, Conflict, ImportRun, AgentNotePermission.
- [ ] N1.03 Define storage strategy: structured local source of truth for block/canvas notes, Markdown export/snapshot for portability, SQLite metadata/index.
- [ ] N1.04 Add migrations/tables/indexes for notes, blocks, canvases, collections, tags, note_tags, backlinks, entity_links, attachments, versions, activity, trash/archive, vaults, sync state, import runs, conflicts.
- [ ] N1.05 Create Notes Workspace route shell with left navigation, main panel, and right inspector pattern.
- [ ] N1.06 Add empty first-run state for creating/selecting a vault.
- [ ] Gate: native app opens Notes Workspace, initializes local vault/database, and persists minimal notes domain records after restart.

## Build Order 2 — Vaults, Files, and Markdown Portability

- [ ] N2.01 Implement Zoid-managed default notes vault inside app data.
- [ ] N2.02 Implement optional external user-selected vault/folder support.
- [ ] N2.03 Implement vault status model: local, external, iCloud/local synced, missing, read-only, error.
- [ ] N2.04 Implement structured note serialization.
- [ ] N2.05 Implement Markdown export/snapshot for simple and block notes.
- [ ] N2.06 Implement clean `.md` save path for simple Markdown-compatible notes where possible.
- [ ] N2.07 Implement attachments folder/storage and attachment metadata.
- [ ] N2.08 Integrate with Files Workspace to reveal/open Markdown exports/snapshots and attachments where safe.
- [ ] Gate: create a note, export/reveal Markdown snapshot, attach a file, restart app, and verify note/attachment/export integrity.

## Build Order 3 — Notes Dashboard + All Notes

- [ ] N3.01 Implement Notes Dashboard header: Search Notes, New Note, New Collection, Import Apple Notes, Settings.
- [ ] N3.02 Implement summary cards: recent, unlinked, imported, needs organization.
- [ ] N3.03 Implement recent notes list.
- [ ] N3.04 Implement collections list and tags list.
- [ ] N3.05 Implement linked entity notes panel.
- [ ] N3.06 Implement recent note history panel.
- [ ] N3.07 Implement suggested organization actions.
- [ ] N3.08 Implement right inspector for selected note/collection/tag/activity.
- [ ] N3.09 Implement All Notes list with title, collection, source/workspace, tags, updated, linked entities, summary.
- [ ] N3.10 Implement search/filter/sort toolbar and preview inspector.
- [ ] Gate: dashboard/all-notes reflect real persisted notes, filters work, and selected item inspector updates.

## Build Order 4 — Collections, Tags, Inbox, and Organization

- [ ] N4.01 Implement one primary collection per note.
- [ ] N4.02 Implement many tags per note.
- [ ] N4.03 Create default Inbox / Unorganized collection.
- [ ] N4.04 Route quick captures, imported notes needing review, and unclear agent notes into Inbox / Unorganized.
- [ ] N4.05 Implement Collections Management screen: create, rename, archive, delete, counts, unorganized notes, linked entities, health warnings.
- [ ] N4.06 Implement Tags Management screen: create, rename, archive, delete, merge tags, aliases, counts, linked entities, unused/duplicate warnings.
- [ ] N4.07 Implement bulk move/tag/untag operations.
- [ ] N4.08 Implement dashboard health warnings for duplicate/unused tags and unorganized notes.
- [ ] Gate: collection/tag CRUD, merge, aliases, bulk actions, and unorganized workflow persist and are reflected across Dashboard, All Notes, and Editor.

## Build Order 5 — Note Detail / Block Editor

- [ ] N5.01 Implement Note Detail / Editor header: title, save status, file/export path, actions.
- [ ] N5.02 Implement metadata row: source/workspace, collection, tags, updated, sync state.
- [ ] N5.03 Implement block editor base: headings, paragraphs, checklists, bullets, numbered lists, quotes, code blocks, tables.
- [ ] N5.04 Implement images/attachments blocks.
- [ ] N5.05 Implement embedded/linked entity cards.
- [ ] N5.06 Implement slash commands.
- [ ] N5.07 Implement drag/reorder blocks.
- [ ] N5.08 Implement Markdown shortcuts and Markdown export/preview pane.
- [ ] N5.09 Implement autosave and explicit save status.
- [ ] N5.10 Implement duplicate/delete/open/reveal actions.
- [ ] Gate: block note can be created, edited, reordered, autosaved, exported, reopened after restart, and shown in All Notes/Dashboard.

## Build Order 6 — Canvas / Whiteboard Blocks

- [ ] N6.01 Define canvas block data model and persistence.
- [ ] N6.02 Implement canvas block inside Note Detail.
- [ ] N6.03 Add text nodes/cards.
- [ ] N6.04 Add basic shapes/connectors.
- [ ] N6.05 Add linked note/entity cards.
- [ ] N6.06 Add image/attachment placement.
- [ ] N6.07 Add pan/zoom.
- [ ] N6.08 Add canvas export/snapshot.
- [ ] Gate: note with canvas block persists, reopens, exports/snapshots, and remains searchable by canvas text.

## Build Order 7 — Backlinks + Full Zoid Entity Linking

- [ ] N7.01 Implement note-to-note backlinks.
- [ ] N7.02 Implement entity links for projects, agent sessions/runs, files/folders, content/assets, browser captures, repos, tasks, chats, products/business entities, and calendar events.
- [ ] N7.03 Implement manual link/unlink in right inspector.
- [ ] N7.04 Implement deterministic automatic links for notes created from known source entities.
- [ ] N7.05 Implement uncertain link suggestions.
- [ ] N7.06 Surface linked notes in relevant module inspectors where integration points exist.
- [ ] Gate: create notes from multiple source modules and verify links/backlinks appear correctly with manual and automatic linking.

## Build Order 8 — History, Restore, Trash, and Activity Log

- [ ] N8.01 Implement autosave revisions.
- [ ] N8.02 Implement manual save/version checkpoints where applicable.
- [ ] N8.03 Implement restore previous version.
- [ ] N8.04 Implement diff view where feasible.
- [ ] N8.05 Track actor/source: user, agent, import, sync, system.
- [ ] N8.06 Implement soft delete to Trash/Archive.
- [ ] N8.07 Implement restore from Trash/Archive.
- [ ] N8.08 Implement permanent delete with explicit confirmation.
- [ ] N8.09 Implement user-facing Notes Activity / History screen with filters.
- [ ] Gate: edits by user/import/agent/sync create history/activity rows, restore works, trash/restore works, permanent delete requires confirmation.

## Build Order 9 — Local Structured Search

- [ ] N9.01 Implement fast local search over title/body/blocks.
- [ ] N9.02 Index tags, collections, linked entities, imported source, summaries, backlinks.
- [ ] N9.03 Extract/index canvas text.
- [ ] N9.04 Extract/index attachment text where OCR/text extraction is available.
- [ ] N9.05 Add filters for collection, tag, source, entity type, dates, actor, attachments, conflicts, needs organization, trash/archive.
- [ ] N9.06 Add search index rebuild control in Settings.
- [ ] Gate: search and filters return correct notes from structured content, metadata, links, canvas text, and available attachment text.

## Build Order 10 — Local Semantic / Vector Search

- [ ] N10.01 Inspect available local embedding/vector options in the Zoid stack.
- [ ] N10.02 Implement local embedding/index pipeline where available.
- [ ] N10.03 Include title, body, blocks, canvas text, OCR-able attachment text, imported metadata, tags, and links.
- [ ] N10.04 Add semantic search availability status.
- [ ] N10.05 Fall back to text search when local embeddings are unavailable.
- [ ] N10.06 Ensure full notes are not sent to remote models by default.
- [ ] Gate: semantic search works locally where available; otherwise UI clearly marks semantic unavailable and text fallback works.

## Build Order 11 — Apple Notes Import

- [ ] N11.01 Spike Apple Notes access/import options and document platform limits.
- [ ] N11.02 Implement permission/onboarding flow for Apple Notes access.
- [ ] N11.03 Import title/body.
- [ ] N11.04 Import created/updated dates where available.
- [ ] N11.05 Map Apple Notes folders to collections.
- [ ] N11.06 Copy attachments where feasible.
- [ ] N11.07 Implement import status panel.
- [ ] N11.08 Implement failed import retry/cancel/history.
- [ ] N11.09 Route unmapped/import-needs-review notes to Inbox / Unorganized.
- [ ] Gate: import real Apple Notes sample set with dates/folders/attachments where feasible and verify no unintended Apple Notes mutation.

## Build Order 12 — Apple Notes One-Way Refresh / Imported Update Handling

- [ ] N12.01 Spike feasible Apple Notes monitoring/refresh mechanisms and document hard constraints.
- [ ] N12.02 Implement source identity mapping between Apple Notes and Zoid imported notes.
- [ ] N12.03 Detect Apple Notes source changes where platform access allows.
- [ ] N12.04 Refresh imported Zoid copies from Apple Notes without writing back to Apple Notes.
- [ ] N12.05 Implement import/refresh state/status/errors.
- [ ] N12.06 If the Zoid copy was not edited, apply source refresh automatically into Zoid.
- [ ] N12.07 If both Apple source and Zoid copy changed, create imported-source update versions instead of silent overwrite.
- [ ] N12.08 Implement compare/merge banner/actions: keep Zoid, accept Apple-source refresh into Zoid, create merged Zoid version.
- [ ] Gate: Apple Notes one-way refresh works within documented platform limits and never mutates Apple Notes.

## Build Order 13 — Agent Note Operations

- [ ] N13.01 Implement selected-note read/summarize action invoked by user.
- [ ] N13.02 Add off-by-default Notes setting: Allow agents to create/edit/organize notes.
- [ ] N13.03 Implement agent create note.
- [ ] N13.04 Implement agent summarize note.
- [ ] N13.05 Implement agent organize collections/tags.
- [ ] N13.06 Implement agent suggest links.
- [ ] N13.07 Implement agent create task from note.
- [ ] N13.08 Implement agent append/patch/draft edits.
- [ ] N13.09 Log every agent note operation in activity stream.
- [ ] N13.10 Back every agent write with note version history.
- [ ] Gate: with toggle off, autonomous operations are blocked; with toggle on, operations work, log, version, and can be restored.

## Build Order 14 — iCloud / Local Folder Sync

- [ ] N14.01 Define supported local/iCloud synced vault layout.
- [ ] N14.02 Implement vault mode switch/setup for iCloud Drive/local folder sync.
- [ ] N14.03 Implement sync status and error detection.
- [ ] N14.04 Implement offline-first read/write behavior.
- [ ] N14.05 Implement file/folder conflict detection for synced vault.
- [ ] N14.06 Implement recovery/export tools.
- [ ] N14.07 Test sync behavior with multiple local copies/devices where feasible.
- [ ] Gate: desktop Zoid can use synced vault path, detect sync state/errors, and recover from local conflict scenarios.

## Build Order 15 — iOS Companion App

- [ ] N15.01 Decide iOS companion architecture and repo/package location.
- [ ] N15.02 Implement iCloud Drive/app-container synced vault access where possible.
- [ ] N15.03 Implement quick note capture.
- [ ] N15.04 Implement voice capture attachment.
- [ ] N15.05 Implement photo attachment capture.
- [ ] N15.06 Implement read/search.
- [ ] N15.07 Implement collections/tags/links viewing.
- [ ] N15.08 Implement basic block editing.
- [ ] N15.09 Implement offline cache.
- [ ] N15.10 Implement sync status/error display.
- [ ] Gate: iOS companion captures, reads/searches, lightly edits, works offline, and syncs with desktop through iCloud/local synced vault.

## Build Order 16 — Settings, Privacy, and Operations

- [ ] N16.01 Implement Notes Settings shell.
- [ ] N16.02 Add vault location/status/external folder/iCloud setup controls.
- [ ] N16.03 Add Apple Notes import/refresh settings and history.
- [ ] N16.04 Add history retention/storage limit controls.
- [ ] N16.05 Add trash retention controls.
- [ ] N16.06 Add search/index rebuild controls.
- [ ] N16.07 Add semantic search availability/index controls.
- [ ] N16.08 Add agent note-operations toggle.
- [ ] N16.09 Add default collection setting.
- [ ] N16.10 Add privacy/local-only explanation and export/recovery options.
- [ ] Gate: settings accurately control runtime behavior and persist after restart.

## Build Order 17 — Cross-Module Integration + Native QA

- [ ] N17.01 Verify Files Workspace reveal/open exports and attachments.
- [ ] N17.02 Verify Agents Workspace session/run links and agent-created notes.
- [ ] N17.03 Verify Tasks/Calendar task creation and links.
- [ ] N17.04 Verify Projects/entity links and inspector surfaces.
- [ ] N17.05 Verify Content/Browser/Code/Chats source links where modules exist.
- [ ] N17.06 Run frontend/unit tests.
- [ ] N17.07 Run native/Tauri tests.
- [ ] N17.08 Run manual native app E2E for all core flows.
- [ ] N17.09 Run iCloud/local sync manual scenario.
- [ ] N17.10 Run iOS companion manual scenario.
- [ ] N17.11 Prepare `.hermes/reviews/notes-workspace/handoff.md`.
- [ ] N17.12 Run feature critique workflow and fix Required issues until APPROVED.
- [ ] Gate: full Notes product scope passes native verification and critique approval.

## Completion Checklist

- [ ] PRD scope implemented without MVP/V1 reduction.
- [ ] Structured source-of-truth storage works.
- [ ] Markdown export/snapshot portability works.
- [ ] Vault/external/iCloud/local sync modes work.
- [ ] Dashboard, All Notes, Editor, Imported Apple Notes, Collections, Tags, Settings, Activity screens work.
- [ ] Block editor works.
- [ ] Canvas blocks work.
- [ ] Collections/tags/backlinks/entity links work.
- [ ] Apple Notes import works.
- [ ] Apple Notes import/one-way refresh works within documented limits and never mutates Apple Notes.
- [ ] Imported-source update compare/merge works.
- [ ] History/restore/trash/activity work.
- [ ] Local structured search works.
- [ ] Local semantic search or honest fallback works.
- [ ] Agent note operations work behind toggle with logs/history.
- [ ] iOS companion works with iCloud/local synced vault.
- [ ] Collaboration/sharing absent.
- [ ] Native Tauri verification complete.
- [ ] Feature critique report verdict APPROVED.
