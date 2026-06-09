const RECENT_COMMANDS_KEY = "zoid25:recent-hermes-commands";
const MAX_RECENT_COMMANDS = 20;

function looksSensitive(command: string) {
  return command.length > 500 || /(api[_-]?key|token|secret|password|bearer\s+[a-z0-9._-]+)/i.test(command);
}

export function loadRecentCommands(): string[] {
  try {
    const parsed = JSON.parse(localStorage.getItem(RECENT_COMMANDS_KEY) || "[]");
    return Array.isArray(parsed) ? parsed.filter((item): item is string => typeof item === "string").slice(0, MAX_RECENT_COMMANDS) : [];
  } catch {
    return [];
  }
}

export function saveRecentCommand(command: string) {
  const trimmed = command.trim();
  if (!trimmed.startsWith("/") || looksSensitive(trimmed)) return;
  const next = [trimmed, ...loadRecentCommands().filter((item) => item !== trimmed)].slice(0, MAX_RECENT_COMMANDS);
  localStorage.setItem(RECENT_COMMANDS_KEY, JSON.stringify(next));
}
