# Feature Handoff: Task New Button Fix

## Original request

"the add new task button is not working"

## Root cause

The previous Tasks layout cleanup disabled `New task` whenever the native task bridge was unavailable (`state.mode !== "ready"`). In browser preview, the Tauri `invoke` bridge is unavailable by design, so the button was disabled and the create form was hidden. That preserved no-fake-data behavior, but it also blocked the user from opening/using the task creation UI.

## Implementation summary

- Re-enabled `New task` after bridge loading completes, including browser-preview error state.
- Kept task lists/details hidden unless the real native bridge returns persisted records.
- Restored the create form in non-loading states so the user can prepare a task and submit it.
- Browser-preview submissions still fail closed through the real native bridge call; no browser-only task records are created.
- `handleNewTask` now preserves the current unavailable/error state instead of switching to a misleading loading state.

## Changed files

- `src/taskWorkspace.tsx`: New task button enablement, create form visibility, truthful native-only copy.
- `src/App.tsx`: `handleNewTask` preserves current non-ready state while resetting the draft.

## How to test

1. Open `http://127.0.0.1:1420/`.
2. Click Tasks.
3. Verify `New task` is enabled in browser preview after native bridge error loads.
4. Click `New task`.
5. Verify the Create task form is visible and editable.
6. Enter a title and verify `Create task` enables.
7. Submit in browser preview: it must not create fake records; it should remain native-backend-truthful.

## Tests run

- `npm run test:frontend && npm run build`: PASS.
- Browser DOM check at localhost: PASS — `New task` enabled, task editor visible, `Create task` enables after title, no horizontal overflow.
- Browser console check: PASS — no JS errors from the interaction.

## Frontend/backend/database notes

- Frontend still calls `create_task_command` through the existing Tauri bridge path.
- No backend/database code changed.
- Browser preview remains a fail-closed UI-only environment because `invoke` is unavailable outside the Tauri desktop app.

## Reviewer focus areas

- Confirm the fix does not reintroduce fake task records in browser preview.
- Confirm the Add/New Task button is usable and the form can be edited.
- Confirm native desktop behavior should still create persisted tasks through the existing bridge.
