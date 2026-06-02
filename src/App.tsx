import { invoke } from "@tauri-apps/api/core";
import type { ReactNode } from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import "./App.css";
import {
  buildConfirmationPolicyView,
  type ConfirmationPolicyRequirementView,
  type ConfirmationPolicyView,
} from "./confirmationPolicy";
import {
  buildSettingsStatusShellView,
  defaultIntegrationStates,
  type FoundationStatus,
  type IntegrationState,
  type SettingsStatusItem,
  type SettingsStatusShellView,
  type WorkspaceRecord,
} from "./settingsStatus";
import { buildTodayFoundationView, type TodayFoundationView, type TodayWidgetView } from "./todayFoundation";
import {
  buildTodayWidgetsView,
  type TodayDataState,
  type TodayNotificationRecord,
  type TodayTaskRecord,
  type TodayWidgetPanelView,
  type TodayWidgetsView,
} from "./todayWidgets";
import {
  createInitialTaskBridgeState,
  createTaskThroughBridge,
  formDraftForTask,
  refreshTasksFromBridge,
  selectTaskThroughBridge,
  updateTaskThroughBridge,
  type TaskBridgeInvoke,
  type TaskBridgeUiState,
} from "./taskBridgeIntegration";
import type { TaskFormDraft } from "./taskViewModel";
import { TaskWorkspace } from "./taskWorkspace";
import {
  buildWorkspaceChromeView,
  buildWorkspaceRegistryView,
  type WorkspaceRegistryView,
} from "./workspaceRegistry";

const integrationStates: IntegrationState[] = defaultIntegrationStates;

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

type ConfirmationPolicyPanelProps = {
  view: ConfirmationPolicyView;
};

function ConfirmationRequirementItem({ requirement }: { requirement: ConfirmationPolicyRequirementView }) {
  return (
    <li>
      <div>
        <strong>{requirement.label}</strong>
        <StatusBadge tone={requirement.tone}>{requirement.status}</StatusBadge>
      </div>
      <p>{requirement.detail}</p>
    </li>
  );
}

function ConfirmationPolicyPanel({ view }: ConfirmationPolicyPanelProps) {
  return (
    <InspectorCard className="confirmation-policy-panel">
      <div className="card-header compact">
        <div>
          <p className="eyebrow">Confirmation policy</p>
          <h3>{view.overallStatus}</h3>
        </div>
        <StatusBadge tone={view.tone}>{view.mode === "native" ? "Native" : view.mode === "checking" ? "Checking" : "Preview"}</StatusBadge>
      </div>

      <p>{view.summary}</p>

      <dl className="confirmation-policy-facts" aria-label="Policy reason and source">
        <div><dt>Source</dt><dd>{view.sourceLabel}</dd></div>
        <div><dt>Category</dt><dd>{view.category}</dd></div>
        <div><dt>Policy</dt><dd>{view.policy}</dd></div>
        <div><dt>Reason</dt><dd>{view.reason}</dd></div>
      </dl>

      <section className="settings-status-section" aria-label="Required confirmation and review gates">
        <p className="eyebrow">Required gates</p>
        <ul className="confirmation-requirement-list compact-list">
          {view.requirements.map((requirement) => (
            <ConfirmationRequirementItem key={requirement.label} requirement={requirement} />
          ))}
        </ul>
      </section>

      <EmptyState icon="!">{view.emptyActionCopy}</EmptyState>
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

type TodayWidgetPanelProps = {
  panel: TodayWidgetPanelView;
};

function TodayWidgetPanel({ panel }: TodayWidgetPanelProps) {
  return (
    <InfoCard className="today-widget-card today-data-widget-card">
      <div className="card-header compact">
        <div>
          <p className="eyebrow">Today data</p>
          <h3>{panel.title}</h3>
        </div>
        <StatusBadge tone={panel.tone}>{panel.status}</StatusBadge>
      </div>
      <p>{panel.copy}</p>
      {panel.items.length > 0 ? (
        <ul className="today-widget-list">
          {panel.items.map((item) => (
            <li key={item.id}>
              <span className={`status-dot ${item.tone}`} aria-hidden="true" />
              <div>
                <strong>{item.title}</strong>
                <small>{item.meta}</small>
              </div>
            </li>
          ))}
        </ul>
      ) : panel.emptyCopy ? (
        <EmptyState icon="∅">{panel.emptyCopy}</EmptyState>
      ) : null}
    </InfoCard>
  );
}

const ACTIVE_RUNS_BRIDGE_GAP =
  "No persisted run-list command is registered in the native bridge yet; Today cannot query active AgentRun rows truthfully.";

const taskInvoke: TaskBridgeInvoke = (command, args) => invoke(command, args);

function bridgeErrorReason(label: string, error: unknown) {
  const detail = error instanceof Error ? error.message : typeof error === "string" ? error : "unknown native bridge error";
  return `${label} bridge is unavailable (${detail}). No browser preview or fallback records are simulated.`;
}

type TodayWorkspaceOverviewProps = {
  activeWorkspaceDescription: string;
  activeWorkspaceLabel: string;
  status: FoundationStatus | null;
  statusError: string | null;
  statusTone: StatusTone;
  todayView: TodayFoundationView;
  todayWidgets: TodayWidgetsView;
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
  todayWidgets,
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

        <TodayWidgetPanel panel={todayWidgets.tasks} />
        <TodayWidgetPanel panel={todayWidgets.activeRuns} />
        <TodayWidgetPanel panel={todayWidgets.blockers} />
        <TodayWidgetPanel panel={todayWidgets.completions} />

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
  const [todayTasks, setTodayTasks] = useState<TodayDataState<TodayTaskRecord>>({ state: "checking" });
  const [todayInbox, setTodayInbox] = useState<TodayDataState<TodayNotificationRecord>>({ state: "checking" });
  const [taskBridgeUi, setTaskBridgeUi] = useState<TaskBridgeUiState>(() => createInitialTaskBridgeState("tasks"));

  useEffect(() => {
    invoke<FoundationStatus>("get_foundation_status")
      .then(setStatus)
      .catch(() => {
        setStatusError("Native foundation status is available inside the packaged Tauri app. Browser preview is UI-only.");
      });
  }, []);

  const applyTaskState = useCallback((state: TaskBridgeUiState["state"]) => {
    setTaskBridgeUi((current) => ({ ...current, state }));
    if (state.mode === "ready") setTodayTasks({ state: "ready", records: state.tasks });
    if (state.mode === "error") setTodayTasks({ state: "unavailable", reason: bridgeErrorReason("Native task", state.error) });
  }, []);

  const loadTaskWorkspace = useCallback(async (selectedTaskId: string | null) => {
    setTaskBridgeUi((current) => ({ ...current, state: { mode: "loading", selectedTaskId } }));
    applyTaskState(await refreshTasksFromBridge(taskInvoke, { selectedTaskId }));
  }, [applyTaskState]);

  useEffect(() => {
    let cancelled = false;

    refreshTasksFromBridge(taskInvoke, { selectedTaskId: null })
      .then((state) => {
        if (!cancelled) applyTaskState(state);
      });

    invoke<TodayNotificationRecord[]>("list_inbox_notifications_command", {
      request: { active_only: true, limit: 50 },
    })
      .then((records) => {
        if (!cancelled) setTodayInbox({ state: "ready", records });
      })
      .catch((error) => {
        if (!cancelled) setTodayInbox({ state: "unavailable", reason: bridgeErrorReason("Native inbox notification", error) });
      });

    return () => {
      cancelled = true;
    };
  }, [applyTaskState]);

  const handleTaskFormChange = useCallback((form: TaskFormDraft) => {
    setTaskBridgeUi((current) => ({ ...current, form, formErrors: {} }));
  }, []);

  const handleNewTask = useCallback(() => {
    setTaskBridgeUi((current) => ({
      form: createInitialTaskBridgeState(current.form.workspace_key || "tasks").form,
      formErrors: {},
      state: current.state.mode === "ready"
        ? { ...current.state, selectedTaskId: null }
        : { mode: "loading", selectedTaskId: null },
    }));
  }, []);

  const handleCreateTask = useCallback(async (form: TaskFormDraft) => {
    const next = await createTaskThroughBridge(taskInvoke, form);
    setTaskBridgeUi(next);
    if (next.state.mode === "ready") setTodayTasks({ state: "ready", records: next.state.tasks });
    if (next.state.mode === "error") setTodayTasks({ state: "unavailable", reason: bridgeErrorReason("Native task", next.state.error) });
  }, []);

  const handleUpdateTask = useCallback(async (taskId: string, form: TaskFormDraft) => {
    const next = await updateTaskThroughBridge(taskInvoke, taskId, form);
    setTaskBridgeUi(next);
    if (next.state.mode === "ready") setTodayTasks({ state: "ready", records: next.state.tasks });
    if (next.state.mode === "error") setTodayTasks({ state: "unavailable", reason: bridgeErrorReason("Native task", next.state.error) });
  }, []);

  const handleSelectTask = useCallback(async (taskId: string) => {
    const state = await selectTaskThroughBridge(taskInvoke, taskId);
    setTaskBridgeUi((current) => {
      const selectedTask = state.mode === "ready" ? state.tasks.find((task) => task.id === state.selectedTaskId) : null;
      return {
        ...current,
        form: selectedTask ? formDraftForTask(selectedTask) : current.form,
        formErrors: {},
        state,
      };
    });
    if (state.mode === "ready") setTodayTasks({ state: "ready", records: state.tasks });
    if (state.mode === "error") setTodayTasks({ state: "unavailable", reason: bridgeErrorReason("Native task", state.error) });
  }, [applyTaskState]);

  const workspaceRegistry = useMemo(() => buildWorkspaceRegistryView(status, statusError), [status, statusError]);
  const workspaces = workspaceRegistry.workspaces;
  const workspaceChrome = useMemo(
    () => buildWorkspaceChromeView(workspaceRegistry, activeWorkspace),
    [activeWorkspace, workspaceRegistry],
  );
  const active = workspaceChrome.activeWorkspace;

  const nativeState = statusError ? "Preview" : status ? "Native ready" : "Checking";
  const statusTone = statusError ? "blocked" : status ? "ready" : "pending";
  const workspaceSourceLabel = workspaceRegistry.sourceLabel;
  const activeWorkspaceLabel = workspaceChrome.activeWorkspaceLabel;
  const activeWorkspaceDescription = workspaceChrome.activeWorkspaceDescription;
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
  const todayWidgets = useMemo(
    () => buildTodayWidgetsView({
      source: status ? "native" : statusError ? "preview" : "checking",
      tasks: todayTasks,
      inbox: todayInbox,
      activeRuns: { state: "unavailable", reason: ACTIVE_RUNS_BRIDGE_GAP },
    }),
    [status, statusError, todayTasks, todayInbox],
  );
  const settingsStatusView = useMemo(
    () => buildSettingsStatusShellView({
      mode: status ? "native" : statusError ? "preview" : "checking",
      status,
      integrations: integrationStates,
    }),
    [status, statusError],
  );
  const confirmationPolicyView = useMemo(
    () => buildConfirmationPolicyView({
      mode: status ? "native" : statusError ? "preview" : "checking",
      policy: status?.secure_services.sample_policy ?? null,
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
              glyph={workspaceChrome.glyphs[workspace.id]}
              key={workspace.id}
              onSelect={() => setActiveWorkspace(workspace.id)}
              workspace={workspace}
            />
          )) : <p className="muted-copy">{workspaceChrome.sidebarEmptyCopy}</p>}
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
                todayWidgets={todayWidgets}
                workspaceRegistry={workspaceRegistry}
                workspaces={workspaces}
              />
            ) : active?.id === "tasks" ? (
              <TaskWorkspace
                form={taskBridgeUi.form}
                formErrors={taskBridgeUi.formErrors}
                onCreateTask={handleCreateTask}
                onFormChange={handleTaskFormChange}
                onNewTask={handleNewTask}
                onRefresh={() => loadTaskWorkspace(taskBridgeUi.state.selectedTaskId)}
                onSelectTask={handleSelectTask}
                onUpdateTask={handleUpdateTask}
                state={taskBridgeUi.state}
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
                  )) : <p className="muted-copy">{workspaceChrome.registryEmptyCopy}</p>}
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

            <ConfirmationPolicyPanel view={confirmationPolicyView} />

            <SettingsStatusShell view={settingsStatusView} />
          </InspectorPanel>
        </div>
      </section>
    </main>
  );
}

export default App;
