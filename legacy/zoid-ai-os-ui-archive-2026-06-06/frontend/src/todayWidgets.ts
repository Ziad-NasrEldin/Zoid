export type TodayDataSource = "native" | "preview" | "checking";
export type TodayTone = "ready" | "blocked" | "pending";

export type TodayTaskStatus =
  | "inbox"
  | "planned"
  | "active"
  | "waiting"
  | "review_required"
  | "blocked"
  | "completed"
  | "failed"
  | "cancelled"
  | "archived"
  | "deleted";

export type TodayTaskPriority = "low" | "normal" | "high" | "urgent";

export type TodayTaskRecord = {
  id: string;
  title: string;
  detail?: string | null;
  status: TodayTaskStatus;
  priority: TodayTaskPriority;
  workspace_key: string;
  created_at: string;
  updated_at: string;
  archived_at?: string | null;
  deleted_at?: string | null;
  metadata_json: string;
};

export type TodayNotificationType = "completion" | "blocker" | "failure" | "review_required" | "attention";
export type TodayNotificationSeverity = "info" | "success" | "warning" | "error" | "critical";
export type TodayNotificationState =
  | "pending"
  | "delivered"
  | "read"
  | "action_required"
  | "resolved"
  | "dismissed"
  | "failed";

export type TodayNotificationRecord = {
  id: string;
  notification_type: TodayNotificationType;
  title: string;
  message: string;
  severity: TodayNotificationSeverity;
  state: TodayNotificationState;
  action_route?: string | null;
  task_id?: string | null;
  run_id?: string | null;
  review_record_id?: string | null;
  read_at?: string | null;
  dismissed_at?: string | null;
  resolved_at?: string | null;
  created_at: string;
  updated_at: string;
  metadata_json: string;
};

export type TodayActiveRunRecord = {
  id: string;
  task_id: string;
  status: "queued" | "starting" | "running" | "waiting_for_input" | "review_required" | "completed" | "failed" | "cancelled" | "blocked";
  updated_at: string;
  output_summary?: string | null;
  error_summary?: string | null;
};

export type TodayDataState<T> =
  | { state: "checking" }
  | { state: "unavailable"; reason: string }
  | { state: "ready"; records: T[] };

export type TodayWidgetsInput = {
  source: TodayDataSource;
  tasks: TodayDataState<TodayTaskRecord>;
  inbox: TodayDataState<TodayNotificationRecord>;
  activeRuns: TodayDataState<TodayActiveRunRecord>;
};

export type TodayListItemView = {
  id: string;
  title: string;
  meta: string;
  tone: TodayTone;
};

export type TodayWidgetPanelView = {
  title: string;
  status: string;
  tone: TodayTone;
  copy: string;
  emptyCopy?: string;
  items: TodayListItemView[];
};

export type TodayWidgetsView = {
  tasks: TodayWidgetPanelView;
  activeRuns: TodayWidgetPanelView;
  blockers: TodayWidgetPanelView;
  completions: TodayWidgetPanelView;
};

const taskStatusLabels: Record<TodayTaskStatus, string> = {
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

const notificationTypeLabels: Record<TodayNotificationType, string> = {
  completion: "Completion",
  blocker: "Blocker",
  failure: "Failure",
  review_required: "Review required",
  attention: "Attention",
};

function countLabel(count: number, singular: string, plural = `${singular}s`) {
  return `${count} ${count === 1 ? singular : plural}`;
}

function priorityLabel(priority: TodayTaskPriority) {
  return priority === "normal" ? "normal priority" : `${priority} priority`;
}

function taskTone(task: TodayTaskRecord): TodayTone {
  if (["blocked", "failed", "review_required"].includes(task.status)) return "blocked";
  if (["active", "completed"].includes(task.status)) return "ready";
  return "pending";
}

function notificationTone(notification: TodayNotificationRecord): TodayTone {
  if (["blocker", "failure", "review_required"].includes(notification.notification_type)) return "blocked";
  if (notification.notification_type === "completion" || notification.severity === "success") return "ready";
  return "pending";
}

function newestFirst<T extends { updated_at?: string; created_at?: string }>(records: T[]) {
  return [...records].sort((a, b) => (b.updated_at ?? b.created_at ?? "").localeCompare(a.updated_at ?? a.created_at ?? ""));
}

function taskItem(task: TodayTaskRecord): TodayListItemView {
  return {
    id: task.id,
    title: task.title,
    meta: `${taskStatusLabels[task.status]} · ${priorityLabel(task.priority)} · ${task.workspace_key}`,
    tone: taskTone(task),
  };
}

function notificationItem(notification: TodayNotificationRecord): TodayListItemView {
  const linked = [notification.task_id ? `task ${notification.task_id}` : null, notification.run_id ? `run ${notification.run_id}` : null]
    .filter(Boolean)
    .join(" · ");
  return {
    id: notification.id,
    title: notification.title,
    meta: `${notificationTypeLabels[notification.notification_type]} · ${notification.severity}${linked ? ` · ${linked}` : ""}`,
    tone: notificationTone(notification),
  };
}

function buildTasksPanel(tasks: TodayDataState<TodayTaskRecord>): TodayWidgetPanelView {
  if (tasks.state === "checking") {
    return {
      title: "Today tasks",
      status: "Checking",
      tone: "pending",
      copy: "Reading local task records from SQLite through the native task bridge…",
      items: [],
    };
  }
  if (tasks.state === "unavailable") {
    return {
      title: "Today tasks",
      status: "Unavailable",
      tone: "blocked",
      copy: tasks.reason,
      emptyCopy: "No tasks are simulated in browser preview or bridge error states.",
      items: [],
    };
  }

  const visibleTasks = tasks.records.filter((task) => !task.archived_at && !task.deleted_at && task.status !== "archived" && task.status !== "deleted");
  const items = newestFirst(visibleTasks).slice(0, 5).map(taskItem);
  return {
    title: "Today tasks",
    status: visibleTasks.length > 0 ? countLabel(visibleTasks.length, "task") : "Empty",
    tone: visibleTasks.length > 0 ? "ready" : "pending",
    copy: visibleTasks.length > 0
      ? "Real active task records from list_tasks_command. Archived and deleted records are not included in Today."
      : "No active local tasks returned by list_tasks_command.",
    emptyCopy: "Create/list/detail UI is scheduled for P2.21; Today will populate from persisted tasks only.",
    items,
  };
}

function buildActiveRunsPanel(activeRuns: TodayDataState<TodayActiveRunRecord>): TodayWidgetPanelView {
  if (activeRuns.state === "checking") {
    return {
      title: "Active runs",
      status: "Checking",
      tone: "pending",
      copy: "Checking for active run data available to this frontend session…",
      items: [],
    };
  }
  if (activeRuns.state === "unavailable") {
    return {
      title: "Active runs",
      status: "Unavailable",
      tone: "pending",
      copy: activeRuns.reason,
      emptyCopy: "No run rows are fabricated. A backend run-list bridge is still needed for persisted active-run widgets.",
      items: [],
    };
  }

  const activeStatuses = new Set(["queued", "starting", "running", "waiting_for_input", "review_required"]);
  const records = activeRuns.records.filter((run) => activeStatuses.has(run.status));
  const items = newestFirst(records).slice(0, 5).map((run) => ({
    id: run.id,
    title: run.output_summary || run.error_summary || `Run ${run.id}`,
    meta: `${run.status.replace(/_/g, " ")} · task ${run.task_id}`,
    tone: run.status === "review_required" ? "blocked" : "ready" as TodayTone,
  }));
  return {
    title: "Active runs",
    status: records.length > 0 ? countLabel(records.length, "run") : "Empty",
    tone: records.length > 0 ? "ready" : "pending",
    copy: records.length > 0 ? "Active run records are real records known to this frontend session." : "No active runs are currently known to this frontend session.",
    emptyCopy: "Persisted active runs need a run-list command; Today does not infer them from task or notification data.",
    items,
  };
}

function buildNotificationPanel(
  inbox: TodayDataState<TodayNotificationRecord>,
  kind: "blockers" | "completions",
): TodayWidgetPanelView {
  const title = kind === "blockers" ? "Blockers" : "Completions";
  const types = kind === "blockers" ? new Set(["blocker", "failure", "review_required"] as TodayNotificationType[]) : new Set(["completion"] as TodayNotificationType[]);
  if (inbox.state === "checking") {
    return {
      title,
      status: "Checking",
      tone: "pending",
      copy: "Reading persisted inbox notifications from the native notification bridge…",
      items: [],
    };
  }
  if (inbox.state === "unavailable") {
    return {
      title,
      status: "Unavailable",
      tone: "blocked",
      copy: inbox.reason,
      emptyCopy: "No blocker or completion notifications are simulated.",
      items: [],
    };
  }

  const records = newestFirst(inbox.records.filter((notification) => types.has(notification.notification_type)));
  return {
    title,
    status: records.length > 0 ? countLabel(records.length, kind === "blockers" ? "item" : "completion") : "Empty",
    tone: records.length > 0 ? (kind === "blockers" ? "blocked" : "ready") : "pending",
    copy: records.length > 0
      ? `Real ${kind} from list_inbox_notifications_command.`
      : kind === "blockers"
        ? "No active blocker, failure, or review-required notifications returned by the inbox bridge."
        : "No active completion notifications returned by the inbox bridge.",
    emptyCopy: kind === "blockers"
      ? "Blockers will appear only when persisted notifications exist."
      : "Completions will appear only when real run/task/review notifications exist.",
    items: records.slice(0, 5).map(notificationItem),
  };
}

export function buildTodayWidgetsView(input: TodayWidgetsInput): TodayWidgetsView {
  return {
    tasks: buildTasksPanel(input.tasks),
    activeRuns: buildActiveRunsPanel(input.activeRuns),
    blockers: buildNotificationPanel(input.inbox, "blockers"),
    completions: buildNotificationPanel(input.inbox, "completions"),
  };
}
