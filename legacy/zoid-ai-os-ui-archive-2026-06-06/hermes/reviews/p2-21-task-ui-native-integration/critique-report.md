# Combined Critique: P2.21 Task UI native integration

Verdict: APPROVED

## Summary

Lean re-review focused only on the required fixes from the prior `REQUEST_CHANGES` report. Both required issues are resolved:

- Existing persisted tasks no longer auto-select the first task on refresh when the user has not explicitly selected one, so the Tasks workspace stays in create mode on initial native load.
- A visible `New task` action clears selection and restores a create draft, preserving an explicit create path after viewing/editing persisted tasks.
- Explicit task selection hydrates the edit form from the selected persisted task via `formDraftForTask`, so edit mode is no longer shown with a blank bridge form for selected tasks.
- The status control is disabled and includes explanatory copy that status is shown from persisted state and changed via the separate native status action, avoiding the prior misleading editable-status behavior.

## Required-fix verification

### 1. Existing persisted tasks hide/break task creation — fixed

- `refreshTasksFromBridge` now only preserves a requested selected task if it is still visible; with `selectedTaskId: null`, it returns `selectedTaskId: null` even when persisted tasks exist.
- `TaskWorkspace` exposes a `New task` button wired through `App` to clear selection and reset the form.
- `App.handleSelectTask` hydrates `taskBridgeUi.form` from the selected persisted task before rendering edit mode.
- Regression coverage verifies create-mode preservation and edit-form hydration.

### 2. Status editing is misleading — fixed

- The status `<select>` is disabled.
- The UI copy now states: "Status is shown from persisted task state; status changes use the separate native status action."
- This truthfully scopes status outside the create/update submit path, matching the existing backend bridge commands.

## Verification run

Ran from `/Users/ziadnasreldin/Zoid`:

- `npx tsx src/taskBridgeIntegration.test.ts` — PASS
- `npm run test:frontend` — PASS
- `npm run build` — PASS
- `git diff --check` — PASS

## Notes

No source files were edited during this re-review. Only this critique report was updated.
