# Code page Brain design-system handoff

Scope: Apply the new Brain page sumi-e / monochrome ink design system to the Zoid 25 Code page only.

Changed files in this pass:
- src/code/CodeWorkspace.tsx
- src/App.css
- src/scaffold.test.ts

Important context:
- Repo already has a broad dirty tree and many untracked review folders. Review only the three scoped files above for this change.
- `git diff -- src/App.css` is noisy because App.css already contained many unrelated dirty changes before this task; focus on Code workspace selectors/classes.

Implementation summary:
- CodeWorkspace root now uses `code-workspace-shell code-sumi-e`.
- Header now mirrors Brain's design-system structure: `code-hero-copy`, big serif title, kana line, `code-reference-line`, and a decorative `code-ink-mark` with spans.
- Code CSS now defines page-local sumi-e tokens (`--code-ink-black`, `--code-seal`, serif font variables), fixed-shell page-owned scrolling, Brain-like scrollbar, ink background strokes, pale ruled panels, red seal accents, serif hierarchy, and restrained monochrome buttons/status/list cards.
- Repository list grid row was corrected to auto-height with a real `min-height: 280px`; it no longer collapses to a 2px panel after the oversized hero/actions.
- Scaffold guard now checks that Code uses the Brain-style sumi-e design details.

Commands already run:
- `npm run build` passed after the final CSS fix.
- `npm run test:frontend` failed before reaching this change's new guard because of a pre-existing unrelated scaffold assertion: `Hermes topbar must keep the visible title/header: <h2>Hermes Agent</h2>`.

Browser inspection:
- Opened `http://127.0.0.1:1420/` on Code workspace.
- Console had 0 messages / 0 JS errors.
- Geometry probe after fix: shell clientWidth 909, scrollWidth 909, bodyOverflow 0; repository list panel height 280.
- Visual inspection: Code page shows the sumi-e/ink hero, red seal accent, pale panels, and full repository list panel; no obvious horizontal overflow.

Review request:
- Check whether the Code page really internalizes the Brain design system rather than just copying decoration.
- Check for any CSS regressions from global/unscoped selectors in the Code section.
- Check responsive/scroll ownership: fixed shell should scroll vertically; no horizontal overflow; repository list should remain visible as a real panel.
- Return verdict APPROVED or CHANGES_REQUESTED with Required fixes only for this scoped Code page design-system pass.

Review iteration 2 update:
- Fixed reviewer-required narrow-width overflow risk in repository list heading/search row.
- Added `@media (max-width: 560px)` rules so the heading becomes a two-column grid, the open search morph spans the full row at width 100%, and the count pill stays shrink-safe.
- Re-ran `npm run test:frontend && npm run build`; both passed after the fix.
