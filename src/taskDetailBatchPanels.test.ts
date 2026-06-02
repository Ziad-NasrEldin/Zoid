import {
  buildTaskScopedInboxState,
  createInitialManualReviewState,
  createManualReviewThroughBridge,
  runHistoryRecordsForRun,
  updateManualReviewDraft,
  type ManualReviewInvoke,
} from "./taskDetailBatchPanels";
import type { InboxNotificationRecord } from "./inboxViewModel";
import type { HistoryTimelineRecord } from "./historyTimelineViewModel";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function notification(overrides: Partial<InboxNotificationRecord>): InboxNotificationRecord {
  return {
    id: overrides.id ?? "n-1",
    notification_type: overrides.notification_type ?? "review_required",
    title: overrides.title ?? "Review required",
    message: overrides.message ?? "Needs review",
    severity: overrides.severity ?? "warning",
    state: overrides.state ?? "action_required",
    action_route: overrides.action_route ?? null,
    task_id: overrides.task_id ?? null,
    run_id: overrides.run_id ?? null,
    review_record_id: overrides.review_record_id ?? null,
    read_at: overrides.read_at ?? null,
    dismissed_at: overrides.dismissed_at ?? null,
    resolved_at: overrides.resolved_at ?? null,
    created_at: overrides.created_at ?? "2026-06-02T10:00:00.000Z",
    updated_at: overrides.updated_at ?? "2026-06-02T10:00:00.000Z",
    metadata_json: overrides.metadata_json ?? "{}",
  };
}

function historyRecord(eventId: string, targets: Array<{ entity_type: string; entity_id: string; relation_type: string }>): HistoryTimelineRecord {
  return {
    event: {
      id: eventId,
      action_type: "run_status_changed",
      outcome: "success",
      timestamp: `2026-06-02T10:0${eventId.slice(-1)}:00.000Z`,
      summary: "Run event persisted",
      source: "agent_run_service",
      targets,
    },
    matched_entities: targets,
  };
}

async function testManualReviewCreateUsesNativeBridgeShape() {
  const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
  const invoke: ManualReviewInvoke = async <T = unknown>(command: string, args?: Record<string, unknown>) => {
    calls.push({ command, args });
    return {
      id: "review-1",
      task_id: "task-1",
      run_id: "run-1",
      verdict: "approved",
      evidence_summary: "Reviewed evidence from clean session cards.",
      required_fixes_json: "[]",
    } as T;
  };

  let state = createInitialManualReviewState("task-1", "run-1");
  state = updateManualReviewDraft(state, { verdict: "approved", evidenceSummary: "Reviewed evidence from clean session cards." });
  state = await createManualReviewThroughBridge(invoke, state);

  assert(state.mode === "ready", "manual review creation should enter ready mode");
  assert(calls[0].command === "create_manual_review_command", "should call native manual review command");
  assert(JSON.stringify(calls[0].args).includes('"task_id":"task-1"'), "request should include task_id");
  assert(JSON.stringify(calls[0].args).includes('"run_id":"run-1"'), "request should include run_id");
  assert(JSON.stringify(calls[0].args).includes('"required_fixes_json":"[]"'), "request should include required_fixes_json");
}

async function testManualReviewBlocksWeakEvidenceAndSecrets() {
  const invoke: ManualReviewInvoke = async () => {
    throw new Error("should not invoke invalid review");
  };
  let state = createInitialManualReviewState("task-1", null);
  state = updateManualReviewDraft(state, { evidenceSummary: "api_key=SECRET_TOKEN_VALUE", requiredFixesJson: "not-json" });
  state = await createManualReviewThroughBridge(invoke, state);

  assert(state.mode === "blocked", "invalid manual review should be blocked locally");
  assert(state.errorMessage?.includes("Evidence summary"), "evidence validation should be visible");
  assert(state.errorMessage?.includes("JSON array"), "required fixes JSON validation should be visible");
  assert(state.errorMessage?.includes("secret-looking"), "secret-looking evidence should be rejected");
}

function testTaskScopedInboxFiltersPersistedNotifications() {
  const scoped = buildTaskScopedInboxState("task-1", [
    notification({ id: "task", task_id: "task-1", run_id: "run-1" }),
    notification({ id: "run", task_id: null, run_id: "run-1" }),
    notification({ id: "other", task_id: "task-2", run_id: "run-9" }),
  ], ["run-1"]);

  assert(scoped.state === "ready", "filtered inbox should be ready");
  assert(scoped.records.map((record) => record.id).join(",") === "task,run", "should keep selected-task and linked-run notifications only");
}

function testRunHistoryRecordsFilterByRunTarget() {
  const records = [
    historyRecord("event-1", [{ entity_type: "task", entity_id: "task-1", relation_type: "primary" }]),
    historyRecord("event-2", [{ entity_type: "run", entity_id: "run-1", relation_type: "related" }]),
    historyRecord("event-3", [{ entity_type: "run", entity_id: "run-2", relation_type: "related" }]),
  ];

  const runRecords = runHistoryRecordsForRun(records, "run-1");
  assert(runRecords.length === 1, "should include only history records linked to selected run");
  assert(runRecords[0].event.id === "event-2", "should preserve the matching run history event");
}

await testManualReviewCreateUsesNativeBridgeShape();
await testManualReviewBlocksWeakEvidenceAndSecrets();
testTaskScopedInboxFiltersPersistedNotifications();
testRunHistoryRecordsFilterByRunTarget();

console.log("taskDetailBatchPanels tests passed");
