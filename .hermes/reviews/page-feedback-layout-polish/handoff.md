# Page feedback layout polish handoff

## Scope
Addressed Page Feedback for `/` at 1758×982:
- Rail language nav flickered/glitched during sidebar minimize/maximize.
- Hermes topbar needed to sit a little higher.
- Automations hero/header had excessive empty vertical space.
- Automations Refresh button had a red square that should be removed only from that button.

## Changes made
- `src/App.css`
  - Anchored `.rail-language` absolutely at the bottom center of `.ink-rail`, added stable transform/will-change/transition, and set `.ink-rail` to `height: 100vh` so the language switch no longer gets pushed below the viewport during collapsed rail layout.
  - Reduced `.agents-sumi-e` top padding from `clamp(20px, 2vw, 32px)` to `clamp(14px, 1.5vw, 24px)` to move the Hermes topbar upward.
  - Reduced Automations header min-height from `clamp(460px, 38vw, 580px)` to `clamp(240px, 22vw, 340px)`, tightened gap/padding, and reduced the decorative clock width to reduce empty space.
  - Added `.automation-header-actions .automation-refresh-button::after { content: none; }` so the red seal square is removed only from the header Refresh button while other primary buttons keep their styling.
- `src/automations/AutomationsWorkspace.tsx`
  - Added `automation-refresh-button` class to the header Refresh button.
- `src/scaffold.test.ts`
  - Added source guards for the rail anchoring, top padding, compact Automations header, smaller clock, and Refresh-only pseudo-element removal.
- `src/App.tsx`
  - Added missing `NumberProfileKey` type so release TypeScript build passes; this was an unrelated existing build blocker surfaced during verification.

## Browser evidence
Using the existing Vite server at `http://127.0.0.1:1420/`:
- Automations header computed at viewport used by browser probe: `top: 56.3125`, `height: 281.59375`, `minHeight: 281.6px` instead of the reported 580px header.
- Refresh button pseudo-element: `getComputedStyle(button, '::after').content === 'none'`.
- Hermes topbar after padding adjustment: `top: 19.1875px`.
- Rail language before and after collapse remained at same visible position in browser viewport: `top: 499`, `left: 23.421875`, `.ink-rail` height `577`, class changed from `zoid25-shell` to `zoid25-shell sidebar-collapsed`.

## Commands run
- `npx tsx src/scaffold.test.ts` — passed.
- `npx tsx src/automations/AutomationsWorkspace.behavior.test.tsx` — passed.
- `npm run build` — passed after adding `NumberProfileKey`.
- `npm run tauri:build` — passed; bundle built at `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.

## Known unrelated issue
- `npm run test:frontend` currently fails in `src/scaffold.test.ts` on an existing Hermes topbar source guard expecting `grid-template-columns: minmax(220px, 0.72fr) minmax(320px, 1fr) minmax(148px, max-content);` while current CSS has `grid-template-columns: max-content minmax(320px, 1fr) minmax(148px, max-content);`.
- Earlier a full test run also hit an unrelated agent file-manager assertion. The focused tests for this page feedback pass.

## Review request
Please review for:
- Whether the rail language fix truly avoids collapsed/expanded flicker without unintended sidebar layout regressions.
- Whether topbar upward movement is appropriately scoped to Hermes/Agents.
- Whether Automations header reduction preserves the sumi-e design while removing excessive empty space.
- Whether the red square removal is scoped only to the header Refresh button.
- Whether source guards are narrow and useful.
- Whether the unrelated `NumberProfileKey` addition is acceptable or should be separated.
