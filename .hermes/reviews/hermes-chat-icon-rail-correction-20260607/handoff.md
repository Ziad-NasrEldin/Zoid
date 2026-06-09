# Feature Handoff: Hermes chat icon and rail correction 2026-06-07

## Original correction request

User asked to:
- Remove the decorative sessions rail spine because it looked horrible.
- Remove the custom Hermes profile-picture/sigil markup that was added.
- Enhance the profile icon itself instead, using the appropriate product UI craft/Impeccable workflow.

## Skill/process used

Loaded `product-ui-craft-workflow` and its `icon-button-surface-simplification` reference. Applied the pattern by keeping the existing avatar/initials surface as the owned UI element, removing the decorative replacement image/markup, and enhancing the icon through restrained CSS only.

## Implementation summary

- Restored `Avatar.tsx` to render normal participant initials (`HA`) instead of the custom `hermes-sigil` nested markup.
- Removed the decorative sessions rail spine pseudo-elements (`.sessions-rail::before` / `.sessions-rail::after`) and replaced that treatment with a plain `border-right` plus a very soft shadow.
- Enhanced the Hermes avatar/icon itself via `.chat-avatar--hermes` CSS: tighter monogram chip, calmer orbital pseudo-lines, darker branded gradient, no replacement image/illustration markup.
- Updated scaffold checks to require the simple rail border and enhanced initials styling, and to reject the removed decorative spine/sigil selectors.

## Changed files

- `src/agents/Avatar.tsx`
- `src/App.css`
- `src/scaffold.test.ts`

## Verification performed

- `npm run test:frontend && npm run build`: PASS. Vite emitted only the existing chunk-size warning.
- Browser preview at `http://127.0.0.1:1420`, Agents screen DOM probe:
  - `.sessions-rail::before` content: `none`
  - `.sessions-rail::after` content: `none`
  - rail border: `1px solid rgb(0, 0, 0)`
  - Hermes avatar text: `HA`
  - `.hermes-sigil` present: `false`
  - enhanced initials inner border/size present
- Browser console after inspection: no messages/errors.

## Reviewer focus areas

- Confirm the rejected rail spine is fully removed, not just hidden behind another decorative element.
- Confirm Hermes avatar uses existing initials markup and not replacement sigil/image markup.
- Confirm the initials icon treatment is an enhancement to the existing avatar surface and remains restrained in dense chat UI.
- Confirm no regressions to previous terminal-plumbing hiding / empty streaming bubble fix.

## Verdict needed

Please write critique report to `/Users/ziadnasreldin/Zoid/.hermes/reviews/hermes-chat-icon-rail-correction-20260607/critique-report.md` with verdict APPROVED or REQUEST_CHANGES.
