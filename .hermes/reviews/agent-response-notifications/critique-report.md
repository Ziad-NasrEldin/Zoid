# Critique Report — Agent Response Notifications

Verdict: APPROVED

## What I reviewed

Focused only on the previously required issues and touched files from the handoff/prior critique:

- `src/agents/AgentsHermesScreen.tsx`
- `src/App.tsx`
- `src-tauri/src/lib.rs`
- `src/scaffold.test.ts`
- Previously reported stale workspace visibility/unmount behavior
- Previously reported backend email command-boundary bounding

## Findings

No remaining required changes.

### Previously required fix: stale workspace visibility on unmount

Approved.

`AgentsHermesScreen.tsx` now keeps `isAgentsWorkspaceOpenRef` synchronized while mounted and, importantly, the effect cleanup sets `isAgentsWorkspaceOpenRef.current = false` on unmount:

- `isAgentsWorkspaceOpenRef` is initialized from the prop.
- The effect updates the ref from `isAgentsWorkspaceOpen`.
- The cleanup mutates the same ref object captured by in-flight async send/command continuations.
- `notifyForBackgroundAgentResponse` suppresses only when `isAgentsWorkspaceOpenRef.current` is true and the responding session is selected.

This closes the previous hole: if the user sends a prompt in the selected Hermes session, leaves the Agents workspace before the response resolves, and the component unmounts, the cleanup flips the ref to false. The pending async continuation still sees the same ref object and will treat the selected session as not currently open, so it can mark `needsReply` and send notifications.

`App.tsx` still conditionally renders `AgentsHermesScreen`, but with the cleanup fix this is no longer a blocker for the previously identified stale-ref path.

### Previously required fix: backend command-boundary bounding

Approved.

`src-tauri/src/lib.rs` includes backend-side bounding helpers required by the prior critique:

- `bounded_email_header` for subject/session-title style header values.
- `bounded_email_body` for summary/body size control before SMTP send.

This satisfies the previous backend boundary requirement, independent of frontend summary limits.

### Scaffold guards

`src/scaffold.test.ts` includes string guards for the notification client/backend/UI and the backend bounding symbols. As noted before, these are not behavior-level tests for the unmount race, but the implementation itself addresses the required issue.

## Validation run

- `npm run test:frontend` passed.

## Required fixes

None.
