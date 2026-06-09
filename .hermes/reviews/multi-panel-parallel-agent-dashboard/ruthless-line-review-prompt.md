You are a ruthless independent reviewer. Review every line of code changed or created for the Zoid multi-panel parallel agent dashboard feature.

Repo: /Users/ziadnasreldin/Zoid
Feature handoff: /Users/ziadnasreldin/Zoid/.hermes/reviews/multi-panel-parallel-agent-dashboard/handoff.md
Plan: /Users/ziadnasreldin/Zoid/.hermes/plans/2026-06-09_113546-multi-panel-agent-chat.md

Scope only the multi-panel agent dashboard feature. Review every line in these feature files and relevant diffs:
- src/agents/AgentsHermesScreen.tsx
- src/agents/AgentMonitorPanel.tsx
- src/agents/useAgentRuntime.ts
- src/agents/dashboardLayoutState.ts
- src/agents/dashboardLayoutState.test.ts
- src/agents/continuationBrief.ts
- src/agents/continuationBrief.test.ts
- src/agents/hermesClient.ts
- src/agents/AgentsHermesScreen.file-manager.test.tsx
- src-tauri/src/lib.rs, but only the Hermes multi-run registry/commands/tests sections related to send_hermes_cli_run_message, cancel_hermes_cli_run, list_hermes_cli_runs, active run registry, scoped cancellation, and max-4 concurrency.
- src/App.css, but only dashboard/panel/monitor styling related to this feature.
- src/scaffold.test.ts, only guards added/changed for this feature.

Rules:
1. Read files from disk. Do not trust previous summaries.
2. Review line by line for correctness, race conditions, state leaks, fake concurrency, stale run handling, wrong queueing, wrong stop target, UI regressions, persistence bugs, type holes, backend command registration, security/secret exposure, and tests that give false confidence.
3. Verify claims with targeted commands/searches where useful. You may run read-only commands and tests. Do not edit files, commit, stash, reset, or modify the repo.
4. Treat unrelated dirty files as out of scope unless they directly affect this feature.
5. Output a clear verdict: APPROVED or REQUIRED_FIXES.
6. For every required fix, include exact file:line references, why it blocks the user requirement, and the smallest safe fix.
7. Include a concise list of important lines/areas you checked and accepted.
8. Save your final report to /Users/ziadnasreldin/Zoid/.hermes/reviews/multi-panel-parallel-agent-dashboard/ruthless-line-review-report.md and also print the same report as your final answer.

The user explicitly asked for a background ruthless reviewer sub-agent. Be strict and skeptical.