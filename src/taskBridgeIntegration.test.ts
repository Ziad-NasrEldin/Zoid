import {
  createInitialTaskBridgeState,
  createTaskThroughBridge,
  formDraftForTask,
  performTaskActionThroughBridge,
  refreshTasksFromBridge,
  selectTaskThroughBridge,
  updateTaskThroughBridge,
  type TaskBridgeInvoke,
} from "./taskBridgeIntegration";
import type { TaskFormDraft, TaskRecord } from "./taskViewModel";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function assertEqual<T>(actual: T, expected: T, message: string) {
  if (actual !== expected) throw new Error(`${message}: expected ${String(expected)}, got ${String(actual)}`);
}

const baseTask: TaskRecord = {
  id: "task-1",
  title: "Persist real task UI",
  detail: "Wire the task workspace to the native bridge.",
  status: "active",
  priority: "high",
  workspace_key: "tasks",
  created_at: "2026-06-02T12:00:00Z",
  updated_at: "2026-06-02T12:00:00Z",
  archived_at: null,
  deleted_at: null,
  metadata_json: '{"ticket":"P2.21"}',
};

function makeInvoke(responses: Record<string, unknown[]>) {
  const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
  const invoke: TaskBridgeInvoke = async <T = unknown>(command: string, args?: Record<string, unknown>) => {
    calls.push({ command, args });
    const queue = responses[command] ?? [];
    if (queue.length === 0) throw new Error(`unexpected command ${command}`);
    const response = queue.shift();
    if (response instanceof Error) throw response;
    return response as T;
  };
  return { invoke, calls };
}

{
  const { invoke, calls } = makeInvoke({ list_tasks_command: [[baseTask]] });
  const state = await refreshTasksFromBridge(invoke, { selectedTaskId: null });
  assertEqual(state.mode, "ready", "refresh should return ready state");
  assert(state.mode === "ready" && state.tasks[0]?.id === "task-1", "refresh should use persisted tasks from bridge");
  assertEqual(state.selectedTaskId, null, "refresh should keep create mode when nothing is selected");
  assertEqual(calls[0]?.command, "list_tasks_command", "refresh uses list_tasks_command");
}

{
  const { invoke } = makeInvoke({ list_tasks_command: [[]] });
  const state = await refreshTasksFromBridge(invoke, { selectedTaskId: "missing" });
  assertEqual(state.mode, "ready", "empty refresh should still be ready");
  assert(state.mode === "ready" && state.tasks.length === 0, "empty refresh should not invent tasks");
  assertEqual(state.selectedTaskId, null, "missing selection should clear when list is empty");
}

{
  const { invoke } = makeInvoke({ list_tasks_command: [new Error("native bridge unavailable")] });
  const state = await refreshTasksFromBridge(invoke, { selectedTaskId: "task-1" });
  assertEqual(state.mode, "error", "bridge failure should produce explicit error state");
  assert(state.mode === "error" && state.error.includes("native bridge unavailable"), "bridge error should be surfaced");
}

{
  const created: TaskRecord = { ...baseTask, id: "task-created", title: "Created from UI", status: "inbox" };
  const { invoke, calls } = makeInvoke({ create_task_command: [created], list_tasks_command: [[created]] });
  const draft: TaskFormDraft = {
    title: " Created from UI ",
    detail: " Real persistence ",
    status: "inbox",
    priority: "normal",
    workspace_key: "tasks",
    metadata_json: '{"source":"ui"}',
  };
  const result = await createTaskThroughBridge(invoke, draft);
  assertEqual(result.form.title, "", "successful create resets the form title");
  assertEqual(result.state.selectedTaskId, "task-created", "successful create selects created task");
  assertEqual(calls[0]?.command, "create_task_command", "create uses native create command");
  assertEqual((calls[0]?.args?.request as Record<string, unknown>).title, "Created from UI", "create payload is validated/trimmed");
  assertEqual(calls[1]?.command, "list_tasks_command", "create refreshes persisted task list after bridge write");
}

{
  const { invoke, calls } = makeInvoke({ update_task_command: [{ ...baseTask, title: "Updated task" }], list_tasks_command: [[{ ...baseTask, title: "Updated task" }]] });
  const result = await updateTaskThroughBridge(invoke, "task-1", {
    title: "Updated task",
    detail: "Updated detail",
    status: "active",
    priority: "urgent",
    workspace_key: "tasks",
    metadata_json: "{}",
  });
  assertEqual(result.state.selectedTaskId, "task-1", "update preserves selected task");
  assertEqual(calls[0]?.command, "update_task_command", "update uses native update command");
  assertEqual(calls[0]?.args?.taskId, "task-1", "update passes taskId camel-case arg expected by Tauri invoke");
  assertEqual((calls[0]?.args?.request as Record<string, unknown>).priority, "urgent", "update payload includes validated priority");
}

{
  const { invoke, calls } = makeInvoke({ read_task_command: [{ ...baseTask, title: "Read detail" }], list_tasks_command: [[{ ...baseTask, title: "Read detail" }]] });
  const state = await selectTaskThroughBridge(invoke, "task-1");
  assertEqual(state.selectedTaskId, "task-1", "select should choose requested task");
  assert(state.mode === "ready" && state.tasks[0]?.title === "Read detail", "select should include read detail record");
  assertEqual(calls[0]?.command, "read_task_command", "select uses read_task_command before refreshing list");
  assertEqual(calls[0]?.args?.taskId, "task-1", "select passes taskId camel-case arg expected by Tauri invoke");
}

{
  const draft = formDraftForTask(baseTask);
  assertEqual(draft.title, baseTask.title, "selected task title should hydrate edit form");
  assertEqual(draft.detail, baseTask.detail, "selected task detail should hydrate edit form");
  assertEqual(draft.status, baseTask.status, "selected task status should be visible as read-only form context");
  assertEqual(draft.priority, baseTask.priority, "selected task priority should hydrate edit form");
  assertEqual(draft.workspace_key, baseTask.workspace_key, "selected task workspace should hydrate edit form");
  assertEqual(draft.metadata_json, baseTask.metadata_json, "selected task metadata should hydrate edit form");
}

{
  const { invoke, calls } = makeInvoke({ update_task_status_command: [{ ...baseTask, status: "completed" }], list_tasks_command: [[{ ...baseTask, status: "completed" }]] });
  const result = await performTaskActionThroughBridge(invoke, createInitialTaskBridgeState("tasks"), "task-1", { kind: "status", status: "completed" });
  assertEqual(result.state.selectedTaskId, "task-1", "status action preserves selected task");
  assertEqual(calls[0]?.command, "update_task_status_command", "status action uses native status command");
  assertEqual(calls[0]?.args?.taskId, "task-1", "status action passes taskId camel-case arg expected by Tauri invoke");
  assertEqual((calls[0]?.args?.request as Record<string, unknown>).status, "completed", "status action passes native status request");
  assertEqual(calls[1]?.command, "list_tasks_command", "status action refreshes persisted tasks");
}

{
  const { invoke, calls } = makeInvoke({ archive_task_command: [{ ...baseTask, status: "archived" }], list_tasks_command: [[{ ...baseTask, status: "archived" }]] });
  const result = await performTaskActionThroughBridge(invoke, createInitialTaskBridgeState("tasks"), "task-1", { kind: "archive" });
  assertEqual(result.state.selectedTaskId, "task-1", "archive keeps archived task selected when still returned by backend");
  assertEqual(calls[0]?.command, "archive_task_command", "archive action uses native archive command");
  assertEqual(calls[0]?.args?.taskId, "task-1", "archive action passes taskId camel-case arg expected by Tauri invoke");
}

{
  const { invoke, calls } = makeInvoke({ delete_task_command: [{ ...baseTask, status: "deleted" }], list_tasks_command: [[]] });
  const result = await performTaskActionThroughBridge(invoke, createInitialTaskBridgeState("tasks"), "task-1", { kind: "delete" });
  assertEqual(result.state.selectedTaskId, null, "delete clears selected task after backend delete");
  assertEqual(calls[0]?.command, "delete_task_command", "delete action uses native delete command");
  assertEqual(calls[0]?.args?.taskId, "task-1", "delete action passes taskId camel-case arg expected by Tauri invoke");
}

{
  const { invoke, calls } = makeInvoke({ create_task_command: [baseTask] });
  const result = await createTaskThroughBridge(invoke, { ...createInitialTaskBridgeState("tasks").form, title: " ", workspace_key: "../bad" });
  assertEqual(result.state.mode, "error", "invalid create should produce local validation error state");
  assertEqual(calls.length, 0, "invalid create must not call the native bridge");
}

console.log("taskBridgeIntegration tests passed");
