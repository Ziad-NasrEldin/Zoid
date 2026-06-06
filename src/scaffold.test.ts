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

if (!app.includes('useState<ActiveWorkspace>("Code")')) {
  throw new Error("Code workspace must be the default active page after importing the new feature");
}

if (!app.includes("AgentsHermesScreen")) {
  throw new Error("App must still render the Hermes Agents screen");
}

if (!app.includes('aria-label="Code workspace"') || !app.includes("empty-code-workspace")) {
  throw new Error("Code workspace must render as an empty page");
}

for (const removedCodeSurface of ["CodeWorkspaceFlow", "codeWorkspaceFlowView", "One guided flow", "Native local", "Browser preview"]) {
  if (app.includes(removedCodeSurface)) {
    throw new Error(`Code workspace page must be empty and not render old flow UI: ${removedCodeSurface}`);
  }
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

for (const requiredMetric of [
  "Context used:",
  "Compressions:",
  "Repository:",
  "Codex usage:",
  "Elapsed:",
  "Model:",
  "Session:",
]) {
  if (!screen.includes(requiredMetric)) {
    throw new Error(`Hermes stats strip is missing metric: ${requiredMetric}`);
  }
}

for (const removedMetric of ["<span>Messages:", "<span>Bridge:", "<span>Operator:"]) {
  if (screen.includes(removedMetric)) {
    throw new Error(`Old Hermes stats strip metric must be replaced: ${removedMetric}`);
  }
}

if (!client.includes("linkedRepository") || !screen.includes("sendHermesCliMessage(") || !screen.includes("linkedRepository")) {
  throw new Error("Linked repository must be passed into the Hermes send path");
}

if (!backend.includes("linked_repository") || !backend.includes("current_dir")) {
  throw new Error("Backend Hermes CLI command must receive and apply a linked repository workdir");
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
