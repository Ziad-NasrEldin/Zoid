# Global Zoid Dropdown Handoff

## Request
Page feedback on `/` asked to replace the Code workspace default-branch native dropdown with the Zoid 25 design-system dropdown, add it as a global rule for this project, and wire up a global dropdown menu.

## Implementation
- Added shared `GlobalDropdown` component at `src/ui/GlobalDropdown.tsx`.
  - Branded trigger/menu with `data-global-dropdown="true"`.
  - Uses a button-triggered ARIA menu pattern: `aria-haspopup="menu"`, `role="menu"`, `role="menuitemradio"`, `aria-checked`, disabled-state support, outside-click close, Escape close, Enter/Space selection, and Arrow/Home/End keyboard movement.
- Added global `.zoid-dropdown*` styles in `src/App.css` using Zoid tokens: hard black borders, blue soft menu rail, blue chevron, yellow focus outline, compact sizing.
- Replaced native selects with `GlobalDropdown` in:
  - `src/code/CodeWorkspace.tsx`: default branch editor from the feedback target.
  - `src/agents/AgentsHermesScreen.tsx`: linked repository dropdown.
  - `src/agents/ChatComposer.tsx`: attachment action dropdown.
  - `src/App.tsx`: Settings access/approval mode dropdowns.
- Added behavior coverage at `src/ui/GlobalDropdown.behavior.test.tsx` using React + happy-dom.
  - Verifies accessible trigger name/expanded state, click open/selection/close, Enter open, Escape close, ArrowDown focus movement, disabled option, and disabled dropdown behavior.
- Updated `src/scaffold.test.ts` to enforce:
  - Default branch uses `GlobalDropdown`.
  - Agent repository link uses global dropdown styles.
  - `GlobalDropdown` keeps menu/menuitemradio structure, keyboard handlers, selected/disabled accessibility states.
  - `GlobalDropdown.behavior.test.tsx` exists and App/Composer reuse the component.
  - App/code/agents/composer surfaces do not introduce native `<select>` controls.
- Added `happy-dom` dev dependency and wired `npm run test:frontend` to run scaffold + dropdown behavior tests.
- Added project rule to `CONTEXT.md`: dropdowns in Zoid 25 must use `src/ui/GlobalDropdown.tsx` + `.zoid-dropdown*`, not native `<select>` or one-off dropdown styling.
- Saved the Zoid dropdown convention into durable memory.

## Verification performed
- Static search under `src` found no source `<select` except the scaffold guard itself ✅
- `npm run test:frontend && npm run build` ✅
- `npm run tauri:build` ✅
- Reinstalled and relaunched `/Applications/Zoid 25.app`; running PID was confirmed as `/Applications/Zoid 25.app/Contents/MacOS/zoid` ✅
- Browser dev server check was attempted; direct page interaction was blocked by the feedback overlay state, but test coverage and source search verify the global dropdown behavior and no native selects in app surfaces.

## Critique cycle
- Initial critique report: `REQUIRED_FIXES` due to invalid listbox semantics and missing behavior tests.
- Fixes applied:
  1. Switched from listbox/options implemented as buttons to a valid menu/menuitemradio pattern.
  2. Added explicit behavior/accessibility tests and wired them into `test:frontend`.

## Notes / boundaries
- Repository already had many unrelated dirty/untracked files before this task; no commit was made.
- A dev server on port `1420` may already be running from previous app work; one attempted `npm run dev` failed because the port was in use.
