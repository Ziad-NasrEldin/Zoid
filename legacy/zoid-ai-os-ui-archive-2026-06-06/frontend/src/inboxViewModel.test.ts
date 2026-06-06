import {
  applyInboxIntent,
  buildInboxAttentionViewModel,
  formatLinkedLabels,
  type InboxDataState,
  type InboxNotificationRecord,
} from "./inboxViewModel.ts";

const assert = {
  equal<T>(actual: T, expected: T) {
    if (actual !== expected) throw new Error(`Expected ${String(expected)}, got ${String(actual)}`);
  },
  deepEqual(actual: unknown, expected: unknown) {
    const actualJson = JSON.stringify(actual);
    const expectedJson = JSON.stringify(expected);
    if (actualJson !== expectedJson) throw new Error(`Expected ${expectedJson}, got ${actualJson}`);
  },
  match(actual: string | undefined, regex: RegExp) {
    if (!actual || !regex.test(actual)) throw new Error(`Expected ${actual ?? "undefined"} to match ${regex.source}`);
  },
};

const baseNotification = (overrides: Partial<InboxNotificationRecord>): InboxNotificationRecord => ({
  id: overrides.id ?? "n-default",
  notification_type: overrides.notification_type ?? "attention",
  title: overrides.title ?? "Needs attention",
  message: overrides.message ?? "A persisted notification requires attention.",
  severity: overrides.severity ?? "warning",
  state: overrides.state ?? "delivered",
  action_route: overrides.action_route ?? null,
  task_id: overrides.task_id ?? null,
  run_id: overrides.run_id ?? null,
  review_record_id: overrides.review_record_id ?? null,
  read_at: overrides.read_at ?? null,
  dismissed_at: overrides.dismissed_at ?? null,
  resolved_at: overrides.resolved_at ?? null,
  created_at: overrides.created_at ?? "2026-06-01T09:00:00.000Z",
  updated_at: overrides.updated_at ?? "2026-06-01T09:00:00.000Z",
  metadata_json: overrides.metadata_json ?? "{}",
});

function ready(records: InboxNotificationRecord[]): InboxDataState<InboxNotificationRecord> {
  return { state: "ready", records };
}

{
  const view = buildInboxAttentionViewModel({ state: "checking" });
  assert.equal(view.state, "loading");
  assert.equal(view.title, "Inbox attention");
  assert.deepEqual(view.items, []);
  assert.match(view.status, /Checking/i);
}

{
  const view = buildInboxAttentionViewModel({ state: "unavailable", reason: "native bridge unavailable" });
  assert.equal(view.state, "error");
  assert.equal(view.error, "native bridge unavailable");
  assert.deepEqual(view.items, []);
  assert.match(view.emptyCopy, /No fake notifications/i);
}

{
  const view = buildInboxAttentionViewModel(ready([]));
  assert.equal(view.state, "empty");
  assert.equal(view.status, "Empty");
  assert.deepEqual(view.items, []);
  assert.match(view.emptyCopy, /No active inbox notifications/i);
}

{
  const records = [
    baseNotification({ id: "resolved", title: "Resolved", state: "resolved", resolved_at: "2026-06-01T10:00:00.000Z" }),
    baseNotification({ id: "dismissed", title: "Dismissed", state: "dismissed", dismissed_at: "2026-06-01T10:00:00.000Z" }),
    baseNotification({ id: "read", title: "Read warning", severity: "warning", state: "read", read_at: "2026-06-01T10:00:00.000Z", updated_at: "2026-06-01T10:00:00.000Z" }),
    baseNotification({ id: "action", title: "Action required", severity: "info", state: "action_required", updated_at: "2026-06-01T08:00:00.000Z" }),
    baseNotification({ id: "critical", title: "Critical failure", notification_type: "failure", severity: "critical", state: "delivered", updated_at: "2026-06-01T07:00:00.000Z" }),
    baseNotification({ id: "newer-info", title: "Newer info", severity: "info", state: "delivered", updated_at: "2026-06-01T11:00:00.000Z" }),
  ];

  const view = buildInboxAttentionViewModel(ready(records));
  assert.equal(view.state, "ready");
  assert.equal(view.status, "3 active");
  assert.deepEqual(view.items.map((item) => item.id), ["action", "critical", "newer-info"]);
  assert.equal(view.items[0].stateLabel, "Action required");
  assert.equal(view.items[0].severityLabel, "Info");
  assert.equal(view.items[1].tone, "critical");
  assert.equal(view.items[2].isUnread, true);
}

{
  const notification = baseNotification({
    id: "linked",
    notification_type: "review_required",
    severity: "error",
    state: "action_required",
    action_route: "zoid://reviews/rev-1",
    task_id: "task-7",
    run_id: "run-3",
    review_record_id: "rev-1",
  });
  const view = buildInboxAttentionViewModel(ready([notification]));
  assert.equal(view.items[0].typeLabel, "Review required");
  assert.equal(view.items[0].actionRouteLabel, "zoid://reviews/rev-1");
  assert.deepEqual(view.items[0].linkedLabels, ["task task-7", "run run-3", "review rev-1"]);
  assert.deepEqual(formatLinkedLabels(notification), ["task task-7", "run run-3", "review rev-1"]);
  assert.deepEqual(view.items[0].intents.map((intent) => intent.kind), ["mark_read", "dismiss", "resolve", "require_action", "open_route"]);
}

{
  const original = baseNotification({ id: "intent", state: "delivered", action_route: "zoid://tasks/task-1" });
  const now = "2026-06-02T12:00:00.000Z";
  const readRecords = applyInboxIntent([original], { kind: "mark_read", notificationId: "intent" }, now);
  assert.equal(readRecords[0].state, "read");
  assert.equal(readRecords[0].read_at, now);

  const actionRecords = applyInboxIntent(readRecords, { kind: "require_action", notificationId: "intent" }, now);
  assert.equal(actionRecords[0].state, "action_required");

  const resolvedRecords = applyInboxIntent(actionRecords, { kind: "resolve", notificationId: "intent" }, now);
  assert.equal(resolvedRecords[0].state, "resolved");
  assert.equal(resolvedRecords[0].resolved_at, now);

  const dismissedRecords = applyInboxIntent([original], { kind: "dismiss", notificationId: "intent" }, now);
  assert.equal(dismissedRecords[0].state, "dismissed");
  assert.equal(dismissedRecords[0].dismissed_at, now);

  const routeRecords = applyInboxIntent([original], { kind: "open_route", notificationId: "intent", route: "zoid://tasks/task-1" }, now);
  assert.equal(routeRecords[0], original);
}

console.log("inboxViewModel tests passed");
