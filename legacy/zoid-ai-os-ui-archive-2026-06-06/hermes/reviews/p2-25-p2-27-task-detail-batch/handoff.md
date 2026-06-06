# Feature Handoff: P2.25-P2.27 Task detail frontend batch

## Original request

User asked to continue with a bunch of slices and allow subagents if needed/recommended. Active tracker targets:

- `P2.25 Frontend: manual review stub UI.`
- `P2.26 Frontend: notification/Inbox attention card basics.`
- `P2.27 Frontend: History view for task/run events.`

## Scope implemented

Integrated one frontend batch because all three sit under selected task detail and share existing native bridge/history data.

### P2.25 manual review stub UI

- Added `src/taskDetailBatchPanels.ts` model and bridge wrapper.
- Added `src/taskDetailBatchPanelsView.tsx` manual review panel.
- Wired into `src/App.tsx` and `src/taskLinkedPanelsView.tsx`.
- `createManualReviewThroughBridge` calls native `create_manual_review_command` with `{ request: { task_id, run_id, reviewer_profile_id, verdict, evidence_summary, required_fixes_json, metadata_json } }`.
- Local validation blocks missing task, weak evidence, invalid required-fixes JSON array, and secret-looking evidence before native invoke.
- Reviewer profile is a placeholder text input only, matching tracker scope.

### P2.26 notification/Inbox attention cards

- Reuses existing `inboxViewModel.ts` for attention-card behavior.
- Adds task-scoped filtering in `buildTaskScopedInboxState`:
  - selected `task_id`; or
  - notifications linked to runs discovered under selected task.
- App passes active native inbox records into task detail.
- No fake inbox records are generated.

### P2.27 task/run History view

- Existing task-level `HistoryTimeline` stays under linked panels.
- Added `LinkedRunHistoryPanels`, rendering per-run history timelines filtered from the selected task history by `run` targets.
- Filtering helper: `runHistoryRecordsForRun`.

## Files changed

- `src/taskDetailBatchPanels.ts`
- `src/taskDetailBatchPanelsView.tsx`
- `src/taskDetailBatchPanels.test.ts`
- `src/taskLinkedPanelsView.tsx`
- `src/App.tsx`
- `package.json`

## Verification already run

RED/focused:

- `npx tsx src/taskDetailBatchPanels.test.ts` failed before module existed.

Final local checks:

- `npx tsx src/taskDetailBatchPanels.test.ts` passed.
- `npm run test:frontend` passed.
- `npm run build` passed.
- `git diff --check` passed.

## Reviewer focus

1. Spec compliance for P2.25-P2.27 without scope creep.
2. Native bridge argument names for manual review creation.
3. Truthfulness: no fake review/notification/history records.
4. Task scoping: inbox and run history should not leak unrelated task/run records.
5. Security/redaction: secret-looking manual review evidence should be blocked; summaries sanitized.
6. UX: clear/error/empty states are truthful and actionable enough for frontend batch scope.

## Fix cycle notes

Initial lean combined critique request.
