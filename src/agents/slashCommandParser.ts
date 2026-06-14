import type { HermesSlashCommand } from "./hermesCommands";

export type ParsedSlashCommand = {
  raw: string;
  token: string;
  args: string;
  canonicalName: string;
  command?: HermesSlashCommand;
};

export function parseSlashCommand(input: string, commands: HermesSlashCommand[]): ParsedSlashCommand | null {
  const raw = input.trim();
  if (!raw.startsWith("/")) return null;
  const withoutSlash = raw.slice(1).trimStart();
  const [token = "", ...rest] = withoutSlash.split(/\s+/);
  if (!token) return null;
  const normalized = token.toLowerCase();
  const command = commands.find((candidate) => candidate.name === normalized || candidate.aliases.includes(normalized));
  return {
    raw,
    token: normalized,
    args: rest.join(" "),
    canonicalName: command?.name ?? normalized,
    command,
  };
}

export function commandNeedsArgs(command: HermesSlashCommand) {
  return Boolean(command.argsHint || command.subcommands.length > 0);
}

export function commandNeedsRequiredArgs(command: HermesSlashCommand) {
  return Boolean(command.argsHint?.trim().startsWith("<"));
}
