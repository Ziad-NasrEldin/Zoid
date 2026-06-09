import type { HermesChatSession } from "./sessionState";

const MAX_CONTINUATION_BRIEF_CHARS = 3_200;
const PATH_PATTERN = /(?:^|\s)(\/?(?:[\w.-]+\/)+[\w.@:%+~#=,;()/-]+)/g;

function clip(value: string, max: number) {
  const compact = value.replace(/\s+/g, " ").trim();
  return compact.length > max ? `${compact.slice(0, max - 1)}…` : compact;
}

export function buildContinuationBrief(session: HermesChatSession): string {
  const userMessages = session.messages.filter((message) => message.role === "user");
  const assistantMessages = session.messages.filter((message) => message.role === "assistant" && message.content.trim());
  const latestUserGoal = userMessages[userMessages.length - 1]?.content ?? "No prior user goal was found in this Zoid session.";
  const lastAssistantState = assistantMessages[assistantMessages.length - 1]?.content ?? "No prior assistant result was found.";
  const recentMessages = session.messages
    .filter((message) => message.content.trim())
    .slice(-6)
    .map((message) => `${message.role}: ${clip(message.content, 360)}`)
    .join("\n");
  const paths = Array.from(new Set(session.messages.flatMap((message) => Array.from(message.content.matchAll(PATH_PATTERN), (match) => match[1])))).slice(0, 8);

  const brief = [
    "Continue this same session. Do not switch tasks or assume a different session.",
    `Session title: ${session.title}`,
    `Latest user goal: ${clip(latestUserGoal, 700)}`,
    `Last assistant state/result: ${clip(lastAssistantState, 700)}`,
    paths.length ? `Files/repos mentioned in this session only: ${paths.join(", ")}` : "Files/repos mentioned in this session only: none detected.",
    "Recent session context:",
    recentMessages || "No usable recent messages beyond the session title.",
    "Proceed from the available context, preserve unresolved TODOs, and ask only if continuation is blocked.",
  ].join("\n\n");
  return brief.length > MAX_CONTINUATION_BRIEF_CHARS ? `${brief.slice(0, MAX_CONTINUATION_BRIEF_CHARS - 1)}…` : brief;
}
