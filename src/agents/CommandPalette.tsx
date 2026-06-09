import { useEffect, useMemo, useRef, useState, type FormEvent, type KeyboardEvent } from "react";
import type { HermesSlashCommand } from "./hermesCommands";
import { commandDisplayName, commandSearchText } from "./hermesCommands";

export type CommandPaletteProps = {
  open: boolean;
  commands: HermesSlashCommand[];
  recentCommands: string[];
  onClose: () => void;
  onRunCommand: (command: string) => void;
  onInsertCommand: (command: string) => void;
};

function clampIndex(index: number, length: number) {
  if (length <= 0) return 0;
  return Math.min(Math.max(index, 0), length - 1);
}

function nextIndex(current: number, length: number, delta: 1 | -1) {
  if (length <= 0) return 0;
  return (current + delta + length) % length;
}

export function CommandPalette({ open, commands, recentCommands, onClose, onRunCommand, onInsertCommand }: CommandPaletteProps) {
  const [query, setQuery] = useState("");
  const [highlightedIndex, setHighlightedIndex] = useState(0);
  const optionRefs = useRef<Array<HTMLButtonElement | null>>([]);
  const filtered = useMemo(() => {
    const search = query.trim().toLowerCase();
    if (!search) return commands;
    return commands.filter((command) => commandSearchText(command).includes(search));
  }, [commands, query]);

  useEffect(() => {
    setHighlightedIndex(0);
  }, [query, filtered.length, open]);

  useEffect(() => {
    if (!open) return;
    setHighlightedIndex((current) => clampIndex(current, filtered.length));
  }, [filtered.length, open]);

  useEffect(() => {
    if (!open) return;
    optionRefs.current[highlightedIndex]?.scrollIntoView({ block: "nearest" });
  }, [highlightedIndex, open]);

  if (!open) return null;

  function commandText(command: HermesSlashCommand) {
    const display = commandDisplayName(command);
    const hasArgs = Boolean(command.argsHint || command.subcommands.length);
    return `${display}${hasArgs ? " " : ""}`;
  }

  function insertOrRunCommand(command: HermesSlashCommand, runNow: boolean) {
    const text = commandText(command);
    if (runNow || !Boolean(command.argsHint || command.subcommands.length)) onRunCommand(text.trim());
    else onInsertCommand(text);
  }

  function handleSearchChange(event: FormEvent<HTMLInputElement>) {
    setHighlightedIndex(0);
    setQuery(event.currentTarget.value);
  }

  function handlePaletteKeyDown(event: KeyboardEvent<HTMLInputElement>) {
    if (event.key === "Escape") {
      event.preventDefault();
      onClose();
      return;
    }
    if (event.key === "ArrowDown") {
      event.preventDefault();
      setHighlightedIndex((current) => nextIndex(current, filtered.length, 1));
      return;
    }
    if (event.key === "ArrowUp") {
      event.preventDefault();
      setHighlightedIndex((current) => nextIndex(current, filtered.length, -1));
      return;
    }
    if (event.key === "Enter" && filtered.length > 0) {
      event.preventDefault();
      const command = filtered[highlightedIndex] ?? filtered[0];
      if (command) insertOrRunCommand(command, event.metaKey || event.ctrlKey);
    }
  }

  return (
    <div className="command-palette-backdrop" role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget) onClose(); }}>
      <section aria-label="Command palette" className="command-palette" role="dialog">
        <header>
          <div><strong>Commands</strong><span>Search and run native session commands</span></div>
          <button onClick={onClose} type="button">Close</button>
        </header>
        <input
          aria-activedescendant={filtered[highlightedIndex] ? `command-palette-option-${filtered[highlightedIndex].name}` : undefined}
          aria-controls="command-palette-results"
          aria-expanded={open}
          aria-label="Search commands"
          autoFocus
          onChange={handleSearchChange}
          onInput={handleSearchChange}
          onKeyDown={handlePaletteKeyDown}
          placeholder="Search command, alias, or subcommand…"
          role="combobox"
          value={query}
        />
        {!query.trim() && recentCommands.length > 0 ? (
          <div className="command-palette-section">
            <h3>Recent</h3>
            {recentCommands.map((command) => (
              <button key={command} onClick={() => onRunCommand(command)} type="button"><strong>{command}</strong><span>Run recent command</span></button>
            ))}
          </div>
        ) : null}
        <div className="command-palette-section command-palette-section--scroll" id="command-palette-results" role="listbox">
          <h3>All commands</h3>
          {filtered.map((command, index) => {
            const display = commandDisplayName(command);
            const hasArgs = Boolean(command.argsHint || command.subcommands.length);
            const active = index === highlightedIndex;
            return (
              <button
                aria-selected={active}
                className={active ? "command-palette-option--active" : undefined}
                id={`command-palette-option-${command.name}`}
                key={command.name}
                onClick={(event) => insertOrRunCommand(command, event.metaKey || event.ctrlKey || !hasArgs)}
                onMouseEnter={() => setHighlightedIndex(index)}
                ref={(element) => { optionRefs.current[index] = element; }}
                role="option"
                type="button"
              >
                <strong>{display} {command.argsHint ? <em>{command.argsHint}</em> : null}</strong>
                <span>{command.description}</span>
                <small>{[command.category, command.aliases.length ? `Aliases: ${command.aliases.map((alias) => `/${alias}`).join(", ")}` : null, command.subcommands.length ? `Subcommands: ${command.subcommands.join(", ")}` : null].filter(Boolean).join(" · ")}</small>
              </button>
            );
          })}
        </div>
      </section>
    </div>
  );
}
