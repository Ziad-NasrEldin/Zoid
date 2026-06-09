# Ruthless line-by-line review: Zoid 25 animation system

Verdict: REQUIRED_FIXES

Review target: commit `0a3c8f8` versus parent `0b946f9`.

Note: the main working tree had uncommitted edits in `src/App.css`, `src/social/SocialDashboard.behavior.test.tsx`, and `src/social/SocialDashboard.tsx` during review. To avoid mixing post-commit edits into the verdict, verification used a clean archive snapshot of `0a3c8f8` at `/private/tmp/zoid-25-review-0a3c8f8`.

## Blocking findings

1. `src/agents/AgentsHermesScreen.tsx:552` / `src/agents/AgentsHermesScreen.tsx:560` / `src/agents/AgentsHermesScreen.tsx:1281`
   - Logic regression: backend terminal events can start the next queued prompt before the original `sendHermesPrompt` call reaches its `finally`.
   - `agent-run-completed` marks runtime finished and calls `runNextQueuedPrompt()` immediately at line 560.
   - The queued prompt can start and overwrite `activeHermesRunsRef.current` for the same session.
   - Then the original call's `finally` deletes `activeHermesRunsRef.current` for that session at line 1281, potentially deleting the newly started run from frontend state.
   - Result: Stop/running UI can desync, and subsequent sends can be incorrectly allowed or queued against the wrong active run.
   - Existing frontend tests do not simulate `agent-run-completed`; they only resolve the IPC promise, so they miss this event-path race.

2. `src/agents/AgentMonitorPanel.tsx:43`
   - Accessibility/UI regression: the panel is focusable with `tabIndex={0}` and has click behavior, but it has no keyboard equivalent and no interactive role.
   - Keyboard users can tab to the panel but Enter/Space do nothing.
   - Either remove the focusable/clickable panel shell or add a role plus keyboard handling that performs the same action.

3. `src-tauri/src/lib.rs:4382`, `src-tauri/src/lib.rs:4488`, `src-tauri/src/lib.rs:4649`
   - Broken tests / side-effect regression: ordinary read/list operations now persist permission markers through `remember_file_permission_path*` without test isolation.
   - `npm run test:rust` fails in a clean commit snapshot:
     - `tests::file_manager_listing_is_lazy_and_finder_sorted` panics at `src/lib.rs:7272`
     - `tests::scan_repository_folder_returns_nested_git_repositories_without_duplicates` panics at `src/lib.rs:7416`
     - `tests::github_branch_lookup_uses_gh_api_and_marks_default` panics at `src/lib.rs:7494`
   - Failure text: `Failed to save permissions marker: Operation not permitted (os error 1)`.
   - The tests must redirect marker storage to a temp app-data path, avoid marker persistence in pure unit paths, or handle permission-marker failures without breaking read-only repository/file-manager operations.

## Security / secret exposure

- No committed API keys, access tokens, passwords, or private key material found in changed code.
- MaVoid Buffer code exposes endpoint and job IDs, not token values.
- Provider API key UI continues to use password inputs and Keychain-backed storage paths.

## Accessibility / reduced motion

- Required fix above: `AgentMonitorPanel` has focusable click-only panel behavior.
- Reduced-motion CSS coverage exists for the newly added dashboard feed animation at `src/App.css:1646-1655`.
- Smooth scroll remains in `AgentsHermesScreen.tsx:510`; not marked blocking because it predates/extends existing behavior, but it should respect `prefers-reduced-motion` in a follow-up.

## UI / layout risks

- `AgentMonitorPanel.tsx:51-52` uses text arrow controls instead of icon buttons. This is not a blocker, but it is less consistent with the app's icon-button conventions.
- Dashboard panels clamp feed lines to 3 lines at `src/App.css:577`; acceptable for a monitor view, but users may miss long error output unless the expanded view is obvious.

## Tests and stale assertions

- `npm run build` passes in the clean snapshot.
- `npm run test:frontend` passes in the clean snapshot.
- `npm run test:rust` fails with 3 failing tests listed above.
- Existing frontend parallel runtime tests at `src/agents/AgentsHermesScreen.file-manager.test.tsx:449-516` do not cover backend event ordering, so they are stale relative to the new event-streaming code path.

## Exact commands run

```sh
lean-ctx -c "git rev-parse --show-toplevel && git status --short && git diff --stat 0b946f9 0a3c8f8 && git diff --name-only 0b946f9 0a3c8f8"
lean-ctx -c "git diff --find-renames --find-copies --full-index --unified=80 0b946f9 0a3c8f8"
lean-ctx -c "git diff --name-only 0b946f9 0a3c8f8 | rg '\\.(ts|tsx|css|rs|json|md)$'"
git diff --name-only 0b946f9 0a3c8f8 | rg '\\.(ts|tsx|css|rs|json|md)$'
/Users/ziadnasreldin/.local/bin/lean-ctx -c 'git diff --name-only 0b946f9 0a3c8f8' | rg '\\.(ts|tsx|css|rs|json|md)$'
lean-ctx -c "git diff --name-only 0b946f9 0a3c8f8" --raw | rg '\\.(ts|tsx|css|rs|json|md)$'
lean-ctx --help | sed -n '1,120p'
lean-ctx -c --raw "git diff --name-only 0b946f9 0a3c8f8" | rg '\\.(ts|tsx|css|rs|json|md)$'
lean-ctx -c --raw "rm -rf /private/tmp/zoid-25-review-0a3c8f8 && git worktree add --detach /private/tmp/zoid-25-review-0a3c8f8 0a3c8f8"
lean-ctx -c --raw "rm -rf /private/tmp/zoid-25-review-0a3c8f8 && mkdir -p /private/tmp/zoid-25-review-0a3c8f8 && git archive 0a3c8f8 | tar -x -C /private/tmp/zoid-25-review-0a3c8f8"
lean-ctx -c --raw "git diff --unified=20 0b946f9 0a3c8f8 -- src/agents/useAgentRuntime.ts src/agents/AgentsHermesScreen.tsx src-tauri/src/lib.rs src/App.tsx src/App.css"
lean-ctx -c --raw "cat package.json"
lean-ctx -c --raw "rg -n \"TODO|FIXME|secret|api[_-]?key|password|token|dangerouslySetInnerHTML|innerHTML|localStorage|prefers-reduced-motion|animation|transition|@keyframes|setInterval|setTimeout|invoke\\(|Command::new|shell|spawn|unwrap\\(|expect\\(\" src src-tauri .hermes/reviews/zoid-25-animation-system package.json vite.config.ts"
lean-ctx -c --raw "rg -n \"startAgentRun|stopAgentRun|listAgentRuns|agent-run-event|tryStartSessionRun|markSessionRunStarted|markSessionRunFinished|activeHermesRunsRef|queueHermesPrompt|runSlashCommand|handleStopHermesRun|useAgentRuntime\" src/agents/AgentsHermesScreen.tsx src/agents/useAgentRuntime.ts src-tauri/src/lib.rs"
lean-ctx -c --raw "ln -s /Users/ziadnasreldin/Zoid/node_modules /private/tmp/zoid-25-review-0a3c8f8/node_modules 2>/dev/null || true && npm run build"
lean-ctx -c --raw "npm run test:frontend"
lean-ctx -c --raw "nl -ba src/agents/AgentsHermesScreen.tsx | sed -n '500,580p;1018,1155p;1160,1308p;1388,1416p;1628,1660p;1686,1700p;1724,1738p'"
lean-ctx -c --raw "nl -ba src/agents/useAgentRuntime.ts | sed -n '1,140p'"
lean-ctx -c --raw "nl -ba src-tauri/src/lib.rs | sed -n '2840,3225p;4330,4385p;4900,4965p;5000,5075p'"
lean-ctx -c --raw "git diff --unified=0 0b946f9 0a3c8f8 -- src/App.css | sed -n '1,260p'"
lean-ctx -c --raw "git diff --unified=20 0b946f9 0a3c8f8 -- src/agents/AgentsHermesScreen.file-manager.test.tsx src/agents/dashboardLayoutState.test.ts src/agents/continuationBrief.test.ts src/social/SocialDashboard.behavior.test.tsx src/code/CodeWorkspace.behavior.test.tsx"
lean-ctx -c --raw "rg -n \"motion-|@keyframes|prefers-reduced-motion|scrollTo\\(|behavior: \\\"smooth\\\"|animation:\" src/App.css src/agents/AgentsHermesScreen.tsx src/code/CodeWorkspace.tsx src/social/SocialDashboard.tsx src/automations/AutomationsWorkspace.tsx src/content/ContentWorkspace.tsx"
lean-ctx -c --raw "nl -ba src/App.css | sed -n '580,610p;1638,1662p'"
lean-ctx -c --raw "npm run test:rust"
lean-ctx -c --raw "rg -n \"parallel|queued|runNextQueuedPrompt|agent-run-completed|activeHermesRunsRef|list_hermes_cli_runs|cancel_hermes_cli_run|send_hermes_cli_run_message|MAX_ACTIVE_AGENT_RUNS|4 agents\" src/agents/AgentsHermesScreen.file-manager.test.tsx src-tauri/src/lib.rs"
lean-ctx -c --raw "nl -ba src/agents/AgentsHermesScreen.file-manager.test.tsx | sed -n '400,525p'"
lean-ctx -c --raw "nl -ba src-tauri/src/lib.rs | sed -n '6030,6175p;7710,7895p'"
lean-ctx -c --raw "nl -ba src/agents/hermesClient.ts | sed -n '1,160p'"
lean-ctx -c --raw "nl -ba src/agents/AgentMonitorPanel.tsx | sed -n '1,240p'"
lean-ctx -c --raw "nl -ba src-tauri/src/lib.rs | sed -n '7248,7280p;7404,7422p;7478,7502p'"
lean-ctx -c --raw "git diff --unified=20 0b946f9 0a3c8f8 -- src-tauri/src/lib.rs | rg -n \"remember_file_permission_path|list_file_manager_directory_inner|scan_repository_folder|list_remote_branches|permission\" -C 6"
lean-ctx -c --raw 'rg -n "fn remember_file_permission_path|remember_file_permission_path\\(" src-tauri/src/lib.rs'
lean-ctx -c --raw "nl -ba src-tauri/src/lib.rs | sed -n '3928,3972p;4374,4390p;4478,4492p;4708,4720p'"
lean-ctx -c --raw "nl -ba src-tauri/src/lib.rs | sed -n '4550,4590p;4636,4660p'"
```
