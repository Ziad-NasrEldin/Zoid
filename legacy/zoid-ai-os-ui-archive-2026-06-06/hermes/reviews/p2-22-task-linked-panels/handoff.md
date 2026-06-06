# Feature Handoff: P2.22 Task detail linked panels

## Original request

User asked to see what is next and continue using the lean review cycle. Current tracker target: `P2.22 Frontend: linked run/review/history panels inside task detail.`

## Implementation summary

- Added frontend linked-panels state/model layer for selected task detail.
- Task detail now shows linked run, linked review, and task history panels when a persisted task is selected.
- Panels load from real native bridge commands rather than fake data:
  - `list_entity_history_command` for task-related timeline records with related entities.
  - `read_run_status_command` for linked run IDs discovered from history targets/matches.
  - `read_review_record_command` for linked review IDs discovered from history targets/matches.
- Errors are sanitized with the existing history redaction helper.
- Empty/loading/error states are explicit and truthful.
- Existing `HistoryTimeline` component is reused for the task history panel.

## Changed files

- `src/taskLinkedPanels.ts`: linked panel state, bridge loader, run/review extraction, redacted view model.
- `src/taskLinkedPanels.test.ts`: focused P2.22 tests for bridge command names/args, run/review hydration, history rendering, and redacted errors.
- `src/taskLinkedPanelsView.tsx`: React panels for linked runs, reviews, and history timeline.
- `src/taskWorkspace.tsx`: accepts and renders linked panels inside selected task detail.
- `src/App.tsx`: owns linked panel state, loads panels on task selection/create/update, clears on New Task, wires panel refresh.
- `package.json`: includes `taskLinkedPanels.test.ts` in `npm run test:frontend`.

## How to test

- `npx tsx src/taskLinkedPanels.test.ts`
- `npm run test:frontend`
- `npm run build`
- `git diff --check`

## Tests run

- RED: `npx tsx src/taskLinkedPanels.test.ts` failed before implementation with `ERR_MODULE_NOT_FOUND` for `./taskLinkedPanels`.
- GREEN: `npx tsx src/taskLinkedPanels.test.ts`: PASS (`taskLinkedPanels tests passed`).
- `npm run test:frontend`: PASS.
- `npm run build`: PASS (`✓ 43 modules transformed`).
- `git diff --check`: PASS.

## Git info

- Branch: `main`
- Base before feature: `b083327 feat: wire task workspace to native bridge`
- Commit SHA: pending

## Frontend/backend/database notes

- Frontend: linked panels render under selected task detail in the Tasks workspace.
- Backend: no Rust changes; uses existing P2.18/P2.19 bridge commands.
- Database: no schema changes; data comes from existing history/run/review records.

## Reviewer focus areas

- Confirm P2.22 scope is satisfied without overbuilding P2.23/P2.24/P2.25.
- Confirm task history command args match the Tauri bridge shape.
- Confirm linked run/review IDs are discovered only from persisted history targets/matches.
- Confirm no fake linked runs/reviews/history are generated in empty/error states.
- Confirm errors and summaries do not leak secret-like text.

## Fix cycle notes

Initial lean combined critique request.
