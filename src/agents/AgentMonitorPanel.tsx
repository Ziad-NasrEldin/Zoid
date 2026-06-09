import { useState, type KeyboardEvent } from "react";
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
  queueOnly?: boolean;
  onFocus: (sessionId: string) => void;
  onSend: (sessionId: string, prompt: string) => void | Promise<void>;
  onStop: (sessionId: string) => void | Promise<void>;
  onContinue: (sessionId: string) => void | Promise<void>;
  onExpand: (sessionId: string) => void;
  onRemoveFromDashboard: (sessionId: string) => void;
  onMakePrimary: (sessionId: string) => void;
  onMove: (sessionId: string, direction: "left" | "right") => void;
};

export function AgentMonitorPanel({ session, runtimeState, repository, isPrimary, isFocused, disabled, disabledReason, queueOnly, onFocus, onSend, onStop, onContinue, onExpand, onRemoveFromDashboard, onMakePrimary, onMove }: Props) {
  const [draft, setDraft] = useState("");
  const latestMessages = session.messages.filter((message) => message.content.trim()).slice(-4);
  const isRunning = runtimeState.status === "running" || runtimeState.status === "needs-input";
  const queuedCount = runtimeState.queuedPrompts.length;
  async function submit() {
    const prompt = draft.trim();
    if (!prompt || disabled) return;
    await onSend(session.id, prompt);
    setDraft("");
  }
  function handleKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      void submit();
    }
  }
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
        <div>
          <span className="agent-monitor-kicker">{isPrimary ? "Primary" : "Tiled agent"}</span>
          <h3>{session.title}</h3>
          <p>{repository ? `${repository.name} · ${repository.branch || "unknown"}` : "Unlinked repository"}</p>
        </div>
        <div className="agent-monitor-actions">
          <button className="agent-monitor-action agent-monitor-action--icon" type="button" onClick={(event) => { event.stopPropagation(); onMove(session.id, "left"); }} aria-label={`Move ${session.title} left`}>←</button>
          <button className="agent-monitor-action agent-monitor-action--icon" type="button" onClick={(event) => { event.stopPropagation(); onMove(session.id, "right"); }} aria-label={`Move ${session.title} right`}>→</button>
          {!isPrimary ? <button className="agent-monitor-action agent-monitor-action--chip" type="button" onClick={(event) => { event.stopPropagation(); onMakePrimary(session.id); }}>Primary</button> : <span className="agent-monitor-primary-chip">Primary</span>}
          <button className="agent-monitor-action agent-monitor-action--compact" type="button" onClick={(event) => { event.stopPropagation(); onExpand(session.id); }}>Expand</button>
          <button className="agent-monitor-action agent-monitor-action--quiet" type="button" onClick={(event) => { event.stopPropagation(); onRemoveFromDashboard(session.id); }}>Untile</button>
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
      <footer className="agent-monitor-composer" onClick={(event) => event.stopPropagation()}>
        <textarea value={draft} onChange={(event) => setDraft(event.target.value)} onInput={(event) => setDraft(event.currentTarget.value)} onKeyDown={handleKeyDown} placeholder={isRunning || queueOnly ? "Queue a follow-up for this session…" : "Prompt this agent…"} aria-label={`Prompt ${session.title}`} />
        <div>
          <button type="button" disabled={disabled || draft.trim().length === 0} title={disabledReason} onClick={() => void submit()}>{isRunning || queueOnly ? "Queue" : "Send"}</button>
          <button type="button" disabled={!isRunning} onClick={() => void onStop(session.id)}>Stop</button>
          <button className="agent-monitor-continue-button" type="button" disabled={isRunning || disabled} title={disabledReason} onClick={() => void onContinue(session.id)}>Continue</button>
        </div>
      </footer>
    </article>
  );
}
