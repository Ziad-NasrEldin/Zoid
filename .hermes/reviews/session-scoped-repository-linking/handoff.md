# Feature critique handoff: session-scoped repository linking

## Scope to review

Feature request: In Zoid 25 Agents chat, the repository link button at the top of the chat window must reflect the active chat session's repository only.

Required behavior:
- Each Hermes/agent chat session owns its own repository link.
- New sessions start unlinked.
- Manual repository selection changes only the active session.
- Automatic detection from chat/command text changes only the sending session.
- App-level/global Code workspace repository link must not spill into Agents chat sessions.
- Repository-operation sessions that are intentionally created with a `linkedRepositoryId` must still use that repository.

## Dirty tree warning

The repository already contains many unrelated modified/untracked files from prior Zoid work. Review this feature by source behavior, tests, and these intended touched files only:
- `src/agents/AgentsHermesScreen.tsx`
- `src/agents/AgentsHermesScreen.file-manager.test.tsx`

Do not fail this feature merely because the broader repo has unrelated dirty files, but do call out any real conflicts caused by this feature.

## Implementation summary

Implemented session-only repository linkage in `src/agents/AgentsHermesScreen.tsx`:
- Removed global `linkedRepositoryId` / `onLinkedRepositoryIdChange` props from the Agents component contract.
- Removed global `linkedRepositoryId` fallback from active topbar selection.
- Removed global `onLinkedRepositoryIdChange` mutation from Agents topbar manual selection.
- Normal prompt sends now use only `sessionForSend.linkedRepositoryId` as an existing repo; otherwise they detect from the prompt and write that detection back to that session only.
- Slash command behavior already relied on `selectedRepository`; because `selectedRepository` is now session-only, slash command routing is session-scoped too.
- Repository operation sessions remain covered because they are created with their own session `linkedRepositoryId`.

Regression coverage added in `src/agents/AgentsHermesScreen.file-manager.test.tsx`:
- New Agents session ignores a globally linked Code repository and shows `Unlinked / 未接続`.
- Generic unlinked prompt sends no `linkedRepository` path.
- Prompt mentioning `Liwan Repo` auto-links only the active session and sends the Liwan path.
- New session does not inherit the previous session’s auto-linked repo.
- Manual selection links only the active session.
- Switching sessions restores each session’s own repo link.

## Verification already run

- RED before implementation:
  - `npx tsx src/agents/AgentsHermesScreen.file-manager.test.tsx`
  - Failed as expected: new Agents session showed global `Zoid Repo` instead of `Unlinked / 未接続`.

- Focused GREEN:
  - `npx tsx src/agents/AgentsHermesScreen.file-manager.test.tsx`
  - Exit code 0.

- Build:
  - `npm run build`
  - Exit code 0.

- Full tests:
  - `npm test`
  - Exit code 0.
  - Frontend tests passed.
  - Rust tests: 73 passed, 1 ignored, 0 failed.

## Reviewer instructions

Act as the required critique-agent reviewer. Be strict.

Check specifically for:
1. Any remaining path where global Code workspace `linkedRepositoryId` can affect an unlinked Agents chat session.
2. Any manual selection path that mutates global Code workspace repo linkage from Agents.
3. Any auto-detection path that can leak into another session.
4. Queued/background sends or pending slash confirmations using the wrong session repo.
5. Tests that pass without proving the requested behavior.
6. Type/build problems from unused props or stale assumptions.
7. Regression to repository-operation sessions with intentional session repo links.

Return a critique report with:
- Verdict: `APPROVED` or `CHANGES_REQUIRED`
- Required fixes: numbered list, or `None`
- Evidence: exact files/functions/lines inspected
- Suggested non-blocking improvements separately
