# Stitch AI Prompt: Zoid Files Workspace Screens

Use this prompt in Stitch AI to design a complete screen set for Zoid's Files Workspace.

## Product context
Zoid is a macOS desktop AI OS for orchestrating agents, projects, content, automations, repos, browser captures, tasks, chats, logs, and review workflows. The Files module becomes the local filesystem layer inside Zoid: a Finder-familiar workspace that uses native macOS permission, operates on real on-device files, watches changes made outside Zoid, indexes supported content locally, and gives agents controlled file-operation tools when the user explicitly enables agent file access.

This is not a fake file browser. Design for real local files, truthful permission states, local indexing, audit-grade activity, snapshots, and safety.

Explicit MVP exclusion: do not design Finder extensions, Finder context menus, or macOS share sheet integration as MVP requirements.

## Existing workspace references to preserve
The design must feel like an extension of these existing Zoid workspace screen models.

### Files Workspace reference
Existing planned screens include:
- Files Dashboard: header, summary cards, folder shortcuts, recent files, content assets, browser captures, repo folders, file events, right inspector.
- File Explorer: path bar, folder tree, file list, preview pane, right inspector, basic actions.
- File/Folder Detail: name/type/path header, metadata, preview, linked entities, repo-aware status, events, actions.

### Agents Workspace relationship
Files must link to agent runs and expose agent activity without creating a separate agent system.
Reuse these ideas:
- Agent run status chips and linked run panels.
- Permission preview language for agent file access.
- Actor labeling: user, agent, system.
- Linked entities and event timelines.

### Content Workspace relationship
Files must surface content assets and browser captures where relevant.
Reuse these ideas:
- Asset cards with name, type, path, linked content, last used.
- Right inspector pattern.
- Publishing/evidence style event history where files are linked to content work.

### Automations/Projects/Repos relationships
Files can link to projects, repos, automations/runs, tasks, and chats. Show these as relationship chips and inspector sections, not as separate competing file concepts.

## Visual style direction
Use Zoid's Apple-inspired design system:
- Clean, low-chrome desktop macOS interface.
- Primary action color: Action Blue #0066cc for links, selected states, focus rings, and primary actions only.
- Surfaces: white #ffffff, parchment #f5f5f7, near-black tiles #272729/#2a2a2c/#252527 for high-emphasis preview/status sections.
- Typography: SF Pro Display/SF Pro Text or system font equivalent. Headlines 600 weight with tight tracking; body 17px regular; captions 14px.
- Geometry: pill CTAs and filter chips, 18px utility cards, 8px compact utility controls, 1px hairline borders, no decorative gradients.
- Avoid heavy SaaS chrome. Prefer spacious sections, quiet hairlines, precise status chips, native-feeling file tables, and right inspectors.
- Do not use card shadows except if representing actual preview imagery. Use surface alternation and hairlines for hierarchy.
- Desktop-first macOS app at 1440px wide; include responsive adaptations where relevant.

## UX principles
1. Finder-familiar, Zoid-enhanced.
   - Users should immediately understand sidebar, breadcrumbs, file list, preview, open, reveal, rename, move, copy, duplicate, delete-to-Trash.
   - Zoid adds relationship chips, index/snapshot state, agent activity, repo status, and audit trail.

2. Truthful native permission.
   - If permission is not granted, show an honest blocked state and explain what Zoid needs.
   - Do not show fake folders/files as if connected.
   - Re-request access and troubleshooting should be visible.

3. Operational command center first.
   - Dashboard is not a generic Finder clone.
   - It should show permission/indexing health, pinned folders, recent/active files, agent-touched files, linked assets, repo folders, file events, and suggested actions.
   - Explorer handles deep browsing.

4. Local-only privacy is visible.
   - Index and snapshots are local-only by default.
   - Users can pause indexing, exclude folders, clear index, clear snapshots, and manage retention.
   - Raw indexed content should not look like it is synced to cloud.

5. Agent file access is separate from macOS file permission.
   - macOS permission enables Files Workspace access.
   - Agent file access is a separate off-by-default setting.
   - When enabled, show clear warning copy: agents can operate on files through logged tools.

6. Every meaningful action has evidence.
   - Activity log must show user/agent/system/external actor, operation, path, result, timestamp, linked entity, snapshot availability, and undo/restore availability.

7. Safety without hiding power.
   - Deletes go to Trash.
   - Eligible edits create pre-edit snapshots.
   - Undo/restore affordances appear where feasible.
   - Unsupported or skipped snapshot states must be explicit.

## Core data and status model to reflect visually
Show these entities and statuses throughout the UI:
- Permission status: not requested, requested, granted, denied, revoked, partial/unavailable, troubleshooting required.
- Index status: idle, indexing, paused, excluded, stale, failed extraction, unsupported, complete.
- Snapshot status: eligible, snapshot available, not eligible, skipped due to size/type, restore available, retention warning.
- File activity source: user action, agent action, system/indexer, external Finder/local filesystem change.
- File operation result: success, failed, blocked, skipped, undoable, restored.
- Agent access status: off by default, enabled, blocked by permission, blocked by setting, logged operation.
- Entity links: project, agent run, content asset, browser capture, repo, task, chat.
- Repo status where applicable: clean, modified, untracked, ignored, conflict, branch/repo badge.
- Privacy/storage state: local-only, encrypted if feasible/available, storage limit warning, clearable.

## Screens to design
Design every screen below. Include desktop layout and responsive behavior. Use realistic sample data from a macOS/Zoid environment, but make it clear these are example local files.

### 01. Files Dashboard
Purpose: Main Files command center.

Must include:
- Top workspace header: Search Files, Add Folder / Grant Access, New Folder, Settings.
- Permission/indexing health banner or card:
  - macOS file permission state.
  - Indexing state with progress/pause/resume.
  - Agent file access off/on state.
  - Local-only privacy indicator.
- Summary cards:
  - Pinned folders.
  - Zoid folder.
  - Recent files.
  - Linked files.
  - Agent-touched files.
  - Index health.
- Folder shortcuts: Projects, Downloads, Desktop, Documents, Repos, Content Assets, Browser Captures, custom pinned folders.
- Recent/active files list with file kind, path, modified time, linked entity, index/snapshot badges.
- Content assets panel showing files linked to Content Workspace.
- Browser captures panel showing saved captures/downloads linked to browser sessions.
- Repo folders panel with repo status badges.
- File events panel showing recent activity log entries.
- Suggested actions panel: grant permission, resume indexing, review agent-touched files, clear failed extraction, open settings.
- Right inspector for selected folder/file/event with metadata, links, index state, activity, and actions.

States:
- Empty/no permission: explain Files needs macOS permission; CTA “Grant File Access”.
- Permission denied/revoked: show troubleshooting and re-request access.
- Loading/skeleton while permission/index state loads.
- Indexing in progress with progress, queue count, paused/resume action.
- Healthy: all watched folders current.
- Warning: failed extractions, storage limit, snapshots retention warning.
- Agent access off: visible but non-alarming status.

### 02. File Explorer
Purpose: Finder-familiar deep browsing and basic file operations.

Must include:
- Path bar with back, forward, up, breadcrumbs, search, new folder, more menu.
- Left sidebar/tree with pinned folders, common folders, repos, content assets, browser captures, recent locations.
- Main file list/table with columns:
  - Name.
  - Kind.
  - Size.
  - Modified.
  - Linked entity.
  - Repo status.
  - Index status.
  - Snapshot/activity badge.
- Optional grid/list toggle if appropriate.
- Preview pane for supported files and an unsupported-preview state.
- Right inspector with metadata, path, linked entities, repo status, index status, snapshot state, recent events, and actions.
- Basic Finder actions: Open, Preview, Rename, Move, Copy, Duplicate, Delete to Trash, New Folder, Reveal in Finder, Copy Path.
- Search/filter behavior for current folder and indexed content.

States:
- Folder loading.
- Empty folder.
- Permission missing for location.
- File missing/moved externally.
- External change detected.
- Operation failed/locked file.
- Delete moved to Trash success.
- Snapshot created before eligible edit.
- Not snapshotted due to type/size.

### 03. File / Folder Detail or Inspector State
Purpose: Deep detail state for selected file/folder, usable as full page or right inspector.

Must include:
- Header: name, type, path, quick actions: Open, Reveal, Copy Path, Rename, Delete to Trash.
- Metadata section: size, modified/created dates, extension/kind, permissions, owner if available, index status, snapshot state, repo status.
- Preview section:
  - Text/code preview.
  - PDF/DOCX extracted text summary state.
  - Image/video/audio metadata preview.
  - Unsupported preview state.
- Linked entities section:
  - Projects.
  - Agent runs.
  - Content assets.
  - Browser captures.
  - Repos.
  - Tasks.
  - Chats.
- Activity/events timeline for that file/folder.
- Snapshot/restore area where available.
- External change notice if file changed outside Zoid.
- Privacy note if indexed locally.

States:
- No links yet.
- Suggested link available.
- Agent touched file.
- Snapshot restore available.
- File moved/deleted externally.
- Index extraction failed.

### 04. Files Settings / Permissions
Purpose: Control permissions, indexing, exclusions, snapshots, logs, and agent access.

Must include:
- macOS file permission card:
  - Status.
  - Grant/re-request access.
  - Troubleshooting instructions for denied/revoked permission.
- Indexing controls:
  - Current status.
  - Pause/resume.
  - Reindex current roots if included.
  - Clear index with confirmation.
- Exclusions editor:
  - System folders.
  - App bundles.
  - Caches.
  - node_modules.
  - .git internals.
  - build outputs.
  - virtualenvs.
  - vendor/dependency dirs.
  - .ssh/.gnupg.
  - keychains.
  - browser profiles.
  - mail stores.
  - Photos libraries unless opted in.
  - file size limits.
- Snapshot controls:
  - Retention.
  - Storage limit.
  - Clear snapshots with confirmation.
  - Explanation: only eligible text/code/docs/config files are snapshotted.
- Activity log controls:
  - Retention.
  - Clear/export if appropriate.
- Privacy panel:
  - Index and snapshots are local-only by default.
  - Encryption/secure-storage status if available.
  - Raw indexed content is not sent to remote models unless used in a user/agent task.
- Agent file-access toggle:
  - Off by default.
  - Clear warning: “Allow Zoid agents to operate on files.”
  - Explain agents can list/read/search/create/write text/patch/rename/move/copy/delete-to-Trash/open/reveal/stat through logged tools.

States:
- Permission not granted.
- Permission granted but agent access off.
- Agent access enabled.
- Indexing paused.
- Storage limit warning.
- Clear index confirmation.
- Clear snapshots confirmation.

### 05. Activity Log Drawer / Modal
Purpose: Audit-grade, filterable history of file operations and external changes.

Must include:
- Filter bar: actor, operation, result, source, linked entity, date, path.
- Event list rows showing:
  - Actor: user, agent, system, external.
  - Operation: create, read, write, patch, rename, move, copy, delete-to-Trash, open, reveal, index, snapshot, restore, external modify.
  - Path and secondary path where relevant.
  - Timestamp.
  - Result/status.
  - Linked project/agent run/entity chips.
  - Snapshot availability.
  - Undo/restore action where possible.
- Detail panel for selected event with full metadata and error details if failed.
- Clear distinction between Zoid operations and external Finder/local filesystem changes.

States:
- Empty log.
- Filtered empty.
- Operation failed.
- Undo available.
- Restore available.
- Snapshot skipped.
- External change detected.

### 06. Permission / Empty / Indexing State Board
Purpose: State board for all key Files module states.

Must include variants for:
- First-run no permission.
- Permission denied.
- Permission revoked after previously granted.
- Folder empty.
- Folder unavailable/moved.
- Indexing in progress.
- Indexing paused.
- Extraction failed.
- File excluded.
- Storage limit warning.
- Agent access off.
- Agent access enabled.
- Operation blocked.
- Delete moved to Trash.
- Snapshot created.
- Snapshot skipped.
- Restore success.

## Cross-screen components to design
Create reusable components with variants:
- Files workspace header.
- Permission health card.
- Indexing status chip/progress card.
- Agent file-access status chip.
- Local-only privacy badge.
- Folder shortcut card.
- File row/list item.
- File grid tile.
- Breadcrumb/path bar.
- Folder tree/sidebar item.
- File preview pane.
- Right inspector shell.
- Linked entity chips.
- Repo status badge.
- Snapshot status badge.
- Activity event row.
- Activity log filter bar.
- Settings section card.
- Exclusion rule row/chip.
- Dangerous-action confirmation dialog.
- Empty/loading/error/blocked/success panels.

## Required interaction notes
Include annotations for:
- First-run permission flow.
- Permission denied/re-request flow.
- Browse/open/reveal flow.
- Rename/move/copy/duplicate/delete-to-Trash flow.
- External Finder change sync.
- Index pause/resume and failed extraction handling.
- Snapshot before eligible edit and restore from snapshot.
- Agent file-access toggle enablement warning.
- Agent operation appears in activity log.
- Clear index and clear snapshots confirmation.

## Accessibility and interaction requirements
- Minimum target size: 44x44px for interactive controls.
- Keyboard navigable tables, trees, breadcrumbs, menus, modals, drawers, and settings controls.
- Clear focus ring using Action Blue.
- Status cannot rely on color alone; use text labels/icons.
- File tables support sorting and row selection.
- Dangerous actions require confirmation and explain impact.
- Delete copy must say “Move to Trash”, not “permanently delete”.
- Permission and privacy copy must be plain and direct.

## Desktop and responsive requirements
Design desktop first for a macOS app at 1440px wide.
Also provide responsive adaptations for:
- 1024px small desktop/tablet landscape: collapse right inspector into a drawer.
- 736-833px tablet portrait: sidebar collapses, list-first layout, inspector as drill-in.
- 420-640px phone: single-column views, filters in sheets, inspector as full-screen drill-in, sticky action bar where needed.

## Exact deliverables expected from Stitch
Stitch AI should output:
1. High-fidelity desktop screens for Files Dashboard, File Explorer, File/Folder Detail or Inspector, Files Settings/Permissions, Activity Log drawer/modal, and state board.
2. Responsive variants for Dashboard, Explorer, Detail/Inspector, Settings, and Activity Log.
3. Component library page with all cross-screen components and state variants.
4. State board covering permission, empty, loading, indexing, paused, excluded, failed, blocked, operation success, operation failure, snapshot, restore, and agent-access states.
5. Interaction notes for permission, browsing/actions, external sync, indexing, snapshots, activity log, and agent access toggle.
6. Visual token usage summary matching the Apple-inspired Zoid design style.
7. Handoff annotations naming each screen, primary actions, secondary actions, data shown, and linked workspace relationships.

## Sample copy/data to use
Use these examples in mockups:
- App: Zoid.
- Workspace: Files.
- Pinned folders: Zoid, Projects, Downloads, Desktop, Repos, Content Assets, Browser Captures.
- Example project: MaVoid Operating System.
- Example repo folder: `~/Zoid` with modified/untracked repo badges.
- Example files:
  - `campaign-brief.md` linked to Content Workspace and MaVoid project.
  - `agent-output-summary.md` linked to Agent Run.
  - `browser-capture-homepage.png` linked to Browser Capture.
  - `zoid-roadmap.docx` indexed.
  - `launch-notes.pdf` indexed.
  - `promo-video.mov` metadata-only.
- Example activity events:
  - User renamed `draft.md` to `campaign-brief.md`.
  - Agent patched `launch-notes.md`; snapshot available.
  - External Finder change modified `roadmap.md`.
  - System indexed `zoid-roadmap.docx`.
  - Delete moved `old-export.csv` to Trash.
- Agent access setting default: Off.
- Warning copy: “Allow Zoid agents to operate on files. Agent file operations are logged. Eligible text edits create snapshots before changes.”
- Privacy copy: “Index and snapshots stay local on this Mac by default.”

## Important design reminders
- Do not show fake connected file data before permission.
- Do not design Finder extensions, Finder context menus, or share sheet as MVP.
- Do not imply cloud sync for index or snapshots.
- Do not hide agent file access inside macOS permission; it is a separate setting and off by default.
- Do not call delete “permanent delete” in MVP; use Move to Trash.
- Do not show raw secrets or sensitive file contents in logs/settings examples.
- Do not make Files a separate agent runner; show linked Agent Runs from the Agents Workspace model.
- Do not make the Dashboard a generic Finder clone; make it an operational command center.
