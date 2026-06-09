# Global Zoid Dropdown Critique

Verdict: APPROVED

## Scope reviewed
- `src/ui/GlobalDropdown.tsx`
- `src/ui/GlobalDropdown.behavior.test.tsx`
- `src/App.css`
- `src/code/CodeWorkspace.tsx`
- `src/agents/AgentsHermesScreen.tsx`
- `src/agents/ChatComposer.tsx`
- `src/App.tsx`
- `src/scaffold.test.ts`
- `CONTEXT.md`
- `.hermes/reviews/global-zoid-dropdown/handoff.md`
- `package.json`

## Re-review findings

### Initial REQUIRED_FIX 1: invalid/incomplete dropdown accessibility semantics
Resolved.

`src/ui/GlobalDropdown.tsx` no longer renders a `role="listbox"` containing interactive `button role="option"` children. The component now implements a button-triggered menu pattern:
- trigger has `aria-haspopup="menu"`, `aria-controls`, `aria-expanded`, and an accessible label;
- popup uses `role="menu"`;
- options use `role="menuitemradio"` with `aria-checked` for selected state;
- disabled options are represented with disabled button state and `aria-disabled`;
- keyboard handling covers trigger open behavior plus menu Escape, Enter/Space selection, ArrowUp/ArrowDown, Home, End, and Tab close;
- focus is moved into the menu on open and returned to the trigger on Escape/selection.

This satisfies the previously requested semantic fix and provides a concrete focus/keyboard model for the chosen ARIA pattern.

### Initial REQUIRED_FIX 2: missing behavior/accessibility tests
Resolved.

`src/ui/GlobalDropdown.behavior.test.tsx` was added and is wired into `npm run test:frontend` via `package.json`. The test renders the component with React + happy-dom and verifies:
- trigger accessible label, `aria-haspopup`, and `aria-expanded` state;
- click opens the menu;
- menu/menuitemradio structure is present;
- selecting an enabled option calls `onChange` and closes the menu;
- Enter opens the menu;
- Escape closes the menu;
- ArrowDown moves menu focus;
- disabled options do not call `onChange`;
- disabled dropdown trigger cannot open.

`src/scaffold.test.ts` also adds static guardrails for preserving global dropdown structure, keyboard/accessibility markers, behavior-test presence, reuse in app/composer surfaces, and absence of native `<select>` in reviewed surfaces.

## Page Feedback completion
Complete.

The Page Feedback request asked to replace the Code workspace default-branch native dropdown with the Zoid 25 design-system dropdown, add it as a global project rule, and wire a global dropdown menu. The implementation satisfies that request:
- `src/code/CodeWorkspace.tsx` uses `GlobalDropdown` for the default-branch selector.
- `src/ui/GlobalDropdown.tsx` provides the shared global dropdown component.
- `src/App.css` defines global `.zoid-dropdown*` design-system styling.
- Additional dropdown surfaces have been migrated to the shared component:
  - linked repository dropdown in `src/agents/AgentsHermesScreen.tsx`;
  - attachment action dropdown in `src/agents/ChatComposer.tsx`;
  - access/approval dropdowns in `src/App.tsx`.
- `CONTEXT.md` documents the global UI rule that Zoid 25 dropdowns must use `GlobalDropdown` and `.zoid-dropdown*` styles, not native `<select>` controls or one-off dropdown styling.

## Verification performed in this re-review
- Read and reviewed the relevant source, test, CSS, package, context, scaffold, and handoff files.
- Searched reviewed source for `<select`, `GlobalDropdown`, dropdown roles, and global dropdown styling.
- Ran frontend verification:
  - `npm run test:frontend` ✅ passed

## Notes / non-blocking observations
- The menu remains absolutely positioned inside its local layout container, so clipping could still occur in constrained/overflow-hidden ancestors. This was previously noted as a follow-up and is not a blocker for the requested accessibility/test fixes.
- The selected ARIA pattern is now a menu/menuitemradio pattern rather than a native-select-equivalent listbox. That is acceptable for the current project UI rule and is covered by behavior/static tests.

## Required fixes
None.

## Overall assessment
The original required fixes are resolved, the Page Feedback request is complete, and the implementation has both static guardrails and behavior coverage. Approved.
