Verdict: APPROVED

Scope reviewed:
- src/App.css compact Hermes session portrait rules only.
- src/scaffold.test.ts compact portrait regression guard only.
- Handoff at .hermes/reviews/session-avatar-fill-compact/handoff.md.

Findings:
- The compact portrait-specific CSS exists as `.sessions-rail--compact .session-tab-icon.session-tab-portrait` and sets `position: absolute`, `inset: 0`, `width: 100%`, `height: 100%`, `place-self: stretch`, `background: transparent`, `backdrop-filter: none`, and `box-shadow: none`.
- The rule appears after the generic compact `.session-tab-icon` rule, so it overrides the smaller 34px translucent inner-box treatment for portrait avatars.
- The New Session plus icon is not `.session-tab-portrait`, so it remains governed by the generic compact icon treatment and is not broadened to full-box.
- The added scaffold guard checks the key full-box compact portrait requirements and protects against the reported regression.

Verification performed:
- Read the handoff and inspected the scoped source regions in `src/App.css` and `src/scaffold.test.ts`.
- Ran `npm run test:frontend -- --runInBand`; it exited 0 in this workspace.

Required fixes: none.
