import { invoke } from "@tauri-apps/api/core";
import type { HermesSlashCommand, HermesSlashCommandExecution } from "./hermesCommands";
import type { HermesCliMessage, HermesCliResponse, HermesCliStatus } from "./types";

export async function getHermesCliStatus(): Promise<HermesCliStatus> {
  try {
    return await invoke<HermesCliStatus>("check_hermes_cli");
  } catch (error) {
    return {
      ok: false,
      status: "error",
      message: error instanceof Error ? error.message : String(error),
      session: "most-recent-hermes-cli-session",
    };
  }
}

export async function sendHermesCliMessage(
  messages: HermesCliMessage[],
  linkedRepository?: string,
  hermesSession?: string,
): Promise<HermesCliResponse> {
  return invoke<HermesCliResponse>("send_hermes_cli_message", { messages, linkedRepository, hermesSession });
}

export async function sendHermesCliRunMessage(
  messages: HermesCliMessage[],
  linkedRepository?: string,
  hermesSession?: string,
  sessionId?: string,
  runId?: string,
): Promise<HermesCliResponse> {
  return invoke<HermesCliResponse>("send_hermes_cli_run_message", { messages, linkedRepository, hermesSession, sessionId, runId });
}

export function cancelHermesCliMessage(): Promise<boolean> {
  return invoke<boolean>("cancel_hermes_cli_message");
}

export function cancelHermesCliRun(sessionId?: string, runId?: string): Promise<boolean> {
  return invoke<boolean>("cancel_hermes_cli_run", { sessionId, runId });
}

export function listHermesSlashCommands(): Promise<HermesSlashCommand[]> {
  return invoke<HermesSlashCommand[]>("list_hermes_slash_commands");
}

export function executeHermesSlashCommand(
  command: string,
  linkedRepository?: string,
  hermesSession?: string,
  confirmed = false,
): Promise<HermesSlashCommandExecution> {
  return invoke<HermesSlashCommandExecution>("execute_hermes_slash_command", { command, linkedRepository, hermesSession, confirmed });
}

export type FileManagerEntry = {
  name: string;
  path: string;
  kind: "directory" | "file" | "symlink" | "other";
  size?: number;
  modified?: string;
  hidden: boolean;
  readonly: boolean;
  childrenCount?: number;
};

export type FileManagerDirectoryListing = {
  path: string;
  name: string;
  parent?: string;
  entries: FileManagerEntry[];
};

export function listFileManagerDirectory(path?: string): Promise<FileManagerDirectoryListing> {
  return invoke<FileManagerDirectoryListing>("list_file_manager_directory", { path: path || null });
}
