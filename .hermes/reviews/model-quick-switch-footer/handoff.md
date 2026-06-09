# Handoff: model quick switch footer

Scope: Zoid 25 page feedback for `/` on Hermes Agents.

Requested changes:
- Add a small button at the right side of the footer model stats segment so the user can quickly change model and model reasoning.
- The button must open the existing native model controls panel, not a separate oversized menu.
- Make the footer copy show both active model and active reasoning.
- Fix/invert the global sidebar rail menu state so expanded/default sidebar shows the X/close affordance and collapsed sidebar shows hamburger/open affordance.

Implementation notes:
- `src/agents/AgentsHermesScreen.tsx`
  - Footer model stats now include `Reasoning {activeReasoningLabel}`.
  - Added compact `Tune` button with `aria-label="Change model and reasoning"`, `aria-haspopup="dialog"`, and `onClick={() => setActiveCommandPanel("model")}`.
  - Existing model panel still provides provider/model/reasoning dropdowns and save behavior.
- `src/App.css`
  - `.chat-stats-model-section` uses a two-column grid: text + small button pinned right.
  - `.chat-stats-model-button` reduced to compact 26px-ish height.
  - Rail menu state selectors now use explicit `.rail-menu--close` and `.rail-menu--open` classes.
- `src/App.tsx`
  - Rail menu class now explicitly switches between `rail-menu--close` when expanded and `rail-menu--open` when collapsed.
- `src/scaffold.test.ts`
  - Added guards for compact model/reasoning quick switch and explicit rail menu inversion.

Verification already run:
- `npx tsx src/scaffold.test.ts` passed.
- `npm run build` passed.
- `npm run test:frontend` passed.
- Browser dev validation at `http://127.0.0.1:1420/` confirmed:
  - Expanded/default rail menu has class `rail-menu rail-menu--close`, label `Minimize sidebar`, and X transforms.
  - Collapsed rail menu has class `rail-menu rail-menu--open`, label `Maximize sidebar`, and hamburger transforms.
  - Footer model copy renders `Model gpt-5.5 · Reasoning medium · Codex 5h / 5h / week`.
  - Compact button text is `Tune`, height about 26px, right aligned in the model stats segment.
  - Clicking it opens `Model controls command panel` with provider/model/reasoning controls.

Review request:
- Check only this requested slice.
- Return verdict APPROVED or REQUIRED_FIXES.
- If required fixes exist, list only concrete blockers and exact files/areas.
