import {
  buildTaskWorkspaceView,
  createInitialTaskForm,
  isSafeWorkspaceKey,
  taskBridgeCommands,
  validateTaskForm,
  type TaskRecord,
  type TaskWorkspaceState,
} from "./taskViewModel";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

const taskA: TaskRecord = {
  id: "task-a",
  title: "Ship task workspace",
  detail: "Build isolated view-model and component",
  status: "active",
  priority: "urgent",
  workspace_key: "zoid-main",
  created_at: "2026-06-02T10:00:00Z",
  updated_at: "2026-06-02T11:00:00Z",
  archived_at: null,
  deleted_at: null,
  metadata_json: '{"source":"manual"}',
};

const taskB: TaskRecord = {
  ...taskA,
  id: "task-b",
  title: "Older planned task",
  status: "planned",
  priority: "normal",
  updated_at: "2026-06-02T09:00:00Z",
};

assert(taskBridgeCommands.create === "create_task_command", "create bridge command name must match backend command");
assert(taskBridgeCommands.list === "list_tasks_command", "list bridge command name must match backend command");
assert(taskBridgeCommands.detail === "read_task_command", "detail bridge command name must match backend command");
assert(taskBridgeCommands.update === "update_task_command", "update bridge command name must match backend command");

const emptyForm = createInitialTaskForm("zoid-main");
assert(emptyForm.workspace_key === "zoid-main", "initial form should preserve provided workspace key");
assert(emptyForm.status === "inbox" && emptyForm.priority === "normal", "initial form should use safe defaults");

const valid = validateTaskForm({
  title: "  Build P2.21  ",
  detail: "  Useful detail  ",
  status: "active",
  priority: "high",
  workspace_key: "zoid-main",
  metadata_json: '{"ticket":"P2.21","count":2}',
});
assert(valid.ok, "valid form should pass validation");
assert(valid.value.title === "Build P2.21", "title should be trimmed");
assert(valid.value.detail === "Useful detail", "detail should be trimmed");
assert(valid.value.metadata_json === '{"ticket":"P2.21","count":2}', "valid metadata JSON should be preserved");

const invalid = validateTaskForm({
  title: " ",
  status: "made-up",
  priority: "panic",
  workspace_key: "../unsafe key",
  metadata_json: '{"apiToken":"secret-value"}',
});
assert(!invalid.ok, "invalid form should fail validation");
assert(invalid.errors.title?.includes("required"), "title is required");
assert(invalid.errors.status?.includes("Unsupported"), "status enum is validated");
assert(invalid.errors.priority?.includes("Unsupported"), "priority enum is validated");
assert(invalid.errors.workspace_key?.includes("safe"), "workspace key safety is validated");
assert(invalid.errors.metadata_json?.includes("secret-looking"), "secret-looking metadata is rejected");

const invalidJson = validateTaskForm({
  title: "Task",
  status: "inbox",
  priority: "normal",
  workspace_key: "zoid-main",
  metadata_json: "{not json}",
});
assert(!invalidJson.ok && invalidJson.errors.metadata_json?.includes("valid JSON"), "metadata must be valid JSON");

assert(isSafeWorkspaceKey("project_123.alpha-beta"), "safe workspace keys allow slug punctuation");
assert(!isSafeWorkspaceKey(""), "workspace key is required");
assert(!isSafeWorkspaceKey("has space"), "workspace key cannot contain spaces");
assert(!isSafeWorkspaceKey("../escape"), "workspace key cannot contain path traversal");

const loadingView = buildTaskWorkspaceView({ mode: "loading", selectedTaskId: null });
assert(loadingView.list.statusLabel === "Loading tasks", "loading state should be truthful");
assert(loadingView.list.items.length === 0, "loading state must not fabricate tasks");

const errorView = buildTaskWorkspaceView({ mode: "error", selectedTaskId: null, error: "Native bridge failed" });
assert(errorView.list.statusLabel === "Task data unavailable", "error state should be explicit");
assert(errorView.list.emptyCopy.includes("Native bridge failed"), "error view should surface the real bridge error");

const emptyView = buildTaskWorkspaceView({ mode: "ready", selectedTaskId: null, tasks: [] });
assert(emptyView.list.statusLabel === "No tasks", "empty list should be labeled empty");
assert(emptyView.list.items.length === 0 && emptyView.detail.kind === "empty", "empty list should not produce fake detail");

const state: TaskWorkspaceState = { mode: "ready", selectedTaskId: "task-a", tasks: [taskB, taskA] };
const populated = buildTaskWorkspaceView(state);
assert(populated.list.statusLabel === "2 tasks", "ready list should show real task count");
assert(populated.list.items[0]?.id === "task-a", "tasks should be newest first");
assert(populated.list.items[0]?.isSelected === true, "selected task should be marked in list");
assert(populated.detail.kind === "task" && populated.detail.task.id === "task-a", "selected detail should use real task");
assert(populated.detail.metadataPreview === '{\n  "source": "manual"\n}', "detail should format valid metadata JSON");

const missingSelection = buildTaskWorkspaceView({ mode: "ready", selectedTaskId: "missing", tasks: [taskA] });
assert(missingSelection.detail.kind === "missing", "missing selected task should be represented truthfully");

console.log("taskViewModel tests passed");
