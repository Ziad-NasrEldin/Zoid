# P2.22 Lean Combined Critique — Task Linked Panels

## Verdict

APPROVED

## Scope reviewed

Changed files reviewed:

- `src/App.tsx`
- `src/taskWorkspace.tsx`
- `src/taskLinkedPanels.ts`
- `src/taskLinkedPanelsView.tsx`
- `src/taskLinkedPanels.test.ts`
- `package.json`

Focus areas: P2.22 spec compliance, bridge/API correctness, UX truthfulness, security/redaction, and focused tests/build.

## Findings

### P2.22 spec compliance

The implementation satisfies the requested P2.22 scope: selected persisted task detail now receives linked panels for runs, reviews, and history. Panels are only rendered when a task is selected, avoiding fake create-mode content. The implementation stays within the requested frontend scope and does not overbuild unrelated run/review workflows.

### Bridge/API correctness

The code uses existing native bridge commands instead of mock data:

- `list_entity_history_command` with `entity_type: "task"`, selected `entity_id`, `include_related: true`, and a bounded `limit`.
- `read_run_status_command` for run IDs discovered from persisted history targets/matches.
- `read_review_record_command` for review IDs discovered from persisted history targets/matches.

I checked the Rust command definitions in `src-tauri/src/lib.rs`; command names and request shapes are consistent with the existing P2.18/P2.19 bridge surface. Hydration failures for individual linked runs/reviews are treated as missing records rather than invented data, which is acceptable for a summary panel.

### UX truthfulness

Loading, empty, and error states are explicit. The panels do not synthesize runs/reviews/history when bridge data is unavailable. The linked panels appear under the selected task detail and expose a refresh action for linked activity.

Minor non-blocking note: the task workspace's broader refresh action refreshes the task list/detail state but does not also refresh linked panels; the linked panels have their own refresh button, so this is acceptable for P2.22 and not a required change.

### Security / redaction

Error copy is passed through the existing `sanitizeMessage` helper, and history summaries continue to use the established history timeline redaction path. Run/review status summaries are sanitized. I did not find new raw-log rendering or obvious secret-bearing metadata exposure in the linked panels.

### Tests / build

Focused verification passed locally:

```text
npm run test:frontend
# taskLinkedPanels tests passed
# taskBridgeIntegration tests passed
# taskViewModel tests passed
# inboxViewModel tests passed
# historyTimelineViewModel tests passed
# todayWidgets tests passed
# todayFoundation tests passed
# settingsStatus tests passed
# confirmationPolicy tests passed
# workspaceRegistry tests passed

npm run build
# tsc && vite build passed
# vite built 43 modules successfully

git diff --check
# passed
```

## Conclusion

P2.22 is approved. The feature uses real bridge-backed data, keeps empty/error states truthful, preserves redaction expectations, and passes focused frontend/build verification. No blocker-level changes are required.
