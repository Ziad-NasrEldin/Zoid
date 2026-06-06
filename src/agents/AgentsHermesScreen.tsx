import { useEffect, useMemo, useState } from "react";
import { ChatComposer } from "./ChatComposer";
import { MessageBubble } from "./MessageBubble";
import { getHermesCliStatus, sendHermesCliMessage } from "./hermesClient";
import { hermesParticipant, participantsById, userParticipant } from "./participants";
import type { AgentConnectionState, ChatMessage, HermesCliStatus } from "./types";

const initialMessages: ChatMessage[] = [
  {
    id: "hermes-welcome",
    role: "assistant",
    participantId: "hermes",
    content: "Hermes is linked through the local terminal CLI. Messages run through your configured Hermes/Codex setup.",
    createdAt: new Date().toISOString(),
    status: "sent",
  },
];

function connectionCopy(status: HermesCliStatus | null, state: AgentConnectionState) {
  if (state === "checking") return "Checking local Hermes CLI...";
  if (!status) return "Hermes CLI status is unknown.";
  return status.message;
}

function statusTone(state: AgentConnectionState) {
  if (state === "online") return "ready";
  if (state === "checking") return "idle";
  return "blocked";
}

export function AgentsHermesScreen() {
  const [messages, setMessages] = useState<ChatMessage[]>(initialMessages);
  const [cliStatus, setCliStatus] = useState<HermesCliStatus | null>(null);
  const [connectionState, setConnectionState] = useState<AgentConnectionState>("checking");
  const [isSending, setIsSending] = useState(false);

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

  const hermesWithPresence = useMemo(
    () => ({
      ...hermesParticipant,
      presence: isSending ? "thinking" as const : connectionState === "online" ? "online" as const : "offline" as const,
    }),
    [connectionState, isSending],
  );

  const disabledReason =
    connectionState === "online"
      ? undefined
      : "Hermes CLI is not reachable. Install Hermes or set ZOID_HERMES_CLI to the Hermes executable path.";

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
      const response = await sendHermesCliMessage(
        nextMessages.map((message) => ({ role: message.role, content: message.content })),
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
        <div className="connection-panel" aria-live="polite">
          <span className={`status-dot ${statusTone(connectionState)}`} aria-hidden="true" />
          <span>Hermes CLI {connectionState.toUpperCase()}</span>
        </div>
      </header>

      <div className="chat-stage">
        <aside className="agent-profile-card" aria-label="Active agent profile">
          <span className="profile-label">TERMINAL SESSION</span>
          <div className="profile-row">
            <span className="profile-avatar-wrap">
              <MessageProfile participant={hermesWithPresence} />
            </span>
            <div>
              <strong>{hermesWithPresence.displayName}</strong>
              <span>{hermesWithPresence.handle}</span>
            </div>
          </div>
          <p>{connectionCopy(cliStatus, connectionState)}</p>
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
        <span>Bridge: Hermes CLI</span>
        <span>Operator: {userParticipant.displayName}</span>
        <span>Session: {cliStatus?.session ?? "most-recent-hermes-cli-session"}</span>
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
