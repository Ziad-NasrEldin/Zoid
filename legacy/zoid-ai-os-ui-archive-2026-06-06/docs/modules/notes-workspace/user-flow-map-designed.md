# Zoid Notes Workspace — Designed User Flow Map

Date: 2026-06-06
Module: Notes Workspace
Related docs: `prd.md`, `implementation-tracker.md`, `user-flow-map.md`

This is the resolved designed flow map continued from the gap-heavy map. It applies the user's latest decisions:

- Do not include Apple Notes bidirectional sync.
- Apple Notes is import + one-way source refresh into Zoid only.
- Zoid never writes changes back to Apple Notes.
- iCloud/local sync uses per-note structured files plus SQLite index/cache, not one live synced SQLite database.
- Agent permissions, mobile boundaries, vault setup, search behavior, trash/archive, and remaining UX choices use best-judgment defaults.

## 0. Product Flow Principles

1. Notes is a full product, not MVP/V1.
2. Desktop Zoid is the primary command center.
3. Mobile is capture + read/search + lightweight edit.
4. Files/Finder integration is for vault visibility, exports, attachments, and external folder permissions.
5. Apple Notes is treated as an external source, not a two-way peer.
6. iCloud/local folder sync is filesystem-based and local-first.
7. Agents can help, but write actions are gated, logged, and versioned.
8. No destructive action should be silent.
9. Every major screen has an attention state, normal state, empty state, and error state.

## 1. Final Screen Map

Desktop screens:

1. Notes Dashboard
2. All Notes
3. Note Detail / Editor
4. Imported Apple Notes
5. Collections Management
6. Tags Management
7. Notes Settings
8. Notes Activity / History

Mobile companion screens:

9. Mobile First Launch / Vault Connect
10. Mobile Capture
11. Mobile Notes List / Search
12. Mobile Note Read / Light Edit
13. Mobile Sync Status

Modals / drawers / system states:

- First-run vault setup
- Folder permission prompt
- New note draft
- New collection
- Entity picker
- Agent suggestion review card
- Agent permissions warning
- Delete to Trash confirmation
- Permanent delete confirmation
- Version restore confirmation
- Imported Apple Notes compare/merge banner
- Sync conflict resolver for iCloud/local folder conflicts
- Search fallback notice
- Index rebuild progress

## 2. Navigation Architecture

Main Notes left navigation:

- Dashboard
- All Notes
- Inbox / Unorganized
- Collections
- Tags
- Imported Apple Notes
- Trash
- Activity
- Settings

Global entry points outside Notes:

- Global Zoid search result -> Note Detail
- Files Workspace file/folder -> Create linked note / Save to Notes
- Agents Workspace run -> Create linked note / Summarize into Note
- Tasks/Calendar item -> Create linked note
- Browser/content/code/chat module -> Save to Notes / Create linked note

Right inspector is shared across Notes screens:

- Selected note inspector
- Selected collection inspector
- Selected tag inspector
- Selected activity event inspector
- Selected import/refresh issue inspector

## 3. First-Run Vault Setup Flow

### Screen: First-Run Vault Setup

Entry:

- User opens Notes for the first time.
- No vault is configured.

Layout:

- Hero: “Set up your Notes vault”
- Explanation: local-first, Finder-visible exports, optional iCloud/local sync.
- Three vault cards:
  1. Default Local Vault — recommended
  2. Choose Folder
  3. Use iCloud / Synced Folder
- Footer actions: Continue, Learn more, Cancel

Flow:

1. User opens Notes.
2. System detects no vault.
3. User sees First-Run Vault Setup.
4. User selects vault mode.
5. Branch:
   - Default Local Vault:
     - Create app-data vault.
     - Initialize per-note structured storage, SQLite index/cache, exports, attachments, trash, versions.
   - Choose Folder:
     - Open folder picker.
     - Show Notes-specific permission prompt backed by shared Files native permissions.
     - Validate folder.
     - Initialize vault layout.
   - Use iCloud / Synced Folder:
     - Open folder picker with iCloud Drive guidance.
     - Validate folder and warn that provider sync behavior may vary.
     - Initialize per-note structured files and SQLite cache.
6. Create default collections:
   - Inbox / Unorganized
   - General
7. Land on Notes Dashboard empty state.

Error branches:

- Permission denied -> show folder permission recovery instructions.
- Existing Zoid Notes vault found -> offer Open Existing Vault or Cancel.
- Unknown folder contents -> require explicit confirm before initializing.
- Read-only folder -> reject as active vault, allow read-only open later if needed.

## 4. Notes Dashboard Flow

### Screen: Notes Dashboard

Purpose:

- Operations home for knowledge, attention, and shortcuts.

Layout:

Header:

- Search Notes
- New Note
- New Collection
- Import Apple Notes
- Settings

Top status strip:

- Vault status
- iCloud/local sync status
- Apple Notes import/refresh status
- Search index status
- Agent note-ops state

Summary cards:

- Recent
- Inbox / Unorganized
- Unlinked
- Imported Updates
- Needs Review
- Search/Index Health

Main panels:

- Recent Notes
- Collections
- Tags
- Linked Entity Notes
- Suggested Organization
- Recent Activity

Right inspector:

- Shows selected note/collection/tag/activity/import issue.

### Dashboard load flow

1. User enters Notes.
2. System loads vault, index, sync, import/refresh, activity, collections, tags.
3. Branch:
   - No notes -> empty dashboard with New Note and Import Apple Notes CTAs.
   - Notes exist -> dashboard populated.
   - Vault warning -> warning strip with Settings action.
   - Index rebuilding -> partial-results strip.
   - Agent toggle off -> small “Agents read by request only” indicator.
4. User selects a card/list item/action.

### Dashboard card flows

Recent:

1. Click note row.
2. Opens Note Detail.
3. Right inspector preserves source context.

Inbox / Unorganized:

1. Opens All Notes filtered to Inbox / Unorganized.
2. User bulk organizes.

Unlinked:

1. Opens All Notes filtered to no entity links.
2. User links manually or accepts suggestions.

Imported Updates:

1. Opens Imported Apple Notes filtered to source updates/conflicts.
2. User reviews imported-source update compare/merge.

Needs Review:

Includes:

- Inbox / Unorganized
- Imported-unreviewed
- Agent-created-unreviewed
- Suggested links pending
- Duplicate/empty title
- Missing collection beyond Inbox

Flow:

1. User opens Needs Review queue.
2. Items grouped by reason.
3. User resolves one-by-one or bulk resolves.

Search/Index Health:

1. User opens Settings > Search or starts rebuild.
2. Dashboard shows rebuild progress until complete.

## 5. All Notes Flow

### Screen: All Notes

Purpose:

- Browse, search, filter, inspect, and bulk-organize notes.

Layout:

Top toolbar:

- Search input
- Mode chips: Text, Semantic, Hybrid
- Filters: Collection, Tag, Source, Linked Entity, Date, Actor, Attachments, Needs Review, Trash/Archive
- Sort: Updated, Created, Title, Collection, Source
- New Note
- Bulk actions

Main list columns:

- Checkbox
- Title
- Collection
- Tags
- Source
- Linked entities
- Updated
- Summary/status

Right preview inspector:

- Summary
- Collection/tags
- Source
- Linked entities/backlinks
- Latest activity
- Quick actions

### Browse/search flow

1. User opens All Notes.
2. Default view shows active notes sorted by updated desc.
3. User types search.
4. Branch:
   - Text mode -> exact/local text results.
   - Semantic mode -> local semantic results if index available.
   - Hybrid mode -> text + semantic ranking.
   - Semantic unavailable -> disabled chip and text fallback notice.
5. User applies filters.
6. User selects note for preview or opens Note Detail.

### Bulk action flow

1. User selects one or more notes.
2. Bulk bar appears.
3. Actions:
   - Move collection
   - Add/remove tags
   - Link entity
   - Summarize
   - Mark reviewed
   - Archive
   - Move to Trash
4. Confirmation rules:
   - 1 note metadata changes: no confirmation except delete/archive.
   - 2-9 notes: confirm bulk action.
   - 10+ notes: stronger confirmation with affected count.
   - Delete/archive: always confirm.
5. System applies action.
6. Activity logs bulk operation.

### Preview inspector flow

1. User selects a note row.
2. Inspector opens.
3. Allowed inline edits:
   - Collection
   - Tags
   - Entity links
   - Mark reviewed
4. Body/block edits open Note Detail.
5. Destructive actions open confirmation modal.

## 6. New Note Flow

### Entry points

- Dashboard New Note
- All Notes New Note
- Collection-specific New Note
- Source module Create linked note
- Save to Notes from file/content/browser/chat
- Mobile quick capture
- Agent-created note

### Flow

1. User or system triggers note creation.
2. System creates a draft note record.
3. Destination collection resolution:
   - If launched from collection -> use selected collection.
   - If source context maps to collection -> preselect suggested collection.
   - Otherwise -> Inbox / Unorganized.
4. Source/entity link resolution:
   - Source module creates deterministic entity link.
   - Manual note has no entity link until user adds one.
5. Open Note Detail / Editor.
6. Draft persistence rule:
   - Draft exists immediately.
   - Empty untouched draft is auto-cleaned on close.
   - Draft with title/content/source link is kept.
7. Activity records creation once draft becomes meaningful.

## 7. Note Detail / Editor Flow

### Screen: Note Detail / Editor

Layout:

Header:

- Back / breadcrumb
- Editable title
- Save/autosave status
- Source/import/sync state
- Open/reveal export
- More menu

Metadata row:

- Collection picker
- Tags
- Source
- Linked entities count
- Updated timestamp
- Review state

Main editor:

- Block editor
- Slash commands
- Canvas blocks
- Attachments
- Optional Markdown export/preview pane

Right inspector tabs:

- Summary
- Links
- Backlinks
- History
- Activity
- Agent suggestions
- Import/source update

### Open note flow

1. User opens note from any entry point.
2. System loads structured note, metadata, blocks, canvas, attachments, links, versions, activity, source state.
3. Branch:
   - Normal editable -> editor active.
   - Read-only vault -> editor locked with Duplicate to Local action.
   - Missing attachment/export -> warning in inspector.
   - Imported-source update exists -> compare/merge banner.
   - iCloud/local file conflict -> conflict banner.

Read-only rule:

- Read-only if vault permission is missing, folder is locked/read-only, file conflict blocks safe write, or source adapter cannot provide editable local copy.
- User can duplicate into editable local note.

### Editing flow

1. User edits title, metadata, or blocks.
2. Autosave updates structured note files.
3. SQLite index/cache updates asynchronously.
4. Version checkpoints are created:
   - Before agent edits
   - Before imported-source updates
   - Before conflict merges
   - On meaningful block operations
   - On timed intervals
   - On user-created named checkpoint
5. Markdown export/snapshot updates asynchronously.
6. Activity logs meaningful operations.

### Block editor flow

Supported blocks:

- Paragraph
- Heading
- Checklist
- Bullets
- Numbered list
- Quote
- Code block
- Table
- Image
- Attachment
- Linked entity card
- Canvas block

Flow:

1. User types or invokes slash command.
2. Chooses block.
3. Edits/reorders blocks.
4. System saves structured content.
5. Search index receives block text.

### Canvas block flow

1. User inserts Canvas block.
2. Canvas opens inline with expand option.
3. User adds text nodes, cards, shapes, connectors, images, linked entity cards.
4. Multiple canvas blocks per note are allowed.
5. Canvas text is indexed.
6. Markdown export includes snapshot, title/caption, and summary/link to structured canvas data.

### Entity linking flow

1. User clicks Link Entity.
2. Entity picker opens.
3. Picker shows only currently available module entity types.
4. Disabled/unavailable entity types are hidden unless debug/admin mode is enabled.
5. User searches/selects entity.
6. Link appears in metadata and inspector.
7. Backlink appears in linked module where integration exists.
8. Activity logs link.

### Agent suggestion flow

1. User invokes Summarize, Organize, Create Task, Suggest Links, Append, Patch, or Draft Edit.
2. Agent output appears as a review card.
3. User chooses Apply, Edit, Dismiss.
4. If body edit/patch is proposed, show diff before apply.
5. If metadata-only suggestion and agent setting allows Apply Metadata, it may apply automatically.
6. All agent writes create version checkpoint and activity log.

Agent autonomy levels:

1. Suggest only — default.
2. Apply metadata — allowed after explicit setting.
3. Append only — allowed after explicit setting.
4. Patch body with diff — requires review unless explicitly configured.
5. Full edit — highest-risk, separate confirmation/setting.

Model/privacy rule:

- Local/default path first where possible.
- Remote model use for note content requires a clear provider/privacy setting or per-action confirmation.

### Delete/archive flow from editor

Actions are separate:

- Archive: hide from default active views but keep searchable and restorable.
- Move to Trash: deletion intent; hidden from default active views.
- Permanently Delete: only from Trash, explicit destructive confirmation.

Delete to Trash flow:

1. User clicks Delete.
2. Modal confirms Move to Trash.
3. Note moves to Trash.
4. Activity logs action.
5. Note can be restored.

Archive flow:

1. User clicks Archive.
2. Note becomes archived.
3. Activity logs action.
4. Note remains searchable with Archive filter.

## 8. Imported Apple Notes Flow

### Screen: Imported Apple Notes

Purpose:

- Import Apple Notes and monitor source-side updates into Zoid without writing back to Apple Notes.

Layout:

Header:

- Connect Apple Notes
- Start Import
- Refresh Imported Notes
- Import History
- Settings shortcut

Status strip:

- Permission state
- Last import
- Last refresh
- Failed items
- Source updates pending

Tabs:

1. Import Setup
2. Imported Notes
3. Source Updates
4. Failed / Retry
5. History

### First import flow

1. User opens Imported Apple Notes.
2. User clicks Connect Apple Notes.
3. System guides platform permission flow.
4. System scans available notes/folders/metadata/attachments where feasible.
5. User clicks Start Import.
6. System imports title/body/dates/folders/attachments where feasible.
7. Apple folders map to collections.
8. Unmapped items go to Inbox / Unorganized.
9. Import history records success/failures.
10. Dashboard Imported and Needs Review cards update.

### One-way refresh flow

1. User clicks Refresh Imported Notes or system monitors where possible.
2. System checks mapped Apple Notes source items.
3. Branch:
   - Source unchanged -> no action.
   - Apple source changed, Zoid copy unchanged -> refresh Zoid copy automatically and version prior Zoid copy.
   - Apple source changed, Zoid copy edited -> create imported-source update version and show compare/merge banner.
   - Source missing/unavailable -> mark source unavailable; do not delete Zoid note.
4. Activity logs refresh/update status.

### Compare imported update flow

1. User sees banner: “Apple Notes source changed.”
2. User opens compare.
3. Side-by-side view shows:
   - Current Zoid note
   - Imported Apple-source update
   - Metadata differences
4. Actions:
   - Keep Zoid version
   - Accept imported update into Zoid
   - Create merged Zoid note/version
   - Dismiss for later
5. Zoid saves resolution as versioned event.
6. Apple Notes is never mutated.

### Important exclusion

There is no flow where editing a Zoid note updates Apple Notes. There is no Apple Notes delete write-back. There is no destructive sync to Apple Notes.

## 9. Collections Management Flow

### Screen: Collections Management

Layout:

- Collection list/table
- Counts
- Health warnings
- Linked entities
- Default marker
- Archive state
- Bulk move panel
- Right inspector

Flow:

1. User opens Collections.
2. System lists flat collections.
3. Nested Apple Notes folders are preserved as path metadata/display labels, not required as actual nested hierarchy.
4. User can:
   - Create collection
   - Rename collection
   - Set default collection
   - Archive collection
   - Delete empty collection
   - Move notes out of collection
5. Delete semantics:
   - Empty collection -> confirm delete.
   - Non-empty collection -> cannot delete until notes are moved; offer Move to another collection or Move to Inbox.
6. Activity logs collection operations.

## 10. Tags Management Flow

### Screen: Tags Management

Layout:

- Tags table
- Counts
- Aliases
- Duplicate warnings
- Unused warnings
- Linked entities
- Bulk tag/untag panel
- Right inspector

Flow:

1. User opens Tags.
2. Tags show display name and normalized key.
3. Normalization detects case/spacing/punctuation duplicates while preserving display labels.
4. User can:
   - Create tag
   - Rename tag
   - Archive tag
   - Delete tag from notes
   - Merge tags
   - Add alias
   - Bulk tag/untag notes
5. Merge flow:
   - Show source tags, target tag, affected note count.
   - Confirm.
   - Apply merge and log activity.

## 11. Notes Settings Flow

### Screen: Notes Settings

Sections:

1. Vault
2. iCloud / Local Sync
3. Apple Notes Import / Refresh
4. History & Versions
5. Trash & Archive
6. Search & Indexes
7. Semantic Search
8. Agent Note Operations
9. Default Collection
10. Privacy
11. Export / Recovery

### Vault settings flow

1. User opens Settings > Vault.
2. Sees current vault path/status.
3. Actions:
   - Move Current Vault
   - Open Existing Vault
   - Choose External Folder
   - Use iCloud/Synced Folder
4. Move and Open are separate. No automatic merging.
5. Unknown folder contents require explicit confirmation.

### iCloud/local sync settings flow

Storage decision:

- Synced vault uses per-note structured files as durable synced source.
- SQLite is local index/cache, rebuildable from per-note files.
- Do not sync one live SQLite DB as primary source.

Flow:

1. User selects synced folder.
2. System validates layout.
3. System writes per-note structured records, exports, attachments, and manifest files.
4. Local SQLite cache indexes synced files.
5. If sync conflict detected, show conflict state and recovery tools.

### Search/index settings flow

1. User sees text index status and semantic index status.
2. User can rebuild indexes.
3. Rebuild runs in background.
4. Search remains available with stale/partial marker until complete.
5. Semantic unavailable means disabled semantic chip + explanation.

### Agent settings flow

1. Master toggle defaults off.
2. User turns on agent note operations.
3. Warning explains versions, audit, and autonomy levels.
4. User chooses autonomy levels:
   - Suggest only
   - Apply metadata
   - Append only
   - Patch body with diff
   - Full edit
5. Per-agent/session controls appear when Agents Workspace supports them.

### History/trash settings flow

1. User configures version retention/storage limits.
2. User configures Trash retention.
3. Activity metadata retention is separate and longer-lived by default.
4. Permanent delete clears note content/attachments per policy but may keep lightweight audit metadata unless user clears it.

## 12. Notes Activity / History Flow

### Screen: Notes Activity / History

Layout:

- Filter bar: actor, note, operation, source, result, linked entity, date
- Event list
- Event detail inspector
- Restore/diff/open actions

Flow:

1. User opens Activity.
2. System lists events.
3. User filters/searches.
4. User selects event.
5. Branch:
   - Version event -> view/diff/restore.
   - Trash event -> restore if retained.
   - Apple import/refresh issue -> open Imported Apple Notes.
   - Agent event -> open agent run/session.
   - Sync conflict -> open resolver/recovery.
6. Activity metadata persists longer than full content versions.

## 13. Trash / Archive / Permanent Delete Flow

### Trash as system filter

Trash is not a separate primary screen. It is:

- A left-nav item/filter under Notes.
- An All Notes system filter.
- A Settings retention section.

### Move to Trash

1. User deletes note.
2. Modal: “Move to Trash?”
3. Confirm.
4. Note hidden from active views.
5. Note visible in Trash filter.
6. Restore available until retention expires.

### Restore

1. User opens Trash filter or Activity event.
2. Click Restore.
3. If prior collection exists -> restore there.
4. If prior collection missing/archived -> restore to Inbox / Unorganized and warn user.

### Permanent delete

1. User opens Trash.
2. Selects Permanently Delete.
3. Explicit destructive confirmation.
4. System deletes note content and attachments after retention policy.
5. Attachments referenced elsewhere are retained.
6. Lightweight audit entry may remain unless cleared by policy.

### Archive

1. User clicks Archive from note/list.
2. Note hidden from default active views.
3. Note searchable with Archive filter.
4. Unarchive restores it to active views.

## 14. iCloud / Local Folder Sync Flow

### Desktop synced vault setup

1. User opens Settings > iCloud / Local Sync.
2. Chooses synced folder.
3. System validates access and vault layout.
4. Branch:
   - Empty folder -> initialize vault.
   - Existing Zoid Notes vault -> open existing vault.
   - Unknown contents -> warning + explicit initialize/open choice.
5. System stores per-note structured files and rebuildable SQLite cache.
6. Dashboard shows sync status.

### Local folder conflict flow

1. System detects conflicting per-note files/manifests.
2. Mark affected note(s) with conflict badge.
3. Activity logs conflict.
4. User opens conflict resolver.
5. Side-by-side compare offers:
   - Keep local version
   - Keep synced-folder version
   - Create merged Zoid version
6. Resolution creates version and clears conflict.

### Provider behavior

- Treat sync provider as generic filesystem sync.
- iCloud-specific language appears only where iOS/app-container access requires it.
- Dropbox/Google Drive/Syncthing-like folders are supported only as generic folders with provider limitations shown as warnings.

## 15. Mobile Companion Flow

### Mobile First Launch / Vault Connect

1. User opens iOS companion.
2. App looks for iCloud/app-container synced Zoid Notes vault.
3. Branch:
   - Vault found -> open Mobile Notes List.
   - No vault found -> show setup instructions and allow capture-only holding area.
   - Permission missing -> show permission help.
4. If capture-only holding area is used, notes sync/merge when vault becomes available.

### Mobile Capture

Actions:

- Quick text note
- Voice note
- Photo attachment

Flow:

1. User taps capture action.
2. Note created in Inbox / Unorganized.
3. Saves offline immediately.
4. Voice capture stores audio first.
5. Local transcription runs where available.
6. Remote transcription requires explicit setting/confirmation.
7. Syncs to desktop vault when available.

### Mobile Notes List / Search

1. User opens list.
2. Uses local/offline index.
3. Filters by collection/tag/source.
4. Opens note.
5. If index stale, show stale marker and continue.

### Mobile Note Read / Light Edit

Editable on mobile:

- Title
- Plain text paragraphs
- Headings
- Bullets
- Checklists
- Quotes
- Simple attachments
- Collection/tags

View-only or desktop-first:

- Canvas deep editing
- Tables beyond simple view
- Complex entity cards
- Advanced settings
- Sync repair
- Deep tag/collection maintenance

Flow:

1. User opens note.
2. Reads content.
3. Edits simple blocks/metadata.
4. Saves offline.
5. Syncs through iCloud/local vault.
6. Desktop shows updated note and activity.

### Mobile Sync Status

1. User opens sync status.
2. Sees vault connection, last sync, pending items, conflicts/errors.
3. Can retry sync, open setup help, or keep capture-only mode.
4. Complex conflict repair directs user to desktop.

## 16. End-to-End Designed Journeys

### Journey A — First use to first note

1. Open Notes.
2. Choose Default Local Vault, Choose Folder, or iCloud/Synced Folder.
3. System creates vault + Inbox / Unorganized + General.
4. Dashboard empty state opens.
5. User clicks New Note.
6. Draft opens in Note Detail.
7. User writes title/content.
8. Autosave persists structured note.
9. Markdown snapshot/export generated.
10. Dashboard and All Notes update.
11. Activity logs creation.

### Journey B — Import Apple Notes and organize

1. Open Imported Apple Notes.
2. Connect Apple Notes.
3. Start import.
4. Notes import into mapped collections or Inbox / Unorganized.
5. Failed items appear in Failed / Retry.
6. User opens Imported or Needs Review.
7. Bulk moves/tags/links imported notes.
8. Apple source updates later appear as Imported Updates, not write-back sync.

### Journey C — Apple Notes source update after Zoid edit

1. User imports Apple Note.
2. User edits the Zoid copy.
3. Apple source changes later.
4. Zoid detects source update.
5. Note gets compare/merge banner.
6. User keeps Zoid, accepts imported update into Zoid, or creates merged Zoid version.
7. Apple Notes remains unchanged by Zoid.

### Journey D — Work note linked to agent run

1. User opens an agent run.
2. Clicks Create linked note or Summarize into Note.
3. Note opens with agent run/project link.
4. Agent output appears as review card.
5. User applies/edits/dismisses.
6. Version checkpoint and activity log are created.
7. Task creation opens linked task draft if requested.

### Journey E — Knowledge cleanup session

1. Open Dashboard.
2. Click Needs Review.
3. All Notes opens filtered to unorganized/imported/agent-created/suggested-link items.
4. User bulk moves/tags/marks reviewed.
5. User opens Tags to merge duplicates.
6. User opens Collections to resolve empty/stale collections.
7. Dashboard health improves.

### Journey F — Mobile capture to desktop organization

1. Desktop vault is set to iCloud/synced folder.
2. Mobile connects to vault.
3. User captures voice/photo/quick note.
4. Mobile stores note offline in Inbox / Unorganized.
5. Sync brings note to desktop.
6. Dashboard shows Needs Review.
7. User organizes, links, and optionally asks agent to summarize.

### Journey G — Synced folder conflict

1. Same note is edited on two devices before sync settles.
2. Per-note structured file conflict is detected.
3. Desktop shows conflict badge/banner.
4. User opens resolver.
5. Chooses local, synced-folder version, or merged version.
6. Version history records both versions.
7. Conflict clears.

## 17. Screen-to-Screen Transition Matrix

Dashboard:

- New Note -> Note Detail
- Search -> All Notes filtered
- Recent item -> Note Detail
- Inbox card -> All Notes filtered to Inbox / Unorganized
- Unlinked card -> All Notes filtered to Unlinked
- Imported Updates -> Imported Apple Notes
- Collections panel -> Collections Management
- Tags panel -> Tags Management
- Activity panel -> Notes Activity / History
- Settings -> Notes Settings

All Notes:

- Open note -> Note Detail
- Preview row -> Right inspector
- Bulk move/tag/link -> Bulk action modal/drawer -> All Notes updated
- Trash filter -> All Notes trash view
- Archive filter -> All Notes archive view
- Search mode unavailable -> Search fallback notice

Note Detail:

- Link Entity -> Entity picker -> Note Detail
- Summarize/organize -> Agent review card -> Note Detail
- Create Task -> Task draft with note link
- History restore -> Restore confirmation -> Note Detail
- Delete -> Move to Trash confirmation -> All Notes/Dashboard
- Imported update banner -> Compare Imported Update -> Note Detail
- Sync conflict banner -> Conflict resolver -> Note Detail

Imported Apple Notes:

- Connect -> Permission prompt -> Import Setup
- Start Import -> Import progress -> Imported Notes / Failed
- Refresh -> Source Updates / History
- Compare update -> Compare Imported Update -> Note Detail
- Open imported note -> Note Detail

Collections:

- Create/Rename/Archive/Delete -> Collections updated
- Bulk move notes -> All Notes filtered/confirmation
- Open collection -> All Notes filtered by collection

Tags:

- Create/Rename/Archive/Delete/Merge/Alias -> Tags updated
- Open tag -> All Notes filtered by tag

Settings:

- Change vault -> Folder picker / migration flow
- Rebuild index -> Background progress -> Dashboard/Search status
- Agent toggle -> Warning/autonomy choices
- Apple Notes settings -> Imported Apple Notes
- Export/recovery -> Files/Finder reveal/export flow

Activity:

- Open note event -> Note Detail
- Restore version -> Restore confirmation -> Note Detail
- Agent event -> Agent run/session
- Import event -> Imported Apple Notes
- Sync conflict event -> Conflict resolver

Mobile:

- First launch -> Vault connect / Capture-only holding area
- Capture -> Mobile Note Read/Edit
- Search result -> Mobile Note Read/Edit
- Sync status -> Retry / Setup help / Desktop repair prompt

## 18. Remaining Implementation Warnings

These are not open product questions anymore; they are implementation risks to validate:

1. Apple Notes import/monitoring platform limits may reduce source-refresh capability.
2. iCloud/local sync must not rely on one live synced SQLite DB as durable source.
3. Remote model use for note content must be an explicit privacy/provider setting.
4. Mobile cannot promise deep canvas/table editing.
5. Voice transcription must default to local where possible or explicit opt-in for remote.
6. Files/Finder permissions need native Tauri/macOS testing.
7. Feature is not complete until native desktop and mobile sync/capture flows are verified, not just browser previews.
