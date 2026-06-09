# Hermes repository dropdown overlay fix

## Scope
Page Feedback reported the repository dropdown options (`Unlinked / 未接続` and repo items) are hidden below the chat window.

## Root cause
The repository dropdown menu is rendered inside the Hermes topbar, then extends downward into the chat workspace. The chat workspace is the following sibling and could paint above/clip the dropdown visually because the topbar did not create a higher stacking layer for its open menu.

## Changes
- `src/App.css`
  - Made `.agents-sumi-e .hermes-topbar` an explicit high stacking context:
    - `position: relative`
    - `z-index: 70`
    - `overflow: visible`
  - Promoted the repository dropdown and menu above the chat workspace:
    - `.agents-sumi-e .repository-link-control--topbar .zoid-dropdown { z-index: 90 }`
    - `.agents-sumi-e .repository-link-control--topbar .zoid-dropdown-menu { z-index: 220 }`
  - Kept dropdown menu bounded and internally scrollable:
    - `max-height: min(360px, calc(100vh - 255px))`
    - `overscroll-behavior: contain`
  - Put `.agents-sumi-e .chat-workspace` on a lower explicit layer (`z-index: 1`) so it cannot cover the menu.
- `src/scaffold.test.ts`
  - Added source guards for the topbar stacking/overflow and dropdown menu z-index/max-height behavior.

## Validation
- `npm run build` passed.
- `npm run test:frontend` passed.

## Review focus
Confirm the open repository dropdown paints above the chat workspace, remains scrollable for many repositories, and does not regress the recent topbar spacing/bilingual-label work.
