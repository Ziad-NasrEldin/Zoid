# PRD: Zoid Notes Workspace — Full Product Scope

Date: 2026-06-06
Product area: Notes Workspace, Files Workspace, Agents Workspace, Apple Notes, iCloud/local sync, iOS companion
Source discovery: `/Users/ziadnasreldin/brainstorms/2026-06-06-zoid-notes-workspace.md`
Screen reference: `/Users/ziadnasreldin/Zoid/Docs/designer-screen-reference/04-notes-workspace.md`

## 1. Product Definition

The Notes Workspace is Zoid's full local-first knowledge workspace. It is not a lightweight text editor, MVP, or V1 slice. It is the complete Notes product scope for capturing, writing, organizing, linking, searching, syncing, and agent-operating on knowledge across Zoid.

Notes must combine:

- Structured block/canvas note editing.
- Markdown-compatible portability for simple notes and exports/snapshots.
- Collections, tags, backlinks, and full Zoid entity links.
- Apple Notes import and one-way Apple Notes monitoring/import refresh into Zoid. Zoid does not write back into Apple Notes.
- Local structured search and local semantic/vector search where available.
- Agent-assisted note creation, summarization, organization, linking, task creation, and editing.
- Local history, restore, activity/audit log, trash/archive, and settings.
- iCloud Drive/local folder sync as the primary sync model.
- iOS companion app for capture, reading, search, light editing, and sync.

The implementation tracker may sequence delivery in milestones, but the PRD describes the full product.

## 2. Primary User

Primary user: Ziad, founder/operator/product owner/developer using Zoid as an AI OS across projects, agents, files, content, repos, tasks, chats, products/business entities, and operations.

The workspace must support fast personal capture, durable operational knowledge, deep linking to work happening elsewhere in Zoid, and safe agent operations with auditability.

## 3. Core Problems

Notes Workspace solves:

1. Knowledge scattered across Apple Notes, project files, agent sessions, chats, tasks, browser captures, repos, and content workflows.
2. Notes that are not connected to the actual work entities they describe.
3. Weak capture/import flows that create unorganized knowledge debt.
4. Lack of reliable local-first storage, history, restore, and auditability.
5. Inability for agents to safely create, organize, summarize, and act on notes.
6. Difficulty finding notes by meaning, metadata, source, tags, links, or context.
7. Desktop/mobile sync needs without requiring a custom Zoid account backend.
8. Need for a powerful writing surface that supports blocks, canvas thinking, attachments, and Markdown portability.

## 4. Goals

- Provide a complete local-first Notes Workspace in desktop Zoid.
- Store block/canvas notes in structured local storage as source of truth.
- Preserve Markdown-compatible simple notes and Markdown export/snapshot portability.
- Maintain SQLite metadata/index for tags, links, summaries, history, relationships, search, and sync state.
- Support one primary collection, many tags, many entity links, and many note backlinks per note.
- Include an Inbox / Unorganized collection for quick captures, imports, and agent-created notes needing review.
- Import Apple Notes with title, body, dates where available, folders mapped to collections, attachments where feasible, and history/status.
- Support Apple Notes import plus one-way Apple Notes change monitoring/import refresh into Zoid where platform access allows. Zoid does not write back into Apple Notes.
- Provide a Notion-style block editor plus whiteboard/canvas blocks.
- Provide note history, restore, diff where feasible, actor/source tracking, and retention controls.
- Provide user-facing Notes activity/audit log.
- Support local structured search and local semantic/vector search where available.
- Let agents create, summarize, organize, link, append, patch, draft edits, and create tasks from notes under explicit note-agent controls.
- Support iCloud Drive/local folder sync as the primary sync model.
- Provide an iOS companion app that works with the synced vault/container for capture, read/search, and lightweight editing.
- Provide dedicated Collections and Tags management screens.
- Remove collaboration/sharing from scope.

## 5. Non-Goals

- Do not treat this as an MVP or reduced release definition.
- Do not require a custom Zoid account/cloud backend for Notes sync.
- Do not include collaboration, invited users, shared links, comments, public publishing, or real-time multi-user editing.
- Do not send full notes to remote models by default for semantic search.
- Do not allow autonomous agent note operations without an explicit off-by-default Notes setting.
- Do not silently overwrite conflicting Apple Notes/Zoid edits.
- Do not make permanent deletion the default delete action.

## 6. Storage and Vault Model

### 6.1 Source of Truth

Because full product scope includes block editing and canvas blocks, Markdown cannot be the only source of truth.

Required model:

- Structured local storage is source of truth for block/canvas notes.
- Simple notes can still save clean `.md` where possible.
- Markdown export/snapshot exists for portability and Finder visibility.
- SQLite stores metadata/index/state for tags, links, summaries, history, search, sync, and relationships.
- Attachments live in the notes vault with metadata linking them to notes/blocks.

### 6.2 Vault Location

Default:

- Zoid-managed local notes vault inside app data.

Also required:

- Optional user-selected external folder/vault support.
- iCloud Drive/local folder synced vault support.
- Files Workspace integration to reveal/open Markdown exports/snapshots and relevant vault assets.
- Clear vault status in Settings.

### 6.3 Suggested Logical Vault Contents

Implementation can choose exact paths, but product needs these concepts:

- Structured note records.
- Markdown exports/snapshots.
- Attachments.
- Canvas/block assets.
- Trash/archive.
- Version history.
- Import/sync state.
- Search/vector indexes.

## 7. Notes Data Model

A note includes:

- id
- title
- body/content blocks
- optional Markdown body/export
- canvas data where applicable
- primary collection id
- tags
- backlinks
- linked Zoid entities
- source: manual, Apple Notes, agent, file, browser, content, task, chat, repo, system
- created/updated timestamps
- created/updated actor: user, agent, import, system
- sync state
- conflict state
- version history state
- summary
- attachments
- trash/archive state
- semantic index state

## 8. Organization Model

### 8.1 Collections

Collections are primary containers/folders for notes.

Rules:

- Each note has one primary collection.
- A default Inbox / Unorganized collection exists.
- Imported Apple Notes folders map to collections where feasible.
- Collections can be created, renamed, archived, deleted, and inspected.
- Collection health should show counts, unorganized notes, stale/empty state, linked entities, and warnings.

### 8.2 Tags

Tags are flexible cross-cutting labels.

Rules:

- Notes can have many tags.
- Tags can be created, renamed, archived, deleted, merged, and aliased.
- Tag health should show unused, duplicate, and near-duplicate tags.
- Bulk tag operations are required.

### 8.3 Backlinks and Entity Links

Notes can link to:

- Projects
- Agent sessions/runs
- Files/folders
- Content pieces/assets
- Browser captures
- Repos
- Tasks
- Chats
- Products/business entities
- Calendar events
- Other Zoid entities as the graph grows

Linking behavior:

- Manual link/unlink from inspector.
- Deterministic automatic links when note source is known.
- Notes created from an agent run link to that run/project.
- Notes created from files/browser/content/tasks link to source entities.
- Uncertain matches appear as suggestions.
- Linked entities appear in the right inspector.

## 9. Editor Product Scope

The Note Detail / Editor must support:

- Title editing.
- Autosave and save status.
- Metadata row: collection, tags, source, updated, sync state.
- Notion-style block editing.
- Whiteboard/canvas blocks.
- Markdown shortcuts.
- Markdown-compatible export/snapshot.
- Headings, paragraphs, checklists, bullets, numbered lists, quotes, code blocks, tables.
- Images and attachments.
- Embedded/linked entity cards.
- Slash commands.
- Drag/reorder blocks.
- Backlinks.
- Linked entities.
- Note history.
- Right inspector.
- Summarize, organize, create task, duplicate, delete, open/reveal file/export.

## 10. Whiteboard / Canvas Blocks

Canvas blocks should support visual thinking inside notes:

- Freeform canvas block inside a note.
- Text nodes/cards.
- Basic shapes/connectors.
- Linked note/entity cards.
- Images/attachments.
- Pan/zoom.
- Export/snapshot for portability.
- Structured storage to avoid Markdown loss.

Heavy canvas editing is desktop-first. Mobile can view and perform light/basic interactions if feasible, but full canvas tooling stays desktop-first.

## 11. Apple Notes Import and Sync

### 11.1 Import

Required import behavior:

- Import Apple Notes into Zoid notes.
- Preserve title and body.
- Preserve created/updated dates where available.
- Map Apple Notes folders to Zoid collections where feasible.
- Copy attachments where feasible.
- Show import status, failed imports, retry, cancel, and import history.
- Imported notes needing organization land in Inbox / Unorganized or mapped collections.

### 11.2 One-Way Apple Notes Monitoring / Refresh

Bidirectional Apple Notes sync is removed from scope. Zoid imports and monitors Apple Notes as an external source where platform access allows, but Zoid does not write changes back into Apple Notes.

Target behavior:

- Changes from Apple Notes can refresh/update the mapped Zoid copy where feasible.
- Zoid edits remain inside Zoid notes and do not mutate Apple Notes.
- Apple Notes source state is visible: imported, refreshed, source changed, source unavailable, or refresh failed.
- Refresh errors are visible and actionable.

Conflict handling:

- If an imported Apple Note changes in Apple Notes after the Zoid copy was edited, Zoid must not overwrite the Zoid version silently.
- Zoid creates an imported-source update version and surfaces a compare/merge banner.
- Resolution allows keeping the Zoid version, accepting the Apple-source refresh into Zoid, or creating a merged Zoid note/version.

Risk flag:

- Apple Notes APIs/permissions may constrain import/monitoring. Product intent is one-way Apple Notes-to-Zoid refresh only, and implementation must validate/document platform limits honestly.

## 12. Search and Indexing

### 12.1 Structured Local Search

Search must cover:

- Title
- Body
- Blocks
- Canvas text
- Tags
- Collections
- Linked entities
- Imported source
- Summaries
- Backlinks
- Attachment text where OCR/text extraction is available

Filters should include:

- Collection
- Tag
- Source
- Linked entity type
- Updated date
- Created date
- Actor/source
- Has attachments
- Has conflicts
- Needs organization
- In trash/archive

### 12.2 Semantic / Vector Search

Semantic search should run locally where possible over:

- Note title
- Body
- Blocks
- Canvas text
- OCR-able attachment text
- Imported metadata
- Tags
- Links

Rules:

- If local embeddings are unavailable, fall back to text search.
- Mark semantic search unavailable rather than pretending it ran.
- Do not send full notes to remote models by default.

## 13. Agent Operations

Agents can:

- Create notes.
- Summarize notes.
- Organize collections/tags.
- Suggest links.
- Create tasks from notes.
- Append to notes.
- Patch notes.
- Draft edits.
- Generate summaries and metadata.
- Help resolve unorganized notes.

Permission model:

- Selected-note read/summarize is allowed when the user explicitly invokes it.
- Autonomous agent note operations require an off-by-default setting: “Allow agents to create/edit/organize notes.”
- Operations are logged.
- Edits are backed by version history.
- Direct overwrite/delete requires safe controls.
- Delete routes to trash/archive by default.

## 14. History, Restore, Trash, and Audit

### 14.1 Note History

History must include:

- Autosave revisions.
- Manual save revisions where applicable.
- Restore previous version.
- Diff where feasible.
- Actor/source: user, agent, import, sync, system.
- Linked operation/source where applicable.
- Retention/storage controls.

### 14.2 Trash / Archive

Deletion behavior:

- Soft delete/archive first.
- Deleted notes move to Trash/Archive inside the notes vault.
- History is preserved temporarily.
- Restore is supported.
- Permanent deletion requires explicit confirmation.

### 14.3 Activity / Audit Log

Notes needs a user-facing activity/history stream showing:

- Actor: user, agent, import, sync, system.
- Note.
- Operation.
- Timestamp.
- Linked entity/run if any.
- Version/history availability.
- Restore action where available.
- Result/sync/error state.

## 15. Sync and iOS Companion

### 15.1 Sync Direction

Use iCloud Drive/local folder sync as the primary sync direction, not a custom Zoid account cloud backend.

Required:

- Desktop can use a local vault and/or synced vault.
- iOS companion reads/writes the synced notes vault through iCloud Drive/app container where possible.
- Sync state visible in desktop and mobile.
- Offline-first behavior.
- Conflict handling for concurrent edits.
- Recovery/export.

### 15.2 Mobile Companion

The iOS companion app supports capture + read + lightweight edit:

- Quick notes.
- Voice capture.
- Photo attachment capture.
- Search.
- View collections/tags/links.
- Basic block editing.
- Offline cache.
- Sync with desktop through iCloud/local synced vault.

Desktop-first areas:

- Heavy canvas editing.
- Full admin/settings.
- Deep collection/tag maintenance.
- Advanced sync repair.

## 16. Settings

Notes Settings should include:

- Vault location/status.
- External folder/vault setup.
- iCloud/local sync status.
- Apple Notes import settings/history.
- Apple Notes import/refresh status/errors.
- History retention/storage limit.
- Trash retention.
- Search/index rebuild.
- Semantic search availability/index status.
- Agent note-operations toggle.
- Default collection.
- Privacy/local-only explanation.
- Export/recovery options.

## 17. Screen Requirements

### 17.1 Notes Dashboard

Purpose: knowledge operations dashboard.

Must show:

- Header: Search Notes, New Note, New Collection, Import Apple Notes, Settings.
- Summary cards: recent, unlinked, imported, needs organization.
- Recent notes list.
- Collections list.
- Tags list.
- Linked entity notes.
- Recent note history.
- Collections/tags health.
- Suggested organization actions.
- Right inspector.

States:

- First-run vault state.
- Empty notes state.
- Import in progress.
- Import failed.
- Unorganized notes need review.
- Agent toggle off.
- Semantic search unavailable.

### 17.2 All Notes

Must show:

- Search/filter/sort toolbar.
- Notes list with title, collection, workspace/source, tags, updated, linked entities, summary.
- Note preview inspector.

Actions:

- Open.
- Preview.
- Link.
- Summarize.
- Move.
- Tag.
- Delete.
- Bulk organize.

### 17.3 Note Detail / Editor

Must show:

- Editor header: title, save status, file/export path, actions.
- Metadata row: workspace/source, collection, tags, source, updated, sync state.
- Block editor.
- Canvas blocks.
- Optional Markdown preview/export pane.
- Linked entities.
- Backlinks.
- Note history.
- Right inspector.

Actions:

- Save.
- Link Entity.
- Summarize.
- Organize.
- Create Task.
- Open/reveal file or export.
- Duplicate.
- Delete.
- Restore version.
- Resolve conflict.

### 17.4 Imported Apple Notes

Must show:

- Import controls.
- Import/refresh controls/status.
- Import status panel.
- Imported notes list.
- Mapping / organization panel.
- Import history.
- Failed/retry state.
- Imported-source update/conflict state.

Actions:

- Start Import.
- Cancel.
- Retry Failed.
- Open Imported Note.
- Organize.
- Confirm Migration where needed.
- Compare Imported Update.

### 17.5 Collections Management

Must show:

- Collections list.
- Counts.
- Unorganized notes.
- Linked entities.
- Health warnings.

Actions:

- Create.
- Rename.
- Archive.
- Delete.
- Bulk move notes.
- Set default collection.

### 17.6 Tags Management

Must show:

- Tags list.
- Counts.
- Aliases.
- Duplicate/unused warnings.
- Linked entities.

Actions:

- Create.
- Rename.
- Archive.
- Delete.
- Merge tags.
- Add aliases.
- Bulk tag/untag notes.

### 17.7 Notes Settings

See Settings section.

### 17.8 Notes Activity / History

Must show the user-facing activity/audit stream with filters by actor, note, operation, source, result, linked entity, and date.

## 18. Integration Requirements

### 18.1 Files Workspace

- Reveal/open Markdown exports/snapshots.
- Reveal/open attachments where safe.
- Respect Files Workspace permission model.
- Link notes to files/folders.

### 18.2 Agents Workspace

- Notes can link to agent sessions/runs.
- Agent-created notes link to originating run/project.
- Agent operations require Notes permissions.
- Activity log records agent operations.

### 18.3 Tasks / Calendar / Projects

- Create tasks from notes.
- Link notes to tasks/calendar events/projects.
- Surface linked notes in relevant right inspectors.

### 18.4 Content / Browser / Code / Chats

- Link notes to content assets, browser captures, repos, chats, and related entities.
- Allow deterministic source links when notes are created from those modules.

## 19. Acceptance Criteria

The Notes product is complete when:

- Desktop Zoid can create, edit, save, search, organize, delete/archive, restore, and inspect notes.
- Structured local storage is source of truth for block/canvas notes.
- Simple notes and exports/snapshots remain Markdown-compatible.
- SQLite metadata/index persists tags, links, summaries, history, search, sync state, and relationships.
- Collections/tags/backlinks/entity links work end-to-end.
- Inbox / Unorganized workflow exists.
- Apple Notes import works with status/history and feasible metadata/attachments.
- Apple Notes import/one-way refresh works within documented platform limits without mutating Apple Notes.
- Imported-source update handling creates versions and requires resolution instead of silent overwrite.
- Block editor and canvas blocks work in desktop Zoid.
- Local structured search works.
- Local semantic/vector search runs where available with text fallback.
- Agent note operations work behind explicit off-by-default toggle, with logs and version history.
- Notes Settings exposes vault, sync, import, retention, trash, search, semantic, and agent controls.
- Notes Activity / History is user-facing and filterable.
- iCloud/local folder sync works as primary sync model.
- iOS companion supports capture, read/search, lightweight edit, offline cache, and sync.
- Collaboration/sharing is absent from scope.
- Native/Tauri desktop behavior is verified, not just browser preview.

## 20. Open Risks / Validation Needed

- Apple Notes import/monitoring feasibility and permissions may be constrained by macOS/iOS platform APIs.
- Structured block/canvas source plus Markdown export needs careful non-lossy serialization rules.
- iCloud Drive/local folder sync conflict behavior must be tested with desktop + iOS devices.
- Local semantic search requires local embedding/index availability; fallback must be honest.
- Agent operations need robust permissioning, history, rollback, and audit before broad autonomy.
