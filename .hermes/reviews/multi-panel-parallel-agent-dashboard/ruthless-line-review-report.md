# Ruthless line review: multi-panel parallel agent dashboard

Verdict: REQUIRED_FIXES

Scope reviewed from disk:
- /Users/ziadnasreldin/Zoid/.hermes/reviews/multi-panel-parallel-agent-dashboard/handoff.md
- /Users/ziadnasreldin/Zoid/.hermes/plans/2026-06-09_113546-multi-panel-agent-chat.md
- src/agents/AgentsHermesScreen.tsx
- src/agents/AgentMonitorPanel.tsx
- src/agents/useAgentRuntime.ts
- src/agents/dashboardLayoutState.ts and test
- src/agents/continuationBrief.ts and test
- src/agents/hermesClient.ts
- src/agents/AgentsHermesScreen.file-manager.test.tsx, only dashboard-related sections
- src-tauri/src/lib.rs, only Hermes run registry/commands/tests sections
- src/App.css, only dashboard/panel/monitor styles
- src/scaffold.test.ts, feature-adjacent guards only

Commands run:
- npm run test:frontend: passed
- cargo test --manifest-path src-tauri/Cargo.toml hermes_cli_ -- --test-threads=1: passed, 4 tests
- npm run build: passed

Required fixes:

1. No real monitoring stream exists; the dashboard only shows stale/final-output snapshots.
   - References:
     - src-tauri/src/lib.rs:5640-5645 runs Hermes through run_hermes_command_for_session_with_cancel and returns only after the child exits.
     - src-tauri/src/lib.rs:2868-2892 waits for full output with wait_with_output, then returns stdout/stderr in one final response.
     - src/agents/AgentsHermesScreen.tsx:1141-1157 appends only the final response content after await sendHermesCliRunMessage.
     - src/agents/useAgentRuntime.ts:18-28 and 63-78 define an event schema/reconciler, but searches found no subscription or Tauri event listener using it.
   - Why this blocks: the plan explicitly requires incremental per-run output/status events and says not to fake live monitoring if the path is final-output-only. The current UI is a multi-panel final-response board, not a live agent monitoring dashboard.
   - Smallest safe fix: implement backend per-run event emission with sessionId/runId/sequence for started/output/error/completed/stopped, subscribe in the frontend runtime, append chunks only when event.runId matches the session currentRun, and cover stale-event rejection with tests. If true streaming is intentionally out of scope, get explicit product approval and remove/rename monitoring claims.

2. Queued prompts auto-start after a stopped or failed run, violating the queue lifecycle.
   - References:
     - src/agents/AgentsHermesScreen.tsx:1170-1174 sets finalRuntimeStatus to error/interrupted after failure/stop.
     - src/agents/AgentsHermesScreen.tsx:1199-1205 always calls runNextQueuedPrompt(sendingSessionId) in finally.
     - Same issue for slash commands: src/agents/AgentsHermesScreen.tsx:1010-1017 and 1046-1053 always dequeue after completion/error.
     - src/agents/AgentsHermesScreen.tsx:1056-1077 stop marks the run interrupted but leaves the later send finally path able to start queued work.
   - Why this blocks: the plan requires queued prompts not to auto-start after an error, and after stop to wait for explicit user confirmation before continuing the queue. Current behavior can launch the next queued prompt immediately after the user stops an agent.
   - Smallest safe fix: only call runNextQueuedPrompt when the just-finished run completed successfully. On error/interrupted/stop, keep the per-session queue visible and add an explicit Resume/Run queued action or confirmation.

3. Expanded chat is not wired to the expanded session’s composer/runtime.
   - References:
     - src/agents/AgentsHermesScreen.tsx:1496-1507 expanded mode renders messages and a Back button only.
     - src/agents/AgentsHermesScreen.tsx:1579 renders the single global ChatComposer outside expanded mode and wires it to handleSend.
     - src/agents/AgentsHermesScreen.tsx:1219-1231 handleSend always uses activeSession, not expandedModeSessionId.
     - src/agents/AgentsHermesScreen.tsx:1538 passes onExpand={setExpandedModeSessionId}; expanding a panel does not also focus/open that session.
   - Why this blocks: requirement says expanded full-chat mode must use the same per-session runtime, queue, and stop behavior. Today expanding a non-active panel can leave the footer composer targeting a different active session, causing prompts/stops to hit the wrong session.
   - Smallest safe fix: either focus/open the session when expanding and make the composer explicitly scoped to expandedModeSessionId, or render a scoped ChatComposer inside expanded mode with send/stop/continue bound to that session.

4. Auto-prioritize is a no-op toggle.
   - References:
     - src/agents/AgentsHermesScreen.tsx:1479 toggles dashboardState.autoPrioritize.
     - Search found no other runtime use of autoPrioritize outside persistence/tests.
   - Why this blocks: the handoff and plan require Auto-prioritize behavior, not just a saved boolean. Current UI advertises a capability that does nothing.
   - Smallest safe fix: add a runtime effect that, when autoPrioritize is enabled, reorders/focuses tiled sessions on higher-priority attention events with the planned priority order and throttling, while avoiding reordering during panel typing.

5. Monitor queued count is double-counted.
   - Reference: src/agents/AgentsHermesScreen.tsx:1237 initializes reduce with queuedHermesPromptsRef.current.length while also summing runtime.runtimeBySessionId queuedPrompts, but every queued send adds to both structures at lines 1093-1095 and 1126-1129 / 1222-1225 / 1296-1299.
   - Why this blocks: monitor bar counts are part of the feature contract. One queued prompt appears as two in the aggregate count, giving false operational state.
   - Smallest safe fix: derive the visible queued count from a single source of truth. Prefer runtimeBySessionId only, or store queue metadata in one structure and render all counts from it.

6. Backend/listing contract is incomplete and unused for stale/restart handling.
   - References:
     - src-tauri/src/lib.rs:45-51 HermesRunSnapshot lacks startedAt even though the plan contract requires it.
     - src/agents/useAgentRuntime.ts:49-56 fabricates startedAt with new Date().toISOString instead of backend start time.
     - src/agents/useAgentRuntime.ts:49-56 listAgentRuns exists, but searches found no caller in AgentsHermesScreen.tsx.
   - Why this blocks: the feature requirements include active-run discovery, stale run handling, and interrupted/restart honesty. The backend can list active runs, but the dashboard never reconciles them, and returned timestamps are not truthful.
   - Smallest safe fix: store started_at in HermesRunSlot/Snapshot, return it from list_hermes_cli_runs, and call listAgentRuns on Agents screen mount to seed runtime state or mark interrupted when runs cannot be reattached.

Important checked and accepted areas:
- Backend registry is no longer a single global active lane: src-tauri/src/lib.rs:68-75 stores a HashMap of run slots.
- Backend per-session one-run guard and global max-4 guard are present: src-tauri/src/lib.rs:2735-2755.
- Backend scoped cancellation uses sessionId + runId key and process group signaling: src-tauri/src/lib.rs:2912-2970.
- Tauri command registration includes list_hermes_cli_runs, cancel_hermes_cli_run, and send_hermes_cli_run_message: src-tauri/src/lib.rs:5828-5832.
- Frontend panel sends carry distinct sessionId/runId to send_hermes_cli_run_message: src/agents/AgentsHermesScreen.tsx:1141-1148.
- Same-session queueing path exists and no longer deliberately dequeues another session’s prompt: src/agents/AgentsHermesScreen.tsx:1097-1110.
- Dashboard layout persistence sanitizes unknown versions, duplicate/missing IDs, bad layout modes, and max tile count: src/agents/dashboardLayoutState.ts:24-51.
- Continuation brief uses only the selected session messages and is bounded to 3,200 chars: src/agents/continuationBrief.ts:11-33.
- Dashboard styles provide the requested max four grid, panel, monitor bar, and mobile stacking basics: src/App.css:440-467.
- Tests do prove four frontend invoke calls can be outstanding and backend different-session overlap/scoped cancellation works, but they do not prove live streaming, expanded composer scoping, no auto-start after stop/error, auto-prioritize behavior, or truthful restart/list reconciliation.

Out-of-scope note:
- The worktree contains many unrelated dirty/untracked files. I did not review unrelated changes except where the requested files directly intersected this feature.
