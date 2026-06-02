# Critique Report: P2.25-P2.27 Task Detail Frontend Batch

Verdict: APPROVED

## Scope reviewed

Changed files reviewed:

- `src/taskDetailBatchPanels.ts`
- `src/taskDetailBatchPanelsView.tsx`
- `src/taskDetailBatchPanels.test.ts`
- `src/taskLinkedPanelsView.tsx`
- `src/App.tsx`
- `package.json`

Focus areas checked: manual review native bridge shape, fake-record avoidance, task/run scoping, redaction/security posture, truthful UX states, and relevant frontend verification.

## Findings

### P2.25 Manual review stub UI

- The bridge call uses the registered native command `create_manual_review_command` and passes the expected top-level `request` argument.
- Request field names match the native `ManualReviewCommandCreateRequest` shape: `task_id`, `run_id`, `reviewer_profile_id`, `verdict`, `evidence_summary`, `required_fixes_json`, and `metadata_json`.
- The UI is a stub-style recorder as requested: no fake review records are fabricated; the latest review display is populated only from the native bridge response.
- Local validation blocks missing task selection, weak/empty evidence, malformed required-fixes JSON, and obvious secret-looking evidence before invoking native code.
- Submitted evidence and displayed bridge errors/review summaries are sanitized via the existing history redaction helper.

### P2.26 Inbox attention cards

- Task detail consumes persisted native inbox records already loaded via `list_inbox_notifications_command`; no fake inbox records are introduced.
- `buildTaskScopedInboxState` scopes cards to the selected task ID or linked run IDs discovered for that selected task.
- Empty/unavailable states are truthful and explicitly say no fake notifications are shown/generated.

### P2.27 Task/run history view

- Existing task history remains rendered from persisted native history records.
- Added run history panels are filtered with `runHistoryRecordsForRun`, including only records whose event targets or matched entities include the specific run ID.
- The view does not synthesize history records; empty run histories use the existing truthful `HistoryTimeline` empty state.

## Verification run

- `npx tsx src/taskDetailBatchPanels.test.ts` — passed.
- `npm run test:frontend` — passed.
- `npm run build` — passed.
- `git diff --check` — passed.

## Required fixes

None.

## Non-blocking notes

- Secret detection is intentionally conservative and blocks obvious textual secret markers; sanitization also redacts displayed/persisted summaries. If future tracker scope requires exhaustive secret classification, expand the local detector to mirror all patterns in `sanitizeMessage`.
