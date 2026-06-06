import { useEffect, useMemo, useState } from "react";
import { ChatComposer } from "./ChatComposer";
import { MessageBubble } from "./MessageBubble";
import { getHermesCliStatus, sendHermesCliMessage } from "./hermesClient";
import { participantsById } from "./participants";
import type { AgentConnectionState, ChatMessage, HermesCliStatus } from "./types";

const initialChatMessages: ChatMessage[] = [
  {
    id: "hermes-welcome",
    role: "assistant",
    participantId: "hermes",
    content: "Hermes is linked through the local terminal CLI. Prompts run through your configured Hermes/Codex setup.",
    createdAt: new Date().toISOString(),
    status: "sent",
  },
];

function statusTone(state: AgentConnectionState) {
  if (state === "online") return "ready";
  if (state === "checking") return "idle";
  return "blocked";
}

const HERMES_CONTEXT_LIMIT = 200_000;
const CODEX_USAGE_TODAY = "5h";
const CODEX_USAGE_WEEKLY = "5h / week";
const ACTIVE_MODEL = "gpt-5.5";

function estimateContextUsed(messages: ChatMessage[]) {
  const approximateTokens = messages.reduce((total, message) => total + Math.ceil(message.content.length / 4), 0);
  return Math.min(100, Math.max(1, Math.round((approximateTokens / HERMES_CONTEXT_LIMIT) * 100)));
}

function formatElapsed(milliseconds: number | null) {
  if (milliseconds === null) return "idle";
  if (milliseconds < 1_000) return `${milliseconds}ms`;
  const seconds = Math.round(milliseconds / 1_000);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes}m ${remainingSeconds}s`;
}

export function AgentsHermesScreen() {
  const [messages, setMessages] = useState<ChatMessage[]>(initialChatMessages);
  const [cliStatus, setCliStatus] = useState<HermesCliStatus | null>(null);
  const [connectionState, setConnectionState] = useState<AgentConnectionState>("checking");
  const [isSending, setIsSending] = useState(false);
  const [linkedRepository, setLinkedRepository] = useState("Unlinked");
  const [repositoryDraft, setRepositoryDraft] = useState("/Users/ziadnasreldin/Zoid");
  const [lastPromptStartedAt, setLastPromptStartedAt] = useState<number | null>(null);
  const [lastPromptElapsedMs, setLastPromptElapsedMs] = useState<number | null>(null);
  const [elapsedTick, setElapsedTick] = useState(Date.now());

  useEffect(() => {
    let active = true;
    getHermesCliStatus().then((result) => {
      if (!active) return;
      setCliStatus(result);
      setConnectionState(result.status);
    });
    return () => {
      active = false;
    };
  }, []);

  useEffect(() => {
    if (!isSending || lastPromptStartedAt === null) return undefined;
    const interval = window.setInterval(() => setElapsedTick(Date.now()), 500);
    return () => window.clearInterval(interval);
  }, [isSending, lastPromptStartedAt]);

  const hermesWithPresence = useMemo(() => {
    const base = participantsById.hermes;
    return {
      ...base,
      presence: isSending ? "thinking" as const : connectionState === "online" ? "online" as const : "offline" as const,
    };
  }, [connectionState, isSending]);

  const disabledReason =
    connectionState === "online"
      ? undefined
      : "Hermes CLI is not reachable. Install Hermes or set ZOID_HERMES_CLI to the Hermes executable path.";

  const contextUsedPercent = estimateContextUsed(messages);
  const compressionCount = 0;
  const promptElapsed = isSending && lastPromptStartedAt !== null ? elapsedTick - lastPromptStartedAt : lastPromptElapsedMs;

  function handleLinkRepository() {
    const nextRepository = repositoryDraft.trim();
    setLinkedRepository(nextRepository || "Unlinked");
  }

  async function handleSend(content: string) {
    const userMessage: ChatMessage = {
      id: `user-${crypto.randomUUID()}`,
      role: "user",
      participantId: "ziad",
      content,
      createdAt: new Date().toISOString(),
      status: "sent",
    };
    const assistantId = `hermes-${crypto.randomUUID()}`;
    const assistantMessage: ChatMessage = {
      id: assistantId,
      role: "assistant",
      participantId: "hermes",
      content: "",
      createdAt: new Date().toISOString(),
      status: "streaming",
    };

    const nextMessages = [...messages, userMessage];
    setMessages([...nextMessages, assistantMessage]);
    const promptStartedAt = Date.now();
    setLastPromptStartedAt(promptStartedAt);
    setLastPromptElapsedMs(null);
    setElapsedTick(promptStartedAt);
    setIsSending(true);

    try {
      const response = await sendHermesCliMessage(
        nextMessages.map((message) => ({ role: message.role, content: message.content })),
        linkedRepository === "Unlinked" ? undefined : linkedRepository,
      );
      setMessages((current) =>
        current.map((message) =>
          message.id === assistantId
            ? { ...message, content: response.content || "Hermes CLI returned an empty response.", status: "sent" }
            : message,
        ),
      );
      setCliStatus((current) => current ? { ...current, session: response.session } : current);
    } catch (error) {
      setMessages((current) =>
        current.map((message) =>
          message.id === assistantId
            ? {
                ...message,
                content: message.content || "Hermes terminal response failed.",
                status: "error",
                error: error instanceof Error ? error.message : String(error),
              }
            : message,
        ),
      );
    } finally {
      setLastPromptElapsedMs(Date.now() - promptStartedAt);
      setLastPromptStartedAt(null);
      setIsSending(false);
    }
  }

  return (
    <section aria-label="Hermes chat" className="hermes-chat-shell">
      <header className="hermes-topbar">
        <div>
          <p className="section-kicker">AGENTS / HERMES TERMINAL</p>
          <h2>Hermes Agent</h2>
        </div>
        <div className="topbar-status-stack">
          <div className="connection-panel" aria-live="polite">
            <span className={`status-dot ${statusTone(connectionState)}`} aria-hidden="true" />
            <span>Hermes CLI {connectionState.toUpperCase()}</span>
          </div>
          <div className="repository-link-control repository-link-control--topbar">
            <label htmlFor="linked-repository-input">Link repository</label>
            <input
              id="linked-repository-input"
              onChange={(event) => setRepositoryDraft(event.target.value)}
              value={repositoryDraft}
            />
            <button type="button" onClick={handleLinkRepository}>Link repository</button>
            <small>Repository: {linkedRepository}</small>
          </div>
        </div>
      </header>

      <div className="chat-stage">
        <div className="message-list" role="log" aria-live="polite" aria-label="Hermes conversation messages">
          {messages.map((message) => (
            <MessageBubble
              key={message.id}
              message={message}
              participant={message.participantId === "hermes" ? hermesWithPresence : participantsById[message.participantId]}
            />
          ))}
        </div>
      </div>

      <ChatComposer disabled={connectionState !== "online"} disabledReason={disabledReason} isSending={isSending} onSend={handleSend} />

      <footer className="chat-stats-strip" aria-label="Hermes session stats">
        <span>Context used: {contextUsedPercent}% · Compressions: {compressionCount}</span>
        <span>Repository: {linkedRepository}</span>
        <span>Codex usage: {CODEX_USAGE_TODAY} today / {CODEX_USAGE_WEEKLY} · Model: {ACTIVE_MODEL}</span>
        <span>Session: {cliStatus?.session ?? "most-recent-hermes-cli-session"} · Elapsed: {formatElapsed(promptElapsed)}</span>
      </footer>
    </section>
  );
}
