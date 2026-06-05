import { invoke } from "@tauri-apps/api/core";
import type { FormEvent, ReactNode } from "react";
import { useCallback, useEffect, useMemo, useState } from "react";
import "./App.css";
import {
  appendCleanSessionChunk,
  loadCleanSessionStreamFromBridge,
  nextCleanSessionOffset,
  type CleanSessionState,
} from "./cleanSession";
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
  performTaskActionThroughBridge,
  refreshTasksFromBridge,
  selectTaskThroughBridge,
  updateTaskThroughBridge,
  type TaskBridgeInvoke,
  type TaskBridgeUiState,
} from "./taskBridgeIntegration";
import {
  createInitialNoteBridgeState,
  createNoteThroughBridge,
  editNoteThroughBridge,
  formDraftForNote,
  refreshNotesFromBridge,
  scanNotesThroughBridge,
  selectNoteThroughBridge,
  trashNoteThroughBridge,
  type NoteBridgeInvoke,
  type NoteBridgeUiState,
} from "./noteBridgeIntegration";
import type { NoteFormDraft } from "./noteViewModel";
import { NoteWorkspace } from "./noteWorkspace";
import {
  browseFilesFromBridge,
  createInitialFileBridgeState,
  performFileActionThroughBridge,
  previewFileThroughBridge,
  type FileBridgeInvoke,
  type FileBridgeUiState,
} from "./fileBridgeIntegration";
import type { FileActionDraft } from "./fileViewModel";
import { FileWorkspace } from "./fileWorkspace";
import {
  createIdleContentLinkedPanelsState,
  fileReferenceEntityId,
  loadContentLinkedPanelsFromBridge,
  type ContentLinkedPanelsState,
} from "./contentLinkedPanels";
import { ContentLinkedPanels } from "./contentLinkedPanelsView";
import type { TaskFormDraft } from "./taskViewModel";
import { TaskWorkspace } from "./taskWorkspace";
import { loadTaskLinkedPanelsFromBridge, type TaskLinkedPanelsState } from "./taskLinkedPanels";
import { TaskLinkedPanels } from "./taskLinkedPanelsView";
import { loadPhase6OverviewFromBridge, type Phase6State } from "./phase6Workspace";
import { Phase6Workspace } from "./phase6WorkspaceView";
import type { InboxDataState, InboxNotificationRecord } from "./inboxViewModel";
import {
  buildTaskScopedInboxState,
  createInitialManualReviewState,
  createManualReviewThroughBridge,
  resetManualReviewForTask,
  updateManualReviewDraft,
  type ManualReviewDraft,
  type ManualReviewState,
} from "./taskDetailBatchPanels";
import {
  cancelRunThroughBridge,
  createInitialRunControlsState,
  resetRunControlsForTask,
  startRunThroughBridge,
  updateRunControlsDraft,
  type RunControlsDraft,
  type RunControlsState,
} from "./runControls";
import { RunControlsPanel } from "./runControlsView";
import {
  buildWorkspaceChromeView,
  buildWorkspaceRegistryView,
  type WorkspaceRegistryView,
} from "./workspaceRegistry";
import {
  blockedVerificationRecords,
  omnisocialsActionCopy,
  parsePlatforms,
  pieceScheduleGateSummary,
  type ContentPieceRecord,
  type ContentPlanRecord,
  type ContentReviewGateRecord,
  type ContentScheduleRecord,
  type ContentVerificationRecord,
  type ContentWorkspaceState,
  type MediaAssetRecord,
  type OmniSocialsStatusRecord,
} from "./contentWorkspace";
import {
  allowedAttachmentTargets,
  attachCaptureThroughBridge,
  createCaptureThroughBridge,
  createInitialBrowserWorkspaceState,
  loadBrowserWorkspaceFromBridge,
  moveWidget,
  resetWidgetsThroughBridge,
  resizeWidget,
  saveWorkUrlThroughBridge,
  toggleWidget,
  updateBrowserDraft,
  updateWidgetThroughBridge,
  type BrowserBridgeInvoke,
  type BrowserCaptureTarget,
  type BrowserWorkspaceDraft,
  type BrowserWorkspaceState,
  type WidgetConfigRecord,
  type WidgetSize,
} from "./browserWorkspace";
import {
  dryRunLogCleanup,
  loadReleaseHardeningState,
  type ReleaseHardeningState,
} from "./releaseAbout";

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

type ReleaseHardeningPanelProps = {
  state: ReleaseHardeningState;
  onRefresh: () => void;
  onDryRunCleanup: () => void;
};

function ReleaseHardeningPanel({ state, onRefresh, onDryRunCleanup }: ReleaseHardeningPanelProps) {
  if (state.mode === "loading") {
    return <InspectorCard><p className="eyebrow">Release hardening</p><EmptyState icon="…">Loading release readiness and log retention state.</EmptyState></InspectorCard>;
  }
  if (state.mode === "error") {
    return (
      <InspectorCard>
        <p className="eyebrow">Release hardening</p>
        <h3>Native release state unavailable</h3>
        <BlockerState>{bridgeErrorReason("Release hardening", state.error)}</BlockerState>
        <button className="secondary-action" onClick={onRefresh} type="button">Retry release state</button>
      </InspectorCard>
    );
  }
  return (
    <InspectorCard className="settings-status-shell">
      <div className="card-header compact">
        <div>
          <p className="eyebrow">Release hardening</p>
          <h3>{state.about.appName} {state.about.version}</h3>
        </div>
        <StatusBadge tone="ready">Ready</StatusBadge>
      </div>
      <dl className="settings-status-list" aria-label="Release package status">
        <div><dt>Build</dt><dd>{state.about.build}</dd></div>
        <div><dt>Packaging</dt><dd>{state.about.packaging}</dd></div>
        <div><dt>Signing</dt><dd>{state.about.signing}</dd></div>
        <div><dt>Notarization</dt><dd>{state.about.notarization}</dd></div>
      </dl>
      <section className="settings-status-section" aria-label="Log retention settings">
        <div className="card-header compact">
          <div>
            <p className="eyebrow">Log retention</p>
            <h4>Dry-run before cleanup</h4>
          </div>
          <button className="secondary-action" onClick={onDryRunCleanup} type="button">Dry-run cleanup</button>
        </div>
        <ul className="compact-list">
          {state.retention.map((setting) => (
            <li key={setting.scope}>
              <div><strong>{setting.scope}</strong><StatusBadge tone={setting.enabled ? "ready" : "pending"}>{setting.enabled ? "Enabled" : "Disabled"}</StatusBadge></div>
              <p>{setting.summary}</p>
            </li>
          ))}
        </ul>
        {state.cleanupResult ? <p className="muted-copy">Last dry-run: {state.cleanupResult.files_considered} considered, {state.cleanupResult.files_deleted} deleted.</p> : null}
      </section>
      <section className="settings-status-section" aria-label="Migration failure guidance">
        <p className="eyebrow">Migration recovery</p>
        <h4>{state.migrationGuidance.title}</h4>
        <p>{state.migrationGuidance.message}</p>
        <ul className="compact-list">
          {state.migrationGuidance.actions.map((action) => <li key={action}><p>{action}</p></li>)}
        </ul>
      </section>
      <ul className="compact-list">
        {state.about.safeDiagnostics.map((line) => <li key={line}><p>{line}</p></li>)}
      </ul>
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
const noteInvoke: NoteBridgeInvoke = (command, args) => invoke(command, args);
const fileInvoke: FileBridgeInvoke = (command, args) => invoke(command, args);
const phase6Invoke = <T,>(command: string, args?: Record<string, unknown>) => invoke<T>(command, args);
const browserInvoke: BrowserBridgeInvoke = (command, args) => invoke(command, args);
const releaseInvoke = <T,>(command: string, args?: Record<string, unknown>) => invoke<T>(command, args);

function appFormString(form: FormData, key: string) {
  const value = String(form.get(key) ?? "").trim();
  return value.length > 0 ? value : undefined;
}

function appFormInteger(form: FormData, key: string) {
  const value = appFormString(form, key);
  return value ? Number.parseInt(value, 10) : null;
}

function bridgeErrorReason(label: string, error: unknown) {
  const detail = error instanceof Error ? error.message : typeof error === "string" ? error : "unknown native bridge error";
  if (/invoke|__TAURI|Tauri/i.test(detail)) {
    return `${label} backend is only available inside the Tauri desktop app. Browser preview is UI-only and does not simulate records.`;
  }
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



type CodeRepoRecord = {
  id: string;
  display_name: string;
  root_path: string;
  profile_type: string;
  default_branch?: string | null;
  package_manager?: string | null;
  linked_product_id?: string | null;
  status: string;
  metadata_json: string;
};

type CodeIntegrationRecord = {
  integration_key: string;
  display_name: string;
  status: string;
  config_json: string;
};

type CodePolicyDecision = {
  category: string;
  reason: string;
  policy: { category: string; requires_confirmation: boolean; reviewer_required: boolean };
  reviewer_required: string;
  human_confirmation: string;
};

type LaunchGateRecord = {
  id: string;
  repo_id: string;
  product_id?: string | null;
  task_id?: string | null;
  state: string;
  final_verdict?: string | null;
  metadata_json: string;
};

type LaunchGateEvidenceRecord = {
  id: string;
  launch_gate_id: string;
  evidence_type: string;
  label: string;
  url?: string | null;
  status_code?: number | null;
  manual_note?: string | null;
  metadata_json: string;
};

type CodeWorkspaceState =
  | { mode: "loading" }
  | { mode: "error"; error: string }
  | {
      mode: "ready";
      repos: CodeRepoRecord[];
      integrations: CodeIntegrationRecord[];
      policy: CodePolicyDecision;
      actionStatus?: string | null;
      lastLaunchGate?: LaunchGateRecord | null;
      lastEvidence?: LaunchGateEvidenceRecord | null;
    };

type CodeWorkspaceActions = {
  addRepo: (event: FormEvent<HTMLFormElement>) => void;
  linkRepo: (event: FormEvent<HTMLFormElement>) => void;
  createLaunchGate: (event: FormEvent<HTMLFormElement>) => void;
  addLaunchGateEvidence: (event: FormEvent<HTMLFormElement>) => void;
  evaluateLaunchGate: (event: FormEvent<HTMLFormElement>) => void;
};

function AppFormMetadata() {
  return <input name="metadata_json" placeholder="metadata JSON, e.g. {}" defaultValue="{}" />;
}

function CodeWorkspace({ state, onRefresh, actions }: { state: CodeWorkspaceState; onRefresh: () => void; actions: CodeWorkspaceActions }) {
  if (state.mode === "loading") {
    return <EmptyState icon="</>">Loading repo registry, truthful integration states, and Launch Gate policy previews…</EmptyState>;
  }
  if (state.mode === "error") {
    return (
      <InfoCard className="large-card">
        <p className="eyebrow">Code workspace</p>
        <h3>Native Phase 4 bridge unavailable</h3>
        <BlockerState>{state.error}</BlockerState>
        <button className="secondary-action" onClick={onRefresh} type="button">Retry native load</button>
      </InfoCard>
    );
  }
  return (
    <section className="dashboard-grid">
      <InfoCard className="large-card">
        <div className="card-header">
          <div>
            <p className="eyebrow">Code / repos</p>
            <h3>Lightweight repo registry</h3>
          </div>
          <StatusBadge tone="ready">Native</StatusBadge>
        </div>
        {state.actionStatus ? <p className="muted-copy">Last action: {state.actionStatus}</p> : null}
        <form className="phase6-form" onSubmit={actions.addRepo}>
          <p className="eyebrow">Add repo profile</p>
          <input name="display_name" placeholder="Display name" required />
          <input name="root_path" placeholder="/absolute/repo/path" required />
          <select name="profile_type" defaultValue="product_app">
            <option value="product_app">Product app</option>
            <option value="website">Website</option>
            <option value="library">Library</option>
            <option value="experiment">Experiment</option>
            <option value="client_project">Client project</option>
            <option value="content_docs">Content docs</option>
            <option value="other">Other</option>
          </select>
          <input name="default_branch" placeholder="Default branch" defaultValue="main" />
          <input name="package_manager" placeholder="Package manager" />
          <input name="linked_product_id" placeholder="Linked product id (optional)" />
          <AppFormMetadata />
          <button className="primary-action" type="submit">Add repo profile</button>
        </form>
        {state.repos.length > 0 ? (
          <ul className="compact-list">
            {state.repos.map((repo) => (
              <li key={repo.id}>
                <div><strong>{repo.display_name}</strong><span>{repo.profile_type}</span></div>
                <p>{repo.root_path}</p>
                <small>{repo.id}</small>
              </li>
            ))}
          </ul>
        ) : (
          <EmptyState icon="⌘">No repos are fabricated. Use Add repo profile to persist a native repo record.</EmptyState>
        )}
      </InfoCard>

      <InfoCard>
        <p className="eyebrow">Integrations</p>
        <h3>Truthful state only</h3>
        <ul className="compact-list">
          {state.integrations.map((integration) => (
            <li key={integration.integration_key}>
              <div><strong>{integration.display_name}</strong><span>{integration.status}</span></div>
              <p>No fake connected state or deploy execution is shown.</p>
            </li>
          ))}
        </ul>
      </InfoCard>

      <InfoCard className="large-card">
        <p className="eyebrow">Launch Gate</p>
        <h3>Evidence required</h3>
        <p>Commit/push/merge/deploy actions are policy previews only until real evidence is captured.</p>
        <StatusBadge tone="blocked">{state.policy.category}</StatusBadge>
        <p className="muted-copy">{state.policy.reason}</p>
        {state.lastLaunchGate ? <p className="muted-copy">Last gate: {state.lastLaunchGate.id} · {state.lastLaunchGate.state} · {state.lastLaunchGate.final_verdict ?? "no verdict"}</p> : null}
        {state.lastEvidence ? <p className="muted-copy">Last evidence: {state.lastEvidence.label} · {state.lastEvidence.evidence_type} · {state.lastEvidence.id}</p> : null}
        <form className="phase6-form" onSubmit={actions.createLaunchGate}>
          <p className="eyebrow">Create launch gate</p>
          <input name="repo_id" placeholder="repo id" defaultValue={state.repos[0]?.id ?? ""} required />
          <input name="product_id" placeholder="product id (optional)" />
          <input name="task_id" placeholder="task id (optional)" />
          <AppFormMetadata />
          <button className="secondary-action" type="submit">Create launch gate</button>
        </form>
        <form className="phase6-form" onSubmit={actions.addLaunchGateEvidence}>
          <p className="eyebrow">Add evidence</p>
          <input name="launch_gate_id" placeholder="launch gate id" defaultValue={state.lastLaunchGate?.id ?? ""} required />
          <select name="evidence_type" defaultValue="manual_note"><option value="manual_note">Manual note</option><option value="url_status">URL/status</option><option value="screenshot">Screenshot</option><option value="test_output">Test output</option><option value="deployment_record">Deployment record</option></select>
          <input name="label" placeholder="Evidence label" required />
          <input name="url" placeholder="URL (optional)" />
          <input name="status_code" placeholder="HTTP status (optional)" />
          <input name="manual_note" placeholder="Manual note" />
          <AppFormMetadata />
          <button className="secondary-action" type="submit">Add launch evidence</button>
        </form>
        <form className="phase6-form" onSubmit={actions.evaluateLaunchGate}>
          <p className="eyebrow">Evaluate gate</p>
          <input name="launch_gate_id" placeholder="launch gate id" defaultValue={state.lastLaunchGate?.id ?? ""} required />
          <button className="secondary-action" type="submit">Evaluate launch gate</button>
        </form>
      </InfoCard>

      <InfoCard>
        <p className="eyebrow">Repo links</p>
        <h3>Attach repo context</h3>
        <form className="phase6-form" onSubmit={actions.linkRepo}>
          <input name="repo_id" placeholder="repo id" defaultValue={state.repos[0]?.id ?? ""} required />
          <select name="target_type" defaultValue="product"><option value="product">Product</option><option value="task">Task</option></select>
          <input name="target_id" placeholder="target id" required />
          <input name="relation_type" placeholder="relation" defaultValue="belongs_to" />
          <AppFormMetadata />
          <button className="secondary-action" type="submit">Link repo</button>
        </form>
      </InfoCard>
    </section>
  );
}

type ContentWorkspaceActions = {
  createDraft: () => void;
  updateDraft: (pieceId: string) => void;
  createReviewGate: (pieceId: string) => void;
  approveReviewGate: (gateId: string) => void;
  rejectReviewGate: (gateId: string) => void;
  attemptScheduleIntent: (pieceId: string, platform: string) => void;
  cancelSchedule: (scheduleId: string) => void;
  recordFailClosedUpload: (pieceId: string, platform: string) => void;
  recordFailClosedSchedule: (pieceId: string, platform: string, scheduleId?: string | null) => void;
  recordFailClosedPublish: (pieceId: string, platform: string, scheduleId?: string | null) => void;
};

function ContentWorkspace({ state, onRefresh, actions }: { state: ContentWorkspaceState; onRefresh: () => void; actions: ContentWorkspaceActions }) {
  if (state.mode === "loading") {
    return <EmptyState icon="C">Loading content plans, draft gates, schedules, and OmniSocials status…</EmptyState>;
  }
  if (state.mode === "error") {
    return (
      <InfoCard className="large-card">
        <p className="eyebrow">Content workspace</p>
        <h3>Native Phase 5 bridge unavailable</h3>
        <BlockerState>{state.error}</BlockerState>
        <button className="secondary-action" onClick={onRefresh} type="button">Retry native load</button>
      </InfoCard>
    );
  }
  const blockedRecords = blockedVerificationRecords(state.verifications);
  const selectedPiece = state.pieces.find((piece) => piece.id === state.selectedPieceId) ?? state.pieces[0] ?? null;
  const selectedGates = selectedPiece ? state.reviewGates.filter((gate) => gate.piece_id === selectedPiece.id) : [];
  const selectedAssets = selectedPiece ? state.mediaAssets.filter((asset) => asset.piece_id === selectedPiece.id) : [];
  const selectedSchedules = selectedPiece ? state.schedules.filter((schedule) => schedule.piece_id === selectedPiece.id) : [];
  const firstPlatform = selectedPiece ? parsePlatforms(selectedPiece.platforms_json)[0] ?? "linkedin" : "linkedin";
  return (
    <section className="dashboard-grid">
      <InfoCard className="large-card">
        <div className="card-header">
          <div><p className="eyebrow">Content / draft-first</p><h3>Plans and pieces</h3></div>
          <StatusBadge tone="ready">Phase 5</StatusBadge>
        </div>
        <p className="muted-copy">Content stays local and draft-first. Scheduling requires review and human confirmation before any external write.</p>
        {state.actionStatus ? <p className="muted-copy">Last action: {state.actionStatus}</p> : null}
        <button className="primary-action" type="button" onClick={actions.createDraft}>Create local sample draft</button>
        {state.pieces.length > 0 ? (
          <ul className="compact-list">
            {state.pieces.map((piece) => {
              const platforms = parsePlatforms(piece.platforms_json);
              return (
                <li key={piece.id}>
                  <div><strong>{piece.title}</strong><span>{piece.status} · {piece.required_gate}</span></div>
                  <p>{platforms.join(", ") || "No platforms"}</p>
                  <p className="muted-copy">{pieceScheduleGateSummary(piece, state.reviewGates, state.schedules)}</p>
                </li>
              );
            })}
          </ul>
        ) : <EmptyState icon="✎">No content pieces yet. Use the local sample action to create a plan and draft through native commands.</EmptyState>}
        <button className="secondary-action" type="button" onClick={onRefresh}>Refresh content state</button>
      </InfoCard>
      <InfoCard>
        <p className="eyebrow">Selected draft workflow</p>
        <h3>{selectedPiece?.title ?? "No draft selected"}</h3>
        {selectedPiece ? (
          <>
            <p className="muted-copy">Review gates: {selectedGates.length}. Media refs: {selectedAssets.length}. Schedule intents: {selectedSchedules.length}.</p>
            <button className="secondary-action" type="button" onClick={() => actions.updateDraft(selectedPiece.id)}>Move draft to review-ready</button>
            <button className="secondary-action" type="button" onClick={() => actions.createReviewGate(selectedPiece.id)}>Create review gate</button>
            {selectedGates.filter((gate) => gate.status !== "approved").map((gate) => (
              <div key={gate.id} className="content-inline-actions">
                <button className="secondary-action" type="button" onClick={() => actions.approveReviewGate(gate.id)}>Approve {gate.gate_type}</button>
                <button className="secondary-action" type="button" onClick={() => actions.rejectReviewGate(gate.id)}>Reject {gate.gate_type}</button>
              </div>
            ))}
            <button className="secondary-action" type="button" onClick={() => actions.attemptScheduleIntent(selectedPiece.id, firstPlatform)}>Attempt schedule intent</button>
            <button className="secondary-action" type="button" onClick={() => actions.recordFailClosedUpload(selectedPiece.id, firstPlatform)}>Record fail-closed upload check</button>
            <button className="secondary-action" type="button" onClick={() => actions.recordFailClosedSchedule(selectedPiece.id, firstPlatform, selectedSchedules[0]?.id)}>Record fail-closed schedule check</button>
            <button className="secondary-action" type="button" onClick={() => actions.recordFailClosedPublish(selectedPiece.id, firstPlatform, selectedSchedules[0]?.id)}>Record fail-closed publish check</button>
          </>
        ) : <p className="muted-copy">Create or load a draft to run review and fail-closed checks.</p>}
      </InfoCard>
      <InfoCard>
        <p className="eyebrow">OmniSocials</p>
        <h3>Fails closed</h3>
        <p>{state.omnisocials.status_note}</p>
        <div className={`registry-meta ${state.omnisocials.state === "connected" ? "ready" : "blocked"}`}><span>Status</span><span>{state.omnisocials.state}</span></div>
        <p className="muted-copy">{omnisocialsActionCopy(state.omnisocials)}</p>
      </InfoCard>
      <InfoCard>
        <p className="eyebrow">Schedules</p>
        <h3>Intent queue</h3>
        {state.schedules.length > 0 ? <ul className="compact-list">{state.schedules.map((schedule) => <li key={schedule.id}><div><strong>{schedule.platform}</strong><span>{schedule.status}</span></div><p>{schedule.scheduled_for}</p><button className="secondary-action" type="button" onClick={() => actions.cancelSchedule(schedule.id)}>Cancel local intent</button></li>)}</ul> : <p className="muted-copy">No schedule intents yet.</p>}
      </InfoCard>
      <InfoCard>
        <p className="eyebrow">Verification</p>
        <h3>Failure reports</h3>
        {blockedRecords.length > 0 ? <ul className="compact-list">{blockedRecords.map((record) => <li key={record.id}><div><strong>{record.action_type}</strong><span>{record.outcome}</span></div><p>{record.failure_report || record.provider_status || "Blocked by local policy"}</p></li>)}</ul> : <p className="muted-copy">No blocked/failed verification records.</p>}
      </InfoCard>
    </section>
  );
}

type BrowserWorkspaceActions = {
  refresh: () => void;
  updateDraft: (patch: Partial<BrowserWorkspaceDraft>) => void;
  saveUrl: () => void;
  createCapture: () => void;
  attachCapture: () => void;
  selectCapture: (captureId: string) => void;
  persistWidget: (widget: WidgetConfigRecord) => void;
  resetWidgets: () => void;
};

function formatWidgetLabel(widgetKey: string) {
  return widgetKey.replace(/_/g, " ");
}

function BrowserWorkspace({ state, actions }: { state: BrowserWorkspaceState; actions: BrowserWorkspaceActions }) {
  if (state.mode === "loading") {
    return <EmptyState icon="◌">Loading Browser workspace tabs, captures, and widget preferences from the native bridge…</EmptyState>;
  }
  if (state.mode === "error") {
    return (
      <InfoCard className="large-card">
        <p className="eyebrow">Browser workspace</p>
        <h3>Native Phase 7 bridge unavailable</h3>
        <BlockerState>{bridgeErrorReason("Native Browser workspace", state.error)}</BlockerState>
        <button className="secondary-action" onClick={actions.refresh} type="button">Retry native load</button>
      </InfoCard>
    );
  }

  const selectedCapture = state.captures.find((capture) => capture.id === state.selectedCaptureId) ?? state.captures[0] ?? null;

  return (
    <section aria-label="Browser workspace" className="dashboard-grid">
      <InfoCard className="large-card">
        <div className="card-header">
          <div>
            <p className="eyebrow">Work webview/capture workspace</p>
            <h3>Open work URL</h3>
          </div>
          <StatusBadge tone="ready">Native data</StatusBadge>
        </div>
        <label className="field-label" htmlFor="browser-url-input">URL</label>
        <input id="browser-url-input" aria-label="Work URL" onChange={(event) => actions.updateDraft({ url: event.target.value })} placeholder="https://example.com/work" value={state.draft.url} />
        <label className="field-label" htmlFor="browser-title-input">Title</label>
        <input id="browser-title-input" aria-label="Work URL title" onChange={(event) => actions.updateDraft({ title: event.target.value })} placeholder="Evidence title" value={state.draft.title} />
        <label className="field-label" htmlFor="browser-note-input">Manual note</label>
        <textarea id="browser-note-input" aria-label="Browser manual note" onChange={(event) => actions.updateDraft({ manualNote: event.target.value })} placeholder="Optional non-secret note" value={state.draft.manualNote} />
        <p className="muted-copy">This workspace saves work URLs and metadata fallback captures. It does not claim personal browser sync, extensions, cookies, auth headers, or password management.</p>
        <button className="secondary-action" onClick={actions.saveUrl} type="button" aria-label="Save work URL metadata fallback">Save URL metadata</button>
        <button className="secondary-action" onClick={actions.createCapture} type="button" aria-label="Create metadata fallback capture">Capture metadata fallback</button>
        {state.message ? <p className="muted-copy">{state.message}</p> : null}
        {state.errorMessage ? <BlockerState>{state.errorMessage}</BlockerState> : null}
      </InfoCard>

      <InfoCard>
        <p className="eyebrow">Tabs / saved pages</p>
        <h3>Real native rows</h3>
        {state.tabs.length > 0 ? (
          <ul className="compact-list">
            {state.tabs.map((tab) => (
              <li key={tab.id}>
                <div><strong>{tab.title || tab.url}</strong><span>{tab.state}</span></div>
                <p>{tab.url}</p>
              </li>
            ))}
          </ul>
        ) : <EmptyState icon="∅">No saved work URLs returned by the native browser bridge.</EmptyState>}
      </InfoCard>

      <InfoCard>
        <p className="eyebrow">Captures</p>
        <h3>Metadata fallback evidence</h3>
        <EmptyState icon="◌">Tauri screenshot capture is unsupported here; capture stores URL, title, timestamp, optional HTTP status, manual note, and entity links.</EmptyState>
        {state.captures.length > 0 ? (
          <ul className="compact-list">
            {state.captures.map((capture) => (
              <li key={capture.id}>
                <div><strong>{capture.title || capture.url}</strong><span>{capture.capture_mode}</span></div>
                <p>{capture.url}</p>
                <button className="secondary-action" onClick={() => actions.selectCapture(capture.id)} type="button" aria-label={`Select capture ${capture.id}`}>Select capture</button>
              </li>
            ))}
          </ul>
        ) : <p className="muted-copy">No capture records returned yet.</p>}
        {selectedCapture ? <p className="muted-copy">Selected capture: {selectedCapture.id}; screenshot supported: {selectedCapture.screenshot_supported ? "yes" : "no"}</p> : null}
        <label className="field-label" htmlFor="browser-attach-target">Attachment target</label>
        <select id="browser-attach-target" aria-label="Attachment target picker" onChange={(event) => actions.updateDraft({ entityType: event.target.value as BrowserCaptureTarget })} value={state.draft.entityType}>
          {allowedAttachmentTargets.map((target) => <option key={target} value={target}>{target.replace(/_/g, " ")}</option>)}
        </select>
        <label className="field-label" htmlFor="browser-attach-entity">Entity id</label>
        <input id="browser-attach-entity" aria-label="Attachment entity id" onChange={(event) => actions.updateDraft({ entityId: event.target.value })} placeholder="task-123 / launch-gate id" value={state.draft.entityId} />
        <button className="secondary-action" onClick={actions.attachCapture} type="button" aria-label="Attach browser capture evidence">Attach capture evidence</button>
      </InfoCard>

      <InfoCard>
        <p className="eyebrow">Widget customization</p>
        <h3>Persisted layout controls</h3>
        {state.widgets.length > 0 ? (
          <ul className="compact-list">
            {state.widgets.map((widget) => (
              <li key={widget.widget_key}>
                <div><strong>{formatWidgetLabel(widget.widget_key)}</strong><span>{widget.visible ? "visible" : "hidden"} · {widget.size}</span></div>
                <button type="button" aria-label={`Toggle ${widget.widget_key}`} onClick={() => actions.persistWidget(toggleWidget(widget))}>Show/hide</button>
                <button type="button" aria-label={`Move ${widget.widget_key} up`} onClick={() => actions.persistWidget(moveWidget(state.widgets, widget.widget_key, "up").find((item) => item.widget_key === widget.widget_key) ?? widget)}>Move up</button>
                <button type="button" aria-label={`Resize ${widget.widget_key}`} onClick={() => actions.persistWidget(resizeWidget(widget, widget.size === "large" ? "small" : (widget.size === "small" ? "medium" : "large") as WidgetSize))}>Resize</button>
              </li>
            ))}
          </ul>
        ) : <p className="muted-copy">No widget configs returned yet.</p>}
        <button type="button" aria-label="Reset widgets" onClick={actions.resetWidgets}>Reset widgets</button>
      </InfoCard>
    </section>
  );
}

function App() {
  const [activeWorkspace, setActiveWorkspace] = useState("today");
  const [status, setStatus] = useState<FoundationStatus | null>(null);
  const [statusError, setStatusError] = useState<string | null>(null);
  const [todayTasks, setTodayTasks] = useState<TodayDataState<TodayTaskRecord>>({ state: "checking" });
  const [todayInbox, setTodayInbox] = useState<TodayDataState<TodayNotificationRecord>>({ state: "checking" });
  const [taskBridgeUi, setTaskBridgeUi] = useState<TaskBridgeUiState>(() => createInitialTaskBridgeState("tasks"));
  const [noteBridgeUi, setNoteBridgeUi] = useState<NoteBridgeUiState>(() => createInitialNoteBridgeState());
  const [fileBridgeUi, setFileBridgeUi] = useState<FileBridgeUiState>(() => createInitialFileBridgeState());
  const [taskLinkedPanels, setTaskLinkedPanels] = useState<TaskLinkedPanelsState>({ mode: "idle", taskId: null });
  const [noteLinkedPanels, setNoteLinkedPanels] = useState<ContentLinkedPanelsState>(() => createIdleContentLinkedPanelsState("note"));
  const [fileLinkedPanels, setFileLinkedPanels] = useState<ContentLinkedPanelsState>(() => createIdleContentLinkedPanelsState("file"));
  const [cleanSessions, setCleanSessions] = useState<Record<string, CleanSessionState>>({});
  const [runControls, setRunControls] = useState<RunControlsState>(() => createInitialRunControlsState({ taskId: null, profileId: "default", cwd: "" }));
  const [manualReview, setManualReview] = useState<ManualReviewState>(() => createInitialManualReviewState(null, null));
  const [codeWorkspace, setCodeWorkspace] = useState<CodeWorkspaceState>({ mode: "loading" });
  const [phase6State, setPhase6State] = useState<Phase6State>({ mode: "loading" });
  const [contentWorkspace, setContentWorkspace] = useState<ContentWorkspaceState>({ mode: "loading" });
  const [browserWorkspace, setBrowserWorkspace] = useState<BrowserWorkspaceState>(() => createInitialBrowserWorkspaceState());
  const [releaseHardening, setReleaseHardening] = useState<ReleaseHardeningState>({ mode: "loading" });

  useEffect(() => {
    invoke<FoundationStatus>("get_foundation_status")
      .then(setStatus)
      .catch(() => {
        setStatusError("Native foundation status is available inside the packaged Tauri app. Browser preview is UI-only.");
      });
  }, []);

  const loadCodeWorkspace = useCallback(async () => {
    setCodeWorkspace({ mode: "loading" });
    try {
      const [repos, integrations, policy] = await Promise.all([
        invoke<CodeRepoRecord[]>("list_repo_profiles_command"),
        invoke<CodeIntegrationRecord[]>("list_repo_integration_states_command"),
        invoke<CodePolicyDecision>("preview_launch_action_policy_command", { actionCategory: "deploy" }),
      ]);
      setCodeWorkspace((current) => ({
        mode: "ready",
        repos,
        integrations,
        policy,
        actionStatus: current.mode === "ready" ? current.actionStatus : null,
        lastLaunchGate: current.mode === "ready" ? current.lastLaunchGate : null,
        lastEvidence: current.mode === "ready" ? current.lastEvidence : null,
      }));
    } catch (error) {
      setCodeWorkspace({ mode: "error", error: bridgeErrorReason("Native Code workspace", error) });
    }
  }, []);


  const loadContentWorkspace = useCallback(async () => {
    setContentWorkspace({ mode: "loading" });
    try {
      const [plans, pieces, schedules, verifications, omnisocials] = await Promise.all([
        invoke<ContentPlanRecord[]>("list_content_plans_command", { request: { limit: 50 } }),
        invoke<ContentPieceRecord[]>("list_content_pieces_command", { request: { limit: 50 } }),
        invoke<ContentScheduleRecord[]>("list_content_schedules_command", { pieceId: null }),
        invoke<ContentVerificationRecord[]>("list_content_verification_records_command", { request: { limit: 50 } }),
        invoke<OmniSocialsStatusRecord>("get_omnisocials_status_command"),
      ]);
      const selectedPieceId = pieces[0]?.id ?? null;
      const [mediaAssets, reviewGates] = selectedPieceId
        ? await Promise.all([
            invoke<MediaAssetRecord[]>("list_media_asset_references_command", { pieceId: selectedPieceId }),
            invoke<ContentReviewGateRecord[]>("list_content_review_gates_command", { pieceId: selectedPieceId }),
          ])
        : [[], []];
      setContentWorkspace({ mode: "ready", plans, pieces, mediaAssets, reviewGates, schedules, verifications, omnisocials, selectedPieceId });
    } catch (error) {
      setContentWorkspace({ mode: "error", error: bridgeErrorReason("Native Content workspace", error) });
    }
  }, []);

  const loadBrowserWorkspace = useCallback(() => {
    setBrowserWorkspace((current) => {
      void loadBrowserWorkspaceFromBridge(browserInvoke, current.draft).then(setBrowserWorkspace);
      return { mode: "loading", draft: current.draft };
    });
  }, []);

  const loadReleaseHardening = useCallback(() => {
    setReleaseHardening({ mode: "loading" });
    void loadReleaseHardeningState(releaseInvoke).then(setReleaseHardening);
  }, []);

  useEffect(() => {
    void loadCodeWorkspace();
    void loadContentWorkspace();
    loadBrowserWorkspace();
    loadReleaseHardening();
  }, [loadCodeWorkspace, loadContentWorkspace, loadBrowserWorkspace, loadReleaseHardening]);

  const applyTaskState = useCallback((state: TaskBridgeUiState["state"]) => {
    setTaskBridgeUi((current) => ({ ...current, state }));
    if (state.mode === "ready") setTodayTasks({ state: "ready", records: state.tasks });
    if (state.mode === "error") setTodayTasks({ state: "unavailable", reason: bridgeErrorReason("Native task", state.error) });
  }, []);

  const loadTaskWorkspace = useCallback(async (selectedTaskId: string | null) => {
    setTaskBridgeUi((current) => ({ ...current, state: { mode: "loading", selectedTaskId } }));
    applyTaskState(await refreshTasksFromBridge(taskInvoke, { selectedTaskId }));
  }, [applyTaskState]);

  const loadCleanSession = useCallback(async (runId: string) => {
    const previousState = cleanSessions[runId];
    setCleanSessions((current) => ({ ...current, [runId]: { mode: "loading", runId } }));
    const next = await loadCleanSessionStreamFromBridge(taskInvoke, {
      runId,
      logsDir: status?.logs_dir,
      offset: nextCleanSessionOffset(previousState),
      maxBytes: 4096,
    });
    setCleanSessions((current) => ({ ...current, [runId]: appendCleanSessionChunk(previousState, next) }));
  }, [cleanSessions, status?.logs_dir]);

  const loadPhase6Workspace = useCallback(async () => {
    setPhase6State({ mode: "loading" });
    setPhase6State(await loadPhase6OverviewFromBridge(phase6Invoke));
  }, []);

  const loadLinkedPanels = useCallback(async (taskId: string) => {
    setTaskLinkedPanels({ mode: "loading", taskId });
    const linkedState = await loadTaskLinkedPanelsFromBridge(taskInvoke, taskId);
    setTaskLinkedPanels(linkedState);
    if (linkedState.mode === "ready") {
      const primaryRunId = linkedState.runs[0]?.id ?? null;
      setManualReview((current) => resetManualReviewForTask(current, taskId, primaryRunId));
      await Promise.all(linkedState.runs.map((run) => loadCleanSession(run.id)));
    } else {
      setManualReview((current) => resetManualReviewForTask(current, taskId, null));
    }
  }, [loadCleanSession]);

  const loadNoteLinkedPanels = useCallback(async (noteId: string) => {
    setNoteLinkedPanels({ mode: "loading", entityType: "note", entityId: noteId });
    setNoteLinkedPanels(await loadContentLinkedPanelsFromBridge(noteInvoke, "note", noteId));
  }, []);

  const loadFileLinkedPanels = useCallback(async (relativePath: string) => {
    const fileId = fileReferenceEntityId(fileBridgeUi.rootKey, relativePath);
    setFileLinkedPanels({ mode: "loading", entityType: "file", entityId: fileId });
    setFileLinkedPanels(await loadContentLinkedPanelsFromBridge(fileInvoke, "file", fileId));
  }, [fileBridgeUi.rootKey]);


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
        : { ...current.state, selectedTaskId: null },
    }));
    setTaskLinkedPanels({ mode: "idle", taskId: null });
    setCleanSessions({});
    setRunControls((current) => updateRunControlsDraft(current, { taskId: null, clearStatus: true }));
    setManualReview(createInitialManualReviewState(null, null));
  }, []);

  const handleCreateTask = useCallback(async (form: TaskFormDraft) => {
    const next = await createTaskThroughBridge(taskInvoke, form);
    setTaskBridgeUi(next);
    if (next.state.mode === "ready") setTodayTasks({ state: "ready", records: next.state.tasks });
    if (next.state.mode === "error") setTodayTasks({ state: "unavailable", reason: bridgeErrorReason("Native task", next.state.error) });
    if (next.state.mode === "ready" && next.state.selectedTaskId) {
      setRunControls((current) => resetRunControlsForTask(current, next.state.selectedTaskId, status?.visible_root || ""));
      await loadLinkedPanels(next.state.selectedTaskId);
    }
  }, [loadLinkedPanels, status?.visible_root]);

  const handleUpdateTask = useCallback(async (taskId: string, form: TaskFormDraft) => {
    const next = await updateTaskThroughBridge(taskInvoke, taskId, form);
    setTaskBridgeUi(next);
    if (next.state.mode === "ready") setTodayTasks({ state: "ready", records: next.state.tasks });
    if (next.state.mode === "error") setTodayTasks({ state: "unavailable", reason: bridgeErrorReason("Native task", next.state.error) });
    if (next.state.mode === "ready" && next.state.selectedTaskId) await loadLinkedPanels(next.state.selectedTaskId);
  }, [loadLinkedPanels]);

  const handleTaskLifecycleAction = useCallback(async (taskId: string, action: { kind: "status"; status: string } | { kind: "archive" } | { kind: "delete" }) => {
    const next = await performTaskActionThroughBridge(taskInvoke, taskBridgeUi, taskId, action);
    setTaskBridgeUi(next);
    if (next.state.mode === "ready") setTodayTasks({ state: "ready", records: next.state.tasks });
    if (next.state.mode === "error") setTodayTasks({ state: "unavailable", reason: bridgeErrorReason("Native task", next.state.error) });
    if (next.state.mode === "ready" && next.state.selectedTaskId) await loadLinkedPanels(next.state.selectedTaskId);
    if (action.kind === "delete") setTaskLinkedPanels({ mode: "idle", taskId: null });
  }, [loadLinkedPanels, taskBridgeUi]);

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
    if (state.mode === "ready" && state.selectedTaskId) {
      setRunControls((current) => resetRunControlsForTask(current, state.selectedTaskId, status?.visible_root || ""));
      await loadLinkedPanels(state.selectedTaskId);
    }
  }, [loadLinkedPanels, status?.visible_root]);

  const handleRunDraftChange = useCallback((patch: Partial<RunControlsDraft>) => {
    setRunControls((current) => updateRunControlsDraft(current, patch));
  }, []);

  const handleClearRunStatus = useCallback(() => {
    setRunControls((current) => updateRunControlsDraft(current, { clearStatus: true, taskId: current.draft.taskId, cwd: current.draft.cwd }));
  }, []);

  const handleStartRun = useCallback(async () => {
    setRunControls((current) => ({ ...current, mode: "starting", errorMessage: null, validationErrors: [] }));
    const next = await startRunThroughBridge(taskInvoke, runControls, { logsDir: status?.logs_dir });
    setRunControls(next);
    if (next.mode === "ready" && next.activeRun?.id) {
      await loadCleanSession(next.activeRun.id);
      if (next.draft.taskId) await loadLinkedPanels(next.draft.taskId);
    }
  }, [loadCleanSession, loadLinkedPanels, runControls, status?.logs_dir]);

  const handleCancelRun = useCallback(async () => {
    setRunControls((current) => ({ ...current, mode: "cancelling", errorMessage: null, validationErrors: [] }));
    const next = await cancelRunThroughBridge(taskInvoke, runControls, "Cancelled from Zoid task detail");
    setRunControls(next);
    if (next.mode === "ready" && next.activeRun?.id) await loadCleanSession(next.activeRun.id);
  }, [loadCleanSession, runControls]);

  const handleManualReviewDraftChange = useCallback((patch: Partial<ManualReviewDraft>) => {
    setManualReview((current) => updateManualReviewDraft(current, patch));
  }, []);

  const handleClearManualReview = useCallback(() => {
    setManualReview((current) => updateManualReviewDraft(current, { clear: true }));
  }, []);

  const handleSubmitManualReview = useCallback(async () => {
    setManualReview((current) => ({ ...current, mode: "submitting", errorMessage: null, validationErrors: [] }));
    const next = await createManualReviewThroughBridge(taskInvoke, manualReview);
    setManualReview(next);
    if (next.mode === "ready" && next.draft.taskId) await loadLinkedPanels(next.draft.taskId);
  }, [loadLinkedPanels, manualReview]);

  useEffect(() => {
    let cancelled = false;
    if (activeWorkspace === "notes") {
      setNoteBridgeUi((current) => ({ ...current, state: { mode: "loading", selectedNoteId: current.state.selectedNoteId } }));
      refreshNotesFromBridge(noteInvoke, { selectedNoteId: noteBridgeUi.state.selectedNoteId }).then((state) => {
        if (!cancelled) setNoteBridgeUi((current) => ({ ...current, state }));
      });
    }
    if (activeWorkspace === "files") {
      setFileBridgeUi((current) => ({ ...current, state: { mode: "loading", rootKey: current.rootKey, relativePath: current.relativePath, selectedPath: current.state.selectedPath } }));
      browseFilesFromBridge(fileInvoke, { rootKey: fileBridgeUi.rootKey, relativePath: fileBridgeUi.relativePath, selectedPath: fileBridgeUi.state.selectedPath }).then((state) => {
        if (!cancelled) setFileBridgeUi((current) => ({ ...current, state }));
      });
    }
    if (["inbox", "calendar", "business", "products"].includes(activeWorkspace)) {
      loadPhase6OverviewFromBridge(phase6Invoke).then((state) => {
        if (!cancelled) setPhase6State(state);
      });
    }
    return () => { cancelled = true; };
  }, [activeWorkspace]);

  const handleNoteFormChange = useCallback((form: NoteFormDraft) => setNoteBridgeUi((current) => ({ ...current, form, formErrors: {} })), []);
  const handleCreateNote = useCallback(async (form: NoteFormDraft) => setNoteBridgeUi(await createNoteThroughBridge(noteInvoke, form)), []);
  const handleEditNote = useCallback(async (noteId: string, form: NoteFormDraft) => setNoteBridgeUi(await editNoteThroughBridge(noteInvoke, noteId, form)), []);
  const handleSelectNote = useCallback(async (noteId: string) => {
    const state = await selectNoteThroughBridge(noteInvoke, noteId);
    setNoteBridgeUi((current) => ({ ...current, form: state.mode === "ready" ? formDraftForNote(state.notes.find((note) => note.id === state.selectedNoteId) ?? state.notes[0] ?? { id: "", title: "", slug: "", relative_path: "Notes/untitled.md", status: "active", conflict_state: "clean", body_digest: "", metadata_json: "{}", markdown: "" }) : current.form, formErrors: {}, state }));
    if (state.mode === "ready" && state.selectedNoteId) await loadNoteLinkedPanels(state.selectedNoteId);
  }, [loadNoteLinkedPanels]);
  const handleRefreshNotes = useCallback(async () => {
    const state = await refreshNotesFromBridge(noteInvoke, { selectedNoteId: noteBridgeUi.state.selectedNoteId });
    setNoteBridgeUi((current) => ({ ...current, state }));
  }, [noteBridgeUi.state.selectedNoteId]);
  const handleScanNotes = useCallback(async () => {
    const state = await scanNotesThroughBridge(noteInvoke, noteBridgeUi.state.selectedNoteId);
    setNoteBridgeUi((current) => ({ ...current, state }));
  }, [noteBridgeUi.state.selectedNoteId]);
  const handleTrashNote = useCallback(async (noteId: string) => {
    const state = await trashNoteThroughBridge(noteInvoke, noteId);
    setNoteBridgeUi((current) => ({ ...current, state }));
  }, []);

  const handleFileBrowsePathChange = useCallback((rootKey: string, relativePath: string) => setFileBridgeUi((current) => ({ ...current, rootKey, relativePath })), []);
  const handleRefreshFiles = useCallback(async () => {
    const state = await browseFilesFromBridge(fileInvoke, { rootKey: fileBridgeUi.rootKey, relativePath: fileBridgeUi.relativePath, selectedPath: fileBridgeUi.state.selectedPath });
    setFileBridgeUi((current) => ({ ...current, state }));
  }, [fileBridgeUi.rootKey, fileBridgeUi.relativePath, fileBridgeUi.state.selectedPath]);
  const handleSelectFile = useCallback(async (relativePath: string) => {
    const state = await previewFileThroughBridge(fileInvoke, fileBridgeUi.state, relativePath);
    setFileBridgeUi((current) => ({ ...current, actionDraft: { ...current.actionDraft, source_relative_path: relativePath }, state }));
    await loadFileLinkedPanels(relativePath);
  }, [fileBridgeUi.state, loadFileLinkedPanels]);
  const handleFileActionDraftChange = useCallback((actionDraft: FileActionDraft) => setFileBridgeUi((current) => ({ ...current, actionDraft, actionErrors: [] })), []);
  const handlePerformFileActionClick = useCallback(async () => setFileBridgeUi(await performFileActionThroughBridge(fileInvoke, fileBridgeUi)), [fileBridgeUi]);

  const refreshContentWithStatus = useCallback(async (actionStatus: string) => {
    await loadContentWorkspace();
    setContentWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus } : current);
  }, [loadContentWorkspace]);

  const handleCreateContentDraft = useCallback(async () => {
    try {
      const plan = await invoke<ContentPlanRecord>("create_content_plan_command", {
        request: {
          title: "Phase 5 local content plan",
          pillar: "operations",
          owner_actor_type: "human",
          metadata_json: "{}",
        },
      });
      const piece = await invoke<ContentPieceRecord>("create_content_piece_command", {
        request: {
          plan_id: plan.id,
          title: "Draft-first OmniSocials post",
          body_markdown: "Draft content created locally. External publishing remains fail-closed.",
          platforms: ["linkedin", "instagram"],
          required_gate: "specialist_review",
          metadata_json: "{}",

        },
      });
      await invoke<MediaAssetRecord>("add_media_asset_reference_command", {
        request: {
          piece_id: piece.id,
          asset_kind: "image",
          storage_ref: "assets/content/phase-5-placeholder.png",
          mime_type: "image/png",
          byte_size: 1024,
          width: 1080,
          height: 1080,
          duration_seconds: null,
          alt_text: "Placeholder content asset",
          metadata_json: "{}",

        },
      });
      await refreshContentWithStatus("Created local plan, draft, and media reference.");
    } catch (error) {
      setContentWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Content draft create", error) } : { mode: "error", error: bridgeErrorReason("Content draft create", error) });
    }
  }, [refreshContentWithStatus]);

  const handleUpdateContentDraft = useCallback(async (pieceId: string) => {
    try {
      await invoke<ContentPieceRecord>("update_content_piece_draft_command", {
        request: {
          piece_id: pieceId,
          body_markdown: "Draft moved to review-ready locally. External publishing remains fail-closed.",
          status: "review_ready",
          metadata_json: "{}",
        },

      });
      await refreshContentWithStatus("Updated draft to review-ready.");
    } catch (error) {
      setContentWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Content draft update", error) } : current);
    }
  }, [refreshContentWithStatus]);

  const handleCreateContentReviewGate = useCallback(async (pieceId: string) => {
    try {
      await invoke<ContentReviewGateRecord>("create_content_review_gate_command", {
        request: {
          piece_id: pieceId,
          gate_type: "specialist_review",
          reviewer_actor_type: "reviewer",
          reviewer_actor_id: null,
          evidence_summary: "Awaiting specialist review.",
          metadata_json: "{}",
        },

      });
      await refreshContentWithStatus("Created specialist review gate.");
    } catch (error) {
      setContentWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Review gate create", error) } : current);
    }
  }, [refreshContentWithStatus]);

  const handleApproveContentReviewGate = useCallback(async (gateId: string) => {
    try {
      await invoke<ContentReviewGateRecord>("approve_content_review_gate_command", {
        gateId,
        request: { evidence_summary: "Approved locally for schedule-intent testing.", reviewer_actor_type: "reviewer", reviewer_actor_id: "local-user", metadata_json: "{}" },

      });
      await refreshContentWithStatus("Approved specialist review gate.");
    } catch (error) {
      setContentWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Review gate approve", error) } : current);
    }
  }, [refreshContentWithStatus]);

  const handleRejectContentReviewGate = useCallback(async (gateId: string) => {
    try {
      await invoke<ContentReviewGateRecord>("reject_content_review_gate_command", {
        gateId,
        request: { evidence_summary: "Rejected locally for schedule-gate testing.", reviewer_actor_type: "reviewer", reviewer_actor_id: "local-user", metadata_json: "{}" },

      });
      await refreshContentWithStatus("Rejected specialist review gate.");
    } catch (error) {
      setContentWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Review gate reject", error) } : current);
    }
  }, [refreshContentWithStatus]);

  const handleAttemptContentScheduleIntent = useCallback(async (pieceId: string, platform: string) => {
    try {
      await invoke<ContentScheduleRecord>("create_content_schedule_command", {
        request: {
          piece_id: pieceId,
          platform,
          scheduled_for: "2026-06-06T09:00:00Z",
          confirmation_id: null,
          metadata_json: "{}",

        },
      });
      await refreshContentWithStatus("Created local schedule intent.");
    } catch (error) {
      await refreshContentWithStatus(bridgeErrorReason("Schedule intent", error));
    }
  }, [refreshContentWithStatus]);

  const handleCancelContentSchedule = useCallback(async (scheduleId: string) => {
    try {
      await invoke<ContentScheduleRecord>("cancel_content_schedule_command", { scheduleId });
      await refreshContentWithStatus("Cancelled local schedule intent.");
    } catch (error) {
      setContentWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Schedule cancel", error) } : current);
    }
  }, [refreshContentWithStatus]);

  const handleRecordFailClosedUpload = useCallback(async (pieceId: string, platform: string) => {
    try {
      await invoke<ContentVerificationRecord>("omnisocials_upload_media_command", { request: { piece_id: pieceId, platform, schedule_id: null } });

      await refreshContentWithStatus("Recorded fail-closed upload verification.");
    } catch (error) {
      setContentWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Fail-closed upload check", error) } : current);
    }
  }, [refreshContentWithStatus]);

  const handleRecordFailClosedSchedule = useCallback(async (pieceId: string, platform: string, scheduleId?: string | null) => {
    try {
      await invoke<ContentVerificationRecord>("omnisocials_schedule_content_command", { request: { piece_id: pieceId, platform, schedule_id: scheduleId ?? null } });

      await refreshContentWithStatus("Recorded fail-closed schedule verification.");
    } catch (error) {
      setContentWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Fail-closed schedule check", error) } : current);
    }
  }, [refreshContentWithStatus]);

  const handleRecordFailClosedPublish = useCallback(async (pieceId: string, platform: string, scheduleId?: string | null) => {
    try {
      await invoke<ContentVerificationRecord>("omnisocials_publish_content_command", { request: { piece_id: pieceId, platform, schedule_id: scheduleId ?? null } });

      await refreshContentWithStatus("Recorded fail-closed publish verification.");
    } catch (error) {
      setContentWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Fail-closed publish check", error) } : current);
    }
  }, [refreshContentWithStatus]);



  const refreshCodeWithStatus = useCallback(async (actionStatus: string, extras: Partial<Extract<CodeWorkspaceState, { mode: "ready" }>> = {}) => {
    await loadCodeWorkspace();
    setCodeWorkspace((current) => current.mode === "ready" ? { ...current, ...extras, actionStatus } : current);
  }, [loadCodeWorkspace]);

  const handleAddRepoProfile = useCallback(async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    try {
      const repo = await invoke<CodeRepoRecord>("add_repo_profile_command", { request: {
        display_name: appFormString(form, "display_name") ?? "",
        root_path: appFormString(form, "root_path") ?? "",
        profile_type: appFormString(form, "profile_type") ?? "product_app",
        default_branch: appFormString(form, "default_branch") ?? null,
        package_manager: appFormString(form, "package_manager") ?? null,
        linked_product_id: appFormString(form, "linked_product_id") ?? null,
        metadata_json: appFormString(form, "metadata_json") ?? "{}",
      } });
      await refreshCodeWithStatus(`Added repo profile ${repo.id}.`);
    } catch (error) {
      setCodeWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Repo profile add", error) } : { mode: "error", error: bridgeErrorReason("Repo profile add", error) });
    }
  }, [refreshCodeWithStatus]);

  const handleLinkRepoEntity = useCallback(async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    try {
      await invoke("link_repo_entity_command", { request: {
        repo_id: appFormString(form, "repo_id") ?? "",
        target_type: appFormString(form, "target_type") ?? "product",
        target_id: appFormString(form, "target_id") ?? "",
        relation_type: appFormString(form, "relation_type") ?? "belongs_to",
        metadata_json: appFormString(form, "metadata_json") ?? "{}",
      } });
      await refreshCodeWithStatus("Linked repo to target entity.");
    } catch (error) {
      setCodeWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Repo link", error) } : current);
    }
  }, [refreshCodeWithStatus]);

  const handleCreateLaunchGate = useCallback(async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    try {
      const gate = await invoke<LaunchGateRecord>("create_launch_gate_command", { request: {
        repo_id: appFormString(form, "repo_id") ?? "",
        product_id: appFormString(form, "product_id") ?? null,
        task_id: appFormString(form, "task_id") ?? null,
        metadata_json: appFormString(form, "metadata_json") ?? "{}",
      } });
      await refreshCodeWithStatus(`Created launch gate ${gate.id}.`, { lastLaunchGate: gate });
    } catch (error) {
      setCodeWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Launch gate create", error) } : current);
    }
  }, [refreshCodeWithStatus]);

  const handleAddLaunchGateEvidence = useCallback(async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    try {
      const evidence = await invoke<LaunchGateEvidenceRecord>("add_launch_gate_evidence_command", { request: {
        launch_gate_id: appFormString(form, "launch_gate_id") ?? "",
        evidence_type: appFormString(form, "evidence_type") ?? "manual_note",
        label: appFormString(form, "label") ?? "",
        url: appFormString(form, "url") ?? null,
        status_code: appFormInteger(form, "status_code"),
        manual_note: appFormString(form, "manual_note") ?? null,
        metadata_json: appFormString(form, "metadata_json") ?? "{}",
      } });
      await refreshCodeWithStatus(`Added launch evidence ${evidence.id}.`, { lastEvidence: evidence });
    } catch (error) {
      setCodeWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Launch evidence add", error) } : current);
    }
  }, [refreshCodeWithStatus]);

  const handleEvaluateLaunchGate = useCallback(async (event: FormEvent<HTMLFormElement>) => {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const launchGateId = appFormString(form, "launch_gate_id") ?? "";
    try {
      const gate = await invoke<LaunchGateRecord>("evaluate_launch_gate_command", { launchGateId });
      await refreshCodeWithStatus(`Evaluated launch gate: ${gate.final_verdict ?? gate.state}.`, { lastLaunchGate: gate });
    } catch (error) {
      setCodeWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Launch gate evaluate", error) } : current);
    }
  }, [refreshCodeWithStatus]);
  const handleBrowserDraftChange = useCallback((patch: Partial<BrowserWorkspaceDraft>) => {
    setBrowserWorkspace((current) => updateBrowserDraft(current, patch));
  }, []);
  const handleSaveBrowserUrl = useCallback(async () => {
    setBrowserWorkspace(await saveWorkUrlThroughBridge(browserInvoke, browserWorkspace));
  }, [browserWorkspace]);
  const handleCreateBrowserCapture = useCallback(async () => {
    setBrowserWorkspace(await createCaptureThroughBridge(browserInvoke, browserWorkspace));
  }, [browserWorkspace]);
  const handleAttachBrowserCapture = useCallback(async () => {
    setBrowserWorkspace(await attachCaptureThroughBridge(browserInvoke, browserWorkspace));
  }, [browserWorkspace]);
  const handleSelectBrowserCapture = useCallback((captureId: string) => {
    setBrowserWorkspace((current) => current.mode === "ready" ? { ...current, selectedCaptureId: captureId, errorMessage: null } : current);
  }, []);
  const handlePersistBrowserWidget = useCallback(async (widget: WidgetConfigRecord) => {
    setBrowserWorkspace(await updateWidgetThroughBridge(browserInvoke, browserWorkspace, widget));
  }, [browserWorkspace]);
  const handleResetBrowserWidgets = useCallback(async () => {
    setBrowserWorkspace(await resetWidgetsThroughBridge(browserInvoke, browserWorkspace));
  }, [browserWorkspace]);


  const workspaceRegistry = useMemo(() => buildWorkspaceRegistryView(status, statusError), [status, statusError]);
  const handleDryRunLogCleanup = useCallback(async () => {
    const result = await dryRunLogCleanup(releaseInvoke, "default");
    setReleaseHardening((current) => current.mode === "ready" ? { ...current, cleanupResult: result } : current);
  }, []);

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
  const linkedRunIds = useMemo(
    () => taskLinkedPanels.mode === "ready" ? taskLinkedPanels.runs.map((run) => run.id) : [],
    [taskLinkedPanels],
  );
  const taskScopedInbox = useMemo<InboxDataState<InboxNotificationRecord>>(() => {
    if (todayInbox.state === "checking") return { state: "checking" };
    if (todayInbox.state === "unavailable") return { state: "unavailable", reason: todayInbox.reason };
    return buildTaskScopedInboxState(taskLinkedPanels.taskId, todayInbox.records as InboxNotificationRecord[], linkedRunIds);
  }, [linkedRunIds, taskLinkedPanels.taskId, todayInbox]);
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

        <div className={`split-view ${active?.id === "notes" || active?.id === "files" || active?.id === "content" ? "native-editor-active" : ""}`}>
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
                onUpdateTaskStatus={(taskId, status) => handleTaskLifecycleAction(taskId, { kind: "status", status })}
                onArchiveTask={(taskId) => handleTaskLifecycleAction(taskId, { kind: "archive" })}
                onDeleteTask={(taskId) => handleTaskLifecycleAction(taskId, { kind: "delete" })}
                linkedPanels={
                  <TaskLinkedPanels
                    state={taskLinkedPanels}
                    cleanSessions={cleanSessions}
                    inboxState={taskScopedInbox}
                    manualReview={manualReview}
                    runControls={
                      <RunControlsPanel
                        state={runControls}
                        onDraftChange={handleRunDraftChange}
                        onStart={handleStartRun}
                        onCancel={handleCancelRun}
                        onClear={handleClearRunStatus}
                      />
                    }
                    onManualReviewDraftChange={handleManualReviewDraftChange}
                    onSubmitManualReview={handleSubmitManualReview}
                    onClearManualReview={handleClearManualReview}
                    onRefresh={loadLinkedPanels}
                    onRefreshCleanSession={loadCleanSession}
                  />
                }
                state={taskBridgeUi.state}
              />
            ) : active?.id === "code" ? (
              <CodeWorkspace
                state={codeWorkspace}
                onRefresh={loadCodeWorkspace}
                actions={{
                  addRepo: handleAddRepoProfile,
                  linkRepo: handleLinkRepoEntity,
                  createLaunchGate: handleCreateLaunchGate,
                  addLaunchGateEvidence: handleAddLaunchGateEvidence,
                  evaluateLaunchGate: handleEvaluateLaunchGate,

                }}
              />
            ) : active?.id === "content" ? (
              <ContentWorkspace
                state={contentWorkspace}
                onRefresh={loadContentWorkspace}
                actions={{
                  createDraft: handleCreateContentDraft,
                  updateDraft: handleUpdateContentDraft,
                  createReviewGate: handleCreateContentReviewGate,
                  approveReviewGate: handleApproveContentReviewGate,
                  rejectReviewGate: handleRejectContentReviewGate,
                  attemptScheduleIntent: handleAttemptContentScheduleIntent,
                  cancelSchedule: handleCancelContentSchedule,
                  recordFailClosedUpload: handleRecordFailClosedUpload,
                  recordFailClosedSchedule: handleRecordFailClosedSchedule,
                  recordFailClosedPublish: handleRecordFailClosedPublish,
                }}
              />
            ) : active?.id === "notes" ? (
              <NoteWorkspace
                form={noteBridgeUi.form}
                formErrors={noteBridgeUi.formErrors}
                onCreateNote={handleCreateNote}
                onEditNote={handleEditNote}
                onFormChange={handleNoteFormChange}
                onRefresh={handleRefreshNotes}
                onScan={handleScanNotes}
                onSelectNote={handleSelectNote}
                onTrashNote={handleTrashNote}
                linkedPanels={<ContentLinkedPanels state={noteLinkedPanels} onRefresh={loadNoteLinkedPanels} />}
                state={noteBridgeUi.state}
              />
            ) : active?.id === "browser" ? (
              <BrowserWorkspace
                state={browserWorkspace}
                actions={{
                  refresh: loadBrowserWorkspace,
                  updateDraft: handleBrowserDraftChange,
                  saveUrl: handleSaveBrowserUrl,
                  createCapture: handleCreateBrowserCapture,
                  attachCapture: handleAttachBrowserCapture,
                  selectCapture: handleSelectBrowserCapture,
                  persistWidget: handlePersistBrowserWidget,
                  resetWidgets: handleResetBrowserWidgets,
                }}
              />
            ) : ["inbox", "calendar", "business", "products"].includes(active?.id ?? "") ? (
              <Phase6Workspace workspaceId={active?.id ?? "inbox"} state={phase6State} onRefresh={loadPhase6Workspace} invoke={phase6Invoke} />
            ) : active?.id === "files" ? (
              <FileWorkspace
                actionDraft={fileBridgeUi.actionDraft}
                actionErrors={fileBridgeUi.actionErrors}
                onActionDraftChange={handleFileActionDraftChange}
                onBrowsePathChange={handleFileBrowsePathChange}
                onPerformAction={handlePerformFileActionClick}
                onRefresh={handleRefreshFiles}
                onSelectFile={handleSelectFile}
                linkedPanels={<ContentLinkedPanels state={fileLinkedPanels} onRefresh={() => fileBridgeUi.state.selectedPath && loadFileLinkedPanels(fileBridgeUi.state.selectedPath)} />}
                relativePath={fileBridgeUi.relativePath}
                rootKey={fileBridgeUi.rootKey}
                state={fileBridgeUi.state}
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

            <ReleaseHardeningPanel state={releaseHardening} onRefresh={loadReleaseHardening} onDryRunCleanup={handleDryRunLogCleanup} />
          </InspectorPanel>
        </div>
      </section>
    </main>
  );
}

export default App;
