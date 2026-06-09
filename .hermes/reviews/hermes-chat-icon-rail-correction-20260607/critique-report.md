# Critique Report: Hermes chat icon and rail correction 2026-06-07

## Verdict

APPROVED

## Scope reviewed

Reviewed the focused source and diff for:

- `src/agents/Avatar.tsx`
- `src/App.css`
- `src/scaffold.test.ts`

Cross-checked against the handoff requirements for removing the decorative sessions rail spine, removing custom Hermes sigil/profile-picture markup, enhancing the existing HA avatar through CSS only, and retaining prior chat fixes.

## Findings

### Compliance with requested correction

- The custom Hermes sigil/profile-picture markup has been removed from `Avatar.tsx`. The component now consistently renders either an image URL or the participant initials span; Hermes resolves to the existing `HA` initials from `participants.ts`.
- The Hermes-specific icon treatment is CSS-only through `.chat-avatar--hermes`, `.chat-avatar--hermes::before`, `.chat-avatar--hermes::after`, and `.chat-avatar--hermes > span:not(.avatar-presence)`. This satisfies the request to enhance the existing avatar surface rather than replacing it with bespoke markup.
- The rejected decorative sessions rail spine selectors `.sessions-rail::before` and `.sessions-rail::after` are absent from active CSS/source. The rail now uses a simple right border and soft shadow. The remaining `.sessions-rail-resize-handle::after` is a functional resize affordance rather than the removed decorative rail spine.
- Prior chat fixes appear retained: terminal command plumbing is stripped from message content, empty streaming bubbles are suppressed, and the scaffold checks continue to guard those paths.

### UX and accessibility

- The avatar retains an accessible outer `aria-label` and hides decorative/initials text from assistive tech, which is appropriate because the label already identifies the participant avatar.
- The CSS-only Hermes avatar enhancement is restrained enough for dense chat UI: it preserves the small circular avatar footprint, keeps the `HA` monogram visible, and avoids adding extra DOM or nonsemantic image-like chrome.
- The sessions rail correction improves visual clarity by removing the spine and relying on standard separation (`border-right`) plus subtle depth. No accessibility regression was found in the focused changes.

### Test adequacy

- `src/scaffold.test.ts` includes focused guards requiring the simple rail border and enhanced initials styling, while rejecting `.sessions-rail::before`, `.sessions-rail::after`, `hermes-sigil`, and `hermes-sigil__core`.
- The tests also continue checking the prior terminal-plumbing hiding and empty streaming bubble behavior.
- Limitation: the added coverage is static/string-based rather than rendered visual or DOM behavior coverage, so it will catch selector/markup regressions but not subtle visual balance issues. This is acceptable for this focused correction given the existing project test style.

## Verification performed

- `npm run test:frontend` passed.
- `npm run build` passed. Vite emitted only the existing chunk-size warning.
- Search confirmed rejected active selectors/markup are absent outside the scaffold test's forbidden-token list.

## Issues

No blocking issues found.
