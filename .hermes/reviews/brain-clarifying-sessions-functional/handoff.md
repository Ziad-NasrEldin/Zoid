# Feature Handoff: Brain Clarifying Sessions functional workflow

## Original request

Page Feedback for `/`: Clarifying Sessions section feels useless; re-implement to make sure it is functional.

## Implementation summary

- Reworked the Brain Clarifying Sessions panel from a passive placeholder list into an active workbench.
- Added selectable session tabs, current-question form, answer saving, transcript rendering, generated agent brief display, and Copy brief action.
- Added a typed Tauri command/client path for saving answers: `answer_brain_clarifying_session`.
- Backend now consumes open questions, records user answers in the transcript, generates a deterministic agent brief when all questions are answered, marks the session `briefReady`, and marks linked candidates `readyForAgent`.
- The workflow remains fail-closed: Brain prepares/copies a brief only; it does not execute Hermes or create a Hermes session automatically.

## Changed files

- `src/brain/BrainWorkspace.tsx`: active Clarifying Sessions workbench UI, answer form, copy brief, state wiring.
- `src/brain/brainClient.ts`: typed `answerBrainClarifyingSession` invoke wrapper.
- `src/App.css`: workbench/session tabs/question/brief styling.
- `src/scaffold.test.ts`: updated source guards for the now-functional surface.
- `src/brain/BrainWorkspace.behavior.test.tsx`: UI behavior coverage for starting, answering, and rendering a generated brief.
- `src-tauri/src/lib.rs`: answer command, brief generation helper, command registration, Rust unit coverage.

## How to test

- `npm run test:frontend`
- `npm run test:rust`
- `npm run build`
- `npm run tauri:build`
- Relaunch `/Applications/Zoid 25.app`, open Brain, extract task candidates, start clarifying questions, answer questions, verify a brief appears and can be copied without launching Hermes.

## Tests run

- `npx tsx src/brain/BrainWorkspace.behavior.test.tsx`: PASS.
- `npm run test:rust`: PASS, 73 passed / 1 ignored after adding the new Rust test.
- `npm run build`: PASS.
- `npm run tauri:build`: PASS; app bundle rebuilt.
- `/Applications/Zoid 25.app` relaunched; process path verified with `pgrep` as `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- `npm run test:frontend`: BLOCKED by unrelated current scaffold guard expecting old `SESSION_FIGURE_PORTRAITS`; current repo has `SESSION_UKIYO_PORTRAITS` in `src/agents/sessionPortraits.ts`. The focused Brain behavior test passes.

## Git info

- Branch: current working tree, heavily dirty before this task.
- Commit SHA: not committed.
- Diff base: current repository working tree has many unrelated dirty/untracked Zoid changes; review should scope to the files listed above and avoid treating unrelated dirty state as part of this feature.

## Frontend/backend/database notes

- Frontend route/component: Brain workspace (`src/brain/BrainWorkspace.tsx`).
- Backend commands: new Tauri command `answer_brain_clarifying_session(sessionId, answer)`.
- Database: none; uses existing Brain store persistence path.

## Reviewer focus areas

- Does the Clarifying Sessions panel now provide a real answer -> transcript -> brief workflow instead of placeholders?
- Does it remain truthful that Hermes is not executed automatically?
- Are Tauri invoke argument names camelCase-compatible with the frontend wrapper?
- Are linked task candidates moved to `readyForAgent` only after all questions are answered?
- Does the UI remain usable with no sessions, open-question sessions, and brief-ready sessions?

## Fix cycle notes

Initial handoff.
