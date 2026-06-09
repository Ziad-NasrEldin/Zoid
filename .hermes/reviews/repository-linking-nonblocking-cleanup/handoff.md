# Feature critique handoff: repository-linking non-blocking notes cleanup

## Scope

User asked to address all non-blocking reviewer notes from the session-scoped repository linking review:
1. Remove misleading Code workspace "Use for Agents" / "Using for Agents" UI completely.
2. Add stronger App-level proof that global Code repository linkage is not passed into Agents.
3. Fix queued slash-command semantics so slash commands queued while another run is active still execute through the slash-command bridge, not as normal prompts.

## Intended files changed

- `src/code/CodeWorkspace.tsx`
- `src/code/CodeWorkspace.behavior.test.tsx`
- `src/agents/AgentsHermesScreen.tsx`
- `src/agents/AgentsHermesScreen.file-manager.test.tsx`
- `src/scaffold.test.ts`

Repo has unrelated dirty files from prior work. Focus on these changes.

## Implementation details

### Code workspace misleading copy removed

`src/code/CodeWorkspace.tsx`:
- Removed the repository-card button that displayed `Use for Agents` / `Using for Agents` / `Selected for Agents`.
- Removed `linkedRepositoryId` and `onLinkedRepositoryIdChange` from the CodeWorkspace prop contract and destructuring.
- Clone success no longer mutates any global linked repository state and now says the repository was added to the repository list, not selected for Agents.

`src/App.tsx`:
- Removed the stale global linked-repository storage key, initializer, state, persistence effect, CodeWorkspace props, and repository-operation `setLinkedRepositoryId` call.
- Repository operation sessions still intentionally receive their own `linkedRepositoryId` on the chat session itself.

`src/code/CodeWorkspace.behavior.test.tsx`:
- Renders two repositories so both selected/unselected old branches would have been caught.
- Added assertions that rendered Code workspace text does not include `Use for Agents`, `Using for Agents`, or `Selected for Agents`, and that `.repository-link-button` is absent.

### App-level Agents isolation guard

`src/scaffold.test.ts`:
- Added static guard that `LazyAgentsHermesScreen` receives repository catalog + chat sessions, not global linked repository selection props.
- Fails if `linkedRepositoryId=` or `onLinkedRepositoryIdChange=` appears in the `LazyAgentsHermesScreen` JSX block.
- Removed stale scaffold requirement that Code workspace include `Use for Agents`.

### Queued slash command semantics fixed

`src/agents/AgentsHermesScreen.tsx`:
- Added `QueuedHermesPrompt` kind: `prompt` or `slash`.
- Queue now stores `{ sessionId, content, kind }`.
- `handleSend` parses slash commands before queueing. If sending is active, it queues slash content as `kind: "slash"`.
- `runNextQueuedPrompt` dispatches queued slash entries through `runSlashCommand(session, content)` and normal entries through `sendHermesPrompt(session, content)`.
- `runSlashCommand` now accepts `sessionForCommand` explicitly instead of reading `activeSession`, so queued slash commands execute against the session that originally queued them.
- Command palette direct runs now also queue slash commands as slash entries if another run is active.

`src/agents/AgentsHermesScreen.file-manager.test.tsx`:
- Added deferred send mock support.
- Added regression: start a slow normal prompt, submit `/danger` while it is active, resolve the slow prompt, then assert:
  - queued slash did not create a second `send_hermes_cli_message` call;
  - queued slash did call `execute_hermes_slash_command`;
  - command text is preserved;
  - slash result renders.

## RED evidence

Before implementation:
- `npx tsx src/code/CodeWorkspace.behavior.test.tsx` failed because `Using for Agents` was still rendered.
- `npx tsx src/agents/AgentsHermesScreen.file-manager.test.tsx` failed because queued `/danger` was downgraded into a normal prompt send.
- `npx tsx src/scaffold.test.ts` passed for the new App isolation guard because the earlier session-scoped cleanup had already removed the global Agents props; scaffold later needed stale `Use for Agents` requirement removed after the UI was removed.

## Verification

Focused:
- `npx tsx src/code/CodeWorkspace.behavior.test.tsx && npx tsx src/agents/AgentsHermesScreen.file-manager.test.tsx && npx tsx src/scaffold.test.ts`
- Exit code 0.

Build/full:
- `npm run build && npm test`
- Exit code 0.
- Frontend passed.
- Rust: 73 passed, 1 ignored, 0 failed.

## Reviewer instructions

Review ruthlessly, line by line, for:
1. Any remaining `Use for Agents` / `Using for Agents` UI or implication in Code workspace.
2. Whether removing the Code button causes any broken state, accessibility, layout, or TypeScript issues.
3. Whether the App-level isolation guard actually proves global Code repository linkage cannot be passed into Agents.
4. Whether queued slash commands now preserve command semantics, original session, linked repository behavior, and confirmation behavior.
5. Whether the tests are meaningful and not brittle/false-positive.
6. Any hidden regression in normal queued prompts, immediate slash commands, command palette run behavior, or repository linking.

Return:
- Verdict: APPROVED or CHANGES_REQUIRED
- Required fixes with exact file/line evidence, or None
- Non-blocking notes separately
