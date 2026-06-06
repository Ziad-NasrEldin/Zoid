import { useEffect, useMemo, useState } from "react";
import { ChatComposer } from "./ChatComposer";
import { MessageBubble } from "./MessageBubble";
import { getHermesHealth, sendHermesMessage } from "./hermesClient";
import { hermesParticipant, participantsById, userParticipant } from "./participants";
import type { AgentConnectionState, ChatMessage, HermesHealth } from "./types";

const initialMessages: ChatMessage[] = [
  {
    id: "hermes-welcome",
    role: "assistant",
    participantId: "hermes",
    content: "Hermes is ready when the local API server is online. I will never fake a response.",
    createdAt: new Date().toISOString(),
    status: "sent",
  },
];

function connectionCopy(health: HermesHealth | null, state: AgentConnectionState) {
  if (state === "checking") return "Checking local Hermes API server...";
  if (!health) return "Hermes API server status is unknown.";
  return health.message;
}

function statusTone(state: AgentConnectionState) {
  if (state === "online") return "ready";
  if (state === "checking") return "idle";
  return "blocked";
}

export function AgentsHermesScreen() {
  const [messages, setMessages] = useState<ChatMessage[]>(initialMessages);
  const [health, setHealth] = useState<HermesHealth | null>(null);
  const [connectionState, setConnectionState] = useState<AgentConnectionState>("checking");
  const [isSending, setIsSending] = useState(false);

  useEffect(() => {
    let active = true;
    getHermesHealth().then((result) => {
      if (!active) return;
      setHealth(result);
      setConnectionState(result.status);
    });
    return () => {
      active = false;
    };
  }, []);

  const hermesWithPresence = useMemo(
    () => ({ ...hermesParticipant, presence: isSending ? "thinking" as const : connectionState === "online" ? "online" as const : "offline" as const }),
    [connectionState, isSending],
  );

  const disabledReason =
    connectionState === "online"
      ? undefined
      : "Hermes API server is offline. Enable API_SERVER_ENABLED, set API_SERVER_KEY/ZOID_HERMES_API_KEY, then restart hermes gateway.";

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
    setIsSending(true);

    try {
      const response = await sendHermesMessage(
        nextMessages.map((message) => ({ role: message.role, content: message.content })),
      );
      setMessages((current) =>
        current.map((message) =>
          message.id === assistantId
            ? { ...message, content: response.content || "Hermes returned an empty response.", status: "sent" }
            : message,
        ),
      );
      if (response.model) {
        setHealth((current) => current ? { ...current, model: response.model } : current);
      }
    } catch (error) {
      setMessages((current) =>
        current.map((message) =>
          message.id === assistantId
            ? {
                ...message,
                content: message.content || "Hermes response failed.",
                status: "error",
                error: error instanceof Error ? error.message : String(error),
              }
            : message,
        ),
      );
    } finally {
      setIsSending(false);
    }
  }

  return (
    <section aria-label="Hermes chat" className="hermes-chat-shell">
      <header className="hermes-topbar">
        <div>
          <p className="section-kicker">AGENTS / HERMES</p>
          <h2>Hermes Agent</h2>
        </div>
        <div className="connection-panel" aria-live="polite">
          <span className={`status-dot ${statusTone(connectionState)}`} aria-hidden="true" />
          <span>Hermes {connectionState.toUpperCase()}</span>
        </div>
      </header>

      <div className="chat-stage">
        <aside className="agent-profile-card" aria-label="Active agent profile">
          <span className="profile-label">ACTIVE SESSION</span>
          <div className="profile-row">
            <span className="profile-avatar-wrap">
              <MessageProfile participant={hermesWithPresence} />
            </span>
            <div>
              <strong>{hermesWithPresence.displayName}</strong>
              <span>{hermesWithPresence.handle}</span>
            </div>
          </div>
          <p>{connectionCopy(health, connectionState)}</p>
        </aside>

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
        <span>Messages: {messages.length}</span>
        <span>Model: {health?.model ?? "local gateway"}</span>
        <span>Operator: {userParticipant.displayName}</span>
        <span>Session: runtime-only</span>
      </footer>
    </section>
  );
}

function MessageProfile({ participant }: { participant: typeof hermesParticipant }) {
  return (
    <span aria-label={`${participant.displayName} avatar`} className="chat-avatar chat-avatar--lg chat-avatar--hermes">
      <span aria-hidden="true">{participant.initials}</span>
      <span aria-hidden="true" className={`avatar-presence avatar-presence--${participant.presence}`} />
    </span>
  );
}
