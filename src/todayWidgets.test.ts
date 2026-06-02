import { buildTodayWidgetsView, type TodayNotificationRecord, type TodayTaskRecord } from "./todayWidgets";

const tasks: TodayTaskRecord[] = [
  {
    id: "task-active",
    title: "Wire real Today widgets",
    detail: null,
    status: "active",
    priority: "urgent",
    workspace_key: "today",
    created_at: "2026-06-02T10:00:00Z",
    updated_at: "2026-06-02T10:10:00Z",
    archived_at: null,
    deleted_at: null,
    metadata_json: "{}",
  },
  {
    id: "task-archived",
    title: "Archived should not show",
    detail: null,
    status: "archived",
    priority: "normal",
    workspace_key: "today",
    created_at: "2026-06-02T09:00:00Z",
    updated_at: "2026-06-02T09:10:00Z",
    archived_at: "2026-06-02T09:20:00Z",
    deleted_at: null,
    metadata_json: "{}",
  },
];

const notifications: TodayNotificationRecord[] = [
  {
    id: "notification-blocker",
    notification_type: "blocker",
    title: "Profile is unconfigured",
    message: "A real blocker notification",
    severity: "warning",
    state: "action_required",
    action_route: null,
    task_id: "task-active",
    run_id: null,
    review_record_id: null,
    read_at: null,
    dismissed_at: null,
    resolved_at: null,
    created_at: "2026-06-02T11:00:00Z",
    updated_at: "2026-06-02T11:00:00Z",
    metadata_json: "{}",
  },
  {
    id: "notification-completion",
    notification_type: "completion",
    title: "Run completed",
    message: "A real completion notification",
    severity: "success",
    state: "delivered",
    action_route: null,
    task_id: "task-active",
    run_id: "run-1",
    review_record_id: null,
    read_at: null,
    dismissed_at: null,
    resolved_at: null,
    created_at: "2026-06-02T12:00:00Z",
    updated_at: "2026-06-02T12:00:00Z",
    metadata_json: "{}",
  },
];

const populated = buildTodayWidgetsView({
  source: "native",
  tasks: { state: "ready", records: tasks },
  inbox: { state: "ready", records: notifications },
  activeRuns: {
    state: "unavailable",
    reason: "No persisted run-list command exists yet; Today cannot query active AgentRun rows truthfully.",
  },
});

if (populated.tasks.status !== "1 task" || populated.tasks.items[0]?.id !== "task-active") {
  throw new Error("Today task widget must render only real active, non-archived task records");
}

if (populated.tasks.items.some((item) => item.id === "task-archived")) {
  throw new Error("Today task widget must not show archived/deleted tasks");
}

if (populated.blockers.status !== "1 item" || populated.blockers.tone !== "blocked") {
  throw new Error("Today blockers widget must derive blocker count/tone from real notification records");
}

if (populated.completions.status !== "1 completion" || populated.completions.tone !== "ready") {
  throw new Error("Today completions widget must derive completion count/tone from real notification records");
}

if (populated.activeRuns.status !== "Unavailable" || !populated.activeRuns.copy.includes("No persisted run-list command")) {
  throw new Error("Today active-runs widget must state the truthful backend gap instead of inventing runs");
}

const empty = buildTodayWidgetsView({
  source: "native",
  tasks: { state: "ready", records: [] },
  inbox: { state: "ready", records: [] },
  activeRuns: { state: "ready", records: [] },
});

for (const panel of Object.values(empty)) {
  if (panel.items.length !== 0) {
    throw new Error("empty Today widgets must not fabricate list items");
  }
  if (panel.status !== "Empty") {
    throw new Error(`empty Today widget must be labeled Empty, got ${panel.title}: ${panel.status}`);
  }
}

const preview = buildTodayWidgetsView({
  source: "preview",
  tasks: { state: "unavailable", reason: "Native task data is unavailable in browser preview." },
  inbox: { state: "unavailable", reason: "Native inbox data is unavailable in browser preview." },
  activeRuns: { state: "unavailable", reason: "Native run data is unavailable in browser preview." },
});

if (!preview.tasks.emptyCopy?.includes("No tasks are simulated") || !preview.completions.emptyCopy?.includes("simulated")) {
  throw new Error("preview Today widgets must explicitly reject simulated tasks/notifications");
}
