export const AGENT_DASHBOARD_STORAGE_KEY = "zoid25:agents-dashboard";
export const AGENT_DASHBOARD_MAX_TILES = 4;

export type AgentDashboardLayoutMode = "auto" | "split-2" | "focus-stack" | "quad";

export type AgentDashboardStateV1 = {
  version: 1;
  tiledSessionIds: string[];
  primarySessionId?: string;
  focusedSessionId?: string;
  layoutMode: AgentDashboardLayoutMode;
  autoPrioritize: boolean;
};

const layoutModes = new Set<AgentDashboardLayoutMode>(["auto", "split-2", "focus-stack", "quad"]);

export const defaultAgentDashboardState: AgentDashboardStateV1 = {
  version: 1,
  tiledSessionIds: [],
  layoutMode: "auto",
  autoPrioritize: false,
};

export function sanitizeAgentDashboardState(value: unknown, validSessionIds: readonly string[]): AgentDashboardStateV1 {
  const validIds = new Set(validSessionIds);
  if (!value || typeof value !== "object") return { ...defaultAgentDashboardState };
  const candidate = value as Partial<AgentDashboardStateV1> & { version?: unknown };
  if (candidate.version !== 1) return { ...defaultAgentDashboardState };
  const seen = new Set<string>();
  const tiledSessionIds = (Array.isArray(candidate.tiledSessionIds) ? candidate.tiledSessionIds : [])
    .filter((id): id is string => typeof id === "string" && validIds.has(id))
    .filter((id) => {
      if (seen.has(id)) return false;
      seen.add(id);
      return true;
    })
    .slice(0, AGENT_DASHBOARD_MAX_TILES);
  const primarySessionId = typeof candidate.primarySessionId === "string" && tiledSessionIds.includes(candidate.primarySessionId)
    ? candidate.primarySessionId
    : tiledSessionIds[0];
  const focusedSessionId = typeof candidate.focusedSessionId === "string" && tiledSessionIds.includes(candidate.focusedSessionId)
    ? candidate.focusedSessionId
    : primarySessionId;
  return {
    version: 1,
    tiledSessionIds,
    primarySessionId,
    focusedSessionId,
    layoutMode: layoutModes.has(candidate.layoutMode as AgentDashboardLayoutMode) ? candidate.layoutMode as AgentDashboardLayoutMode : "auto",
    autoPrioritize: candidate.autoPrioritize === true,
  };
}

export function loadAgentDashboardState(validSessionIds: readonly string[], storage: Storage | undefined = typeof window === "undefined" ? undefined : window.localStorage): AgentDashboardStateV1 {
  if (!storage) return sanitizeAgentDashboardState(defaultAgentDashboardState, validSessionIds);
  try {
    const raw = storage.getItem(AGENT_DASHBOARD_STORAGE_KEY);
    return sanitizeAgentDashboardState(raw ? JSON.parse(raw) : defaultAgentDashboardState, validSessionIds);
  } catch {
    return sanitizeAgentDashboardState(defaultAgentDashboardState, validSessionIds);
  }
}

export function saveAgentDashboardState(state: AgentDashboardStateV1, storage: Storage | undefined = typeof window === "undefined" ? undefined : window.localStorage) {
  if (!storage) return;
  storage.setItem(AGENT_DASHBOARD_STORAGE_KEY, JSON.stringify(state));
}
