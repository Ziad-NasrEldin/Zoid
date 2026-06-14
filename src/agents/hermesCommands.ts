export type HermesCommandPanel = "model" | "tools" | "skills" | "cron" | "agents" | "profile" | "history" | "usage" | "debug" | "browser";

export type HermesSlashCommand = {
  name: string;
  aliases: string[];
  description: string;
  category: string;
  argsHint?: string | null;
  subcommands: string[];
  cliOnly: boolean;
  gatewayOnly: boolean;
  zoidBehavior: "native-panel" | "forward" | "noop" | "confirm-forward";
  panel?: HermesCommandPanel | null;
};

export type HermesSlashCommandExecution = {
  kind: "text" | "panel" | "confirmation" | "new-session" | "close-session" | "error";
  content?: string | null;
  session?: string | null;
  panel?: HermesCommandPanel | null;
  requiresConfirmation: boolean;
  command: string;
  scope: "current-session" | "global-hermes" | string;
};

const CORE_COMMAND_DESCRIPTIONS: Record<string, string> = {
  plan: "Draft a plan before Zoid/Hermes acts.",
  help: "Show the command reference for this session.",
  clear: "Start with a clean session surface.",
  new: "Begin a new Hermes session.",
};

export const fallbackHermesSlashCommands: HermesSlashCommand[] = [
  { name: "plan", aliases: ["p"], description: "Prepare an implementation plan", category: "offline reference", argsHint: "<request>", subcommands: [], cliOnly: false, gatewayOnly: false, zoidBehavior: "forward" },
  { name: "help", aliases: ["h"], description: "Show help", category: "offline reference", subcommands: [], cliOnly: false, gatewayOnly: false, zoidBehavior: "forward" },
  { name: "clear", aliases: [], description: "Clear screen and start clean", category: "offline reference", subcommands: [], cliOnly: false, gatewayOnly: false, zoidBehavior: "forward" },
  { name: "new", aliases: ["reset"], description: "Start a new session", category: "offline reference", subcommands: [], cliOnly: false, gatewayOnly: false, zoidBehavior: "forward" },
];

export function commandDisplayName(command: HermesSlashCommand) {
  return `/${command.name}`;
}

export function commandDisplayDescription(command: HermesSlashCommand) {
  return CORE_COMMAND_DESCRIPTIONS[command.name] ?? command.description;
}

export function commandSearchText(command: HermesSlashCommand) {
  return [command.name, ...command.aliases, command.category, command.description, commandDisplayDescription(command), command.argsHint ?? "", ...command.subcommands].join(" ").toLowerCase();
}

function commandRank(command: HermesSlashCommand, rawSearch: string) {
  const search = rawSearch.trim().toLowerCase();
  if (!search) return 0;
  if (command.name === search) return 0;
  if (command.name.startsWith(search)) return 1;
  if (command.aliases.some((alias) => alias === search)) return 2;
  if (command.aliases.some((alias) => alias.startsWith(search))) return 3;
  if (command.category.toLowerCase().includes(search)) return 4;
  if ((command.argsHint ?? "").toLowerCase().includes(search)) return 5;
  if (command.subcommands.some((subcommand) => subcommand.toLowerCase().includes(search))) return 6;
  if (command.description.toLowerCase().includes(search) || commandDisplayDescription(command).toLowerCase().includes(search)) return 7;
  return 8;
}

export function sortSlashCommandsForSearch(commands: HermesSlashCommand[], search: string) {
  if (!search.trim()) return commands;
  return [...commands].sort((left, right) => {
    const rankDelta = commandRank(left, search) - commandRank(right, search);
    if (rankDelta !== 0) return rankDelta;
    return left.category.localeCompare(right.category) || left.name.localeCompare(right.name);
  });
}
