import { existsSync, readFileSync } from "node:fs";

const app = readFileSync(new URL("./App.tsx", import.meta.url), "utf8");
const css = readFileSync(new URL("./App.css", import.meta.url), "utf8");
const client = readFileSync(new URL("./agents/hermesClient.ts", import.meta.url), "utf8");
const screen = readFileSync(new URL("./agents/AgentsHermesScreen.tsx", import.meta.url), "utf8");
const backend = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");

if (!app.includes("Zoid 25")) {
  throw new Error("Zoid 25 brand label is missing");
}

if (!app.includes('aria-label="Primary navigation"')) {
  throw new Error("Primary navigation sidebar scaffold is missing");
}

if (!app.includes("blue-rail")) {
  throw new Error("Kujoyama-style blue rail is missing");
}

if (!app.includes('aria-current={item.label === "Agents" ? "page" : undefined}')) {
  throw new Error("Agents navigation row must be the active page");
}

if (!app.includes("AgentsHermesScreen")) {
  throw new Error("App must render the Hermes Agents screen");
}

if (!existsSync(new URL("./agents/participants.ts", import.meta.url))) {
  throw new Error("Hermes and user participants must be defined");
}

if (!existsSync(new URL("./agents/AgentsHermesScreen.tsx", import.meta.url))) {
  throw new Error("Agents Hermes screen must exist");
}

if (!css.includes("hermes-chat-shell")) {
  throw new Error("Hermes chat shell styling is missing");
}

if (!client.includes("check_hermes_cli") || !client.includes("send_hermes_cli_message")) {
  throw new Error("Frontend must invoke Hermes CLI bridge commands");
}

if (!backend.includes('Command::new') || !backend.includes('"hermes"')) {
  throw new Error("Backend must spawn the Hermes CLI, not call an HTTP API");
}

for (const forbidden of ["/v1/chat/completions", "/v1/models", "API_SERVER_KEY", "ZOID_HERMES_API_KEY", "API server"]) {
  if (backend.includes(forbidden) || screen.includes(forbidden)) {
    throw new Error(`Hermes API wording/path must not remain in active bridge: ${forbidden}`);
  }
}
