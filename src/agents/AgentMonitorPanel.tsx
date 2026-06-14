import type { KeyboardEvent } from "react";
import { ChatComposer } from "./ChatComposer";
import type { HermesSlashCommand } from "./hermesCommands";
import type { HermesChatSession } from "./sessionState";
import type { AgentSessionRuntimeState } from "./useAgentRuntime";
import type { CodeRepository } from "../code/types";

type Props = {
  session: HermesChatSession;
  runtimeState: AgentSessionRuntimeState;
  repository?: CodeRepository;
  isPrimary: boolean;
  isFocused: boolean;
  disabled?: boolean;
  disabledReason?: string;
  contextUsedPercent?: number;
  modelLabel?: string;
  queueOnly?: boolean;
  slashCommands?: HermesSlashCommand[];
  slashCommandSource?: "live" | "fallback" | "unavailable";
  onFocus: (sessionId: string) => void;
  onSend: (sessionId: string, prompt: string) => void | Promise<void>;
  onStop: (sessionId: string) => void | Promise<void>;
  onContinue: (sessionId: string) => void | Promise<void>;
  onExpand: (sessionId: string) => void;
  onRemoveFromDashboard: (sessionId: string) => void;
  onMakePrimary: (sessionId: string) => void;
  onMove: (sessionId: string, direction: "left" | "right") => void;
};

export function AgentMonitorPanel({ session, runtimeState, repository, isPrimary, isFocused, disabled, disabledReason, contextUsedPercent = 1, modelLabel = "gpt-5.5", queueOnly, slashCommands = [], slashCommandSource = "live", onFocus, onSend, onStop, onContinue, onExpand, onRemoveFromDashboard, onMakePrimary, onMove }: Props) {
  const latestMessages = session.messages.filter((message) => message.content.trim()).slice(-4);
  const isRunning = runtimeState.status === "running" || runtimeState.status === "needs-input";
  const queuedCount = runtimeState.queuedPrompts.length;
  function handlePanelKeyDown(event: KeyboardEvent<HTMLElement>) {
    if (event.target !== event.currentTarget) return;
    if (event.key === "Enter" || event.key === " ") {
      event.preventDefault();
      onFocus(session.id);
    }
  }
  return (
    <article className={`agent-monitor-panel agent-monitor-panel--${runtimeState.status}${isPrimary ? " agent-monitor-panel--primary" : ""}${isFocused ? " agent-monitor-panel--focused" : ""}`} onClick={() => onFocus(session.id)} onKeyDown={handlePanelKeyDown} role="group" tabIndex={0} aria-label={`Agent panel ${session.title}`}>
      <header className="agent-monitor-panel-header">
        <div className="agent-monitor-title-block">
          <span className="agent-monitor-kicker">{isPrimary ? "Primary" : "Tiled agent"}</span>
          <div className="agent-monitor-title-row">
            <h3>{session.title}</h3>
            <div className="agent-monitor-actions">
              <button className="agent-monitor-action agent-monitor-action--icon" type="button" onClick={(event) => { event.stopPropagation(); onMove(session.id, "left"); }} aria-label={`Move ${session.title} left`}>←</button>
              <button className="agent-monitor-action agent-monitor-action--icon" type="button" onClick={(event) => { event.stopPropagation(); onMove(session.id, "right"); }} aria-label={`Move ${session.title} right`}>→</button>
              {!isPrimary ? <button className="agent-monitor-action agent-monitor-action--chip" type="button" onClick={(event) => { event.stopPropagation(); onMakePrimary(session.id); }}>Primary</button> : <span className="agent-monitor-primary-chip">Primary</span>}
              <button className="agent-monitor-action agent-monitor-action--compact" type="button" onClick={(event) => { event.stopPropagation(); onExpand(session.id); }}>Expand</button>
              <button className="agent-monitor-action agent-monitor-action--quiet" type="button" onClick={(event) => { event.stopPropagation(); onRemoveFromDashboard(session.id); }}>Untile</button>
            </div>
          </div>
          <p>{repository ? `${repository.name} · ${repository.branch || "unknown"}` : "Unlinked repository"}</p>
        </div>
      </header>
      <div className="agent-monitor-status-strip" aria-live="polite">
        <strong>{runtimeState.status}</strong>
        {queuedCount ? <span>{queuedCount} queued</span> : <span>No queue</span>}
        {runtimeState.lastError ? <span>{runtimeState.lastError}</span> : null}
      </div>
      <div className="agent-monitor-feed" role="log" aria-label={`${session.title} recent messages`}>
        {latestMessages.map((message) => <p key={message.id} className={`agent-monitor-feed-line agent-monitor-feed-line--${message.role}`}><b>{message.role}</b> {message.content}</p>)}
      </div>
      <footer className="agent-monitor-composer">
        <ChatComposer
          ariaLabel={`Hermes message composer for ${session.title}`}
          canStop={isRunning}
          contextUsedPercent={contextUsedPercent}
          disabled={disabled}
          disabledReason={disabledReason}
          inputLabel={`Message ${session.title}`}
          isSending={isRunning || Boolean(queueOnly)}
          modelLabel={modelLabel}
          placeholder={isRunning || queueOnly ? "Queue follow-up…" : "Message… / commands"}
          slashCommands={slashCommands}
          slashCommandSource={slashCommandSource}
          variant="panel"
          onSend={(prompt) => onSend(session.id, prompt)}
          onStop={() => onStop(session.id)}
        />
        <div className="agent-monitor-composer-actions">
          <button className="agent-monitor-continue-button" type="button" disabled={isRunning || disabled} title={disabledReason} onClick={() => void onContinue(session.id)}>Continue</button>
        </div>
      </footer>
    </article>
  );
}
