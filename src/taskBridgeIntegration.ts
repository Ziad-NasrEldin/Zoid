import {
  buildCreateTaskPayload,
  buildUpdateTaskPayload,
  createInitialTaskForm,
  taskBridgeCommands,
  type TaskFormDraft,
  type TaskRecord,
  type TaskValidationErrors,
  type ValidatedTaskInput,
} from "./taskViewModel";

export type TaskBridgeInvoke = <T = unknown>(command: string, args?: Record<string, unknown>) => Promise<T>;

export type TaskBridgeState =
  | { mode: "loading"; selectedTaskId: string | null }
  | { mode: "error"; selectedTaskId: string | null; error: string }
  | { mode: "ready"; selectedTaskId: string | null; tasks: TaskRecord[] };

export type TaskBridgeUiState = {
  form: TaskFormDraft;
  formErrors: TaskValidationErrors;
  state: TaskBridgeState;
};

export function createInitialTaskBridgeState(workspaceKey = "tasks"): TaskBridgeUiState {
  return {
    form: createInitialTaskForm(workspaceKey),
    formErrors: {},
    state: { mode: "loading", selectedTaskId: null },
  };
}

function bridgeError(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "unknown native task bridge error";
}

function selectVisibleTask(tasks: TaskRecord[], requestedTaskId: string | null): string | null {
  if (requestedTaskId && tasks.some((task) => task.id === requestedTaskId)) return requestedTaskId;
  return null;
}

export function formDraftForTask(task: TaskRecord): TaskFormDraft {
  return {
    title: task.title,
    detail: task.detail ?? "",
    status: task.status,
    priority: task.priority,
    workspace_key: task.workspace_key,
    metadata_json: task.metadata_json || "{}",
  };
}

function createRequest(value: ValidatedTaskInput) {
  return {
    title: value.title,
    detail: value.detail,
    priority: value.priority,
    workspace_key: value.workspace_key,
    metadata_json: value.metadata_json,
  };
}

function updateRequest(value: ValidatedTaskInput) {
  return {
    title: value.title,
    detail: value.detail,
    priority: value.priority,
    workspace_key: value.workspace_key,
    metadata_json: value.metadata_json,
  };
}

export async function refreshTasksFromBridge(
  invoke: TaskBridgeInvoke,
  options: { selectedTaskId: string | null },
): Promise<TaskBridgeState> {
  try {
    const tasks = await invoke<TaskRecord[]>(taskBridgeCommands.list);
    return {
      mode: "ready",
      selectedTaskId: selectVisibleTask(tasks, options.selectedTaskId),
      tasks,
    };
  } catch (error) {
    return {
      mode: "error",
      selectedTaskId: options.selectedTaskId,
      error: bridgeError(error),
    };
  }
}

export async function selectTaskThroughBridge(invoke: TaskBridgeInvoke, taskId: string): Promise<TaskBridgeState> {
  try {
    const task = await invoke<TaskRecord>(taskBridgeCommands.detail, { taskId });
    const refreshed = await refreshTasksFromBridge(invoke, { selectedTaskId: task.id });
    if (refreshed.mode !== "ready" || refreshed.tasks.some((candidate) => candidate.id === task.id)) return refreshed;

    return {
      mode: "ready",
      selectedTaskId: task.id,
      tasks: [task, ...refreshed.tasks],
    };
  } catch (error) {
    return {
      mode: "error",
      selectedTaskId: taskId,
      error: bridgeError(error),
    };
  }
}

export async function createTaskThroughBridge(invoke: TaskBridgeInvoke, form: TaskFormDraft): Promise<TaskBridgeUiState> {
  const validation = buildCreateTaskPayload(form);
  if (!validation.ok) {
    return {
      form,
      formErrors: validation.errors,
      state: { mode: "error", selectedTaskId: null, error: "Task form has validation errors. No native command was called." },
    };
  }

  try {
    const created = await invoke<TaskRecord>(taskBridgeCommands.create, { request: createRequest(validation.value) });
    return {
      form: createInitialTaskForm(validation.value.workspace_key),
      formErrors: {},
      state: await refreshTasksFromBridge(invoke, { selectedTaskId: created.id }),
    };
  } catch (error) {
    return {
      form,
      formErrors: {},
      state: { mode: "error", selectedTaskId: null, error: bridgeError(error) },
    };
  }
}

export async function updateTaskThroughBridge(
  invoke: TaskBridgeInvoke,
  taskId: string,
  form: TaskFormDraft,
): Promise<TaskBridgeUiState> {
  const validation = buildUpdateTaskPayload(form);
  if (!validation.ok) {
    return {
      form,
      formErrors: validation.errors,
      state: { mode: "error", selectedTaskId: taskId, error: "Task form has validation errors. No native command was called." },
    };
  }

  try {
    const updated = await invoke<TaskRecord>(taskBridgeCommands.update, { taskId, request: updateRequest(validation.value) });
    return {
      form: createInitialTaskForm(validation.value.workspace_key),
      formErrors: {},
      state: await refreshTasksFromBridge(invoke, { selectedTaskId: updated.id }),
    };
  } catch (error) {
    return {
      form,
      formErrors: {},
      state: { mode: "error", selectedTaskId: taskId, error: bridgeError(error) },
    };
  }
}
