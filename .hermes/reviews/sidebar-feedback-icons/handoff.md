# Sidebar feedback icons handoff

Scope: Page Feedback `/` small UI slice.

Requested changes:
- Expanded sidebar should be default-open and the rail menu should show an X/close affordance; collapsed sidebar should show hamburger/maximize affordance.
- Remove the black/brush divider line under the ZOID25 brand block for this instance.
- Replace generic lucide primary navigation icons with something that fits the sumi-e/ink/seal design system.

Changed source:
- `src/App.tsx`
  - Replaced generic lucide nav icon imports with a local `InkSigil` SVG component and per-section sigil variants.
  - Stopped passing the global Code linked repository id into Agents to preserve unlinked-by-default sessions while keeping optional prop compatibility in the Agents component.
- `src/App.css`
  - Inverted `.rail-menu` transforms so default expanded state shows X; `.sidebar-collapsed` resets spans into hamburger.
  - Removed `.brand-block::after` divider.
  - Added `.nav-sigil` / `.nav-sigil-seal` styling.
- `src/scaffold.test.ts`
  - Updated sidebar/source guards for removed brand divider, X default affordance, and custom sigil icon set.
- `src/agents/AgentsHermesScreen.tsx` / `src/agents/AgentsHermesScreen.file-manager.test.tsx`
  - Narrow build blockers found during verification: optional legacy prop type restored without runtime fallback; `.at(-1)` replaced for current TS lib target.

Evidence already run:
- `npx tsx src/scaffold.test.ts` passed.
- `npm run test:frontend` passed.
- `npm run build` passed.
- Browser dev verification at `http://127.0.0.1:1420/`:
  - Expanded shell class `zoid25-shell`; rail menu aria-label `Minimize sidebar`; span transforms show X; `.brand-block::after` content is `none`; all primary nav SVGs have class `nav-sigil`; seal fill is `rgb(194, 58, 46)`.
  - After clicking rail menu: shell class `zoid25-shell sidebar-collapsed`; aria-label `Maximize sidebar`; span transforms reset to hamburger.

Review focus:
- Confirm the menu affordance is no longer inverted.
- Confirm removing `.brand-block::after` does not break other sumi-e sidebar requirements.
- Confirm custom sigils are scoped, accessible, and do not reintroduce generic/loud icon styling.
- Watch for accidental behavior changes from the Agents linked-repository prop cleanup; current intent is session-scoped/unlinked by default.
