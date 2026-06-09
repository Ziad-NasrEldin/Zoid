import {
  approveSpecialistGate,
  blockedVerificationRecords,
  createDefaultContentWorkspaceState,
  createLocalScheduleIntent,
  omnisocialsActionCopy,
  parseContentWorkspaceState,
  pieceScheduleGateSummary,
  recordFailClosedSocialAction,
  validateScheduleIntent,
} from "./contentModel";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function assertEqual<T>(actual: T, expected: T, message: string) {
  if (actual !== expected) throw new Error(`${message}: expected ${String(expected)}, got ${String(actual)}`);
}

const state = createDefaultContentWorkspaceState();
const piece = state.pieces[0];

assertEqual(state.omnisocials.provider, "omnisocials", "Content uses OmniSocials as Buffer alternative provider");
assertEqual(state.omnisocials.state, "not_configured", "OmniSocials starts fail-closed");
assert(omnisocialsActionCopy(state.omnisocials).includes("Buffer alternative"), "copy names Buffer alternative boundary");
assert(omnisocialsActionCopy(state.omnisocials).includes("fail closed"), "copy states fail-closed action policy");

assertEqual(
  pieceScheduleGateSummary(piece, state.reviewGates, state.schedules),
  "Blocked until specialist review is approved.",
  "specialist gate blocks schedule",
);

const instagramBlocked = validateScheduleIntent(piece, state.mediaAssets, state.reviewGates, "instagram", true);
assertEqual(instagramBlocked.ok, false, "Instagram intent needs media");
assert(instagramBlocked.reason.includes("image or video"), "Instagram blocker explains media requirement");

const linkedinBlocked = validateScheduleIntent(piece, state.mediaAssets, state.reviewGates, "linkedin", true);
assertEqual(linkedinBlocked.ok, false, "LinkedIn intent still needs specialist gate");
assert(linkedinBlocked.reason.includes("Specialist review"), "LinkedIn blocker explains review gate");

const approved = approveSpecialistGate(state, piece.id);
assertEqual(approved.reviewGates[0].status, "approved", "specialist gate approval updates gate");
assertEqual(
  pieceScheduleGateSummary(piece, approved.reviewGates, approved.schedules),
  "Review gate passed; scheduling still needs human confirmation.",
  "approved specialist gate still requires human confirmation",
);

const unconfirmed = validateScheduleIntent(piece, approved.mediaAssets, approved.reviewGates, "linkedin", false);
assertEqual(unconfirmed.ok, false, "human confirmation is required");
assert(unconfirmed.reason.includes("Human confirmation"), "human confirmation blocker is explicit");

const intentState = createLocalScheduleIntent(approved, piece.id, "linkedin", true);
assertEqual(intentState.schedules.length, 1, "human-confirmed LinkedIn intent is recorded locally");
assertEqual(intentState.schedules[0].status, "intent", "schedule state remains local intent");
assert(intentState.verifications[0].failureReport?.includes("external publishing still disabled"), "intent verification does not imply external publish");

const blockedAction = recordFailClosedSocialAction(intentState, piece.id, "linkedin", "publish");
assertEqual(blockedAction.verifications[0].outcome, "blocked", "publish action blocks when provider is unconfigured");
assert(blockedAction.verifications[0].failureReport?.includes("did not call Buffer"), "blocked action records no Buffer/API call");
assert(blockedVerificationRecords(blockedAction.verifications).length >= 2, "blocked evidence ledger includes fail-closed actions");

const parsedFallback = parseContentWorkspaceState("not-json");
assertEqual(parsedFallback.omnisocials.state, "not_configured", "invalid persisted state falls back safely");

console.log("contentWorkspace tests passed");
