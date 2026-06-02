# Critique Report: P2 UI isolated components batch

## Verdict

APPROVED

## Required fixes

None. RF-1 has been resolved.

## Re-review findings

- Confirmed `src/inboxViewModel.ts` active-only filtering now matches backend/native active inbox semantics: only `pending`, `delivered`, `action_required`, and `failed` records are included, with dismissed/resolved records still excluded.
- Confirmed `src/inboxViewModel.test.ts` now asserts `read`/`read_at` notifications are excluded from the active attention list and that the active count/order reflects only active inbox states.
- Re-checked the backend contract in `src-tauri/src/lib.rs`: `NotificationState::is_active_inbox()` matches `Pending | Delivered | ActionRequired | Failed`.
- No new blocking issues found in the re-review scope.

## Improvements

- Consider adding explicit bridge command/intention naming for the inbox and history isolated components, similar to `taskBridgeCommands`, so future App/native wiring has a concrete contract to verify.
- Consider adding a small component render smoke test once a frontend test renderer/testing-library setup exists. The current direct view-model tests cover the important data behavior but do not exercise JSX output.
- For history pagination, document that `nextCursor` is only derived heuristically from a full page in the isolated view-model and should be supplied by native integration when available.

## Tests performed

- Inspected handoff fix-cycle notes for RF-1.
- Inspected `src/inboxViewModel.ts` and `src/inboxViewModel.test.ts` to verify the active filtering fix and regression coverage.
- Compared inbox active-state semantics against backend `NotificationState::is_active_inbox` in `src-tauri/src/lib.rs`.
- Ran `npm run test:frontend`: PASS.
- Ran `npm run build`: PASS (`tsc && vite build`, 36 modules transformed, built in 294ms).
- Ran `git diff --check`: PASS.
