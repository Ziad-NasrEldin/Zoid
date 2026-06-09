# Content page sumi-e/scroll feedback handoff

## Request
Page Feedback for `/` Content/MaVoid Buffer page:
1. The eyebrow `MaVoid · Buffer social automation` should be Japanese.
2. The content page could not scroll at 1758×982 inside `tauri://localhost`.
3. The dashboard toolbar buttons did not match the Zoid design system.
4. The whole content page felt different from other pages.

## Changed files
- `src/social/SocialDashboard.tsx`
- `src/social/SocialDashboard.behavior.test.tsx`
- `src/App.css`

## Implementation summary
- Converted Content dashboard shell to `social-sumi-e`, matching the accepted Zoid sumi-e page system vocabulary.
- Replaced English eyebrow with Japanese: `マヴォイド・バッファ自動投稿` and kept it as `.kana-line`.
- Added `social-reference-line` and `social-ink-mark` so the hero follows the same structure as Automations/Brain-style pages.
- Changed the dashboard scroll surface from `height: 100%` / generic `overflow: auto` to `height: 100vh`, `overflow-y: auto`, `overflow-x: hidden`, `overscroll-behavior: contain`, and `scrollbar-gutter: stable` for the fixed Zoid shell.
- Removed internal `.social-panel` scroll ownership (`overflow: visible`) so the page owns vertical scroll.
- Restyled `.social-toolbar button` from heavy generic pill-ish buttons to the shared ink-rule square/button register: serif Latin font, 42px height, transparent background, bottom ink-rule pseudo-element, no heavy box shadow.
- Updated behavior tests/source guards for Japanese eyebrow, `social-sumi-e`, `social-ink-mark`, page scroll ownership, and toolbar ink-rule button affordance.

## Verification already run
- `npx tsx src/social/SocialDashboard.behavior.test.tsx` -> passed.
- `npm run build` -> passed (`tsc && vite build`).

## Review focus
Please critique for:
- Whether the visual direction now coheres with the accepted Zoid/Automations sumi-e design system.
- Whether the scroll fix is robust for the reported 1758×982 Tauri viewport.
- Whether the toolbar buttons now match the design system rather than generic dashboard buttons.
- Whether any source guards are too brittle or insufficient.

Return verdict APPROVED or CHANGES_REQUESTED with required fixes only if blocking.