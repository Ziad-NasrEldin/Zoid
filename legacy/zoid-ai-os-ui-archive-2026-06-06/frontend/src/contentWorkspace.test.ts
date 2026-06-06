import {
  blockedVerificationRecords,
  buildContentWorkspaceDesignView,
  buildContentWorkspaceFlow,
  buildContentWorkspaceRefinementChecklist,
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

const contentDesign = buildContentWorkspaceDesignView();
assertEqual(contentDesign.sourceProjectId, "2534809720873389640", "content workspace design source project id");
assertEqual(contentDesign.screens.length, 16, "content workspace must expose all 16 Stitch screens");
assertEqual(
  contentDesign.screens.map((screen) => screen.title).join("|"),
  "Autonomous Campaign Dashboard|Brand Management - MaVoid|New Campaign Wizard|Advanced Campaign Editor|Content Slot Calendar|Today's Content Pipeline|Content Piece Detail & Adaptations|Content Editor / Override Flow|Approval-Needed Queue|Dry Test Report - MaVoid Daily|Run Now Modal|Recovery / Failure Center|OmniSocials & Account Mappings|Evidence & Artifact Library|Agent Execution & Notifications|Campaign Automation Mirror",
  "content workspace screen order",
);
assert(
  contentDesign.sampleNotice.includes("design-copy only") && contentDesign.sampleNotice.includes("fail-closed in preview"),
  "content workspace sample data must be visibly disclosed",
);
assert(
  contentDesign.designSystem.name === "Apple-design-analysis" && contentDesign.designSystem.primary === "#0066cc" && contentDesign.designSystem.disallowedUiShadow,
  "content workspace design must follow current Zoid DESIGN.md tokens",
);
assert(
  contentDesign.screens.every((screen) => screen.intent.length > 0 && screen.keyRegions.length >= 3),
  "every content workspace screen needs implementation intent and key visual regions",
);
assertEqual(
  contentDesign.screens.map((screen) => screen.surfaceKind).join("|"),
  "dashboard|brand|wizard|editor|calendar|pipeline|detail|override|queue|report|modal|recovery|mapping|library|agents|mirror",
  "content workspace screen surface design kinds",
);

const contentFlow = buildContentWorkspaceFlow();
assertEqual(contentFlow.defaultScreen, "dashboard", "content workspace default screen");
assertEqual(contentFlow.screens.length, 16, "content workspace must define 16 distinct flow screens");
assertEqual(contentFlow.screens.filter((screen) => screen.type === "modal").map((screen) => screen.id).join("|"), "run-now-modal", "only run-now is a modal state");
assertEqual(
  contentFlow.screens.map((screen) => screen.id).join("|"),
  "dashboard|brand-management|new-campaign|campaign-editor|slot-calendar|today-pipeline|piece-detail|piece-editor|approval-queue|dry-test-report|run-now-modal|recovery-center|omnisocials-mappings|evidence-library|agent-execution|automation-mirror",
  "content workspace distinct screen state order",
);
assertEqual(contentFlow.sections.join("|"), "Dashboard|Campaigns|Calendar|Pipeline|Approvals|Runs|Library|Settings", "content workspace navigation sections");
assert(
  contentFlow.screens.every((screen) => screen.routeLabel.startsWith("Content > ") && screen.outgoing.length > 0),
  "every content flow screen must have a route label and outgoing transitions",
);
assert(
  contentFlow.screens.some((screen) => screen.id === "dashboard" && screen.outgoing.includes("new-campaign") && screen.outgoing.includes("run-now-modal")),
  "dashboard must open campaign creation and run-now modal",
);
assert(
  contentFlow.screens.some((screen) => screen.id === "piece-detail" && screen.outgoing.includes("piece-editor") && screen.outgoing.includes("evidence-library")),
  "piece detail must lead to editor and evidence library",
);
assert(
  contentFlow.screens.every((screen) => screen.id !== "dashboard" || screen.renderMode === "primary-screen"),
  "dashboard is a primary screen, not a catalog wrapper",
);
assert(
  !contentFlow.allScreensVisibleAtOnce,
  "content flow must not render all 16 screens at once",
);

const refinementChecklist = buildContentWorkspaceRefinementChecklist();
assertEqual(refinementChecklist.length, 16, "every content flow state must have a refinement checklist entry");
assertEqual(
  refinementChecklist.map((item) => item.screenId).join("|"),
  contentFlow.screens.map((screen) => screen.id).join("|"),
  "refinement checklist must match flow screen order",
);
assert(
  refinementChecklist.every((item) => item.visibleRegions.length >= 4 && item.primaryAction.length > 0 && item.visualPolishNotes.length >= 2),
  "each user-visible page needs concrete regions, primary action, and polish notes",
);
assert(
  refinementChecklist.every((item) => item.pageFeelsLike === "finished-product-screen"),
  "each page must target a finished product screen, not placeholder/card filler",
);

console.log("contentWorkspace tests passed");
