# Handoff: Code default-branch dropdown sizing

## User request
The previous default-branch editor/dropdown fix made the control take too much page width and still had visibility issues. Tighten the dropdown/menu sizing and make it readable without clipping.

## Scope
Review only the focused default-branch editor/dropdown sizing and visibility changes in:
- `src/code/CodeWorkspace.tsx`
- `src/App.css`
- `src/scaffold.test.ts`

The repo has many unrelated dirty/untracked files from previous Zoid work; do not treat those as part of this review.

## Changes made
- Added a scoped `className="default-branch-dropdown"` to the default-branch `GlobalDropdown` in `RepositoryMeta`.
- Changed the default-branch editor from the prior wide `width: min(100%, 620px)` / flexible dropdown to a compact fit-content grid:
  - dropdown column: `clamp(230px, 28vw, 360px)`
  - Save/Cancel remain fixed auto columns
  - editor maxes at its metadata column instead of spanning the card/page
- Scoped menu styles to `.default-branch-dropdown`:
  - menu aligns to trigger with `width: 100%`
  - capped height `min(188px, 32vh)`
  - paper background + ink text for readability
  - selected row gets pale blue + inset blue accent
  - option rows explicitly `width: 100%` so selected/hover background spans the whole menu
- Preserved mobile fallback: at `max-width: 560px`, editor/dropdown/buttons become full width within the narrow column.
- Updated `src/scaffold.test.ts` source guards to check the compact sizing and scoped dropdown class.

## Verification performed
- `npm run build` — passed.
- `npm run test:frontend` — passed.
- Browser dev server at `http://127.0.0.1:1420/` — HTTP 200.
- Browser DOM/geometry checks with seeded repository and default-branch editor open:
  - repository card width: 811px
  - default branch grid item width: 386px
  - editor width: 386px, left/right 435/822
  - dropdown width: 358px, left/right 443/802
  - opened menu width: 358px, left/right 443/802
  - option row width: 356px after fix, background readable, full-row selected state
  - editor/menu overflow visible; not clipped
- `npm run tauri:build` — passed.
- Replaced `/Applications/Zoid 25.app` with the newly built bundle and launched it.
- Verified running installed process: `/Applications/Zoid 25.app/Contents/MacOS/zoid` PID 6371.
- Verified macOS window exists via System Events: `Zoid 25`.

## Browser-only note
In browser preview, branch listing reports `Cannot read properties of undefined (reading 'invoke')` because Tauri invoke is unavailable outside native. This is expected for browser-only preview; the edit UI still renders fallback state for visual verification.

## Acceptance criteria
- The default-branch editor should no longer take the whole page/card.
- Dropdown/menu should remain compact, aligned to the trigger, readable, and unclipped.
- Save/Cancel should remain visible next to the dropdown on desktop and stack on narrow screens.
