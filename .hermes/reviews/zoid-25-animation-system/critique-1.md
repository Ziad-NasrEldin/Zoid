# Critique 1 — Zoid 25 Animation System

Verdict: CHANGES_REQUIRED

## Required fixes

1. Define or replace missing `motion-ink-reveal` keyframes.
   - `src/App.css` referenced `animation: motion-ink-reveal ...` for Content/Social surfaces.
   - Browser/CSSOM check confirmed no `@keyframes motion-ink-reveal` existed.
   - Result: Content/Social reveal animations were silently non-functional.

2. Re-verify Content/Social after fixing the keyframe.
   - Confirm Content/Social surfaces animate on entry.
   - Confirm no horizontal overflow/clipping after reveal animation is active.
   - Confirm reduced-motion disables the corrected animation.

## Nice-to-have

- Consolidate repeated page reveal keyframes in the future.
- Add a small automated CSS sanity check for referenced vs defined keyframes.
- Consider splitting large motion CSS into clearer sections later.

## Fix status

- Added shared `@keyframes motion-ink-reveal` near the global motion primitives in `src/App.css`.
- Rebuilt successfully with `npm run build`.
- Browser CSSOM now reports `motion-ink-reveal` exists.
- Content/Social browser probe now reports `.social-hero` animation name as `motion-ink-reveal`, expected Social selectors present, and no horizontal overflow.
- Static CSS check confirms Content/Social reduced-motion blocks mention the relevant animated selectors.
