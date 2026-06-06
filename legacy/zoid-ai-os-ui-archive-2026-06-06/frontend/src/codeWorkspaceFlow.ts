export type CodeFlowScreenId =
  | "command-center"
  | "empty-onboarding"
  | "repo-discovery"
  | "repository-detail"
  | "checks"
  | "start-agent"
  | "launch-gate"
  | "evidence"
  | "commit-pr"
  | "deployment"
  | "github-auth"
  | "repo-settings"
  | "search-history"
  | "handoff-export"
  | "diagnostics";

export type CodeFlowOrigin = CodeFlowScreenId | "attention";

export type CodeFlowAction =
  | "open-code"
  | "scan-folders"
  | "approve-discovery"
  | "select-repo"
  | "open-details"
  | "run-checks"
  | "checks-pass"
  | "checks-fail"
  | "review-complete"
  | "commit-complete"
  | "deploy-recorded"
  | "production-verified"
  | "mark-verified"
  | "start-agent"
  | "open-launch-gate"
  | "resolve-git"
  | "attach-evidence"
  | "commit-pr"
  | "deploy"
  | "github-auth"
  | "fix-path"
  | "search-history"
  | "export-handoff"
  | "diagnostics"
  | "cancel"
  | "return-home";

export type CodeFlowScreen = {
  id: CodeFlowScreenId;
  title: string;
  routeLabel: string;
  purpose: string;
  primaryAction: string;
  evidenceRule: string;
  visibleRegions: string[];
};

export type CodeFlowState = {
  screen: CodeFlowScreenId;
  origin: CodeFlowOrigin;
  returnTarget: CodeFlowScreenId;
  selectedRepoId: string | null;
  lastEvent: string;
  gateItems: { gitState: boolean; checks: boolean; review: boolean; commitPr: boolean; deploy: boolean; production: boolean };
  checksStatus: "idle" | "passed" | "failed";
  agentStatus: "not-started" | "running" | "waiting" | "done";
  historyEvents: string[];
};

export const codeFlowScreens: CodeFlowScreen[] = [
  { id: "command-center", title: "Repo Health Command Center", routeLabel: "Code / command center", purpose: "Select repos, see attention items, open inspector actions, and keep normal clicks lightweight.", primaryAction: "Select repo", evidenceRule: "No shipping state can be marked verified here without Launch Gate evidence.", visibleRegions: ["What Needs Me", "Managed repositories", "Right inspector", "Launch readiness", "Linked agents"] },
  { id: "empty-onboarding", title: "Empty Code Workspace", routeLabel: "First-time entry", purpose: "Explain that no repos are fabricated and route the user to folder scanning.", primaryAction: "Add Scan Folder", evidenceRule: "Folder access starts only through explicit permission flow.", visibleRegions: ["Empty state", "Add Scan Folder CTA", "Permission note", "No synthetic repos"] },
  { id: "repo-discovery", title: "Repo Discovery / Scan Folders", routeLabel: "Discovery", purpose: "Group discovered repos as Recommended, Needs Review, Duplicates, and Ignored before approval.", primaryAction: "Approve selected repos", evidenceRule: "Approving records imports local repo profiles only after permission.", visibleRegions: ["Recommended", "Needs Review", "Duplicates", "Ignored", "Finder exit"] },
  { id: "repository-detail", title: "Repository Detail", routeLabel: "Repo detail", purpose: "Focused repo workspace for status, diff, checks, PRs, deployments, settings, and history.", primaryAction: "Run checks", evidenceRule: "Repo health is informational until a command output or manual evidence is attached.", visibleRegions: ["Summary", "Diff", "Checks", "PR/Git", "Deployments", "History"] },
  { id: "checks", title: "Run Checks", routeLabel: "Checks", purpose: "Show detected commands, run required checks, preserve failed output, and save passing output as launch evidence.", primaryAction: "Run required", evidenceRule: "Failed commands can create blocked evidence but never success evidence.", visibleRegions: ["Detected commands", "Progress", "Failed output", "Save as evidence", "Start agent"] },
  { id: "start-agent", title: "Start Agent From Repo", routeLabel: "Agent modal", purpose: "Create a linked agent run with repo/workdir/diff/context attached while staying in Code Workspace.", primaryAction: "Start linked agent", evidenceRule: "Agent output is attached as history/evidence only after the run completes.", visibleRegions: ["Preset", "Prompt", "Permission preview", "Linked run", "Open in Agents"] },
  { id: "launch-gate", title: "Launch Gate", routeLabel: "Shipping", purpose: "Central shipping checklist that routes every missing item to the exact resolution flow.", primaryAction: "Resolve missing item", evidenceRule: "Complete every checklist item before the final verified-launch action appears.", visibleRegions: ["Checklist", "Blocked item", "Evidence stack", "Verdict", "Return target"] },
  { id: "evidence", title: "Evidence Attachment / Verification", routeLabel: "Evidence", purpose: "Attach test output, URLs, screenshots, deployment records, or manual notes and classify them as required/supporting.", primaryAction: "Save evidence", evidenceRule: "Insufficient evidence returns to Launch Gate with the missing requirement focused.", visibleRegions: ["Evidence type", "Source metadata", "Required/supporting", "Sufficiency", "Return to gate"] },
  { id: "commit-pr", title: "Commit / PR Workflow", routeLabel: "Git", purpose: "Review dirty files, select staged scope, generate/edit commit message, and optionally create draft PR.", primaryAction: "Confirm commit scope", evidenceRule: "No commit/push is simulated in browser preview; native Git/GitHub must execute it.", visibleRegions: ["Dirty files", "Risky files", "Commit message", "PR draft", "GitHub auth"] },
  { id: "deployment", title: "Deployment Tracking / Actions", routeLabel: "Deploy", purpose: "Select environment/provider, run or record deployment, capture URLs/status, and route to production verification.", primaryAction: "Record deployment", evidenceRule: "Deploy success is not launch success until production verification evidence is attached.", visibleRegions: ["Target", "Provider", "Command/manual", "Result", "Verify production"] },
  { id: "github-auth", title: "GitHub / PR Integration", routeLabel: "GitHub", purpose: "Gracefully connect GitHub for remote, PR, and CI actions without blocking local repo work.", primaryAction: "Connect GitHub", evidenceRule: "Remote state remains unavailable until real auth succeeds.", visibleRegions: ["Connect CTA", "Return target", "Local fallback", "PR status"] },
  { id: "repo-settings", title: "Repo Settings / Fix Path", routeLabel: "Settings", purpose: "Repair missing paths, edit repo profile/group/commands/deployment notes, and validate the replacement folder.", primaryAction: "Choose new folder", evidenceRule: "A path update requires same-repo validation or an explicit override reason.", visibleRegions: ["Path status", "Profile", "Commands", "Rules", "Validation"] },
  { id: "search-history", title: "Search / History / Archive", routeLabel: "History", purpose: "Search repos, diffs, PRs, evidence, agents, and launch gates while restoring origin context.", primaryAction: "Open result", evidenceRule: "History can navigate to evidence but does not create success evidence by itself.", visibleRegions: ["Search", "Filters", "Archive", "Event detail", "Restore repo"] },
  { id: "handoff-export", title: "Repo Handoff Export", routeLabel: "Handoff", purpose: "Preview repo state, choose include/exclude options, and copy/save/attach a handoff.", primaryAction: "Copy handoff", evidenceRule: "Handoff export records history but does not satisfy launch verification alone.", visibleRegions: ["Preview", "Include options", "Destination", "Attach to agent", "History event"] },
  { id: "diagnostics", title: "Native Verification / Diagnostics", routeLabel: "Diagnostics", purpose: "Check Tauri status, folder permissions, git availability, registry persistence, GitHub auth, and evidence storage.", primaryAction: "Run diagnostics", evidenceRule: "Diagnostics explain capability gaps; passing diagnostics still requires feature evidence.", visibleRegions: ["Tauri", "Permissions", "Git", "Registry", "Evidence paths"] },
];

export const codeFlowScreenById = Object.fromEntries(codeFlowScreens.map((screen) => [screen.id, screen])) as Record<CodeFlowScreenId, CodeFlowScreen>;

export function createInitialCodeFlowState(hasManagedRepos: boolean): CodeFlowState {
  return {
    screen: hasManagedRepos ? "command-center" : "empty-onboarding",
    origin: "command-center",
    returnTarget: "command-center",
    selectedRepoId: hasManagedRepos ? "repo-zoid-main" : null,
    lastEvent: hasManagedRepos ? "Returning entry selected the first Needs Me repo." : "First-time entry has no managed repos yet.",
    gateItems: { gitState: false, checks: false, review: false, commitPr: false, deploy: false, production: false },
    checksStatus: "idle",
    agentStatus: "not-started",
    historyEvents: [hasManagedRepos ? "Returning entry selected first Needs Me repo." : "First-time entry opened empty onboarding."],
  };
}

function pushHistory(state: CodeFlowState, event: string): string[] {
  return [...state.historyEvents.slice(-7), event];
}

function openSubflow(state: CodeFlowState, screen: CodeFlowScreenId, event: string, options: { origin?: CodeFlowOrigin; returnTarget?: CodeFlowScreenId; patch?: Partial<CodeFlowState> } = {}): CodeFlowState {
  const returnTarget = options.returnTarget ?? (state.screen === screen ? state.returnTarget : state.screen);
  return {
    ...state,
    ...options.patch,
    screen,
    origin: options.origin ?? state.origin,
    returnTarget,
    lastEvent: event,
    historyEvents: pushHistory(state, event),
  };
}

export function reduceCodeFlow(state: CodeFlowState, action: CodeFlowAction): CodeFlowState {
  switch (action) {
    case "open-code":
    case "return-home":
      return openSubflow(state, "command-center", "Returned to Repo Health Command Center.", { origin: "command-center", returnTarget: "command-center" });
    case "scan-folders":
      return openSubflow(state, "repo-discovery", "Opened folder permission and discovery flow.", { origin: "command-center", returnTarget: "empty-onboarding" });
    case "approve-discovery":
      return { ...openSubflow(state, "command-center", "Approved selected repos and highlighted the first imported repo.", { origin: "command-center", returnTarget: "command-center" }), selectedRepoId: state.selectedRepoId ?? "repo-zoid-main" };
    case "select-repo":
      return { ...openSubflow(state, "command-center", "Selected repo and updated right inspector without leaving command center.", { origin: "command-center", returnTarget: "command-center" }), selectedRepoId: state.selectedRepoId ?? "repo-zoid-main" };
    case "open-details":
      return { ...openSubflow(state, "repository-detail", "Opened focused repository detail.", { origin: state.screen, returnTarget: state.screen }), selectedRepoId: state.selectedRepoId ?? "repo-zoid-main" };
    case "run-checks":
      return openSubflow(state, "checks", "Opened detected checks for selected repo.", { origin: state.screen, returnTarget: state.screen });
    case "checks-pass":
      return openSubflow(state, "evidence", "Checks passed and were saved as required check evidence; remaining Launch Gate items still gate verification.", {
        origin: state.origin,
        returnTarget: state.origin === "launch-gate" ? "launch-gate" : state.returnTarget,
        patch: { checksStatus: "passed", gateItems: { ...state.gateItems, checks: true } },
      });
    case "checks-fail":
      return openSubflow(state, "checks", "Failed check output stayed expanded and can route to an agent or blocked evidence.", { origin: "attention", returnTarget: "command-center", patch: { checksStatus: "failed" } });
    case "review-complete":
      return openSubflow(state, "launch-gate", "Reviewer agent evidence attached to Launch Gate.", { origin: "launch-gate", returnTarget: "launch-gate", patch: { agentStatus: "done", gateItems: { ...state.gateItems, review: true } } });
    case "commit-complete":
      return openSubflow(state, "launch-gate", "Commit / PR evidence attached to Launch Gate.", { origin: "launch-gate", returnTarget: "launch-gate", patch: { gateItems: { ...state.gateItems, gitState: true, commitPr: true } } });
    case "deploy-recorded":
      return openSubflow(state, "evidence", "Deployment record captured; production verification still required.", { origin: "launch-gate", returnTarget: "launch-gate", patch: { gateItems: { ...state.gateItems, deploy: true } } });
    case "production-verified":
      return openSubflow(state, "launch-gate", "Production verification evidence attached to Launch Gate.", { origin: "launch-gate", returnTarget: "launch-gate", patch: { gateItems: { ...state.gateItems, production: true } } });
    case "mark-verified": {
      const allowed = launchGateCanMarkVerified(state);
      return openSubflow(state, "launch-gate", allowed ? "Launch Gate marked verified with full required evidence." : "Mark Verified blocked because required evidence is incomplete.", { origin: "launch-gate", returnTarget: "launch-gate" });
    }
    case "start-agent":
      return openSubflow(state, "start-agent", "Started linked agent run in Code Workspace context.", { origin: state.screen, returnTarget: state.screen, patch: { agentStatus: "running" } });
    case "open-launch-gate":
      return openSubflow(state, "launch-gate", "Opened Launch Gate with missing evidence focused.", { origin: "launch-gate", returnTarget: "command-center" });
    case "resolve-git":
      return openSubflow(state, "repository-detail", "Launch Gate routed Git state to repo detail diff.", { origin: "launch-gate", returnTarget: "launch-gate" });
    case "attach-evidence":
      return openSubflow(state, "evidence", "Opened evidence attachment for missing verification.", { origin: state.screen, returnTarget: state.screen });
    case "commit-pr":
      return openSubflow(state, "commit-pr", "Opened commit and PR workflow with origin preserved.", { origin: state.screen, returnTarget: state.screen });
    case "deploy":
      return openSubflow(state, "deployment", "Opened deployment tracking and verification route.", { origin: state.screen, returnTarget: state.screen });
    case "github-auth":
      return openSubflow(state, "github-auth", "Opened GitHub connection flow and preserved return target.", { origin: state.screen, returnTarget: state.screen });
    case "fix-path":
      return openSubflow(state, "repo-settings", "Opened missing path repair flow from attention inbox or current repo context.", { origin: state.screen === "command-center" ? "attention" : state.screen, returnTarget: state.screen });
    case "search-history":
      return openSubflow(state, "search-history", "Opened search, history, and archive.", { origin: state.screen, returnTarget: state.screen });
    case "export-handoff":
      return openSubflow(state, "handoff-export", "Opened repo handoff export.", { origin: state.screen, returnTarget: state.screen });
    case "diagnostics":
      return openSubflow(state, "diagnostics", "Opened native verification diagnostics.", { origin: state.screen, returnTarget: state.screen });
    case "cancel":
      return openSubflow(state, state.returnTarget, "Cancelled subflow and returned to preserved origin.", { origin: state.returnTarget, returnTarget: state.returnTarget });
  }
}


export function launchGateCanMarkVerified(state: CodeFlowState): boolean {
  return state.checksStatus === "passed"
    && state.gateItems.gitState
    && state.gateItems.checks
    && state.gateItems.review
    && state.gateItems.commitPr
    && state.gateItems.deploy
    && state.gateItems.production;
}

export type CodeAttentionType = "failed-checks" | "launch-blocked" | "deploy-unverified" | "agent-waiting" | "dirty-changes" | "secrets-config" | "pr-failing" | "path-missing";

export function codeFlowRouteForAttention(type: CodeAttentionType): CodeFlowAction {
  const routes: Record<CodeAttentionType, CodeFlowAction> = {
    "failed-checks": "run-checks",
    "launch-blocked": "open-launch-gate",
    "deploy-unverified": "deploy",
    "agent-waiting": "start-agent",
    "dirty-changes": "open-details",
    "secrets-config": "open-details",
    "pr-failing": "github-auth",
    "path-missing": "fix-path",
  };
  return routes[type];
}
