# PRD: Zoid Files Module

Date: 2026-06-06
Product area: Files Workspace, Agents Workspace, Projects, Content Workspace, Browser Captures, Repos, Tasks, Chats
Source discovery: `/Users/ziadnasreldin/brainstorms/2026-06-06-zoid-files-module.md`
Screen reference: `../../designer-screen-reference/09-files-workspace.md`

## 1. Overview
Zoid Files is the AI OS file layer for the macOS Zoid app. It is a Finder-like local file workspace that integrates with the on-device filesystem after native macOS permission, lets users browse and operate on real local files, watches external Finder changes, indexes supported content locally, and exposes a logged agent file-operation capability behind an off-by-default setting.

The feature is not UI-only. The MVP must work in the packaged Tauri app against real local files with truthful permission, indexing, activity, and safety states. Finder extensions, Finder context menus, and share sheet integrations are explicitly out of MVP.

## 2. Goals and Non-Goals
### Goals
- Provide a Finder-familiar Files Workspace inside Zoid.
- Request native macOS filesystem permission before accessing local files.
- Support full in-app local file browsing and basic Finder actions against real files.
- Open files with default apps and reveal selected items in Finder.
- Watch local filesystem changes and reflect changes made outside Zoid.
- Index file metadata and supported content locally.
- Maintain a user-facing audit-grade file activity log.
- Create pre-edit snapshots for eligible text/code/docs/config files.
- Let Zoid agents operate on files through an approved backend tool surface when the user enables agent file access.
- Link files/folders to Zoid projects, agent runs, content assets, browser captures, repos, tasks, and chats.
- Keep index and snapshots local-only by default.

### Non-Goals
- UI-only fake file browser.
- Cloud sync of index, snapshots, or raw file content.
- Finder extension, Finder context menu, or share sheet integration in MVP.
- Arbitrary binary editing by agents in MVP.
- Sending raw indexed content to remote models unless the user/agent task explicitly uses the file.
- Permanent delete as the normal delete path; deletes should route to macOS Trash.

## 3. Key Decisions
- Product boundary: full local file operations layer, not only a browser or index.
- Permission model: full Finder-like access after native macOS permission.
- Agent authority: when agent file access is enabled, agents can operate like the user through the approved backend tool surface.
- Agent access default: off by default, separately enabled after macOS file permission.
- UI action surface: MVP UI starts with basic Finder actions only.
- Backend capability: agents get a broader logged text-oriented file tool surface in MVP.
- Safety model: activity log, delete-to-Trash, undo where feasible, pre-edit snapshots for eligible files.
- Snapshot policy: text/code/docs/config-like files under size/type rules only.
- Indexing: content indexing for everything feasible, with MVP extraction scoped to text/code/Markdown/PDF/DOCX plus media metadata.
- Local storage: index and snapshots live in Zoid app data, local-only.
- Privacy: encryption if feasible, plus clear local-only controls and exclusions.

## 4. Users
- Founder/operator: uses Files as the local command center for project files and agent-accessible work.
- Developer/operator: browses repos, inspects metadata, opens/reveals files, and uses agents to read or patch text files.
- Content operator: manages local content assets, browser captures, linked campaign files, and exported evidence.
- Reviewer/auditor: checks activity logs, snapshots, external changes, and agent file operations.

## 5. Core Entities
- FileRecord: path, name, kind, size, modified time, permissions, indexed state, snapshot state, repo state, external change state.
- FolderRecord: path, children summary, pinned state, indexed state, watch state.
- FileIndexEntry: metadata, extracted text where supported, media metadata, parse status, last indexed time, excluded reason.
- FileSnapshot: pre-edit copy for eligible files, source path, actor, operation, timestamp, size, retention state, restore metadata.
- FileActivityEvent: actor, operation, path(s), timestamp, result, source, linked entities, snapshot availability, undo/restore action.
- FileEntityLink: file/folder relationship to project, agent run, content asset, browser capture, repo, task, or chat.
- FilesSettings: permission state, indexing state, exclusions, retention limits, agent access toggle, clear controls.

## 6. Workspace Architecture
### Files Dashboard
Operational command center, not a generic Finder home. Shows permission/indexing health, pinned folders, recent/active files, agent-touched files, linked assets, repo folders, file events, suggested actions, and right inspector.

### File Explorer
Finder-familiar, Zoid-enhanced browsing surface with sidebar/tree, breadcrumb/path bar, sortable file table, optional grid toggle, preview pane, right inspector, command/search affordances, and badges for linked entity, repo status, indexed/snapshot state, and agent activity.

### File/Folder Detail or Inspector
Detail state for selected file/folder showing metadata, preview, path, linked entities, repo-aware status, events, snapshot/restore state, and actions.

### Files Settings and Permissions
Controls for native permission, indexing pause/resume, exclusions, retention, clearing local index/snapshots, activity log retention, privacy explanation, and agent file-access toggle.

### Activity Log
User-facing audit-grade log. MVP can be a dashboard panel plus filterable drawer/modal; full dedicated page can come later.

## 7. Finder and Local Filesystem Integration
MVP must include:
- Native macOS permission request and state handling.
- Real local file/folder listing after permission.
- Basic Finder actions in UI.
- Open with default app.
- Reveal in Finder.
- Local filesystem watching.
- External-change sync when files are changed outside Zoid.
- Truthful permission denied, indexing disabled, excluded, and failed states.

Out of MVP:
- Finder extension.
- Finder context menu.
- macOS share sheet integration.

## 8. MVP UI Actions
The UI should expose basic Finder actions first:
- Browse folders.
- Open file.
- Reveal in Finder.
- Preview where supported.
- Search.
- Create folder/file where appropriate.
- Rename.
- Move.
- Copy.
- Duplicate.
- Delete to Trash.
- Copy path.

Advanced UI actions like link/unlink, ask-agent, reindex, and batch operations can follow after the Finder-like foundation is stable, even if backend capabilities already exist.

## 9. Agent File Tools
Agent file access is controlled by a global Files Settings toggle in MVP.

Default state:
- macOS Files permission can be granted while agent file access remains off.
- Agent file access is off by default.
- User separately enables “Allow Zoid agents to operate on files” with clear warning copy.

If off:
- Agents can ask the user to pick files.
- Agents cannot autonomously read/write through file tools.

If on:
- Agents can use the approved MVP tool surface.
- Every operation is logged.

MVP tool surface:
- `list`
- `read`
- `search`
- `create_file`
- `create_folder`
- `write_text`
- `patch_text`
- `rename`
- `move`
- `copy`
- `delete_to_trash`
- `open`
- `reveal`
- `stat`

Rules:
- Write/patch creates snapshots when eligible.
- Binary writes are blocked unless explicitly supported later.
- Deletes route to macOS Trash.
- Operations record actor, path, linked project/run, result, and snapshot/undo availability.

## 10. Indexing and Extraction
Indexing goal: full content indexing for everything feasible, bounded by performance, privacy, and supported extraction.

MVP extraction scope:
- Extract content from plain text, code, Markdown, PDF, and DOCX.
- Extract metadata from images, videos, and audio.
- Defer image OCR and audio/video transcription unless later prioritized.

Index must show:
- indexed
- indexing
- excluded
- failed extraction
- stale/reindex needed
- paused

Default exclusions:
- System folders.
- App bundles.
- Caches.
- `node_modules`.
- `.git` internals.
- Build outputs.
- Virtualenvs.
- Dependency/vendor dirs.
- Hidden sensitive dirs like `.ssh` and `.gnupg`.
- Keychains.
- Browser profiles.
- Mail stores.
- Photos libraries unless explicitly opted in.
- Files above limits.

## 11. Safety, Snapshots, Undo, and Recovery
Delete behavior:
- Delete goes to macOS Trash.
- Permanent delete is not the normal MVP path.

Snapshots:
- Create pre-edit snapshots for eligible text/code/docs/config-like files.
- Skip giant binaries, videos, archives, app bundles, `node_modules`, `.git` internals, caches, and system folders.
- If a file is not snapshotted, the activity log must make that visible.

Undo/restore:
- Support undo where feasible for move, rename, and create.
- Support restore from snapshot for eligible edits.
- Record before/after metadata.

## 12. Activity Log Requirements
Activity log is user-facing and audit-grade. It must show:
- Actor: user, agent, or system.
- Operation.
- Path and secondary path where relevant.
- Timestamp.
- Linked project/run/entity if any.
- Result: success, failed, skipped, blocked, external.
- Snapshot availability.
- Undo/restore action where possible.
- Whether change came from Zoid or external Finder/local filesystem change.

## 13. Entity Linking
Files/folders can link to:
- Projects.
- Agent runs.
- Content assets.
- Browser captures.
- Repos.
- Tasks.
- Chats.

A file can have multiple links. The right inspector should show relationships.

Linking behavior:
- Manual link/unlink eventually.
- Automatic links where source is deterministic.
- Suggestions where uncertain.

MVP may prioritize deterministic automatic links and visible relationships before exposing all advanced link management UI.

## 14. Files Settings MVP
Settings must include:
- Permission status and re-request access.
- Indexing status, pause, resume.
- Excluded folders/patterns.
- Snapshot retention and storage limit.
- Clear index.
- Clear snapshots.
- Activity log retention.
- Privacy note about local-only index.
- Global agent file-access toggle.

Destructive settings actions should require confirmation.

## 15. Privacy and Security
- Index and snapshots stay local-only by default.
- Use encryption if feasible in the current macOS/Tauri stack.
- Use OS-level app sandbox/keychain-protected storage where available.
- Provide settings to clear index and snapshots.
- Provide sensitive folder exclusions.
- Do not expose raw indexed content to remote models unless user/agent task explicitly uses the file.
- Raw secrets should never appear in prompts, logs, snapshots, events, exports, or UI copy.

## 16. Screens to Design
- Files Dashboard.
- File Explorer.
- File/Folder Detail or Inspector State.
- Files Settings/Permissions.
- Activity Log panel plus filterable drawer/modal.
- Empty state.
- Permission-denied state.
- Indexing/paused/failed states.

## 17. Acceptance Criteria
- Packaged Tauri app requests native macOS file permission before local access.
- User can browse real local files and folders after permission.
- User can perform basic Finder actions against real files.
- User can open files with default apps and reveal files in Finder.
- Zoid reflects file changes made outside the app.
- Zoid indexes supported local content and shows indexing health.
- Zoid respects default exclusions and file limits.
- Index and snapshots are stored locally in app data.
- User can pause/resume indexing and clear index/snapshots.
- Eligible text/code/docs/config edits create pre-edit snapshots.
- Deletes route to macOS Trash.
- Activity log records user, agent, system, and external events.
- Agent file access is off by default and separately enabled.
- Agent file operations use only the approved MVP tool surface.
- Agent file operations are logged and snapshotted where eligible.
- UI never claims fake permission, fake files, fake index state, or fake successful operations.
- Finder extensions/context menus are not part of MVP acceptance.
- Feature is not complete until local/native verification passes and feature critique verdict is APPROVED.
