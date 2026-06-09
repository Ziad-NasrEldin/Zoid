# Handoff: model controls redesign

Scope: Zoid 25 Page Feedback for `/`, specifically the open `Model controls command panel` reached from the Hermes footer model quick switch.

User feedback:
- The current model controls panel is horrible, messy, and unorganized.
- Redesign it completely and structure it properly.

Implementation:
- `src/agents/AgentsHermesScreen.tsx`
  - Model panel now uses `zoid-native-command-panel--model` for a dedicated layout rather than the generic command panel body.
  - Added structured header with bilingual kicker.
  - Added `model-command-current` summary strip for Provider / Model / Reasoning.
  - Added `model-command-controls` section with numbered heading `01`, clear title `Choose runtime defaults`, and three grouped control cards.
  - Kept the real existing `GlobalDropdown` controls and immediate persistence handlers: provider, model, reasoning.
  - Added `model-command-contract` section with numbered heading `02` explaining persistence/source behavior.
  - Preserved status/error roles.
  - Also fixed pre-existing TypeScript call-site errors in this file around queued slash prompts and `runSlashCommand(activeSession, command)` so build remains green.
- `src/App.css`
  - Replaced the old generic small panel styling with a structured model-specific panel: 760px command sheet, section dividers, current-state strip, grouped field cards, compact typography, responsive one-column fallback.
  - Follow-up fix after first critique: `.zoid-native-command-panel--model` uses `overflow: visible` so dropdown menus are not clipped by the redesigned sheet.
- `src/scaffold.test.ts`
  - Added source guards so the model panel cannot regress to the previous generic messy layout.
- `src/code/CodeWorkspace.tsx`
  - Fixed pre-existing build/test blockers by restoring `linkedRepositoryId` / `onLinkedRepositoryIdChange` destructuring and a `Use for Agents` repository action required by existing scaffold guards.

Verification run:
- `npx tsx src/scaffold.test.ts` passed.
- `npm run build` passed.
- `npm run test:frontend` passed.
- Browser validation at `http://127.0.0.1:1420/`:
  - Opened Agents workspace and clicked footer quick model button.
  - Panel class: `zoid-native-command-panel zoid-native-command-panel--model`.
  - Panel width: 760px, height about 489px, no horizontal overflow.
  - Panel computed overflow: `visible`.
  - Current strip present with 3 columns.
  - Controls section present with Provider / Model / Reasoning cards.
  - Persistence contract present.
  - Reasoning dropdown menu opens and is not clipped by panel overflow.

Review history:
- First critique returned REQUIRED_FIXES for `overflow: hidden` clipping dropdowns.
- That required fix has been applied and reverified.

Review request:
- Read the current source and this handoff.
- Do not modify files.
- Return verdict APPROVED or REQUIRED_FIXES.
- Required fixes should focus on whether the redesign is truly structured, usable, and preserves the real model/provider/reasoning behavior.
