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
  const message = error instanceof Error ? error.message : typeof error === "string" ? error : "";
  if (/invoke|__TAURI|Tauri/i.test(message)) {
    return "Native task backend is only available inside the Tauri desktop app. Browser preview keeps task data unavailable instead of simulating records.";
  }
  if (message) return message;
  return "unknown native task bridge error";
}

function selectVisibleTask(tasks: TaskRecord[], requestedTaskId: string | null): string | null {
  if (requestedTaskId && tasks.some((task) => task.id === requestedTaskId)) return requestedTaskId;
  return tasks[0]?.id ?? null;
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

export function applyBridgeStateToTaskUi(current: TaskBridgeUiState, state: TaskBridgeState): TaskBridgeUiState {
  const selectedTask = state.mode === "ready" && state.selectedTaskId
    ? state.tasks.find((task) => task.id === state.selectedTaskId) ?? null
    : null;

  return {
    ...current,
    form: selectedTask ? formDraftForTask(selectedTask) : current.form,
    formErrors: selectedTask ? {} : current.formErrors,
    state,
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

export async function performTaskActionThroughBridge(
  invoke: TaskBridgeInvoke,
  current: TaskBridgeUiState,
  taskId: string,
  action: { kind: "status"; status: string } | { kind: "archive" } | { kind: "delete" },
): Promise<TaskBridgeUiState> {
  try {
    const selectedTaskId = action.kind === "delete" ? null : taskId;
    if (action.kind === "status") {
      await invoke<TaskRecord>(taskBridgeCommands.updateStatus, { taskId, request: { status: action.status } });
    } else if (action.kind === "archive") {
      await invoke<TaskRecord>(taskBridgeCommands.archive, { taskId });
    } else {
      await invoke<TaskRecord>(taskBridgeCommands.delete, { taskId });
    }

    return {
      ...current,
      formErrors: {},
      state: await refreshTasksFromBridge(invoke, { selectedTaskId }),
    };
  } catch (error) {
    return {
      ...current,
      formErrors: {},
      state: { mode: "error", selectedTaskId: taskId, error: bridgeError(error) },
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
