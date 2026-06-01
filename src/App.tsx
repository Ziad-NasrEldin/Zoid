import { invoke } from "@tauri-apps/api/core";
import type { ReactNode } from "react";
import { useEffect, useMemo, useState } from "react";
import "./App.css";
import {
  buildSettingsStatusShellView,
  defaultIntegrationStates,
  type FoundationStatus,
  type IntegrationState,
  type SettingsStatusItem,
  type SettingsStatusShellView,
} from "./settingsStatus";
import { buildTodayFoundationView, type TodayFoundationView, type TodayWidgetView } from "./todayFoundation";

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

type WorkspaceRegistrySource = "native" | "fallback" | "checking";

type WorkspaceRegistryView = {
  countLabel: string;
  source: WorkspaceRegistrySource;
  sourceLabel: string;
  truthCopy: string;
  workspaces: WorkspaceRecord[];
};

const integrationStates: IntegrationState[] = defaultIntegrationStates;

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

function formatWorkspaceCount(count: number) {
  return `${count} workspace${count === 1 ? "" : "s"}`;
}

function sortWorkspaces(workspaces: WorkspaceRecord[]) {
  return [...workspaces].sort((a, b) => a.position - b.position);
}

function buildWorkspaceRegistryView(status: FoundationStatus | null, statusError: string | null): WorkspaceRegistryView {
  if (status) {
    const workspaces = sortWorkspaces(status.workspaces);
    const countLabel = formatWorkspaceCount(workspaces.length);

    return {
      countLabel,
      source: "native",
      sourceLabel: "Native registry",
      truthCopy: `Rendering ${countLabel} returned by get_foundation_status. Browser preview fallback is not mixed into native data.`,
      workspaces,
    };
  }

  if (statusError) {
    const workspaces = sortWorkspaces(fallbackWorkspaces);
    const countLabel = formatWorkspaceCount(workspaces.length);

    return {
      countLabel,
      source: "fallback",
      sourceLabel: "Browser preview fallback",
      truthCopy: `Showing ${countLabel} from static browser preview data because native status is unavailable outside Tauri.`,
      workspaces,
    };
  }

  const workspaces = sortWorkspaces(fallbackWorkspaces);
  const countLabel = formatWorkspaceCount(workspaces.length);

  return {
    countLabel,
    source: "checking",
    sourceLabel: "Checking native registry",
    truthCopy: `Temporarily showing ${countLabel} from browser preview data while get_foundation_status is loading.`,
    workspaces,
  };
}

type StatusTone = "ready" | "blocked" | "pending";

type SidebarItemProps = {
  workspace: WorkspaceRecord;
  active: boolean;
  glyph: string;
  onSelect: () => void;
};

function SidebarItem({ workspace, active, glyph, onSelect }: SidebarItemProps) {
  return (
    <button
      aria-current={active ? "page" : undefined}
      className={active ? "workspace-item active" : "workspace-item"}
      onClick={onSelect}
      type="button"
    >
      <span className="workspace-glyph" aria-hidden="true">{glyph}</span>
      <span className="workspace-copy">
        <strong>{workspace.label}</strong>
        <small>{workspace.description}</small>
      </span>
    </button>
  );
}

type StatusBadgeProps = {
  children: ReactNode;
  tone?: StatusTone;
  className?: string;
};

function StatusBadge({ children, tone = "pending", className = "" }: StatusBadgeProps) {
  return <span className={`badge ${tone}${className ? ` ${className}` : ""}`}>{children}</span>;
}

type WorkspaceHeaderProps = {
  title: string;
  nativeState: string;
  statusTone: StatusTone;
};

function WorkspaceHeader({ title, nativeState, statusTone }: WorkspaceHeaderProps) {
  return (
    <header className="toolbar">
      <div className="toolbar-title">
        <p className="eyebrow">Workspace</p>
        <h2>{title}</h2>
      </div>

      <div className="toolbar-center" role="search">
        <span aria-hidden="true">⌕</span>
        <input disabled aria-label="Search is unavailable" placeholder="Search will appear when indexing is available" />
      </div>

      <div className="toolbar-actions" aria-label="App status">
        <StatusBadge tone={statusTone}>{nativeState}</StatusBadge>
        <span className="status-pill">Review gate enforced</span>
      </div>
    </header>
  );
}

type InfoCardProps = {
  children: ReactNode;
  className?: string;
};

function InfoCard({ children, className = "" }: InfoCardProps) {
  return <article className={`card${className ? ` ${className}` : ""}`}>{children}</article>;
}

type EmptyStateProps = {
  icon: string;
  children: ReactNode;
};

function EmptyState({ icon, children }: EmptyStateProps) {
  return (
    <div className="empty-state">
      <span aria-hidden="true">{icon}</span>
      <p>{children}</p>
    </div>
  );
}

type BlockerStateProps = {
  icon?: string;
  children: ReactNode;
};

function BlockerState({ icon = "!", children }: BlockerStateProps) {
  return (
    <div className="empty-state" role="alert" aria-live="polite">
      <span aria-hidden="true">{icon}</span>
      <p className="error-copy">{children}</p>
    </div>
  );
}

type InspectorPanelProps = {
  children: ReactNode;
  label: string;
};

function InspectorPanel({ children, label }: InspectorPanelProps) {
  return <aside className="inspector-pane" aria-label={label}>{children}</aside>;
}

type InspectorCardProps = {
  children: ReactNode;
  className?: string;
};

function InspectorCard({ children, className = "" }: InspectorCardProps) {
  return <article className={`inspector-card${className ? ` ${className}` : ""}`}>{children}</article>;
}

type SettingsStatusListProps = {
  items: SettingsStatusItem[];
};

function SettingsStatusList({ items }: SettingsStatusListProps) {
  return (
    <dl className="settings-status-list">
      {items.map((item) => (
        <div key={`${item.label}:${item.value}`}>
          <dt>{item.label}</dt>
          <dd>
            {item.tone ? <StatusBadge tone={item.tone}>{item.value}</StatusBadge> : item.value}
          </dd>
        </div>
      ))}
    </dl>
  );
}

type SettingsStatusShellProps = {
  view: SettingsStatusShellView;
};

function SettingsStatusShell({ view }: SettingsStatusShellProps) {
  return (
    <InspectorCard className="settings-status-shell">
      <div className="card-header compact">
        <div>
          <p className="eyebrow">Settings/status</p>
          <h3>{view.modeLabel}</h3>
        </div>
        <StatusBadge tone={view.mode === "native" ? "ready" : view.mode === "checking" ? "pending" : "blocked"}>
          {view.mode === "native" ? "Native" : view.mode === "checking" ? "Checking" : "Preview"}
        </StatusBadge>
      </div>
      <p>{view.summary}</p>

      <section className="settings-status-section" aria-label="Paths">
        <p className="eyebrow">Paths</p>
        <SettingsStatusList items={view.paths} />
      </section>

      <section className="settings-status-section" aria-label="Database, migrations, and events">
        <p className="eyebrow">DB / migrations / events</p>
        <SettingsStatusList items={view.database} />
      </section>

      <section className="settings-status-section" aria-label="Keychain readiness">
        <p className="eyebrow">Keychain</p>
        <SettingsStatusList items={view.keychain} />
      </section>

      <section className="settings-status-section" aria-label="Safeguards">
        <p className="eyebrow">Safeguards</p>
        <SettingsStatusList items={view.safeguards} />
      </section>

      <section className="settings-status-section" aria-label="Policy summary">
        <p className="eyebrow">Policy summary</p>
        <dl className="settings-status-list">
          <div><dt>Category</dt><dd>{view.policy.category}</dd></div>
          <div><dt>Policy</dt><dd>{view.policy.policy}</dd></div>
          <div><dt>Reviewer</dt><dd>{view.policy.reviewerRequired}</dd></div>
          <div><dt>Confirmation</dt><dd>{view.policy.humanConfirmation}</dd></div>
          <div><dt>Reason</dt><dd>{view.policy.reason}</dd></div>
        </dl>
      </section>

      <section className="settings-status-section" aria-label="Event writer status">
        <p className="eyebrow">Events</p>
        <SettingsStatusList items={view.events} />
      </section>

      <section className="settings-status-section" aria-label="Integrations">
        <p className="eyebrow">Integrations</p>
        <ul className="integration-list compact-list">
          {view.integrations.map((integration) => (
            <li key={integration.name}>
              <div>
                <strong>{integration.name}</strong>
                <span>{integration.state}</span>
              </div>
              <p>{integration.note}</p>
            </li>
          ))}
        </ul>
      </section>
    </InspectorCard>
  );
}

type TodayMetricProps = {
  label: string;
  value: string;
};

function TodayMetric({ label, value }: TodayMetricProps) {
  return (
    <div className="today-metric">
      <dt>{label}</dt>
      <dd>{value}</dd>
    </div>
  );
}

type TodayWidgetCardProps = {
  widget: TodayWidgetView;
};

function TodayWidgetCard({ widget }: TodayWidgetCardProps) {
  return (
    <InfoCard className="today-widget-card">
      <div className="card-header compact">
        <div>
          <p className="eyebrow">Today widget</p>
          <h3>{widget.title}</h3>
        </div>
        <StatusBadge tone={widget.tone}>{widget.status}</StatusBadge>
      </div>
      <p>{widget.copy}</p>
    </InfoCard>
  );
}

type TodayWorkspaceOverviewProps = {
  activeWorkspaceDescription: string;
  activeWorkspaceLabel: string;
  status: FoundationStatus | null;
  statusError: string | null;
  statusTone: StatusTone;
  todayView: TodayFoundationView;
  workspaceRegistry: WorkspaceRegistryView;
  workspaces: WorkspaceRecord[];
  activeWorkspaceId: string | undefined;
  onSelectWorkspace: (workspaceId: string) => void;
};

function TodayWorkspaceOverview({
  activeWorkspaceDescription,
  activeWorkspaceLabel,
  status,
  statusError,
  statusTone,
  todayView,
  workspaceRegistry,
  workspaces,
  activeWorkspaceId,
  onSelectWorkspace,
}: TodayWorkspaceOverviewProps) {
  return (
    <>
      <article className="hero-card today-hero">
        <div>
          <p className="eyebrow">Today foundation</p>
          <h3>{activeWorkspaceLabel}</h3>
          <p>{todayView.heroCopy}</p>
        </div>
        <StatusBadge tone={statusTone}>{todayView.heroStatus}</StatusBadge>
      </article>

      <section className="dashboard-grid today-dashboard">
        <InfoCard className="large-card today-foundation-card">
          <div className="card-header">
            <div>
              <p className="eyebrow">Foundation overview</p>
              <h3>{todayView.sourceLabel}</h3>
            </div>
            <StatusBadge tone={statusTone}>{statusError ? "Preview" : status ? "Native" : "Checking"}</StatusBadge>
          </div>

          <p>{activeWorkspaceDescription}</p>

          <dl className="today-metric-grid">
            <TodayMetric label="Registered workspaces" value={todayView.metrics.registeredWorkspaces} />
            <TodayMetric label="Foundation events" value={todayView.metrics.foundationEvents} />
            <TodayMetric label="Migration version" value={todayView.metrics.migrationVersion} />
            <TodayMetric label="Starter directories" value={todayView.metrics.starterDirectories} />
            <TodayMetric label="Secure safeguards" value={todayView.metrics.secureSafeguards} />
            <TodayMetric label="Keychain status" value={todayView.metrics.keychainStatus} />
          </dl>

          {status ? (
            <dl className="status-list today-path-list">
              <div><dt>Visible root</dt><dd>{status.visible_user.root}</dd></div>
              <div><dt>Starter directories</dt><dd>{status.visible_user.starter_directories.join(", ") || "—"}</dd></div>
              <div><dt>App support</dt><dd>{status.app_support.root}</dd></div>
              <div><dt>SQLite DB</dt><dd>{status.app_support.database_path}</dd></div>
              <div><dt>Logs</dt><dd>{status.app_support.logs_dir}</dd></div>
              <div><dt>Config</dt><dd>{status.app_support.config_dir}</dd></div>
            </dl>
          ) : (
            <EmptyState icon="⌁">Native-only paths, migrations, events, secure readiness, and keychain status are unavailable in this browser/checking state.</EmptyState>
          )}

          <EmptyState icon="∞">Sample policy: {todayView.metrics.samplePolicy}. Consequential actions remain gated; no tasks, runs, or completions are simulated.</EmptyState>
        </InfoCard>

        <InfoCard>
          <p className="eyebrow">Workspace registry</p>
          <h3>{workspaceRegistry.sourceLabel}</h3>
          <p>{workspaceRegistry.truthCopy}</p>
          <div className={`registry-meta ${workspaceRegistry.source}`}>
            <span>{workspaceRegistry.countLabel}</span>
            <span>{status ? "Real native registry" : "UI-only preview data"}</span>
          </div>
          <div className="registry-list">
            {workspaces.length > 0 ? workspaces.map((workspace) => (
              <button
                aria-current={workspace.id === activeWorkspaceId ? "page" : undefined}
                className={workspace.id === activeWorkspaceId ? "registry-chip active" : "registry-chip"}
                key={workspace.id}
                onClick={() => onSelectWorkspace(workspace.id)}
                type="button"
              >
                {workspace.label}
              </button>
            )) : <p className="muted-copy">The native registry returned no workspaces.</p>}
          </div>
        </InfoCard>

        <TodayWidgetCard widget={todayView.widgets.tasks} />
        <TodayWidgetCard widget={todayView.widgets.runs} />
        <TodayWidgetCard widget={todayView.widgets.inbox} />
        <TodayWidgetCard widget={todayView.widgets.integrations} />

        <InfoCard className="today-widget-card">
          <p className="eyebrow">Integration states</p>
          <h3>Truthful setup</h3>
          <ul className="integration-list compact-list">
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
        </InfoCard>
      </section>
    </>
  );
}

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

  const workspaceRegistry = useMemo(() => buildWorkspaceRegistryView(status, statusError), [status, statusError]);
  const workspaces = workspaceRegistry.workspaces;

  const active = useMemo(
    () => workspaces.find((workspace) => workspace.id === activeWorkspace) ?? workspaces[0] ?? null,
    [activeWorkspace, workspaces],
  );

  const nativeState = statusError ? "Preview" : status ? "Native ready" : "Checking";
  const statusTone = statusError ? "blocked" : status ? "ready" : "pending";
  const workspaceSourceLabel = workspaceRegistry.sourceLabel;
  const activeWorkspaceLabel = active?.label ?? "No workspaces registered";
  const activeWorkspaceDescription = active?.description ?? "The native workspace registry is empty.";
  const starterDirectoryCount = status?.visible_user.starter_directories.length ?? 0;
  const todayView = useMemo(
    () => buildTodayFoundationView({
      countLabel: workspaceRegistry.countLabel,
      source: workspaceRegistry.source,
      sourceLabel: workspaceRegistry.sourceLabel,
      status,
    }),
    [status, workspaceRegistry.countLabel, workspaceRegistry.source, workspaceRegistry.sourceLabel],
  );
  const settingsStatusView = useMemo(
    () => buildSettingsStatusShellView({
      mode: status ? "native" : statusError ? "preview" : "checking",
      status,
      integrations: integrationStates,
    }),
    [status, statusError],
  );

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
            <SidebarItem
              active={workspace.id === active?.id}
              glyph={workspaceGlyphs[workspace.id] ?? workspace.label.slice(0, 1)}
              key={workspace.id}
              onSelect={() => setActiveWorkspace(workspace.id)}
              workspace={workspace}
            />
          )) : <p className="muted-copy">No native workspaces registered.</p>}
        </nav>

        <div className="sidebar-footer">
          <span className={`status-dot ${statusTone}`} />
          <div>
            <strong>{nativeState}</strong>
            <small>{workspaceRegistry.sourceLabel}: {workspaceRegistry.countLabel}</small>
          </div>
        </div>
      </aside>

      <section className="app-stage">
        <WorkspaceHeader nativeState={nativeState} statusTone={statusTone} title={activeWorkspaceLabel} />

        <div className="split-view">
          <section className="primary-pane" aria-label="Workspace overview">
            {active?.id === "today" ? (
              <TodayWorkspaceOverview
                activeWorkspaceDescription={activeWorkspaceDescription}
                activeWorkspaceId={active?.id}
                activeWorkspaceLabel={activeWorkspaceLabel}
                onSelectWorkspace={setActiveWorkspace}
                status={status}
                statusError={statusError}
                statusTone={statusTone}
                todayView={todayView}
                workspaceRegistry={workspaceRegistry}
                workspaces={workspaces}
              />
            ) : (
            <>
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
              <InfoCard className="large-card">
                <div className="card-header">
                  <div>
                    <p className="eyebrow">Foundation status</p>
                    <h3>Local app state</h3>
                  </div>
                  <StatusBadge tone={statusTone}>{statusError ? "Preview" : status ? "Ready" : "Loading"}</StatusBadge>
                </div>

                {statusError ? (
                  <BlockerState>{statusError}</BlockerState>
                ) : status ? (
                  <dl className="status-list">
                    <div><dt>Visible root</dt><dd>{status.visible_user.root}</dd></div>
                    <div><dt>Starter directories</dt><dd>{starterDirectoryCount}</dd></div>
                    <div><dt>App support</dt><dd>{status.app_support.root}</dd></div>
                    <div><dt>SQLite DB</dt><dd>{status.app_support.database_path}</dd></div>
                    <div><dt>Logs</dt><dd>{status.app_support.logs_dir}</dd></div>
                    <div><dt>Config</dt><dd>{status.app_support.config_dir}</dd></div>
                    <div><dt>Migration version</dt><dd>{status.migration_version}</dd></div>
                    <div><dt>Registered workspaces</dt><dd>{workspaceRegistry.countLabel}</dd></div>
                    <div><dt>Foundation events</dt><dd>{status.event_count}</dd></div>
                  </dl>
                ) : (
                  <p className="muted-copy">Creating local folders and reading migration state…</p>
                )}
              </InfoCard>

              <InfoCard>
                <p className="eyebrow">Workspace registry</p>
                <h3>{workspaceRegistry.sourceLabel}</h3>
                <p>{workspaceRegistry.truthCopy}</p>
                <div className={`registry-meta ${workspaceRegistry.source}`}>
                  <span>{workspaceRegistry.countLabel}</span>
                  <span>{status ? "Real native registry" : "UI-only preview data"}</span>
                </div>
                <div className="registry-list">
                  {workspaces.length > 0 ? workspaces.map((workspace) => (
                    <button
                      aria-current={workspace.id === active?.id ? "page" : undefined}
                      className={workspace.id === active?.id ? "registry-chip active" : "registry-chip"}
                      key={workspace.id}
                      onClick={() => setActiveWorkspace(workspace.id)}
                      type="button"
                    >
                      {workspace.label}
                    </button>
                  )) : <p className="muted-copy">The native registry returned no workspaces.</p>}
                </div>
              </InfoCard>

              <InfoCard>
                <p className="eyebrow">Unavailable actions</p>
                <h3>Nothing is simulated</h3>
                <p>Search, module opening, and integration activity remain disabled until their real local capabilities exist.</p>
                <EmptyState icon="⌘">Preview shell only. Consequential actions stay behind native status and review policy.</EmptyState>
              </InfoCard>
            </section>
            </>
            )}
          </section>

          <InspectorPanel label="Workspace details">
            <InspectorCard className="active-summary">
              <p className="eyebrow">Details</p>
              <h3>{activeWorkspaceLabel}</h3>
              <p>{activeWorkspaceDescription}</p>
              <dl>
                <div><dt>ID</dt><dd>{active?.id ?? "—"}</dd></div>
                <div><dt>Position</dt><dd>{active?.position ?? "—"}</dd></div>
                <div><dt>Source</dt><dd>{workspaceSourceLabel}</dd></div>
                <div><dt>Visible count</dt><dd>{workspaceRegistry.countLabel}</dd></div>
              </dl>
            </InspectorCard>

            <SettingsStatusShell view={settingsStatusView} />
          </InspectorPanel>
        </div>
      </section>
    </main>
  );
}

export default App;
