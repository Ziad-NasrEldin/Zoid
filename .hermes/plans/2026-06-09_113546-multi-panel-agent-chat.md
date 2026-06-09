# Multi-Panel Parallel Agent Monitoring Dashboard Implementation Plan

> Plan-only artifact. Do not implement until Ziad explicitly approves execution.

Status: revised after critique-agent review.
Critique verdict before this revision: NEEDS_REVISION.
Primary critique correction: the plan needed a concrete runtime contract, event model, process lifecycle, queue semantics, stale-event handling, and objective proof of true parallelism before dashboard UI work.

Brainstorm source:
- `/Users/ziadnasreldin/brainstorms/2026-06-09-zoid-multi-panel-agent-chat.md`

Goal:
Build the Zoid 25 Agents page into a true multi-agent monitoring dashboard where the user can drag sessions/agents from the Sessions rail into the main area, tile up to 4 panels, and run up to 4 different agent tasks concurrently with independent per-session controls.

Critical requirement:
This is not just multi-panel viewing. Ziad requires real parallel execution: 4 panels must be able to run 4 different agent tasks at the same time. Do not ship a UI that looks parallel while using one global Hermes lane.

---

## Current context / assumptions

Repo:
- `/Users/ziadnasreldin/Zoid`

Known app/test setup from `package.json`:
- App: `zoid-25`, version `0.25.0`.
- Frontend build: `npm run build` = `tsc && vite build`.
- Frontend tests currently use `tsx` directly through `npm run test:frontend`.
- Rust/Tauri tests: `npm run test:rust` = `cargo test --manifest-path src-tauri/Cargo.toml --lib --bins -- --test-threads=1`.
- Full tests: `npm run test` = frontend + Rust tests.

Known frontend files:
- `src/agents/AgentsHermesScreen.tsx`
- `src/agents/ChatComposer.tsx`
- `src/agents/sessionState.ts`
- `src/agents/types.ts`
- `src/App.css`

Current Agents page appears to have:
- one `activeSessionId` chat stage;
- one global composer;
- global send/stop state such as `isSending` and `activeHermesRunRef`;
- a global pending prompt queue.

Required correction:
- The global run lane must be replaced by session-scoped runtime state before the dashboard UX can be considered complete.
- `useAgentRuntime.ts` or an equivalent dedicated runtime module is required, not optional.
- `AgentsHermesScreen.tsx` should orchestrate UI but must not become the owner of all process/event/queue lifecycle logic.

---

## Non-negotiable runtime invariants

1. One active run per Zoid session.
   - A session may have at most one `currentRun` at a time.
   - Sending another prompt to the same running session queues behind that same session only.

2. Up to 4 active runs globally for this build.
   - This is a hard v1 concurrency cap.
   - Same-session queued prompts do not count as active runs.
   - If 4 runs are already active, Send/Continue for idle sessions is disabled or blocked with: `4 agents already running`.
   - Running sessions can be visible or hidden, but hidden running sessions must remain discoverable in the Sessions rail and via Show active agents.

3. Tiled panels and active runs are related but not identical.
   - Max visible tiled panels: 4.
   - Max active runs: 4.
   - Removing/untilling a panel does not stop that session’s run.
   - Replacing a panel does not stop the replaced session’s run.
   - Running sessions not currently tiled must show running markers in the rail and be recoverable with Show active agents.

4. No output mixing.
   - Every backend event must include `sessionId` and `runId`.
   - Frontend appends output only when `event.runId` matches the session’s current run, or when explicitly processing a known historical/replay event.
   - Late/stale events from stopped/completed runs must not append to a new run.

5. Stop is scoped.
   - Stop affects one run ID only.
   - Stop must not kill other panels/runs.
   - Stop is idempotent: stopping an unknown, already-completed, or already-stopped run returns safely.

6. Dashboard UI must not imply capabilities the runtime does not prove.
   - Dashboard UX work is blocked until mock/backend concurrency tests prove scoped concurrent execution.

---

## Required frontend/backend runtime contract

Implementation must converge on a concrete contract like this. Names can adapt to existing Tauri conventions, but the semantics are required.

Frontend callable commands:

```ts
type StartAgentRunInput = {
  sessionId: string;
  prompt: string;
  hermesSessionId?: string;
  cwd?: string;
  metadata?: Record<string, string>;
};

type StartAgentRunResult = {
  runId: string;
  sessionId: string;
  startedAt: string;
};

startAgentRun(input: StartAgentRunInput): Promise<StartAgentRunResult>;

stopAgentRun(input: { runId: string }): Promise<{
  runId: string;
  stopped: boolean;
  reason?: "running" | "already-completed" | "already-stopped" | "unknown-run";
}>;

listAgentRuns(): Promise<Array<{
  runId: string;
  sessionId: string;
  status: "running" | "stopping";
  startedAt: string;
  pid?: number;
}>>;
```

Event schema:

```ts
type AgentRunEvent = {
  type:
    | "agent-run-started"
    | "agent-run-output"
    | "agent-run-error"
    | "agent-run-needs-input"
    | "agent-run-completed"
    | "agent-run-stopped";
  runId: string;
  sessionId: string;
  timestamp: string;
  sequence: number;
  channel?: "stdout" | "stderr" | "system";
  chunk?: string;
  message?: string;
  exitCode?: number;
};
```

Backend run registry requirements:
- Registry keyed by `runId`.
- Each entry stores `sessionId`, process handle/child pid if applicable, start time, current state, and sequence counter.
- Completion/error/stop emits a terminal event before registry cleanup.
- Cleanup happens on completion, error, stop, and app exit.
- App exit must not leave orphan Hermes processes.
- If user confirms close while runs are active, backend attempts graceful stop/kill for active child processes.
- Stop escalation: graceful stop first, then force kill after an implementation-defined timeout.

Streaming requirement:
- v1 dashboard requires per-run incremental output or status events.
- If the existing Hermes path is final-output-only, implementation must introduce event streaming before dashboard UX can be called complete.
- If true streaming cannot be implemented, stop and ask Ziad to approve a revised scope; do not fake live monitoring.

Security/process safety:
- Do not spawn Hermes commands through shell-concatenated user prompts.
- Pass prompts, cwd, session IDs, and file paths as safe process arguments/stdin/structured payloads.
- Ensure user prompt text cannot break shell command boundaries.

---

## Session runtime state model

Required file/module:
- Create: `src/agents/useAgentRuntime.ts`

Likely supporting files:
- Modify: `src/agents/types.ts`
- Optional create: `src/agents/agentRuntimeTypes.ts`

Candidate types:

```ts
type AgentRunStatus = "idle" | "running" | "needs-input" | "error" | "interrupted";

type AgentRunHandle = {
  runId: string;
  sessionId: string;
  startedAt: string;
  processId?: number;
  backendHandle?: string;
};

type AgentSessionRuntimeState = {
  sessionId: string;
  status: AgentRunStatus;
  currentRun?: AgentRunHandle;
  queuedPrompts: string[];
  lastStartedAt?: string;
  lastFinishedAt?: string;
  lastError?: string;
  wasRunningBeforeClose?: boolean;
};
```

`useAgentRuntime.ts` responsibilities:
- Runtime state map keyed by session ID.
- Start/stop lifecycle.
- Per-session queue operations.
- Event subscription and event reconciliation.
- Stale event rejection.
- Derived counts for Monitor Bar.
- Global concurrency cap handling.
- Close/restart interrupted state coordination.

Agent runtime API from hook/module:

```ts
sendPromptToSession(sessionId: string, prompt: string): Promise<void>;
stopSessionRun(sessionId: string): Promise<void>;
continueSession(sessionId: string): Promise<void>;
getSessionRuntime(sessionId: string): AgentSessionRuntimeState;
getActiveRunCount(): number;
```

Stale event handling:
- If no matching session exists, ignore and log compactly.
- If `event.runId !== runtimeState[sessionId].currentRun?.runId`, ignore or store as historical only; never append to the active run transcript.
- If a session is removed from dashboard while running, the run continues and rail markers remain active.
- If a session is archived/deleted while running, block or require confirmation and stop the run first.

---

## Status derivation rules

Status strip states must be deterministic.

- `running`: session has `currentRun` and no terminal event has been received.
- `needs-input`: backend emits explicit `agent-run-needs-input`; fallback heuristic may detect clear requests for user input, but must be conservative.
- `error`: latest terminal event for the session was `agent-run-error`.
- `interrupted`: persisted `wasRunningBeforeClose` or restart sees a run that was active before close but no longer reconnectable.
- `idle`: no current run and no error/interrupted/needs-input state.

Needs-input heuristic if backend lacks explicit signal:
- Only mark needs-input for strongly matching phrases or metadata, not every assistant question.
- Add tests to avoid over-detecting normal rhetorical/implementation questions.

Priority order for Auto-prioritize:
1. needs-input
2. error
3. interrupted
4. running
5. idle

Auto-prioritize rules:
- Off by default.
- Reorder only when a session enters a higher-priority state.
- Do not reorder more than once every 5 seconds.
- Do not reorder while user is typing in any panel prompt.
- Manual Make primary/focus pins primary until the next higher-priority attention event.
- Show a subtle indication when Auto-prioritize moved a panel.

---

## Queue lifecycle

Per-session queue:
- Sending to a running session appends prompt to that session’s `queuedPrompts`.
- Queue count appears in that panel and in the rail.
- When current run completes successfully, the next queued prompt for that same session can start automatically.
- If current run errors, queued prompts do not auto-start; show queue and Retry/Continue options.
- If user stops the current run, ask whether to keep or clear that session’s queue.
- After a stop, do not auto-start kept queued prompts until user confirms continuing that queue.

Persistence:
- Do not persist queued prompts across app restart for v1 unless a deliberate saved-queue mechanism is added.
- On app close/restart with queued prompts behind a running prompt, mark the session interrupted/needs-review and clear volatile currentRun.
- If queued prompts are not persisted, show an honest message rather than resurrecting stale work.

---

## Continue where you left off

Placement:
- Sessions rail: small Continue action.
- Panel header: visible Continue when idle/interrupted/needs-review.
- Expanded chat view: full “Continue where left off” action near composer.
- If session is running: show “Already running”.
- If global active run cap is reached: disable and show `4 agents already running`.

Send behavior:
- Continue sends immediately, with no preview modal by default.
- It uses `sendPromptToSession(sessionId, continuationBrief)`.

Continuation brief policy:
- Max brief target: compact enough for normal prompt use; implementation should define a character/token budget before coding.
- Do not dump full raw chat by default.
- Include:
  - session title;
  - original/latest user goal;
  - recent conversation summary;
  - last assistant state/result;
  - unresolved TODOs/open questions;
  - files/repos mentioned;
  - explicit instruction: `Continue this same session. Do not switch tasks or assume a different session.`
- Message selection strategy:
  - inspect only this session’s messages;
  - include the latest user task and recent assistant state;
  - include last N relevant messages or summarized excerpts;
  - extract file paths/repos from this session only.
- Fallback for empty/short session:
  - send a brief stating insufficient prior context and ask the agent to proceed only from the available session title/latest prompt.
- Tests must prove the generated brief does not include messages from another session.

Likely file:
- Create: `src/agents/continuationBrief.ts`

---

## Dashboard UX decisions

Mental model:
- Multi-agent monitoring dashboard.

Panel priority:
- State first, transcript second.
- Show name, status, latest action/response preview, elapsed time, repo/context.

Layouts:
- 1 panel: single-chat view.
- 2 panels: two columns desktop.
- 3 panels: large primary left + two stacked right.
- 4 panels: 2x2 grid.
- Narrow widths: focused panel + panel switcher chips or stacked cards; never crush four unusable tiny panels.

Panel behavior:
- Single click: focus/select panel.
- Double click or Expand: full single-chat view.
- Escape or Back to dashboard: return from expanded mode, but avoid stealing Escape from non-empty composer or modal dialogs.

Expanded state model:
- Use `expandedModeSessionId` separate from `focusedSessionId` and `primarySessionId`.
- Expanded mode is a view overlay/state, not dashboard mutation.
- Other runs continue in background.
- Show active run count/back-to-dashboard affordance while expanded.
- Full ChatComposer uses the same per-session runtime state, queue, and stop behavior.

Monitor Bar:
- Auto-prioritize toggle.
- Show active agents button.
- Clear dashboard / untile all.
- Layout mode selector: Auto / 2-col / Focus+stack / 2x2.
- Compact counts: tiled, running, needs reply, queued.

Show active agents:
- One-click smart fill.
- Includes hidden running sessions.
- Deterministic ordering by priority: needs-input, error, interrupted, running, queued, idle recently active.
- Fills empty slots without removing manually pinned panels.
- If full, opens keyboard-accessible replacement picker.
- If replacing/hiding a running panel, explain: `Agent will continue running in background` and offer Stop + replace.

Remove/untile language:
- Use “Remove from dashboard” or “Untile”.
- Do not use “Close” for non-destructive panel removal.
- Untiling never deletes session and never stops a run.

Sessions rail requirements:
- Running marker.
- Needs-reply marker.
- Error/interrupted marker.
- Queue count.
- Add to dashboard action.
- Continue / Already running action.
- Running sessions hidden from dashboard remain visible/discoverable in rail.

---

## Dashboard persistence

Likely file:
- Create: `src/agents/dashboardLayoutState.ts`

Storage key:
- `zoid25:agents-dashboard`

State must include versioning:

```ts
type AgentDashboardStateV1 = {
  version: 1;
  tiledSessionIds: string[];
  primarySessionId?: string;
  focusedSessionId?: string;
  layoutMode: "auto" | "split-2" | "focus-stack" | "quad";
  autoPrioritize: boolean;
};
```

Requirements:
- Safe parse with fallback.
- Discard or migrate unknown versions.
- Validate layout mode.
- Remove duplicate session IDs.
- Remove archived/missing sessions.
- Cap tiled sessions at 4.
- Ensure primary/focused IDs point to existing visible sessions or are reset.
- Tests for corrupt localStorage, unknown version, archived sessions, and missing IDs.

---

## Build gates

Gate A — Runtime discovery complete:
- A runtime discovery note exists in the implementation handoff or plan addendum.
- It documents current Tauri command names, event names, process handle ownership, blocking/streaming behavior, and stop semantics.

Gate B — Mock backend concurrency tests pass:
- Four mock runs can start concurrently.
- Interleaved events stay separated by sessionId/runId.
- Stopping one run does not stop others.

Gate C — Real backend four-run proof passes:
- Four real or integration-level Hermes runs overlap by timestamp.
- Events/logs show four distinct runIds.
- Stopping one leaves the other three running.

Gate D — Dashboard UX implemented on top of proven runtime:
- Tiling, Monitor Bar, compact panel composers, and layout persistence work.

Gate E — Close/restart/Continue proof:
- Closing warns on active runs.
- Confirmed close cleans child processes.
- Restart marks interrupted honestly.
- Continue sends correct session-specific context.

---

## Step-by-step implementation plan

### Task 1: Runtime discovery note

Objective:
Understand the real current Hermes/Tauri runtime before changing code.

Inspect:
- `src/agents/AgentsHermesScreen.tsx`
- `src/agents/sessionState.ts`
- `src/agents/types.ts`
- `src-tauri/**`
- current command strings/events related to Hermes send/stop.

Deliverable before coding:
- A short runtime discovery note in `.hermes/reviews/multi-panel-agent-chat/runtime-discovery.md` or included in implementation handoff.

It must answer:
- current command names;
- whether command blocks or streams;
- where process handle lives;
- whether stop is global or scoped;
- current event names/payloads;
- exact backend/frontend changes needed for four concurrent runs.

### Task 2: Add runtime types and dedicated runtime module

Files:
- Create: `src/agents/useAgentRuntime.ts`
- Modify: `src/agents/types.ts`
- Optional create: `src/agents/agentRuntimeTypes.ts`

Requirements:
- Add runtime state types.
- Add per-session state map.
- Add cap handling.
- Add queue operations.
- Add event reconciliation and stale event rejection.

### Task 3: Add deterministic mock runner/test mode

Files:
- Tauri/mock backend file depending on discovered structure.
- Frontend/Rust tests depending on existing patterns.

Mock runner must:
- delay output;
- emit numbered chunks;
- emit interleaved output across runs;
- request input;
- error intentionally;
- run long enough to test stop;
- identify runId/sessionId in every event.

### Task 4: Add backend concurrency tests

Use actual repo tooling:
- Rust/Tauri tests should be included under `npm run test:rust` when backend logic is Rust-side.
- Frontend tests use `tsx` per current `package.json`.

Required tests:
- start four mock Hermes runs;
- assert four distinct runIds/process handles or run registry entries;
- assert interleaved output retains correct sessionId/runId;
- assert stop(runId2) only terminates runId2;
- assert terminal events clean registry;
- assert stale events after stop/completion do not mutate active frontend state;
- assert backend does not globally serialize runs.

### Task 5: Implement backend concurrent run registry and event stream

Files:
- Modify under `src-tauri/**` based on discovery.

Requirements:
- Run registry keyed by runId.
- Start returns runId immediately after process/run starts.
- Incremental events emitted per run.
- Stop by runId.
- Idempotent stop.
- Cleanup on terminal states.
- Graceful stop then force kill on app close.
- No orphan processes.
- Safe command/prompt handling without shell concatenation.

### Task 6: Refactor frontend send/stop to session-scoped runtime

Files:
- Modify: `src/agents/useAgentRuntime.ts`
- Modify: `src/agents/AgentsHermesScreen.tsx`

Requirements:
- Remove global `isSending` as source of truth.
- Remove/replace global pending prompt queue.
- Use `sendPromptToSession(sessionId, prompt)`.
- Use `stopSessionRun(sessionId)`.
- Maintain per-session queue count.
- Block non-running session sends at global cap with clear message.

### Task 7: Add close/restart warning and interrupted restore

Files:
- Modify frontend close handling and Tauri close handling as needed.

Requirements:
- If any run active, warn before close.
- If user cancels close, runs continue.
- If user confirms close, mark affected sessions interrupted/needs-review and stop/kill child processes.
- On restart, clear stale currentRun handles and show interrupted state.

### Task 8: Add Continue where left off

Files:
- Create: `src/agents/continuationBrief.ts`
- Modify: `src/agents/AgentsHermesScreen.tsx`
- Add tests: `src/agents/AgentsHermesScreen.continue.test.tsx` or equivalent.

Requirements:
- Build compact session-specific continuation brief.
- Send immediately.
- Disable if running or cap reached.
- No cross-session contamination.
- Continue appears in rail, panel header, and expanded view.

### Task 9: Add dashboard test harness

Files:
- Create: `src/agents/AgentsHermesScreen.multi-panel.test.tsx`
- Optional shared harness only if it reduces duplication safely.
- Modify `package.json` test script after tests are proven.

### Task 10: Add dashboard layout state and Monitor Bar

Files:
- Create: `src/agents/dashboardLayoutState.ts`
- Modify: `src/agents/AgentsHermesScreen.tsx`
- Modify: `src/App.css`

Requirements:
- Versioned persistence.
- Auto-prioritize toggle.
- Show active agents button.
- Clear dashboard.
- Layout mode selector.
- Counts.

### Task 11: Add AgentMonitorPanel

Files:
- Create: `src/agents/AgentMonitorPanel.tsx`
- Modify: `src/agents/AgentsHermesScreen.tsx`
- Modify: `src/App.css`

Panel should be mostly presentational with props:

```ts
session
runtimeState
isPrimary
isFocused
onSend(sessionId, prompt)
onStop(sessionId)
onContinue(sessionId)
onExpand(sessionId)
onRemoveFromDashboard(sessionId)
onMakePrimary(sessionId)
```

Panel includes:
- header;
- status strip;
- compact live feed;
- compact prompt bar;
- Send / Stop / Continue / Expand / Untile actions.

### Task 12: Add drag/drop, replacement, reorder, and keyboard equivalents

Requirements:
- Rail item: Add to dashboard button/menu action.
- Panel: Move left/right, Make primary, Remove from dashboard.
- Replacement picker is keyboard navigable.
- Dragging onto running/needs-input/error/interrupted panel requires confirmation or clear “continues in background” message.
- Escape cancels drag unless a composer/modal should own Escape.

### Task 13: Add Show active agents and Auto-prioritize

Requirements:
- Deterministic active-agent ordering.
- Hidden running sessions included.
- Full replacement picker if no slots available.
- Auto-prioritize priority/debounce rules implemented.
- No reordering while user is typing.

### Task 14: Add expanded full-chat mode

Requirements:
- `expandedModeSessionId` is separate from dashboard state.
- Other runs continue in background.
- Full ChatComposer binds to expanded session and per-session runtime state.
- Back to dashboard returns without changing tiled layout.

### Task 15: CSS/responsive/sumi-e polish

Requirements:
- Preserve Zoid Agents sumi-e ink/paper/red-seal style.
- Use clear focus/attention markers not based on color alone.
- Compact live feed renders only recent messages in dashboard mode.
- Avoid high render churn from four streaming agents.
- Respect `prefers-reduced-motion`.
- Narrow layout keeps Stop visible for running panels.

---

## Validation plan

Use repo-confirmed commands:

```bash
npm run test:frontend
npm run test:rust
npm run test
npm run build
```

Targeted frontend tests to add, then wire into `npm run test:frontend`:

```bash
tsx src/agents/AgentsHermesScreen.parallel-runtime.test.tsx
tsx src/agents/AgentsHermesScreen.multi-panel.test.tsx
tsx src/agents/AgentsHermesScreen.continue.test.tsx
```

Keep existing relevant tests passing:

```bash
tsx src/agents/AgentsHermesScreen.file-manager.test.tsx
tsx src/agents/ChatComposer.behavior.test.tsx
tsx src/agents/ChatComposer.slash.test.tsx
```

Objective runtime proof required:
- Logs/events showing four distinct runIds with overlapping timestamps.
- Stop-one proof showing other three runs continue producing output after the stopped run terminates.
- Output isolation proof showing every event’s sessionId/runId appends to the correct session only.
- Stale-event proof showing late events after stop/completion are ignored.
- Cap proof showing fifth concurrent start is blocked with clear UI.
- Close proof showing active runs warn, confirmed close cleans child processes, restart marks interrupted.

Manual UX proof required:
- Tile 2, 3, and 4 sessions.
- Run 4 different prompts concurrently.
- Per-panel Send/Stop/Continue all target correct session.
- Show active agents fills panels correctly.
- Auto-prioritize moves panels only under deterministic rules.
- Rail markers show hidden running sessions.
- Remove from dashboard never stops/deletes session.
- File manager/sidebar do not clip dashboard.
- Narrow width remains usable.

---

## Risks and safeguards

- Runtime may currently be single-run/global. Mitigation: runtime discovery and backend concurrency gates before dashboard UX.
- Output mixing is the highest-severity failure. Mitigation: runId/sessionId on every event and stale-event tests.
- Stop could kill wrong process. Mitigation: scoped stop command and backend run registry tests.
- Orphan processes after app close. Mitigation: close warning plus graceful/force cleanup and test proof.
- Dashboard can hide active runs. Mitigation: rail markers, Show active agents, and replacement confirmations.
- Continuation brief can mix sessions. Mitigation: session-specific message selection and cross-session contamination tests.
- Four streaming panels can cause render churn. Mitigation: compact live feed, cap rendered messages, throttle high-volume output rendering if needed.
- Auto-prioritize can feel chaotic. Mitigation: off by default, priority rules, debounce, no reorder while typing.

---

## Feature critique gate

Before calling the feature complete:

1. Create `.hermes/reviews/multi-panel-agent-chat/handoff.md` with:
   - runtime discovery note;
   - backend contract actually implemented;
   - exact files changed;
   - test commands and real outputs;
   - objective four-run proof logs/events;
   - manual UX proof;
   - known limitations.
2. Trigger/wait for separate critique-agent review.
3. Fix every Required fix.
4. Re-run targeted tests and build.
5. Re-review until verdict is APPROVED.

---

## Definition of done

- Four overlapping agent runs are objectively verified by distinct runIds and overlapping timestamps.
- Start/stop APIs are run-scoped.
- No global run lane blocks unrelated sessions.
- No stale/cross-run output appends.
- Fifth concurrent active run is blocked clearly.
- Each panel/session has independent Send, Stop, status, queue, output, and Continue behavior.
- Running sessions hidden from dashboard remain discoverable in rail and Show active agents.
- Continue where left off sends an immediate session-specific context brief with no cross-session contamination.
- Dashboard layout persists with versioning and corrupt-state fallback.
- Closing with running agents warns; confirmed close cleans processes; restart marks interrupted honestly.
- 2/3/4-panel layouts are smooth, readable, accessible, and consistent with Zoid’s sumi-e UI.
- `npm run test`, `npm run build`, and targeted runtime/dashboard tests pass.
- Feature critique verdict is APPROVED after Required fixes are handled.
