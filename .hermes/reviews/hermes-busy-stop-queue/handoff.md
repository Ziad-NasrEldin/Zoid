# Hermes busy stop and queue handoff

## Scope
User reported the Hermes chat composer locked them out while the agent was running and asked for Hermes-like behavior: keep editing possible, allow Ctrl/Cmd+C to stop, and allow messages during a running turn to queue/steer instead of being blocked.

## Changes
- `src/agents/ChatComposer.tsx`
  - Busy composer no longer treats `isSending` as a send blocker when there is a draft/attachment.
  - Busy button shows `QUEUE` when a draft exists, `STOP` when empty.
  - Empty busy submit explains queue/stop behavior.
  - `Ctrl/Cmd+C` with no active text selection calls `onStop`; selected text preserves normal copy semantics.
  - Exported `shouldStopHermesFromCopyShortcut` for focused behavior coverage.
- `src/agents/AgentsHermesScreen.tsx`
  - Added per-session queued prompt ref.
  - If user sends while `isSending`, Zoid queues the prompt instead of invoking another Hermes process.
  - After the current run finishes/stops/errors, the next queued prompt for that session is automatically submitted using the latest session snapshot.
- `src/agents/ChatComposer.behavior.test.tsx`
  - Covers slash completion close behavior, Ctrl/Cmd+C stop predicate, editable busy composer, and busy submit queue behavior.

## Verification already run
- `npm run test:frontend` — passed
- `npm run build` — passed
- `npm run test:rust` — passed (64 tests, warnings only for pre-existing unused functions)

## Review focus
Check for Required fixes around:
- queued prompt ordering after the original assistant message resolves
- stale session / stale repository/session id bugs
- Ctrl/Cmd+C overriding normal copy behavior
- button semantics: STOP vs QUEUE when busy
- regressions to normal send/slash command behavior
