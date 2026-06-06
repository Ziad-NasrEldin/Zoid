# Zoid Notes Workspace — Finished Product Screen Design Spec

Date: 2026-06-06
Module: Notes Workspace
Related docs: `prd.md`, `implementation-tracker.md`, `user-flow-map-designed.md`

Purpose: define the finished-product UI for every Notes Workspace screen, including layout, buttons, states, modals, drawers, inspectors, and screen-to-screen behavior. This is not a prompt and not an MVP slice. It is the product design spec to hand to implementation/design.

Hard product decisions applied:

- Apple Notes is import + one-way source refresh into Zoid only.
- Zoid never writes back to Apple Notes.
- iCloud/local folder sync uses per-note structured files plus SQLite index/cache.
- Agents are helpful but gated, logged, versioned, and reviewable.
- Trash and Archive are separate states.
- Mobile supports capture, read/search, and lightweight edit only.

## 1. Product Feel

The Notes Workspace should feel like a finished operating-system-grade knowledge workspace, not a generic notes app.

Design qualities:

- Dense but calm.
- Native desktop command-center feel.
- Local-first, trustworthy, explicit about state.
- Fast scanability for a solo operator managing real work.
- Serious enough for agent/audit/versioning workflows.
- Familiar Finder/Notion/Linear-style interaction patterns without copying any single app.

Avoid:

- Decorative AI gradients.
- Fake dashboard metrics.
- Oversized marketing cards.
- Empty SaaS gloss.
- Hidden destructive behavior.
- Modals that obscure critical context unless the action is destructive or permission-related.

## 2. Global Layout System

### Desktop shell

Use a three-region desktop app layout:

1. Global Zoid sidebar
   - Existing app-wide workspace nav.
   - Notes selected state.

2. Notes workspace navigation rail
   - Width: compact, 220–260px.
   - Contains Notes-specific destinations and status badges.

3. Main content area
   - Responsive main work surface.
   - Contains screen header, toolbar, primary content.

4. Right inspector
   - Width: 320–380px.
   - Collapsible.
   - Context-sensitive.
   - Persistent across Dashboard, All Notes, Editor, Imported Apple Notes, Collections, Tags, Activity.

### Standard desktop screen structure

Each major desktop screen uses:

- Screen header.
- Optional status strip.
- Primary action cluster.
- Secondary toolbar/filters.
- Main body.
- Right inspector.
- Toast/action feedback area.

### Standard buttons

Primary buttons:

- New Note
- Start Import
- Apply
- Restore
- Save Setting

Secondary buttons:

- New Collection
- Refresh
- Rebuild Index
- Reveal in Finder
- Open Settings

Destructive buttons:

- Move to Trash
- Permanently Delete
- Remove Tag
- Delete Collection

Destructive buttons must use clear labels and confirmation where required.

### Status language

Use short explicit status labels:

- Vault ready
- Indexed
- Index rebuilding
- Local only
- Synced folder
- Source update pending
- Agent suggest-only
- Agent metadata apply enabled
- Read-only
- Conflict
- Needs review

## 3. Notes Navigation Rail

### Layout

Top:

- Notes title
- Small vault status chip
- Quick New Note icon button

Primary nav:

1. Dashboard
2. All Notes
3. Inbox / Unorganized
4. Collections
5. Tags
6. Imported Apple Notes
7. Trash
8. Activity
9. Settings

Secondary footer:

- Vault path short label
- Sync state dot
- Index state dot
- Agent state dot

### Badges

- Inbox / Unorganized: count of notes needing organization.
- Imported Apple Notes: count of source updates / failed imports.
- Trash: count of trashed notes.
- Activity: count of unresolved errors/conflicts.

### Interactions

- Click nav item -> opens screen.
- Hover status chip -> tooltip with full state.
- Click vault chip -> Settings > Vault.
- Click sync dot -> Settings > iCloud / Local Sync.
- Click index dot -> Settings > Search & Indexes.
- Click agent dot -> Settings > Agent Note Operations.

## 4. Shared Right Inspector

### Default states

No selection:

- Title: “Inspector”
- Message: “Select a note, collection, tag, activity event, or import issue.”
- Shows quick tips and current workspace status.

Selected note:

- Note title
- Summary
- Collection picker
- Tags editor
- Source
- Linked entities
- Backlinks
- Latest activity
- Quick actions: Open, Link Entity, Mark Reviewed, Archive, Move to Trash

Selected collection:

- Collection name
- Count
- Default/archived state
- Health warnings
- Linked entities
- Actions: Open, Rename, Set Default, Archive, Move Notes

Selected tag:

- Tag name
- Count
- Aliases
- Duplicate warnings
- Actions: Open, Rename, Merge, Archive, Delete Tag

Selected activity event:

- Actor
- Operation
- Timestamp
- Target note/entity
- Result
- Actions: Open Note, View Diff, Restore, Open Agent Run, Open Import Issue

Selected import issue:

- Source title
- Status
- Error/update summary
- Metadata differences
- Actions: Retry, Compare, Open Zoid Copy, Dismiss

### Inspector tabs

For notes only:

- Details
- Links
- History
- Activity
- Agent
- Source

## 5. Screen 1 — First-Run Vault Setup

### Purpose

Get the user into Notes with a valid local-first vault while making storage choices clear.

### Layout

Centered setup panel inside Notes shell.

Header:

- Title: “Set up your Notes vault”
- Subtitle: “Store notes locally, keep Finder-visible exports, and optionally sync through iCloud or a local folder.”

Vault cards:

1. Default Local Vault
   - Badge: Recommended
   - Copy: “Fastest start. Zoid manages the vault locally.”
   - Details: app-data path, can move later.
   - Button: Use Default Vault

2. Choose Folder
   - Copy: “Use a folder you control.”
   - Details: asks for Finder/file permissions.
   - Button: Choose Folder

3. iCloud / Synced Folder
   - Copy: “Use iCloud Drive or another synced folder for desktop/mobile sync.”
   - Details: provider behavior may vary.
   - Button: Choose Synced Folder

Footer:

- Learn more
- Cancel

### Flow

Use Default Vault:

1. Click Use Default Vault.
2. Button enters loading state: “Creating vault…”
3. Success toast: “Notes vault ready.”
4. Route to Dashboard empty state.

Choose Folder:

1. Click Choose Folder.
2. Folder permission modal opens.
3. User grants access.
4. Folder validation panel appears.
5. If valid, Initialize Vault.
6. Route to Dashboard.

Choose Synced Folder:

1. Click Choose Synced Folder.
2. Folder permission modal opens with iCloud/sync guidance.
3. User selects folder.
4. Validation detects empty/existing/unknown folder.
5. User confirms Initialize or Open Existing.
6. Route to Dashboard.

### States

Empty/no vault:

- Show setup cards.

Permission denied:

- Alert panel: “Zoid needs folder access to use this vault.”
- Buttons: Try Again, Use Default Vault, Open macOS Privacy Settings

Existing vault found:

- Modal: “Existing Zoid Notes vault found.”
- Buttons: Open Existing Vault, Choose Different Folder, Cancel

Unknown folder contents:

- Modal: “This folder is not empty.”
- Buttons: Initialize Zoid Vault Here, Choose Different Folder, Cancel

Read-only folder:

- Error banner: “This folder is read-only. Choose another folder or fix permissions.”

## 6. Screen 2 — Notes Dashboard

### Purpose

The operational home for Notes: what changed, what needs review, what is linked to work, and what to do next.

### Layout

Header row:

- Breadcrumb: Zoid / Notes
- Page title: Notes Dashboard
- Search Notes input
- Buttons: New Note, New Collection, Import Apple Notes, Settings

Status strip:

- Vault: Ready / Missing / Read-only
- Sync: Local only / Synced folder / Conflict / Pending
- Apple Notes: Not connected / Imported / Source updates / Failed
- Index: Indexed / Rebuilding / Semantic unavailable
- Agent: Suggest-only / Off / Metadata apply / Full edit

Summary card grid:

1. Recent
2. Inbox / Unorganized
3. Unlinked
4. Imported Updates
5. Needs Review
6. Search / Index Health

Main content:

Left large panel:

- Recent Notes list

Middle stacked panels:

- Collections
- Tags
- Linked Entity Notes

Lower panel:

- Suggested Organization
- Recent Activity

Right inspector:

- Selection details.

### Header buttons

Search Notes:

- Focus opens inline search suggestions.
- Enter routes to All Notes with query.

New Note:

- Creates draft and opens Note Detail.

New Collection:

- Opens New Collection modal.

Import Apple Notes:

- Opens Imported Apple Notes.

Settings:

- Opens Notes Settings.

### Summary cards

Recent card:

- Shows count and last updated time.
- Click opens All Notes sorted by recent.

Inbox / Unorganized card:

- Shows count and severity if >0.
- Click opens All Notes filtered to Inbox / Unorganized.

Unlinked card:

- Shows notes without entity links.
- Click opens All Notes filtered to Unlinked.

Imported Updates card:

- Shows source updates/failed imports.
- Click opens Imported Apple Notes Source Updates tab.

Needs Review card:

- Shows total grouped by reason.
- Click opens All Notes Needs Review filter.

Search / Index Health card:

- Shows text/semantic index state.
- Click opens Settings > Search.

### Recent Notes list

Columns:

- Title
- Collection
- Tags
- Source
- Updated
- Status badge

Row actions on hover:

- Open
- Preview
- Link
- Archive
- More

Click row:

- Selects row and updates inspector.

Double click / Enter:

- Opens Note Detail.

### Suggested Organization panel

Cards:

- “12 notes in Inbox” -> Review
- “5 suggested entity links” -> Apply/Review
- “3 duplicate tags” -> Open Tags
- “2 imported updates” -> Compare

Each card has:

- Reason
- Affected count
- Primary action
- Dismiss option

### Empty state

If no notes:

- Main empty panel: “Start your knowledge base.”
- Primary: New Note
- Secondary: Import Apple Notes
- Tertiary: Choose synced folder
- Explanation: “Notes stay local-first and can export to Markdown snapshots.”

### Error states

Vault missing:

- Red/amber banner: “Notes vault unavailable.”
- Buttons: Reconnect Vault, Use Default Vault, Open Settings

Index rebuilding:

- Blue banner: “Search is rebuilding. Results may be partial.”
- Progress indicator.

Agent off:

- Neutral chip: “Agents can only act when you ask.”

## 7. Screen 3 — All Notes

### Purpose

The central list/search/bulk organization surface.

### Layout

Header:

- Title: All Notes
- Saved view dropdown
- New Note button

Search/filter toolbar:

- Search input
- Mode chips: Text, Semantic, Hybrid
- Filter button
- Quick filters: Inbox, Unlinked, Imported, Needs Review, Archived, Trash
- Sort dropdown
- View switch: Table / Compact / Cards

Main table:

Columns:

- Checkbox
- Title
- Collection
- Tags
- Source
- Linked entities
- Updated
- Status

Right preview inspector.

Bulk action bar appears above table when selected.

### Search behavior

Search input:

- Placeholder: “Search notes, tags, source, links…”
- Typeahead suggestions:
  - Notes
  - Tags
  - Collections
  - Linked entities
  - Recent searches

Mode chips:

- Text: default exact/local search.
- Semantic: local semantic only; disabled if unavailable.
- Hybrid: blends text + local semantic.

Semantic unavailable notice:

- Inline info: “Semantic search is unavailable on this device. Text search is active.”
- Button: Open Search Settings

### Filter drawer

Open with Filter button.

Sections:

- Collection
- Tags
- Source
- Linked entity type
- Date range
- Actor
- Has attachments
- Review state
- Conflict/source update state
- Archive/Trash state

Buttons:

- Apply Filters
- Reset
- Save View

### Row states

Normal:

- Clean text, subtle metadata chips.

Needs review:

- Amber dot/chip.

Imported update:

- Blue chip: Source update.

Conflict:

- Red/amber chip: Conflict.

Archived:

- Muted row.

Trash:

- Muted row with restore/permanent-delete actions.

Read-only:

- Lock icon.

### Bulk action bar

Text: “N selected”

Actions:

- Move
- Tag
- Link Entity
- Summarize
- Mark Reviewed
- Archive
- Move to Trash
- More

Bulk confirmation modal:

For 2–9 notes:

- Title: “Apply bulk change?”
- Copy includes affected count.
- Buttons: Apply, Cancel

For 10+ notes:

- Title: “Apply bulk change to N notes?”
- Requires explicit confirmation checkbox: “I understand this changes N notes.”
- Buttons: Apply to N Notes, Cancel

For delete/archive:

- Always confirm.

### Preview inspector buttons

- Open Note
- Link Entity
- Edit Tags
- Move Collection
- Mark Reviewed
- Archive
- Move to Trash

Body editing is not allowed in preview.

## 8. Screen 4 — Note Detail / Editor

### Purpose

Primary writing, editing, linking, versioning, agent operation, and source-update surface.

### Layout

Header:

- Back button
- Breadcrumb: Notes / Collection / Note
- Editable title
- Save state: Saved / Saving / Unsaved / Error
- Source state chip
- Reveal Export button
- More menu

Status/banner zone:

- Imported source update banner
- Sync conflict banner
- Read-only vault banner
- Missing attachment warning
- Agent suggestion pending banner

Metadata row:

- Collection picker
- Tags editor
- Source chip
- Linked entities button
- Review state chip
- Updated timestamp

Editor body:

- Left margin block controls
- Main block editor column
- Optional split Markdown preview/export pane
- Canvas blocks inline with expand action

Right inspector:

Tabs:

- Details
- Links
- History
- Activity
- Agent
- Source

### Header actions

Back:

- If saving, wait/indicate save.
- If empty untouched draft, auto-clean and return.

Title:

- Inline edit.
- Empty title uses generated placeholder from first line or “Untitled note.”

Reveal Export:

- Opens/reveals Markdown snapshot/export through Files/Finder integration.
- Disabled if export not generated; tooltip explains.

More menu:

- Duplicate
- Export Markdown
- Create Version Checkpoint
- Archive
- Move to Trash
- Permanently Delete only if already in Trash
- Copy Note Link

### Block editor toolbar / slash menu

Slash menu sections:

Text:

- Paragraph
- Heading 1
- Heading 2
- Heading 3
- Quote
- Code block

Lists:

- Bullet list
- Numbered list
- Checklist

Data:

- Table
- Attachment
- Image

Zoid:

- Linked entity card
- Linked note card
- Task from selection
- Agent summary block

Canvas:

- Canvas block
- Entity map canvas
- Workflow sketch canvas

### Block interactions

Every block supports:

- Drag handle
- More menu
- Duplicate
- Delete
- Convert block type where safe
- Move up/down

Keyboard:

- Enter creates new block.
- Slash opens insert menu.
- Markdown shortcuts convert blocks.
- Cmd/Ctrl+K opens link/entity command.

### Canvas block finished UI

Collapsed state:

- Canvas title
- Snapshot preview
- Last edited
- Buttons: Open, Expand, Export Snapshot

Expanded state:

- Canvas toolbar: Select, Text, Shape, Connector, Image, Entity Card, Note Card
- Zoom controls
- Minimap if large
- Properties side panel when object selected
- Done / Collapse

Canvas object actions:

- Edit text
- Link entity
- Link note
- Change color/style minimally
- Duplicate
- Delete

### Metadata interactions

Collection picker:

- Search collections.
- Create new collection inline.
- Move note.

Tags editor:

- Search/add tags.
- Create tag.
- Suggested tags grouped under “Suggested.”

Linked entities button:

- Opens Entity Picker modal.

Review state:

- Mark Reviewed
- Mark Needs Review

### Agent panel

Agent tab states:

Off / suggest-only:

- Message: “Agents can suggest changes when you ask.”
- Buttons: Summarize, Suggest Tags, Suggest Links, Create Task

Autonomy enabled:

- Shows active permission level.
- Actions enabled according to level.

Suggestion card:

- Title
- Reason
- Proposed changes
- Diff if body patch
- Buttons: Apply, Edit, Dismiss
- Metadata: model/path, actor, timestamp

Agent actions:

- Summarize Note
- Summarize Selection
- Suggest Tags
- Suggest Entity Links
- Organize Note
- Create Task
- Append Insight
- Draft Edit
- Patch with Diff

### Source tab for imported Apple Notes

Shows:

- Source: Apple Notes
- Original folder
- Imported date
- Last refresh
- Source status: Connected / Changed / Missing / Unavailable
- Buttons: Refresh Source, Compare Update, Open Import History

No button writes to Apple Notes.

### History tab

List items:

- Version timestamp
- Actor
- Operation
- Summary
- Buttons: View, Diff, Restore

Restore flow:

1. Click Restore.
2. Restore Version modal opens.
3. Shows current version will be checkpointed first.
4. Buttons: Restore Version, Cancel

### Activity tab

Timeline:

- User edits
- Agent suggestions/applies
- Import/refresh events
- Sync/file conflicts
- Trash/archive/restore

Each event links to relevant screen.

### Editor states

Saving error:

- Header error: “Could not save.”
- Buttons: Retry Save, Duplicate to Local, Open Activity

Read-only:

- Banner: “This note is read-only because the vault cannot be written.”
- Buttons: Reconnect Vault, Duplicate to Local

Imported source update:

- Banner: “Apple Notes source changed after this Zoid copy was edited.”
- Buttons: Compare Update, Dismiss

Sync conflict:

- Banner: “Synced folder conflict detected.”
- Buttons: Resolve Conflict, View Activity

## 9. Screen 5 — Imported Apple Notes

### Purpose

Connect, import, refresh, review, and reconcile Apple Notes source updates without ever mutating Apple Notes.

### Layout

Header:

- Title: Imported Apple Notes
- Buttons: Connect Apple Notes, Start Import, Refresh Imported Notes, Settings

Status strip:

- Permission state
- Last import
- Last refresh
- Imported count
- Failed count
- Source updates pending

Tabs:

1. Setup
2. Imported Notes
3. Source Updates
4. Failed / Retry
5. History

Right inspector:

- Import run details or selected imported note/source update.

### Setup tab

Cards:

- Connection status
- What will import
- What will not happen

Explicit copy:

- “Zoid imports and refreshes from Apple Notes. Zoid does not write changes back to Apple Notes.”

Buttons:

- Connect Apple Notes
- Start Import
- Open Permission Help

### Import progress state

Progress panel:

- Scanning folders
- Importing notes
- Copying attachments
- Mapping folders to collections
- Indexing imported notes

Buttons:

- Cancel Import
- Run in Background

### Imported Notes tab

Table columns:

- Title
- Apple folder / mapped collection
- Imported date
- Last source refresh
- Review state
- Status

Row actions:

- Open Zoid Note
- Mark Reviewed
- Move Collection
- Retry Refresh
- View Source Metadata

### Source Updates tab

Shows Apple-source changes detected after import.

Columns:

- Source title
- Zoid note
- Change type
- Detected at
- Risk state
- Action

Actions:

- Compare
- Accept into Zoid
- Keep Zoid
- Create Merged Version
- Dismiss

### Compare Imported Update modal/screen

Use full-screen modal or route-level compare view.

Layout:

- Left: Current Zoid note
- Right: Imported Apple-source update
- Top metadata diff
- Bottom action bar

Actions:

- Keep Zoid
- Accept Imported Update into Zoid
- Create Merged Zoid Version
- Dismiss for Later

Footer copy:

- “Apple Notes will not be changed.”

### Failed / Retry tab

List failures:

- Permission issue
- Unsupported attachment
- Missing source
- Parse failure
- Refresh failed

Actions:

- Retry Selected
- Retry All
- Open Permission Help
- Mark Ignored

### History tab

Timeline of import/refresh runs:

- Started by
- Date/time
- Imported count
- Failed count
- Source updates count
- Duration
- Details

## 10. Screen 6 — Collections Management

### Purpose

Manage primary note containers.

### Layout

Header:

- Title: Collections
- Search collections
- New Collection button

Main table:

Columns:

- Name
- Notes count
- Needs review count
- Linked entities
- Default
- Archived
- Updated
- Health

Right inspector:

- Selected collection details.

### Actions

New Collection:

- Opens modal.

Rename:

- Inline or modal from row menu.

Set Default:

- Button in inspector/row menu.

Archive:

- Confirm if collection has notes.

Delete:

- Empty collection only.
- Non-empty collection opens Move Notes First modal.

Open Collection:

- Routes to All Notes filtered by collection.

Bulk Move Notes:

- Opens All Notes filtered by collection with bulk move bar.

### New Collection modal

Fields:

- Collection name
- Optional description
- Set as default checkbox

Buttons:

- Create Collection
- Cancel

Validation:

- Name required.
- Duplicate warning.

### Move Notes First modal

Title: “Move notes before deleting collection”

Fields:

- Destination collection dropdown
- Option: Move to Inbox / Unorganized

Buttons:

- Move Notes
- Cancel

## 11. Screen 7 — Tags Management

### Purpose

Manage cross-cutting labels and prevent tag sprawl.

### Layout

Header:

- Title: Tags
- Search tags
- New Tag button

Status cards:

- Total tags
- Unused tags
- Duplicate candidates
- Aliases

Main table:

Columns:

- Display name
- Normalized key
- Notes count
- Aliases
- Duplicate warning
- Last used
- Status

Right inspector:

- Tag details and actions.

### Actions

Create:

- Opens New Tag modal.

Rename:

- Modal or inline edit.

Merge:

- Opens Merge Tags modal.

Add Alias:

- Adds alias to canonical tag.

Archive:

- Hides from default suggestions.

Delete:

- Removes tag from notes; does not delete notes.

Open tag:

- Routes to All Notes filtered by tag.

### Merge Tags modal

Fields:

- Source tags
- Target canonical tag
- Affected note count
- Preview affected notes

Buttons:

- Merge Tags
- Cancel

Warning:

- “This changes tags on N notes.”

## 12. Screen 8 — Notes Settings

### Purpose

All operational controls for vaults, sync, Apple Notes import/refresh, history, search, agents, privacy, and recovery.

### Layout

Settings uses two-column layout:

Left section nav:

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

Right settings panel for selected section.

### Vault section

Shows:

- Current vault path
- Vault mode
- Permission state
- Size
- Last verified

Actions:

- Move Current Vault
- Open Existing Vault
- Choose External Folder
- Use iCloud / Synced Folder
- Reveal Vault in Finder
- Verify Vault

Move Current Vault modal:

- Explains copy/move.
- Destination picker.
- Validation.
- Buttons: Move Vault, Cancel

Open Existing Vault modal:

- Folder picker.
- Validation.
- Buttons: Open Existing Vault, Cancel

### iCloud / Local Sync section

Shows:

- Sync folder path
- Provider assumption: generic filesystem sync
- Last local write
- Pending conflicts
- Cache/index state

Actions:

- Choose Synced Folder
- Rebuild Local Cache
- Open Conflict Resolver
- Reveal Sync Folder

Warning copy:

- “Zoid syncs per-note structured files. SQLite is a rebuildable local cache, not the synced source of truth.”

### Apple Notes Import / Refresh section

Shows:

- Permission state
- Last import
- Last refresh
- Imported notes count
- Source updates count
- Failed items

Actions:

- Connect Apple Notes
- Start Import
- Refresh Imported Notes
- Open Imported Apple Notes
- Clear Import Errors

Permanent copy:

- “Zoid does not write changes back to Apple Notes.”

### History & Versions section

Controls:

- Version retention period
- Max storage size
- Named checkpoints toggle
- Pre-agent-change checkpoint toggle
- Pre-source-update checkpoint toggle

Actions:

- View Activity
- Prune Old Versions

### Trash & Archive section

Controls:

- Trash retention period
- Auto-empty trash toggle
- Archive visibility setting

Actions:

- Open Trash
- Empty Trash

Empty Trash requires destructive confirmation.

### Search & Indexes section

Shows:

- Text index status
- Last rebuild
- Indexed notes count
- Attachment text extraction status

Actions:

- Rebuild Text Index
- Rebuild Attachment Index
- Pause Indexing

### Semantic Search section

Shows:

- Local embeddings availability
- Semantic index status
- Indexed note count
- Remote model policy state

Actions:

- Build Semantic Index
- Clear Semantic Index
- Configure Provider Policy

Default:

- Do not send full notes remote by default.

### Agent Note Operations section

Master toggle:

- Off by default.

Permission levels:

1. Suggest only
2. Apply metadata
3. Append only
4. Patch body with diff
5. Full edit

Per-action toggles:

- Create notes
- Summarize
- Suggest links
- Apply tags
- Append content
- Patch body
- Create tasks

Buttons:

- Save Agent Permissions
- Reset to Safe Defaults
- View Agent Activity

Turning on risky levels opens Agent Permission Warning modal.

### Privacy section

Shows:

- Local-first explanation
- What leaves device
- Remote model policy
- Apple Notes source behavior
- Finder/export behavior

Buttons:

- Configure Model Privacy
- View Audit Activity

### Export / Recovery section

Actions:

- Export all Markdown snapshots
- Export selected collections
- Verify vault integrity
- Rebuild from per-note files
- Download recovery bundle
- Reveal export folder

## 13. Screen 9 — Notes Activity / History

### Purpose

A user-facing audit trail and recovery center.

### Layout

Header:

- Title: Activity / History
- Search activity
- Export Activity button

Filter bar:

- Actor
- Operation
- Source
- Result
- Linked entity
- Date
- Has version

Main event list:

Columns:

- Time
- Actor
- Operation
- Target
- Source
- Result
- Version

Right inspector:

- Event details and actions.

### Event types

- Note created
- Note edited
- Version checkpoint
- Agent suggestion generated
- Agent change applied
- Import started/completed/failed
- Source update detected/resolved
- Tag/collection changed
- Moved to Trash
- Restored
- Archived
- Permanent delete
- Sync conflict detected/resolved

### Event detail actions

- Open Note
- View Diff
- Restore Version
- Open Agent Run
- Open Import Issue
- Open Conflict Resolver
- Copy Event ID

### Empty state

- “No activity yet.”
- Shows after first-run before notes exist.

## 14. Modals and Drawers

### Folder Permission Prompt

Title: “Allow Zoid to access this folder”

Copy:

- Explains Notes vault needs read/write access.
- Mentions shared Files native permission system.

Buttons:

- Choose Folder
- Cancel
- Use Default Vault

### New Note modal/draft behavior

Usually no modal. New Note opens editor immediately.

If created from source module, use a small preflight drawer:

Fields:

- Title suggestion
- Collection
- Link source entity checkbox
- Start blank / Summarize source into note

Buttons:

- Create Note
- Cancel

### New Collection modal

Defined in Collections screen.

### Entity Picker modal

Layout:

- Search input
- Entity type tabs/chips
- Results list
- Selected entities tray

Entity types:

- Projects
- Agent runs
- Files/folders
- Content/assets
- Browser captures
- Repos
- Tasks
- Chats
- Products/business entities
- Calendar events
- Notes

Buttons:

- Link Selected
- Cancel

Unavailable entity types are hidden by default.

### Agent Permission Warning modal

Trigger:

- User enables agent writes above suggest-only.

Title: “Allow agents to change notes?”

Copy:

- Explains actions are logged and versioned.
- Explains risky levels.

Controls:

- Permission level selector
- Checkbox: “Create a version before agent writes” locked on
- Checkbox: “Show body diffs before patching” locked on unless full edit chosen

Buttons:

- Enable Permissions
- Keep Suggest Only

### Agent Suggestion Review card

Not a modal by default; appears inline in editor inspector or dashboard review queue.

Sections:

- Proposed change summary
- Reason
- Diff/metadata changes
- Risk level
- Buttons: Apply, Edit, Dismiss

### Move to Trash modal

Title: “Move note to Trash?”

Copy:

- “You can restore it before trash retention expires.”

Buttons:

- Move to Trash
- Cancel

### Permanent Delete modal

Title: “Permanently delete this note?”

Copy:

- “This deletes note content and unshared attachments according to retention policy. This cannot be undone.”

Required checkbox:

- “I understand this permanently deletes the note.”

Buttons:

- Permanently Delete
- Cancel

### Restore Version modal

Title: “Restore this version?”

Copy:

- “Zoid will checkpoint the current version first.”

Buttons:

- Restore Version
- Cancel

### Compare Imported Update modal

Defined in Imported Apple Notes screen.

### Synced Folder Conflict Resolver

Layout:

- Left: Local current version
- Right: Synced-folder conflicting version
- Metadata diff top
- Action footer

Buttons:

- Keep Local
- Keep Synced Version
- Create Merged Version
- Cancel

### Index Rebuild Progress drawer

Shows:

- Current phase
- Progress bar
- Indexed count
- Errors count

Buttons:

- Run in Background
- Pause
- View Search Settings

## 15. Mobile Screen Designs

## 15.1 Mobile First Launch / Vault Connect

Layout:

- Title: “Connect your Zoid Notes vault”
- Status card: Vault found / Not found / Permission needed
- Primary action: Connect iCloud Vault
- Secondary: Capture without vault
- Help link: Set up desktop synced vault

Flow:

- Vault found -> Notes List.
- No vault -> capture-only holding area available.
- Permission missing -> permission help.

## 15.2 Mobile Capture

Home capture layout:

- Quick Note large button
- Voice Note button
- Photo Note button
- Search bar
- Recent captures list
- Sync status chip

Quick Note:

- Opens simple editor.
- Saves to Inbox / Unorganized.

Voice Note:

- Recording screen.
- Buttons: Record/Pause/Stop/Save
- After save: audio attachment + optional local transcription.

Photo Note:

- Camera/library picker.
- Adds attachment.
- Optional caption.

## 15.3 Mobile Notes List / Search

Layout:

- Search bar
- Filter chips: Inbox, Recent, Tags, Collections, Imported
- Notes list cards
- Floating capture button
- Sync status chip

Card content:

- Title
- Snippet
- Collection/tags
- Updated
- Status badge

Actions:

- Tap -> Read/Edit
- Long press -> quick actions: Tag, Move, Mark Reviewed, Archive

## 15.4 Mobile Note Read / Light Edit

Layout:

- Header: back, title, save/sync state, more
- Metadata chips: collection, tags
- Simple block editor
- Attachments
- Read-only canvas/table previews

Editable:

- Title
- Paragraphs
- Headings
- Bullets
- Checklists
- Quotes
- Simple attachments
- Collection/tags

View-only:

- Canvas deep editing
- Complex tables
- Complex entity cards

More menu:

- Mark Reviewed
- Archive
- Move to Trash
- Copy Link
- Open Sync Status

## 15.5 Mobile Sync Status

Layout:

- Vault state card
- Last sync
- Pending items
- Conflicts/errors
- Capture-only holding area count

Actions:

- Retry Sync
- Connect Vault
- View Setup Help
- Keep Capture-Only

Complex conflicts direct to desktop.

## 16. Finished Product End-to-End Flow Map

### First note

1. Notes nav -> First-Run Vault Setup.
2. Use Default Vault.
3. Dashboard empty state.
4. New Note.
5. Editor opens draft.
6. User writes.
7. Autosave + index + activity.
8. Dashboard recent updates.

### Import and review Apple Notes

1. Dashboard -> Import Apple Notes.
2. Connect Apple Notes.
3. Start Import.
4. Progress visible.
5. Imported Notes tab shows imported items.
6. Needs Review card updates.
7. User bulk organizes imported notes.
8. Later source updates go to Source Updates tab.
9. Compare/merge affects Zoid only.

### Agent-assisted organization

1. Dashboard -> Needs Review.
2. Select imported/unlinked notes.
3. Click Suggest Links or Summarize.
4. Agent review cards appear.
5. User applies selected suggestions.
6. Versions/activity are recorded.

### Knowledge cleanup

1. Dashboard -> Needs Review.
2. All Notes opens filtered.
3. Bulk tag/move/mark reviewed.
4. Tags screen merges duplicates.
5. Collections screen resolves stale/empty collections.
6. Dashboard health clears.

### Mobile capture to desktop

1. Mobile opens connected vault.
2. User creates voice/photo/quick note.
3. Note saves offline to Inbox / Unorganized.
4. Sync reaches desktop vault.
5. Desktop Dashboard shows Needs Review.
6. User organizes and links.

### Restore from mistake

1. User moves note to Trash.
2. Activity logs event.
3. User opens Trash filter or Activity.
4. Restore.
5. Note returns to prior collection or Inbox if prior collection missing.

## 17. Interaction State Requirements

Every button/action must define these states:

- Default
- Hover
- Focus
- Active/pressed
- Disabled
- Loading
- Error

Every screen must define:

- Loading state
- Empty state
- Normal state
- Partial/degraded state
- Error state

Every write action must show:

- Immediate feedback
- Activity log entry where meaningful
- Version checkpoint where content changes
- Undo/restore path where feasible

## 18. Design QA Checklist

Before implementation/design signoff:

- All screens have primary actions and empty states.
- No Apple Notes write-back action exists anywhere.
- Destructive actions are confirm-gated.
- Agent writes are permission-gated and versioned.
- Right inspector has useful content for each selection type.
- Dashboard surfaces real attention states, not fake metrics.
- All Notes supports search/filter/bulk work.
- Editor supports block/canvas/source/history/agent workflows.
- Imported Apple Notes makes one-way behavior explicit.
- Settings exposes all operational controls.
- Activity can recover/version/trace actions.
- Mobile only promises lightweight edit.
- Finder/Files integration is visible through vault/export/reveal actions.
