import {
  createHistoryTimelineViewModel,
  sanitizeMessage,
  type HistoryTimelineRecord,
  type HistoryTimelineViewModel,
} from "./historyTimelineViewModel";

export type TaskLinkedPanelsInvoke = <T = unknown>(command: string, args?: Record<string, unknown>) => Promise<T>;

export type LinkedRunRecord = {
  id: string;
  status?: string;
  exit_code?: number | null;
  started_at?: string | null;
  completed_at?: string | null;
  [key: string]: unknown;
};

export type LinkedReviewRecord = {
  id: string;
  verdict?: string;
  subject_type?: string;
  subject_id?: string;
  created_at?: string | null;
  [key: string]: unknown;
};

export type TaskLinkedPanelsState =
  | { mode: "idle"; taskId: string | null }
  | { mode: "loading"; taskId: string }
  | { mode: "error"; taskId: string; error: string }
  | {
      mode: "ready";
      taskId: string;
      runs: LinkedRunRecord[];
      reviews: LinkedReviewRecord[];
      history: HistoryTimelineRecord[];
    };

export type LinkedPanelItem = {
  id: string;
  title: string;
  summary: string;
  meta: string;
  tone: "success" | "warning" | "error" | "neutral";
};

export type TaskLinkedPanelsViewModel = {
  taskId: string | null;
  isLoading: boolean;
  errorMessage: string | null;
  runPanel: { title: string; emptyCopy: string; items: LinkedPanelItem[] };
  reviewPanel: { title: string; emptyCopy: string; items: LinkedPanelItem[] };
  historyPanel: HistoryTimelineViewModel;
};

const PAGE_SIZE = 25;

export async function loadTaskLinkedPanelsFromBridge(
  invoke: TaskLinkedPanelsInvoke,
  taskId: string,
): Promise<TaskLinkedPanelsState> {
  try {
    const history = await invoke<HistoryTimelineRecord[]>("list_entity_history_command", {
      request: {
        entity_type: "task",
        entity_id: taskId,
        include_related: true,
        limit: PAGE_SIZE,
      },
    });
    const runIds = extractEntityIds(history, "run");
    const reviewIds = extractEntityIds(history, "review");
    const runs = await Promise.all(
      runIds.map((runId) => invoke<LinkedRunRecord>("read_run_status_command", { runId }).catch(() => null)),
    );
    const reviews = await Promise.all(
      reviewIds.map((reviewRecordId) =>
        invoke<LinkedReviewRecord>("read_review_record_command", { reviewRecordId }).catch(() => null),
      ),
    );
    return {
      mode: "ready",
      taskId,
      runs: runs.filter((run): run is LinkedRunRecord => run !== null),
      reviews: reviews.filter((review): review is LinkedReviewRecord => review !== null),
      history,
    };
  } catch (error) {
    return { mode: "error", taskId, error: bridgeError(error) };
  }
}

export function createTaskLinkedPanelsViewModel(state: TaskLinkedPanelsState): TaskLinkedPanelsViewModel {
  const taskId = state.taskId;
  const history = state.mode === "ready" ? state.history : [];
  return {
    taskId,
    isLoading: state.mode === "loading",
    errorMessage: state.mode === "error" ? sanitizeMessage(state.error) : null,
    runPanel: {
      title: "Linked runs",
      emptyCopy: state.mode === "ready" ? "No runs are linked to this task yet." : "Select a task to load linked runs.",
      items: state.mode === "ready" ? state.runs.map(runItem) : [],
    },
    reviewPanel: {
      title: "Linked reviews",
      emptyCopy: state.mode === "ready" ? "No reviews are linked to this task yet." : "Select a task to load linked reviews.",
      items: state.mode === "ready" ? state.reviews.map(reviewItem) : [],
    },
    historyPanel: createHistoryTimelineViewModel({
      mode: "task",
      primary: { entity_type: "task", entity_id: taskId || "unselected" },
      records: history,
      status: state.mode === "loading" ? "loading" : state.mode === "error" ? "error" : "ready",
      error: state.mode === "error" ? state.error : null,
      pageSize: PAGE_SIZE,
      includeRelated: true,
    }),
  };
}

function extractEntityIds(history: HistoryTimelineRecord[], entityType: "run" | "review"): string[] {
  const ids = history.flatMap((record) => [
    ...record.event.targets,
    ...record.matched_entities,
  ]).filter((target) => target.entity_type === entityType).map((target) => target.entity_id);
  return [...new Set(ids)].sort();
}

function runItem(run: LinkedRunRecord): LinkedPanelItem {
  const status = String(run.status ?? "unknown");
  const exit = typeof run.exit_code === "number" ? ` · exit ${run.exit_code}` : "";
  return {
    id: run.id,
    title: `Run ${run.id}`,
    summary: sanitizeMessage(`${status}${exit}`),
    meta: [run.started_at, run.completed_at].filter(Boolean).join(" → ") || "No persisted run timing",
    tone: toneFor(status),
  };
}

function reviewItem(review: LinkedReviewRecord): LinkedPanelItem {
  const verdict = String(review.verdict ?? "unknown");
  return {
    id: review.id,
    title: `Review ${review.id}`,
    summary: sanitizeMessage(verdict),
    meta: [review.subject_type, review.subject_id, review.created_at].filter(Boolean).join(" · ") || "No persisted review context",
    tone: toneFor(verdict),
  };
}

function toneFor(value: string): LinkedPanelItem["tone"] {
  const normalized = value.toLowerCase();
  if (["success", "completed", "approved", "passed"].includes(normalized)) return "success";
  if (["failed", "rejected", "error"].includes(normalized)) return "error";
  if (["pending", "running", "review_required", "blocked"].includes(normalized)) return "warning";
  return "neutral";
}

function bridgeError(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "unknown linked task bridge error";
}
