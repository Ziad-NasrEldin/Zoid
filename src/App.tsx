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
  { name: "CLI profiles", state: "not_configured", note: "Add a local CLI command later. No AI backend is assumed." },
  { name: "Gmail", state: "not_configured", note: "Read/send flows remain disabled until configured." },
  { name: "Apple Calendar", state: "needs_permission", note: "EventKit is gated by native-app validation." },
  { name: "OmniSocials", state: "not_configured", note: "Publishing cannot run without credentials and review policy." },
];

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

  const workspaces = status?.workspaces.length ? status.workspaces : fallbackWorkspaces;

  const active = useMemo(
    () => workspaces.find((workspace) => workspace.id === activeWorkspace) ?? workspaces[0],
    [activeWorkspace, workspaces],
  );

  return (
    <main className="zoid-shell">
      <aside className="sidebar" aria-label="Zoid workspaces">
        <div className="brand-lockup">
          <div className="brand-mark">Z</div>
          <div>
            <p className="eyebrow">Local-first workspace</p>
            <h1>Zoid</h1>
          </div>
        </div>

        <nav className="workspace-nav">
          {workspaces.map((workspace) => (
            <button
              className={workspace.id === activeWorkspace ? "workspace-item active" : "workspace-item"}
              key={workspace.id}
              onClick={() => setActiveWorkspace(workspace.id)}
              type="button"
            >
              <span>{workspace.label}</span>
            </button>
          ))}
        </nav>
      </aside>

      <section className="content-panel">
        <header className="topbar">
          <div>
            <p className="eyebrow">Today / Foundation Slice</p>
            <h2>{active.label}</h2>
          </div>
          <div className="status-pill">Review gate required for consequential work</div>
        </header>

        <section className="hero-card">
          <div>
            <p className="eyebrow">Active workspace</p>
            <h3>{active.label}</h3>
            <p>{active.description}</p>
          </div>
          <button className="primary-action" disabled type="button">
            Useful action pending real module
          </button>
        </section>

        <section className="dashboard-grid">
          <article className="card large-card">
            <div className="card-header">
              <div>
                <p className="eyebrow">Foundation status</p>
                <h3>Local app state</h3>
              </div>
              <span className={statusError ? "badge blocked" : "badge"}>{statusError ? "preview" : status ? "ready" : "loading"}</span>
            </div>

            {statusError ? (
              <p className="error-copy">{statusError}</p>
            ) : status ? (
              <dl className="status-list">
                <div><dt>Visible root</dt><dd>{status.visible_user.root}</dd></div>
                <div><dt>Starter directories</dt><dd>{status.visible_user.starter_directories.length}</dd></div>
                <div><dt>App support</dt><dd>{status.app_support.root}</dd></div>
                <div><dt>SQLite DB</dt><dd>{status.app_support.database_path}</dd></div>
                <div><dt>Logs</dt><dd>{status.app_support.logs_dir}</dd></div>
                <div><dt>Config</dt><dd>{status.app_support.config_dir}</dd></div>
                <div><dt>Migration version</dt><dd>{status.migration_version}</dd></div>
                <div><dt>Registered workspaces</dt><dd>{status.workspace_count}</dd></div>
                <div><dt>Foundation events</dt><dd>{status.event_count}</dd></div>
              </dl>
            ) : (
              <p>Creating local folders and reading migration state…</p>
            )}
          </article>

          <article className="card">
            <p className="eyebrow">Workspace registry</p>
            <h3>All core workspaces are present</h3>
            <div className="mini-grid">
              {workspaces.map((workspace) => (
                <span key={workspace.id}>{workspace.label}</span>
              ))}
            </div>
          </article>

          <article className="card">
            <p className="eyebrow">Secure foundation</p>
            <h3>Services are centralized</h3>
            {status ? (
              <ul className="security-list">
                <li><strong>Redaction</strong><span>{status.secure_services.redaction_ready ? "ready" : "blocked"}</span></li>
                <li><strong>Safe logging</strong><span>{status.secure_services.safe_logging_ready ? "ready" : "blocked"}</span></li>
                <li><strong>Action policy</strong><span>{status.secure_services.action_policy_ready ? "ready" : "blocked"}</span></li>
                <li><strong>Event writer</strong><span>{status.secure_services.event_writer_ready ? "ready" : "blocked"}</span></li>
                <li><strong>Keychain</strong><span className="warning">unverified</span></li>
              </ul>
            ) : (
              <p>Secure services are reported by the native app only.</p>
            )}
            {status ? <p className="policy-note">Sample policy: {status.secure_services.sample_policy.category} requires {status.secure_services.sample_policy.human_confirmation.replace(/_/g, " ")} confirmation.</p> : null}
          </article>

          <article className="card">
            <p className="eyebrow">Integration truth</p>
            <h3>No fake connected states</h3>
            <ul className="integration-list">
              {integrationStates.map((integration) => (
                <li key={integration.name}>
                  <strong>{integration.name}</strong>
                  <span>{integration.state}</span>
                  <p>{integration.note}</p>
                </li>
              ))}
            </ul>
          </article>
        </section>
      </section>
    </main>
  );
}

export default App;
