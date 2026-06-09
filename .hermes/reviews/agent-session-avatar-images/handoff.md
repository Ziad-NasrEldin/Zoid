# Feature Handoff: Agent session avatar images

## Original request

'/Users/ziadnasreldin/Zoid/Assets/Agent Avatars'
 use these profile images as the avatars in the agents sessions rail
make sure each session gets added gets assigned a new avatar pic that isny being used currently in the rest of the sessions

## Implementation summary

- Agents sessions rail uses the 14 JPG profile images from `Assets/Agent Avatars`, published under `public/agent-avatars`.
- New Hermes chat sessions call `chooseUniqueSessionAgentAvatarId(existingSessions, id)` so a new session receives an avatar id not currently used by the active sessions until the pool is exhausted.
- Existing sessions keep valid assigned avatar ids; invalid/legacy ids are migrated to valid agent avatar ids during localStorage hydration.
- Slash-command-created sessions use the same `createSession("New session", sessions)` path as the New Session button.
- Archived-session restore preserves the persisted `portraitId`.

## Changed files

- `src/agents/sessionPortraits.ts`: defines the 14 user-provided agent avatars and unique selection helpers.
- `src/agents/AgentsHermesScreen.tsx`: assigns and renders the agent avatars in the sessions rail.
- `src/App.tsx`: migrates missing/invalid session portrait ids to valid agent avatar ids during hydration.
- `src/scaffold.test.ts`: guards the avatar pool, asset directory, uniqueness behavior, current-state session creation, duplicate hydration repair, slash path, and archive restore invariant.
- `src/agents/AgentsHermesScreen.file-manager.test.tsx`: small TS-compatible event helper fix so the already-dirty file-manager test does not block repo build/typecheck.
- `public/agent-avatars/*.jpg`: user-provided profile images used by the rail.

## How to test

- Run `npx tsx src/scaffold.test.ts`.
- Run `npm run test:frontend`.
- Run `npm run build`.
- Run `npm run tauri:build`.
- Relaunch `/Applications/Zoid 25.app` and verify the Agents sessions rail shows image avatars.

## Tests run

- `npx tsx src/scaffold.test.ts`: PASS.
- `npm run build`: PASS.
- `npm run test:frontend`: PASS.
- `npm run tauri:build`: PASS, with pre-existing Rust dead-code warnings for `apply_profile_runtime_args` and `prompt_with_enabled_profile_context`.
- `npm run build && npm run test:frontend`: PASS after fixing the unrelated file-manager test's TS event construction.

## Git info

- Branch: current local branch.
- Commit SHA: not committed in this session.
- Diff base: current working tree has no uncommitted source diff for this feature; implementation is present in HEAD.

## Frontend/backend/database notes

- Frontend: `AgentsHermesScreen` sessions rail and app-level localStorage hydration in `App.tsx`.
- Backend: not applicable.
- Database: not applicable.

## Reviewer focus areas

- Confirm every JPG under `public/agent-avatars` is represented exactly once in the avatar pool.
- Confirm new session creation avoids currently used active-session avatars before reuse.
- Confirm legacy/missing session avatar ids are migrated to valid agent avatar ids.
- Confirm slash-command-created sessions use the same unique assignment path.
- Confirm restored archived sessions preserve the assigned avatar id.

## Fix cycle notes

Re-review request after fixing required critique items:

- R1: New-session creation now happens inside a functional `onSessionsChange((current) => ...)` updater via `prependNewSession()`, so rapid additions and slash-command additions assign against the latest current sessions, not stale render props. A pending activation ref activates the created session after the parent state update lands.
- R2: Hydration now preserves only the first valid unused avatar id; missing, invalid, or duplicate-but-valid legacy ids are reassigned using the already-resolved sessions so duplicates are repaired before pool exhaustion.
- Added scaffold guards for current-state creation, slash-command parity, and duplicate-valid hydration repair.

Additional tests after fixes:

- `npx tsx src/scaffold.test.ts`: PASS.
- `npm run build && npm run test:frontend`: PASS.
- `npm run tauri:build`: PASS, with pre-existing Rust dead-code warnings for `apply_profile_runtime_args` and `prompt_with_enabled_profile_context`.

Final verification note:

- A re-review initially found unrelated TypeScript errors in the already-dirty file-manager test file. Those were fixed with a TS-compatible `createDomEvent()` helper, then `npm run build && npm run test:frontend` and `npm run tauri:build` both passed.
