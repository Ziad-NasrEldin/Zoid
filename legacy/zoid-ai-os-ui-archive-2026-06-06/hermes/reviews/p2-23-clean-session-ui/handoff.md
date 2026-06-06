# Feature Handoff: P2.23 Clean Session UI

## Original request

User asked to continue Phase 2 quickly using the lean review cycle. Current tracker target: `P2.23 Frontend: Clean Session UI that streams output as clean cards/status, not raw terminal-first UI.`

## Implementation summary

- Added a Clean Session frontend model that reads persisted run status plus stream chunks from the existing native run bridge.
- Converts native log stream lines into clean product cards (`command`, `status`, `success`, `error`, `output`) instead of exposing a raw terminal blob.
- Preserves truthful states:
  - no simulated output when stream data is unavailable;
  - explicit unavailable state when `logs_dir` is missing;
  - explicit error state when native stream fails.
- Redacts/sanitizes stream lines and summaries via the existing history redaction helper.
- Wired clean session panels into selected task detail under linked runs from P2.22.
- Uses existing bridge commands only:
  - `read_run_status_command`
  - `stream_run_output_command`

## Changed files

- `src/cleanSession.ts`: Clean Session state, bridge loader, card classification, redacted view model.
- `src/cleanSession.test.ts`: focused P2.23 RED/GREEN coverage for bridge calls, clean cards, unavailable state, status mapping, and no raw terminal-first output.
- `src/cleanSessionView.tsx`: React panel rendering clean session cards/status.
- `src/taskLinkedPanelsView.tsx`: renders clean session output panels for linked task runs.
- `src/App.tsx`: owns clean session state, streams linked run output using `status.logs_dir`, clears output on New Task.
- `package.json`: includes `cleanSession.test.ts` in frontend test script.

## How to test

- `npx tsx src/cleanSession.test.ts`
- `npm run test:frontend`
- `npm run build`
- `git diff --check`

## Tests run

- RED: `npx tsx src/cleanSession.test.ts` failed before implementation with missing `./cleanSession` module.
- GREEN: `npx tsx src/cleanSession.test.ts`: PASS (`cleanSession tests passed`).
- `npm run test:frontend`: PASS.
- `npm run build`: PASS (`✓ 45 modules transformed`).
- `git diff --check`: PASS.

## Git info

- Branch: `main`
- Base before feature: `a432046 feat: add task linked activity panels`
- Commit SHA: pending

## Frontend/backend/database notes

- Frontend: clean cards render under selected task detail for linked persisted runs.
- Backend: no Rust changes; uses existing P2.18 stream/status bridge commands.
- Database: no schema changes; stream command reads persisted run/log references and log files through native bridge.

## Reviewer focus areas

- Confirm P2.23 scope is satisfied without overbuilding P2.24 run controls.
- Confirm stream command argument shape is correct, especially `request.run_id` and `request.logs_dir`.
- Confirm no raw terminal-first UI or raw terminal blob is exposed.
- Confirm unavailable/error states do not fabricate stream cards.
- Confirm redaction/sanitization protects secret-like stream text.
- Confirm task detail integration stays truthful when no linked run or no logs dir exists.

## Fix cycle notes

Initial lean combined critique returned `REQUEST_CHANGES` for one blocking issue: refresh always streamed from offset `0`, so later chunks of persisted run output were unreachable.

Fix applied:

- Added `nextCleanSessionOffset` and `appendCleanSessionChunk` in `src/cleanSession.ts`.
- Updated `src/App.tsx` to refresh a run from the prior `next_offset` and append the new chunk instead of replacing from the beginning.
- Added regression coverage in `src/cleanSession.test.ts` proving the next refresh uses prior `next_offset` and appended clean cards include both prior and new chunks.

Post-fix verification:

- `npx tsx src/cleanSession.test.ts`: PASS.
- `npm run test:frontend`: PASS.
- `npm run build`: PASS.
- `git diff --check`: PASS.
