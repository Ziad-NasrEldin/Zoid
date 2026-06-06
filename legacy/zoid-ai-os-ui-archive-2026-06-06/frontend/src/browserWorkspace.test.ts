import {
  attachCaptureThroughBridge,
  browserBridgeCommands,
  buildCapturePreview,
  captureEvidenceEligible,
  createCaptureThroughBridge,
  loadBrowserWorkspaceFromBridge,
  moveWidget,
  normalizeWidgetConfigs,
  redactBrowserUrl,
  resetWidgetsThroughBridge,
  resizeWidget,
  saveWorkUrlThroughBridge,
  toggleWidget,
  updateWidgetThroughBridge,
  validateAttachmentTarget,
  validateWorkUrl,
  type BrowserBridgeInvoke,
  type BrowserCaptureRecord,
  type BrowserTabRecord,
  type WidgetConfigRecord,
} from "./browserWorkspace";

const assertEqual = <T>(actual: T, expected: T, message: string) => {
  if (actual !== expected) {
    throw new Error(`${message}: expected ${String(expected)}, got ${String(actual)}`);
  }
};

const assertMatch = (actual: string, expected: RegExp, message: string) => {
  if (!expected.test(actual)) {
    throw new Error(`${message}: ${actual}`);
  }
};

const assertDeepEqual = <T>(actual: T, expected: T, message: string) => {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) {
    throw new Error(`${message}: expected ${JSON.stringify(expected)}, got ${JSON.stringify(actual)}`);
  }
};

const widget = (widget_key: string, position: number, visible = true, size: "small"|"medium"|"large" = "medium"): WidgetConfigRecord => ({ workspace_key: "browser", profile_key: "default", widget_key, position, visible, size });
const tab: BrowserTabRecord = { id: "tab-1", workspace_key: "browser", profile_key: "default", url: "https://docs.example", title: "Docs", state: "open", http_status: null, manual_note: "" };
const capture: BrowserCaptureRecord = { id: "cap-1", tab_id: "tab-1", workspace_key: "browser", profile_key: "default", url: "https://docs.example", title: "Docs", captured_at: "2026-06-05T00:00:00Z", screenshot_supported: false, screenshot_path: null, capture_mode: "metadata_fallback", http_status: null, manual_note: "", metadata_json: "{}" };

function makeInvoke(responses: Record<string, unknown[]>) {
  const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
  const invoke: BrowserBridgeInvoke = async <T = unknown>(command: string, args?: Record<string, unknown>) => {
    calls.push({ command, args });
    const queue = responses[command] ?? [];
    if (queue.length === 0) throw new Error(`unexpected command ${command}`);
    const response = queue.shift();
    if (response instanceof Error) throw response;
    return response as T;
  };
  return { invoke, calls };
}

assertEqual(redactBrowserUrl("https://example.com/a?token=abc&ok=1&password=x"), "https://example.com/a?token=[REDACTED]&ok=1&password=[REDACTED]", "secret query params must be redacted");
assertMatch(validateWorkUrl("file:///etc/passwd") ?? "", /http\(s\)/, "non-http URLs must be rejected");
const preview = buildCapturePreview({ url: "https://docs.example/?api_key=abc", title: "Docs", manualNote: "verified" });
assertEqual(preview.capture_mode, "metadata_fallback", "capture preview must use metadata fallback");
assertEqual(preview.screenshot_supported, false, "screenshot capture must not be claimed supported");
assertMatch(preview.url, /api_key=\[REDACTED\]/, "capture preview URL must be redacted");
assertEqual(captureEvidenceEligible(preview), true, "valid metadata capture should be evidence eligible");
assertEqual(captureEvidenceEligible({ ...preview, title: "" }), false, "capture without title should not be evidence eligible");
assertEqual(validateAttachmentTarget("launch_gate"), true, "launch gate target should be allowed");
assertEqual(validateAttachmentTarget("cookie_jar"), false, "cookie jar target should be rejected");
const configs = [widget("blockers", 2), widget("today_tasks", 0), widget("unknown", 1)];
assertDeepEqual(normalizeWidgetConfigs(configs).map((w) => w.widget_key), ["today_tasks", "blockers"], "widget configs should be filtered and sorted");
assertEqual(toggleWidget(configs[0]).visible, false, "widget visibility should toggle");
assertEqual(resizeWidget(configs[0], "large").size, "large", "widget size should change");
assertDeepEqual(moveWidget([widget("today_tasks", 0), widget("active_runs", 1)], "active_runs", "up").map((w) => w.widget_key), ["active_runs", "today_tasks"], "widget move up should reorder widgets");

{
  const { invoke, calls } = makeInvoke({
    [browserBridgeCommands.listTabs]: [[tab]],
    [browserBridgeCommands.listCaptures]: [[capture]],
    [browserBridgeCommands.readWidgets]: [[widget("browser_captures", 0)]],
  });
  const state = await loadBrowserWorkspaceFromBridge(invoke);
  assertEqual(state.mode, "ready", "browser bridge load should return ready state");
  assertEqual(calls[0]?.command, browserBridgeCommands.listTabs, "load invokes native tab list command");
  assertEqual(calls[1]?.command, browserBridgeCommands.listCaptures, "load invokes native capture list command");
  assertEqual(calls[2]?.command, browserBridgeCommands.readWidgets, "load invokes native widget read command");
  assertEqual((calls[0]?.args?.request as Record<string, unknown>).workspace_key, "browser", "tab list is scoped to browser workspace");
}

{
  const { invoke, calls } = makeInvoke({
    [browserBridgeCommands.openTab]: [tab],
    [browserBridgeCommands.listTabs]: [[tab]],
    [browserBridgeCommands.listCaptures]: [[capture]],
    [browserBridgeCommands.readWidgets]: [[widget("browser_captures", 0)]],
  });
  const state = await saveWorkUrlThroughBridge(invoke, { mode: "ready", tabs: [], captures: [], widgets: [], selectedCaptureId: null, draft: { url: "https://docs.example", title: "Docs", manualNote: "safe note", entityType: "launch_gate", entityId: "gate-1" }, message: null, errorMessage: null });
  assertEqual(state.mode, "ready", "save URL should reload ready state");
  assertEqual(calls[0]?.command, browserBridgeCommands.openTab, "save URL invokes native open tab command");
  assertEqual(((calls[0]?.args?.request as Record<string, unknown>).manual_note), "safe note", "manual note is sent through request for backend redaction/persistence");
}

{
  const { invoke, calls } = makeInvoke({
    [browserBridgeCommands.httpStatus]: [null],
    [browserBridgeCommands.createCapture]: [capture],
    [browserBridgeCommands.listTabs]: [[tab]],
    [browserBridgeCommands.listCaptures]: [[capture]],
    [browserBridgeCommands.readWidgets]: [[widget("browser_captures", 0)]],
  });
  const state = await createCaptureThroughBridge(invoke, { mode: "ready", tabs: [tab], captures: [], widgets: [], selectedCaptureId: null, draft: { url: "https://docs.example", title: "Docs", manualNote: "safe", entityType: "launch_gate", entityId: "gate-1" }, message: null, errorMessage: null });
  assertEqual(state.mode, "ready", "capture create should reload ready state");
  assertEqual(calls[0]?.command, browserBridgeCommands.httpStatus, "capture checks native HTTP status truthfully");
  assertEqual(calls[1]?.command, browserBridgeCommands.createCapture, "capture invokes native create capture command");
  assertEqual((calls[1]?.args?.request as Record<string, unknown>).tab_id, "tab-1", "capture links to matching tab when available");
}

{
  const { invoke, calls } = makeInvoke({
    [browserBridgeCommands.attachCapture]: [{ capture_id: "cap-1", entity_type: "launch_gate", entity_id: "gate-1", relation_type: "evidence" }],
    [browserBridgeCommands.listTabs]: [[tab]],
    [browserBridgeCommands.listCaptures]: [[capture]],
    [browserBridgeCommands.readWidgets]: [[widget("browser_captures", 0)]],
  });
  const state = await attachCaptureThroughBridge(invoke, { mode: "ready", tabs: [tab], captures: [capture], widgets: [], selectedCaptureId: "cap-1", draft: { url: "https://docs.example", title: "Docs", manualNote: "safe", entityType: "launch_gate", entityId: "gate-1" }, message: null, errorMessage: null });
  assertEqual(state.mode, "ready", "attach should reload ready state");
  assertEqual(calls[0]?.command, browserBridgeCommands.attachCapture, "attach invokes native capture attachment command");
  assertEqual((calls[0]?.args?.request as Record<string, unknown>).relation_type, "evidence", "capture attachment uses evidence relation");
}

{
  const { invoke, calls } = makeInvoke({
    [browserBridgeCommands.updateWidget]: [widget("browser_captures", 0, false)],
    [browserBridgeCommands.listTabs]: [[tab]],
    [browserBridgeCommands.listCaptures]: [[capture]],
    [browserBridgeCommands.readWidgets]: [[widget("browser_captures", 0, false)]],
  });
  const state = await updateWidgetThroughBridge(invoke, { mode: "ready", tabs: [], captures: [], widgets: [widget("browser_captures", 0)], selectedCaptureId: null, draft: { url: "https://docs.example", title: "Docs", manualNote: "safe", entityType: "launch_gate", entityId: "gate-1" }, message: null, errorMessage: null }, widget("browser_captures", 0, false));
  assertEqual(state.mode, "ready", "widget update reloads ready state");
  assertEqual(calls[0]?.command, browserBridgeCommands.updateWidget, "widget update invokes native persistence command");
}

{
  const { invoke, calls } = makeInvoke({ [browserBridgeCommands.resetWidgets]: [[widget("browser_captures", 0)]] });
  const state = await resetWidgetsThroughBridge(invoke, { mode: "ready", tabs: [], captures: [], widgets: [], selectedCaptureId: null, draft: { url: "https://docs.example", title: "Docs", manualNote: "safe", entityType: "launch_gate", entityId: "gate-1" }, message: null, errorMessage: null });
  assertEqual(state.mode, "ready", "widget reset returns ready state");
  assertEqual(calls[0]?.command, browserBridgeCommands.resetWidgets, "reset invokes native reset command");
}

console.log("browserWorkspace tests passed");
