# Zoid 25 Animation System — Critique Handoff

## Scope

Repo: `/Users/ziadnasreldin/Zoid`

Feature: product-wide Zoid 25 motion/animation system for all user-visible app surfaces, using the accepted sumi-e / ink / white paper / red seal visual language.

Primary implementation files in this scope:

- `src/App.css`
- `src/App.tsx`
- `src/ui/GlobalDropdown.tsx`
- `.hermes/reviews/zoid-25-animation-system/animation-inventory.md`

Important dirty-tree note: this repo has substantial unrelated existing changes and untracked review folders/files. Review this feature by scoped files and rendered behavior, not by assuming the whole tree belongs to this feature.

## Motion direction

- Editorial Ink Mechanics: sumi-e paper/ink/red-seal movement, restrained and tactile.
- CSS-driven motion tokens and keyframes in `src/App.css`.
- Page/component motion is scoped to existing wrappers/classes to preserve behavior.
- Reduced-motion guards are required and were added for global, page, and workspace-specific motion.

## Implemented batches

1. Global foundation and shell
   - Motion tokens: durations, easing, press/lift offsets, paper shadows, seal wash.
   - Shared helpers/keyframes for paper panel entry, notice entry, seal stamp, ink scan, attention ping.
   - Shell/nav/sidebar/rail/status dots/dropdowns/loading notice motion.
   - `GlobalDropdown.tsx` now exposes open/disabled state classes/data hooks.
   - `App.tsx` loading notice has motion state classes.

2. Agents/Hermes workspace
   - Topbar ink rule, sessions rail/tabs, chat stage/composer, message bubbles/actions, agent monitor panels/buttons/feed, file-manager toggle/side panels.

3. Code workspace
   - Header/action panels, repository list/search/cards, repo status pills, feedback/default-branch editor, buttons/inputs.

4. Content/Social and Automations
   - Content tab actual routed surface is `SocialDashboard`.
   - Social hero/provider/rhythm/alert/status/toolbar/panels/post/media/gate/caption motion.
   - Automations header, clock mark, command/status/summary/toolbar/section/job card, badges, search, buttons, confirm panel motion.
   - Automations alert readability was fixed with grid layout/line-height/top padding.

5. Brain and Settings
   - Brain panels, bridge/error/empty/placeholder strips, source rows, note rows, session tabs, badges, clarifying textarea focus, panel chips, status seal mark, action press states, and staggered panel reveal.
   - Settings hero/profile cards, sticky heading, nav tabs, overview stat cards, sections, catalog/provider/archive cards, toggles, fields/dropdowns/options, save/security/error notes, confirm backdrop/panel/actions, press/focus states, and reduced-motion handling.

## Latest caveat-resolution batch

The follow-up work specifically targets the remaining caveats around hidden/conditional states that were not fully rendered in the earlier empty/blocked local app state:

- Settings archive live-data cards now animate `archived-session-card`, `archive-session-select`, archive checkboxes, seeded list rows, empty archive copy, and destructive confirmation rule/backdrop/panel states.
- Settings provider live-data states now animate provider meta rows, provider status badges, validated/invalid provider cards, and provider error notes.
- Settings catalog/toggle conditional controls now animate checkbox micro-states and live row reveal.
- Automations hidden/conditional error and empty states now include `automation-modal-error`, `automation-error-line`, and `.automation-sumi-e .repo-empty-state` in the motion/reduced-motion coverage.
- Social/Content empty state `.social-empty` now has hover/lift transition coverage and reduced-motion handling.
- Scaffold tests now assert the above caveat coverage so future cleanup cannot silently drop it.

## Validation performed

Automated:

- `npm run test:frontend` passed after the caveat-resolution batch.
- `npm run build` passed after the caveat-resolution batch.
- `npm run test:rust` passed after the caveat-resolution batch: 76 passed, 1 ignored macOS Notes mutating E2E.
- Earlier in this feature:
  - `npm run build` passed after latest Brain + Settings additions.
  - `npm run build` passed again after fixing the missing shared `motion-ink-reveal` keyframe.
  - `npx tsx src/brain/BrainWorkspace.behavior.test.tsx` passed.
  - `npx tsx src/code/CodeWorkspace.behavior.test.tsx && npx tsx src/code/repositoryOperations.test.ts` passed.
  - `npx tsx src/automations/AutomationsWorkspace.behavior.test.tsx` passed.

Browser probes:

- Code: selectors present and no horizontal overflow.
- Content/Social: selectors present and no horizontal overflow.
- Content/Social re-check after critique fix: browser CSSOM reports `motion-ink-reveal` exists; `.social-hero` computed animation name is `motion-ink-reveal`; `.social-dashboard`, `.social-hero`, `.social-rhythm-lane`, `.social-alert`, `.social-toolbar`, `.social-grid`, and three `.social-panel` nodes are present; no horizontal overflow.
- Automations: selectors present and no horizontal overflow.
- Brain: `.brain-sumi-e`, `.brain-hero`, `.brain-ink-mark`, `.brain-status-line` present and no horizontal overflow. Local blocked Apple Notes bridge state means some deeper link/panel selectors are not rendered in this state.
- Settings: `.settings-sumi-e`, `.settings-hero`, `.settings-ink-mark`, `.profile-hero-card`, `.profile-settings-heading--sticky`, `.profile-settings-workspace`, `.profile-section`, `.profile-nav-list` present and no horizontal overflow.
- Static CSS check: `motion-ink-reveal` is defined and Content/Social reduced-motion blocks mention the relevant animated selectors.

Visual browser checks:

- Code, Content/Social, Automations, Brain, and Settings were visually inspected after motion additions.
- Latest Brain and Settings screenshots looked structurally intact: no obvious clipping, overlap, unreadable text, or horizontal overflow.

Known existing blocker / caveat:

- Full scaffold/frontend checks have a pre-existing sessions-rail width mismatch noted earlier under Build Order 2. Do not classify that as newly caused by this feature without proof.

## Critique request

Review the scoped motion system for:

1. Visual quality: motion feels sumi-e/editorial/ink-based, not generic dashboard animation.
2. Coverage: obvious user-visible controls/cards/panels/status/empty/error/focus/active/disabled/dialog/dropdown surfaces have motion or deliberate stillness.
3. Safety: no layout shifts that damage readability, no clipping/overlap/horizontal overflow.
4. Accessibility: reduced-motion coverage is adequate and interactive focus remains clear.
5. Code quality: CSS is scoped, maintainable, not over-broad, and does not break existing behavior.

Return a verdict: APPROVED or CHANGES_REQUIRED. If CHANGES_REQUIRED, separate Required fixes from Nice-to-have polish.
