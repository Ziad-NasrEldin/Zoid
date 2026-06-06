# Feature Handoff: Content Workspace Stitch frontend implementation

## Original request

"ok great , i want you to design them now in the actual zoid app
just focus on the design and the full frontend for now, later we likn the baack to the front"

Context from prior Stitch inspection: core Content Workspace count is 16 actual desktop UI screens in Stitch project `2534809720873389640` (`Zoid macOS Desktop Sitemap`).

## Implementation summary

- Added a frontend-only Content Workspace design surface inside the existing `content` workspace.
- Mapped all 16 Stitch Content Workspace screens into visible designed frontend surfaces, not only names:
  - dashboard hero/health surface
  - brand management surface
  - campaign wizard steps
  - advanced editor/policy surface
  - slot calendar grid
  - today pipeline board
  - piece detail/adaptations split view
  - override/editor diff surface
  - approval queue rows
  - dry-test report/check surface
  - run-now modal surface
  - recovery/failure timeline
  - OmniSocials mapping table
  - evidence/artifact library grid
  - agent execution rail
  - automation mirror run cards
- Kept the existing real/native content panels below the design surface so static design-copy cannot be confused with live state.
- Added an always-visible disclosure: operational cards are design-copy only; real native content state remains in the live status panels; no external publishing is implied.
- Added view-model metadata/tests for source project, screen count/order, disclosure, design-system tokens, and screen surface kinds.
- Preserved existing native content actions and fail-closed OmniSocials/schedule/verification copy.

## Changed files

- `src/contentWorkspace.ts`: adds `buildContentWorkspaceDesignView()` and the 16-screen frontend design model with per-screen `surfaceKind`.
- `src/contentWorkspace.test.ts`: asserts source project, 16-screen order, sample-data disclosure, design tokens, per-screen key regions, and the 16 designed surface kinds.
- `src/App.tsx`: renders the new Content Workspace design hero, nav chips, group KPIs, 16 screen cards, actual per-screen designed preview surfaces, and live native panels.
- `src/App.css`: adds Content Workspace layout/styles and per-screen preview surface styles using existing Apple-design-analysis/Zoid tokens.

Note: repository already contains unrelated modified/untracked files from prior work (for example Today screen and task bridge files). Review this feature against the files above.

## Critique fix cycle

Initial critique verdict: `REQUEST_CHANGES`.

Required fix from critique: the first pass rendered a 16-card catalog rather than actual designed frontend screen surfaces.

Fix applied:
- Added `ContentScreenPreview` in `src/App.tsx`.
- Added `surfaceKind` to each of the 16 Content Workspace screen records in `src/contentWorkspace.ts`.
- Added concrete mini-layouts/surfaces for all 16 screen kinds.
- Added `.content-screen-surface` CSS and surface-specific styles in `src/App.css`.
- Added test coverage for the 16 surface-kind mapping in `src/contentWorkspace.test.ts`.

## How to test

- `npm run test:frontend`
- `npm run build`
- `npm run dev`, open `http://127.0.0.1:1420/`, click Content.
- In browser, verify:
  - Content page headline says `Autonomous content operations, designed as 16 frontend screens.`
  - disclosure includes `design-copy only`.
  - exactly 16 `.content-screen-card` elements render.
  - exactly 16 `.content-screen-surface` elements render.
  - live native panels render below the screen-card grid.

## Tests run

- `npm run test:frontend`: PASS. Output included `contentWorkspace tests passed` and all frontend test scripts completed.
- `npm run build`: PASS. `tsc && vite build`, 64 modules transformed, build completed.
- `curl -I --max-time 5 http://127.0.0.1:1420/`: PASS, HTTP 200.
- Browser smoke at `http://127.0.0.1:1420/` -> Content: PASS.
  - Browser console: no JS errors observed.
  - DOM probe after fix: `{ screenCards: 16, designedSurfaces: 16, disclosure: true, livePanels: 5 }`.
  - Visual/snapshot inspection: page renders hero, section chips, group KPIs, 16 screen cards, 16 designed screen surfaces, and live state panels.

## Git info

- Branch: `main`
- Current HEAD before commit: `106732c`
- Commit SHA: not committed.

## Frontend/backend/database notes

- Frontend routes/components: existing Content workspace in `src/App.tsx`; no new route introduced.
- Backend endpoints/services: no backend changes; native bridge actions remain as existing fail-closed/live panels.
- Database tables/migrations: no database changes.
- Sample/static data: the 16 screen-design surfaces are static design-copy and visibly disclosed as non-live.

## Reviewer focus areas

- Confirm all 16 Stitch Content Workspace screens are present in order and not mixed with adjacent Agents Workspace screens.
- Confirm each screen has a concrete designed surface, not only a title/description card.
- Confirm sample/design-copy disclosure is visible and truthful.
- Confirm real native content state panels remain present and fail closed when unavailable.
- Confirm design does not imply live publishing or connected OmniSocials.
- Confirm CSS does not break existing Today/Tasks/Notes/Files layout.
