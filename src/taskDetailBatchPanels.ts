import type { HistoryTimelineRecord } from "./historyTimelineViewModel";
import { sanitizeMessage } from "./historyTimelineViewModel";
import type { InboxDataState, InboxNotificationRecord } from "./inboxViewModel";

export type ManualReviewInvoke = <T = unknown>(command: string, args?: Record<string, unknown>) => Promise<T>;

export type ManualReviewRecord = {
  id: string;
  task_id: string;
  run_id?: string | null;
  reviewer_profile_id?: string | null;
  verdict: string;
  evidence_summary: string;
  required_fixes_json: string;
  metadata_json?: string | null;
  [key: string]: unknown;
};

export type ManualReviewDraft = {
  taskId: string | null;
  runId: string | null;
  reviewerProfileId: string;
  verdict: "approved" | "changes_required" | "insufficient_evidence" | "blocked";
  evidenceSummary: string;
  requiredFixesJson: string;
};

export type ManualReviewState = {
  mode: "idle" | "submitting" | "ready" | "blocked" | "error";
  draft: ManualReviewDraft;
  review: ManualReviewRecord | null;
  errorMessage: string | null;
  validationErrors: string[];
};

export type ManualReviewViewModel = {
  title: string;
  statusLabel: string;
  canSubmit: boolean;
  canClear: boolean;
  isBusy: boolean;
  lastReviewLabel: string | null;
  errorMessage: string | null;
  validationErrors: string[];
};

export function createInitialManualReviewState(taskId: string | null, runId: string | null): ManualReviewState {
  return {
    mode: "idle",
    draft: {
      taskId,
      runId,
      reviewerProfileId: "",
      verdict: "approved",
      evidenceSummary: "",
      requiredFixesJson: "[]",
    },
    review: null,
    errorMessage: null,
    validationErrors: [],
  };
}

export function updateManualReviewDraft(state: ManualReviewState, patch: Partial<ManualReviewDraft> & { clear?: boolean }): ManualReviewState {
  if (patch.clear) return createInitialManualReviewState(state.draft.taskId, state.draft.runId);
  const { clear: _clear, ...draftPatch } = patch;
  return { ...state, draft: { ...state.draft, ...draftPatch }, errorMessage: null, validationErrors: [] };
}

export function resetManualReviewForTask(state: ManualReviewState, taskId: string | null, runId: string | null): ManualReviewState {
  if (state.draft.taskId === taskId && state.draft.runId === runId) return state;
  return createInitialManualReviewState(taskId, runId);
}

export async function createManualReviewThroughBridge(invoke: ManualReviewInvoke, state: ManualReviewState): Promise<ManualReviewState> {
  const validationErrors = validateManualReview(state.draft);
  if (validationErrors.length > 0) return { ...state, mode: "blocked", validationErrors, errorMessage: validationErrors.join(" ") };

  try {
    const review = await invoke<ManualReviewRecord>("create_manual_review_command", {
      request: {
        task_id: state.draft.taskId,
        run_id: state.draft.runId,
        reviewer_profile_id: state.draft.reviewerProfileId.trim() || null,
        verdict: state.draft.verdict,
        evidence_summary: sanitizeMessage(state.draft.evidenceSummary),
        required_fixes_json: normalizeRequiredFixes(state.draft.requiredFixesJson),
        metadata_json: JSON.stringify({ source: "task_detail_manual_review_stub" }),
      },
    });
    return { ...state, mode: "ready", review, errorMessage: null, validationErrors: [] };
  } catch (error) {
    return { ...state, mode: "error", errorMessage: bridgeError(error), validationErrors: [] };
  }
}

export function createManualReviewViewModel(state: ManualReviewState): ManualReviewViewModel {
  return {
    title: "Manual review stub",
    statusLabel: state.review ? `Latest review: ${humanize(state.review.verdict)}` : state.mode === "blocked" ? "Blocked" : state.mode === "error" ? "Error" : "Ready to record evidence",
    canSubmit: state.mode !== "submitting",
    canClear: Boolean(state.review || state.errorMessage || state.validationErrors.length > 0 || state.draft.evidenceSummary),
    isBusy: state.mode === "submitting",
    lastReviewLabel: state.review ? `${state.review.id} · ${sanitizeMessage(state.review.evidence_summary)}` : null,
    errorMessage: state.errorMessage,
    validationErrors: state.validationErrors,
  };
}

export function buildTaskScopedInboxState(
  taskId: string | null,
  records: InboxNotificationRecord[],
  linkedRunIds: string[] = [],
): InboxDataState<InboxNotificationRecord> {
  if (!taskId) return { state: "ready", records: [] };
  const linkedRuns = new Set(linkedRunIds);
  return {
    state: "ready",
    records: records.filter((record) => record.task_id === taskId || Boolean(record.run_id && linkedRuns.has(record.run_id))),
  };
}

export function runHistoryRecordsForRun(records: HistoryTimelineRecord[], runId: string): HistoryTimelineRecord[] {
  return records.filter((record) => {
    const targets = [...record.event.targets, ...record.matched_entities];
    return targets.some((target) => target.entity_type === "run" && target.entity_id === runId);
  });
}

function validateManualReview(draft: ManualReviewDraft): string[] {
  const errors: string[] = [];
  if (!draft.taskId) errors.push("A selected task is required before recording review evidence.");
  const secretInput = secretLike(`${draft.evidenceSummary}\n${draft.requiredFixesJson}`);
  if (!draft.evidenceSummary.trim() || draft.evidenceSummary.trim().length < 12 || secretInput) errors.push("Evidence summary must describe what was reviewed without secret-looking input.");
  if (secretInput) errors.push("Manual review rejected secret-looking evidence before native invoke.");
  try {
    normalizeRequiredFixes(draft.requiredFixesJson);
  } catch {
    errors.push("Required fixes must be a JSON array.");
  }
  return errors;
}

function normalizeRequiredFixes(value: string): string {
  const parsed = JSON.parse(value.trim() || "[]") as unknown;
  if (!Array.isArray(parsed)) throw new Error("Required fixes must be a JSON array.");
  return JSON.stringify(parsed.map((item) => typeof item === "string" ? sanitizeMessage(item) : item));
}

function secretLike(value: string): boolean {
  return /secret|api[_-]?key|token|bearer|password/i.test(value);
}

function bridgeError(error: unknown): string {
  if (error instanceof Error) return sanitizeMessage(error.message);
  if (typeof error === "string") return sanitizeMessage(error);
  return "unknown manual review bridge error";
}

function humanize(value: string): string {
  return value.replace(/[_-]+/g, " ").replace(/\b\w/g, (letter) => letter.toUpperCase());
}
