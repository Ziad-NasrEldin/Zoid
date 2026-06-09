# Critique Report: Hermes chat stage focus
## Verdict
APPROVED

## Summary
The scoped implementation satisfies the Page Feedback request. `AgentsHermesScreen` wires `.chat-stage` pointer-down events to an imperative `ChatComposer` ref, and `ChatComposer` exposes `focusMessageField()` that focuses the actual `<textarea>` through `messageInputRef.current?.focus({ preventScroll: true })`. The stage handler ignores common interactive descendants before focusing, so buttons, links, inputs, textareas, selects, role=button elements, and explicitly contenteditable descendants are not hijacked.

Reviewed only the handoff-listed scope:
- `src/agents/AgentsHermesScreen.tsx`
- `src/agents/ChatComposer.tsx`
- `src/scaffold.test.ts`
- `.hermes/reviews/hermes-chat-stage-focus/handoff.md`

## What was changed
- `src/agents/AgentsHermesScreen.tsx`: adds `composerRef`, `handleChatStagePointerDown`, attaches it to `.chat-stage`, and passes the ref to `<ChatComposer>`.
- `src/agents/ChatComposer.tsx`: converts the composer to `forwardRef`, defines `ChatComposerHandle`, stores the message textarea in `messageInputRef`, and focuses that textarea from `focusMessageField()`.
- `src/scaffold.test.ts`: adds a static scaffold assertion covering the stage-click-to-composer-focus wiring.

## Required fixes
| ID | Severity | Area | Issue | Evidence | Required fix |
| — | — | — | No required fixes. | Source review and focused checks passed. | — |

## Improvements
| ID | Priority | Area | Suggestion | Why it matters |
| I1 | Low | Test | Consider adding a real DOM/component test for pointer-down focus behavior when a DOM test runner is available. | The current scaffold assertion verifies source wiring but cannot execute focus behavior or descendant-guard behavior. |
| I2 | Low | Interactive guard | If future message content includes additional custom interactive roles such as `role="link"`, `summary`, or elements with positive `tabIndex`, consider broadening the guard. | The current guard covers the likely controls in this UI and the handoff requirements, but future custom interactive descendants may need explicit preservation. |

## Tests performed
- Inspected the scoped diff for `src/agents/AgentsHermesScreen.tsx`, `src/agents/ChatComposer.tsx`, and `src/scaffold.test.ts`.
- Source-verified the focus target is the real composer textarea: `messageInputRef` is attached directly to `<textarea>` and `focusMessageField()` calls `messageInputRef.current?.focus({ preventScroll: true })`.
- Source-verified `.chat-stage` has `onPointerDown={handleChatStagePointerDown}` and the handler calls `composerRef.current?.focusMessageField()` on `requestAnimationFrame`.
- Source-verified the handler returns early for `button`, `a`, `input`, `textarea`, `select`, `[role='button']`, and `[contenteditable='true']` descendants.
- `npm run test:frontend`: PASS.
- `npm run build`: PASS. Vite emitted only the chunk-size warning for the generated JS bundle.

## Dev-agent instructions
1. No changes are required for this scoped fix.
2. Keep any cleanup of unrelated dirty/untracked repository state out of this feature review unless separately requested.
