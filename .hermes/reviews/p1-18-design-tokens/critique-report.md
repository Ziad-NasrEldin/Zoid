# Critique Report: P1.18 Apple-style design tokens

## Verdict

APPROVED

## Summary

The P1.18 implementation satisfies the requested Apple-style design token scope. The change is CSS-only in `src/App.css`, introduces a comprehensive token layer for system/monospace fonts, spacing, radii, materials, borders, focus ring, shadows, light/dark palettes, and state styling, and applies those tokens across the existing P1.17 shell. I found no React behavior changes, copy changes, backend changes, database changes, dependency changes, or Tauri command changes in the current feature diff.

## Scope reviewed

Reviewed from `/Users/ziadnasreldin/Zoid`:

- `.hermes/reviews/p1-18-design-tokens/handoff.md`
- `git status --short`
- `git diff -- src/App.css .hermes/reviews/p1-18-design-tokens/handoff.md`
- `src/App.css`
- `src/App.tsx` for behavior/copy context only

Only `src/App.css` is modified in the application source. The review handoff directory is untracked, and this critique report is the only file I wrote.

## Findings

### Spec compliance

PASS.

The CSS defines and applies tokens for:

- Fonts: `--font-sans` applied at `:root`; `--font-mono` applied to status/detail values.
- Spacing: `--space-*` tokens are broadly applied to shell, sidebar, toolbar, panes, cards, lists, chips, badges, empty/loading copy, and responsive surfaces.
- Materials: page, shell, sidebar, toolbar, card, elevated, control, hover, subtle, glyph, and state materials are tokenized and applied.
- Shadows: control, sidebar, brand, selected, card, inset highlight, and window control shadows are tokenized and applied.
- Light/dark: `@media (prefers-color-scheme: dark)` overrides the text, accent, material, border, focus, shadow, and state tokens without requiring markup or React changes.
- Empty/error/loading states: empty state uses `--state-empty-*`; error copy uses `--state-error`; pending/loading badges and loading copy use `--state-loading` and `--state-loading-bg`.

### Meaningful token application

PASS.

The tokens are not merely defined. They replace the prior hard-coded styling across representative shell surfaces including the sidebar, toolbar, workspace items, status dots, badges, cards, registry chips, empty state, error copy, loading copy, integration/security lists, and inspector sections.

A token usage audit found 82 defined token names and 79 token names referenced through `var(...)`. The only unused token names found were `--material-empty`, `--state-error-bg`, and `--state-pending-bg`. This does not block P1.18 because the corresponding required state categories are still meaningfully represented and applied through `--state-empty-bg`, `--state-error`, `--state-loading`, `--state-loading-bg`, and pending strong/background/ring tokens. If desired, those three unused tokens can be removed or wired in during a future cleanup pass.

### Dark-mode accent contrast

PASS.

The dark accent fix is acceptable. Calculated contrast ratios for inverse text on accent surfaces are comfortably above the normal-text WCAG AA threshold:

- `#ffffff` on dark `--color-accent` `#454c5a`: 8.63:1
- `#ffffff` on dark `--color-accent-soft` `#5f6674`: 5.77:1
- `#ffffff` on light `--color-accent` `#424957`: 9.04:1
- `#ffffff` on light `--color-accent-soft` `#5d6470`: 5.96:1

### Loading token usage after fixes

PASS.

Loading/pending UI now uses the loading tokens directly:

- `.badge.pending` uses `--state-loading` and `--state-loading-bg`.
- `.large-card > .muted-copy`, which styles the foundation loading message, uses `--state-loading` and `--state-loading-bg`.

### Behavior/copy/backend/database scope

PASS.

The git diff shows no application source changes outside `src/App.css`. `src/App.tsx` remains unchanged in the working diff, so P1.17 React state, copy, fallback behavior, native invoke behavior, disabled controls, and truthful preview states are preserved. No backend, database, Tauri, package, or dependency files are modified.

## Required fixes

None.

## Non-blocking notes

- Three token names are currently unused: `--material-empty`, `--state-error-bg`, and `--state-pending-bg`. This is cleanup-only, not a P1.18 blocker.
- The brand mark gradient still includes a hard-coded lower stop `#242833`. Because the gradient is already token-driven through the upper accent stop and has strong contrast in both modes, this is acceptable for P1.18; it can be further tokenized in a future design-system cleanup if desired.

## Tests performed

Commands run from `/Users/ziadnasreldin/Zoid`:

1. `npm run build`
   - Result: PASS
   - TypeScript and Vite production build completed successfully.
   - Output included `dist/index.html`, `dist/assets/index-C6wymmIb.css`, and `dist/assets/index-Sf2NW0IA.js`.

2. `npm run verify:local`
   - Result: PASS
   - Rust tests: 75 passed, 0 failed.
   - Frontend build: PASS.
   - Local push verification: PASS with `--skip-package`.

3. Token usage audit via Python script
   - Result: PASS for required coverage.
   - 82 token names defined; 79 token names referenced through `var(...)`.
   - Required font/material/shadow/empty/error/loading/accent tokens are present and meaningfully applied.

4. Accent contrast calculation via Python script
   - Result: PASS.
   - Inverse text on accent surfaces ranged from 5.77:1 to 9.04:1.

## Tests still needed

None for this final critique gate.

## Dev-agent instructions

- No required fixes are needed for P1.18.
- Keep this task CSS-only through completion; do not add React behavior, copy changes, backend/database work, or reusable component abstractions under P1.18.
- If doing optional cleanup later, either remove unused tokens or wire them into appropriate state surfaces without changing app behavior.
