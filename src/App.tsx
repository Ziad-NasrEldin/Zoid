import { invoke } from "@tauri-apps/api/core";
import type { ReactNode } from "react";
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
const noteInvoke: NoteBridgeInvoke = (command, args) => invoke(command, args);
const fileInvoke: FileBridgeInvoke = (command, args) => invoke(command, args);

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

type CodeWorkspaceState =
  | { mode: "loading" }
  | { mode: "error"; error: string }
  | { mode: "ready"; repos: CodeRepoRecord[]; integrations: CodeIntegrationRecord[]; policy: CodePolicyDecision };

function CodeWorkspace({ state, onRefresh }: { state: CodeWorkspaceState; onRefresh: () => void }) {
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
        {state.repos.length > 0 ? (
          <ul className="compact-list">
            {state.repos.map((repo) => (
              <li key={repo.id}>
                <div><strong>{repo.display_name}</strong><span>{repo.profile_type}</span></div>
                <p>{repo.root_path}</p>
              </li>
            ))}
          </ul>
        ) : (
          <EmptyState icon="⌘">No repos are fabricated. Add/list/profile/link is native-only.</EmptyState>
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

      <InfoCard>
        <p className="eyebrow">Launch Gate</p>
        <h3>Evidence required</h3>
        <p>Commit/push/merge/deploy actions are policy previews only until real evidence is captured.</p>
        <StatusBadge tone="blocked">{state.policy.category}</StatusBadge>
        <p className="muted-copy">{state.policy.reason}</p>
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
  const [contentWorkspace, setContentWorkspace] = useState<ContentWorkspaceState>({ mode: "loading" });

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
      setCodeWorkspace({ mode: "ready", repos, integrations, policy });
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

  useEffect(() => {
    void loadCodeWorkspace();
    void loadContentWorkspace();
  }, [loadCodeWorkspace, loadContentWorkspace]);

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
        request: { title: "Phase 5 local content plan", pillar: "operations", ownerActorType: "human", metadataJson: "{}" },
      });
      const piece = await invoke<ContentPieceRecord>("create_content_piece_command", {
        request: {
          planId: plan.id,
          title: "Draft-first OmniSocials post",
          bodyMarkdown: "Draft content created locally. External publishing remains fail-closed.",
          platforms: ["linkedin", "instagram"],
          requiredGate: "specialist_review",
          metadataJson: "{}",
        },
      });
      await invoke<MediaAssetRecord>("add_media_asset_reference_command", {
        request: {
          pieceId: piece.id,
          assetKind: "image",
          storageRef: "assets/content/phase-5-placeholder.png",
          mimeType: "image/png",
          byteSize: 1024,
          width: 1080,
          height: 1080,
          durationSeconds: null,
          altText: "Placeholder content asset",
          metadataJson: "{}",
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
        pieceId,
        request: { bodyMarkdown: "Draft moved to review-ready locally. External publishing remains fail-closed.", status: "review_ready", metadataJson: "{}" },
      });
      await refreshContentWithStatus("Updated draft to review-ready.");
    } catch (error) {
      setContentWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Content draft update", error) } : current);
    }
  }, [refreshContentWithStatus]);

  const handleCreateContentReviewGate = useCallback(async (pieceId: string) => {
    try {
      await invoke<ContentReviewGateRecord>("create_content_review_gate_command", {
        request: { pieceId, gateType: "specialist_review", reviewerActorType: "reviewer", reviewerActorId: null, evidenceSummary: "Awaiting specialist review.", metadataJson: "{}" },
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
        request: { evidenceSummary: "Approved locally for schedule-intent testing.", reviewerActorType: "reviewer", reviewerActorId: "local-user", metadataJson: "{}" },
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
        request: { evidenceSummary: "Rejected locally for schedule-gate testing.", reviewerActorType: "reviewer", reviewerActorId: "local-user", metadataJson: "{}" },
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
          pieceId,
          platform,
          scheduledFor: "2026-06-06T09:00:00Z",
          confirmationId: null,
          metadataJson: "{}",
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
      await invoke<ContentVerificationRecord>("omnisocials_upload_media_command", { request: { pieceId, platform, scheduleId: null } });
      await refreshContentWithStatus("Recorded fail-closed upload verification.");
    } catch (error) {
      setContentWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Fail-closed upload check", error) } : current);
    }
  }, [refreshContentWithStatus]);

  const handleRecordFailClosedSchedule = useCallback(async (pieceId: string, platform: string, scheduleId?: string | null) => {
    try {
      await invoke<ContentVerificationRecord>("omnisocials_schedule_content_command", { request: { pieceId, platform, scheduleId: scheduleId ?? null } });
      await refreshContentWithStatus("Recorded fail-closed schedule verification.");
    } catch (error) {
      setContentWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Fail-closed schedule check", error) } : current);
    }
  }, [refreshContentWithStatus]);

  const handleRecordFailClosedPublish = useCallback(async (pieceId: string, platform: string, scheduleId?: string | null) => {
    try {
      await invoke<ContentVerificationRecord>("omnisocials_publish_content_command", { request: { pieceId, platform, scheduleId: scheduleId ?? null } });
      await refreshContentWithStatus("Recorded fail-closed publish verification.");
    } catch (error) {
      setContentWorkspace((current) => current.mode === "ready" ? { ...current, actionStatus: bridgeErrorReason("Fail-closed publish check", error) } : current);
    }
  }, [refreshContentWithStatus]);

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

        <div className={`split-view ${active?.id === "notes" || active?.id === "files" ? "native-editor-active" : ""}`}>
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
              <CodeWorkspace state={codeWorkspace} onRefresh={loadCodeWorkspace} />
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
          </InspectorPanel>
        </div>
      </section>
    </main>
  );
}

export default App;
