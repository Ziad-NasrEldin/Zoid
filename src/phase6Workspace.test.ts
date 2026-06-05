import {
  assertPhase6CalendarConfirmation,
  assertPhase6NoSilentSend,
  buildPhase6WorkspaceView,
  loadPhase6OverviewFromBridge,
  phase6SafeBridgeError,
  type Phase6OverviewRecord,
} from "./phase6Workspace";

function expect(condition: boolean, message: string) {
  if (!condition) throw new Error(message);
}

const overview: Phase6OverviewRecord = {
  integrations: [
    { key: "eventkit", state: "needs_permission", safe_copy: "Calendar access needs macOS permission before sync can run." },
    { key: "gmail", state: "not_configured", safe_copy: "Mail is unconfigured and safe." },
  ],
  inbox: [{ id: "i1", item_type: "email", title: "Draft", detail: "confirm", state: "draft", priority: "high" }],
  calendar: [{ id: "c1", title: "Review", starts_at: "2026-06-05T09:00", ends_at: "2026-06-05T10:00", state: "created" }],
  emails: [{ id: "e1", subject: "Proposal", recipients_json: "[]", state: "draft" }],
  companies: [{ id: "co1", name: "MaVoid", status: "active" }],
  contacts: [{ id: "ct1", full_name: "Ziad", status: "active" }],
  follow_ups: [{ id: "f1", subject: "Call", state: "open", priority: "high" }],
  products: [{ id: "p1", name: "Zoid", status: "active" }],
  product_links: [{ id: "l1", source_type: "product", source_id: "p1", target_type: "task", target_id: "t1", relation_type: "ships" }],
};

const readyView = buildPhase6WorkspaceView("business", { mode: "ready", overview });
expect(readyView.title === "Business", "business workspace must render Phase 6 Business title");
expect(readyView.sections.find((section) => section.key === "business")?.count === 3, "business count must include companies, contacts, and follow-ups");
expect(readyView.blockers.some((copy) => copy.includes("Calendar access")), "integration states must render safe permission blockers");

expect(!assertPhase6NoSilentSend("send_email_draft_command", { request: {} }), "Gmail send must fail closed without confirmation_id");
expect(assertPhase6NoSilentSend("send_email_draft_command", { request: { confirmation_id: "confirm-1" } }), "Gmail send with confirmation_id should be allowed through UI guard");
expect(!assertPhase6CalendarConfirmation("create_calendar_event_command", { request: {} }), "calendar create must fail closed without confirmation_id");
expect(assertPhase6CalendarConfirmation("delete_calendar_event_command", { request: { confirmation_id: "confirm-2" } }), "calendar delete with confirmation_id should be allowed through UI guard");

const loaded = await loadPhase6OverviewFromBridge(<T,>(command: string) => {
  expect(command === "get_phase6_overview_command", "Phase 6 workspace must load through native overview bridge command");
  return Promise.resolve(overview as T);
});
expect(loaded.mode === "ready", "successful native overview load must produce ready state");

const errored = await loadPhase6OverviewFromBridge(async () => {
  throw new Error("Cannot read properties of undefined (reading 'invoke')");
});
expect(errored.mode === "error", "native overview failure must produce safe error state");
expect(errored.mode === "error" && errored.error.includes("Tauri desktop app"), "browser preview bridge failures must use safe user-facing copy");
expect(!phase6SafeBridgeError(new Error("Cannot read properties of undefined (reading 'invoke')")).includes("undefined"), "safe bridge error must not leak JavaScript implementation details");
