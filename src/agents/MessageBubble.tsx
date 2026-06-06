import { Avatar } from "./Avatar";
import type { ChatMessage, ChatParticipant } from "./types";

type MessageBubbleProps = {
  message: ChatMessage;
  participant: ChatParticipant;
};

function formatMessageTime(value: string) {
  return new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit" }).format(
    new Date(value),
  );
}

export function MessageBubble({ message, participant }: MessageBubbleProps) {
  const isUser = message.role === "user";
  const statusLabel =
    message.status === "streaming" ? "HERMES WRITING" : message.status === "sending" ? "SENDING" : message.status === "error" ? "FAILED" : null;

  return (
    <article className={`message-row ${isUser ? "message-row--user" : "message-row--assistant"}`}>
      {!isUser ? <Avatar participant={participant} size="md" /> : null}
      <div className="message-bubble-wrap">
        <header className="message-meta">
          <span>{participant.displayName}</span>
          <time dateTime={message.createdAt}>{formatMessageTime(message.createdAt)}</time>
        </header>
        <div className="message-bubble">
          <p>{message.content}</p>
          {message.error ? <p className="message-error">{message.error}</p> : null}
        </div>
        {statusLabel ? <span className={`message-status message-status--${message.status}`}>{statusLabel}</span> : null}
      </div>
      {isUser ? <Avatar participant={participant} size="md" /> : null}
    </article>
  );
}
