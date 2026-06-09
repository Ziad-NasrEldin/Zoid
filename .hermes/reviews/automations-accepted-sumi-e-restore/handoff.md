# Automations accepted sumi-e restore handoff

Scope: restore the previously accepted Automations sumi-e design from session history, not a new redesign.

User correction: the prior "revert" was not the old accepted sumi-e design. We recovered the original accepted design markers from the prior session (`automation-sumi-e`, large open hero, ink clock/ritual mark, paper/ink/red-seal palette, serif title, brush wash/pseudo-elements) and reapplied them.

Files in scope:
- `src/App.css`: replaced the Automations `.automation-sumi-e` CSS block with the recovered accepted sumi-e treatment. Adapted only for current `.automation-section-labels` markup and kept scroll-safe modal error styles.
- `src/automations/AutomationsWorkspace.tsx`: no broad rewrite in this pass; verified existing root/motif/copy are present.

Design invariants to verify:
- Root class is `automations-workspace-shell automation-sumi-e`.
- Hero has tall open layout: header min-height around `clamp(460px, 38vw, 580px)`.
- Title is large serif: `clamp(52px, 6.8vw, 96px)`.
- Background includes radial ink washes and `repeating-linear-gradient` paper rule.
- Shell has `::before` and `::after` brush/wash pseudo-elements.
- `.automation-ink-clock` exists and is large (`clamp(118px, 16vw, 218px)`) with ink/red-seal details.
- Constraint line remains: `Provider-owned schedules · protected system jobs · watcher state is read-only`.
- No `automation-kujoyama` class or blue boxed Kujoyama treatment should be active in Automations.

Verification already run:
- `npx tsx src/automations/AutomationsWorkspace.behavior.test.tsx && npm run build` passed.
- Browser computed styles verified: shell class, repeating-gradient background, h2 ~87px, header min-height ~486px, clock width ~205px, no horizontal overflow.
- Browser visual inspection showed the large old sumi-e look: tall open hero, ink/paper/red-seal palette, large serif title, large ink clock, no boxed blue treatment.
- Browser console was empty after inspection.

Review request: confirm this is a restoration of the accepted old sumi-e Automations design rather than a new redesign, and that behavior fixes remain intact in scope. Return APPROVED or REQUIRED_FIXES with exact evidence.