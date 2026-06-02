import {
  createTaskLinkedPanelsViewModel,
  loadTaskLinkedPanelsFromBridge,
  type TaskLinkedPanelsInvoke,
  type TaskLinkedPanelsState,
} from "./taskLinkedPanels";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function assertEqual<T>(actual: T, expected: T, message: string) {
  if (actual !== expected) throw new Error(`${message}: expected ${String(expected)}, got ${String(actual)}`);
}

const historyRecord = {
  event: {
    id: "event-1",
    action_type: "task_created",
    outcome: "success",
    timestamp: "2026-06-02T10:00:00Z",
    summary: "Task created without raw logs",
    source: "task_service",
    targets: [{ entity_type: "task", entity_id: "task-1", relation_type: "primary" }],
  },
  matched_entities: [{ entity_type: "task", entity_id: "task-1", relation_type: "primary" }],
};

function makeInvoke() {
  const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
  const invoke: TaskLinkedPanelsInvoke = async <T = unknown>(command: string, args?: Record<string, unknown>) => {
    calls.push({ command, args });
    if (command === "list_entity_history_command") {
      return [
        historyRecord,
        {
          event: {
            id: "event-run",
            action_type: "run_completed",
            outcome: "success",
            timestamp: "2026-06-02T11:00:00Z",
            summary: "Run run-1 completed",
            source: "agent_execution",
            targets: [
              { entity_type: "task", entity_id: "task-1", relation_type: "task" },
              { entity_type: "run", entity_id: "run-1", relation_type: "run" },
            ],
          },
          matched_entities: [
            { entity_type: "task", entity_id: "task-1", relation_type: "task" },
            { entity_type: "run", entity_id: "run-1", relation_type: "run" },
          ],
        },
        {
          event: {
            id: "event-review",
            action_type: "review_approved",
            outcome: "approved",
            timestamp: "2026-06-02T12:00:00Z",
            summary: "Review review-1 approved",
            source: "review_service",
            targets: [
              { entity_type: "task", entity_id: "task-1", relation_type: "task" },
              { entity_type: "review", entity_id: "review-1", relation_type: "review" },
            ],
          },
          matched_entities: [
            { entity_type: "task", entity_id: "task-1", relation_type: "task" },
            { entity_type: "review", entity_id: "review-1", relation_type: "review" },
          ],
        },
      ] as T;
    }
    if (command === "read_run_status_command") {
      return { id: (args?.runId ?? "run-1") as string, status: "completed", exit_code: 0, started_at: "2026-06-02T10:30:00Z", completed_at: "2026-06-02T11:00:00Z" } as T;
    }
    if (command === "read_review_record_command") {
      return { id: (args?.reviewRecordId ?? "review-1") as string, verdict: "approved", subject_type: "task", subject_id: "task-1", created_at: "2026-06-02T11:30:00Z" } as T;
    }
    throw new Error(`unexpected command ${command}`);
  };
  return { invoke, calls };
}

{
  const { invoke, calls } = makeInvoke();
  const state = await loadTaskLinkedPanelsFromBridge(invoke, "task-1");
  assertEqual(state.mode, "ready", "linked panels should load");
  assertEqual(calls[0]?.command, "list_entity_history_command", "must query entity history for task");
  assertEqual((calls[0]?.args?.request as Record<string, unknown>)?.entity_id, "task-1", "history request uses task id");
  assert(calls.some((call) => call.command === "read_run_status_command" && call.args?.runId === "run-1"), "must hydrate linked run status by camelCase runId");
  assert(calls.some((call) => call.command === "read_review_record_command" && call.args?.reviewRecordId === "review-1"), "must hydrate linked review by camelCase reviewRecordId");
  assert(state.mode === "ready" && state.runs[0]?.id === "run-1", "ready state includes linked run");
  assert(state.mode === "ready" && state.reviews[0]?.id === "review-1", "ready state includes linked review");
  assert(state.mode === "ready" && state.history.length === 3, "ready state includes history records");
}

{
  const view = createTaskLinkedPanelsViewModel({
    mode: "ready",
    taskId: "task-1",
    runs: [{ id: "run-1", status: "completed", exit_code: 0, started_at: "2026-06-02T10:30:00Z", completed_at: "2026-06-02T11:00:00Z" }],
    reviews: [{ id: "review-1", verdict: "approved", subject_type: "task", subject_id: "task-1", created_at: "2026-06-02T11:30:00Z" }],
    history: [historyRecord],
  } satisfies TaskLinkedPanelsState);
  assertEqual(view.runPanel.title, "Linked runs", "run panel title should be stable");
  assert(view.runPanel.items[0]?.summary.includes("completed"), "run panel should summarize real run status");
  assert(view.reviewPanel.items[0]?.summary.includes("approved"), "review panel should summarize real review verdict");
  assert(view.historyPanel.entries[0]?.summary.includes("Task created"), "history panel should render real history summary");
}

{
  const view = createTaskLinkedPanelsViewModel({ mode: "error", taskId: "task-1", error: "token=raw-secret-value" });
  assert(!view.errorMessage?.includes("raw-secret-value"), "error copy must redact secret-like values");
}

console.log("taskLinkedPanels tests passed");
