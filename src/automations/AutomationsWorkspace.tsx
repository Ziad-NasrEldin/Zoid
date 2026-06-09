import { AlertTriangle, Clock3, Play, RefreshCcw, Shield, TerminalSquare, Trash2 } from "lucide-react";
import { useEffect, useMemo, useRef, useState, type KeyboardEvent } from "react";
import { listHermesAutomations, manageHermesCronJob } from "./automationClient";
import { deriveAutomationNavStatus, filterAutomationJobs, getAutomationStatusKind, summarizeAutomationJobs } from "./automationViewModel";
import type { AutomationAction, AutomationCronJob, AutomationFilter, AutomationList } from "./types";

type AutomationsWorkspaceProps = {
  onStatusChange?: (status: "ready" | "idle" | "blocked") => void;
};

type PendingAction = {
  job: AutomationCronJob;
  action: Extract<AutomationAction, "run" | "remove">;
} | null;

const filters: Array<{ label: string; value: AutomationFilter }> = [
  { label: "All", value: "all" },
  { label: "Running", value: "running" },
  { label: "Paused", value: "paused" },
  { label: "Failed", value: "failed" },
  { label: "Script-only", value: "script" },
];

function formatDate(value: string | null) {
  if (!value) return "—";
  const date = parseAutomationDate(value);
  if (Number.isNaN(date.getTime())) return value;
  return date.toLocaleString();
}

function parseAutomationDate(value: string) {
  const trimmed = value.trim();
  const numeric = Number(trimmed);
  if (Number.isFinite(numeric) && trimmed !== "") {
    const absolute = Math.abs(numeric);
    const milliseconds = absolute > 0 && absolute < 10_000_000_000 ? numeric * 1000 : numeric;
    return new Date(milliseconds);
  }
  return new Date(trimmed);
}

function comparableAutomationTime(value: string | null) {
  if (!value) return Number.POSITIVE_INFINITY;
  const date = parseAutomationDate(value);
  return Number.isNaN(date.getTime()) ? Number.POSITIVE_INFINITY : date.getTime();
}

function statusLabel(job: AutomationCronJob) {
  const kind = getAutomationStatusKind(job);
  if (kind === "error") return "Failed";
  if (kind === "paused") return "Paused";
  if (kind === "ok") return "OK";
  return "Unknown";
}

function displayName(value: string | null | undefined, fallback = "—") {
  return value && value.trim() ? value : fallback;
}

function formatAutomationError(error: unknown) {
  const message = error instanceof Error ? error.message : String(error);
  if (message.includes("invoke") || message.includes("__TAURI__")) {
    return "Hermes desktop bridge is unavailable in this preview. Open Zoid in the Tauri desktop shell to read native automation state.";
  }
  return message;
}

function Detail({ label, value }: { label: string; value: string | null | undefined }) {
  return (
    <span className="automation-detail">
      <strong>{label}</strong>
      <span title={value ?? undefined}>{displayName(value)}</span>
    </span>
  );
}

function SummaryCard({ label, value, tone = "neutral" }: { label: string; value: string | number; tone?: "neutral" | "ink" | "seal" }) {
  return (
    <article className={`automation-summary-card automation-summary-card--${tone}`}>
      <span>{label}</span>
      <strong title={String(value)}>{value}</strong>
    </article>
  );
}

export function AutomationsWorkspace({ onStatusChange }: AutomationsWorkspaceProps) {
  const [automationList, setAutomationList] = useState<AutomationList | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [busyJobId, setBusyJobId] = useState<string | null>(null);
  const [busyAction, setBusyAction] = useState<AutomationAction | null>(null);
  const [filter, setFilter] = useState<AutomationFilter>("all");
  const [searchQuery, setSearchQuery] = useState("");
  const [pendingAction, setPendingAction] = useState<PendingAction>(null);
  const [pendingActionError, setPendingActionError] = useState<string | null>(null);
  const [actionMessage, setActionMessage] = useState<string | null>(null);
  const confirmPanelRef = useRef<HTMLDivElement | null>(null);
  const previousFocusRef = useRef<HTMLElement | null>(null);
  const actionInFlightRef = useRef(false);
  const actionRequestIdRef = useRef(0);
  const loadRequestIdRef = useRef(0);
  const isMountedRef = useRef(false);

  async function loadAutomations() {
    const requestId = loadRequestIdRef.current + 1;
    loadRequestIdRef.current = requestId;
    setIsLoading(true);
    setErrorMessage(null);
    try {
      const nextList = await listHermesAutomations();
      if (!isMountedRef.current || requestId !== loadRequestIdRef.current) return;
      setAutomationList(nextList);
      onStatusChange?.(deriveAutomationNavStatus(nextList, null));
    } catch (error) {
      if (!isMountedRef.current || requestId !== loadRequestIdRef.current) return;
      setErrorMessage(formatAutomationError(error));
      onStatusChange?.("blocked");
    } finally {
      if (isMountedRef.current && requestId === loadRequestIdRef.current) {
        setIsLoading(false);
      }
    }
  }

  useEffect(() => {
    isMountedRef.current = true;
    void loadAutomations();
    return () => {
      isMountedRef.current = false;
      loadRequestIdRef.current += 1;
    };
  }, []);

  useEffect(() => {
    if (!pendingAction) return undefined;
    previousFocusRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    window.requestAnimationFrame(() => {
      const firstFocusable = confirmPanelRef.current?.querySelector<HTMLElement>("button:not(:disabled), [href], input:not(:disabled), textarea:not(:disabled), select:not(:disabled), [tabindex]:not([tabindex='-1'])");
      (firstFocusable ?? confirmPanelRef.current)?.focus();
    });
    return () => {
      previousFocusRef.current?.focus?.();
      previousFocusRef.current = null;
    };
  }, [pendingAction]);

  function cancelPendingAction() {
    setPendingActionError(null);
    setPendingAction(null);
  }

  function openPendingAction(job: AutomationCronJob, action: Extract<AutomationAction, "run" | "remove">) {
    setPendingActionError(null);
    setPendingAction({ job, action });
  }

  function handleConfirmKeyDown(event: KeyboardEvent<HTMLDivElement>) {
    if (event.key === "Escape") {
      event.preventDefault();
      if (!isActionInFlight) cancelPendingAction();
      return;
    }
    if (event.key !== "Tab") return;
    const focusable = [...event.currentTarget.querySelectorAll<HTMLElement>("button:not(:disabled), [href], input:not(:disabled), textarea:not(:disabled), select:not(:disabled), [tabindex]:not([tabindex='-1'])")];
    if (focusable.length === 0) {
      event.preventDefault();
      event.currentTarget.focus();
      return;
    }
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    const active = document.activeElement;
    if (event.shiftKey && active === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && active === last) {
      event.preventDefault();
      first.focus();
    }
  }

  const summary = useMemo(() => automationList ? summarizeAutomationJobs(automationList) : null, [automationList]);
  const visibleJobs = useMemo(
    () => automationList ? filterAutomationJobs(automationList.jobs, filter, searchQuery) : [],
    [automationList, filter, searchQuery],
  );
  const failedJobs = useMemo(() => automationList?.jobs.filter((job) => getAutomationStatusKind(job) === "error") ?? [], [automationList]);
  const scriptJobs = useMemo(() => automationList?.jobs.filter((job) => job.noAgent || Boolean(job.script)) ?? [], [automationList]);
  const nextJob = useMemo(() => {
    return (automationList?.jobs ?? [])
      .filter((job) => Boolean(job.nextRunAt))
      .sort((a, b) => comparableAutomationTime(a.nextRunAt) - comparableAutomationTime(b.nextRunAt))[0] ?? null;
  }, [automationList]);
  const isActionInFlight = busyJobId !== null;

  async function runAction(job: AutomationCronJob, action: AutomationAction) {
    if (actionInFlightRef.current) return;
    if (action === "remove" && job.protected) return;
    actionInFlightRef.current = true;
    const requestId = actionRequestIdRef.current + 1;
    actionRequestIdRef.current = requestId;
    loadRequestIdRef.current += 1;
    setIsLoading(false);
    setBusyJobId(job.jobId);
    setBusyAction(action);
    setErrorMessage(null);
    setPendingActionError(null);
    setActionMessage(null);
    try {
      const nextList = await manageHermesCronJob(job.jobId, action);
      if (requestId !== actionRequestIdRef.current) return;
      if (action === "remove" && nextList.jobs.some((nextJob) => nextJob.jobId === job.jobId)) {
        throw new Error("Hermes read-back still includes this cron job after remove.");
      }
      setAutomationList(nextList);
      onStatusChange?.(deriveAutomationNavStatus(nextList, null));
      if (pendingAction?.job.jobId === job.jobId && pendingAction.action === action) {
        setPendingAction(null);
      }
      const actionVerb = action === "run" ? "Run requested for" : action === "remove" ? "Removed" : action === "pause" ? "Paused" : "Resumed";
      setActionMessage(`${actionVerb} “${job.name || job.jobId}”. Hermes provider read-back refreshed.`);
    } catch (error) {
      if (requestId !== actionRequestIdRef.current) return;
      const message = formatAutomationError(error);
      setErrorMessage(message);
      if (pendingAction?.job.jobId === job.jobId && pendingAction.action === action) {
        setPendingActionError(message);
      }
      onStatusChange?.("blocked");
    } finally {
      if (requestId === actionRequestIdRef.current) {
        actionInFlightRef.current = false;
        setBusyJobId(null);
        setBusyAction(null);
      }
    }
  }

  return (
    <section aria-label="Automations" className="automations-workspace-shell automation-sumi-e">
      <header className="automations-workspace-header">
        <div className="automation-hero-copy">
          <p className="kana-line">自動化</p>
          <h2>Hermes automations</h2>
          <p>Control the local cron ledger without making Zoid the source of truth. Every run, pause, resume, and removal is read back from Hermes before the interface claims it.</p>
          <p className="automation-reference-line">Provider-owned schedules · protected system jobs · watcher state is read-only</p>
        </div>
        <div className="automation-ink-clock" aria-hidden="true"><span /><span /><span /></div>
        <div className="automation-header-actions" aria-label="Automation profile and refresh">
          <span className="automation-profile-label">Profile: {automationList?.activeProfile ?? "default"}</span>
          <button className="automation-primary-button automation-refresh-button" disabled={isLoading || isActionInFlight || pendingAction !== null} onClick={loadAutomations} type="button">
            <RefreshCcw aria-hidden="true" size={16} />
            {isLoading ? "Refreshing" : "Refresh"}
          </button>
        </div>
      </header>

      <p className="automation-status-line" role="status">
        {errorMessage
          ? "Hermes automation bridge is blocked."
          : actionMessage ?? (isLoading
            ? "Refreshing Hermes automation ledger…"
            : automationList
              ? `Hermes ledger refreshed for ${automationList.activeProfile}.`
              : "Hermes automation ledger has not loaded yet.")}
      </p>

      {errorMessage ? (
        <div className="automation-alert" role="alert">
          <AlertTriangle aria-hidden="true" size={18} />
          <span>{errorMessage}</span>
        </div>
      ) : null}

      {actionMessage ? (
        <div className="automation-action-status" role="status" aria-live="polite">
          {actionMessage}
        </div>
      ) : null}

      <section className="automation-command-panel" aria-label="Automation command deck">
        <div className="automation-command-main">
          <span className="automation-panel-kicker">Control room</span>
          <h3>Cron ledger</h3>
          <p>{automationList ? `${visibleJobs.length} of ${automationList.jobs.length} jobs in view. Destructive operations require confirmation and provider read-back.` : "Loading Hermes cron jobs from the active local profile."}</p>
        </div>
        <div className="automation-next-run-card" aria-label="Next scheduled run">
          <Clock3 aria-hidden="true" size={20} />
          <span>Next ritual</span>
          <strong>{nextJob ? displayName(nextJob.name, nextJob.jobId) : "No scheduled run"}</strong>
          <small>{formatDate(nextJob?.nextRunAt ?? null)}</small>
        </div>
        <div className="automation-truth-card" aria-label="Automation truth rules">
          <TerminalSquare aria-hidden="true" size={20} />
          <span>Truth contract</span>
          <strong>Hermes owns state</strong>
          <small>{automationList?.hermesCommand ?? "Command unavailable until bridge loads"}</small>
        </div>
      </section>

      <div className="automation-summary-grid" aria-label="Automation summary">
        <SummaryCard label="Total jobs" value={summary?.totalJobs ?? "—"} tone="ink" />
        <SummaryCard label="Enabled" value={summary?.enabledJobs ?? "—"} />
        <SummaryCard label="Paused" value={summary?.pausedJobs ?? "—"} />
        <SummaryCard label="Failed" value={summary?.failedJobs ?? "—"} tone={failedJobs.length > 0 ? "seal" : "neutral"} />
        <SummaryCard label="Script-only" value={automationList ? scriptJobs.length : "—"} />
      </div>

      <div className="automation-toolbar">
        <div className="automation-filter-tabs" role="group" aria-label="Cron job filters">
          {filters.map((item) => (
            <button
              aria-pressed={filter === item.value}
              className={filter === item.value ? "active" : ""}
              key={item.value}
              onClick={() => setFilter(item.value)}
              type="button"
            >
              {item.label}
            </button>
          ))}
        </div>
        <input
          aria-label="Search automations"
          className="automation-search-input"
          onChange={(event) => setSearchQuery(event.target.value)}
          placeholder="Search name, id, schedule, script…"
          type="search"
          value={searchQuery}
        />
      </div>

      <div className="automation-section-labels" aria-label="Visible automation sections">
        <span>Cron Jobs</span>
        <span>Watchers</span>
      </div>

      <div className="automation-workbench-grid">
        <section className="automation-section automation-section--jobs" aria-label="Cron Jobs">
          <div className="automation-section-heading">
            <div>
              <span className="automation-panel-kicker">Schedules</span>
              <h3>Cron Jobs</h3>
            </div>
            <p>{automationList ? `${visibleJobs.length} of ${automationList.jobs.length} shown` : "Loading Hermes cron jobs"}</p>
          </div>

          {isLoading && !automationList ? <p className="repo-empty-state">Loading Hermes cron jobs…</p> : null}
          {!isLoading && automationList && automationList.jobs.length === 0 ? <p className="repo-empty-state">No Hermes cron jobs found.</p> : null}
          {!isLoading && automationList && automationList.jobs.length > 0 && visibleJobs.length === 0 ? <p className="repo-empty-state">No cron jobs match this filter.</p> : null}

          <div className="automation-job-list" role="list">
            {visibleJobs.map((job) => {
              const kind = getAutomationStatusKind(job);
              const isThisJobBusy = busyJobId === job.jobId;
              const canPause = job.enabled && kind !== "paused";
              const canResume = !job.enabled || kind === "paused";
              const jobLabel = displayName(job.name, job.jobId);
              return (
                <article className={`automation-job-card automation-job-card--${kind}`} key={job.jobId} role="listitem">
                  <div className="automation-job-card-header">
                    <div>
                      <p className="automation-job-id">{job.jobId}</p>
                      <h4 title={jobLabel}>{jobLabel}</h4>
                    </div>
                    <span className={`automation-status-badge automation-status-badge--${kind}`}>{statusLabel(job)}</span>
                  </div>

                  {job.protected ? (
                    <div className="automation-protection-note">
                      <Shield aria-hidden="true" size={15} />
                      <span>{job.protectionReason ?? "Protected internal automation"}</span>
                    </div>
                  ) : null}

                  <div className="automation-detail-grid">
                    <Detail label="Schedule" value={job.schedule} />
                    <Detail label="Repeat" value={job.repeat} />
                    <Detail label="Next run" value={formatDate(job.nextRunAt)} />
                    <Detail label="Last run" value={formatDate(job.lastRunAt)} />
                    <Detail label="Deliver" value={job.deliver} />
                    <Detail label="Script" value={job.script ?? (job.noAgent ? "script-only" : null)} />
                    <Detail label="Skills" value={job.skills.join(", ")} />
                    <Detail label="Toolsets" value={job.enabledToolsets.join(", ")} />
                  </div>

                  {job.lastDeliveryError ? <p className="automation-error-line">{job.lastDeliveryError}</p> : null}
                  {job.promptPreview ? <p className="automation-prompt-preview">Prompt preview: {job.promptPreview}</p> : null}

                  <div className="automation-action-row">
                    <button disabled={isActionInFlight || !canPause} onClick={() => runAction(job, "pause")} type="button">{isThisJobBusy && busyAction === "pause" ? "Pausing" : "Pause"}</button>
                    <button disabled={isActionInFlight || !canResume} onClick={() => runAction(job, "resume")} type="button">{isThisJobBusy && busyAction === "resume" ? "Resuming" : "Resume"}</button>
                    <button disabled={isActionInFlight} onClick={() => openPendingAction(job, "run")} type="button"><Play aria-hidden="true" size={14} /> Run now</button>
                    <button className="automation-danger-button" disabled={isActionInFlight || job.protected} title={job.protectionReason ?? undefined} onClick={() => openPendingAction(job, "remove")} type="button"><Trash2 aria-hidden="true" size={14} /> Remove</button>
                  </div>
                </article>
              );
            })}
          </div>
        </section>

        <aside className="automation-side-ledger" aria-label="Automation operational edge cases">
          <section className="automation-section" aria-label="Watchers">
            <div className="automation-section-heading">
              <div>
                <span className="automation-panel-kicker">Read-only</span>
                <h3>Watchers</h3>
              </div>
              <p>{automationList?.watcherSourceStatus === "unavailable" ? "Watcher source unavailable" : `${automationList?.watchers.length ?? 0} watcher(s)`}</p>
            </div>
            {automationList?.watchers.length ? (
              <div className="automation-job-list automation-watcher-list" role="list">
                {automationList.watchers.map((watcher) => (
                  <article className="automation-job-card automation-watcher-card" key={watcher.id} role="listitem">
                    <div className="automation-job-card-header">
                      <div><p className="automation-job-id">{watcher.id}</p><h4 title={displayName(watcher.name, watcher.id)}>{displayName(watcher.name, watcher.id)}</h4></div>
                      <span className={`automation-status-badge automation-status-badge--${watcher.state === "failed" ? "error" : watcher.state === "paused" ? "paused" : watcher.state === "running" ? "ok" : "unknown"}`}>{watcher.state}</span>
                    </div>
                    <div className="automation-detail-grid automation-detail-grid--watcher">
                      <Detail label="Source" value={watcher.source} />
                      <Detail label="Last seen" value={formatDate(watcher.lastSeenAt)} />
                      <Detail label="Last status" value={watcher.lastStatus} />
                      <Detail label="Detail" value={watcher.detail} />
                    </div>
                    <p className="automation-prompt-preview">Watchers are read-only in V1.</p>
                  </article>
                ))}
              </div>
            ) : (
              <p className="repo-empty-state">No watchers found. {automationList?.watcherSourceStatus === "unavailable" ? "Hermes does not expose inspectable watcher state yet." : ""}</p>
            )}
          </section>

          <section className="automation-edge-panel" aria-label="Automation safety notes">
            <span className="automation-panel-kicker">Blind spots patched</span>
            <ul>
              <li>Protected jobs cannot be removed from the UI.</li>
              <li>Run now warns about external side effects before execution.</li>
              <li>Remove only completes after Hermes read-back no longer contains the job.</li>
              <li>Watcher controls stay read-only until Hermes exposes a writable source.</li>
            </ul>
          </section>
        </aside>
      </div>

      <footer className="automation-footer-note">
        <span>Last refreshed: {formatDate(automationList?.refreshedAt ?? null)}</span>
        <span>Hermes command: {automationList?.hermesCommand ?? "—"}</span>
      </footer>

      {pendingAction ? (
        <div className="automation-confirm-backdrop" role="presentation">
          <div className="automation-confirm-panel" ref={confirmPanelRef} role="dialog" aria-modal="true" aria-labelledby="automation-confirm-title" aria-describedby="automation-confirm-description" onKeyDown={handleConfirmKeyDown} tabIndex={-1}>
            {(() => {
              const currentPendingAction = pendingAction;
              const currentPendingJob = automationList?.jobs.find((job) => job.jobId === currentPendingAction.job.jobId) ?? currentPendingAction.job;
              const pendingJobMissing = automationList !== null && !automationList.jobs.some((job) => job.jobId === currentPendingAction.job.jobId);
              const pendingJobProtected = currentPendingAction.action === "remove" && currentPendingJob.protected;
              const pendingJobLabel = displayName(currentPendingJob.name, currentPendingJob.jobId);
              const isConfirmBusy = isActionInFlight && busyJobId === currentPendingJob.jobId && busyAction === currentPendingAction.action;
              const confirmLabel = currentPendingAction.action === "remove"
                ? (isConfirmBusy ? "Removing" : "Remove")
                : (isConfirmBusy ? "Running" : "Run now");
              const pendingGuardMessage = pendingJobMissing
                ? "Hermes read-back no longer includes this cron job. Refresh before acting."
                : pendingJobProtected
                  ? currentPendingJob.protectionReason ?? "Hermes read-back now marks this cron job as protected."
                  : null;

              function confirmPendingAction() {
                if (pendingJobMissing) {
                  setPendingActionError("Hermes read-back no longer includes this cron job. Refresh before acting.");
                  return;
                }
                if (pendingJobProtected) {
                  setPendingActionError(currentPendingJob.protectionReason ?? "Hermes read-back now marks this cron job as protected.");
                  return;
                }
                void runAction(currentPendingJob, currentPendingAction.action);
              }

              return (
                <>
                  <p className="kana-line">確認</p>
                  <h3 id="automation-confirm-title">{currentPendingAction.action === "remove" ? "Remove cron job?" : "Run cron job now?"}</h3>
                  <p id="automation-confirm-description">
                    {currentPendingAction.action === "remove"
                      ? `This will remove “${pendingJobLabel}” from Hermes after provider read-back verifies it is gone.`
                      : `This will run “${pendingJobLabel}” now and may trigger external side effects.`}
                  </p>
                  {pendingActionError || pendingGuardMessage ? <p className="automation-modal-error" role="alert">{pendingActionError ?? pendingGuardMessage}</p> : null}
                  <div className="automation-confirm-actions">
                    <button disabled={isActionInFlight} onClick={cancelPendingAction} type="button">Cancel</button>
                    <button className={currentPendingAction.action === "remove" ? "automation-danger-button" : "automation-primary-button"} disabled={isActionInFlight} onClick={confirmPendingAction} type="button">
                      {confirmLabel}
                    </button>
                  </div>
                </>
              );
            })()}
          </div>
        </div>
      ) : null}
    </section>
  );
}
