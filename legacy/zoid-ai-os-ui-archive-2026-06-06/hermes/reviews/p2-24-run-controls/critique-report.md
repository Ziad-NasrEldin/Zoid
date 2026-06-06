# P2.24 Run Controls Critique

Verdict: APPROVED

## Summary

Re-reviewed the stale-run reset fix only, plus focused regression checks. The prior blocking issue is resolved: task selection and task creation now route through `resetRunControlsForTask`, which clears local run status when the selected task id changes, preventing a previous task's `activeRun` from remaining visible/cancellable in a different task detail.

The added regression in `src/runControls.test.ts` covers this behavior by switching from `task-a` to `task-b` with a running active run, then asserting the active run is cleared, cancel is disabled, and the status returns to Idle.

## Fix verification

- `src/runControls.ts:94-100` adds `resetRunControlsForTask`, which updates `draft.taskId` and uses `clearStatus: state.draft.taskId !== taskId`.
- `src/App.tsx:626-628` uses the helper for created tasks.
- `src/App.tsx:653-655` uses the helper for selected tasks.
- `src/App.tsx:608-619` still clears status when entering the new-task path.
- `src/runControls.test.ts:79-95` adds the stale active-run regression.
- Tauri command shapes remain aligned with the native signatures: `start_agent_run_command(request)` and `cancel_run_command(run_id, request)` (`src-tauri/src/lib.rs:1930-1959`), with frontend invoke args using `request` and camelCase `runId` as expected by Tauri.

## Verification run

```text
npx tsx src/runControls.test.ts: PASS (runControls tests passed)
npm run test:frontend: PASS
npm run build: PASS (tsc && vite build, 47 modules transformed)
git diff --check: PASS
```

No remaining P2.24 blocking issues found in this lean re-review.
