# Hermes dropdown red divider overlap fix

## Scope
User provided screenshot showing a red horizontal line crossing the open repository dropdown items.

## Root cause
The red line is the Hermes sumi-e topbar divider pseudo-element:
`.agents-sumi-e .hermes-topbar::after`.
It sits at the bottom of the topbar and remains visible while the repository dropdown opens downward, so it visually cuts across the menu.

## Changes
- `src/App.css`
  - Added `.agents-sumi-e .hermes-topbar:has(.zoid-dropdown-menu)::after { opacity: 0; }`
  - Keeps the decorative topbar line when the menu is closed.
  - Hides it only while a dropdown menu is open, so it cannot overlap menu rows.
- `src/scaffold.test.ts`
  - Added source guards for the open-menu topbar divider suppression.

## Validation
- `npm run build` passed.
- `npm run test:frontend` passed.

## Review focus
Confirm this resolves the screenshot issue without removing the closed-state sumi-e divider and without regressing the repository dropdown overlay/z-index behavior.
