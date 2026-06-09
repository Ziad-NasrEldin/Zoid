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

export function commandDisplayName(command: HermesSlashCommand) {
  return `/${command.name}`;
}

export function commandSearchText(command: HermesSlashCommand) {
  return [command.name, ...command.aliases, command.category, command.description, command.argsHint ?? "", ...command.subcommands].join(" ").toLowerCase();
}
