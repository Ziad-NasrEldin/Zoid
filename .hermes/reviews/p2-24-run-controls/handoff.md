# Feature Handoff: P2.24 Run controls

## Original request

User asked to continue Phase 2 quickly using the lean review cycle. Current tracker target: `P2.24 Frontend: run controls for start/cancel and clear status/error handling.`

## Implementation summary

- Added frontend run controls for selected task detail.
- Start uses the existing native `start_agent_run_command` with `AgentRunCommandStartRequest` shape.
- Cancel uses the existing native `cancel_run_command` with `runId` plus cancel request.
- Clear status resets visible run/error/validation state locally.
- Run controls are placed in task detail context beside linked run/review/history panels.
- Successful start/cancel refreshes clean session output and linked task panels.
- Local validation blocks unsafe/unavailable starts before invoking native bridge:
  - selected task required;
  - profile required;
  - cwd required;
  - argv required;
  - logs directory required for truthful persisted output;
  - timeout bounds;
  - metadata must be a JSON object;
  - secret-looking argv/stdin/metadata rejected.

## Changed files

- `src/runControls.ts`: run controls state, validation, bridge start/cancel helpers, clear-state draft update, view model.
- `src/runControls.test.ts`: focused RED/GREEN coverage for start, cancel, clear, validation, and bridge argument shapes.
- `src/runControlsView.tsx`: React panel for start/cancel/clear and command fields.
- `src/taskLinkedPanelsView.tsx`: renders run controls in task detail context.
- `src/App.tsx`: owns run controls state, wires selected task/log directory context, refreshes linked panels/clean output after start/cancel.
- `package.json`: includes `runControls.test.ts` in frontend test script.

## How to test

- `npx tsx src/runControls.test.ts`
- `npm run test:frontend`
- `npm run build`
- `git diff --check`

## Tests run

- RED: `npx tsx src/runControls.test.ts` failed before implementation with missing `./runControls` module.
- GREEN: `npx tsx src/runControls.test.ts`: PASS (`runControls tests passed`).
- `npm run test:frontend`: PASS.
- `npm run build`: PASS (`✓ 47 modules transformed`).
- `git diff --check`: PASS.

## Git info

- Branch: `main`
- Base before feature: `e67a14c feat: add clean session output cards`
- Commit SHA: pending

## Frontend/backend/database notes

- Frontend: selected task detail now exposes run controls.
- Backend: no Rust changes; uses existing P2.18/P2.29 bridge commands.
- Database: no schema changes; backend commands create/update persisted runs/sessions/logs/events/notifications.

## Reviewer focus areas

- Confirm P2.24 scope is satisfied without overbuilding manual review P2.25 or E2E P2.31.
- Confirm `start_agent_run_command` and `cancel_run_command` argument shapes match Tauri command signatures.
- Confirm local validation avoids unsafe or untruthful native invokes.
- Confirm clear status is local-only and does not mutate persisted run data.
- Confirm UI remains truthful when logs dir is missing or bridge returns errors.
- Confirm focused tests cover start/cancel/clear/status/error paths.

## Fix cycle notes

Initial lean combined critique returned `REQUEST_CHANGES` for one blocking issue: run controls retained the previous task's active run when selecting/creating another task, which could display/cancel a stale run in the wrong task detail.

Fix added `resetRunControlsForTask`, switched App task selection/create paths to use it, and added a regression test proving task switches clear stale active runs and disable Cancel. Verification after fix:

- `npx tsx src/runControls.test.ts`: PASS.
- `npm run test:frontend`: PASS.
- `npm run build`: PASS.
- `git diff --check`: PASS.
