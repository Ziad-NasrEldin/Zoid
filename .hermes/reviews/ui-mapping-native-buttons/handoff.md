# UI mapping native buttons handoff

## Goal
Ensure completed Zoid implementation/tracker items map to real UI controls and native backend commands, not fake browser-only state.

## Files changed
- `src/App.tsx`
  - Added Code workspace Phase 4 forms/actions for real repo profile, repo link, launch gate, evidence, and evaluate native commands.
  - Added task lifecycle callbacks for status/archive/delete native command actions.
- `src/taskWorkspace.tsx`
  - Added selected-task lifecycle controls: update status, archive task, delete task.
  - Controls call parent bridge callbacks and explicitly reference native command names.
- `src/taskBridgeIntegration.ts`
  - Added `performTaskActionThroughBridge` for `update_task_status_command`, `archive_task_command`, and `delete_task_command`.
- `src/taskBridgeIntegration.test.ts`
  - Added regression coverage for task status/archive/delete command names and Tauri arg shapes.

## Verification already run
- `npm run test:frontend && npm run build` passed.
- Vite dev server started on `http://127.0.0.1:1421/` and `curl -I` returned HTTP 200.
- Browser E2E smoke with browser tools loaded the app and clicked into Tasks. Browser preview correctly showed native backend unavailable and did not simulate records; create form remained native-only/fail-closed.

## Important behavior
- Browser preview cannot execute actual Tauri commands, so native command controls only fully verify inside the Tauri desktop app. The browser smoke confirms no fake fallback records are exposed.
- Task lifecycle controls render only when a real selected task is loaded from the native backend.
- Command arg convention used: top-level Tauri invoke args camelCase (`taskId`), nested request payload snake_case/canonical Rust serde fields where applicable.

## Review request
Critique whether the implementation satisfies the user goal for real completed-item UI mapping. Focus on Required fixes only:
1. broken TypeScript/build/test issues,
2. fake UI/state instead of native commands,
3. missing or incorrect command argument shapes,
4. browser-preview truthfulness regressions,
5. obvious UX blockers that prevent a human from using the mapped controls.

Return verdict APPROVED or CHANGES_REQUIRED with exact required fixes.