import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import {
  codeFlowRouteForAttention,
  codeFlowScreenById,
  codeFlowScreens,
  createInitialCodeFlowState,
  launchGateCanMarkVerified,
  reduceCodeFlow,
  type CodeAttentionType,
  type CodeFlowAction,
  type CodeFlowScreenId,
} from "./codeWorkspaceFlow";

assert.equal(codeFlowScreens.length, 15, "all primary Code Workspace screens should be modeled");
assert.equal(createInitialCodeFlowState(false).screen, "empty-onboarding", "first-time entry starts with empty onboarding");
assert.equal(createInitialCodeFlowState(true).screen, "command-center", "returning entry starts at command center");

let state = createInitialCodeFlowState(false);
state = reduceCodeFlow(state, "scan-folders");
assert.equal(state.screen, "repo-discovery", "Add Scan Folder opens discovery flow");
state = reduceCodeFlow(state, "approve-discovery");
assert.equal(state.screen, "command-center", "approving repos returns to command center");
assert.equal(state.selectedRepoId, "repo-zoid-main", "approved discovery selects first imported repo");

const attentionExpectations: Array<[CodeAttentionType, CodeFlowScreenId]> = [
  ["failed-checks", "checks"],
  ["launch-blocked", "launch-gate"],
  ["deploy-unverified", "deployment"],
  ["agent-waiting", "start-agent"],
  ["dirty-changes", "repository-detail"],
  ["secrets-config", "repository-detail"],
  ["pr-failing", "github-auth"],
  ["path-missing", "repo-settings"],
];
for (const [type, expectedScreen] of attentionExpectations) {
  const routed = reduceCodeFlow(state, codeFlowRouteForAttention(type));
  assert.equal(routed.screen, expectedScreen, `${type} should route to ${expectedScreen}`);
}

state = reduceCodeFlow(state, "open-launch-gate");
assert.equal(state.origin, "launch-gate", "Launch Gate origin is preserved");
state = reduceCodeFlow(state, "run-checks");
assert.equal(state.screen, "checks", "Launch Gate checks item opens checks");
assert.equal(state.returnTarget, "launch-gate", "checks preserve launch-gate return target");
state = reduceCodeFlow(state, "checks-fail");
assert.equal(launchGateCanMarkVerified(state), false, "failed checks cannot unlock verification");
state = reduceCodeFlow(state, "start-agent");
assert.equal(state.agentStatus, "running", "failed checks can start linked agent");
state = reduceCodeFlow(state, "open-launch-gate");
state = reduceCodeFlow(state, "run-checks");
state = reduceCodeFlow(state, "checks-pass");
assert.equal(state.screen, "evidence", "passing checks route to evidence attachment");
assert.equal(state.gateItems.checks, true, "passing checks attach check evidence");
assert.equal(launchGateCanMarkVerified(state), false, "checks alone do not unlock Mark Verified");
for (const action of ["review-complete", "commit-complete", "deploy-recorded"] as CodeFlowAction[]) {
  state = reduceCodeFlow(state, action);
  assert.equal(launchGateCanMarkVerified(state), false, `${action} still needs remaining required evidence`);
}
state = reduceCodeFlow(state, "production-verified");
assert.equal(launchGateCanMarkVerified(state), true, "all required evidence unlocks Mark Verified");
state = reduceCodeFlow(state, "mark-verified");
assert.match(state.lastEvent, /marked verified/, "mark verified records success only after all gates pass");
state = reduceCodeFlow(state, "cancel");
assert.equal(state.screen, "launch-gate", "cancel returns to preserved launch origin");

const originCases: Array<[CodeFlowScreenId, CodeFlowAction, CodeFlowScreenId]> = [
  ["repository-detail", "run-checks", "repository-detail"],
  ["launch-gate", "attach-evidence", "launch-gate"],
  ["repository-detail", "commit-pr", "repository-detail"],
  ["launch-gate", "deploy", "launch-gate"],
  ["command-center", "github-auth", "command-center"],
  ["repository-detail", "fix-path", "repository-detail"],
  ["repository-detail", "diagnostics", "repository-detail"],
  ["repository-detail", "start-agent", "repository-detail"],
];
for (const [start, action, expectedReturn] of originCases) {
  const base = { ...createInitialCodeFlowState(true), screen: start, returnTarget: start, origin: start };
  const routed = reduceCodeFlow(base, action);
  const returned = reduceCodeFlow(routed, "cancel");
  assert.equal(returned.screen, expectedReturn, `${action} should cancel back to ${expectedReturn}`);
}

const requiredActions: CodeFlowAction[] = ["resolve-git", "run-checks", "start-agent", "commit-pr", "deploy", "attach-evidence", "github-auth", "fix-path", "search-history", "export-handoff", "diagnostics"];
for (const action of requiredActions) {
  const routed = reduceCodeFlow(createInitialCodeFlowState(true), action);
  assert.ok(codeFlowScreenById[routed.screen], `${action} should land on a modeled screen`);
  assert.ok(routed.historyEvents.length > 0, `${action} should write history`);
}

const viewSource = readFileSync(new URL("./codeWorkspaceFlowView.tsx", import.meta.url), "utf8");
assert.match(viewSource, /One guided flow from repo to verified launch/, "redesign should present a clear guided-flow headline");
assert.match(viewSource, /Next best action/, "redesign should emphasize one next action instead of a button wall");
assert.doesNotMatch(viewSource, /Every item has a deterministic target/, "old abstract inspector copy should be removed");
assert.doesNotMatch(viewSource, /Record passed checks evidence/, "old action-wall labels should be removed");
assert.doesNotMatch(viewSource, /Mark Verified is locked/, "locked Launch Gate should not be presented as an enabled primary action");
assert.doesNotMatch(viewSource, /screen\.id === "empty-onboarding" \? "return-home"/, "Connect step should not route first-time users to command center");
assert.match(viewSource, /case "empty-onboarding":\n\s+return "scan-folders"/, "Connect step should open repo discovery");

console.log("codeWorkspaceFlow tests passed");
