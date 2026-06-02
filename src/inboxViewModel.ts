export type InboxNotificationType = "completion" | "blocker" | "failure" | "review_required" | "attention";
export type InboxNotificationSeverity = "info" | "success" | "warning" | "error" | "critical";
export type InboxNotificationState =
  | "pending"
  | "delivered"
  | "read"
  | "action_required"
  | "resolved"
  | "dismissed"
  | "failed";

export type InboxNotificationRecord = {
  id: string;
  notification_type: InboxNotificationType;
  title: string;
  message: string;
  severity: InboxNotificationSeverity;
  state: InboxNotificationState;
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

export type InboxDataState<T> =
  | { state: "checking" }
  | { state: "unavailable"; reason: string }
  | { state: "ready"; records: T[] };

export type InboxIntentKind = "mark_read" | "dismiss" | "resolve" | "require_action" | "open_route";

export type InboxIntent =
  | { kind: "mark_read"; notificationId: string }
  | { kind: "dismiss"; notificationId: string }
  | { kind: "resolve"; notificationId: string }
  | { kind: "require_action"; notificationId: string }
  | { kind: "open_route"; notificationId: string; route: string };

export type InboxTone = "neutral" | "info" | "success" | "warning" | "danger" | "critical";

export type InboxAttentionItemView = {
  id: string;
  title: string;
  message: string;
  typeLabel: string;
  severityLabel: string;
  stateLabel: string;
  tone: InboxTone;
  isUnread: boolean;
  createdAt: string;
  updatedAt: string;
  actionRouteLabel?: string;
  linkedLabels: string[];
  intents: InboxIntent[];
};

export type InboxAttentionViewModel = {
  title: string;
  state: "loading" | "error" | "empty" | "ready";
  status: string;
  summary: string;
  emptyCopy?: string;
  error?: string;
  items: InboxAttentionItemView[];
};

const typeLabels: Record<InboxNotificationType, string> = {
  completion: "Completion",
  blocker: "Blocker",
  failure: "Failure",
  review_required: "Review required",
  attention: "Attention",
};

const severityLabels: Record<InboxNotificationSeverity, string> = {
  info: "Info",
  success: "Success",
  warning: "Warning",
  error: "Error",
  critical: "Critical",
};

const stateLabels: Record<InboxNotificationState, string> = {
  pending: "Pending",
  delivered: "Delivered",
  read: "Read",
  action_required: "Action required",
  resolved: "Resolved",
  dismissed: "Dismissed",
  failed: "Failed",
};

const severityRank: Record<InboxNotificationSeverity, number> = {
  critical: 0,
  error: 1,
  warning: 2,
  info: 3,
  success: 4,
};

function countLabel(count: number) {
  return `${count} active`;
}

function timestamp(record: InboxNotificationRecord) {
  return record.updated_at || record.created_at || "";
}

function isActiveNotification(notification: InboxNotificationRecord) {
  return ["pending", "delivered", "action_required", "failed"].includes(notification.state)
    && !notification.dismissed_at
    && !notification.resolved_at;
}

function attentionRank(notification: InboxNotificationRecord) {
  if (notification.state === "action_required") return -1;
  if (notification.state === "failed") return 0;
  return severityRank[notification.severity] + 1;
}

function isUnread(notification: InboxNotificationRecord) {
  return notification.state !== "read" && !notification.read_at;
}

function toneFor(notification: InboxNotificationRecord): InboxTone {
  if (notification.severity === "critical") return "critical";
  if (notification.severity === "error" || notification.notification_type === "failure") return "danger";
  if (notification.severity === "warning" || notification.notification_type === "blocker" || notification.notification_type === "review_required") return "warning";
  if (notification.severity === "success" || notification.notification_type === "completion") return "success";
  if (notification.severity === "info") return "info";
  return "neutral";
}

function compareAttention(a: InboxNotificationRecord, b: InboxNotificationRecord) {
  const rankDifference = attentionRank(a) - attentionRank(b);
  if (rankDifference !== 0) return rankDifference;

  const unreadDifference = Number(isUnread(b)) - Number(isUnread(a));
  if (unreadDifference !== 0) return unreadDifference;

  return timestamp(b).localeCompare(timestamp(a));
}

export function formatLinkedLabels(notification: Pick<InboxNotificationRecord, "task_id" | "run_id" | "review_record_id">): string[] {
  return [
    notification.task_id ? `task ${notification.task_id}` : null,
    notification.run_id ? `run ${notification.run_id}` : null,
    notification.review_record_id ? `review ${notification.review_record_id}` : null,
  ].filter((label): label is string => Boolean(label));
}

function intentsFor(notification: InboxNotificationRecord): InboxIntent[] {
  const intents: InboxIntent[] = [
    { kind: "mark_read", notificationId: notification.id },
    { kind: "dismiss", notificationId: notification.id },
    { kind: "resolve", notificationId: notification.id },
    { kind: "require_action", notificationId: notification.id },
  ];
  if (notification.action_route) {
    intents.push({ kind: "open_route", notificationId: notification.id, route: notification.action_route });
  }
  return intents;
}

function toItem(notification: InboxNotificationRecord): InboxAttentionItemView {
  return {
    id: notification.id,
    title: notification.title,
    message: notification.message,
    typeLabel: typeLabels[notification.notification_type],
    severityLabel: severityLabels[notification.severity],
    stateLabel: stateLabels[notification.state],
    tone: toneFor(notification),
    isUnread: isUnread(notification),
    createdAt: notification.created_at,
    updatedAt: notification.updated_at,
    actionRouteLabel: notification.action_route ?? undefined,
    linkedLabels: formatLinkedLabels(notification),
    intents: intentsFor(notification),
  };
}

export function buildInboxAttentionViewModel(inbox: InboxDataState<InboxNotificationRecord>): InboxAttentionViewModel {
  if (inbox.state === "checking") {
    return {
      title: "Inbox attention",
      state: "loading",
      status: "Checking",
      summary: "Checking persisted inbox notifications from the native notification bridge…",
      items: [],
    };
  }

  if (inbox.state === "unavailable") {
    return {
      title: "Inbox attention",
      state: "error",
      status: "Unavailable",
      summary: inbox.reason,
      error: inbox.reason,
      emptyCopy: "No fake notifications are shown while inbox data is unavailable.",
      items: [],
    };
  }

  const activeItems = inbox.records.filter(isActiveNotification).sort(compareAttention).map(toItem);
  if (activeItems.length === 0) {
    return {
      title: "Inbox attention",
      state: "empty",
      status: "Empty",
      summary: "No active inbox notifications need attention.",
      emptyCopy: "No active inbox notifications returned by the persisted inbox source. No fake notifications are generated.",
      items: [],
    };
  }

  return {
    title: "Inbox attention",
    state: "ready",
    status: countLabel(activeItems.length),
    summary: "Active persisted inbox notifications ordered by action requirement, severity, unread status, then recency.",
    items: activeItems,
  };
}

export function applyInboxIntent(records: InboxNotificationRecord[], intent: InboxIntent, nowIso: string): InboxNotificationRecord[] {
  return records.map((record) => {
    if (record.id !== intent.notificationId) return record;

    if (intent.kind === "open_route") return record;

    if (intent.kind === "mark_read") {
      return { ...record, state: "read", read_at: record.read_at ?? nowIso, updated_at: nowIso };
    }

    if (intent.kind === "dismiss") {
      return { ...record, state: "dismissed", dismissed_at: record.dismissed_at ?? nowIso, updated_at: nowIso };
    }

    if (intent.kind === "resolve") {
      return { ...record, state: "resolved", resolved_at: record.resolved_at ?? nowIso, updated_at: nowIso };
    }

    return { ...record, state: "action_required", updated_at: nowIso };
  });
}
