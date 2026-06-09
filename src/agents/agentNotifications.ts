import { invoke } from "@tauri-apps/api/core";
import { isPermissionGranted, requestPermission, sendNotification } from "@tauri-apps/plugin-notification";

export const DEFAULT_AGENT_NOTIFICATION_EMAIL = "ziad.ahmed.25.25.25@gmail.com";

export type AgentNotificationMessage = {
  role: "user" | "assistant" | "system";
  content: string;
};

export type AgentNotificationSession = {
  id: string;
  title: string;
  messages: AgentNotificationMessage[];
};

export type AgentResponseEmailNotificationPayload = {
  to?: string;
  subject: string;
  summary: string;
  sessionTitle: string;
};

function compactText(value: string, limit: number) {
  const compact = value.replace(/```[\s\S]*?```/g, "[code block omitted]").replace(/\s+/g, " ").trim();
  if (compact.length <= limit) return compact;
  return `${compact.slice(0, Math.max(0, limit - 1)).trimEnd()}…`;
}

function latestUserMessageBefore(messages: AgentNotificationMessage[], assistantIndex: number) {
  for (let index = assistantIndex - 1; index >= 0; index -= 1) {
    if (messages[index]?.role === "user" && messages[index].content.trim()) return messages[index].content;
  }
  return "No user prompt found before this response.";
}

export function buildAgentResponseEmailSummary(session: AgentNotificationSession, assistantMessageIdOrContent: string) {
  const assistantIndex = session.messages.findIndex((message) => message.role === "assistant" && message.content === assistantMessageIdOrContent)
    >= 0
    ? session.messages.findIndex((message) => message.role === "assistant" && message.content === assistantMessageIdOrContent)
    : session.messages.map((message) => message.content).lastIndexOf(assistantMessageIdOrContent);
  const safeAssistantIndex = assistantIndex >= 0 ? assistantIndex : session.messages.length - 1;
  const assistantResponse = session.messages[safeAssistantIndex]?.content || assistantMessageIdOrContent;
  const latestUserPrompt = latestUserMessageBefore(session.messages, safeAssistantIndex);
  const recentContext = session.messages
    .slice(Math.max(0, safeAssistantIndex - 4), safeAssistantIndex + 1)
    .filter((message) => message.content.trim())
    .map((message) => `- ${message.role}: ${compactText(message.content, 220)}`)
    .join("\n");

  return [
    `Session: ${session.title || "Hermes session"}`,
    "Status: Hermes replied and may need your reply.",
    "",
    "Latest user message:",
    compactText(latestUserPrompt, 1000),
    "",
    "Hermes response summary:",
    compactText(assistantResponse, 1600),
    "",
    "Recent context:",
    recentContext || "- No recent context available.",
    "",
    "Open Zoid to continue the session.",
  ].join("\n");
}

export async function ensureDesktopNotificationPermission() {
  try {
    if (await isPermissionGranted()) return true;
    const permission = await requestPermission();
    return permission === "granted";
  } catch {
    return false;
  }
}

export async function sendDesktopAgentNotification(input: { sessionTitle: string; responsePreview: string }) {
  const granted = await ensureDesktopNotificationPermission();
  if (!granted) return false;
  try {
    sendNotification({
      title: `Hermes replied in ${input.sessionTitle || "a session"}`,
      body: compactText(input.responsePreview, 180),
    });
    return true;
  } catch {
    return false;
  }
}

export async function sendAgentResponseEmailNotification(payload: AgentResponseEmailNotificationPayload) {
  try {
    return await invoke<{ ok: boolean; message: string; sentAt: string }>("send_agent_response_email_notification", {
      request: {
        to: payload.to || DEFAULT_AGENT_NOTIFICATION_EMAIL,
        subject: payload.subject,
        summary: payload.summary,
        sessionTitle: payload.sessionTitle,
      },
    });
  } catch (error) {
    return { ok: false, message: error instanceof Error ? error.message : String(error), sentAt: new Date().toISOString() };
  }
}

export function agentResponseEmailSubject(sessionTitle: string) {
  return `Zoid: Hermes replied in ${sessionTitle || "a session"}`;
}

export function agentResponsePreview(content: string) {
  return compactText(content, 180);
}
