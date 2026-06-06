export type HistoryMode = "task" | "run" | "notification" | "entity";

export type HistoryLoadStatus = "idle" | "loading" | "ready" | "error";

export interface HistoryEntityRef {
  entity_type: string;
  entity_id: string;
}

export interface HistoryCursor {
  timestamp: string;
  event_id: string;
}

export interface HistoryEventTargetRecord extends HistoryEntityRef {
  relation_type: string;
}

export interface HistoryEventRecord {
  id: string;
  action_type: string;
  outcome: string;
  timestamp: string;
  actor_type?: string;
  actor_id?: string | null;
  workspace_key?: string | null;
  summary: string;
  source: string;
  metadata_json?: string;
  targets: HistoryEventTargetRecord[];
}

export interface HistoryTimelineRecord {
  event: HistoryEventRecord;
  matched_entities: HistoryEventTargetRecord[];
}

export interface HistoryTimelineInput {
  mode: HistoryMode;
  primary: HistoryEntityRef;
  records?: HistoryTimelineRecord[] | null;
  status?: HistoryLoadStatus;
  error?: string | null;
  nextCursor?: HistoryCursor | null;
  pageSize?: number;
  includeRelated?: boolean;
}

export interface HistoryTimelineEntryViewModel {
  id: string;
  timestamp: string;
  timeLabel: string;
  actionLabel: string;
  sourceLabel: string;
  outcomeLabel: string;
  summary: string;
  matchedLabels: string[];
  targetLabels: string[];
  isPrimaryMatch: boolean;
  tone: "success" | "warning" | "error" | "neutral";
}

export interface HistoryTimelineViewModel {
  mode: HistoryMode;
  title: string;
  status: HistoryLoadStatus;
  isLoading: boolean;
  errorMessage: string | null;
  emptyMessage: string | null;
  entries: HistoryTimelineEntryViewModel[];
  nextCursor: HistoryCursor | null;
  paginationLabel: string;
  hasNextPage: boolean;
  includeRelated: boolean;
}

const MODE_LABELS: Record<HistoryMode, string> = {
  task: "Task history",
  run: "Run history",
  notification: "Notification history",
  entity: "Entity history",
};

const SECRET_PATTERNS = [
  /\b(api[_-]?key|token|secret|password|passwd|authorization|bearer)\b\s*[:=]\s*([^\s,;]+)/gi,
  /\b(sk-[a-z0-9_-]{8,})\b/gi,
];

export function createHistoryTimelineViewModel(input: HistoryTimelineInput): HistoryTimelineViewModel {
  const status = input.status ?? "ready";
  const records = input.records ?? [];
  const entries = sortHistoryRecords(records).map((record) =>
    createEntryViewModel(record, input.primary),
  );
  const errorMessage = status === "error" ? sanitizeMessage(input.error || "History could not be loaded.") : null;
  const emptyMessage = status === "ready" && entries.length === 0 ? emptyHistoryMessage(input.mode) : null;
  const nextCursor = input.nextCursor ?? deriveNextCursor(records, input.pageSize);

  return {
    mode: input.mode,
    title: MODE_LABELS[input.mode],
    status,
    isLoading: status === "loading",
    errorMessage,
    emptyMessage,
    entries,
    nextCursor,
    paginationLabel: paginationLabel(status, entries.length, nextCursor),
    hasNextPage: nextCursor !== null,
    includeRelated: input.includeRelated ?? true,
  };
}

export function sortHistoryRecords(records: readonly HistoryTimelineRecord[]): HistoryTimelineRecord[] {
  return [...records].sort((left, right) => {
    const byTimestamp = compareDescending(left.event.timestamp, right.event.timestamp);
    if (byTimestamp !== 0) {
      return byTimestamp;
    }
    return compareDescending(left.event.id, right.event.id);
  });
}

export function deriveNextCursor(
  records: readonly HistoryTimelineRecord[],
  pageSize?: number,
): HistoryCursor | null {
  if (!pageSize || records.length < pageSize || records.length === 0) {
    return null;
  }
  const last = sortHistoryRecords(records)[records.length - 1]?.event;
  return last ? { timestamp: last.timestamp, event_id: last.id } : null;
}

export function sanitizeMessage(value: string): string {
  const singleLine = value.replace(/[\r\n\t]+/g, " ").replace(/\s+/g, " ").trim();
  const withoutLogPrefix = singleLine.replace(/^raw\s+log\s*[:=-]?\s*/i, "");
  const withoutMaskFragments = withoutLogPrefix.replace(/\*{3,}/g, "••••");
  const truncated =
    withoutMaskFragments.length > 220 ? `${withoutMaskFragments.slice(0, 217).trim()}...` : withoutMaskFragments;
  return SECRET_PATTERNS.reduce(
    (message, pattern) => message.replace(pattern, (_match, key) => `${key}=••••`),
    truncated,
  );
}

export function formatEntityLabel(entity: HistoryEntityRef): string {
  return `${humanize(entity.entity_type)} ${entity.entity_id}`.trim();
}

function createEntryViewModel(
  record: HistoryTimelineRecord,
  primary: HistoryEntityRef,
): HistoryTimelineEntryViewModel {
  const event = record.event;
  const matched = record.matched_entities.length > 0 ? record.matched_entities : event.targets;
  return {
    id: event.id,
    timestamp: event.timestamp,
    timeLabel: formatTimestamp(event.timestamp),
    actionLabel: humanize(event.action_type),
    sourceLabel: event.source ? humanize(event.source) : "Unknown source",
    outcomeLabel: event.outcome ? humanize(event.outcome) : "Unknown outcome",
    summary: sanitizeMessage(event.summary || "No summary provided."),
    matchedLabels: uniqueLabels(matched.map(formatEntityWithRelation)),
    targetLabels: uniqueLabels(event.targets.map(formatEntityWithRelation)),
    isPrimaryMatch: matched.some(
      (entity) => entity.entity_type === primary.entity_type && entity.entity_id === primary.entity_id,
    ),
    tone: toneForOutcome(event.outcome),
  };
}

function formatEntityWithRelation(entity: HistoryEventTargetRecord): string {
  const relation = entity.relation_type ? ` (${humanize(entity.relation_type)})` : "";
  return `${formatEntityLabel(entity)}${relation}`;
}

function compareDescending(left: string, right: string): number {
  if (left === right) {
    return 0;
  }
  return left > right ? -1 : 1;
}

function formatTimestamp(timestamp: string): string {
  const parsed = new Date(timestamp);
  if (Number.isNaN(parsed.getTime())) {
    return timestamp;
  }
  return parsed.toLocaleString("en-US", {
    year: "numeric",
    month: "short",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    hour12: false,
    timeZone: "UTC",
  });
}

function humanize(value: string): string {
  return value
    .replace(/[_-]+/g, " ")
    .replace(/\s+/g, " ")
    .trim()
    .replace(/\b\w/g, (letter) => letter.toUpperCase());
}

function emptyHistoryMessage(mode: HistoryMode): string {
  switch (mode) {
    case "task":
      return "No task history events yet.";
    case "run":
      return "No run history events yet.";
    case "notification":
      return "No notification history events yet.";
    case "entity":
      return "No entity history events yet.";
  }
}

function paginationLabel(status: HistoryLoadStatus, count: number, cursor: HistoryCursor | null): string {
  if (status === "loading") {
    return "Loading history...";
  }
  if (status === "error") {
    return "History unavailable";
  }
  if (count === 0) {
    return "No history to paginate";
  }
  return cursor ? `Showing ${count} events; more available` : `Showing ${count} events`;
}

function toneForOutcome(outcome: string): HistoryTimelineEntryViewModel["tone"] {
  const normalized = outcome.toLowerCase();
  if (["success", "completed", "resolved", "sent"].includes(normalized)) {
    return "success";
  }
  if (["failed", "error", "blocked", "rejected"].includes(normalized)) {
    return "error";
  }
  if (["warning", "pending", "needs_review", "retry"].includes(normalized)) {
    return "warning";
  }
  return "neutral";
}

function uniqueLabels(labels: string[]): string[] {
  return [...new Set(labels.filter(Boolean))];
}
