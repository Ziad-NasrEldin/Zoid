import { invoke } from "@tauri-apps/api/core";
import type { HermesChatRequestMessage, HermesChatResponse, HermesHealth } from "./types";

export async function getHermesHealth(): Promise<HermesHealth> {
  try {
    return await invoke<HermesHealth>("check_hermes_health");
  } catch (error) {
    return {
      ok: false,
      status: "error",
      message: error instanceof Error ? error.message : String(error),
    };
  }
}

export async function sendHermesMessage(
  messages: HermesChatRequestMessage[],
): Promise<HermesChatResponse> {
  return invoke<HermesChatResponse>("send_hermes_message", { messages });
}
