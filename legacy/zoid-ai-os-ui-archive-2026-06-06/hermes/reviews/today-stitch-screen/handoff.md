# Today Stitch Screen implementation handoff

Feature slug: today-stitch-screen

## User request
Implement the first Stitch-created screen, the Today screen, inside Zoid using the updated `DESIGN.md` Apple-design-analysis design system.

## Source design
Stitch screen: `Today Dashboard` from project `projects/2534809720873389640`, screen `projects/2534809720873389640/screens/030f07d6785e4854b96c3f67886eb490`.

Visible Stitch structure implemented:
- Left Zoid workspace sidebar remains from native app shell.
- Today top bar with date, Dashboard/Analytics/Review tabs, Search/Command/Run/New actions.
- Priority row: AI Daily Brief, Needs Attention, Active Work.
- Operations grid: Tasks, Calendar, Content Queue, Agents Status.
- Secondary grid: Dirty Repos, Automations, Recent Activity, Inbox Brief.
- Native/local truth panel is preserved as a collapsible details section to avoid fake data while keeping existing Zoid foundation state accessible.

## DESIGN.md application
Uses Apple-design-analysis tokens in `src/App.css`:
- `#0066cc` Action Blue for all primary/action UI.
- `#ffffff` canvas and `#f5f5f7` parchment surfaces.
- SF Pro Display/Text stack.
- 17px body rhythm and negative letter spacing.
- 18px utility-card radius and full-pill CTA/tag radius.
- No UI card shadows; only `.today-screen-media.product-photo` uses the reserved product-photo shadow.

## Files changed by this feature
- `src/todayDashboard.ts` — typed view-model/data for Stitch Today dashboard content and Apple design-system metadata.
- `src/todayDashboard.test.ts` — focused RED/GREEN test for Stitch content/order and design-system token requirements.
- `src/App.tsx` — Today workspace now renders the Stitch Today screen via `buildTodayDashboardView`; retained truth/foundation section in details.
- `src/App.css` — added Apple-design tokens and Today screen styles.
- `package.json` — added `src/todayDashboard.test.ts` to `test:frontend`.

## Required-fix response
After first critique, fixes were applied:
- Added an always-visible sample banner: Stitch operational cards are explicitly labeled as design-copy only, with native Zoid truth state called out.
- Removed red/green Today dashboard tones from the view model and CSS; Today screen uses Action Blue plus neutral/muted treatment.
- Removed Today-screen decorative gradients and brief pseudo-atmosphere; the media tile is flat dark surface, with reserved product-photo shadow scoped to `.today-screen-media.product-photo div` only.
- Changed inactive Dashboard tabs from inert anchors to disabled buttons.
- App render now passes a runtime formatted date label instead of relying on the view-model default.
- Added test coverage for visible sample notice and no red/green tones.

## Verification performed
- `npx tsx src/todayDashboard.test.ts` — passed.
- `npm run build` — passed.
- `npm run test:frontend` — passed.
- `npm run verify:local` — passed: Rust tests, frontend tests, production build.
- Browser preview at `http://127.0.0.1:1420` — rendered Today Dashboard with visible sample banner; no console messages or JS errors.

## Known repo state / do not conflate
There are pre-existing unrelated modified/untracked files visible in `git status`:
- `src/taskBridgeIntegration.ts`
- `src/taskBridgeIntegration.test.ts`
- `.hermes/reviews/tasks-page-audit/`
- `.hermes/screenshots/`
- `Backups/`
- `DESIGN.md`
- `Docs/zoid-layout-only-desktop-app-sitemap-page-structure.md`

The feature work above intentionally did not edit the task bridge files.

## Critique focus
Please review for:
1. Whether the Today UI closely implements the Stitch first screen without breaking the native Zoid truth/fail-closed rules.
2. Whether the Apple-design-analysis rules are respected: Action Blue single accent, SF Pro typography, no UI shadows, product-photo-only shadow, pill CTAs, light/parchment surfaces.
3. Whether the implementation should replace static Stitch sample data with native-backed data now, or if the truthful collapsible native state is acceptable for first visual implementation.
4. Any Required fixes before this can be considered approved.
