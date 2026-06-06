import { useState, type KeyboardEvent } from "react";

type ChatComposerProps = {
  disabled?: boolean;
  disabledReason?: string;
  isSending?: boolean;
  onSend: (message: string) => void | Promise<void>;
};

export function ChatComposer({ disabled = false, disabledReason, isSending = false, onSend }: ChatComposerProps) {
  const [value, setValue] = useState("");
  const trimmed = value.trim();
  const cannotSend = disabled || isSending || trimmed.length === 0;

  async function submit() {
    if (cannotSend) return;
    const message = trimmed;
    setValue("");
    await onSend(message);
  }

  function handleKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      void submit();
    }
  }

  return (
    <form
      aria-label="Hermes message composer"
      className="chat-composer"
      onSubmit={(event) => {
        event.preventDefault();
        void submit();
      }}
    >
      <button aria-disabled="true" className="composer-attach" disabled title="Attachments are not active yet" type="button">
        +
      </button>
      <label className="composer-input-wrap">
        <span>Message Hermes</span>
        <textarea
          aria-describedby={disabledReason ? "composer-disabled-reason" : undefined}
          disabled={disabled || isSending}
          onChange={(event) => setValue(event.target.value)}
          onKeyDown={handleKeyDown}
          placeholder="Message Hermes..."
          rows={2}
          value={value}
        />
      </label>
      <button className="composer-send" disabled={cannotSend} type="submit">
        {isSending ? "SENDING" : "SEND"}
      </button>
      {disabledReason ? <p id="composer-disabled-reason">{disabledReason}</p> : null}
    </form>
  );
}
