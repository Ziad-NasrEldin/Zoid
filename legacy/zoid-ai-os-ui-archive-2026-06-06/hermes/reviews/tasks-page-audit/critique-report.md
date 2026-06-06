# Critique Report: Tasks Page Audit/Fix Re-review

Verdict: APPROVED

## Scope reviewed

- Handoff: `/Users/ziadnasreldin/Zoid/.hermes/reviews/tasks-page-audit/handoff.md`
- Required fix from prior review: R1 hydration/link panel issue for first-task auto-selection.
- Changed files reviewed for this re-review: `src/taskBridgeIntegration.ts`, `src/App.tsx`, `src/taskBridgeIntegration.test.ts`.
- Product source code was not modified by this review. Only this critique report was overwritten.

## Verification performed by reviewer

- Re-read the updated handoff fix-cycle notes.
- Inspected the relevant source paths for the R1 fix:
  - `src/taskBridgeIntegration.ts`
  - `src/App.tsx`
  - `src/taskBridgeIntegration.test.ts`
- Reviewed the working diff for those files.
- Ran `npm run test:frontend` from `/Users/ziadnasreldin/Zoid`.

Reviewer test result:

- `npm run test:frontend`: PASS
  - Includes `taskBridgeIntegration tests passed`.
  - Command exited with code 0.

The developer-reported broader gates remain noted from handoff:

- `npm run test:rust`: PASS, 190 passed, 0 failed, 1 ignored.
- `npm run build`: PASS.

## Summary assessment

The required R1 issue is resolved.

The previous blocker was that first-task auto-selection updated `selectedTaskId` but did not hydrate the controlled task editor form or initialize task-scoped linked panels/run controls. That created a state where the UI could show a selected persisted task while the adjacent controlled editor still displayed the blank create form.

The updated implementation now closes that gap:

- `src/taskBridgeIntegration.ts` adds `applyBridgeStateToTaskUi`, which detects the selected persisted task in a ready bridge state and hydrates `taskBridgeUi.form` via `formDraftForTask(selectedTask)`.
- The same helper clears stale form errors when a selected persisted task is applied.
- `src/App.tsx` now uses `applyBridgeStateToTaskUi` in `applyTaskState`, so initial load and refresh paths that call `refreshTasksFromBridge` also hydrate the controlled editor form when a task is auto-selected.
- `src/App.tsx` adds a selected-task effect that observes `selectedTaskIdForPanels` and initializes task-scoped run controls plus linked panels when a selected task appears from initial load/refresh rather than from a manual row click.
- `src/taskBridgeIntegration.test.ts` adds focused coverage for editor form hydration on an auto-selected persisted task and preservation of a create-mode draft when no selected task is applied.

## Findings

### R1 - Auto-selected first task hydration/link panel integration

Severity: Pass

Evidence:

- `src/taskBridgeIntegration.ts:58-69` implements `applyBridgeStateToTaskUi`:
  - Finds the selected task from a ready state.
  - Sets `form` to `formDraftForTask(selectedTask)` when present.
  - Clears `formErrors` when a selected task is applied.
  - Preserves the current form when there is no selected task.
- `src/App.tsx:1208-1212` routes all `applyTaskState` updates through `applyBridgeStateToTaskUi`, covering initial/background refresh and explicit task refresh paths.
- `src/App.tsx:1249-1257` adds the selected-task side effect:
  - Derives `selectedTaskIdForPanels` from ready task state.
  - Skips when no task is selected.
  - Skips when linked panels are already for the selected task.
  - Clears clean session state.
  - Resets run controls for the selected task.
  - Calls `loadLinkedPanels(selectedTaskIdForPanels)`.
- `src/taskBridgeIntegration.test.ts:49-56` verifies that applying a ready selected task hydrates title/detail/priority and clears stale form errors.
- `src/taskBridgeIntegration.test.ts:58-63` verifies that create-mode form data is preserved when no selected task is applied.
- `npm run test:frontend` passed in this re-review.

Impact:

- Initial load/refresh auto-selection now results in selected task state and controlled edit form state matching the same persisted task.
- Linked task panels/run controls are now initialized for a selected task that is applied via initial load/refresh, not only via manual row selection.
- The prior misleading edit UI condition is addressed.

### Non-blocking note: possible duplicate linked-panel load on manual selection

Severity: Informational

The new selected-task effect may overlap with existing manual-selection follow-up work in `handleSelectTask`, which already resets run controls and awaits `loadLinkedPanels`. Because the effect is guarded by `taskLinkedPanels.taskId`, and `loadLinkedPanels` sets a loading state for the task synchronously before awaiting bridge work, this is unlikely to be user-visible. I do not consider it a blocker for this audit, but it is worth watching if future logs show duplicate linked-panel bridge calls on manual task selection.

## Regression considerations

- Browser preview fail-closed behavior remains intact because `refreshTasksFromBridge` still returns an error state on bridge failure rather than fabricating tasks.
- Create-mode safety is improved relative to the prior review: the hydration helper does not overwrite a draft unless a persisted task is actually selected.
- The explicit `New task` handler still clears selected task, linked panels, clean sessions, run controls, and manual review state.

## Final decision

APPROVED. The required R1 hydration/link panel issue is resolved in source and covered by focused frontend bridge tests. Reviewer-ran `npm run test:frontend` passed. No source edits were made during this re-review.
