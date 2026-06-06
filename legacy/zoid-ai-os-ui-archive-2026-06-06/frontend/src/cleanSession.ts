import { sanitizeMessage } from "./historyTimelineViewModel";

export type CleanSessionInvoke = <T = unknown>(command: string, args?: Record<string, unknown>) => Promise<T>;

export type CleanSessionRunRecord = {
  id: string;
  status?: string;
  output_summary?: string | null;
  error_summary?: string | null;
  updated_at?: string | null;
  [key: string]: unknown;
};

export type CleanSessionStreamChunk = {
  run_id: string;
  log_reference_id: string;
  offset: number;
  next_offset: number;
  eof: boolean;
  status: string;
  content: string;
};

export type CleanSessionState =
  | { mode: "idle"; runId: string | null }
  | { mode: "loading"; runId: string }
  | { mode: "unavailable"; runId: string; reason: string }
  | { mode: "error"; runId: string; error: string }
  | { mode: "ready"; runId: string; run: CleanSessionRunRecord; chunk: CleanSessionStreamChunk };

export type CleanSessionCard = {
  id: string;
  kind: "command" | "success" | "error" | "status" | "output";
  title: string;
  body: string;
};

export type CleanSessionViewModel = {
  runId: string | null;
  statusLabel: string;
  statusTone: "ready" | "running" | "blocked" | "pending";
  summary: string;
  emptyCopy: string;
  isLoading: boolean;
  errorMessage: string | null;
  cards: CleanSessionCard[];
  nextOffset: number | null;
  eof: boolean;
};

export type CleanSessionStreamRequest = {
  runId: string;
  logsDir?: string | null;
  offset?: number;
  maxBytes?: number;
};

export async function loadCleanSessionStreamFromBridge(
  invoke: CleanSessionInvoke,
  request: CleanSessionStreamRequest,
): Promise<CleanSessionState> {
  if (!request.logsDir) {
    return { mode: "unavailable", runId: request.runId, reason: "logs directory is not available to the frontend" };
  }

  try {
    const run = await invoke<CleanSessionRunRecord>("read_run_status_command", { runId: request.runId });
    const chunk = await invoke<CleanSessionStreamChunk>("stream_run_output_command", {
      request: {
        run_id: request.runId,
        logs_dir: request.logsDir,
        offset: request.offset ?? 0,
        max_bytes: request.maxBytes ?? 4096,
      },
    });
    return { mode: "ready", runId: request.runId, run, chunk };
  } catch (error) {
    return { mode: "error", runId: request.runId, error: bridgeError(error) };
  }
}

export function nextCleanSessionOffset(state: CleanSessionState | undefined): number {
  if (state?.mode === "ready" && !state.chunk.eof) return state.chunk.next_offset;
  return 0;
}

export function appendCleanSessionChunk(previous: CleanSessionState | undefined, next: CleanSessionState): CleanSessionState {
  if (previous?.mode !== "ready" || next.mode !== "ready" || next.chunk.offset === 0) return next;
  return {
    ...next,
    chunk: {
      ...next.chunk,
      offset: previous.chunk.offset,
      content: [previous.chunk.content, next.chunk.content].filter((part) => part.trim().length > 0).join("\n"),
    },
  };
}

export function createCleanSessionViewModel(state: CleanSessionState): CleanSessionViewModel {
  const runId = state.runId;
  if (state.mode === "idle") return emptyView(runId, "Select a run to load clean session output.");
  if (state.mode === "loading") {
    return {
      ...emptyView(runId, "Loading clean session output from native run logs."),
      statusLabel: "Loading",
      statusTone: "pending",
      isLoading: true,
    };
  }
  if (state.mode === "unavailable") {
    return {
      ...emptyView(runId, "No terminal output is simulated. Native log streaming needs a configured log directory."),
      statusLabel: "Unavailable",
      statusTone: "pending",
      summary: sanitizeMessage(state.reason),
    };
  }
  if (state.mode === "error") {
    return {
      ...emptyView(runId, "No output cards are fabricated when the native stream fails."),
      statusLabel: "Error",
      statusTone: "blocked",
      summary: "Native run stream failed.",
      errorMessage: sanitizeMessage(state.error),
    };
  }

  const status = String(state.run.status ?? state.chunk.status ?? "unknown");
  const summary = sanitizeMessage(
    String(state.run.output_summary || state.run.error_summary || `${humanStatus(status)} run output`),
  );
  return {
    runId,
    statusLabel: humanStatus(status),
    statusTone: toneForStatus(status),
    summary,
    emptyCopy: state.chunk.content.trim() ? "" : "No output has been written to this persisted run log yet.",
    isLoading: false,
    errorMessage: null,
    cards: buildCleanCards(state.chunk.content),
    nextOffset: state.chunk.next_offset,
    eof: state.chunk.eof,
  };
}

function emptyView(runId: string | null, emptyCopy: string): CleanSessionViewModel {
  return {
    runId,
    statusLabel: "Idle",
    statusTone: "pending",
    summary: "Clean session output is waiting for native run data.",
    emptyCopy,
    isLoading: false,
    errorMessage: null,
    cards: [],
    nextOffset: null,
    eof: false,
  };
}

function buildCleanCards(content: string): CleanSessionCard[] {
  return content
    .split(/\r?\n/)
    .map((line) => sanitizeMessage(line.trim()))
    .filter((line) => line.length > 0)
    .slice(-12)
    .map((line, index) => {
      const kind = classifyLine(line);
      return {
        id: `${index}-${kind}`,
        kind,
        title: titleFor(kind, line),
        body: line,
      };
    });
}

function classifyLine(line: string): CleanSessionCard["kind"] {
  const lower = line.toLowerCase();
  if (/^(npm|pnpm|yarn|cargo|git|node|python|tsx|tauri)\b/.test(lower) || lower.startsWith("$ ")) return "command";
  if (lower.includes("error") || lower.includes("failed") || lower.includes("panic")) return "error";
  if (line.startsWith("✓") || lower.includes("success") || lower.includes("ready") || lower.includes("passed")) return "success";
  if (lower.includes("running") || lower.includes("starting") || lower.includes("waiting")) return "status";
  return "output";
}

function titleFor(kind: CleanSessionCard["kind"], line: string): string {
  if (kind === "command") return `Command · ${line.replace(/^\$\s*/, "").slice(0, 48)}`;
  if (kind === "success") return "Success";
  if (kind === "error") return "Issue";
  if (kind === "status") return "Status";
  return "Output";
}

function humanStatus(status: string): string {
  return status
    .split("_")
    .filter(Boolean)
    .map((part) => part.charAt(0).toUpperCase() + part.slice(1))
    .join(" ") || "Unknown";
}

function toneForStatus(status: string): CleanSessionViewModel["statusTone"] {
  const normalized = status.toLowerCase();
  if (["completed", "success"].includes(normalized)) return "ready";
  if (["failed", "cancelled", "blocked"].includes(normalized)) return "blocked";
  if (["running", "starting", "queued", "waiting_for_input", "review_required"].includes(normalized)) return "running";
  return "pending";
}

function bridgeError(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "unknown clean session bridge error";
}
