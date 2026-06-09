import { useState } from "react";
import { Check, Clipboard, RotateCcw } from "lucide-react";
import { Avatar } from "./Avatar";
import type { ChatMessage, ChatParticipant } from "./types";

type MessageBubbleProps = {
  message: ChatMessage;
  participant: ChatParticipant;
  canRollback?: boolean;
  onRollback?: () => void;
};

function formatMessageTime(value: string) {
  return new Intl.DateTimeFormat(undefined, { hour: "2-digit", minute: "2-digit" }).format(
    new Date(value),
  );
}

function isHermesCliCommandMessage(message: string) {
  const trimmed = message.trim();
  if (!trimmed.startsWith("hermes")) return false;
  const rest = trimmed.slice("hermes".length);
  return rest.length === 0 || /^\s/.test(rest);
}

function stripTerminalCommandPlumbing(content: string) {
  let skippingCommandBlock = false;
  return content
    .split("\n")
    .filter((line) => {
      const trimmed = line.trim();
      const isCommandHeader = /^terminal command used:$/i.test(trimmed);
      const isShellCommandLine = /^\$\s*(cd\s+.+&&\s+)?hermes\b/.test(trimmed);
      if (isCommandHeader || isShellCommandLine) {
        skippingCommandBlock = true;
        return false;
      }
      if (skippingCommandBlock && trimmed.startsWith("$")) {
        return false;
      }
      if (skippingCommandBlock && trimmed.length === 0) {
        skippingCommandBlock = false;
        return false;
      }
      return true;
    })
    .join("\n")
    .trim();
}

async function copyTextToClipboard(text: string) {
  if (navigator.clipboard?.writeText) {
    await navigator.clipboard.writeText(text);
    return;
  }

  const textarea = document.createElement("textarea");
  textarea.value = text;
  textarea.setAttribute("readonly", "true");
  textarea.style.position = "fixed";
  textarea.style.inset = "0 auto auto -9999px";
  document.body.appendChild(textarea);
  textarea.select();
  document.execCommand("copy");
  textarea.remove();
}

export function MessageBubble({ message, participant, canRollback = false, onRollback }: MessageBubbleProps) {
  const [copyState, setCopyState] = useState<"idle" | "copied" | "failed">("idle");
  const isUser = message.role === "user";
  const isHermesCommand = isUser && isHermesCliCommandMessage(message.content);
  const visibleContent = stripTerminalCommandPlumbing(message.content);
  const copyContent = [visibleContent, message.error].filter(Boolean).join("\n\n");
  const hasVisibleContent = visibleContent.length > 0 || Boolean(message.error);
  const shouldShowBubble = hasVisibleContent && !(message.status === "streaming" && visibleContent.length === 0 && !message.error);
  const statusLabel =
    message.status === "streaming" ? "HERMES WRITING" : message.status === "sending" ? "SENDING" : message.status === "error" ? "FAILED" : null;

  async function handleCopyMessage() {
    if (!copyContent) return;
    try {
      await copyTextToClipboard(copyContent);
      setCopyState("copied");
      window.setTimeout(() => setCopyState("idle"), 1400);
    } catch {
      setCopyState("failed");
      window.setTimeout(() => setCopyState("idle"), 1800);
    }
  }

  return (
    <article className={`message-row ${isUser ? "message-row--user" : "message-row--assistant"}`}>
      {!isUser ? <Avatar participant={participant} size="md" /> : null}
      <div className="message-bubble-wrap">
        <header className="message-meta">
          <span>{participant.displayName}</span>
          {isHermesCommand ? <strong className="message-command-chip">Hermes CLI command</strong> : null}
          <time dateTime={message.createdAt}>{formatMessageTime(message.createdAt)}</time>
        </header>
        {shouldShowBubble ? (
          <div className="message-bubble-frame">
            <div className="message-bubble">
              {visibleContent.length > 0 ? <p className="message-content">{visibleContent}</p> : null}
              {message.error ? <p className="message-error message-content">{message.error}</p> : null}
            </div>
            <div className="message-actions" aria-label={`${participant.displayName} message actions`}>
              {canRollback ? (
                <button
                  aria-label={`Roll back conversation to ${participant.displayName} message`}
                  className="message-action-button message-rollback-button"
                  onClick={onRollback}
                  title="Roll back conversation to here"
                  type="button"
                >
                  <RotateCcw size={13} strokeWidth={2.4} />
                </button>
              ) : null}
              <button
                aria-label={`${copyState === "copied" ? "Copied" : "Copy"} ${participant.displayName} message`}
                className={`message-action-button message-copy-button${copyState === "copied" ? " message-copy-button--copied" : ""}${copyState === "failed" ? " message-copy-button--failed" : ""}`}
                disabled={!copyContent}
                onClick={handleCopyMessage}
                title={copyState === "copied" ? "Copied" : copyState === "failed" ? "Copy failed" : "Copy message"}
                type="button"
              >
                {copyState === "copied" ? <Check size={13} strokeWidth={2.5} /> : <Clipboard size={13} strokeWidth={2.4} />}
              </button>
            </div>
          </div>
        ) : null}
        {statusLabel ? (
          <span className={`message-status message-status--${message.status}`}>
            <span>{statusLabel}</span>
            {message.status === "streaming" ? <span className="message-writing-glyph" aria-hidden="true"><i /><i /><i /></span> : null}
          </span>
        ) : null}
      </div>
      {isUser ? <Avatar participant={participant} size="md" /> : null}
    </article>
  );
}
