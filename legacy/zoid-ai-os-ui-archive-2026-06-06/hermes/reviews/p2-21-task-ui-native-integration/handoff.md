# Feature Handoff: P2.21 Task UI native integration

## Original request

User asked to continue Zoid Phase 2 quickly using a lean review cycle. Current target: `P2.21 Frontend: task create/list/detail UI with validation and persistence.`

## Implementation summary

- Added `src/taskBridgeIntegration.ts` as a small testable controller layer between the existing isolated `TaskWorkspace` component and Tauri task commands.
- Wired `src/App.tsx` so selecting the `Tasks` workspace renders `TaskWorkspace` instead of the generic placeholder.
- Task list/detail/create/update now use real native task bridge commands:
  - `list_tasks_command`
  - `read_task_command`
  - `create_task_command`
  - `update_task_command`
- Local validation runs before create/update; invalid forms return local errors and do **not** call the native bridge.
- After create/update/select/refresh, the app refreshes persisted tasks and syncs Today task widgets with the same real task list.
- Browser/native failures remain truthful error states; no fake tasks are generated.

## Changed files

- `src/taskBridgeIntegration.ts`: task bridge controller functions, validation-before-invoke, create/update/select/refresh state handling.
- `src/taskBridgeIntegration.test.ts`: focused P2.21 tests for bridge command names/payloads, local validation, refresh, select, create, update, and error handling.
- `src/App.tsx`: imports task bridge/controller/component, initializes task UI state, syncs task list with Today widgets, renders `TaskWorkspace` for the Tasks workspace.
- `package.json`: adds `taskBridgeIntegration.test.ts` to `npm run test:frontend`.

## How to test

- `npx tsx src/taskBridgeIntegration.test.ts`
- `npm run test:frontend`
- `npm run build`
- `git diff --check`

Expected behavior:

- Task bridge integration tests pass.
- Frontend test suite passes.
- TypeScript/Vite build passes.
- Invalid task form inputs do not call native Tauri commands.
- Task workspace uses native task commands and no sample/fake persisted tasks.

## Tests run

- RED: `npx tsx src/taskBridgeIntegration.test.ts` failed before implementation with `ERR_MODULE_NOT_FOUND` for `./taskBridgeIntegration`.
- GREEN: `npx tsx src/taskBridgeIntegration.test.ts`: PASS (`taskBridgeIntegration tests passed`).
- `npm run test:frontend`: PASS.
- `npm run build`: PASS (`✓ 39 modules transformed`).
- `git diff --check`: PASS.

## Git info

- Branch: `main`
- Base before feature: `13494f6 feat: add isolated task inbox history UI models`
- Commit SHA: pending

## Frontend/backend/database notes

- Frontend: task workspace is now wired in `App.tsx` for the `tasks` workspace.
- Backend: no Rust/backend changes; uses existing P2.17 task Tauri bridge commands.
- Database: no schema changes; persistence is through existing task service/repository behind Tauri commands.

## Reviewer focus areas

- Verify P2.21 scope is satisfied: create/list/detail UI with validation and persistence via real task bridge commands.
- Confirm invalid task forms cannot call native bridge commands.
- Confirm App task state and Today task state stay truthful and do not fabricate data.
- Confirm Tauri invoke argument names match backend command parameters (`taskId`, `request`).
- Confirm this lean batch has enough focused test coverage and build verification.

## Fix cycle notes

Initial combined critique returned `REQUEST_CHANGES` with two findings:

1. Existing persisted tasks auto-selected the first task, hiding create mode and leaving the edit form blank/misleading.
2. Status looked editable even though create/update commands do not persist status; status uses a separate native command.

Fixes applied:

- `refreshTasksFromBridge` now preserves create mode when no task was explicitly selected instead of auto-selecting the first persisted task.
- Added `New task` action in `TaskWorkspace` to clear selection and reset the create form.
- Selecting a task now hydrates the edit form from the selected persisted task using `formDraftForTask`.
- Status select is disabled and includes copy explaining that status is shown from persisted state and changed via the separate native status action.
- Added regression coverage for create-mode preservation and edit-form hydration.

Post-fix verification:

- `npx tsx src/taskBridgeIntegration.test.ts`: PASS.
- `npm run test:frontend`: PASS.
- `npm run build`: PASS.
- `git diff --check`: PASS.
