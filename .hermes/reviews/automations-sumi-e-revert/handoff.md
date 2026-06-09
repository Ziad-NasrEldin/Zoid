# Automations sumi-e revert handoff

## Scope
User reported the Automations page was messed up and asked to revert it back to the sumi-e design.

## Changed by this fix
- `src/automations/AutomationsWorkspace.tsx`
  - Restored the page wrapper from `automation-kujoyama` to `automation-sumi-e`.
  - Restored the sumi-e hero ritual/clock mark (`automation-ink-clock`).
  - Restored the original source-of-truth copy: `Provider-owned schedules · protected system jobs · watcher state is read-only`.
  - Preserved the earlier functional safety fixes: request guards, action serialization, bridge-preview error copy, guarded destructive modal behavior, blank-name fallbacks, and timestamp parsing.
- `src/App.css`
  - Replaced the Automations Kujoyama boxed/card style block with scoped sumi-e ink/paper/red-seal styling.
  - Uses sumi tokens, serif typography, ink wash background, red seal accents, clock/ritual mark styling, thin paper rules, custom scrollbar, and responsive header/grid rules.
  - Kept controls, filters, cards, modals, status lines, and edge panels scoped under `.automation-sumi-e`.
- `src/automations/AutomationsWorkspace.behavior.test.tsx`
  - Restored assertions for `.automation-sumi-e`, `.automation-ink-clock`, and the sumi-e constraint copy.
  - Preserved regression coverage for the functional fixes.

## Verification already run
- `npm run test:frontend` passed.
- `npm run build` passed.
- Relaunched local Vite dev server on `http://127.0.0.1:1420`.
- Browser inspection of Automations page:
  - `.automations-workspace-shell automation-sumi-e` present.
  - `.automation-ink-clock` present.
  - computed background is ink/paper radial sumi-e wash.
  - computed font family is sumi serif stack.
  - no horizontal overflow reported.
  - browser console empty after inspection.

## Notes
- Browser preview shows the expected truthful bridge-blocked alert because the native Tauri Hermes bridge is not available in browser mode.
- Repo has broad unrelated dirty work; review only this scoped fix unless widening scope explicitly.
