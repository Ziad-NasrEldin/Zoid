# Feature Handoff: Brain link panel restore + layout fix

## Original request

Initial request: "get the link apple notes and safe import panel back, its actually useful, but make sure its visible and follows brand design"

Follow-up screenshot feedback: "/var/folders/b3/v4_9c_2n163g0q8bz_d235t80000gn/T/TemporaryItems/NSIRD_screencaptureui_RNX4Ah/Screenshot 2026-06-08 at 2.54.46 AM.png ITS LITERALLY BURIED BEHIND OTHER ELEMENTS, FIX OUR SHIT"

## Implementation summary

- Restored the Brain Apple Notes linking panel as a real workflow surface.
- The panel renders when the Brain store loads and includes: `Link Apple Notes folder`, `safe read/import`, list folders, Apple Notes folder dropdown, sync-mode dropdown, sync-mode helper copy, and `Link selected folder`.
- The panel uses the existing typed Apple Notes bridge/client functions and preserves fail-closed behavior.
- Fixed the follow-up layout bug where the panel was visually buried behind the `Sources` / `Brain Inbox` grid. The Brain workspace grid now gives the link panel its own auto-height row before the inbox grid, uses auto rows for overflow content, and keeps the link panel in its own stacking context.
- Added a scaffold CSS guard so the Brain link panel cannot regress into the old `minmax(0, 1fr)` row that allowed overlap.

## Changed files

- `src/brain/BrainWorkspace.tsx`: restored the Apple Notes link/safe-import panel and wiring.
- `src/brain/BrainWorkspace.behavior.test.tsx`: added behavior coverage for restored panel copy, folder listing, sync-mode selection, and link payload.
- `src/App.css`: restored branded link-panel styling and fixed workspace row ownership/stacking to prevent overlap with later panels.
- `src/scaffold.test.ts`: added source guard for the no-overlap layout invariant.

## How to test

- `npm run test:frontend`
- `npm run build`
- `npm run tauri:build`
- Relaunch `/Applications/Zoid 25.app`, open Brain, and visually confirm the Apple Notes link panel sits between the status line and the Sources/Brain Inbox grid with clear vertical separation.

## Tests run

- `npm exec -- tsx src/scaffold.test.ts`: PASS
- `npm exec -- tsx src/brain/BrainWorkspace.behavior.test.tsx`: PASS
- `npm run test:frontend`: PASS
- `npm run build`: PASS
- `npm run tauri:build`: PASS
- Native app relaunch: PASS; `/Applications/Zoid 25.app` process running.
- Native screenshot proof: PASS; `/tmp/zoid-brain-link-panel-layout-fix.png` shows the Brain page with the link panel fully visible and separated above the Sources/Brain Inbox grid.

## Git info

- Branch: current working tree, not committed by this task.
- Diff base: current repository working tree.

## Frontend/backend/database notes

- Frontend route/component: Brain workspace.
- Backend: no new backend commands in the follow-up fix; restored panel uses existing Apple Notes bridge commands.
- Database: not applicable.

## Reviewer focus areas

- Confirm the follow-up CSS change actually prevents the link panel from sharing a constrained `1fr` row with later Brain content.
- Confirm the restored panel remains brand-aligned and visible.
- Confirm tests/build still pass after the layout fix.

## Fix cycle notes

Follow-up fix after screenshot feedback:
- Changed Brain workspace rows to `auto auto auto minmax(0, 1fr)` plus `grid-auto-rows: auto` and `align-content: start`.
- Added `position: relative; z-index: 1` to `.brain-link-panel`.
- Added scaffold guard for the no-overlap CSS invariant.
- Rebuilt, reinstalled, relaunched the native app, and captured native screenshot proof.
