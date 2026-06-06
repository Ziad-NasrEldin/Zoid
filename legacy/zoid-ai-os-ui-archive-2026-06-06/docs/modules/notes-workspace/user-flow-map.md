# Zoid Notes Workspace User Flow Map — Full Product Scope

Date: 2026-06-06
Module: Notes Workspace
Related docs: `prd.md`, `implementation-tracker.md`, `/Users/ziadnasreldin/Zoid/Docs/designer-screen-reference/04-notes-workspace.md`
Source discovery: `/Users/ziadnasreldin/brainstorms/2026-06-06-zoid-notes-workspace.md`

Important: this is the initial gap-register map. It has been superseded for final flow design by `user-flow-map-designed.md`, which applies the latest decisions including Apple Notes import/one-way refresh only and no Apple Notes write-back.

## 1. Screen Inventory Covered

Desktop Zoid screens:

1. Notes Dashboard
2. All Notes
3. Note Detail / Editor
4. Imported Apple Notes
5. Collections Management
6. Tags Management
7. Notes Settings
8. Notes Activity / History

Mobile companion surfaces:

9. Mobile Capture
10. Mobile Notes List / Search
11. Mobile Note Read / Light Edit
12. Mobile Sync Status

System/modal states included:

- First-run vault setup
- New note modal/route
- New collection flow
- Link entity flow
- Agent operation flow
- Delete/trash/restore flow
- Conflict resolution flow
- Search unavailable/fallback state
- iCloud/local sync error state
- Apple Notes permission/import/sync error state

## 2. Primary Entry Points

### 2.1 User opens Notes Workspace from Zoid sidebar

Flow:

1. User clicks `Notes` in Zoid navigation.
2. System checks whether a Notes vault exists and is reachable.
3. Branch:
   - If no vault: show First-Run Vault Setup.
   - If vault exists and reachable: open Notes Dashboard.
   - If vault exists but missing/unreachable: show vault error state with Settings/reconnect actions.
   - If vault exists but sync is pending/error: open Dashboard with sync warning banner.

GAP 1 — Vault setup choices are not fully specified.
- We know default is Zoid-managed app-data vault, with optional external folder and iCloud/local synced vault.
- We do not yet know the exact first-run choices shown to the user.
- Needed decision: Should first-run show only “Use default local vault” + later Settings options, or should it immediately offer Default / External Folder / iCloud Synced Vault?

Recommended resolution:
- First-run should offer three clear choices: Default Local Vault, Choose Folder, Use iCloud/Local Synced Folder. Default highlighted.

### 2.2 User creates/captures from another Zoid module

Possible source modules:

- Agents Workspace
- Files Workspace
- Browser captures
- Content workspace
- Code/repos
- Tasks/calendar
- Chats
- Projects/products/business entities

Flow:

1. User chooses `Create Note` or `Save to Notes` from source module.
2. System creates note with deterministic source link.
3. System selects destination collection.
4. Branch:
   - If source has obvious project/entity context: prefill linked entity and suggested collection.
   - If collection unknown: place in Inbox / Unorganized.
   - If agent creates note autonomously: require Notes agent toggle enabled.
5. User lands in Note Detail / Editor or sees a confirmation toast linking to note.

GAP 2 — Source-module entry points are not enumerated per module.
- PRD says notes can be created from many modules, but not which exact buttons/actions exist per source.
- Needed decision: For each module, is the action `Create Note`, `Save to Notes`, `Attach Note`, `Summarize into Note`, or all of these?

Recommended resolution:
- Use consistent action names:
  - `Create linked note` for blank/manual note from context.
  - `Save to Notes` for captured content/file/browser/chat snippets.
  - `Summarize into Note` for generated summaries.

### 2.3 User opens note from global search or linked inspector

Flow:

1. User searches globally or opens linked notes in another module inspector.
2. User selects a note result.
3. Notes opens Note Detail / Editor with source context preserved.
4. Right inspector shows linked entity that brought the user in.

GAP 3 — Global Zoid search behavior is not defined.
- Notes search is defined, but global search routing is not.
- Needed decision: Does global Zoid search include notes by default, and should it open Notes Detail or preview in the source module inspector?

Recommended resolution:
- Global search includes notes by default and opens Note Detail. Source module inspector can preview but not become the main editing surface.

## 3. First-Run Vault Setup Flow

Trigger:

- User opens Notes for the first time.
- User has no Notes vault configured.

Flow:

1. Show first-run state on Notes Dashboard shell.
2. Explain local-first Notes storage.
3. User chooses vault mode.
4. Branch:
   - Default local vault: create Zoid app-data vault.
   - External folder: open folder picker and request permissions.
   - iCloud/local synced folder: open folder picker targeting iCloud Drive/local synced folder.
5. System initializes structured storage, metadata DB, export/snapshot folders, attachments, trash/archive, index state.
6. System creates default collections: Inbox / Unorganized and optionally General.
7. User lands on empty Notes Dashboard.

GAP 4 — Default collections beyond Inbox are unknown.
- We know Inbox / Unorganized is required.
- We do not know whether to create General, Work, Projects, Imported, or none.

Recommended resolution:
- Create only Inbox / Unorganized and General by default. Avoid fake taxonomy.

GAP 5 — External vault permissions are not specified.
- Files module has a permission model. Notes needs similar permission rules for external vaults.
- Needed decision: Does Notes reuse Files Workspace permission prompts or have its own?

Recommended resolution:
- Use a Notes-specific vault permission prompt but rely on shared native file-access infrastructure.

## 4. Notes Dashboard Flow

Purpose:

- Knowledge operations dashboard.
- Shows what needs attention, not just recent notes.

Main components:

- Header: Search Notes, New Note, New Collection, Import Apple Notes, Settings
- Summary cards: Recent, Unlinked, Imported, Needs Organization
- Recent notes list
- Collections list
- Tags list
- Linked entity notes
- Recent note history
- Collections/tags health
- Suggested organization actions
- Right inspector

### 4.1 Open Dashboard

Flow:

1. User opens Notes.
2. System loads vault status, sync status, index status, note counts, recent notes, unorganized notes, import status, activity summary.
3. Dashboard displays primary attention cards.
4. User can:
   - Search notes.
   - Create a note.
   - Create a collection.
   - Import/sync Apple Notes.
   - Open Settings.
   - Open All Notes.
   - Open Collections/Tags management.
   - Open Activity/History.
   - Select note/collection/tag to inspect.

### 4.2 Dashboard summary card branches

Recent:

1. User clicks Recent card/list item.
2. Opens filtered All Notes or Note Detail.

Unlinked:

1. User clicks Unlinked card.
2. Opens All Notes filtered to notes with no entity links/backlinks.
3. User can link manually or accept suggestions.

Imported:

1. User clicks Imported card.
2. Opens Imported Apple Notes screen or All Notes filtered by source = Apple Notes.

Needs Organization:

1. User clicks Needs Organization.
2. Opens All Notes filtered to Inbox / Unorganized and/or missing tags/links.
3. User bulk moves/tags/links notes.

GAP 6 — “Needs organization” scoring is undefined.
- Is a note unorganized if it is in Inbox only, missing tags, missing entity links, no summary, imported but unreviewed, or all of those?

Recommended resolution:
- Needs organization if any of these are true: in Inbox / Unorganized, imported-unreviewed, agent-created-unreviewed, no collection beyond Inbox, suggested links pending, or duplicate/empty title.

### 4.3 Dashboard right inspector

When user selects:

- Note: show summary, source, links, tags, latest activity, quick actions.
- Collection: show counts, health, recent notes, actions.
- Tag: show counts, aliases, related tags, health, actions.
- Activity: show event details and restore/version if available.

GAP 7 — Right inspector action density is not defined.
- We know inspector exists, but not which actions are safe inline vs full-screen.

Recommended resolution:
- Inspector supports quick metadata/link actions. Destructive actions, conflict resolution, and complex bulk changes should open full screen/modals.

## 5. All Notes Flow

Purpose:

- Primary list/search/filter/bulk-operation surface.

Components:

- Search/filter/sort toolbar
- Notes list: title, collection, workspace/source, tags, updated, linked entities, summary
- Note preview inspector

Actions:

- Open
- Preview
- Link
- Summarize
- Move
- Tag
- Delete
- Bulk organize

### 5.1 Browse/search/filter

Flow:

1. User opens All Notes from Dashboard/sidebar.
2. System shows default sort by updated date.
3. User searches or applies filters.
4. Branch:
   - Text search available: show matching results.
   - Semantic search available: allow semantic mode/toggle.
   - Semantic unavailable: show fallback notice and text results.
   - Index rebuilding: show partial results with warning.
5. User selects a note for preview or opens editor.

GAP 8 — Search mode UI is unresolved.
- Should semantic search be a toggle, command chip, separate tab, or automatic blend with text search?

Recommended resolution:
- Search bar supports mode chips: `Text`, `Semantic`, `Hybrid`. If semantic unavailable, chip is disabled with explanation.

### 5.2 Bulk organize

Flow:

1. User filters/selects notes.
2. User chooses bulk action: Move, Tag, Untag, Link, Summarize, Delete.
3. Branch:
   - Safe metadata change: confirm if many notes, then apply.
   - Agent-assisted organize: require agent toggle if autonomous edits are applied.
   - Delete: move to Trash/Archive, not permanent delete.
4. Activity log records operation.

GAP 9 — Bulk operation confirmation threshold is undefined.
- We need a threshold for when bulk actions require confirmation.

Recommended resolution:
- Confirm bulk changes when affecting more than one note; stronger confirmation when affecting 10+ notes or delete/archive.

### 5.3 Preview inspector

Flow:

1. User selects note row.
2. Inspector shows title, summary, tags, collection, source, linked entities, backlinks, latest activity, quick actions.
3. User can open full editor or perform simple safe metadata edits.

GAP 10 — Whether preview supports inline editing is unclear.

Recommended resolution:
- Allow lightweight metadata edits in preview; body/block edits require Note Detail / Editor.

## 6. New Note Flow

Entry points:

- Dashboard `New Note`
- All Notes `New Note`
- Source module `Create linked note`
- Mobile quick capture
- Agent-created note

Flow:

1. User clicks New Note or source creates note.
2. System creates draft note.
3. Determine collection:
   - If user selected a collection: use it.
   - If source context maps to a collection: suggest/preselect it.
   - Otherwise use Inbox / Unorganized.
4. Determine source/entity links:
   - If created from source module: add deterministic link.
   - If manual: no entity link unless user adds it.
5. Open Note Detail / Editor.
6. Autosave starts after first meaningful edit.
7. Activity log records creation.

GAP 11 — Draft note persistence timing is unclear.
- Should blank drafts exist immediately, or only after first content/title edit?

Recommended resolution:
- Create local draft immediately but hide/delete empty untouched drafts after close unless user typed title/content or source context exists.

## 7. Note Detail / Editor Flow

Purpose:

- Main writing/editing/inspection surface.

Components:

- Header: title, save status, file/export path, actions
- Metadata row: source/workspace, collection, tags, updated, sync state
- Block editor
- Canvas blocks
- Optional Markdown preview/export pane
- Linked entities
- Backlinks
- Note history
- Right inspector

### 7.1 Open note

Flow:

1. User opens note from Dashboard, All Notes, link, source module, or mobile sync.
2. System loads structured note, metadata, blocks, canvas, attachments, links, versions, activity, sync state.
3. Branch:
   - Clean note: editable state.
   - Sync conflict: conflict banner visible.
   - Read-only vault/source: read-only state with explanation.
   - Missing attachment/export: warning state.

GAP 12 — Read-only rules are undefined.
- External folders/iCloud can become read-only, and Apple Notes-synced notes may have source constraints.

Recommended resolution:
- Read-only if vault permission is missing, file lock/error exists, or source sync adapter reports write-back unavailable. User can duplicate into editable local note.

### 7.2 Editing blocks

Flow:

1. User edits title/metadata/blocks.
2. System autosaves structured content.
3. System updates save status.
4. System updates metadata/index.
5. System creates revisions based on retention/version policy.
6. System updates Markdown export/snapshot asynchronously where enabled.

GAP 13 — Autosave revision granularity is undefined.
- Every keystroke is too much; manual-only is too weak.

Recommended resolution:
- Autosave continuously, but create version checkpoints by time interval, after meaningful block operations, before agent/sync changes, and on manual named checkpoints.

### 7.3 Add canvas block

Flow:

1. User uses slash command or insert menu.
2. User selects Canvas / Whiteboard block.
3. Canvas opens inline or expanded.
4. User adds nodes/shapes/connectors/images/entity cards.
5. Canvas persists structured data and indexed text.
6. Markdown export includes snapshot/link/summary, not lossy full content.

GAP 14 — Canvas interaction depth on desktop is not fully specified.
- We know pan/zoom, nodes, shapes, connectors, entity cards, images, snapshots.
- We do not know whether multi-canvas per note is allowed.

Recommended resolution:
- Allow multiple canvas blocks per note, each with title/caption. Simpler than forcing one canvas per note.

### 7.4 Link entity

Flow:

1. User clicks Link Entity.
2. System opens entity picker.
3. User searches/selects entity type and record.
4. Link appears in inspector and note metadata.
5. Activity log records link creation.

GAP 15 — Entity picker taxonomy and availability are unresolved.
- Product lists entity types, but current data models may not exist for all.

Recommended resolution:
- Entity picker should be plugin/module-aware: only enabled entity types with data are active; unavailable types are hidden or disabled.

### 7.5 Summarize / Organize / Create Task

Flow:

1. User invokes action from Note Detail.
2. Branch:
   - Summarize selected/current note: allowed because user invoked it.
   - Organize metadata only as suggestion: allowed.
   - Apply edits autonomously: requires agent toggle.
   - Create task: opens task draft with note link.
3. Changes create note versions and activity entries.

GAP 16 — Agent draft-vs-apply UX is not fully defined.
- We know autonomous operations require toggle, but not how suggestions are reviewed.

Recommended resolution:
- Default agent output appears as a review card with Apply / Edit / Dismiss. If autonomous toggle is enabled, agent may apply low-risk metadata changes but body edits still show diff unless user opts into direct apply.

### 7.6 Delete note from editor

Flow:

1. User clicks Delete.
2. System confirms move to Trash/Archive.
3. Note leaves active lists and appears in Trash/Archive filter.
4. History retained based on retention policy.
5. Activity log records delete.
6. User can restore from Trash/Archive or Activity.

GAP 17 — Trash vs Archive terminology is unresolved.
- Prior language uses “Trash/Archive” interchangeably.

Recommended resolution:
- Use `Trash` for deleted notes and `Archive` for intentionally hidden but retained notes. These should be separate states/actions.

## 8. Imported Apple Notes Flow

Purpose:

- Import and sync Apple Notes.

Components:

- Import controls
- Sync controls/status
- Import status panel
- Imported notes list
- Mapping / organization panel
- Import history
- Failed/retry state
- Conflict state

### 8.1 First Apple Notes import

Flow:

1. User opens Imported Apple Notes screen.
2. System shows permission/status panel.
3. User grants Apple Notes access.
4. System scans available notes/folders/metadata/attachments where feasible.
5. User starts import.
6. System imports title/body/dates/folders/attachments where feasible.
7. Folders map to collections.
8. Unmapped notes go to Inbox / Unorganized.
9. Import history records results.
10. User opens imported note or organizes imported batch.

GAP 18 — Apple Notes permission UX is platform-dependent.
- Need implementation spike to know exact macOS permission steps and what can be automated.

Recommended resolution:
- Product UI should show a clear “Connect Apple Notes” step, but implementation must document exact platform-specific permission limitations.

### 8.2 Apple Notes one-way refresh

Flow:

1. User refreshes imported Apple Notes, or system monitors source changes where platform access allows.
2. System maps Apple Notes records to imported Zoid notes.
3. Refresh loop checks Apple-source changes only.
4. Branch:
   - Apple source changed, Zoid copy unchanged: update Zoid copy and version previous copy.
   - Apple source changed, Zoid copy edited: create imported-source update version and show compare/merge banner.
   - Source unavailable: mark source unavailable; do not delete Zoid note.
5. Activity log records import/refresh events.
6. Zoid never mutates Apple Notes.

RESOLVED 19 — Apple Notes write-back removed.
- Latest decision: do not include Apple Notes write-back.
- Zoid imports and refreshes from Apple Notes only.
- Zoid deletions/edits never mutate Apple Notes.

RESOLVED 20 — Apple Notes delete behavior.
- Deleting/trashing in Zoid affects only the Zoid copy.
- Missing/deleted Apple source marks source unavailable; it does not delete the Zoid copy automatically.

### 8.3 Conflict resolution

Flow:

1. Conflict banner appears in Note Detail and Imported Apple Notes screen.
2. User opens conflict resolver.
3. User sees Zoid version and Apple Notes version.
4. User chooses:
   - Keep Zoid.
   - Keep Apple Notes.
   - Manual merge.
5. System stores resolved version and keeps previous versions in history.
6. Sync resumes.

GAP 21 — Conflict resolver UI detail is missing.
- Side-by-side diff? Block-level merge? Plain text fallback?

Recommended resolution:
- Start with side-by-side diff and Keep Zoid / Keep Apple / Create merged copy. Block-level merge can come after plain conflict safety works.

## 9. Collections Management Flow

Purpose:

- Maintain primary containers/folders.

Flow:

1. User opens Collections Management from Dashboard/sidebar/Settings.
2. System lists collections with counts, unorganized notes, linked entities, health warnings.
3. User can create, rename, archive, delete, set default, bulk move notes.
4. Branch:
   - Rename: update notes and activity.
   - Archive: hide collection from default views; notes remain searchable.
   - Delete empty collection: confirm and delete.
   - Delete non-empty collection: require move destination or archive.
   - Bulk move: confirm when multiple notes affected.

GAP 22 — Collection delete semantics are undefined.
- What happens to notes in a deleted collection?

Recommended resolution:
- Non-empty collection cannot be deleted until notes are moved; offer Move to another collection or Move to Inbox.

GAP 23 — Collection hierarchy is undefined.
- Are nested collections/folders supported?

Recommended resolution:
- Support flat collections initially unless user explicitly wants hierarchy. If Apple Notes folders are nested, preserve path as collection name/path metadata.

## 10. Tags Management Flow

Purpose:

- Maintain cross-cutting labels.

Flow:

1. User opens Tags Management.
2. System lists tags with counts, aliases, duplicate/unused warnings, linked entities.
3. User can create, rename, archive, delete, merge, alias, bulk tag/untag notes.
4. Branch:
   - Merge tags: show affected note count and confirmation.
   - Delete tag: remove tag from notes, do not delete notes.
   - Archive tag: hide from default suggestions but preserve historical metadata.
   - Add alias: alias resolves to canonical tag.
5. Activity logs changes.

GAP 24 — Tag casing/normalization rules are undefined.
- Example: `AI`, `ai`, `A.I.` could duplicate.

Recommended resolution:
- Tags have display name and normalized key. Normalize case/spacing for duplicate detection while preserving display label.

## 11. Notes Settings Flow

Purpose:

- Control vault, sync, import, retention, search, semantic, agents, privacy, recovery.

Sections:

1. Vault location/status
2. External folder/iCloud setup
3. Apple Notes import/sync
4. History retention/storage limit
5. Trash retention
6. Search/index rebuild
7. Semantic search availability/index
8. Agent note-operations toggle
9. Default collection
10. Privacy/local-only explanation
11. Export/recovery

### 11.1 Change vault location

Flow:

1. User opens Settings > Vault.
2. User chooses change/move vault.
3. System explains migration/sync risks.
4. User selects destination.
5. System validates permissions, copies/moves/indexes data.
6. Branch:
   - Success: update vault and status.
   - Permission failure: keep old vault and show error.
   - Conflict/existing vault: ask whether to open existing vault or migrate into it.

GAP 25 — Vault migration/open-existing behavior is unresolved.

Recommended resolution:
- Separate actions: `Move current vault` vs `Open existing vault`. Never merge automatically without review.

### 11.2 Agent toggle

Flow:

1. User opens Settings > Agent note operations.
2. Toggle is off by default.
3. User turns on.
4. System shows warning explaining create/edit/organize actions are logged and versioned.
5. User confirms.
6. Agents can perform configured operations.
7. User can turn off anytime.

GAP 26 — Per-agent/session permissions are not defined.
- Prior decision: global toggle now, per-agent/session later? For Notes, the full product could need both.

Recommended resolution:
- Product should include global master toggle plus per-agent/session controls when Agents Workspace supports them.

### 11.3 Search/index rebuild

Flow:

1. User opens Settings > Search.
2. Sees index status and semantic availability.
3. User can rebuild text index and semantic index.
4. System shows progress and degraded search state.

GAP 27 — Index rebuild background behavior is undefined.

Recommended resolution:
- Rebuild runs in background, search remains available with stale/partial marker until complete.

## 12. Notes Activity / History Flow

Purpose:

- User-facing audit stream.

Flow:

1. User opens Activity / History.
2. System lists activity events.
3. User filters by actor, note, operation, source, result, linked entity, date.
4. User selects event.
5. Inspector shows details and available actions.
6. Branch:
   - Version exists: view/restore/diff.
   - Trash event: restore note if retained.
   - Sync/import error: open relevant resolver/settings.
   - Agent event: open agent run/session.

GAP 28 — Activity retention rules are not specified.

Recommended resolution:
- Keep activity metadata longer than full content versions. Let Settings control version retention/storage, but keep lightweight audit entries unless user clears them.

## 13. Agent Operation Flow

### 13.1 User-invoked summarize

Flow:

1. User selects note and clicks Summarize.
2. System sends selected note context to local/allowed agent path.
3. Summary appears as suggestion.
4. User applies to note summary/metadata or dismisses.
5. Activity records operation.

GAP 29 — Remote model policy for agent note content is unresolved.
- Semantic search says do not send full notes remote by default, but agent summarization may need model context.

Recommended resolution:
- Treat agent note content separately from search: local/default if available; remote model use requires clear provider/privacy setting or per-action confirmation.

### 13.2 Autonomous organize

Flow:

1. Agent wants to organize notes without direct user click.
2. System checks Notes agent toggle.
3. Branch:
   - Toggle off: block and ask user to enable/review.
   - Toggle on: agent creates proposed changes or applies allowed metadata changes.
4. System versions changed notes and logs activity.

GAP 30 — Autonomy levels for agent note changes are not granular enough.

Recommended resolution:
- Add levels: Suggest only, Apply metadata, Append only, Patch body with diff, Full edit. Default should be Suggest only or Apply metadata.

## 14. Delete / Trash / Restore / Permanent Delete Flow

### 14.1 Soft delete

Flow:

1. User deletes note.
2. System asks confirm move to Trash.
3. Note becomes trashed.
4. Note hidden from default lists.
5. History retained.
6. Activity recorded.

### 14.2 Restore

Flow:

1. User opens Trash filter or Activity event.
2. User selects Restore.
3. System restores note to prior collection if collection exists.
4. If collection missing/archived, restore to Inbox / Unorganized and warn user.

GAP 31 — Trash screen location is not specified.
- Is Trash part of All Notes filter, Settings, Activity, or its own screen?

Recommended resolution:
- Include Trash as an All Notes system filter and Settings retention section, not a separate primary screen.

### 14.3 Permanent delete

Flow:

1. User selects Permanently Delete from Trash.
2. System shows explicit destructive confirmation.
3. User confirms.
4. System deletes structured note and retention-eligible versions/attachments according to policy.
5. Activity may retain lightweight deletion metadata if allowed.

GAP 32 — Attachment deletion retention is undefined.

Recommended resolution:
- Attachments tied only to a permanently deleted note are deleted after retention window unless also referenced elsewhere.

## 15. iCloud / Local Folder Sync Flow

### 15.1 Desktop synced vault setup

Flow:

1. User opens Settings > Vault/Sync.
2. User selects iCloud Drive/local synced folder.
3. System validates folder permissions and vault layout.
4. Branch:
   - Empty folder: initialize vault.
   - Existing Zoid vault: open existing vault.
   - Unknown folder contents: warn and require explicit choice.
5. System tracks sync status and file conflicts.

GAP 33 — Sync provider assumptions are unclear.
- User chose iCloud/local folder sync, but “local folder sync” could mean Dropbox/Google Drive/Syncthing/any folder.

Recommended resolution:
- Treat provider as generic filesystem sync folder. iCloud-specific UX only where app container/iOS requires it.

### 15.2 Desktop sync conflict

Flow:

1. System detects external file/storage conflict.
2. System marks note/vault conflict.
3. User sees banner and Activity entry.
4. User resolves through conflict resolver or recovery tools.

GAP 34 — Structured storage conflict format needs definition.
- Conflict resolution depends on whether structured notes are SQLite-only, files, JSON docs, or hybrid.

Recommended resolution:
- Use per-note structured files plus SQLite index/cache if sync folder is source; avoid syncing a single live SQLite DB across iCloud as primary source.

## 16. Mobile Companion Flow

### 16.1 Mobile first launch

Flow:

1. User opens iOS companion.
2. App requests access to iCloud Drive/app container synced vault.
3. Branch:
   - Vault found: open Mobile Notes List.
   - No vault: show setup instructions to enable desktop synced vault.
   - Permission missing: show permission help.
4. App builds/offline-caches lightweight index.

GAP 35 — Mobile setup without desktop is undefined.
- Can mobile create the first vault, or must desktop create it?

Recommended resolution:
- Desktop creates primary vault first. Mobile can create capture-only local holding area if no vault, then merge once synced vault exists.

### 16.2 Mobile capture

Flow:

1. User taps Quick Note / Voice / Photo.
2. Creates note in Inbox / Unorganized.
3. Captures text/audio/photo attachment.
4. Saves offline if needed.
5. Syncs when available.

GAP 36 — Voice capture processing is undefined.
- Is voice stored as audio only, transcribed locally, or transcribed remotely?

Recommended resolution:
- Store audio attachment first; transcribe locally where available; remote transcription requires explicit setting/confirmation.

### 16.3 Mobile read/search/light edit

Flow:

1. User opens mobile list/search.
2. Searches local/offline index.
3. Opens note.
4. Can read, edit title/simple text/basic blocks, change tags/collection lightly.
5. Syncs back through iCloud/local synced vault.

GAP 37 — Mobile block support boundary is undefined.
- “Basic block editing” needs a precise list.

Recommended resolution:
- Mobile can edit text, headings, bullets, checklists, quotes, simple attachments. It can view but not deeply edit canvas/table/entity complex blocks.

## 17. End-to-End Core User Journeys

### Journey A — First use to first note

1. Open Notes.
2. Choose/create vault.
3. Land on empty Dashboard.
4. Click New Note.
5. Type title/content.
6. Add collection/tags.
7. Save/autosave.
8. Note appears in Dashboard and All Notes.
9. Activity logs creation.

Blocking gaps: GAP 1, GAP 4, GAP 11.

### Journey B — Import Apple Notes and organize

1. Open Imported Apple Notes.
2. Connect Apple Notes.
3. Start import.
4. Review import results.
5. Open Imported/Needs Organization list.
6. Bulk move/tag notes.
7. Resolve failed imports.
8. Imported notes appear in collections and search.

Blocking gaps: GAP 18, GAP 20.

### Journey C — Work note linked to an agent run

1. Open Agents Workspace run.
2. Click Create linked note or Summarize into Note.
3. Note created with run/project link.
4. User edits in Note Detail.
5. Agent suggests summary/tags/tasks.
6. User applies suggestions or enables broader agent operations.
7. Activity shows user/agent actions.

Blocking gaps: GAP 2, GAP 16, GAP 29, GAP 30.

### Journey D — Knowledge cleanup session

1. Open Dashboard.
2. Click Needs Organization.
3. Review unorganized/imported/agent-created notes.
4. Bulk tag/move/link.
5. Merge duplicate tags.
6. Resolve unlinked notes.
7. Dashboard health improves.

Blocking gaps: GAP 6, GAP 9, GAP 24.

### Journey E — Imported Apple Notes source update after Zoid edit

1. Edit imported note's source in Apple Notes.
2. Edit the mapped Zoid copy.
3. Refresh detects source update.
4. Note Detail shows imported-source update banner.
5. User opens compare/merge.
6. User keeps Zoid, accepts imported source into Zoid, or creates merged Zoid version.
7. Apple Notes remains unchanged by Zoid.

Blocking gaps: GAP 21 only in this initial map; final design resolves it as side-by-side compare/merge.

### Journey F — Desktop/mobile synced capture

1. Desktop vault configured in iCloud/local synced folder.
2. Mobile app opens synced vault.
3. User captures voice/photo/quick note on mobile.
4. Mobile stores in Inbox / Unorganized offline if needed.
5. Sync brings note to desktop.
6. Desktop Dashboard shows Needs Organization.
7. User organizes and links note.

Blocking gaps: GAP 33, GAP 35, GAP 36, GAP 37.

## 18. Gap Register — Decisions Needed Before Final UI/Implementation

Critical blockers:

1. First-run vault choices: default-only vs full vault mode selection.
2. Apple Notes import/refresh behavior without write-back.
3. Apple Notes platform feasibility and permission UX.
4. Structured storage sync format for iCloud/local folder sync.
5. Agent remote/local model privacy policy for note content.
6. Agent autonomy levels beyond one global toggle.
7. Mobile setup dependency: desktop-created vault required or mobile can create first vault.
8. Mobile basic block editing boundary.
9. Conflict resolver depth: side-by-side diff vs block-level merge.
10. Trash vs Archive separation.

Important UX/product gaps:

11. Default collections beyond Inbox / Unorganized.
12. Exact source-module entry actions.
13. Global Zoid search routing into Notes.
14. Needs Organization scoring.
15. Right inspector inline actions vs full-screen actions.
16. Search mode UI: text/semantic/hybrid.
17. Bulk operation confirmation threshold.
18. Preview inspector inline editing boundary.
19. Draft note persistence timing.
20. Multi-canvas-per-note support.
21. Entity picker taxonomy and disabled unavailable entities.
22. Collection delete semantics.
23. Nested collection support.
24. Tag normalization/casing/alias rules.
25. Vault migration vs open-existing behavior.
26. Index rebuild background/stale-state behavior.
27. Activity retention vs version retention.
28. Attachment retention on permanent delete.
29. Sync provider assumptions beyond iCloud.
30. Voice capture transcription policy.

## 19. Recommended Next Step

Before screen design or implementation, resolve the critical blockers with a short decision pass. The highest-risk decisions are:

1. Apple Notes import/refresh platform feasibility.
2. iCloud/local synced structured storage format.
3. Agent content privacy/model routing.
4. Mobile first-run/setup model.
5. Agent autonomy levels.

These are resolved in `user-flow-map-designed.md` using best-judgment product defaults, but still require implementation validation.

Do not let UI design pretend these are solved. They directly change screens, warnings, settings, empty states, and implementation architecture.
