# Composer expansion-only motion handoff

## Request
Fix the Zoid Hermes composer shaking on every typed character. The user only wanted motion when the textarea expands to fit new lines, not a typing/glow animation on every keystroke.

## Scoped changes
- `src/agents/ChatComposer.tsx`
  - Removed per-keystroke typing/glow state behavior.
  - Auto-height still measures `scrollHeight` and clamps between `COMPOSER_MIN_HEIGHT` and `COMPOSER_MAX_HEIGHT`.
  - Height transition only runs when `nextHeight > previousHeight`; normal same-line typing keeps the same height with no new class/animation.
  - Shrinking clears the expansion data flag and sets the new height directly.
- `src/App.css`
  - Composer textarea has no default transition; `height 220ms cubic-bezier(0.22, 1, 0.36, 1)` only applies while `data-expanding="true"` is present.
  - Removed typing ring / per-keystroke classes and keyframes.
- `src/scaffold.test.ts`
  - Existing guard now rejects `typingEffectTimerRef`, `composer-input-wrap--typing`, `@keyframes composerTypingRing`, `animation: composerTypingRing`, and non-height transition tokens.

## Verification already run
- `npm run test:frontend` passed.
- `npm run build` passed.
- `npm run test:rust` passed.
- `npm run tauri:build` passed.
- Reinstalled and relaunched `/Applications/Zoid 25.app`.
- Browser smoke on `http://127.0.0.1:1420`:
  - same-line typing kept height at 44px and no typing/expanding class appeared.
  - adding a newline animated height from 44px to 57px.
  - browser console had no messages/errors.

## Review focus
Confirm there is no remaining per-keystroke animation or glow behavior, and that textarea expansion still animates only when height increases.

## Notes
Repo contains unrelated dirty work from prior tasks. Review only the scoped composer motion change above.