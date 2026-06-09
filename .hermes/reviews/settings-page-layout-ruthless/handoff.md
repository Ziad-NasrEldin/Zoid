# Settings page layout implementation handoff

Feature slug: settings-page-layout-ruthless

## User request
Implement the ruthless Impeccable /layout review fixes for the Zoid 25 Settings page.

## Scope changed
- `src/App.tsx`
  - Replaced the Settings shell class from `settings-sumi-e` to `settings-operational-shell`.
  - Renamed top hero title from `Profile, Memory & Soul` to `Settings`.
  - Removed decorative `settings-ink-mark` from the Settings hero.

- `src/App.css`
  - Added `.settings-operational-shell` layout system.
  - Collapsed Settings hero into a compact operational header.
  - Restored product-register typography and color treatment: display/mono-forward, cobalt active states, no sumi-e serif override for active Settings.
  - Converted Settings navigation from seven horizontal cards to a vertical ruled list on desktop.
  - Kept overview metrics secondary in the left rail below navigation instead of ahead of active fields.
  - Preserved responsive collapse to two-column tabs at tablet and one-column at mobile.
  - Removed Settings-specific Impeccable detector side-tab warnings introduced by the new implementation by replacing `border-left` accents with inset shadows.
  - Neutralized the inactive `settings-sumi-e .settings-reference-line` detector warning from `border-left` to `border-top`.

- `src/scaffold.test.ts`
  - Updated the profile surface scaffold assertion from removed copy `Profile, Memory & Soul` to the new `settings-operational-shell` class.

## Verification already run
- `npm run test:frontend && npm run build` passed.
- `npm run test` passed:
  - frontend suite passed.
  - Rust suite passed: 73 passed, 1 ignored.
- `curl -I --max-time 5 http://127.0.0.1:1420/` returned HTTP 200.
- Browser DOM verification on live app:
  - `.settings-operational-shell` present.
  - Settings hero heading is `Settings`.
  - `.settings-ink-mark` absent.
  - Settings nav grid columns computed as `230px` on desktop, i.e. vertical list.
  - 7 Settings tabs present.
  - Active tab background computed as cobalt `rgb(53, 88, 162)`.
  - First Identity input visible without scrolling.
  - Browser console had no JS errors; only Vite/React dev info logs.
- `npx --yes impeccable detect --json src/App.tsx src/App.css src/providers/ProvidersSettings.tsx` still exits 2 because of existing global/non-Settings CSS warnings, but the active Settings implementation warnings introduced in this pass were cleared. Remaining warnings are outside Settings or inactive legacy/global CSS.

## Review focus requested
Critique this implementation against Impeccable `/layout` and `/critique`, and the Zoid `PRODUCT.md` / `DESIGN.md` product register. Be ruthless. Identify Required fixes only if they should block delivery.
