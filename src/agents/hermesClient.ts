import { invoke } from "@tauri-apps/api/core";
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
): Promise<HermesCliResponse> {
  return invoke<HermesCliResponse>("send_hermes_cli_message", { messages });
}
