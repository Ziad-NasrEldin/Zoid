import {
  blockedVerificationRecords,
  omnisocialsActionCopy,
  parsePlatforms,
  pieceScheduleGateSummary,
  type ContentPieceRecord,
  type ContentReviewGateRecord,
  type ContentScheduleRecord,
} from "./contentWorkspace";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}
function assertEqual<T>(actual: T, expected: T, message: string) {
  if (actual !== expected) throw new Error(`${message}: expected ${String(expected)}, got ${String(actual)}`);
}
function assertArrayEqual(actual: string[], expected: string[], message: string) {
  assertEqual(JSON.stringify(actual), JSON.stringify(expected), message);
}

const piece: ContentPieceRecord = {
  id: "piece-1",
  plan_id: "plan-1",
  title: "Launch post",
  body_markdown: "Draft body",
  status: "review_ready",
  platforms_json: '["linkedin","instagram"]',
  required_gate: "specialist_review",
  metadata_json: "{}",
};

assertArrayEqual(parsePlatforms(piece.platforms_json), ["linkedin", "instagram"], "valid platforms parse");
assertArrayEqual(parsePlatforms("not-json"), [], "invalid platforms fail closed to empty list");

assertEqual(
  pieceScheduleGateSummary(piece, [], []),
  "Schedule blocked until specialist review is approved.",
  "unreviewed piece is blocked",
);

const approvedGate: ContentReviewGateRecord = {
  id: "gate-1",
  piece_id: "piece-1",
  gate_type: "specialist_review",
  status: "approved",
  evidence_summary: "Reviewed by specialist.",
};
assertEqual(
  pieceScheduleGateSummary(piece, [approvedGate], []),
  "Review gate passed; schedule still needs human confirmation.",
  "approved review still needs confirmation",
);

const schedule: ContentScheduleRecord = {
  id: "schedule-1",
  piece_id: "piece-1",
  platform: "linkedin",
  scheduled_for: "2026-06-05T18:00:00+02:00",
  status: "intent",
  confirmation_id: "confirmation-1",
};
assertEqual(
  pieceScheduleGateSummary(piece, [approvedGate], [schedule]),
  "Schedule intent recorded locally — no external publish implied.",
  "schedule intent is local-only copy",
);

assert(
  /fail-closed/.test(
    omnisocialsActionCopy({
      state: "not_configured",
      platform: "omnisocials",
      credential_ref: null,
      status_note: "No credentials.",
    }),
  ),
  "not configured OmniSocials copy is fail-closed",
);
assertEqual(
  blockedVerificationRecords([
    { id: "1", platform: "linkedin", action_type: "publish", outcome: "blocked" },
    { id: "2", platform: "linkedin", action_type: "publish", outcome: "manual" },
  ]).length,
  1,
  "blocked verification filter",
);

console.log("contentWorkspace tests passed");
