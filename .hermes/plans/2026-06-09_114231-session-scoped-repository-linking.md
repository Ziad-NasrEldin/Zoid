# Session-scoped repository linking for Agents chat

## Goal

Make the repository selector at the top of the Agents chat reflect the repository for the active chat session only.

Required behavior:
- Each Hermes/agent chat session has its own repository link.
- New sessions start unlinked by default.
- Manual selection updates only the active session.
- Automatic detection from chat/slash-command content links only the session that triggered detection.
- Repository linkage must not spill across sessions through the global Code workspace link.
- Existing repository-operation sessions can still open pre-linked to the operation repository.

## Current context observed

Repo: `/Users/ziadnasreldin/Zoid`

Relevant current behavior found in code:
- `src/agents/sessionState.ts` already supports per-session `linkedRepositoryId?: string` and `createSession()` creates sessions without a repo link.
- `src/agents/AgentsHermesScreen.tsx` already has prompt-based repo detection:
  - `detectRepositoryFromPrompt()`
  - `sendHermesPrompt()`
  - `runSlashCommand()`
- The topbar selector currently computes:
  - `activeRepositoryId = activeSession?.linkedRepositoryId ?? linkedRepositoryId ?? "none"`
  - This means a global `linkedRepositoryId` can appear in any unlinked session, violating the requested per-session isolation.
- `handleLinkedRepositoryChange()` currently calls `onLinkedRepositoryIdChange?.(repositoryId)` and updates the active session, so manual selection can still mutate global Code workspace state.
- `sendHermesPrompt()` currently uses:
  - `effectiveRepositoryId = sessionForSend.linkedRepositoryId ?? linkedRepositoryId`
  - This can route an unlinked session through the global repo before chat detection, also violating isolation.
- `src/App.tsx` keeps a global `linkedRepositoryId` for Code workspace and passes it into Agents. That global state should remain for Code workspace, but Agents should not use it as a fallback for session links.

## Proposed approach

Treat repository selection in Agents as session-owned state.

Keep global Code workspace linkage intact for the Code page, but stop using it as a fallback inside `AgentsHermesScreen` for ordinary chat sessions.

Use this precedence in Agents:
1. Active session `linkedRepositoryId` if set.
2. Auto-detected repository from the current submitted prompt/command if the session has no link yet.
3. `none` / unlinked.

Do not use app-level `linkedRepositoryId` as a display or send fallback in Agents.

Repository-operation sessions remain safe because `handleStartRepositoryOperation()` in `src/App.tsx` creates those sessions with `linkedRepositoryId: repository.id`; they will still display and run against that repository because it is session-local.

## Step-by-step implementation plan

1. Tighten Agents session-link model
   - In `src/agents/AgentsHermesScreen.tsx`, change active repo derivation:
     - From: `activeSession?.linkedRepositoryId ?? linkedRepositoryId ?? "none"`
     - To: `activeSession?.linkedRepositoryId ?? "none"`
   - Keep `selectedRepository` based on that session-only ID.
   - Leave the prop temporarily if still needed for compatibility, or remove it if no remaining uses exist after cleanup.

2. Make manual topbar selection session-only
   - In `handleLinkedRepositoryChange(repositoryId)`:
     - Remove the call to `onLinkedRepositoryIdChange?.(repositoryId)` for the Agents topbar.
     - Update only `activeSession.linkedRepositoryId`.
     - Store `undefined` when the dropdown value is `"none"`.
   - If TypeScript shows `onLinkedRepositoryIdChange` is now unused in Agents, remove it from `AgentsHermesScreenProps` and from the `LazyAgentsHermesScreen` call in `src/App.tsx`.
   - Do not change `CodeWorkspace` behavior; its “Use for Agents” copy may need follow-up because the requested model makes Agents session-scoped, not globally linked.

3. Make send routing session-only
   - In `sendHermesPrompt(sessionForSend, content)`:
     - Remove fallback to app-level `linkedRepositoryId`.
     - Use `sessionForSend.linkedRepositoryId` as the only existing link.
     - If no current session link, run `detectRepositoryFromPrompt(content, repositories)`.
     - Save the detected repo ID onto only `sendingSessionId`.
     - Pass only that repo’s path to `sendHermesCliMessage()`.
   - Preserve existing behavior where, once auto-detected, later messages in the same session keep using that session repository until manually changed/unlinked.

4. Make slash-command routing session-only
   - In `runSlashCommand(command, confirmed)`:
     - Keep using `selectedRepository` because it will now be session-only.
     - If no session repo is selected, detect from the slash command text.
     - Save detection only to the active session.
     - Pass only that session/detected repo path into `executeHermesSlashCommand()`.
   - In pending confirmation flow, keep `linkedRepositoryPath` captured from the session/detected repository so confirmation cannot be affected by later session switching.

5. Add regression coverage
   - Extend `src/agents/AgentsHermesScreen.file-manager.test.tsx` or create a focused `src/agents/AgentsHermesScreen.repository-linking.test.tsx` if the existing test file is too broad.
   - Cover at least:
     1. New session displays `Unlinked / 未接続` even when app-level/global `linkedRepositoryId` is set.
     2. Selecting a repository in session A does not change session B’s topbar value.
     3. Prompt mentioning repository name/path auto-links only the sending session.
     4. Switching between sessions restores each session’s own linked repo/unlinked state.
     5. Sending from an unlinked session with no repo mention calls Hermes with no repository path.
     6. Sending after auto-detection calls Hermes with the detected repository path.
   - Update `src/scaffold.test.ts` only if it contains structural assertions that assume the global Agents link fallback.

6. Clean UX copy if needed
   - Inspect `src/code/CodeWorkspace.tsx` button copy currently saying `Using for Agents` / `Use for Agents`.
   - If that button still controls global `linkedRepositoryId`, either:
     - Leave it for a separate decision if Code page global linking still matters, or
     - Reword in implementation to avoid implying all Agents sessions inherit it.
   - Recommended default for this request: do not expand scope unless tests fail or the label becomes actively misleading in-app.

7. Run local verification
   - `npm run test:frontend`
   - `npm run build`
   - If frontend passes but Rust is untouched, full `npm test` is still preferred before completion because the project script includes Rust.

8. Run required feature critique gate before calling implementation complete
   - Create `.hermes/reviews/session-scoped-repository-linking/handoff.md` after implementation.
   - Trigger or wait for a separate critique-agent review.
   - Fix every Required item.
   - Re-review until verdict is `APPROVED`.
   - Do not mark feature complete before this gate passes unless explicitly waived.

## Likely paths to touch

- `src/agents/AgentsHermesScreen.tsx`
- `src/agents/AgentsHermesScreen.file-manager.test.tsx` or new `src/agents/AgentsHermesScreen.repository-linking.test.tsx`
- `src/scaffold.test.ts` if structural assertions need updates
- Possibly `src/App.tsx` only to remove now-unused Agents props
- Possibly `src/code/CodeWorkspace.tsx` only if copy must stop implying global Agents linkage
- `.hermes/reviews/session-scoped-repository-linking/handoff.md` during implementation review gate

## Tests / validation checklist

Automated:
- `npm run test:frontend`
- `npm run build`
- Prefer full `npm test` before final handoff

Manual/E2E smoke in the running Zoid app:
1. Open Agents.
2. Confirm current/new session topbar starts `Unlinked / 未接続`.
3. Manually select repo A in session A.
4. Create session B; confirm topbar is unlinked, not repo A.
5. Send a message in session B mentioning a known repo name/path; confirm topbar changes only for session B.
6. Switch back to session A; confirm repo A is still selected.
7. Create session C and send a generic message with no repo mention; confirm no repo is selected and Hermes receives no repo path.
8. Confirm repository-operation-launched sessions still open linked to the operation repo.

## Risks and decisions

- Risk: Code workspace has a global `linkedRepositoryId` and UI copy that may imply a shared Agents repo. The core fix should isolate Agents first; copy cleanup can be included if it is small and unambiguous.
- Risk: Existing tests may encode the old global fallback behavior. Update tests to lock the new session-scoped contract.
- Risk: Auto-detection by repository name can false-match short/common names. Current implementation already requires at least 3 chars and word-ish boundaries. Keep that behavior unless a bug appears.
- Decision: New chat sessions should not inherit the current global Code repository. This matches the user’s explicit requirement.
- Decision: Once a prompt auto-detects a repo, the session stays linked until manually changed/unlinked. This matches “reflect in the repository link button” and avoids repeated detection drift.

## Definition of done

- Agents topbar repository value is derived from the active session only.
- Manual repository selection updates only the active chat session.
- Auto-detection updates only the session where the message/command was sent.
- New sessions are unlinked by default.
- Global Code workspace repo state no longer spills into Agents chat sessions.
- Frontend tests/build pass.
- Feature critique gate reaches `APPROVED`.