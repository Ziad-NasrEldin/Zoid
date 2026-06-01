import { invoke } from "@tauri-apps/api/core";
import { useEffect, useMemo, useState } from "react";
import "./App.css";

type WorkspaceRecord = {
  id: string;
  label: string;
  description: string;
  position: number;
};

const fallbackWorkspaces: WorkspaceRecord[] = [
  { id: "today", label: "Today", description: "Command center, attention, and current work.", position: 0 },
  { id: "tasks", label: "Tasks", description: "First-class tasks, review states, and follow-ups.", position: 1 },
  { id: "notes", label: "Notes", description: "Markdown notes with local metadata.", position: 2 },
  { id: "agents", label: "Agents", description: "CLI profiles, sessions, runs, and reviews.", position: 3 },
  { id: "code", label: "Code", description: "Repositories, Launch Gate, and git work.", position: 4 },
  { id: "content", label: "Content", description: "Planning, review, and OmniSocials publishing state.", position: 5 },
  { id: "automations", label: "Automations", description: "Visible recurring jobs and run history.", position: 6 },
  { id: "business", label: "Business", description: "Contacts, companies, follow-ups, and linked work.", position: 7 },
  { id: "products", label: "Products", description: "First-class product hubs and timelines.", position: 8 },
  { id: "files", label: "Files", description: "Local file manager and Zoid-aware attachments.", position: 9 },
  { id: "browser", label: "Browser", description: "Work webview/capture workspace.", position: 10 },
  { id: "inbox", label: "Inbox", description: "Notifications, approvals, blockers, and Gmail state.", position: 11 },
  { id: "calendar", label: "Calendar", description: "Built-in calendar with Apple Calendar integration gates.", position: 12 },
  { id: "history", label: "History", description: "Universal timeline and linked event history.", position: 13 },
];

type ActionPolicyDecision = {
  category: string;
  policy: string;
  reviewer_required: string;
  human_confirmation: string;
  reason: string;
};

type SecureFoundationStatus = {
  redaction_ready: boolean;
  safe_logging_ready: boolean;
  action_policy_ready: boolean;
  event_writer_ready: boolean;
  keychain_status: string;
  sample_policy: ActionPolicyDecision;
};

type VisibleUserPathStatus = {
  root: string;
  starter_directories: string[];
};

type AppSupportPathStatus = {
  root: string;
  logs_dir: string;
  database_parent: string;
  database_path: string;
  config_dir: string;
  config_path: string;
};

type FoundationStatus = {
  visible_root: string;
  app_support_dir: string;
  database_path: string;
  logs_dir: string;
  config_dir: string;
  config_path: string;
  visible_user: VisibleUserPathStatus;
  app_support: AppSupportPathStatus;
  migration_version: number;
  workspace_count: number;
  event_count: number;
  workspaces: WorkspaceRecord[];
  secure_services: SecureFoundationStatus;
};

const integrationStates = [
  { name: "CLI profiles", state: "not configured", note: "Local command wiring is disabled until a real profile is added." },
  { name: "Gmail", state: "not configured", note: "Read and send flows remain unavailable until explicitly configured." },
  { name: "Apple Calendar", state: "needs permission", note: "Calendar access is gated by native app validation and permission." },
  { name: "OmniSocials", state: "not configured", note: "Publishing remains blocked without credentials and review policy." },
];

const workspaceGlyphs: Record<string, string> = {
  agents: "A",
  automations: "ƒ",
  browser: "⌘",
  business: "B",
  calendar: "C",
  code: "</>",
  content: "P",
  files: "F",
  history: "H",
  inbox: "I",
  notes: "N",
  products: "R",
  tasks: "✓",
  today: "•",
};

const readinessLabel = (ready: boolean) => (ready ? "Ready" : "Blocked");

function App() {
  const [activeWorkspace, setActiveWorkspace] = useState("today");
  const [status, setStatus] = useState<FoundationStatus | null>(null);
  const [statusError, setStatusError] = useState<string | null>(null);

  useEffect(() => {
    invoke<FoundationStatus>("get_foundation_status")
      .then(setStatus)
      .catch(() => {
        setStatusError("Native foundation status is available inside the packaged Tauri app. Browser preview is UI-only.");
      });
  }, []);

  const workspaces = useMemo(() => {
    const workspaceRecords = status ? status.workspaces : fallbackWorkspaces;

    return [...workspaceRecords].sort((a, b) => a.position - b.position);
  }, [status]);

  const active = useMemo(
    () => workspaces.find((workspace) => workspace.id === activeWorkspace) ?? workspaces[0] ?? null,
    [activeWorkspace, workspaces],
  );

  const nativeState = statusError ? "Preview" : status ? "Native ready" : "Checking";
  const statusTone = statusError ? "blocked" : status ? "ready" : "pending";
  const workspaceSourceLabel = status ? "Native registry" : statusError ? "Preview fallback" : "Preview while checking native status";
  const activeWorkspaceLabel = active?.label ?? "No workspaces registered";
  const activeWorkspaceDescription = active?.description ?? "The native workspace registry is empty.";
  const starterDirectoryCount = status?.visible_user.starter_directories.length ?? 0;

  return (
    <main className="zoid-shell">
      <aside className="sidebar" aria-label="Zoid workspaces">
        <div className="window-controls" aria-hidden="true">
          <span className="control close" />
          <span className="control minimize" />
          <span className="control zoom" />
        </div>

        <div className="brand-lockup">
          <div className="brand-mark">Z</div>
          <div>
            <p className="eyebrow">Local-first workspace</p>
            <h1>Zoid</h1>
          </div>
        </div>

        <nav className="workspace-nav">
          {workspaces.length > 0 ? workspaces.map((workspace) => (
            <button
              aria-current={workspace.id === active?.id ? "page" : undefined}
              className={workspace.id === active?.id ? "workspace-item active" : "workspace-item"}
              key={workspace.id}
              onClick={() => setActiveWorkspace(workspace.id)}
              type="button"
            >
              <span className="workspace-glyph" aria-hidden="true">{workspaceGlyphs[workspace.id] ?? workspace.label.slice(0, 1)}</span>
              <span className="workspace-copy">
                <strong>{workspace.label}</strong>
                <small>{workspace.description}</small>
              </span>
            </button>
          )) : <p className="muted-copy">No native workspaces registered.</p>}
        </nav>

        <div className="sidebar-footer">
          <span className={`status-dot ${statusTone}`} />
          <div>
            <strong>{nativeState}</strong>
            <small>{status ? `${status.workspace_count} registered workspaces` : statusError ? "Browser preview fallback" : "Checking native registry"}</small>
          </div>
        </div>
      </aside>

      <section className="app-stage">
        <header className="toolbar">
          <div className="toolbar-title">
            <p className="eyebrow">Workspace</p>
            <h2>{activeWorkspaceLabel}</h2>
          </div>

          <div className="toolbar-center" role="search">
            <span aria-hidden="true">⌕</span>
            <input disabled aria-label="Search is unavailable" placeholder="Search will appear when indexing is available" />
          </div>

          <div className="toolbar-actions" aria-label="App status">
            <span className={`badge ${statusTone}`}>{nativeState}</span>
            <span className="status-pill">Review gate enforced</span>
          </div>
        </header>

        <div className="split-view">
          <section className="primary-pane" aria-label="Workspace overview">
            <article className="hero-card">
              <div>
                <p className="eyebrow">Active workspace</p>
                <h3>{activeWorkspaceLabel}</h3>
                <p>{activeWorkspaceDescription}</p>
              </div>
              <button className="secondary-action" disabled type="button">
                Open module when available
              </button>
            </article>

            <section className="dashboard-grid">
              <article className="card large-card">
                <div className="card-header">
                  <div>
                    <p className="eyebrow">Foundation status</p>
                    <h3>Local app state</h3>
                  </div>
                  <span className={`badge ${statusTone}`}>{statusError ? "Preview" : status ? "Ready" : "Loading"}</span>
                </div>

                {statusError ? (
                  <p className="error-copy">{statusError}</p>
                ) : status ? (
                  <dl className="status-list">
                    <div><dt>Visible root</dt><dd>{status.visible_user.root}</dd></div>
                    <div><dt>Starter directories</dt><dd>{starterDirectoryCount}</dd></div>
                    <div><dt>App support</dt><dd>{status.app_support.root}</dd></div>
                    <div><dt>SQLite DB</dt><dd>{status.app_support.database_path}</dd></div>
                    <div><dt>Logs</dt><dd>{status.app_support.logs_dir}</dd></div>
                    <div><dt>Config</dt><dd>{status.app_support.config_dir}</dd></div>
                    <div><dt>Migration version</dt><dd>{status.migration_version}</dd></div>
                    <div><dt>Registered workspaces</dt><dd>{status.workspace_count}</dd></div>
                    <div><dt>Foundation events</dt><dd>{status.event_count}</dd></div>
                  </dl>
                ) : (
                  <p className="muted-copy">Creating local folders and reading migration state…</p>
                )}
              </article>

              <article className="card">
                <p className="eyebrow">Workspace registry</p>
                <h3>Real registry, calm preview</h3>
                <p>The sidebar is driven by native workspace records when available, with browser preview fallbacks only outside Tauri.</p>
                <div className="registry-list">
                  {workspaces.length > 0 ? workspaces.map((workspace) => (
                    <button
                      className={workspace.id === active?.id ? "registry-chip active" : "registry-chip"}
                      key={workspace.id}
                      onClick={() => setActiveWorkspace(workspace.id)}
                      type="button"
                    >
                      {workspace.label}
                    </button>
                  )) : <p className="muted-copy">The native registry returned no workspaces.</p>}
                </div>
              </article>

              <article className="card">
                <p className="eyebrow">Unavailable actions</p>
                <h3>Nothing is simulated</h3>
                <p>Search, module opening, and integration activity remain disabled until their real local capabilities exist.</p>
                <div className="empty-state">
                  <span aria-hidden="true">⌘</span>
                  <p>Preview shell only. Consequential actions stay behind native status and review policy.</p>
                </div>
              </article>
            </section>
          </section>

          <aside className="inspector-pane" aria-label="Workspace details">
            <article className="inspector-card active-summary">
              <p className="eyebrow">Details</p>
              <h3>{activeWorkspaceLabel}</h3>
              <p>{activeWorkspaceDescription}</p>
              <dl>
                <div><dt>ID</dt><dd>{active?.id ?? "—"}</dd></div>
                <div><dt>Position</dt><dd>{active?.position ?? "—"}</dd></div>
                <div><dt>Source</dt><dd>{workspaceSourceLabel}</dd></div>
              </dl>
            </article>

            <article className="inspector-card">
              <p className="eyebrow">Secure foundation</p>
              <h3>Local safeguards</h3>
              {status ? (
                <ul className="security-list">
                  <li><strong>Redaction</strong><span className={status.secure_services.redaction_ready ? "ready" : "blocked"}>{readinessLabel(status.secure_services.redaction_ready)}</span></li>
                  <li><strong>Safe logging</strong><span className={status.secure_services.safe_logging_ready ? "ready" : "blocked"}>{readinessLabel(status.secure_services.safe_logging_ready)}</span></li>
                  <li><strong>Action policy</strong><span className={status.secure_services.action_policy_ready ? "ready" : "blocked"}>{readinessLabel(status.secure_services.action_policy_ready)}</span></li>
                  <li><strong>Event writer</strong><span className={status.secure_services.event_writer_ready ? "ready" : "blocked"}>{readinessLabel(status.secure_services.event_writer_ready)}</span></li>
                  <li><strong>Keychain</strong><span className="neutral">{status.secure_services.keychain_status}</span></li>
                </ul>
              ) : (
                <p className="muted-copy">Secure service readiness is reported by the native app only.</p>
              )}
              {status ? (
                <p className="policy-note">
                  Sample policy: {status.secure_services.sample_policy.category} requires {status.secure_services.sample_policy.human_confirmation.replace(/_/g, " ")} confirmation.
                </p>
              ) : null}
            </article>

            <article className="inspector-card">
              <p className="eyebrow">Integration truth</p>
              <h3>Local status</h3>
              <ul className="integration-list">
                {integrationStates.map((integration) => (
                  <li key={integration.name}>
                    <div>
                      <strong>{integration.name}</strong>
                      <span>{integration.state}</span>
                    </div>
                    <p>{integration.note}</p>
                  </li>
                ))}
              </ul>
            </article>
          </aside>
        </div>
      </section>
    </main>
  );
}

export default App;
