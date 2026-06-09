# Feature Handoff: Brain sumi-e scroll + red + stronger brush pass

## Request
User reported the Brain page could not scroll and wanted:
- Scroll functionality.
- Scrollbar styled to match the sumi-e design system.
- Red as a subsidiary/main color alongside white and black, informed by `/Users/ziadnasreldin/Zoid/Assets/Agent Avatars`.
- Stronger brush effect because the brand is sumi-e / Japanese ink brush painting.
- Think first, plan, execute, then spawn an agent to verify and review.

## Scope
Brain page only. Do not roll out globally.

## Avatar-derived direction
The avatar contact sheet uses a tight sumi-e palette:
- Paper white/off-white.
- Ink black/gray wash.
- Seal red / warm red accent.
Visual motifs: brush arcs, ink wash, dry-brush streaks, red seal marks, restrained monochrome composition with red as an intentional accent.

## Implementation summary
Primary files changed:
- `src/App.css`
- `src/scaffold.test.ts`

Earlier Brain pilot files still relevant:
- `src/brain/BrainWorkspace.tsx`
- `src/brain/BrainWorkspace.behavior.test.tsx`

Changes made in `src/App.css`:
- `.brain-sumi-e` now owns vertical scrolling:
  - `height: 100vh`
  - `min-height: 0`
  - `overflow-y: auto`
  - `overflow-x: hidden`
  - `overscroll-behavior: contain`
  - `scrollbar-gutter: stable`
- Added sumi-e scrollbar styling:
  - Firefox: `scrollbar-color: var(--brain-seal) rgba(13,10,10,0.08)`, `scrollbar-width: thin`
  - WebKit: custom track + thumb using ink black, seal red, deep red.
- Promoted red into the Brain palette:
  - `--brain-seal: #c23a2e`
  - `--brain-seal-deep: #8f211a`
  - `--brain-seal-wash: rgba(194, 58, 46, 0.12)`
- Strengthened original brush effects without external assets:
  - paper/ink wash background layers on `.brain-sumi-e`
  - top diagonal ink wash via `.brain-sumi-e::before`
  - bottom dry brush stroke via `.brain-sumi-e::after`
  - hero brush divider via `.brain-hero::before`
  - stronger ink mark with red seal via `.brain-ink-mark::after`
  - brush underline on Brain primary/secondary actions
  - red seal marker on primary action
  - red seal marker on status line
  - brush-strip tops on panels/link panel/error/empty states
- Fixed post-implementation action/status overlap by increasing hero min height to `clamp(540px, 42vw, 620px)`.
- Fixed reviewer-found row clipping risk:
  - desktop `.brain-note-row` now uses explicit three-column grid: content + action + badge.
  - mobile/narrow media query stacks `.brain-source-row` and `.brain-note-row` to one column and left-aligns actions/badges.

Changes made in `src/scaffold.test.ts`:
- Added static contract checks for Brain scroll, scrollbar, red palette tokens, and stronger brush selectors.

## Verification already run by implementation agent
Commands:
- `npm run test:frontend` passed.
- `npm run build` passed after resolving an unrelated current-tree Content import/casing issue that blocked TypeScript.

Browser checks:
- Opened local dev server at `http://127.0.0.1:1420/`.
- Navigated to Brain.
- Console geometry checks showed:
  - `canScroll: true`
  - `overflowY: auto`
  - internal scrollTop can move to 260.
  - `scrollbarColor: rgb(194, 58, 46) rgba(13, 10, 10, 0.08)`
  - red token `--brain-seal: #c23a2e`
  - hero brush exists.
  - shell brush wash exists.
  - ink mark red seal exists.
  - no document horizontal overflow.
  - no reference line overflow.
  - no ink mark / hero copy overlap.
  - no action / status overlap after the final min-height fix.

Visual checks:
- First viewport shows stronger sumi-e language: paper white, black typography, gray ink arcs/washes, red seal accents.
- Scrolled state at internal `scrollTop=220` shows lower content, confirming content is reachable.

## Known caveat
- Current local Brain state shows native Apple Notes bridge unavailable in the browser/dev environment, which is expected. The design still needs to handle bridge error and empty states cleanly.

## Review request
Be ruthless. Verify both function and design:
1. Does Brain scroll for real inside the fixed Zoid shell?
2. Is the scrollbar actually styled and aligned with the sumi-e palette?
3. Are white/black/red clearly the main palette, with red subsidiary but meaningful?
4. Is brush effect stronger without turning into visual noise?
5. Are there overlaps/clipping issues in first viewport and scrolled state?
6. Is the implementation scoped to Brain only?
7. Are there any regressions from the earlier approved Brain pilot?
8. Run tests/build if possible.

Return verdict: APPROVED or REQUEST_CHANGES, with required fixes if any.