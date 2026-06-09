# Feature Handoff: Settings scrollability and typography refinement

## Original request

"ok but some pages are not scrollable so fix that, because i cant see the contents at the bottom , also it needs a typeset enhances, use impecccable typefaces on all the settings pages
because its all in bold right now and it doesnt feel or look good reading anything"

## Implementation summary

- Fixed the Settings workspace scroll model so the settings shell owns a real `height: 100vh` scroll container while its header/form/content rows size to `max-content`.
- Removed sticky settings heading behavior that visually overlapped tabs/overview/content during scrolling.
- Fixed the grid row sizing bug where active tab panels overflowed the workspace row and the save status appeared before/over content.
- Added Settings-scoped typeface variables using readable system/Inter/SF fallbacks and moved settings text away from the heavy monospace style.
- Reduced Settings heading, label, helper, tab, overview, provider, archive, dropdown, and field weights so body/help text reads normally instead of all-bold.
- Preserved the existing real Hermes/profile/provider/archive data wiring. This is a CSS-only behavior/visual refinement in `src/App.css`.

## Changed files

- `src/App.css`: settings scroll container/row sizing plus Settings-scoped typography and weight refinements.

## How to test

1. Run `npm run test:frontend`.
2. Run `npm run build`.
3. Run `npm run test:rust`.
4. Start/verify local dev server on `http://127.0.0.1:1420/`.
5. Open Settings and click all seven tabs.
6. For each tab, confirm:
   - `.settings-archive-shell` has `overflow-y: auto`.
   - `scrollHeight > clientHeight` when content is longer than the viewport.
   - Scrolling to bottom reaches the bottom content/status.
   - Active panel no longer overlaps the page-level save status.
   - Settings labels/body/help text use normal readable weights, not 800/900 everywhere.

## Tests run

- `npm run test:frontend`: PASS.
- `npm run build`: PASS; Vite emitted the existing chunk-size warning.
- `npm run test:rust`: PASS; 28 Rust tests passed.
- `git diff --check -- src/App.css src/App.tsx`: PASS.
- Browser DOM/geometry check on `http://127.0.0.1:1420/`: PASS.
  - 7 tabs found.
  - Every tab scrolls to bottom.
  - Every tab reported `topOverlap: false` against the page-level save status.
  - Settings shell `overflow-y: auto`.
  - Body/helper weights around 420–460, labels 500, headings 600.
- Browser visual checks:
  - Top of Settings: no heading/tabs/content overlap; typography visibly lighter than previous all-bold pass.
  - Bottom of Settings after scrolling: bottom fields and save status visible, not clipped.

## Git info

- Branch: main.
- Commit SHA: not committed.
- Diff base: current working tree has unrelated pre-existing dirty/untracked files; review should scope to `src/App.css` for this task.

## Frontend/backend/database notes

- Frontend: CSS-only refinement for Settings page layout/typography.
- Backend: not touched.
- Database/native commands: not touched.
- Hermes/profile/provider/archive wiring: unchanged.

## Reviewer focus areas

- Verify the scroll fix is real and not just hidden overflow: long tabs must reach bottom content.
- Verify the page-level save status sits after tab content and does not overlap it.
- Verify typography is scoped to Settings and does not accidentally degrade global Zoid navigation/chat styles.
- Verify type hierarchy remains readable and not all bold.
- Verify no fake data or behavioral rewiring was introduced.

## Fix cycle notes

Initial implementation revealed via browser/vision that the sticky heading and grid row sizing still overlapped content. Follow-up fixes made the settings shell/form/workspace rows `max-content`, removed sticky heading overlay, and reran checks successfully.
