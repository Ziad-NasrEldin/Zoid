import { sanitizeMessage } from "./historyTimelineViewModel";

export type RunControlsInvoke = <T = unknown>(command: string, args?: Record<string, unknown>) => Promise<T>;

export type RunControlsRunRecord = {
  id: string;
  task_id?: string;
  status?: string;
  output_summary?: string | null;
  error_summary?: string | null;
  [key: string]: unknown;
};

export type RunControlsOutcome = {
  session_id: string;
  log_path: string;
  run: RunControlsRunRecord;
};

export type RunControlsDraft = {
  taskId: string | null;
  profileId: string;
  cwd: string;
  argvText: string;
  stdin: string;
  timeoutMsText: string;
  metadataJson: string;
};

export type RunControlsState = {
  mode: "idle" | "starting" | "ready" | "cancelling" | "blocked" | "error";
  draft: RunControlsDraft;
  activeRun: RunControlsRunRecord | null;
  lastMessage: string | null;
  errorMessage: string | null;
  validationErrors: string[];
};

export type RunControlsViewModel = {
  statusLabel: string;
  statusTone: "ready" | "running" | "blocked" | "pending";
  canStart: boolean;
  canCancel: boolean;
  canClear: boolean;
  isBusy: boolean;
  commandPreview: string;
  lastMessage: string | null;
  errorMessage: string | null;
  validationErrors: string[];
};

export type RunControlsDefaults = {
  taskId: string | null;
  profileId?: string;
  cwd?: string;
};

export type RunControlsDraftPatch = Partial<RunControlsDraft> & { clearStatus?: boolean };

export function createInitialRunControlsState(defaults: RunControlsDefaults): RunControlsState {
  return {
    mode: "idle",
    draft: {
      taskId: defaults.taskId,
      profileId: defaults.profileId ?? "default",
      cwd: defaults.cwd ?? "",
      argvText: "",
      stdin: "",
      timeoutMsText: "120000",
      metadataJson: "{}",
    },
    activeRun: null,
    lastMessage: null,
    errorMessage: null,
    validationErrors: [],
  };
}

export function updateRunControlsDraft(state: RunControlsState, patch: RunControlsDraftPatch): RunControlsState {
  const next = patch.clearStatus
    ? {
        ...state,
        mode: "idle" as const,
        activeRun: null,
        lastMessage: null,
        errorMessage: null,
        validationErrors: [],
      }
    : state;
  const { clearStatus: _clearStatus, ...draftPatch } = patch;
  return { ...next, draft: { ...next.draft, ...draftPatch } };
}

export function resetRunControlsForTask(state: RunControlsState, taskId: string | null, cwd: string): RunControlsState {
  return updateRunControlsDraft(state, {
    taskId,
    cwd: state.draft.cwd || cwd,
    clearStatus: state.draft.taskId !== taskId,
  });
}

export async function startRunThroughBridge(
  invoke: RunControlsInvoke,
  state: RunControlsState,
  options: { logsDir?: string | null },
): Promise<RunControlsState> {
  const validationErrors = validateStart(state.draft, options.logsDir);
  if (validationErrors.length > 0) {
    return { ...state, mode: "blocked", errorMessage: validationErrors.join(" "), validationErrors };
  }

  try {
    const outcome = await invoke<RunControlsOutcome>("start_agent_run_command", {
      request: {
        task_id: state.draft.taskId,
        profile_id: state.draft.profileId.trim(),
        cwd: state.draft.cwd.trim(),
        argv: parseArgv(state.draft.argvText),
        stdin: state.draft.stdin.trim() ? state.draft.stdin : null,
        timeout_ms: parseTimeout(state.draft.timeoutMsText),
        logs_dir: options.logsDir,
        metadata_json: normalizeMetadata(state.draft.metadataJson),
      },
    });
    return {
      ...state,
      mode: "ready",
      activeRun: outcome.run,
      lastMessage: sanitizeMessage(outcome.run.output_summary || `Run ${outcome.run.id} started`),
      errorMessage: null,
      validationErrors: [],
    };
  } catch (error) {
    return { ...state, mode: "error", errorMessage: bridgeError(error), validationErrors: [] };
  }
}

export async function cancelRunThroughBridge(
  invoke: RunControlsInvoke,
  state: RunControlsState,
  reason = "Cancelled from Zoid task detail",
): Promise<RunControlsState> {
  if (!state.activeRun?.id) {
    return { ...state, mode: "blocked", errorMessage: "No active run is available to cancel.", validationErrors: ["No active run is available to cancel."] };
  }

  try {
    const run = await invoke<RunControlsRunRecord>("cancel_run_command", {
      runId: state.activeRun.id,
      request: {
        reason,
        metadata_json: JSON.stringify({ source: "task_detail_run_controls" }),
      },
    });
    return {
      ...state,
      mode: "ready",
      activeRun: run,
      lastMessage: sanitizeMessage(run.output_summary || "Run cancellation requested."),
      errorMessage: null,
      validationErrors: [],
    };
  } catch (error) {
    return { ...state, mode: "error", errorMessage: bridgeError(error), validationErrors: [] };
  }
}

export function createRunControlsViewModel(state: RunControlsState): RunControlsViewModel {
  const status = state.activeRun?.status ?? state.mode;
  const busy = state.mode === "starting" || state.mode === "cancelling";
  return {
    statusLabel: humanStatus(status),
    statusTone: toneForStatus(status),
    canStart: !busy,
    canCancel: Boolean(state.activeRun?.id && !isTerminalStatus(state.activeRun.status)),
    canClear: Boolean(state.activeRun || state.errorMessage || state.validationErrors.length > 0 || state.lastMessage),
    isBusy: busy,
    commandPreview: parseArgv(state.draft.argvText).join(" "),
    lastMessage: state.lastMessage,
    errorMessage: state.errorMessage,
    validationErrors: state.validationErrors,
  };
}

function validateStart(draft: RunControlsDraft, logsDir?: string | null): string[] {
  const errors: string[] = [];
  if (!draft.taskId) errors.push("A selected task is required before starting a run.");
  if (!draft.profileId.trim()) errors.push("Profile is required.");
  if (!draft.cwd.trim()) errors.push("Working directory is required.");
  if (parseArgv(draft.argvText).length === 0) errors.push("Command arguments are required.");
  if (!logsDir) errors.push("Native logs directory is unavailable; run output cannot be persisted truthfully.");
  const timeout = parseTimeout(draft.timeoutMsText);
  if (timeout !== null && (timeout < 1000 || timeout > 86_400_000)) errors.push("Timeout must be between 1000 and 86400000 ms.");
  if (secretLike(`${draft.argvText}\n${draft.stdin}\n${draft.metadataJson}`)) errors.push("Run controls rejected secret-looking input before native invoke.");
  try {
    normalizeMetadata(draft.metadataJson);
  } catch (error) {
    errors.push(bridgeError(error));
  }
  return errors;
}

function parseArgv(argvText: string): string[] {
  return argvText
    .split(/\r?\n|\s+/)
    .map((part) => part.trim())
    .filter(Boolean);
}

function parseTimeout(timeoutMsText: string): number | null {
  const value = timeoutMsText.trim();
  if (!value) return null;
  const parsed = Number(value);
  return Number.isFinite(parsed) ? Math.trunc(parsed) : -1;
}

function normalizeMetadata(metadataJson: string): string {
  const value = metadataJson.trim() || "{}";
  const parsed = JSON.parse(value) as unknown;
  if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) throw new Error("Metadata must be a JSON object.");
  return JSON.stringify(parsed);
}

function secretLike(value: string): boolean {
  return /secret|api[_-]?key|token|bearer|password/i.test(value);
}

function isTerminalStatus(status?: string): boolean {
  return ["completed", "failed", "cancelled", "blocked", "review_required"].includes(String(status ?? "").toLowerCase());
}

function humanStatus(status: string): string {
  if (status === "blocked") return "Blocked";
  if (status === "error") return "Error";
  if (status === "idle") return "Idle";
  return status
    .split("_")
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ") || "Idle";
}

function toneForStatus(status: string): RunControlsViewModel["statusTone"] {
  const normalized = status.toLowerCase();
  if (["completed", "success"].includes(normalized)) return "ready";
  if (["failed", "cancelled", "blocked", "error"].includes(normalized)) return "blocked";
  if (["running", "starting", "cancelling", "queued", "waiting_for_input"].includes(normalized)) return "running";
  return "pending";
}

function bridgeError(error: unknown): string {
  if (error instanceof Error) return sanitizeMessage(error.message);
  if (typeof error === "string") return sanitizeMessage(error);
  return "unknown run controls bridge error";
}
