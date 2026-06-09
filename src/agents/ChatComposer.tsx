import { forwardRef, useEffect, useImperativeHandle, useLayoutEffect, useMemo, useRef, useState, type CSSProperties, type ChangeEvent, type FormEvent, type KeyboardEvent } from "react";
import { GlobalDropdown } from "../ui/GlobalDropdown";
import type { HermesSlashCommand } from "./hermesCommands";
import { commandDisplayName, commandSearchText } from "./hermesCommands";
import { commandNeedsArgs } from "./slashCommandParser";

type ComposerAttachmentAction = "context" | "extract" | "upload";
type ComposerPanel = "attach" | "slash" | "settings" | "usage" | null;
type ComposerNoticeTone = "success" | "warning" | "error" | "info";

type ComposerAttachment = {
  id: string;
  file: File;
  action: ComposerAttachmentAction;
  status: "ready" | "unsupported" | "too-large";
};

type ComposerNotice = {
  tone: ComposerNoticeTone;
  text: string;
};

type ChatComposerProps = {
  disabled?: boolean;
  disabledReason?: string;
  isSending?: boolean;
  contextUsedPercent?: number;
  modelLabel?: string;
  temperature?: number;
  maxOutputTokens?: number;
  slashCommands?: HermesSlashCommand[];
  onSend: (message: string) => void | Promise<void>;
  onStop?: () => void | Promise<void>;
};

const attachmentActionLabels: Record<ComposerAttachmentAction, string> = {
  context: "Send as context",
  extract: "Extract text",
  upload: "Upload only",
};

const COMPOSER_MIN_HEIGHT = 44;
const COMPOSER_MAX_HEIGHT = 132;
const TYPING_SOUND_MIN_INTERVAL_MS = 70;
const TYPING_SOUND_VOLUME = 0.009;
const TYPING_SOUND_CLICK_VOLUME = 0.0035;

type ZoidAudioWindow = Window & typeof globalThis & {
  webkitAudioContext?: typeof AudioContext;
};

function formatFileSize(size: number) {
  if (size < 1024) return `${size} B`;
  if (size < 1024 * 1024) return `${Math.round(size / 1024)} KB`;
  return `${(size / (1024 * 1024)).toFixed(1)} MB`;
}

function getAttachmentStatus(file: File): ComposerAttachment["status"] {
  if (file.size > 25 * 1024 * 1024) return "too-large";
  return "ready";
}

function isTextExtractable(file: File) {
  if (file.type.startsWith("text/")) return true;
  return /\.(md|txt|json|csv|log|yaml|yml|xml|html|css|js|jsx|ts|tsx|py|rs|go|java|rb|php|sql|toml|ini|env)$/i.test(file.name);
}

function attachmentStatusCopy(attachment: ComposerAttachment) {
  if (attachment.status === "too-large") return "Too large for composer context";
  if (attachment.action === "upload") return "Kept in tray only; not sent to Hermes yet";
  if (attachment.action === "extract" && !isTextExtractable(attachment.file)) return "Binary text extraction is not available yet";
  if (attachment.action === "context" && !isTextExtractable(attachment.file)) return "Sends file metadata until native ingestion is wired";
  return "Ready";
}

function isHermesCliCommandDraft(message: string) {
  const trimmed = message.trim();
  if (!trimmed.startsWith("hermes")) return false;
  const rest = trimmed.slice("hermes".length);
  return rest.length === 0 || /^\s/.test(rest);
}

export function shouldStopHermesFromCopyShortcut(isSending: boolean, key: string, metaKey: boolean, ctrlKey: boolean, selectionStart: number | null, selectionEnd: number | null) {
  return isSending && (metaKey || ctrlKey) && key.toLowerCase() === "c" && selectionStart === selectionEnd;
}

export function getInlineSlashSearch(value: string) {
  if (!value.startsWith("/") || value.includes("\n")) return null;
  const commandDraft = value.slice(1);
  if (/\s/.test(commandDraft)) return null;
  return commandDraft.toLowerCase();
}

async function buildAttachmentContext(attachments: ComposerAttachment[]) {
  const included = attachments.filter((attachment) => attachment.action !== "upload");
  if (included.length === 0) return "";

  const sections = await Promise.all(included.map(async (attachment) => {
    const { file, action, status } = attachment;
    const header = `File: ${file.name} (${file.type || "unknown type"}, ${formatFileSize(file.size)}, ${attachmentActionLabels[action]}, ${status})`;

    if (status !== "ready") {
      return `${header}\nContent unavailable: ${status === "too-large" ? "file is too large for the current Zoid composer shell" : "unsupported file state"}.`;
    }

    if (action === "extract" && !isTextExtractable(file)) {
      return `${header}\nContent unavailable: binary/unknown file text extraction requires native file ingestion wiring.`;
    }

    if (action === "extract" || (action === "context" && isTextExtractable(file))) {
      try {
        const text = await file.text();
        const clipped = text.length > 16_000 ? `${text.slice(0, 16_000)}\n\n[Zoid clipped this file preview at 16k characters.]` : text;
        return `${header}\nExtracted text:\n${clipped}`;
      } catch (error) {
        return `${header}\nContent unavailable: ${error instanceof Error ? error.message : String(error)}.`;
      }
    }

    return `${header}\nBinary/non-text file attached. Hermes should treat this as file context metadata until backend file ingestion is connected.`;
  }));

  return `[Attached files]\n${sections.join("\n\n---\n\n")}`;
}

export type ChatComposerHandle = {
  focusMessageField: () => void;
  insertText: (text: string) => void;
};

export const ChatComposer = forwardRef<ChatComposerHandle, ChatComposerProps>(function ChatComposer({ disabled = false, disabledReason, isSending = false, contextUsedPercent = 1, modelLabel = "gpt-5.5", temperature = 0.7, maxOutputTokens = 4096, slashCommands = [], onSend, onStop }, ref) {
  const [value, setValue] = useState("");
  const [menuOpen, setMenuOpen] = useState(false);
  const [activePanel, setActivePanel] = useState<ComposerPanel>(null);
  const [commandSearch, setCommandSearch] = useState("");
  const [highlightedSlashCommandIndex, setHighlightedSlashCommandIndex] = useState(0);
  const [attachments, setAttachments] = useState<ComposerAttachment[]>([]);
  const [sessionTemperature, setSessionTemperature] = useState(temperature);
  const [sessionMaxOutputTokens, setSessionMaxOutputTokens] = useState(maxOutputTokens);
  const [notice, setNotice] = useState<ComposerNotice | null>(null);
  const [slashPanelMaxHeight, setSlashPanelMaxHeight] = useState(560);
  const formRef = useRef<HTMLFormElement>(null);
  const fileInputRef = useRef<HTMLInputElement>(null);
  const messageInputRef = useRef<HTMLTextAreaElement>(null);
  const inlineSlashOptionRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const previousComposerHeightRef = useRef(COMPOSER_MIN_HEIGHT);
  const composerHeightRef = useRef(COMPOSER_MIN_HEIGHT);
  const expansionTransitionTimerRef = useRef<number | null>(null);
  const typingAudioContextRef = useRef<AudioContext | null>(null);
  const lastTypingSoundAtRef = useRef(0);

  useImperativeHandle(ref, () => ({
    focusMessageField: () => {
      messageInputRef.current?.focus({ preventScroll: true });
    },
    insertText: (text: string) => {
      setValue((current) => current ? `${current.trimEnd()} ${text}` : text);
      window.requestAnimationFrame(() => messageInputRef.current?.focus({ preventScroll: true }));
    },
  }), []);

  const trimmed = value.trim();
  const isHermesCommandDraft = isHermesCliCommandDraft(trimmed);
  const actionableAttachments = attachments.filter((attachment) => attachment.action !== "upload" && attachment.status === "ready");
  const blockedActionableAttachments = attachments.filter((attachment) => attachment.action !== "upload" && attachment.status !== "ready");
  const cannotSend = disabled || (trimmed.length === 0 && actionableAttachments.length === 0);
  const settingsWiringUnavailable = true;
  const filteredCommands = useMemo(() => {
    const search = commandSearch.trim().toLowerCase();
    if (!search) return slashCommands;
    return slashCommands.filter((command) => commandSearchText(command).includes(search));
  }, [commandSearch, slashCommands]);
  const inlineSlashSearch = useMemo(() => getInlineSlashSearch(value), [value]);
  const inlineSlashCommands = useMemo(() => {
    if (inlineSlashSearch === null) return [];
    return inlineSlashSearch
      ? slashCommands.filter((command) => commandSearchText(command).includes(inlineSlashSearch))
      : slashCommands;
  }, [inlineSlashSearch, slashCommands]);
  const inlineSlashOpen = inlineSlashSearch !== null && inlineSlashCommands.length > 0 && activePanel === null && !menuOpen;

  useEffect(() => {
    setHighlightedSlashCommandIndex(0);
  }, [inlineSlashSearch, inlineSlashCommands.length]);

  useEffect(() => {
    if (!inlineSlashOpen) return;
    setHighlightedSlashCommandIndex((current) => {
      if (inlineSlashCommands.length === 0) return 0;
      return Math.min(Math.max(current, 0), inlineSlashCommands.length - 1);
    });
  }, [inlineSlashCommands.length, inlineSlashOpen]);

  useEffect(() => {
    if (!inlineSlashOpen) return;
    inlineSlashOptionRefs.current[highlightedSlashCommandIndex]?.scrollIntoView({ block: "nearest" });
  }, [highlightedSlashCommandIndex, inlineSlashOpen]);

  useLayoutEffect(() => {
    const textarea = messageInputRef.current;
    if (!textarea) return;

    const previousHeight = previousComposerHeightRef.current;
    textarea.style.height = "auto";
    const nextHeight = Math.min(Math.max(textarea.scrollHeight, COMPOSER_MIN_HEIGHT), COMPOSER_MAX_HEIGHT);
    textarea.style.overflowY = nextHeight >= COMPOSER_MAX_HEIGHT ? "auto" : "hidden";

    if (nextHeight > previousHeight) {
      if (expansionTransitionTimerRef.current !== null) {
        window.clearTimeout(expansionTransitionTimerRef.current);
      }
      textarea.dataset.expanding = "true";
      textarea.style.height = `${previousHeight}px`;
      void textarea.offsetHeight;
      textarea.style.height = `${nextHeight}px`;
      expansionTransitionTimerRef.current = window.setTimeout(() => {
        if (messageInputRef.current === textarea) delete textarea.dataset.expanding;
        expansionTransitionTimerRef.current = null;
      }, 240);
    } else {
      if (expansionTransitionTimerRef.current !== null) {
        window.clearTimeout(expansionTransitionTimerRef.current);
        expansionTransitionTimerRef.current = null;
      }
      delete textarea.dataset.expanding;
      textarea.style.height = `${nextHeight}px`;
    }

    previousComposerHeightRef.current = nextHeight;
    composerHeightRef.current = nextHeight;
    if (nextHeight !== previousHeight) textarea.dataset.composerHeight = String(composerHeightRef.current);
  }, [value]);

  useEffect(() => () => {
    if (expansionTransitionTimerRef.current !== null) {
      window.clearTimeout(expansionTransitionTimerRef.current);
    }
  }, []);

  useLayoutEffect(() => {
    if (activePanel !== "slash") return;

    function updateSlashPanelBounds() {
      const form = formRef.current;
      if (!form) return;
      const workspace = form.closest(".chat-workspace");
      const formRect = form.getBoundingClientRect();
      const workspaceRect = workspace?.getBoundingClientRect();
      const safeTop = workspaceRect?.top ?? 0;
      const availableAboveComposer = Math.floor(formRect.top - safeTop - 22);
      setSlashPanelMaxHeight(Math.max(140, Math.min(560, availableAboveComposer)));
    }

    updateSlashPanelBounds();
    window.addEventListener("resize", updateSlashPanelBounds);
    const resizeObserver = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(updateSlashPanelBounds);
    if (resizeObserver && formRef.current) resizeObserver.observe(formRef.current);
    return () => {
      window.removeEventListener("resize", updateSlashPanelBounds);
      resizeObserver?.disconnect();
    };
  }, [activePanel]);

  function getTypingAudioContext() {
    if (typingAudioContextRef.current) return typingAudioContextRef.current;
    const AudioContextConstructor = window.AudioContext ?? (window as ZoidAudioWindow).webkitAudioContext;
    if (!AudioContextConstructor) return null;
    typingAudioContextRef.current = new AudioContextConstructor();
    return typingAudioContextRef.current;
  }

  function playCalmingTypingSound(inputType?: string) {
    if (inputType?.startsWith("history") || inputType === "insertFromPaste") return;
    const nowMs = performance.now();
    if (nowMs - lastTypingSoundAtRef.current < TYPING_SOUND_MIN_INTERVAL_MS) return;
    lastTypingSoundAtRef.current = nowMs;

    const context = getTypingAudioContext();
    if (!context) return;
    void context.resume();

    const now = context.currentTime;
    const primaryOscillator = context.createOscillator();
    const clickOscillator = context.createOscillator();
    const primaryGain = context.createGain();
    const clickGain = context.createGain();
    const filter = context.createBiquadFilter();
    const pitch = inputType?.startsWith("delete") ? 420 : 640 + Math.random() * 70;

    primaryOscillator.type = "sine";
    primaryOscillator.frequency.setValueAtTime(pitch, now);
    primaryOscillator.frequency.exponentialRampToValueAtTime(Math.max(260, pitch * 0.74), now + 0.055);
    clickOscillator.type = "triangle";
    clickOscillator.frequency.setValueAtTime(pitch * 1.7, now);
    clickOscillator.frequency.exponentialRampToValueAtTime(Math.max(620, pitch * 1.15), now + 0.032);

    filter.type = "bandpass";
    filter.frequency.setValueAtTime(780, now);
    filter.Q.setValueAtTime(0.72, now);

    primaryGain.gain.setValueAtTime(0.0001, now);
    primaryGain.gain.exponentialRampToValueAtTime(TYPING_SOUND_VOLUME, now + 0.008);
    primaryGain.gain.exponentialRampToValueAtTime(0.0001, now + 0.068);
    clickGain.gain.setValueAtTime(0.0001, now);
    clickGain.gain.exponentialRampToValueAtTime(TYPING_SOUND_CLICK_VOLUME, now + 0.004);
    clickGain.gain.exponentialRampToValueAtTime(0.0001, now + 0.035);

    primaryOscillator.connect(primaryGain);
    clickOscillator.connect(clickGain);
    primaryGain.connect(filter);
    clickGain.connect(filter);
    filter.connect(context.destination);
    primaryOscillator.start(now);
    clickOscillator.start(now);
    primaryOscillator.stop(now + 0.075);
    clickOscillator.stop(now + 0.04);
    primaryOscillator.onended = () => {
      primaryOscillator.disconnect();
      clickOscillator.disconnect();
      primaryGain.disconnect();
      clickGain.disconnect();
      filter.disconnect();
    };
  }

  function handleMessageChange(event: ChangeEvent<HTMLTextAreaElement> | FormEvent<HTMLTextAreaElement>) {
    const nextValue = event.currentTarget.value;
    setValue(nextValue);
    if (nextValue.startsWith("/")) {
      setMenuOpen(false);
      setActivePanel(null);
    }
    playCalmingTypingSound((event.nativeEvent as InputEvent).inputType);
  }

  async function submit() {
    if (cannotSend) {
      if (disabled) setNotice({ tone: "warning", text: disabledReason || "Hermes is not reachable yet. You can keep drafting, but Send is locked." });
      if (isSending) setNotice({ tone: "info", text: "Hermes is still responding. Type a message to queue it, or press Ctrl/Cmd+C to stop the active run." });
      return;
    }

    try {
      setNotice(null);
      const attachmentContext = await buildAttachmentContext(attachments);
      const message = [trimmed, attachmentContext].filter(Boolean).join("\n\n");
      await onSend(message);
      setValue("");
      setAttachments((current) => current.filter((attachment) => attachment.action === "upload"));
      setNotice(isSending ? { tone: "success", text: "Queued for the next Hermes turn." } : blockedActionableAttachments.length > 0 ? { tone: "warning", text: `${blockedActionableAttachments.length} blocked attachment(s) were not sent.` } : null);
    } catch (error) {
      setNotice({ tone: "error", text: `Send failed. Draft and attachments were preserved: ${error instanceof Error ? error.message : String(error)}` });
    }
  }

  function handleKeyDown(event: KeyboardEvent<HTMLTextAreaElement>) {
    if (shouldStopHermesFromCopyShortcut(isSending, event.key, event.metaKey, event.ctrlKey, event.currentTarget.selectionStart, event.currentTarget.selectionEnd)) {
      event.preventDefault();
      void onStop?.();
      return;
    }
    if (inlineSlashOpen) {
      if (event.key === "ArrowDown") {
        event.preventDefault();
        setHighlightedSlashCommandIndex((current) => (current + 1) % inlineSlashCommands.length);
        return;
      }
      if (event.key === "ArrowUp") {
        event.preventDefault();
        setHighlightedSlashCommandIndex((current) => (current - 1 + inlineSlashCommands.length) % inlineSlashCommands.length);
        return;
      }
      if (event.key === "Tab" || event.key === "Enter") {
        event.preventDefault();
        const command = inlineSlashCommands[highlightedSlashCommandIndex] ?? inlineSlashCommands[0];
        if (command) insertCommand(command, false);
        return;
      }
      if (event.key === "Escape") {
        event.preventDefault();
        setValue("");
        return;
      }
    }
    if (event.key === "Enter" && !event.shiftKey) {
      event.preventDefault();
      void submit();
    }
  }

  function handleFilesSelected(event: ChangeEvent<HTMLInputElement>) {
    const files = Array.from(event.target.files ?? []);
    if (files.length > 0) {
      setAttachments((current) => [
        ...current,
        ...files.map((file) => ({ id: `attachment-${crypto.randomUUID()}`, file, action: "context" as const, status: getAttachmentStatus(file) })),
      ]);
      setNotice({ tone: "success", text: `${files.length} file(s) attached. Choose how each file should be used before sending.` });
      setActivePanel("attach");
    }
    event.target.value = "";
    setMenuOpen(false);
  }

  function insertCommand(command: HermesSlashCommand, runNow: boolean) {
    const displayName = commandDisplayName(command);
    const commandText = `${displayName}${commandNeedsArgs(command) ? " " : ""}`;
    if (runNow) {
      if (disabled || isSending) {
        setValue((current) => current ? `${current.trimEnd()} ${commandText}` : commandText);
        setNotice({ tone: "warning", text: disabled ? "Hermes is offline, so the command was inserted as a draft instead of sent." : "Hermes is responding, so the command was inserted as a draft." });
        setActivePanel(null);
        setMenuOpen(false);
        return;
      }
      setActivePanel(null);
      setMenuOpen(false);
      setNotice(null);
      void Promise.resolve(onSend(commandText.trim())).catch((error: unknown) => {
        setValue(commandText);
        setNotice({ tone: "error", text: `Slash command failed and was restored as a draft: ${error instanceof Error ? error.message : String(error)}` });
      });
      return;
    }
    setValue((current) => current ? `${current.trimEnd()} ${commandText}` : commandText);
    setNotice({ tone: "success", text: `${displayName} inserted. Add details, then send.` });
    setActivePanel(null);
    setMenuOpen(false);
  }

  function openPanel(panel: ComposerPanel) {
    setActivePanel(panel);
    setMenuOpen(false);
  }

  async function copyUsageReport() {
    const report = `Zoid session usage: ${contextUsedPercent}% context, ${attachments.length} attachment(s), model ${modelLabel}`;
    try {
      await navigator.clipboard?.writeText(report);
      setNotice({ tone: "success", text: "Usage report copied." });
    } catch (error) {
      setNotice({ tone: "error", text: `Could not copy usage report: ${error instanceof Error ? error.message : String(error)}` });
    }
  }

  const composerActions = [
    { id: "attach", label: "Attach files", subtitle: "Any format. Text files can be read into context.", badge: attachments.length ? `${attachments.length} attached` : "Any format" },
    { id: "slash", label: "Slash commands", subtitle: "Live native command registry.", badge: `${slashCommands.length} commands` },
    { id: "settings", label: "Agent settings", subtitle: "View temperature, output, model, tools.", badge: "Requires wiring" },
    { id: "usage", label: "Session usage", subtitle: "Context, token estimates, cleanup actions.", badge: `${contextUsedPercent}% context` },
  ];

  return (
    <form
      aria-label="Hermes message composer"
      className="chat-composer"
      ref={formRef}
      onSubmit={(event) => {
        event.preventDefault();
        void submit();
      }}
    >
      <div className="composer-actions-root">
        <button aria-expanded={menuOpen} aria-haspopup="menu" className="composer-attach" onClick={() => { setMenuOpen((current) => !current); setActivePanel(null); }} title="Composer actions" type="button">
          +
        </button>
        <input aria-label="Attach files" className="composer-file-input" multiple onChange={handleFilesSelected} ref={fileInputRef} type="file" />
        {menuOpen ? (
          <div className="composer-action-popover" role="menu" aria-label="Composer actions">
            <div className="composer-popover-title"><strong>Add to message</strong><span>Choose one action</span></div>
            {composerActions.map((action) => (
              <button
                className="composer-action-row"
                key={action.id}
                onClick={() => {
                  if (action.id === "attach") openPanel("attach");
                  if (action.id === "slash") openPanel("slash");
                  if (action.id === "settings") openPanel("settings");
                  if (action.id === "usage") openPanel("usage");
                }}
                role="menuitem"
                type="button"
              >
                <span><strong>{action.label}</strong><small>{action.subtitle}</small></span>
                <em>{action.badge}</em>
              </button>
            ))}
          </div>
        ) : null}
      </div>

      <div className="composer-input-column">
        {inlineSlashOpen ? (
          <div className="composer-slash-dropup" role="listbox" aria-label="Available slash commands" aria-activedescendant={inlineSlashCommands[highlightedSlashCommandIndex] ? `composer-slash-option-${inlineSlashCommands[highlightedSlashCommandIndex].name}` : undefined}>
            <div className="composer-slash-dropup-header">
              <strong>Slash commands</strong>
              <span>{slashCommands.length} live Hermes commands · ↑↓ navigate · Tab inserts</span>
            </div>
            <div className="composer-slash-dropup-list" id="composer-slash-dropup-list">
              {inlineSlashCommands.map((command, index) => (
                <button
                  aria-selected={index === highlightedSlashCommandIndex}
                  className={`composer-slash-dropup-option${index === highlightedSlashCommandIndex ? " composer-slash-dropup-option--active" : ""}`}
                  id={`composer-slash-option-${command.name}`}
                  key={command.name}
                  onMouseDown={(event) => event.preventDefault()}
                  onMouseEnter={() => setHighlightedSlashCommandIndex(index)}
                  onClick={() => insertCommand(command, false)}
                  ref={(element) => { inlineSlashOptionRefs.current[index] = element; }}
                  role="option"
                  type="button"
                >
                  <span className="slash-command-meta">{command.category}</span>
                  <strong>{commandDisplayName(command)} {command.argsHint ? <em>{command.argsHint}</em> : null}</strong>
                  <small>{command.description}</small>
                </button>
              ))}
            </div>
          </div>
        ) : null}
        {attachments.length > 0 ? (
          <div className="composer-attachment-tray" aria-label="Attached files">
            {attachments.map((attachment) => (
              <span className={`composer-attachment-chip composer-attachment-chip--${attachment.status}`} key={attachment.id}>
                <span className="attachment-chip-main">
                  <strong>{attachment.file.name}</strong>
                  <small>{formatFileSize(attachment.file.size)} · {attachment.file.type || "unknown type"}</small>
                  <small>{attachmentStatusCopy(attachment)}</small>
                </span>
                <GlobalDropdown
                  label={`Action for ${attachment.file.name}`}
                  onChange={(nextAction) => setAttachments((current) => current.map((item) => item.id === attachment.id ? { ...item, action: nextAction as ComposerAttachmentAction } : item))}
                  options={[
                    { value: "context", label: "Send as context" },
                    { value: "extract", label: "Extract text" },
                    { value: "upload", label: "Upload only" },
                  ]}
                  size="compact"
                  value={attachment.action}
                />
                <button aria-label={`Remove ${attachment.file.name}`} onClick={() => setAttachments((current) => current.filter((item) => item.id !== attachment.id))} type="button">×</button>
              </span>
            ))}
          </div>
        ) : null}
        <label className={`composer-input-wrap${isHermesCommandDraft ? " composer-input-wrap--hermes-command" : ""}`}>
          <span className="composer-input-label-row">
            <span>Message Hermes</span>
            {isHermesCommandDraft ? <strong className="composer-mode-chip">Hermes CLI command</strong> : null}
          </span>
          <textarea
            aria-activedescendant={inlineSlashOpen && inlineSlashCommands[highlightedSlashCommandIndex] ? `composer-slash-option-${inlineSlashCommands[highlightedSlashCommandIndex].name}` : undefined}
            aria-controls={inlineSlashOpen ? "composer-slash-dropup-list" : undefined}
            aria-expanded={inlineSlashOpen}
            aria-describedby={[isHermesCommandDraft ? "composer-mode-note" : null, disabledReason || notice ? "composer-status-note" : null].filter(Boolean).join(" ") || undefined}
            onChange={handleMessageChange}
            onInput={handleMessageChange}
            onKeyDown={handleKeyDown}
            placeholder={disabled ? "Hermes is offline. Draft here; Send unlocks when the CLI is reachable." : isSending ? "Hermes is responding. Draft the next message here; Send unlocks after the current response." : "Message Hermes or type hermes tools list..."}
            ref={messageInputRef}
            rows={1}
            value={value}
          />
        </label>
        {isHermesCommandDraft ? (
          <div className="composer-mode-strip composer-mode-strip--hermes-command" id="composer-mode-note" role="status">
            <strong>CLI mode armed</strong>
            <span>Zoid will run this through the Hermes CLI bridge without cluttering the chat transcript.</span>
          </div>
        ) : null}
      </div>
      <button
        className={`composer-send${isHermesCommandDraft ? " composer-send--hermes-command" : ""}${isSending ? " composer-send--stop" : ""}`}
        disabled={disabled || (!isSending && trimmed.length === 0 && actionableAttachments.length === 0)}
        onClick={(event) => {
          if (!isSending || trimmed.length > 0 || actionableAttachments.length > 0) return;
          event.preventDefault();
          void onStop?.();
        }}
        title={isSending ? (trimmed.length > 0 || actionableAttachments.length > 0 ? "Queue this message. Press Ctrl/Cmd+C to stop the current run." : "Stop Hermes run (Ctrl/Cmd+C)") : undefined}
        type="submit"
      >
        {isSending ? (trimmed.length > 0 || actionableAttachments.length > 0 ? "QUEUE" : "STOP") : disabled ? "LOCKED" : isHermesCommandDraft ? "RUN CLI" : "SEND"}
      </button>
      {(disabledReason || notice) ? (
        <p className={`composer-status-note composer-status-note--${notice?.tone ?? "warning"}`} id="composer-status-note" role={notice?.tone === "error" ? "alert" : "status"}>
          {notice?.text ?? disabledReason}
        </p>
      ) : null}

      {activePanel === "attach" ? (
        <div className="composer-deep-panel composer-deep-panel--attach" role="dialog" aria-label="Attach files">
          <div className="composer-panel-header"><strong>Attach files</strong><button onClick={() => setActivePanel(null)} type="button">Close</button></div>
          <section className="composer-attach-dropzone">
            <div><strong>Choose file(s)</strong><span>Any format can be selected. Text-like files can be extracted; binary files are sent as honest metadata until native file ingestion is wired.</span></div>
            <button onClick={() => fileInputRef.current?.click()} type="button">Browse files</button>
          </section>
          <div className="composer-attachment-rules">
            <span>Default: Send as context</span>
            <span>Limit: 25 MB per file</span>
            <span>No fake uploads: Upload only stays local until backend storage exists</span>
          </div>
        </div>
      ) : null}

      {activePanel === "slash" ? (
        <div className="composer-deep-panel composer-deep-panel--slash" role="dialog" aria-label="Slash commands" style={{ "--composer-slash-panel-max-height": `${slashPanelMaxHeight}px` } as CSSProperties}>
          <div className="composer-panel-header"><strong>Slash commands</strong><button onClick={() => setActivePanel(null)} type="button">Close</button></div>
          <input aria-label="Search slash commands" onChange={(event) => setCommandSearch(event.target.value)} placeholder="Search commands…" value={commandSearch} />
          <p className="composer-panel-helper">Based on Hermes Agent slash-command registry reference and pulled from the live command registry. Run <strong>/help</strong> for the full in-session reference. You can also type terminal-style commands such as <strong>hermes tools list</strong> or <strong>hermes cron list</strong>; Zoid runs the CLI bridge and keeps terminal plumbing out of the conversation.</p>
          <div className="slash-command-list">
            {filteredCommands.map((command) => (
              <button
                className="slash-command-option"
                key={command.name}
                onClick={(event) => insertCommand(command, event.metaKey || event.ctrlKey)}
                onKeyDown={(event) => {
                  if (event.key === "Enter" && (event.metaKey || event.ctrlKey)) {
                    event.preventDefault();
                    insertCommand(command, true);
                  }
                }}
                type="button"
              >
                <span className="slash-command-meta">{command.category}</span>
                <strong>{commandDisplayName(command)} {command.argsHint ? <em>{command.argsHint}</em> : null}</strong>
                <span>{command.description}</span>
                <small>{[command.aliases.length ? `Aliases: ${command.aliases.map((alias) => `/${alias}`).join(", ")}` : null, command.subcommands.length ? `Subcommands: ${command.subcommands.join(", ")}` : null, "Enter inserts · Cmd/Ctrl+Enter runs"].filter(Boolean).join(" · ")}</small>
              </button>
            ))}
          </div>
        </div>
      ) : null}

      {activePanel === "settings" ? (
        <div className="composer-deep-panel" role="dialog" aria-label="Agent settings">
          <div className="composer-panel-header"><strong>Agent settings</strong><button onClick={() => setActivePanel(null)} type="button">Close</button></div>
          <section className="composer-settings-section">
            <h3>Session overrides</h3>
            <label>Temperature <output>{sessionTemperature.toFixed(1)}</output><input disabled={settingsWiringUnavailable || isSending} max="2" min="0" onChange={(event) => setSessionTemperature(Number(event.target.value))} step="0.1" type="range" value={sessionTemperature} /></label>
            <label>Max output tokens <input disabled={settingsWiringUnavailable || isSending} min="512" onChange={(event) => setSessionMaxOutputTokens(Number(event.target.value))} step="512" type="number" value={sessionMaxOutputTokens} /></label>
            <p>Session override editing requires Hermes settings wiring. Current values are shown as a shell only and are not applied yet.</p>
          </section>
          <section className="composer-settings-section">
            <h3>Default profile</h3>
            <dl><div><dt>Model/provider</dt><dd>{modelLabel}</dd></div><div><dt>Default tools/profile</dt><dd>Managed by Hermes Agent config</dd></div></dl>
            <button disabled type="button">Open full agent settings — requires Hermes profile wiring</button>
          </section>
        </div>
      ) : null}

      {activePanel === "usage" ? (
        <div className="composer-deep-panel" role="dialog" aria-label="Session usage">
          <div className="composer-panel-header"><strong>Session usage</strong><button onClick={() => setActivePanel(null)} type="button">Close</button></div>
          <div className="usage-meter"><span style={{ width: `${Math.min(100, Math.max(0, contextUsedPercent))}%` }} /></div>
          <dl className="usage-grid">
            <div><dt>Context used</dt><dd>{contextUsedPercent}%</dd></div>
            <div><dt>Input tokens</dt><dd>Estimate unavailable</dd></div>
            <div><dt>Output tokens</dt><dd>Estimate unavailable</dd></div>
            <div><dt>Largest contributors</dt><dd>{attachments.length ? `${attachments.length} attached file(s)` : "Messages"}</dd></div>
          </dl>
          <p className="composer-panel-helper">Exact token counts, compaction, and summary controls need Hermes runtime usage wiring. Local attachment cleanup works now.</p>
          <div className="usage-actions">
            <button disabled type="button">Compact/summarize session — requires Hermes wiring</button>
            <button disabled={attachments.length === 0} onClick={() => { setAttachments([]); setNotice({ tone: "success", text: "Attached files removed from local composer context." }); }} type="button">Remove attached files from context</button>
            <button disabled type="button">Start new session from summary — requires Hermes wiring</button>
            <button onClick={() => void copyUsageReport()} type="button">Copy usage report</button>
          </div>
        </div>
      ) : null}
    </form>
  );
});
