import {
  createHistoryTimelineViewModel,
  deriveNextCursor,
  sanitizeMessage,
  sortHistoryRecords,
  type HistoryMode,
  type HistoryTimelineRecord,
} from "./historyTimelineViewModel";

const primary = { entity_type: "task", entity_id: "task-1" };

function assertEqual<T>(actual: T, expected: T, message?: string): void {
  if (actual !== expected) {
    throw new Error(message ?? `Expected ${String(expected)}, received ${String(actual)}`);
  }
}

function assertDeepEqual(actual: unknown, expected: unknown, message?: string): void {
  const actualJson = JSON.stringify(actual);
  const expectedJson = JSON.stringify(expected);
  if (actualJson !== expectedJson) {
    throw new Error(message ?? `Expected ${expectedJson}, received ${actualJson}`);
  }
}

function record(
  id: string,
  timestamp: string,
  overrides: Partial<HistoryTimelineRecord["event"]> = {},
): HistoryTimelineRecord {
  const targets = overrides.targets ?? [
    { entity_type: "task", entity_id: "task-1", relation_type: "primary" },
  ];
  return {
    event: {
      id,
      action_type: "task_completed",
      outcome: "success",
      timestamp,
      actor_type: "agent",
      actor_id: "agent-1",
      workspace_key: "agents",
      summary: "Task completed",
      source: "local_bridge",
      metadata_json: '{"raw_log":"secret should not render"}',
      targets,
      ...overrides,
    },
    matched_entities: targets,
  };
}

function runTest(name: string, test: () => void): void {
  try {
    test();
    console.log(`PASS ${name}`);
  } catch (error) {
    console.error(`FAIL ${name}`);
    throw error;
  }
}

runTest("sorts unsorted input newest-first with id tie-breaker", () => {
  const sorted = sortHistoryRecords([
    record("event-b", "2026-01-02T00:00:00Z"),
    record("event-a", "2026-01-03T00:00:00Z"),
    record("event-c", "2026-01-03T00:00:00Z"),
  ]);

  assertDeepEqual(
    sorted.map((item) => item.event.id),
    ["event-c", "event-a", "event-b"],
  );
});

runTest("builds task, run, notification, and entity mode titles", () => {
  const expectations: Array<[HistoryMode, string]> = [
    ["task", "Task history"],
    ["run", "Run history"],
    ["notification", "Notification history"],
    ["entity", "Entity history"],
  ];

  for (const [mode, title] of expectations) {
    const viewModel = createHistoryTimelineViewModel({
      mode,
      primary: mode === "run" ? { entity_type: "agent_run", entity_id: "run-1" } : primary,
      records: [record(`${mode}-event`, "2026-01-02T00:00:00Z")],
    });
    assertEqual(viewModel.title, title);
    assertEqual(viewModel.entries.length, 1);
  }
});

runTest("redacts secrets and avoids metadata/raw log display", () => {
  const viewModel = createHistoryTimelineViewModel({
    mode: "task",
    primary,
    records: [
      record("event-secret", "2026-01-02T00:00:00Z", {
        summary: "raw log: token=abc123 password:open-sesame sk-liveSECRET01 finished",
        metadata_json: '{"token":"metadata-secret"}',
      }),
    ],
  });

  const summary = viewModel.entries[0]?.summary ?? "";
  assertEqual(summary.includes("abc123"), false, summary);
  assertEqual(summary.includes("open-sesame"), false, summary);
  assertEqual(summary.includes("***"), false, summary);
  assertEqual(summary.includes("metadata-secret"), false, summary);
  assertEqual(summary.toLowerCase().includes("raw log:"), false, summary);
});

runTest("reports loading, empty, and error states", () => {
  const loading = createHistoryTimelineViewModel({ mode: "task", primary, status: "loading" });
  assertEqual(loading.isLoading, true);
  assertEqual(loading.paginationLabel, "Loading history...");

  const empty = createHistoryTimelineViewModel({ mode: "notification", primary, records: [] });
  assertEqual(empty.emptyMessage, "No notification history events yet.");

  const errored = createHistoryTimelineViewModel({
    mode: "entity",
    primary,
    status: "error",
    error: "failed with secret=do-not-show",
  });
  assertEqual(errored.errorMessage?.includes("do-not-show"), false);
  assertEqual(errored.paginationLabel, "History unavailable");
});

runTest("derives or accepts pagination cursor", () => {
  const records = [
    record("newest", "2026-01-03T00:00:00Z"),
    record("oldest", "2026-01-01T00:00:00Z"),
  ];
  assertDeepEqual(deriveNextCursor(records, 2), {
    timestamp: "2026-01-01T00:00:00Z",
    event_id: "oldest",
  });

  const supplied = { timestamp: "2025-12-31T00:00:00Z", event_id: "cursor-event" };
  const viewModel = createHistoryTimelineViewModel({
    mode: "run",
    primary: { entity_type: "agent_run", entity_id: "run-1" },
    records,
    nextCursor: supplied,
  });
  assertEqual(viewModel.hasNextPage, true);
  assertDeepEqual(viewModel.nextCursor, supplied);
});

runTest("formats matched entities and primary match status", () => {
  const viewModel = createHistoryTimelineViewModel({
    mode: "entity",
    primary: { entity_type: "notification", entity_id: "notif-1" },
    records: [
      {
        ...record("event-entity", "2026-01-02T00:00:00Z", {
          action_type: "notification_sent",
          outcome: "sent",
          source: "gmail_plugin",
          targets: [
            { entity_type: "notification", entity_id: "notif-1", relation_type: "primary" },
            { entity_type: "task", entity_id: "task-1", relation_type: "related" },
          ],
        }),
        matched_entities: [
          { entity_type: "notification", entity_id: "notif-1", relation_type: "primary" },
        ],
      },
    ],
  });

  assertEqual(viewModel.entries[0]?.actionLabel, "Notification Sent");
  assertEqual(viewModel.entries[0]?.sourceLabel, "Gmail Plugin");
  assertEqual(viewModel.entries[0]?.tone, "success");
  assertEqual(viewModel.entries[0]?.isPrimaryMatch, true);
  assertDeepEqual(viewModel.entries[0]?.matchedLabels, ["Notification notif-1 (Primary)"]);
});

runTest("sanitizeMessage truncates multiline operational text", () => {
  const message = sanitizeMessage(`line one\n${"x".repeat(260)}`);
  assertEqual(message.includes("\n"), false);
  assertEqual(message.length <= 220, true);
});
