# Critique Report: P2.20 Today widgets

## Verdict

APPROVED

## Summary

The P2.20 implementation truthfully wires Today widgets to existing native task and inbox-notification bridge commands, avoids fake browser-preview data, and explicitly reports the active-run listing gap instead of inventing run rows. The frontend view-model has focused coverage for task filtering, notification-derived blockers/completions, active-run unavailability, empty states, and preview/unavailable states. Local frontend tests, production build, and whitespace checks passed in this review.

I did not find a blocker requiring source changes for P2.20. The main remaining risks are coverage depth and future semantics: App-level native bridge success/error rendering is not covered by an actual React/Tauri mock test, and the `source` field in `TodayWidgetsInput` is currently not enforced inside the view-model.

## What was changed

- `package.json`
  - Added `src/todayWidgets.test.ts` to `npm run test:frontend`.
- `src/App.tsx`
  - Added native reads for `list_tasks_command` and `list_inbox_notifications_command`.
  - Added Today data panels for tasks, active runs, blockers, and completions.
  - Added explicit native bridge error copy that states no preview/fallback records are simulated.
  - Added an explicit active-runs bridge-gap message because no list-active-runs/list-runs command is registered.
- `src/todayWidgets.ts`
  - Added typed Today widget view-models for tasks, inbox notifications, active runs, panel states, tones, and list items.
  - Filters archived/deleted tasks out of the Today task list.
  - Derives blockers and completions from notification records.
  - Handles checking, unavailable, ready-empty, and ready-populated states without fake data.
- `src/todayWidgets.test.ts`
  - Added lightweight frontend regression coverage for real task filtering, blockers/completions from notifications, active-run gap messaging, empty states, and preview non-simulation copy.

## Required fixes table

| Severity | Area | Required fix | Rationale | Status |
|---|---|---|---|---|
| — | — | None required for approval. | The changed feature satisfies the stated P2.20 truthfulness constraints and passes the requested verification commands. | — |

## Improvements

| Priority | Area | Suggested improvement | Rationale |
|---|---|---|---|
| Medium | Coverage | Add App-level tests with mocked `invoke` for native success, bridge rejection, and browser preview ordering. | Current tests cover the view-model but not the React wiring, native command names, argument shape, or rendered bridge-error/preview states. |
| Medium | View-model contract | Either remove `source` from `TodayWidgetsInput` or enforce it so `preview` cannot render ready records even if accidentally passed by a future caller. | The current App passes safe data states, but the view-model API advertises source awareness while not using it. |
| Low | Task semantics | Clarify whether the Today tasks panel should include all non-archived/non-deleted tasks or only work-in-progress statuses such as inbox/planned/active/waiting/review_required/blocked. | The handoff uses “active non-archived/non-deleted” wording, while the implementation includes completed/failed/cancelled tasks unless archived/deleted. This may be intended, but it should be made explicit. |
| Low | UI polish | Add dedicated CSS for `.today-widget-list` / `.today-data-widget-card` if visual spacing is not already acceptable through inherited card/list styles. | Build passes, but the new list markup currently relies mostly on existing generic styling. |

## Tests performed

- Inspected handoff and changed files:
  - `.hermes/reviews/p2-20-today-widgets/handoff.md`
  - `package.json`
  - `src/App.tsx`
  - `src/todayWidgets.ts`
  - `src/todayWidgets.test.ts`
- Inspected relevant native command definitions/registration in `src-tauri/src/lib.rs`:
  - `list_tasks_command`
  - `list_inbox_notifications_command`
  - `read_run_status_command` exists, but no list-run/list-active-runs command was found/used for this feature.
- Ran `npm run test:frontend` — PASS.
- Ran `npm run build` — PASS.
  - Vite output: `✓ 36 modules transformed`, built in `290ms`.
- Ran `git diff --check` — PASS, no whitespace errors reported.
- Ran `git status --short` and `git diff` for the feature files to confirm scope.

## Tests still needed

- React/App rendering tests that mock `@tauri-apps/api/core` `invoke` and verify:
  - Native task records render in the Today tasks panel.
  - Native notification records render blockers and completions.
  - Task/inbox bridge failures render the “No browser preview or fallback records are simulated” copy.
  - Browser preview does not briefly or permanently render fake tasks/runs/notifications.
  - Active runs remain unavailable until a real list-run bridge exists.
- A regression test for completed/failed/cancelled task treatment once product semantics for “Today tasks” are finalized.
- Native/fixture-backed integration coverage, when practical, that exercises the actual Tauri command payload for `list_inbox_notifications_command` with `{ request: { active_only: true, limit: 50 } }`.

## Dev-agent instructions

- No blocking source fix is required for P2.20 based on this review.
- Do not add simulated/fallback Today records in browser preview or native bridge error states.
- Keep the active-runs panel truthful until a real persisted run-list command is added; do not infer active runs from tasks or notifications.
- If doing a follow-up hardening pass, prioritize App-level mocked-bridge tests and decide whether `TodayWidgetsInput.source` should be enforced or removed.
- Clarify/product-test the exact task statuses that belong in “Today tasks” before changing the current non-archived/non-deleted behavior.

## Notes

- `git status --short` showed an unrelated modified `src-tauri/src/tests.rs` and another review directory in the working tree. I did not review or modify those as part of P2.20.
