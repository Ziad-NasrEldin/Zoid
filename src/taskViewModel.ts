export const taskStatuses = [
  "inbox",
  "planned",
  "active",
  "waiting",
  "review_required",
  "blocked",
  "completed",
  "failed",
  "cancelled",
  "archived",
  "deleted",
] as const;

export const taskPriorities = ["low", "normal", "high", "urgent"] as const;

export type TaskStatus = (typeof taskStatuses)[number];
export type TaskPriority = (typeof taskPriorities)[number];

export type TaskRecord = {
  id: string;
  title: string;
  detail?: string | null;
  status: TaskStatus;
  priority: TaskPriority;
  workspace_key: string;
  created_at: string;
  updated_at: string;
  archived_at?: string | null;
  deleted_at?: string | null;
  metadata_json: string;
};

export type TaskFormDraft = {
  title: string;
  detail?: string | null;
  status: string;
  priority: string;
  workspace_key: string;
  metadata_json?: string | null;
};

export type ValidatedTaskInput = {
  title: string;
  detail: string | null;
  status: TaskStatus;
  priority: TaskPriority;
  workspace_key: string;
  metadata_json: string;
};

export type TaskValidationErrors = Partial<Record<keyof TaskFormDraft, string>>;

export type TaskValidationResult =
  | { ok: true; value: ValidatedTaskInput; errors: TaskValidationErrors }
  | { ok: false; errors: TaskValidationErrors };

export type TaskWorkspaceState =
  | { mode: "loading"; selectedTaskId: string | null }
  | { mode: "error"; selectedTaskId: string | null; error: string }
  | { mode: "ready"; selectedTaskId: string | null; tasks: TaskRecord[] };

export type TaskListItemView = {
  id: string;
  title: string;
  meta: string;
  tone: "ready" | "pending" | "blocked";
  isSelected: boolean;
};

export type TaskListView = {
  statusLabel: string;
  copy: string;
  emptyCopy: string;
  items: TaskListItemView[];
};

export type TaskDetailView =
  | { kind: "loading"; copy: string }
  | { kind: "empty"; copy: string }
  | { kind: "missing"; copy: string }
  | { kind: "error"; copy: string }
  | { kind: "task"; task: TaskRecord; metadataPreview: string; detailLines: string[] };

export type TaskWorkspaceView = {
  list: TaskListView;
  detail: TaskDetailView;
};

export const taskBridgeCommands = {
  create: "create_task_command",
  list: "list_tasks_command",
  detail: "read_task_command",
  update: "update_task_command",
  updateStatus: "update_task_status_command",
  archive: "archive_task_command",
  delete: "delete_task_command",
} as const;

const taskStatusLabels: Record<TaskStatus, string> = {
  inbox: "Inbox",
  planned: "Planned",
  active: "Active",
  waiting: "Waiting",
  review_required: "Review required",
  blocked: "Blocked",
  completed: "Completed",
  failed: "Failed",
  cancelled: "Cancelled",
  archived: "Archived",
  deleted: "Deleted",
};

const blockedStatuses = new Set<TaskStatus>(["blocked", "failed", "review_required"]);
const readyStatuses = new Set<TaskStatus>(["active", "completed"]);
const secretKeyPattern = /(?:secret|token|api[_-]?key|password|passwd|credential|private[_-]?key|authorization|bearer)/i;
const workspaceKeyPattern = /^[A-Za-z0-9][A-Za-z0-9._-]{0,127}$/;

function isTaskStatus(value: string): value is TaskStatus {
  return taskStatuses.includes(value as TaskStatus);
}

function isTaskPriority(value: string): value is TaskPriority {
  return taskPriorities.includes(value as TaskPriority);
}

function countLabel(count: number, singular: string, plural = `${singular}s`) {
  return `${count} ${count === 1 ? singular : plural}`;
}

function priorityLabel(priority: TaskPriority) {
  return priority === "normal" ? "normal priority" : `${priority} priority`;
}

function taskTone(task: TaskRecord): TaskListItemView["tone"] {
  if (blockedStatuses.has(task.status)) return "blocked";
  if (readyStatuses.has(task.status)) return "ready";
  return "pending";
}

function newestFirst(tasks: TaskRecord[]) {
  return [...tasks].sort((a, b) => b.updated_at.localeCompare(a.updated_at) || b.created_at.localeCompare(a.created_at) || a.id.localeCompare(b.id));
}

function hasSecretLookingMetadata(value: unknown): boolean {
  if (Array.isArray(value)) return value.some(hasSecretLookingMetadata);
  if (value && typeof value === "object") {
    return Object.entries(value).some(([key, nestedValue]) => secretKeyPattern.test(key) || hasSecretLookingMetadata(nestedValue));
  }
  return typeof value === "string" && secretKeyPattern.test(value);
}

export function isSafeWorkspaceKey(workspaceKey: string): boolean {
  const trimmed = workspaceKey.trim();
  return workspaceKeyPattern.test(trimmed) && !trimmed.includes("..") && !trimmed.includes("/") && !trimmed.includes("\\");
}

export function createInitialTaskForm(workspaceKey = ""): TaskFormDraft {
  return {
    title: "",
    detail: "",
    status: "inbox",
    priority: "normal",
    workspace_key: workspaceKey,
    metadata_json: "{}",
  };
}

export function validateTaskForm(draft: TaskFormDraft): TaskValidationResult {
  const errors: TaskValidationErrors = {};
  const title = draft.title.trim();
  const detail = draft.detail?.trim() || null;
  const status = draft.status.trim();
  const priority = draft.priority.trim();
  const workspaceKey = draft.workspace_key.trim();
  const metadataJson = draft.metadata_json?.trim() || "{}";

  if (!title) errors.title = "Task title is required.";
  if (!isTaskStatus(status)) errors.status = `Unsupported task status. Use one of: ${taskStatuses.join(", ")}.`;
  if (!isTaskPriority(priority)) errors.priority = `Unsupported task priority. Use one of: ${taskPriorities.join(", ")}.`;
  if (!workspaceKey) {
    errors.workspace_key = "Workspace key is required.";
  } else if (!isSafeWorkspaceKey(workspaceKey)) {
    errors.workspace_key = "Workspace key must be a safe slug using letters, numbers, dots, underscores, or hyphens.";
  }

  let parsedMetadata: unknown = {};
  try {
    parsedMetadata = JSON.parse(metadataJson);
    if (parsedMetadata === null || Array.isArray(parsedMetadata) || typeof parsedMetadata !== "object") {
      errors.metadata_json = "Metadata must be a valid JSON object.";
    } else if (hasSecretLookingMetadata(parsedMetadata)) {
      errors.metadata_json = "Metadata contains secret-looking keys or values; store secrets in the approved secret store, not task metadata.";
    }
  } catch {
    errors.metadata_json = "Metadata must be valid JSON.";
  }

  if (Object.keys(errors).length > 0 || !isTaskStatus(status) || !isTaskPriority(priority)) {
    return { ok: false, errors };
  }

  return {
    ok: true,
    errors: {},
    value: {
      title,
      detail,
      status,
      priority,
      workspace_key: workspaceKey,
      metadata_json: metadataJson,
    },
  };
}

export function buildCreateTaskPayload(draft: TaskFormDraft): TaskValidationResult {
  return validateTaskForm(draft);
}

export function buildUpdateTaskPayload(draft: TaskFormDraft): TaskValidationResult {
  return validateTaskForm(draft);
}

export function buildTaskWorkspaceView(state: TaskWorkspaceState): TaskWorkspaceView {
  if (state.mode === "loading") {
    return {
      list: {
        statusLabel: "Loading tasks",
        copy: `Reading persisted tasks through ${taskBridgeCommands.list}…`,
        emptyCopy: "No placeholder tasks are shown while loading.",
        items: [],
      },
      detail: { kind: "loading", copy: "Select a task after persisted records finish loading." },
    };
  }

  if (state.mode === "error") {
    return {
      list: {
        statusLabel: "Task data unavailable",
        copy: "The native task bridge returned an error.",
        emptyCopy: state.error,
        items: [],
      },
      detail: { kind: "error", copy: state.error },
    };
  }

  const tasks = newestFirst(state.tasks.filter((task) => !task.archived_at && !task.deleted_at && task.status !== "archived" && task.status !== "deleted"));
  const selectedTask = state.selectedTaskId ? tasks.find((task) => task.id === state.selectedTaskId) : null;
  const items = tasks.map((task) => ({
    id: task.id,
    title: task.title,
    meta: `${taskStatusLabels[task.status]} · ${priorityLabel(task.priority)} · ${task.workspace_key}`,
    tone: taskTone(task),
    isSelected: task.id === state.selectedTaskId,
  }));

  let detail: TaskDetailView;
  if (selectedTask) {
    detail = {
      kind: "task",
      task: selectedTask,
      metadataPreview: formatTaskMetadata(selectedTask.metadata_json),
      detailLines: [
        `${taskStatusLabels[selectedTask.status]} · ${priorityLabel(selectedTask.priority)}`,
        `Workspace: ${selectedTask.workspace_key}`,
        `Updated: ${selectedTask.updated_at}`,
      ],
    };
  } else if (state.selectedTaskId) {
    detail = { kind: "missing", copy: `Task ${state.selectedTaskId} was not returned by ${taskBridgeCommands.detail} or is no longer visible.` };
  } else {
    detail = { kind: "empty", copy: tasks.length === 0 ? "No persisted tasks returned by the task bridge." : "Select a task to view details." };
  }

  return {
    list: {
      statusLabel: tasks.length > 0 ? countLabel(tasks.length, "task") : "No tasks",
      copy: tasks.length > 0 ? `Showing persisted tasks from ${taskBridgeCommands.list}.` : `No active task records returned by ${taskBridgeCommands.list}.`,
      emptyCopy: "Use the create form to persist a real task; this view never invents sample tasks.",
      items,
    },
    detail,
  };
}

export function formatTaskMetadata(metadataJson: string): string {
  try {
    const parsed = JSON.parse(metadataJson || "{}");
    return JSON.stringify(parsed, null, 2);
  } catch {
    return metadataJson;
  }
}
