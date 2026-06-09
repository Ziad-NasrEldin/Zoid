# Hermes stats footer text alignment follow-up handoff

## Scope
User follow-up: the Hermes stats footer text was still not all on the same line and not correctly aligned inside containers.

## Changes made
- Kept the footer as deterministic grid.
- Fixed text-level alignment inside cells:
  - direct footer cells now use `line-height: 1`
  - generic footer labels inherit font size and line height
  - `.chat-stats-model-copy` is now `inline-flex` with centered alignment and `line-height: 1`
  - sumi-e footer labels changed from larger `12px inline-grid` labels to `inline-flex`, inherited `10px`, inherited letter spacing, and `line-height: 1`
- Strengthened `src/scaffold.test.ts` after critique to use selector-scoped `cssBlockHas(...)` checks instead of broad substring checks. It now guards the exact footer selector blocks for grid/stretch, cell flex/center/nowrap/line-height, label inherited font/line-height, model copy inline-flex alignment, and sumi-e label inherited metrics.

## Files touched for this follow-up
- `src/App.css`
- `src/scaffold.test.ts`

## Evidence collected
- Browser computed text-node probe on Agents page:
  - all footer stat text nodes: top `528`, bottom `538`, `fontSize: 10px`, `lineHeight: 10px`
  - Tune button text is within 0.5px: top `527.5`, bottom `537.5`, `fontSize: 9px`, `lineHeight: 9px`
  - model cell remains `display: grid`
- Browser screenshot visually shows the footer text now on the same line and vertically centered inside cells.
- `npx tsx src/scaffold.test.ts` passed.
- `npm run build` passed.

## Review request
Re-review only this footer text-alignment follow-up. The prior review requested selector-scoped guards; those have now been added. Confirm APPROVED or list exact remaining footer-slice required fixes.
