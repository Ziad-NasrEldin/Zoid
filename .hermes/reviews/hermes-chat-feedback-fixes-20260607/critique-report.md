# Critique Report: Hermes chat feedback fixes 2026-06-07

## Verdict
APPROVED

## Scope Reviewed

Reviewed the handoff and current focused source changes for:

- `src/agents/MessageBubble.tsx`
- `src/agents/Avatar.tsx`
- `src/App.css`
- `src/scaffold.test.ts`

Validation run:

- `npm run test:frontend` — PASS

## Findings

### Page Feedback compliance

1. **Empty Hermes writing bubble** — Satisfied. `MessageBubble` derives sanitized `visibleContent`, checks `hasVisibleContent`, and only renders `.message-bubble` when there is visible content or an error. A streaming assistant message with no visible content now leaves only the `HERMES WRITING` status/glyph visible.

2. **Hermes avatar uniqueness** — Satisfied. The old `HA` fallback is replaced with a decorative Hermes sigil made from a core and orbit elements, with supporting CSS pseudo-elements and gradients. The markup is hidden from assistive tech while the avatar wrapper retains a participant avatar label.

3. **Sessions rail separation** — Satisfied. The sessions rail now has a clear branded right-edge spine using decorative `::before`/`::after` pseudo-elements, `content: ""`, and `pointer-events: none`, so it improves visual separation without introducing duplicate readable labels or interaction traps.

4. **Terminal command plumbing hiding** — Satisfied. `stripTerminalCommandPlumbing` removes `Terminal command used:` headers and `$ hermes...` command lines from the rendered bubble content. The implementation keeps the source message unchanged and only sanitizes the visible content, which is the right containment for a display-only polish fix.

## Code Quality / UX / Accessibility Notes

- The implementation is appropriately localized to render-layer components/CSS and does not alter Hermes CLI/backend behavior.
- The terminal-plumbing sanitizer is conservative enough for the reported output shape and avoids broad removal of normal prose. It only strips the explicit header, `$ hermes...` shell lines, and immediately related shell prompt lines while in the command block.
- The empty-bubble gate is simple and handles error messages correctly: errors still get a visible bubble even if sanitized content is empty.
- The Hermes sigil is decorative and `aria-hidden`, while the avatar wrapper has a useful label. If future accessibility hardening is desired, consider adding an explicit `role="img"` to the avatar wrapper, but this is not blocking for this focused fix.
- The sessions separator uses decorative pseudo-elements with empty content and no pointer events, which avoids the prior duplicate-label/accessibility concern.

## Test Adequacy

- Scaffold checks cover the expected source markers for sanitizer, visible-content gate, sigil CSS/markup, and sessions rail separator pseudo-elements.
- `npm run test:frontend` passes.
- The tests are string-based scaffold checks rather than behavioral component tests. That is acceptable for this project’s current regression style, but a future React rendering test for “streaming empty assistant renders status but no bubble” and “terminal plumbing is absent from visible output” would provide stronger coverage.

## Required Fixes

None.
