# Feature Handoff: Restore Hermes sessions list rail

## Original request

User said: "where did the sessions list rail go, i want oyu to get it back ... i am talking in zoid 25"

## Implementation summary

- Restored the Agents / Hermes left Sessions list rail that had been removed in the prior `remove-hermes-session-rail-list` change.
- Re-added expanded Sessions header with count, minimize/maximize button, New session row, active session switching, session list rows, archive-session actions, and right-edge resize handle.
- Reconnected `AgentsHermesScreen` props for `onActiveSessionIdChange` and `onArchiveSession` from `App.tsx`.
- Re-added archive-on-delete behavior so session rows still move to Settings archive instead of disappearing.
- Reintroduced rail width/compact state persistence under restored keys. The compact-state key was changed to `zoid25:hermes-sessions-rail-compact-restored` so old minimized state does not keep the restored list hidden.
- Restored CSS for the resizable rail, compact rail, New session row, session rows, archive buttons, and responsive behavior.
- Updated scaffold regression checks to require the sessions rail rather than forbid it.

## Changed files

- `src/agents/AgentsHermesScreen.tsx`
- `src/App.tsx`
- `src/App.css`
- `src/scaffold.test.ts`

## Verification run by dev

- `npm run test:frontend`: PASS
- `npm run build`: PASS
- `npm run test:rust`: PASS, 9 tests passed
- `npm run tauri:build`: PASS
- `git diff --check -- src/App.tsx src/agents/AgentsHermesScreen.tsx src/App.css src/scaffold.test.ts`: PASS
- Reinstalled `/Applications/Zoid 25.app` from the rebuilt bundle: PASS
- Relaunched installed native app: PASS, running process `/Applications/Zoid 25.app/Contents/MacOS/zoid`
- Native screenshot `/tmp/zoid25-sessions-list-rail-restored.png`: PASS, shows Agents/Hermes open with expanded Sessions rail, count, New session row, session list rows, archive buttons, and chat pane.

## Reviewer focus

- Confirm the user-requested Sessions list rail is visible in expanded form by default after restore.
- Confirm New session, session switching, archive session, compact/minimize, and resize wiring are present and type-safe.
- Confirm updated regression test matches restored behavior and does not preserve the removal requirement.
- Keep review scoped; repo has pre-existing unrelated dirty work.
