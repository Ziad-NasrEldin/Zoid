# Feature Handoff: Tasks page audit and usability fixes

## Original request

User asked to go through Zoid AI OS one page at a time, screenshot each page, visually identify what needs work, fix UI/backend mapping/layout/adaptability/implementation issues, verify through E2E/visual checks, and stop after each page for user verification. Current page: Tasks.

## Implementation summary

- Audited Tasks page in browser preview and native Tauri app.
- Confirmed browser preview truthfully fails closed: no fake tasks are created or listed outside Tauri.
- Confirmed native Tauri Tasks backend is mapped: `list_tasks_command` returns persisted task records.
- Fixed Tasks layout to use the full-width native workspace surface instead of also showing the generic workspace inspector. The task workspace already contains task-specific list/detail/editor, so the generic inspector duplicated information and cramped the task UI.
- Fixed native task refresh behavior to auto-select the first persisted task when tasks exist and no requested selection is active, so the page opens with real task details instead of an empty "Select a task" panel.
- Restored app default active workspace to Today after temporary screenshot setup.

## Changed files

- `src/App.tsx`: includes `tasks` in `native-editor-active` split-view mode, hiding the duplicate generic inspector and widening the task workspace.
- `src/taskBridgeIntegration.ts`: auto-selects first visible persisted task when no selected task is requested.
- `src/taskBridgeIntegration.test.ts`: updates bridge expectation for first-task auto-selection.
- `src/App.css`: carries prior Today responsive split-view fix from this audit batch; no new Tasks-specific CSS added in this slice.

## How to test

- Start native dev app: `npm run tauri:dev`
- Open/click Tasks in the native Zoid app.
- Expected native behavior:
  - Header says `Tasks`.
  - Status says `Native ready`.
  - Task list shows persisted tasks from `list_tasks_command`.
  - First task is selected by default when tasks exist.
  - Detail panel shows persisted task details/metadata/status actions.
  - Generic workspace inspector is hidden on Tasks.
- Browser preview expected behavior:
  - Shows `Task backend unavailable`.
  - Does not fabricate task records.
  - Create form remains fail-closed outside native.

## Tests run

- `npm run test:frontend`: PASS; includes `taskBridgeIntegration tests passed`.
- `npm run test:rust`: PASS; 190 passed, 0 failed, 1 ignored.
- `npm run build`: PASS; Vite production build succeeded.
- Native visual evidence:
  - Before/finding screenshot: `.hermes/screenshots/zoid-audit/tasks-native-dev2.png`
  - Post-fix partial native visible screenshot: `.hermes/screenshots/zoid-audit/tasks-native-fixed5.png`
  - Browser-preview fixed layout visual check via browser tool: generic inspector hidden and full-width form visible.

## Git info

- Branch: not checked in this handoff.
- Commit SHA: not committed.
- Diff base: working tree against current repository index.

## Frontend/backend/database notes

- Frontend route/page: single React app state workspace `tasks`, rendered by `TaskWorkspace`.
- Backend commands involved: `list_tasks_command`, `read_task_command`, `create_task_command`, `update_task_command`, `update_task_status_command`, `archive_task_command`, `delete_task_command`.
- Database: existing native SQLite task records are read via native commands. No migration changed.
- Local data note: existing persisted task records were visible, including prior manual/test-looking titles. This slice did not delete existing user data.

## Reviewer focus areas

- Confirm auto-selecting first task does not break intentional create-new mode after clicking `New task`.
- Confirm hiding the generic inspector on Tasks is correct because Tasks already has task-specific detail/editor panels.
- Confirm browser preview remains fail-closed and does not simulate tasks.
- Confirm Tauri invoke arg names still match Rust command signatures.

## Fix cycle notes

- Critique v1 returned `REQUEST_CHANGES` because first-task auto-selection hydrated `selectedTaskId` but not the controlled edit form or task-scoped linked panels.
- Fixed by adding `applyBridgeStateToTaskUi`, which hydrates the controlled form and clears stale form errors when a persisted selected task is applied.
- Added frontend bridge coverage for auto-selected task form hydration and create-mode draft preservation.
- Added App-level selected-task effect to load linked panels/run controls when refresh/initial load selects a task without a manual row click.
- Re-ran `npm run test:frontend && npm run test:rust && npm run build`: PASS.

## Known limitations / blockers

- Full native UI click automation is limited by macOS accessibility/frontmost-app behavior in this session. Native screenshots were captured, but post-fix foreground control was inconsistent because other apps/spaces kept stealing focus. Backend/native mapping was verified by visible native state and existing Rust/frontend tests, not a full automated native click E2E.
- No user data cleanup performed; existing persisted task records were left intact.
