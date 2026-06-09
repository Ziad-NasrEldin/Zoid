# Critique Report: Agent session avatar images

Verdict: APPROVED

## Scope reviewed

- Handoff: `.hermes/reviews/agent-session-avatar-images/handoff.md`
- Source: `src/agents/AgentsHermesScreen.tsx`, `src/App.tsx`, `src/scaffold.test.ts`
- Prior required fixes:
  - R1: stale `sessions` prop used when creating new sessions.
  - R2: hydration preserved duplicate-but-valid avatar ids instead of repairing them.

## Re-review findings

### R1: stale sessions prop in new-session creation

Resolved.

`AgentsHermesScreen` now centralizes new-session insertion in `prependNewSession()`:

- `onSessionsChange((current) => { ... })` is used.
- `createSession("New session", current)` assigns the avatar against the latest session list supplied by React's functional state updater.
- `pendingNewSessionActivationRef.current = nextSession.id` records the created session id inside the updater.
- A `useEffect` activates that pending id after the parent sessions state update lands.
- `handleNewSession()` calls `prependNewSession()`.
- The slash-command `result.kind === "new-session"` path also calls `prependNewSession()` and no longer creates with `createSession("New session", sessions)`.

This addresses the same-render/rapid-addition collision risk from the prior critique: avatar selection is now based on the actual current state at insertion time, not a stale render prop.

### R2: hydration duplicate-but-valid avatar ids not repaired

Resolved.

`getInitialHermesSessions()` in `App.tsx` now reduces through persisted sessions while tracking already-resolved portraits:

- `usedPortraitIds = resolvedSessions.map((item) => item.portraitId)`.
- `hasValidUnusedPortrait = getSessionAgentAvatarById(session.portraitId) && !usedPortraitIds.includes(session.portraitId)`.
- Only valid and not-yet-used portrait ids are preserved.
- Missing, invalid, or duplicate-but-valid ids are reassigned with `chooseUniqueSessionAgentAvatarId(usedPortraitIds, session.id)`.

This preserves the first valid occurrence and repairs duplicate legacy/conflicting active sessions before pool exhaustion.

## Test/build evidence

Commands run locally from `/Users/ziadnasreldin/Zoid` during final re-review:

- `npx tsx src/scaffold.test.ts && npm run build && npm run test:frontend`: PASS.
  - `npm run build` completed `tsc && vite build` successfully.
  - `npm run test:frontend` completed the full frontend chain successfully, including `src/agents/AgentsHermesScreen.file-manager.test.tsx`.
- `npm run tauri:build`: PASS.
  - Tauri build completed and bundled `Zoid 25.app`.
  - Only the known Rust dead-code warnings for `apply_profile_runtime_args` and `prompt_with_enabled_profile_context` were emitted.

The scaffold avatar guards added for this fix cycle are present and pass. The earlier failed-verification notes for the unrelated file-manager test are stale: the file now uses the TS-compatible `createDomEvent()` helper, and the current build/frontend/Tauri verification passes on this checkout.

## Conclusion

The two required avatar-session fixes from the prior critique are resolved in source and covered by the updated scaffold checks. Current local verification is green. No further avatar-feature changes are requested.
