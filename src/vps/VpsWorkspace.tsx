import { AlertTriangle, CheckCircle2, Play, RefreshCw, RotateCcw, Server, Square } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { getHostingerVpsOverview, refreshHostingerVps, runHostingerVpsAction } from "./vpsClient";
import type { HostingerVirtualMachine, HostingerVpsActionLog, HostingerVpsOverview } from "./types";

type LoadState = "idle" | "loading" | "ready" | "error";
type ServerAction = "start" | "stop" | "restart";

function bridgeErrorMessage(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  if (message.includes("invoke") || message.includes("Cannot read")) {
    return "Zoid desktop bridge is unavailable in this preview. Open the native app to manage Hostinger VPS state.";
  }
  return message;
}

function readableDateTime(value: string | null | undefined): string {
  if (!value) return "Never";
  const numeric = Number(value);
  const parsed = Number.isFinite(numeric) ? new Date(numeric) : new Date(value);
  if (Number.isNaN(parsed.getTime())) return value;
  return parsed.toLocaleString(undefined, { dateStyle: "medium", timeStyle: "short" });
}

function stateTone(state: string): "ready" | "busy" | "blocked" | "idle" {
  if (["running", "success", "unlocked"].includes(state)) return "ready";
  if (["starting", "stopping", "restarting", "creating", "recreating", "restoring", "sent", "created", "delayed"].includes(state)) return "busy";
  if (["error", "destroyed", "destroying", "suspended"].includes(state)) return "blocked";
  return "idle";
}

function actionDisabled(server: HostingerVirtualMachine, action: ServerAction, busyAction: string | null): boolean {
  if (busyAction !== null || server.actionsLock === "locked") return true;
  if (action === "start") return server.state === "running" || server.state === "starting";
  if (action === "stop") return server.state === "stopped" || server.state === "stopping";
  return server.state === "stopped" || server.state === "stopping";
}

function ServerStats({ server }: { server: HostingerVirtualMachine }) {
  const stats = [
    { label: "Plan", value: server.plan ?? "—" },
    { label: "CPU", value: server.cpus ? `${server.cpus} vCPU` : "—" },
    { label: "Memory", value: server.memoryMb ? `${server.memoryMb} MB` : "—" },
    { label: "Disk", value: server.diskGb ? `${server.diskGb} GB` : "—" },
  ];
  return (
    <dl className="vps-server-stats">
      {stats.map((stat) => (
        <div key={stat.label}>
          <dt>{stat.label}</dt>
          <dd>{stat.value}</dd>
        </div>
      ))}
    </dl>
  );
}

function ServerCard({ busyAction, onAction, server }: { busyAction: string | null; onAction: (server: HostingerVirtualMachine, action: ServerAction) => void; server: HostingerVirtualMachine }) {
  const tone = stateTone(server.state);
  return (
    <article className="vps-server-card">
      <header>
        <span className={`vps-state-pill vps-state-pill--${tone}`}>{server.state}</span>
        <h3>{server.hostname}</h3>
        <p>{server.primaryIp ?? "No public IP in API response"}</p>
      </header>
      <ServerStats server={server} />
      <footer>
        <span>{server.location ?? "Hostinger VPS"}</span>
        <div className="vps-action-row" aria-label={`Actions for ${server.hostname}`}>
          <button disabled={actionDisabled(server, "start", busyAction)} onClick={() => onAction(server, "start")} type="button"><Play size={14} /> Start</button>
          <button disabled={actionDisabled(server, "stop", busyAction)} onClick={() => onAction(server, "stop")} type="button"><Square size={14} /> Stop</button>
          <button disabled={actionDisabled(server, "restart", busyAction)} onClick={() => onAction(server, "restart")} type="button"><RotateCcw size={14} /> Reboot</button>
        </div>
      </footer>
    </article>
  );
}

function ActionRow({ action }: { action: HostingerVpsActionLog }) {
  return (
    <li className="vps-action-log-row">
      <span className={`vps-state-pill vps-state-pill--${stateTone(action.state)}`}>{action.state}</span>
      <strong>{action.action}</strong>
      <span>{action.virtualMachineId}</span>
      <time>{readableDateTime(action.createdAt)}</time>
    </li>
  );
}

export function VpsWorkspace() {
  const [loadState, setLoadState] = useState<LoadState>("idle");
  const [overview, setOverview] = useState<HostingerVpsOverview | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [busyAction, setBusyAction] = useState<string | null>(null);

  async function loadCached() {
    setLoadState("loading");
    setError(null);
    try {
      setOverview(await getHostingerVpsOverview());
      setLoadState("ready");
    } catch (err) {
      setError(bridgeErrorMessage(err));
      setLoadState("error");
    }
  }

  async function refresh() {
    setLoadState("loading");
    setError(null);
    setMessage(null);
    try {
      setOverview(await refreshHostingerVps());
      setLoadState("ready");
      setMessage("Hostinger VPS state refreshed.");
    } catch (err) {
      setError(bridgeErrorMessage(err));
      setLoadState("error");
    }
  }

  async function handleAction(server: HostingerVirtualMachine, action: ServerAction) {
    if ((action === "stop" || action === "restart") && !window.confirm(`${action === "stop" ? "Stop" : "Reboot"} ${server.hostname}?`)) {
      return;
    }
    const actionKey = `${server.id}:${action}`;
    setBusyAction(actionKey);
    setError(null);
    setMessage(null);
    try {
      const result = await runHostingerVpsAction(server.id, action);
      setOverview(result.overview);
      setMessage(result.message);
      setLoadState("ready");
    } catch (err) {
      setError(bridgeErrorMessage(err));
    } finally {
      setBusyAction(null);
    }
  }

  useEffect(() => { void loadCached(); }, []);

  const runningCount = useMemo(() => overview?.servers.filter((server) => server.state === "running").length ?? 0, [overview]);

  return (
    <section className="vps-workspace" aria-label="Hostinger VPS workspace">
      <header className="vps-hero">
        <p className="kana-line">仮想サーバー</p>
        <div>
          <h2>Hostinger VPS</h2>
          <p>Local-only VPS mirror. Reads Hostinger over an environment token, caches state in the Hermes profile, and exposes only start, stop, and reboot controls.</p>
        </div>
        <button className="vps-primary-action" disabled={loadState === "loading" || busyAction !== null} onClick={() => void refresh()} type="button">
          <RefreshCw size={16} /> Refresh API
        </button>
      </header>

      <div className="vps-status-strip" role="status">
        <span><Server size={15} /> {overview?.servers.length ?? 0} VPS</span>
        <span><CheckCircle2 size={15} /> {runningCount} running</span>
        <span className={overview?.tokenPresent ? "is-ready" : "is-blocked"}>{overview?.tokenPresent ? "HOSTINGER_API_TOKEN present" : "HOSTINGER_API_TOKEN missing"}</span>
        <span>Last sync: {readableDateTime(overview?.lastSyncedAt)}</span>
      </div>

      {message ? <p className="vps-message">{message}</p> : null}
      {error || overview?.lastError ? (
        <div className="vps-error" role="alert">
          <AlertTriangle size={18} />
          <p>{error ?? overview?.lastError}</p>
        </div>
      ) : null}

      <div className="vps-grid">
        <section className="vps-panel vps-panel--servers">
          <div className="vps-panel-heading">
            <h3>Servers</h3>
            <span>{loadState === "loading" ? "Syncing" : "Cached mirror"}</span>
          </div>
          {overview?.servers.length ? (
            <div className="vps-server-list">
              {overview.servers.map((server) => <ServerCard busyAction={busyAction} key={server.id} onAction={handleAction} server={server} />)}
            </div>
          ) : (
            <p className="vps-empty-state">No VPS records cached yet. Add HOSTINGER_API_TOKEN to the native app environment, then refresh.</p>
          )}
        </section>

        <aside className="vps-panel vps-panel--activity">
          <div className="vps-panel-heading">
            <h3>Action log</h3>
            <span>Non-destructive controls only</span>
          </div>
          {overview?.actions.length ? (
            <ol className="vps-action-log">
              {overview.actions.slice(0, 8).map((action) => <ActionRow action={action} key={action.id} />)}
            </ol>
          ) : (
            <p className="vps-empty-state">No start, stop, or reboot actions have been sent from Zoid.</p>
          )}
          <p className="vps-cache-note">Cache: {overview?.cachePath ?? "—"}</p>
        </aside>
      </div>
    </section>
  );
}
