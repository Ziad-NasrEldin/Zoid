# Feature Handoff: P1.18 Apple-style design tokens

## Original request

Continue Zoid development using the Zoid-wide subagent workflow.

Tracker task:

- P1.18 Frontend: Apple-style design tokens for system font, spacing, materials, shadows, light/dark, empty/error/loading states.

## Implementation summary

- Added a CSS design-token layer in `src/App.css` for the P1.17 shell.
- Tokens cover:
  - system and monospace font stacks;
  - spacing scale;
  - radius scale;
  - text/accent/window-control colors;
  - page/shell/sidebar/toolbar/card/control materials;
  - borders, focus ring, and shadows;
  - ready, pending, blocked, error, empty, and loading state colors/backgrounds/rings.
- Added `@media (prefers-color-scheme: dark)` token overrides.
- Refactored representative shell CSS to consume tokens across sidebar, toolbar, split panes, cards, badges, status dots, empty/error/loading copy, lists, registry chips, and inspector sections.
- Preserved all P1.17 behavior/copy and made no backend, database, Tauri, dependency, or React logic changes.
- Fixed review issues before this handoff:
  - dark-mode accent contrast for inverse text;
  - loading token usage;
  - broader spacing-token usage.

## Changed files

- `src/App.css`
  - Added and applied Apple-style design tokens with light/dark variants.
  - Refactored colors/materials/shadows/radii/spacing/states to CSS variables.
  - Kept existing layout structure stable.

## How to test

Commands:

- `npm run build`
- `npm run verify:local`
- Browser preview: `http://127.0.0.1:1420/`
  - Verify the shell still renders and has no horizontal overflow.
  - Verify CSS variables are present, e.g. `--color-accent`, `--state-loading`.
  - Verify visible P1.17 shell behavior remains unchanged/truthful.
  - Optional OS/browser check: switch `prefers-color-scheme` to dark and confirm token-driven dark palette remains readable.

## Tests run

- `npm run build`: PASS after implementation.
- Independent spec review: REQUESTED CHANGES for unused loading tokens and weak spacing-token usage.
- Independent quality review: REQUESTED CHANGES for low dark-mode inverse-text contrast on accent tokens.
- Fix subagent updated CSS only.
- Independent spec re-review: PASS.
- Independent quality re-review: APPROVED.
- Parent browser preview check at `http://127.0.0.1:1420/`: PASS for project identity, no horizontal overflow, shell content present, and token variables available.

Pending before final completion:

- Final critique gate.
- `npm run verify:local` and `npm run verify:release` after final critique/fixes.
- Tracker update and commit.

## Git info

- Branch: `main`
- Current committed base before P1.18: `febdbd9 Implement P1.17 macOS app shell`
- P1.18 is currently uncommitted at handoff creation.

## Frontend/backend/database notes

- Frontend:
  - CSS-only token implementation in `src/App.css`.
- Backend endpoints/services:
  - No backend changes.
- Database tables/migrations:
  - No database changes.

## Reviewer focus areas

- Confirm this remains P1.18 only and does not implement P1.19 reusable components or alter P1.17 React behavior.
- Confirm tokens exist and are meaningfully applied for fonts, spacing, materials, shadows, light/dark, empty/error/loading states.
- Confirm dark-mode contrast on inverse text/accent surfaces is acceptable.
- Confirm no fake states, fake integrations, or copy changes were introduced.
- Confirm build remains green.

## Fix cycle notes

- Initial spec review found loading tokens unused and spacing tokens too weakly applied.
- Initial quality review found dark accent contrast below normal text expectations for inverse text.
- Fix changed dark accent tokens to darker values, made loading token usage explicit in pending badge/loading copy, and broadened spacing-token usage across representative shell surfaces.
- Re-review results: spec PASS; quality APPROVED.
