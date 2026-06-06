import {
  appendCleanSessionChunk,
  createCleanSessionViewModel,
  loadCleanSessionStreamFromBridge,
  nextCleanSessionOffset,
  type CleanSessionInvoke,
} from "./cleanSession";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

async function testBuildsCleanCardsFromRunStatusAndStreamChunks() {
  const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
  const invoke: CleanSessionInvoke = async <T = unknown>(command: string, args?: Record<string, unknown>) => {
    calls.push({ command, args });
    if (command === "read_run_status_command") {
      return {
        id: "run-1",
        status: "running",
        output_summary: "Installing dependencies",
        error_summary: null,
        updated_at: "2026-06-02T12:00:00Z",
      } as T;
    }
    if (command === "stream_run_output_command") {
      return {
        run_id: "run-1",
        log_reference_id: "log-1",
        offset: 0,
        next_offset: 84,
        eof: false,
        status: "running",
        content: "npm install\nadded 12 packages\nAPI_KEY=SECRET_TOKEN_VALUE\n✓ build assets ready",
      } as T;
    }
    throw new Error(`unexpected command ${command}`);
  };

  const state = await loadCleanSessionStreamFromBridge(invoke, {
    runId: "run-1",
    logsDir: "/safe/logs",
    offset: 0,
    maxBytes: 4096,
  });
  const view = createCleanSessionViewModel(state);

  assert(state.mode === "ready", "stream should load into ready state");
  assert(calls[0].command === "read_run_status_command", "should read persisted run status first");
  assert(calls[0].args?.runId === "run-1", "status command should use runId arg");
  assert(calls[1].command === "stream_run_output_command", "should invoke stream command");
  assert(JSON.stringify(calls[1].args).includes('"logs_dir":"/safe/logs"'), "stream request should include snake_case logs_dir");
  assert(view.statusLabel === "Running", "view should expose clean status label");
  assert(view.cards.length >= 3, "stream content should become clean cards, not one terminal blob");
  assert(view.cards.some((card) => card.kind === "command" && card.title.includes("npm install")), "command line should become a command card");
  assert(view.cards.some((card) => card.kind === "success" && card.body.includes("build assets ready")), "success line should become a success card");
  assert(!view.cards.map((card) => card.body).join("\n").includes("SECRET_TOKEN_VALUE"), "secret-like stream text must be redacted");
  assert(!("rawTerminalText" in view), "view model must not expose raw terminal-first text");
}

function testReportsUnavailableWhenLogsDirMissing() {
  const state = createCleanSessionViewModel({ mode: "unavailable", runId: "run-1", reason: "logs directory is not available to the frontend" });
  assert(state.statusLabel === "Unavailable", "missing logs dir should be explicit");
  assert(state.cards.length === 0, "unavailable state should not fabricate output cards");
  assert(state.emptyCopy.includes("No terminal output is simulated"), "empty copy should state no simulated output");
}

function testSummarizesTerminalStatusesAsProductStates() {
  const view = createCleanSessionViewModel({
    mode: "ready",
    runId: "run-2",
    run: { id: "run-2", status: "failed", error_summary: "Command failed" },
    chunk: {
      run_id: "run-2",
      log_reference_id: "log-2",
      offset: 0,
      next_offset: 20,
      eof: true,
      status: "failed",
      content: "Error: command failed\nretry with safer input",
    },
  });
  assert(view.statusLabel === "Failed", "failed run should be clean status, not terminal jargon");
  assert(view.cards.some((card) => card.kind === "error"), "error output should become error card");
  assert(view.cards.some((card) => card.body.includes("retry with safer input")), "plain guidance should be preserved as clean detail");
}

function testAdvancesAndAppendsStreamChunks() {
  const first = {
    mode: "ready" as const,
    runId: "run-3",
    run: { id: "run-3", status: "running" },
    chunk: {
      run_id: "run-3",
      log_reference_id: "log-3",
      offset: 0,
      next_offset: 20,
      eof: false,
      status: "running",
      content: "npm run build",
    },
  };
  const second = {
    mode: "ready" as const,
    runId: "run-3",
    run: { id: "run-3", status: "completed" },
    chunk: {
      run_id: "run-3",
      log_reference_id: "log-3",
      offset: 20,
      next_offset: 44,
      eof: true,
      status: "completed",
      content: "✓ compiled cleanly",
    },
  };

  assert(nextCleanSessionOffset(first) === 20, "next refresh should use the previous next_offset");
  const appended = appendCleanSessionChunk(first, second);
  const view = createCleanSessionViewModel(appended);
  assert(appended.mode === "ready" && appended.chunk.next_offset === 44, "appended state should preserve newest next offset");
  assert(view.cards.some((card) => card.title.includes("npm run build")), "previous chunk cards should remain visible");
  assert(view.cards.some((card) => card.body.includes("compiled cleanly")), "new chunk cards should append to the clean stream");
}

await testBuildsCleanCardsFromRunStatusAndStreamChunks();
testReportsUnavailableWhenLogsDirMissing();
testSummarizesTerminalStatusesAsProductStates();
testAdvancesAndAppendsStreamChunks();

console.log("cleanSession tests passed");
