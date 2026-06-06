import {
  cancelRunThroughBridge,
  createInitialRunControlsState,
  createRunControlsViewModel,
  resetRunControlsForTask,
  startRunThroughBridge,
  updateRunControlsDraft,
  type RunControlsInvoke,
} from "./runControls";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

async function testStartRunValidatesAndInvokesNativeBridge() {
  const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
  const invoke: RunControlsInvoke = async <T = unknown>(command: string, args?: Record<string, unknown>) => {
    calls.push({ command, args });
    return {
      session_id: "session-1",
      log_path: "/tmp/run-1.log",
      run: { id: "run-1", task_id: "task-1", status: "running", output_summary: "Process started" },
    } as T;
  };

  let state = createInitialRunControlsState({ taskId: "task-1", profileId: "codex", cwd: "/Users/example/Zoid" });
  state = updateRunControlsDraft(state, { argvText: "npm\nrun\ntest:frontend", timeoutMsText: "120000" });
  state = await startRunThroughBridge(invoke, state, { logsDir: "/Users/example/Library/Application Support/Zoid/logs" });
  const view = createRunControlsViewModel(state);

  assert(state.mode === "ready", "successful start should enter ready mode");
  assert(view.statusLabel === "Running", "started run should show clean running status");
  assert(calls[0].command === "start_agent_run_command", "start should invoke native start command");
  assert(JSON.stringify(calls[0].args).includes('"task_id":"task-1"'), "start request should include snake_case task_id");
  assert(JSON.stringify(calls[0].args).includes('"profile_id":"codex"'), "start request should include profile_id");
  assert(JSON.stringify(calls[0].args).includes('"logs_dir":"/Users/example/Library/Application Support/Zoid/logs"'), "start request should include logs_dir");
  assert(JSON.stringify(calls[0].args).includes('"argv":["npm","run","test:frontend"]'), "argv text should become argv array");
}

async function testCancelRunInvokesNativeBridgeAndCanClearStatus() {
  const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
  const invoke: RunControlsInvoke = async <T = unknown>(command: string, args?: Record<string, unknown>) => {
    calls.push({ command, args });
    return { id: "run-1", task_id: "task-1", status: "cancelled", output_summary: "Cancelled by user" } as T;
  };

  let state = createInitialRunControlsState({ taskId: "task-1", profileId: "codex", cwd: "/Users/example/Zoid" });
  state = { ...state, mode: "ready", activeRun: { id: "run-1", task_id: "task-1", status: "running" }, lastMessage: "Running" };
  state = await cancelRunThroughBridge(invoke, state, "No longer needed");
  let view = createRunControlsViewModel(state);

  assert(calls[0].command === "cancel_run_command", "cancel should invoke native cancel command");
  assert(calls[0].args?.runId === "run-1", "cancel command should use camelCase runId param");
  assert(JSON.stringify(calls[0].args).includes('"reason":"No longer needed"'), "cancel should include reason");
  assert(view.statusLabel === "Cancelled", "cancelled run should show clean status");

  state = updateRunControlsDraft(state, { clearStatus: true });
  view = createRunControlsViewModel(state);
  assert(view.statusLabel === "Idle", "clear should reset visible status");
  assert(view.errorMessage === null, "clear should reset visible error");
}

async function testBlocksUnsafeOrUnavailableStart() {
  const invoke: RunControlsInvoke = async () => {
    throw new Error("should not invoke when invalid");
  };

  let state = createInitialRunControlsState({ taskId: "task-1", profileId: "codex", cwd: "/Users/example/Zoid" });
  state = updateRunControlsDraft(state, { argvText: "", metadataJson: '{"api_key":"SECRET_TOKEN_VALUE"}' });
  state = await startRunThroughBridge(invoke, state, { logsDir: null });
  const view = createRunControlsViewModel(state);

  assert(state.mode === "blocked", "missing logs dir / invalid command should block locally");
  assert(view.errorMessage?.includes("Command arguments are required"), "empty argv should be shown as validation error");
  assert(view.errorMessage?.includes("logs directory"), "missing logs dir should be explicit");
  assert(view.errorMessage?.includes("secret-looking"), "secret-like metadata should be rejected locally");
}

function testTaskSwitchClearsStaleActiveRun() {
  const state = createInitialRunControlsState({ taskId: "task-a", profileId: "codex", cwd: "/Users/example/Zoid" });
  const running = {
    ...state,
    mode: "ready" as const,
    activeRun: { id: "run-a", task_id: "task-a", status: "running" },
    lastMessage: "Running task A",
  };

  const switched = resetRunControlsForTask(running, "task-b", "/Users/example/Zoid");
  const view = createRunControlsViewModel(switched);

  assert(switched.draft.taskId === "task-b", "task switch should update selected task id");
  assert(switched.activeRun === null, "task switch should clear stale active run");
  assert(view.canCancel === false, "task switch must not leave cancel enabled for previous task run");
  assert(view.statusLabel === "Idle", "task switch should clear previous status label");
}

await testStartRunValidatesAndInvokesNativeBridge();
await testCancelRunInvokesNativeBridgeAndCanClearStatus();
await testBlocksUnsafeOrUnavailableStart();
testTaskSwitchClearsStaleActiveRun();

console.log("runControls tests passed");
