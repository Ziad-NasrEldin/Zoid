# Feature Handoff: P2.20 Today widgets

## Original request

Use the Zoid-wide subagent workflow to finish multiple things at once. Current tracker item: `P2.20 Frontend: Today widgets showing real tasks, active runs, blockers, completions, empty states.`

## Implementation summary

- Added a typed Today widget view-model in `src/todayWidgets.ts`.
- Added frontend coverage in `src/todayWidgets.test.ts` and included it in `npm run test:frontend`.
- Wired `src/App.tsx` to native Tauri task/inbox commands:
  - `list_tasks_command` for Today task records.
  - `list_inbox_notifications_command` for blockers/completions.
- Active runs are truthful: there is no registered list-active-runs bridge yet, so the UI states that persisted active AgentRun rows cannot be queried truthfully instead of fabricating runs.
- Browser preview/native-unavailable states explicitly avoid simulated tasks, runs, or notifications.

## Changed files

- `package.json`: includes `src/todayWidgets.test.ts` in `test:frontend`.
- `src/App.tsx`: loads native Today task/inbox data, renders Today widget panels, and reports the active-run list bridge gap truthfully.
- `src/todayWidgets.ts`: Today widgets view-model/types for tasks, active runs, blockers, completions, empty/unavailable/checking states.
- `src/todayWidgets.test.ts`: frontend regression tests for real task filtering, notification-derived blockers/completions, active-run gap, empty states, and preview non-simulation.

## How to test

- `npm run test:frontend`
- `npm run build`
- `git diff --check`

Expected behavior:

- Today tasks show active non-archived/non-deleted records from `list_tasks_command`.
- Blockers and completions derive from real inbox notification records.
- Active runs panel does not fabricate rows while no list-run bridge exists.
- Empty/unavailable/preview states are explicit and truthful.

## Tests run

- `npm run test:frontend`: PASS
- `npm run build`: PASS (`✓ 36 modules transformed`, built in 284ms in parent verification)
- `git diff --check`: PASS

## Git info

- Branch: `main`
- Commit SHA: not committed yet
- Current base before commit: `bd03152 feat: add review notification history Tauri bridge`

## Frontend/backend/database notes

- Frontend routes/components: Today home in `src/App.tsx`; widget view-model in `src/todayWidgets.ts`.
- Backend endpoints/services: uses existing Tauri commands for tasks and inbox notifications.
- Database tables/migrations: no schema change.
- Known backend gap: no list-active-runs/list-runs bridge command exists; UI must stay truthful until P2.24 or a dedicated run-list bridge adds that capability.

## Reviewer focus areas

- Ensure no fake/preview records are shown as real native data.
- Ensure Today blockers/completions come from notification data rather than hardcoded cards.
- Ensure active-runs widget is truthful about backend gaps.
- Ensure App wiring handles browser preview/native unavailable states cleanly.
- Ensure frontend tests cover the stated behavior.

## Fix cycle notes

Initial critique request.
