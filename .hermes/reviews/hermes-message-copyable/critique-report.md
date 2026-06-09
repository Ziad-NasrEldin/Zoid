Verdict: APPROVED

Scope reviewed:
- src/agents/MessageBubble.tsx
- src/agents/AgentsHermesScreen.tsx
- src/App.css
- src/scaffold.test.ts

Findings:
- Copy affordance: APPROVED. MessageBubble renders a small icon button beside every visible message bubble, with an accessible aria-label/title and copied/failed visual states.
- Clipboard behavior: APPROVED. The copy path uses the rendered visibleContent plus visible error text, and visibleContent is derived through stripTerminalCommandPlumbing rather than backend/terminal plumbing. It uses navigator.clipboard with a textarea fallback.
- Text selection: APPROVED. Message content and bubbles explicitly opt into user-select: text, and the chat-stage pointer-down focus behavior now ignores pointer starts inside .message-bubble and .message-copy-button, so drag selection should not be stolen by composer focus.
- Visibility/layout: APPROVED. The copy control is in a message-bubble-frame beside the bubble, with user/assistant ordering handled in CSS.
- Test/build evidence: APPROVED. npm run test:frontend and npm run build both passed locally. Build emitted only the existing Vite chunk-size warning.

Required fixes: None.

Notes:
- The added scaffold coverage is string-based rather than behavioral DOM/clipboard testing, but it covers the key regression hooks and is reasonable for the existing scaffold-test style.
- The scoped implementation also exposes copy buttons for user bubbles, not only Hermes assistant responses. This is acceptable for the handoff wording of every visible Hermes chat message bubble and does not undermine the requested response-copy feature.
