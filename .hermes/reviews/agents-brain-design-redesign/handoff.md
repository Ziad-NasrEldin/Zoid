# Agents Brain Design Redesign Handoff

## Scope
Apply the Brain page's sumi-e design system to the Agents/Hermes page while preserving current chat, sessions rail, repository linking, file manager, composer, slash/command surfaces, and stats functionality.

## Changed Files
- `src/agents/AgentsHermesScreen.tsx`
  - Added `agents-sumi-e` page class on the existing Hermes shell.
  - Updated the visible title from `Hermes Agent` to `Hermes Agents`.
  - Updated kana/reference copy for an Agents-specific command-room identity.
  - Kept existing controls, props, state, session, repository, file manager, composer, and stats structures intact.
- `src/App.css`
  - Added Agents-specific sumi-e styling that reuses the Brain design tokens/fallbacks: ink/paper/seal, serif typography, red brush/seal accents, monochrome command surfaces.
  - Styled topbar, status controls, repository dropdown, file button, sessions rail, chat stage, message bubbles, composer, file manager, popovers, command palette, stats strip, responsiveness, and reduced-motion states.
  - Added scroll/min-height handling so the redesigned page does not collapse the chat workspace on shorter desktop windows.
- `src/scaffold.test.ts`
  - Added regression checks requiring the Agents page to keep the new Brain-derived sumi-e design hooks/tokens.
  - Updated title assertion to `Hermes Agents`.

## Verification Performed
- `npm run test:frontend` passed.
- `npm run build` passed.
- Browser preview at `http://127.0.0.1:1420/` inspected on Agents page.
  - Initial visual pass found clipped top controls and collapsed workspace on the browser viewport.
  - Patched topbar layout, scroll behavior, workspace min-height, and stats min-height.
  - DOM geometry after patch showed non-overlapping topbar/workspace and visible sessions/stage/composer with page scroll for the stats strip on shorter viewport:
    - topbar height 230px
    - workspace height 320px
    - sessions rail/stage height 174px
    - composer height 124px
    - stats height 34px
    - shell scrollHeight 723px / clientHeight 577px

## Known Context / Caveats
- Hermes CLI is offline in this environment, so composer send is locked by existing behavior; this was not introduced by the redesign.
- Browser preview is not a native Tauri verification. Native `/Applications/Zoid 25.app` rebuild/relaunch remains pending after critique.
- Repository has substantial pre-existing dirty/untracked work outside this feature. Review should focus on the diff captured at `.hermes/reviews/agents-brain-design-redesign/diff.patch`.

## Requested Review
Please review the implementation for:
1. Any Required fixes where functionality was lost or controls became unusable.
2. Visual/design-system misses against the Brain sumi-e direction.
3. Layout regressions at normal desktop and shorter desktop heights.
4. Test coverage gaps for this redesign.

Verdict should be APPROVED only if no Required fixes remain.
