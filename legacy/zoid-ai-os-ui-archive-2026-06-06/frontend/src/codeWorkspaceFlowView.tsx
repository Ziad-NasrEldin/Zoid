import { useMemo, useState } from "react";
import "./codeWorkspaceFlow.css";
import {
  codeFlowRouteForAttention,
  codeFlowScreenById,
  createInitialCodeFlowState,
  launchGateCanMarkVerified,
  reduceCodeFlow,
  type CodeAttentionType,
  type CodeFlowAction,
  type CodeFlowScreenId,
  type CodeFlowState,
} from "./codeWorkspaceFlow";

type CodeWorkspaceFlowProps = {
  nativeMode: "loading" | "ready" | "error";
  nativeError?: string | null;
  repoCount: number;
  actionStatus?: string | null;
};

type ActionSpec = {
  action: CodeFlowAction;
  label: string;
  tone?: "primary" | "quiet" | "danger";
};

type StepSpec = {
  id: CodeFlowScreenId;
  label: string;
  description: string;
};

const guidedSteps: StepSpec[] = [
  { id: "empty-onboarding", label: "Connect", description: "Add a repo folder." },
  { id: "repo-discovery", label: "Approve", description: "Choose discovered repos." },
  { id: "command-center", label: "Command", description: "See repo health." },
  { id: "repository-detail", label: "Inspect", description: "Review diff and settings." },
  { id: "checks", label: "Checks", description: "Run required checks." },
  { id: "launch-gate", label: "Ship", description: "Complete launch evidence." },
  { id: "evidence", label: "Evidence", description: "Attach proof." },
  { id: "commit-pr", label: "PR", description: "Commit or open PR." },
  { id: "deployment", label: "Deploy", description: "Record deployment." },
];

const gateLabels: Array<{ key: keyof CodeFlowState["gateItems"]; label: string; help: string; action: CodeFlowAction }> = [
  { key: "gitState", label: "Git state reviewed", help: "Diff/scope is known before shipping.", action: "resolve-git" },
  { key: "checks", label: "Checks passed", help: "Required command output is saved.", action: "run-checks" },
  { key: "review", label: "Reviewer approved", help: "Human or reviewer-agent evidence exists.", action: "start-agent" },
  { key: "commitPr", label: "Commit / PR ready", help: "Commit or PR evidence is attached.", action: "commit-pr" },
  { key: "deploy", label: "Deployment recorded", help: "A deployment result or URL is captured.", action: "deploy" },
  { key: "production", label: "Production verified", help: "Live URL / screenshot / status proof exists.", action: "attach-evidence" },
];

const attentionItems: Array<{ type: CodeAttentionType; label: string; detail: string }> = [
  { type: "failed-checks", label: "Failed checks", detail: "Open check output." },
  { type: "launch-blocked", label: "Launch blocked", detail: "Open missing gate item." },
  { type: "deploy-unverified", label: "Deploy unverified", detail: "Record deploy evidence." },
  { type: "agent-waiting", label: "Agent waiting", detail: "Open linked run." },
  { type: "dirty-changes", label: "Dirty changes", detail: "Inspect repo diff." },
  { type: "secrets-config", label: "Secrets/config changed", detail: "Review danger-zone diff." },
  { type: "pr-failing", label: "PR needs review", detail: "Open GitHub/PR state." },
  { type: "path-missing", label: "Repo path missing", detail: "Repair folder path." },
];

function dispatchLabel(action: CodeFlowAction): string {
  const labels: Record<CodeFlowAction, string> = {
    "open-code": "Open Code",
    "scan-folders": "Add repo folder",
    "approve-discovery": "Approve selected repos",
    "select-repo": "Select repo",
    "open-details": "Open repo detail",
    "run-checks": "Run checks",
    "checks-pass": "Save passing check evidence",
    "checks-fail": "Save failed check output",
    "review-complete": "Mark review approved",
    "commit-complete": "Mark PR evidence done",
    "deploy-recorded": "Mark deploy recorded",
    "production-verified": "Mark production verified",
    "mark-verified": "Mark launch verified",
    "start-agent": "Start reviewer agent",
    "open-launch-gate": "Open Launch Gate",
    "resolve-git": "Review git state",
    "attach-evidence": "Attach evidence",
    "commit-pr": "Commit / PR",
    "deploy": "Deploy / verify",
    "github-auth": "GitHub auth",
    "fix-path": "Fix path",
    "search-history": "Search history",
    "export-handoff": "Export handoff",
    "diagnostics": "Diagnostics",
    "cancel": "Back",
    "return-home": "Command center",
  };
  return labels[action];
}

function firstMissingGateAction(state: CodeFlowState): ActionSpec | null {
  const missing = gateLabels.find((item) => !state.gateItems[item.key]);
  if (!missing) return null;
  return { action: missing.action, label: `Resolve: ${missing.label}`, tone: "primary" };
}

function primaryActionsFor(state: CodeFlowState, canVerify: boolean): ActionSpec[] {
  switch (state.screen) {
    case "empty-onboarding":
      return [{ action: "scan-folders", label: "Add repo folder", tone: "primary" }];
    case "repo-discovery":
      return [{ action: "approve-discovery", label: "Approve selected repos", tone: "primary" }, { action: "cancel", label: "Back", tone: "quiet" }];
    case "command-center":
      return [{ action: "select-repo", label: "Select repo", tone: "primary" }, { action: "open-launch-gate", label: "Open Launch Gate" }, { action: "open-details", label: "Inspect repo" }];
    case "repository-detail":
      return [{ action: "run-checks", label: "Run checks", tone: "primary" }, { action: "commit-pr", label: "Commit / PR" }, { action: "open-launch-gate", label: "Launch Gate" }];
    case "checks":
      return [{ action: "checks-pass", label: "Checks passed — save evidence", tone: "primary" }, { action: "checks-fail", label: "Checks failed — keep output", tone: "danger" }, { action: "start-agent", label: "Ask agent to fix" }];
    case "start-agent":
      return [{ action: "review-complete", label: "Reviewer approved", tone: "primary" }, { action: "cancel", label: "Back", tone: "quiet" }];
    case "launch-gate": {
      const missingGateAction = firstMissingGateAction(state);
      return canVerify
        ? [{ action: "mark-verified", label: "Mark launch verified", tone: "primary" }, { action: "run-checks", label: "Review checks" }, { action: "start-agent", label: "Review agent" }]
        : [missingGateAction ?? { action: "run-checks", label: "Resolve missing evidence", tone: "primary" }, { action: "run-checks", label: "Resolve checks" }, { action: "start-agent", label: "Get review" }];
    }
    case "evidence":
      return [{ action: "production-verified", label: "Save production proof", tone: "primary" }, { action: "deploy-recorded", label: "Save deployment record" }, { action: "cancel", label: "Back", tone: "quiet" }];
    case "commit-pr":
      return [{ action: "commit-complete", label: "PR evidence attached", tone: "primary" }, { action: "github-auth", label: "Connect GitHub" }, { action: "cancel", label: "Back", tone: "quiet" }];
    case "deployment":
      return [{ action: "deploy-recorded", label: "Deployment recorded", tone: "primary" }, { action: "production-verified", label: "Production verified" }, { action: "cancel", label: "Back", tone: "quiet" }];
    default:
      return [{ action: "cancel", label: "Back", tone: "quiet" }, { action: "return-home", label: "Command center" }];
  }
}

function FlowButton({ action, children, dispatch, tone = "quiet" }: { action: CodeFlowAction; children: React.ReactNode; dispatch: (action: CodeFlowAction) => void; tone?: ActionSpec["tone"] }) {
  return <button className={`code-flow-button ${tone ?? "quiet"}`} onClick={() => dispatch(action)} type="button">{children}</button>;
}

function stepAction(stepId: CodeFlowScreenId): CodeFlowAction {
  switch (stepId) {
    case "empty-onboarding":
      return "scan-folders";
    case "repo-discovery":
      return "scan-folders";
    case "command-center":
      return "return-home";
    case "repository-detail":
      return "open-details";
    case "checks":
      return "run-checks";
    case "launch-gate":
      return "open-launch-gate";
    case "evidence":
      return "attach-evidence";
    case "commit-pr":
      return "commit-pr";
    case "deployment":
      return "deploy";
    default:
      return "return-home";
  }
}

function GuidedSteps({ state, dispatch }: { state: CodeFlowState; dispatch: (action: CodeFlowAction) => void }) {
  return (
    <ol className="code-flow-steps" aria-label="Code Workspace steps">
      {guidedSteps.map((step, index) => {
        const isActive = step.id === state.screen;
        return (
          <li key={step.id} className={isActive ? "active" : ""}>
            <button type="button" onClick={() => dispatch(stepAction(step.id))}>
              <span>{index + 1}</span>
              <strong>{step.label}</strong>
              <small>{step.description}</small>
            </button>
          </li>
        );
      })}
    </ol>
  );
}

function LaunchChecklist({ state, dispatch, canVerify }: { state: CodeFlowState; dispatch: (action: CodeFlowAction) => void; canVerify: boolean }) {
  return (
    <section className="code-flow-panel code-flow-checklist" aria-label="Launch checklist">
      <div className="code-flow-panel-heading">
        <p className="eyebrow">Launch checklist</p>
        <strong>{canVerify ? "Ready to verify" : "Not ready yet"}</strong>
      </div>
      <ul>
        {gateLabels.map((item) => {
          const done = state.gateItems[item.key];
          return (
            <li key={item.key} className={done ? "done" : "missing"}>
              <span aria-hidden="true">{done ? "✓" : ""}</span>
              <div>
                <strong>{item.label}</strong>
                <small>{item.help}</small>
              </div>
              {!done ? <button type="button" onClick={() => dispatch(item.action)}>Resolve</button> : null}
            </li>
          );
        })}
      </ul>
    </section>
  );
}

function AttentionRoutes({ dispatch }: { dispatch: (action: CodeFlowAction) => void }) {
  return (
    <section className="code-flow-panel" aria-label="Needs attention">
      <div className="code-flow-panel-heading">
        <p className="eyebrow">Needs attention</p>
        <strong>Click a problem to go there</strong>
      </div>
      <ul className="code-flow-attention-list">
        {attentionItems.map((item) => (
          <li key={item.type}>
            <button type="button" onClick={() => dispatch(codeFlowRouteForAttention(item.type))}>
              <strong>{item.label}</strong>
              <span>{item.detail}</span>
            </button>
          </li>
        ))}
      </ul>
    </section>
  );
}

export function CodeWorkspaceFlow({ nativeMode, nativeError, repoCount, actionStatus }: CodeWorkspaceFlowProps) {
  const [state, setState] = useState<CodeFlowState>(() => createInitialCodeFlowState(repoCount > 0));
  const screen = codeFlowScreenById[state.screen];
  const canVerify = launchGateCanMarkVerified(state);
  const dispatch = (action: CodeFlowAction) => setState((current) => reduceCodeFlow(current, action));
  const primaryActions = useMemo(() => primaryActionsFor(state, canVerify), [state, canVerify]);
  const nextStep = primaryActions[0];
  const completedCount = Object.values(state.gateItems).filter(Boolean).length;

  return (
    <section className="code-flow" aria-label="Code Workspace guided flow" data-code-flow-screen={state.screen}>
      <header className="code-flow-hero compact">
        <div>
          <p className="eyebrow">Code Workspace</p>
          <h3>One guided flow from repo to verified launch</h3>
          <p>Choose one next action at a time. Advanced routes are still available, but the main path stays clear.</p>
        </div>
        <aside aria-label="Workspace mode">
          <strong>{nativeMode === "ready" ? "Native local" : nativeMode === "loading" ? "Checking native bridge" : "Browser preview"}</strong>
          <span>{nativeMode === "ready" ? `${repoCount} repo profile${repoCount === 1 ? "" : "s"} available` : nativeError ?? "Preview cannot fake files, GitHub, Git, or deploys."}</span>
        </aside>
      </header>

      <div className="code-flow-notice" role="status">
        Browser preview only proves navigation and evidence gating. Real Finder/Git/GitHub/deploy actions stay native and fail closed.
      </div>

      <GuidedSteps state={state} dispatch={dispatch} />

      <section className="code-flow-guided-layout">
        <main className="code-flow-screen" aria-label={screen.title}>
          <div className="code-flow-screen-header simplified">
            <div>
              <p className="eyebrow">Current step</p>
              <h4>{screen.title}</h4>
              <p>{screen.purpose}</p>
            </div>
            <span className={canVerify ? "code-flow-verdict ready" : "code-flow-verdict blocked"}>{canVerify ? "Launch ready" : `${completedCount}/6 gates done`}</span>
          </div>

          <div className="code-flow-next-action">
            <div>
              <span>Next best action</span>
              <strong>{nextStep?.label ?? "Choose an action"}</strong>
              <small>{screen.evidenceRule}</small>
            </div>
            {nextStep ? <FlowButton action={nextStep.action} dispatch={dispatch} tone="primary">{nextStep.label}</FlowButton> : null}
          </div>

          <section className="code-flow-action-row" aria-label="Other useful actions">
            {primaryActions.slice(1).map((item) => (
              <FlowButton key={`${item.action}-${item.label}`} action={item.action} dispatch={dispatch} tone={item.tone}>{item.label}</FlowButton>
            ))}
          </section>

          <dl className="code-flow-summary-list" aria-label="Current state summary">
            <div><dt>Repo</dt><dd>{state.selectedRepoId ?? "No repo selected"}</dd></div>
            <div><dt>Checks</dt><dd>{state.checksStatus}</dd></div>
            <div><dt>Agent</dt><dd>{state.agentStatus}</dd></div>
            <div><dt>Return</dt><dd>{state.returnTarget}</dd></div>
            {actionStatus ? <div><dt>Native action</dt><dd>{actionStatus}</dd></div> : null}
          </dl>

          <div className="code-flow-event">
            <span>Last event</span>
            <p>{state.lastEvent}</p>
          </div>
        </main>

        <aside className="code-flow-side" aria-label="Code Workspace side panels">
          <LaunchChecklist state={state} dispatch={dispatch} canVerify={canVerify} />
          <AttentionRoutes dispatch={dispatch} />
          <section className="code-flow-panel code-flow-utility" aria-label="Tools">
            <div className="code-flow-panel-heading">
              <p className="eyebrow">Tools</p>
              <strong>Less common routes</strong>
            </div>
            <div className="code-flow-tool-grid">
              {["github-auth", "fix-path", "search-history", "export-handoff", "diagnostics", "return-home"].map((action) => (
                <button key={action} type="button" onClick={() => dispatch(action as CodeFlowAction)}>{dispatchLabel(action as CodeFlowAction)}</button>
              ))}
            </div>
          </section>
        </aside>
      </section>
    </section>
  );
}
