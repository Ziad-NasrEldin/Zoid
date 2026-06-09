# Feature Handoff: Slash command panel top clamp

## Original request

User provided Page Feedback for `/` and explicitly said to ignore the first historical-figure chat icon request and only focus on the second request:

> This section has been cut off at the top because of the new boxy design, so can you please fix it?

The flagged element is the Hermes composer deep panel for `Slash commands` (`div.composer-deep-panel`) at `tauri://localhost`, viewport `1706×975`.

## Implementation summary

- Added a dedicated `composer-deep-panel--slash` class for the Slash commands deep panel.
- Changed the slash panel from a generic tall scrolling panel into a bounded grid with fixed header/search/helper rows and a scrollable command list.
- Added runtime measurement in `ChatComposer` so the slash panel sets `--composer-slash-panel-max-height` based on the available space above the composer inside `.chat-workspace`; this prevents the panel top from being clipped by the workspace overflow when the composer is near the bottom.
- Kept the existing boxy Zoid design, shadow, and right-aligned drop-up behavior.
- Added source guard coverage in `ChatComposer.slash.test.tsx` for the slash-specific panel class, dynamic CSS variable, and internal scroll behavior.

Known limitation: at very short preview/browser heights, the panel intentionally shrinks enough to stay unclipped; the command list may be very small/zero-height until the window is taller. This is preferable to top clipping and at the user-reported 975px viewport there is substantially more available vertical space.

## Changed files

- `src/agents/ChatComposer.tsx`: adds form measurement, slash-panel max-height state, and the slash-specific panel class/style.
- `src/App.css`: adds slash-panel grid/max-height/internal-scroll rules and box sizing for panels.
- `src/agents/ChatComposer.slash.test.tsx`: adds source guard assertions for the non-clipping slash panel behavior.

## How to test

- Run `tsx src/agents/ChatComposer.slash.test.tsx`.
- Run `npm run build`.
- Open the app, navigate to Agents/Hermes chat, click `+`, choose `Slash commands`, and verify the panel top is inside the chat workspace instead of clipped.
- Browser geometry smoke used at the current preview viewport:
  - panel top: `217`
  - workspace top: `205.03125`
  - top clearance: `11.96875`
  - `notCut: true`

## Tests run

- `tsx src/agents/ChatComposer.slash.test.tsx`: PASS.
- `npm run build`: PASS.
- `git diff --check -- src/App.css src/agents/ChatComposer.tsx src/agents/ChatComposer.slash.test.tsx`: PASS.
- `npm run test:frontend`: FAIL due unrelated existing scaffold assertion: `Hermes Finder sidebar must not render the useless Up toolbar button` in `src/scaffold.test.ts:341`. This task intentionally did not modify the Finder sidebar because the user asked to ignore the first feedback item and only fix the slash panel.

## Git info

- Branch: current working tree; repo has many unrelated dirty/untracked files.
- Commit SHA: not committed.
- Diff base: current working tree scoped to the three changed files above.

## Frontend/backend/database notes

- Frontend only: React/CSS changes in the Hermes composer.
- Backend: not applicable.
- Database: not applicable.

## Reviewer focus areas

- Confirm the slash commands panel can no longer be clipped at the top by `.chat-workspace { overflow: hidden; }`.
- Confirm the command list remains scrollable inside the bounded panel.
- Confirm the generic attachment/settings/usage panels are not accidentally changed.
- Confirm source guards and build are sufficient for this small scoped Page Feedback fix.

## Fix cycle notes

Initial handoff for review.
