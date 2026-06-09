# Feature Handoff: Brain page Impeccable layout pass

## Request
Run the Impeccable layout skill/pass specifically on the Zoid 25 Brain page because its layout needed deep, thorough enhancement.

## Scope
- Target repo: `/Users/ziadnasreldin/Zoid`
- Target page/component: `src/brain/BrainWorkspace.tsx`
- Touched implementation file: `src/App.css`
- Diff artifact: `.hermes/reviews/brain-page-impeccable-layout/diff.patch`

## Impeccable layout findings addressed
1. Spacing: the Brain hero consumed too much of the first fold and pushed operational state below the fold.
2. Visual hierarchy: primary actions/status/error needed to enter the immediate first-viewport path instead of sitting under a decorative hero composition.
3. Grid/structure: the operational panels needed deterministic areas for sources, inbox, conflicts, candidates, and clarifying sessions instead of a loose two-column grid.
4. Rhythm: the page needed a tighter hero/status/control beat followed by denser panels.
5. Density: lists and task candidates needed calmer row/card density without horizontal overflow.

## Implementation summary
- Reduced the Brain page shell spacing and hero rhythm while preserving the sumi-e/ink/paper/red-seal visual system.
- Reworked the Brain hero grid so the mark is smaller and the action row participates in the layout rather than wasting a huge hero first fold.
- Converted the Brain operational grid into named areas: sources, inbox, conflicts, candidates, clarifying.
- Made Brain Inbox the larger primary working area, constrained long inbox lists with internal scrolling, and made candidates lay out as responsive dense cards.
- Tightened row padding/alignment and added min-width guards for row text/actions/badges.
- Replaced a Brain transcript side-tab style with a top rule to avoid a scoped Impeccable side-tab warning.

## Verification run by development agent
- `npx --yes impeccable detect --json src/brain/BrainWorkspace.tsx src/App.css` ran before/after. Remaining warnings are pre-existing/global or non-Brain legacy selectors; the Brain scoped transcript side-tab warning was removed.
- `npm run test:frontend` passed before critique, then passed again after fixing the critique-required link-actions grid overflow risk.
- `npm run build` passed before critique, then passed again after fixing the critique-required link-actions grid overflow risk.
- Browser preview at `http://127.0.0.1:1420`, Brain page: no document or Brain shell horizontal overflow (`0` delta for both), status/error visible in first fold, actions no longer overlap status after final min-height correction.

## Critique fix summary
- First critique verdict: `REQUEST_CHANGES` because `.brain-link-actions` used `repeat(auto-fit, minmax(180px, max-content))`, which could overflow when real Apple Notes folder/sync dropdown labels render.
- Fix applied: changed `.brain-link-actions` to `repeat(auto-fit, minmax(min(100%, 220px), 1fr))`, set the action grid to `width: 100%`, kept buttons start-aligned, and added min/max-width guards for dropdown surfaces in the link action grid.
- Re-ran `npm run test:frontend`, `npm run build`, and browser overflow probe after this fix.

## Known constraints
- Browser preview cannot access the Tauri Apple Notes native bridge, so the inspected live state is the fail-closed bridge-error state. This is truthful for browser preview; native/Tauri can still load real store state.
- Repo already had extensive unrelated dirty/untracked work before this scoped pass. This handoff covers only the Brain page CSS diff.

## Critique instructions
Review only the scoped Brain page layout delta unless a touched selector creates obvious cross-page regressions. Apply the `/impeccable layout` rubric: spacing, visual hierarchy, grid/structure, rhythm, density. Verdict must be `APPROVED`, `REQUEST_CHANGES`, or `BLOCKED_NEEDS_HUMAN`. If `REQUEST_CHANGES`, list concrete Required fixes only for this scoped delta.
