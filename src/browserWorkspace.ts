export type BrowserTabRecord = { id: string; workspace_key?: string; profile_key?: string; url: string; title: string; state: "open"|"saved"|"closed"|"blocked"|"unsupported"; http_status?: number | null; opened_at?: string; updated_at?: string; closed_at?: string | null; manual_note?: string; metadata_json?: string };
export type BrowserCaptureRecord = { id: string; tab_id?: string | null; workspace_key?: string; profile_key?: string; url: string; title: string; captured_at: string; screenshot_path?: string | null; screenshot_supported: boolean; capture_mode: "screenshot"|"metadata_fallback"; http_status?: number | null; manual_note?: string; metadata_json?: string };
export type BrowserCaptureTarget = "launch_gate" | "task" | "note" | "product" | "content_piece";
export type BrowserCaptureLinkRecord = { capture_id: string; entity_type: BrowserCaptureTarget; entity_id: string; relation_type: string; created_at?: string };
export type WidgetSize = "small" | "medium" | "large";
export type WidgetConfigRecord = { widget_key: string; visible: boolean; position: number; size: WidgetSize; workspace_key: string; profile_key: string; updated_at?: string };
export type BrowserBridgeInvoke = <T = unknown>(command: string, args?: Record<string, unknown>) => Promise<T>;

export type BrowserWorkspaceDraft = {
  url: string;
  title: string;
  manualNote: string;
  entityType: BrowserCaptureTarget;
  entityId: string;
};

export type BrowserWorkspaceState =
  | { mode: "loading"; draft: BrowserWorkspaceDraft }
  | { mode: "error"; error: string; draft: BrowserWorkspaceDraft }
  | { mode: "ready"; tabs: BrowserTabRecord[]; captures: BrowserCaptureRecord[]; widgets: WidgetConfigRecord[]; selectedCaptureId: string | null; draft: BrowserWorkspaceDraft; message: string | null; errorMessage: string | null };

export const browserBridgeCommands = {
  openTab: "browser_open_tab_command",
  listTabs: "browser_list_tabs_command",
  updateTab: "browser_update_tab_command",
  createCapture: "browser_create_capture_command",
  listCaptures: "browser_list_captures_command",
  attachCapture: "browser_attach_capture_command",
  httpStatus: "browser_http_status_command",
  readWidgets: "widget_read_configs_command",
  updateWidget: "widget_update_config_command",
  resetWidgets: "widget_reset_configs_command",
} as const;

const DEFAULT_WORKSPACE = "browser";
const DEFAULT_PROFILE = "default";
const SECRET_QUERY_KEYS = ["token", "access_token", "refresh_token", "auth", "authorization", "password", "secret", "api_key", "apikey", "session", "cookie"];

export const allowedAttachmentTargets: BrowserCaptureTarget[] = ["launch_gate", "task", "note", "product", "content_piece"];
export const defaultWidgetOrder = ["today_tasks", "active_runs", "blockers", "completions", "browser_captures", "launch_gate_evidence", "content_queue"];

export function createInitialBrowserDraft(): BrowserWorkspaceDraft {
  return { url: "", title: "", manualNote: "", entityType: "launch_gate", entityId: "" };
}

export function createInitialBrowserWorkspaceState(): BrowserWorkspaceState {
  return { mode: "loading", draft: createInitialBrowserDraft() };
}

function bridgeErrorMessage(error: unknown): string {
  return error instanceof Error ? error.message : typeof error === "string" ? error : "unknown browser bridge error";
}

export function redactBrowserUrl(input: string): string {
  const trimmed = input.trim();
  const [base, query] = trimmed.split("?", 2);
  if (!query) return trimmed;
  const safeQuery = query.split("&").map((part) => {
    const [key] = part.split("=", 1);
    return SECRET_QUERY_KEYS.some((secret) => key.toLowerCase().includes(secret)) ? `${key}=[REDACTED]` : part;
  }).join("&");
  return `${base}?${safeQuery}`;
}

export function validateWorkUrl(url: string): string | null {
  const safe = redactBrowserUrl(url);
  if (!/^https?:\/\//.test(safe)) return "Browser workspace accepts http(s) work URLs only.";
  return null;
}

export function buildCapturePreview(input: { url: string; title: string; manualNote?: string; httpStatus?: number | null }): BrowserCaptureRecord {
  return { id: "preview", url: redactBrowserUrl(input.url), title: input.title.trim(), captured_at: new Date(0).toISOString(), screenshot_supported: false, screenshot_path: null, capture_mode: "metadata_fallback", http_status: input.httpStatus ?? null, manual_note: input.manualNote ?? "", metadata_json: "{}" };
}

export function captureEvidenceEligible(capture: Pick<BrowserCaptureRecord, "url"|"title"|"captured_at">): boolean {
  return /^https?:\/\//.test(capture.url) && capture.title.trim().length > 0 && capture.captured_at.trim().length > 0;
}

export function validateAttachmentTarget(target: string): target is BrowserCaptureTarget {
  return allowedAttachmentTargets.includes(target as BrowserCaptureTarget);
}

export function normalizeWidgetConfigs(configs: WidgetConfigRecord[]): WidgetConfigRecord[] {
  return configs.filter((w) => defaultWidgetOrder.includes(w.widget_key) && ["small", "medium", "large"].includes(w.size)).sort((a, b) => a.position - b.position);
}

export function moveWidget(configs: WidgetConfigRecord[], widgetKey: string, direction: "up" | "down"): WidgetConfigRecord[] {
  const ordered = normalizeWidgetConfigs(configs).map((w, index) => ({ ...w, position: index }));
  const index = ordered.findIndex((w) => w.widget_key === widgetKey);
  const swap = direction === "up" ? index - 1 : index + 1;
  if (index < 0 || swap < 0 || swap >= ordered.length) return ordered;
  [ordered[index], ordered[swap]] = [ordered[swap], ordered[index]];
  return ordered.map((w, position) => ({ ...w, position }));
}

export function resizeWidget(config: WidgetConfigRecord, size: WidgetSize): WidgetConfigRecord { return { ...config, size }; }
export function toggleWidget(config: WidgetConfigRecord): WidgetConfigRecord { return { ...config, visible: !config.visible }; }

export function updateBrowserDraft(state: BrowserWorkspaceState, patch: Partial<BrowserWorkspaceDraft>): BrowserWorkspaceState {
  return { ...state, draft: { ...state.draft, ...patch } };
}

function readyState(tabs: BrowserTabRecord[], captures: BrowserCaptureRecord[], widgets: WidgetConfigRecord[], draft: BrowserWorkspaceDraft, message: string | null = null): BrowserWorkspaceState {
  return { mode: "ready", tabs, captures, widgets: normalizeWidgetConfigs(widgets), selectedCaptureId: captures[0]?.id ?? null, draft, message, errorMessage: null };
}

export async function loadBrowserWorkspaceFromBridge(invoke: BrowserBridgeInvoke, draft: BrowserWorkspaceDraft = createInitialBrowserDraft()): Promise<BrowserWorkspaceState> {
  try {
    const [tabs, captures, widgets] = await Promise.all([
      invoke<BrowserTabRecord[]>(browserBridgeCommands.listTabs, { request: { workspace_key: DEFAULT_WORKSPACE, profile_key: DEFAULT_PROFILE, limit: 50 } }),
      invoke<BrowserCaptureRecord[]>(browserBridgeCommands.listCaptures, { request: { workspace_key: DEFAULT_WORKSPACE, profile_key: DEFAULT_PROFILE, limit: 50 } }),
      invoke<WidgetConfigRecord[]>(browserBridgeCommands.readWidgets, { workspaceKey: DEFAULT_WORKSPACE, profileKey: DEFAULT_PROFILE }),
    ]);
    return readyState(tabs, captures, widgets, draft);
  } catch (error) {
    return { mode: "error", error: bridgeErrorMessage(error), draft };
  }
}

export async function saveWorkUrlThroughBridge(invoke: BrowserBridgeInvoke, state: BrowserWorkspaceState): Promise<BrowserWorkspaceState> {
  const urlError = validateWorkUrl(state.draft.url);
  if (urlError) return { ...state, mode: "ready", tabs: state.mode === "ready" ? state.tabs : [], captures: state.mode === "ready" ? state.captures : [], widgets: state.mode === "ready" ? state.widgets : [], selectedCaptureId: state.mode === "ready" ? state.selectedCaptureId : null, message: null, errorMessage: urlError };
  try {
    await invoke<BrowserTabRecord>(browserBridgeCommands.openTab, { request: { workspace_key: DEFAULT_WORKSPACE, profile_key: DEFAULT_PROFILE, url: state.draft.url, title: state.draft.title || null, manual_note: state.draft.manualNote || null } });
    const loaded = await loadBrowserWorkspaceFromBridge(invoke, state.draft);
    return loaded.mode === "ready" ? { ...loaded, message: "Saved work URL metadata through the native browser bridge." } : loaded;
  } catch (error) {
    return state.mode === "ready" ? { ...state, errorMessage: bridgeErrorMessage(error), message: null } : { mode: "error", error: bridgeErrorMessage(error), draft: state.draft };
  }
}

export async function createCaptureThroughBridge(invoke: BrowserBridgeInvoke, state: BrowserWorkspaceState): Promise<BrowserWorkspaceState> {
  const urlError = validateWorkUrl(state.draft.url);
  if (urlError) return state.mode === "ready" ? { ...state, errorMessage: urlError, message: null } : { mode: "error", error: urlError, draft: state.draft };
  if (!state.draft.title.trim()) return state.mode === "ready" ? { ...state, errorMessage: "Capture metadata requires a title before it can become evidence.", message: null } : { mode: "error", error: "Capture metadata requires a title before it can become evidence.", draft: state.draft };
  try {
    const httpStatus = await invoke<number | null>(browserBridgeCommands.httpStatus, { url: state.draft.url }).catch(() => null);
    const tabId = state.mode === "ready" ? state.tabs.find((tab) => tab.url === redactBrowserUrl(state.draft.url) || tab.url === state.draft.url)?.id ?? null : null;
    await invoke<BrowserCaptureRecord>(browserBridgeCommands.createCapture, { request: { tab_id: tabId, workspace_key: DEFAULT_WORKSPACE, profile_key: DEFAULT_PROFILE, url: state.draft.url, title: state.draft.title, http_status: httpStatus, manual_note: state.draft.manualNote || null, metadata_json: "{}" } });
    const loaded = await loadBrowserWorkspaceFromBridge(invoke, state.draft);
    return loaded.mode === "ready" ? { ...loaded, message: "Created metadata-fallback capture. Screenshot remains unsupported unless native capture is proven." } : loaded;
  } catch (error) {
    return state.mode === "ready" ? { ...state, errorMessage: bridgeErrorMessage(error), message: null } : { mode: "error", error: bridgeErrorMessage(error), draft: state.draft };
  }
}

export async function attachCaptureThroughBridge(invoke: BrowserBridgeInvoke, state: BrowserWorkspaceState): Promise<BrowserWorkspaceState> {
  if (state.mode !== "ready") return state;
  const captureId = state.selectedCaptureId ?? state.captures[0]?.id;
  if (!captureId) return { ...state, errorMessage: "Create or select a capture before attaching evidence.", message: null };
  if (!validateAttachmentTarget(state.draft.entityType) || !state.draft.entityId.trim()) return { ...state, errorMessage: "Choose a supported target type and enter a real entity id before attaching evidence.", message: null };
  try {
    await invoke<BrowserCaptureLinkRecord>(browserBridgeCommands.attachCapture, { request: { capture_id: captureId, entity_type: state.draft.entityType, entity_id: state.draft.entityId.trim(), relation_type: "evidence" } });
    const loaded = await loadBrowserWorkspaceFromBridge(invoke, state.draft);
    return loaded.mode === "ready" ? { ...loaded, selectedCaptureId: captureId, message: `Attached capture ${captureId} as evidence for ${state.draft.entityType}.` } : loaded;
  } catch (error) {
    return { ...state, errorMessage: bridgeErrorMessage(error), message: null };
  }
}

export async function updateWidgetThroughBridge(invoke: BrowserBridgeInvoke, state: BrowserWorkspaceState, widget: WidgetConfigRecord): Promise<BrowserWorkspaceState> {
  if (state.mode !== "ready") return state;
  try {
    await invoke<WidgetConfigRecord>(browserBridgeCommands.updateWidget, { request: { workspace_key: widget.workspace_key || DEFAULT_WORKSPACE, profile_key: widget.profile_key || DEFAULT_PROFILE, widget_key: widget.widget_key, visible: widget.visible, position: widget.position, size: widget.size } });
    const loaded = await loadBrowserWorkspaceFromBridge(invoke, state.draft);
    return loaded.mode === "ready" ? { ...loaded, message: "Persisted widget layout through the native widget bridge." } : loaded;
  } catch (error) {
    return { ...state, errorMessage: bridgeErrorMessage(error), message: null };
  }
}

export async function resetWidgetsThroughBridge(invoke: BrowserBridgeInvoke, state: BrowserWorkspaceState): Promise<BrowserWorkspaceState> {
  try {
    const widgets = await invoke<WidgetConfigRecord[]>(browserBridgeCommands.resetWidgets, { workspaceKey: DEFAULT_WORKSPACE, profileKey: DEFAULT_PROFILE });
    return state.mode === "ready" ? { ...state, widgets: normalizeWidgetConfigs(widgets), message: "Reset widgets through the native widget bridge.", errorMessage: null } : readyState([], [], widgets, state.draft, "Reset widgets through the native widget bridge.");
  } catch (error) {
    return state.mode === "ready" ? { ...state, errorMessage: bridgeErrorMessage(error), message: null } : { mode: "error", error: bridgeErrorMessage(error), draft: state.draft };
  }
}
