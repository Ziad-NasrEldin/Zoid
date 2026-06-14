import { useEffect, useMemo, useRef, useState } from "react";
import { Archive, BellDot, ChevronDown, ChevronRight, FileText, Folder, FolderTree, Maximize2, Minimize2, Plus, X } from "lucide-react";
import type { CSSProperties, Dispatch, DragEvent as ReactDragEvent, KeyboardEvent as ReactKeyboardEvent, PointerEvent as ReactPointerEvent, ReactNode, SetStateAction } from "react";
import { flushSync } from "react-dom";
import { listen } from "@tauri-apps/api/event";
import { ChatComposer, type ChatComposerHandle } from "./ChatComposer";
import { MessageBubble } from "./MessageBubble";
import { GlobalDropdown } from "../ui/GlobalDropdown";
import { getHermesCliStatus, sendHermesCliRunMessage, cancelHermesCliRun, listHermesSlashCommandRegistry, executeHermesSlashCommand, listFileManagerDirectory, type FileManagerDirectoryListing, type FileManagerEntry } from "./hermesClient";
import { CommandPalette } from "./CommandPalette";
import type { HermesCommandPanel, HermesSlashCommand, HermesSlashCommandExecution } from "./hermesCommands";
import { parseSlashCommand } from "./slashCommandParser";
import { loadRecentCommands, saveRecentCommand } from "./recentCommands";
import { participantsById } from "./participants";
import { getSessionAgentAvatar } from "./sessionPortraits";
import { agentResponsePreview, sendDesktopAgentNotification } from "./agentNotifications";
import { buildRuthlessReviewerPrompt } from "./ruthlessReviewerAgent";
import { createSession } from "./sessionState";
import type { HermesChatSession } from "./sessionState";
import { AgentMonitorPanel } from "./AgentMonitorPanel";
import { buildContinuationBrief } from "./continuationBrief";
import { AGENT_DASHBOARD_MAX_TILES, applyDraggedSessionToDashboard, loadAgentDashboardState, saveAgentDashboardState, type AgentDashboardStateV1 } from "./dashboardLayoutState";
import { MAX_ACTIVE_AGENT_RUNS, listAgentRuns, useAgentRuntime, type AgentRunEvent } from "./useAgentRuntime";
import { defaultHermesProfileSettings, loadHermesProfileSettings, saveHermesProfileSettings } from "./hermesProfileClient";
import type { HermesProfileSettings } from "./hermesProfileClient";
import type { AgentConnectionState, ChatMessage, HermesCliStatus } from "./types";
import type { CodeRepository } from "../code/types";
export { createSession, HERMES_LEGACY_WELCOME_COPY, HERMES_WELCOME_COPY, refreshHermesWelcomeCopy } from "./sessionState";
export type { ArchivedHermesChatSession, HermesChatSession } from "./sessionState";
// Session state is split for code-splitting; HermesChatSession still includes needsReply?: boolean.

function titleFromPrompt(prompt: string) {
  const compact = prompt.replace(/\s+/g, " ").trim();
  if (!compact) return "New session";
  return compact.length > 44 ? `${compact.slice(0, 41)}...` : compact;
}

function statusTone(state: AgentConnectionState) {
  if (state === "online") return "ready";
  if (state === "checking") return "idle";
  return "blocked";
}

const HERMES_CONTEXT_LIMIT = 200_000;
const CODEX_USAGE_TODAY = "5h";
const CODEX_USAGE_WEEKLY = "5h / week";
const ACTIVE_MODEL = "gpt-5.5";
const CONNECTION_STATE_JAPANESE: Record<AgentConnectionState, string> = {
  checking: "確認中",
  online: "接続中",
  offline: "未接続",
  error: "エラー",
};
const SESSIONS_RAIL_MIN_WIDTH = 124;
const SESSIONS_RAIL_MAX_WIDTH = 420;
const SESSIONS_RAIL_DEFAULT_WIDTH = 184;
const SESSIONS_RAIL_COMPACT_WIDTH = 68;
const SESSIONS_RAIL_WIDTH_STORAGE_KEY = "zoid25:hermes-sessions-rail-width";
const SESSIONS_RAIL_COMPACT_STORAGE_KEY = "zoid25:hermes-sessions-rail-compact-polished-2";
const FILE_MANAGER_MIN_WIDTH = 240;

function stripHermesRunMetadata(chunk: string) {
  return chunk
    .split("\n")
    .filter((line) => {
      const trimmed = line.trim();
      return trimmed.length > 0
        && !trimmed.startsWith("Session ID:")
        && !trimmed.startsWith("session_id:")
        && !trimmed.startsWith("Cost:")
        && !trimmed.startsWith("Tokens:")
        && !trimmed.startsWith("Provider:")
        && !trimmed.startsWith("Warning:");
    })
    .join("\n");
}
const FILE_MANAGER_MAX_WIDTH = 520;
const FILE_MANAGER_DEFAULT_WIDTH = 336;
const FILE_MANAGER_WIDTH_STORAGE_KEY = "zoid25:hermes-file-manager-width";
const SESSION_DASHBOARD_DRAG_TYPE = "application/x-zoid-hermes-session";
const SESSIONS_RAIL_MORPH_TIMING: KeyframeAnimationOptions = {
  duration: 540,
  easing: "cubic-bezier(0.16, 1, 0.3, 1)",
};
const SESSIONS_RAIL_MORPH_EXIT_TIMING: KeyframeAnimationOptions = {
  duration: 240,
  easing: "cubic-bezier(0.25, 1, 0.5, 1)",
};

type SessionsRailMorphSnapshot = {
  clone: HTMLElement;
  key: string | null;
  rect: DOMRect;
};


function clampSessionsRailWidth(width: number) {
  if (!Number.isFinite(width)) return SESSIONS_RAIL_DEFAULT_WIDTH;
  return Math.min(SESSIONS_RAIL_MAX_WIDTH, Math.max(SESSIONS_RAIL_MIN_WIDTH, Math.round(width)));
}

function clampFileManagerWidth(width: number) {
  if (!Number.isFinite(width)) return FILE_MANAGER_DEFAULT_WIDTH;
  return Math.min(FILE_MANAGER_MAX_WIDTH, Math.max(FILE_MANAGER_MIN_WIDTH, Math.round(width)));
}

function getInitialSessionsRailWidth() {
  if (typeof window === "undefined") return SESSIONS_RAIL_DEFAULT_WIDTH;
  const storedWidth = window.localStorage.getItem(SESSIONS_RAIL_WIDTH_STORAGE_KEY);
  return storedWidth ? clampSessionsRailWidth(Number(storedWidth)) : SESSIONS_RAIL_DEFAULT_WIDTH;
}

function getInitialSessionsRailCompact() {
  if (typeof window === "undefined") return false;
  return window.localStorage.getItem(SESSIONS_RAIL_COMPACT_STORAGE_KEY) === "true";
}

function getInitialFileManagerWidth() {
  if (typeof window === "undefined") return FILE_MANAGER_DEFAULT_WIDTH;
  const storedWidth = window.localStorage.getItem(FILE_MANAGER_WIDTH_STORAGE_KEY);
  return storedWidth ? clampFileManagerWidth(Number(storedWidth)) : FILE_MANAGER_DEFAULT_WIDTH;
}


function estimateContextUsed(messages: ChatMessage[]) {
  const approximateTokens = messages.reduce((total, message) => total + Math.ceil(message.content.length / 4), 0);
  return Math.min(100, Math.max(1, Math.round((approximateTokens / HERMES_CONTEXT_LIMIT) * 100)));
}

function formatElapsed(milliseconds: number | null) {
  if (milliseconds === null) return "idle";
  if (milliseconds < 1_000) return `${milliseconds}ms`;
  const seconds = Math.round(milliseconds / 1_000);
  if (seconds < 60) return `${seconds}s`;
  const minutes = Math.floor(seconds / 60);
  const remainingSeconds = seconds % 60;
  return `${minutes}m ${remainingSeconds}s`;
}

function elapsedBetween(startedAt?: string, finishedAt?: string, now = Date.now()) {
  if (!startedAt) return null;
  const start = Date.parse(startedAt);
  const finish = finishedAt ? Date.parse(finishedAt) : now;
  if (!Number.isFinite(start) || !Number.isFinite(finish)) return null;
  return Math.max(0, finish - start);
}

function renderNumberAccents(value: string | number): ReactNode[] {
  return String(value).split(/(\d+(?:\.\d+)?(?:\s*[%a-zA-Z]+)?)/g).map((part, index) => (
    /^\d/.test(part) ? <span className="stat-number-accent" key={`${part}-${index}`}>{part}</span> : part
  ));
}


function repositoryLabel(repository?: CodeRepository) {
  if (!repository) return "Unlinked repository";
  return `${repository.name} · ${repository.branch || "unknown"}`;
}

const MIN_REPOSITORY_NAME_DETECTION_LENGTH = 3;

function escapeRegExp(value: string) {
  return value.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
}

function promptContainsRepositoryName(prompt: string, repositoryName: string) {
  const normalizedName = repositoryName.trim().toLowerCase();
  if (normalizedName.length < MIN_REPOSITORY_NAME_DETECTION_LENGTH) return false;
  const explicitNamePattern = new RegExp(`(^|[^a-z0-9_-])${escapeRegExp(normalizedName)}($|[^a-z0-9_-])`, "i");
  return explicitNamePattern.test(prompt);
}

function promptContainsRepositoryPath(prompt: string, value?: string | null) {
  const normalizedValue = value?.trim().toLowerCase();
  if (!normalizedValue || normalizedValue.length < 6) return false;
  return prompt.includes(normalizedValue);
}

function detectRepositoryFromPrompt(prompt: string, repositories: CodeRepository[]) {
  const normalizedPrompt = prompt.toLowerCase();
  return repositories.find((repository) => (
    promptContainsRepositoryPath(normalizedPrompt, repository.path) ||
    promptContainsRepositoryPath(normalizedPrompt, repository.remoteUrl) ||
    promptContainsRepositoryName(normalizedPrompt, repository.name)
  ));
}

function sessionAgeShade(session: HermesChatSession, sessions: HermesChatSession[]) {
  const timestamps = sessions
    .map((item) => Date.parse(item.createdAt))
    .filter((value) => Number.isFinite(value))
    .sort((left, right) => right - left);
  const current = Date.parse(session.createdAt);
  const rank = timestamps.findIndex((value) => value === current);
  const ageRank = rank < 0 ? 0 : rank;
  const alpha = Math.min(0.22, 0.035 + ageRank * 0.028);
  return `rgba(13, 10, 10, ${alpha.toFixed(3)})`;
}

function sessionPortraitStyle(session: HermesChatSession, sessions: HermesChatSession[]) {
  const portrait = getSessionAgentAvatar(session.id, session.portraitId);
  return {
    "--session-age-shade": sessionAgeShade(session, sessions),
    "--session-portrait": `url("${portrait.asset}")`,
    "--session-portrait-accent": portrait.accent,
    "--session-portrait-paper": portrait.paper,
    "--session-portrait-focal-point": portrait.focalPoint,
  } as CSSProperties;
}

function clearSessionNeedsReply(session: HermesChatSession): HermesChatSession {
  return session.needsReply ? { ...session, needsReply: false } : session;
}

type AgentsHermesScreenProps = {
  repositories?: CodeRepository[];
  sessions: HermesChatSession[];
  activeSessionId: string;
  isAgentsWorkspaceOpen?: boolean;
  onSessionsChange: Dispatch<SetStateAction<HermesChatSession[]>>;
  onActiveSessionIdChange: (sessionId: string) => void;
  onArchiveSession: (sessionId: string) => void;
  onRepositoryOperationComplete?: (result: {
    sessionId: string;
    runId: string;
    repositoryId: string;
    action: NonNullable<HermesChatSession["operationAction"]>;
    outcome: "success" | "failed" | "cancelled" | "needs-user" | "blocked";
    responseContent: string;
  }) => void;
};

type PendingCommandConfirmation = {
  result: HermesSlashCommandExecution;
  sessionId: string;
  assistantId: string;
  command: string;
  linkedRepositoryPath?: string;
  hermesCliSessionId?: string;
};

type ModelPanelDraft = {
  provider: string;
  model: string;
  reasoning: string;
};

const REASONING_OPTIONS = ["off", "minimal", "low", "medium", "high", "xhigh"].map((level) => ({ value: level, label: level }));

const COMMAND_PANEL_COPY: Record<HermesCommandPanel, { title: string; body: string }> = {
  model: { title: "Model controls", body: "Review or change the active session model and reasoning settings." },
  tools: { title: "Tool controls", body: "Inspect or change Hermes tools and toolsets for the active runtime." },
  skills: { title: "Skill library", body: "Browse, load, or refresh Hermes skills." },
  cron: { title: "Automation jobs", body: "Inspect Hermes cron jobs and automation actions." },
  agents: { title: "Agent tasks", body: "Inspect background tasks, queues, steering, and agent runs." },
  profile: { title: "Profile settings", body: "Inspect or change Hermes profile-level settings." },
  history: { title: "Session history", body: "Work with Zoid sessions and Hermes state history." },
  usage: { title: "Usage and status", body: "Inspect runtime status, usage, and insights." },
  debug: { title: "Debug tools", body: "View debug output and runtime diagnostics." },
  browser: { title: "Browser tools", body: "Inspect or control browser-related Hermes commands." },
};

type QueuedHermesPrompt = { sessionId: string; content: string; kind: "prompt" | "slash" };
type DashboardPointerDragState = { sessionId: string; startX: number; startY: number; isDragging: boolean };

export function AgentsHermesScreen({ repositories = [], sessions, activeSessionId, isAgentsWorkspaceOpen = true, onSessionsChange, onActiveSessionIdChange, onArchiveSession, onRepositoryOperationComplete }: AgentsHermesScreenProps) {
  const [cliStatus, setCliStatus] = useState<HermesCliStatus | null>(null);
  const [connectionState, setConnectionState] = useState<AgentConnectionState>("checking");
  const [isSending, setIsSending] = useState(false);
  const activeHermesRunsRef = useRef<Map<string, { sessionId: string; assistantId: string; stopCopy?: string }>>(new Map());
  const agentRunEventSequenceRef = useRef<Map<string, number>>(new Map());
  const terminalAgentRunEventsRef = useRef<Set<string>>(new Set());
  const [lastPromptStartedAt, setLastPromptStartedAt] = useState<number | null>(null);
  const [elapsedTick, setElapsedTick] = useState(Date.now());
  const [sessionsRailWidth, setSessionsRailWidth] = useState(getInitialSessionsRailWidth);
  const [isSessionsRailCompact, setIsSessionsRailCompact] = useState(getInitialSessionsRailCompact);
  const [editingSessionId, setEditingSessionId] = useState<string | null>(null);
  const [editingSessionTitle, setEditingSessionTitle] = useState("");
  const [slashCommands, setSlashCommands] = useState<HermesSlashCommand[]>([]);
  const [slashCommandSource, setSlashCommandSource] = useState<"live" | "fallback" | "unavailable">("unavailable");
  const [commandPaletteOpen, setCommandPaletteOpen] = useState(false);
  const [recentCommands, setRecentCommands] = useState<string[]>(() => loadRecentCommands());
  const [pendingConfirmation, setPendingConfirmation] = useState<PendingCommandConfirmation | null>(null);
  const [activeCommandPanel, setActiveCommandPanel] = useState<HermesCommandPanel | null>(null);
  const [profileSettings, setProfileSettings] = useState<HermesProfileSettings | null>(null);
  const [modelPanelDraft, setModelPanelDraft] = useState<ModelPanelDraft>(() => ({
    provider: defaultHermesProfileSettings.modelProvider,
    model: defaultHermesProfileSettings.modelName,
    reasoning: defaultHermesProfileSettings.reasoningEffort,
  }));
  const [modelPanelStatus, setModelPanelStatus] = useState<string | null>(null);
  const [modelPanelError, setModelPanelError] = useState<string | null>(null);
  const [isUpdatingModelPanel, setIsUpdatingModelPanel] = useState(false);
  const queuedHermesPromptsRef = useRef<QueuedHermesPrompt[]>([]);
  const claimedInitialPromptKeysRef = useRef<Set<string>>(new Set());
  const [fileManagerOpen, setFileManagerOpen] = useState(false);
  const [fileManagerVisible, setFileManagerVisible] = useState(false);
  const [fileManagerWidth, setFileManagerWidth] = useState(getInitialFileManagerWidth);
  const [isFileManagerResizing, setIsFileManagerResizing] = useState(false);
  const [fileManagerRootPath, setFileManagerRootPath] = useState<string | null>(null);
  const [fileManagerListings, setFileManagerListings] = useState<Record<string, FileManagerDirectoryListing>>({});
  const [expandedFilePaths, setExpandedFilePaths] = useState<Set<string>>(() => new Set());
  const [fileManagerLoadingPath, setFileManagerLoadingPath] = useState<string | null>(null);
  const [fileManagerError, setFileManagerError] = useState<string | null>(null);
  const composerRef = useRef<ChatComposerHandle>(null);
  const messageListRef = useRef<HTMLDivElement>(null);
  const sessionsListRef = useRef<HTMLDivElement>(null);
  const chatWorkspaceRef = useRef<HTMLDivElement>(null);
  const fileManagerResizeFrameRef = useRef<number | null>(null);
  const chatDashboardDropRef = useRef<HTMLDivElement>(null);
  const activeCommandPanelRef = useRef<HTMLElement>(null);
  const pendingConfirmationRef = useRef<HTMLElement>(null);
  const sessionsRef = useRef(sessions);
  const activeSessionIdRef = useRef(activeSessionId);
  const isAgentsWorkspaceOpenRef = useRef(isAgentsWorkspaceOpen);
  const sessionsRailMorphAnimationsRef = useRef<Animation[]>([]);
  const pendingNewSessionActivationRef = useRef<string | null>(null);
  const dashboardPointerDragRef = useRef<DashboardPointerDragState | null>(null);
  const lastDashboardNativeDragPointRef = useRef<{ clientX: number; clientY: number } | null>(null);
  const suppressNextSessionClickRef = useRef(false);
  const activeDashboardDragSessionIdRef = useRef<string | null>(null);
  const dashboardStateRef = useRef<AgentDashboardStateV1 | null>(null);
  const [showSessionsOverflowCue, setShowSessionsOverflowCue] = useState(false);
  const [dashboardState, setDashboardState] = useState<AgentDashboardStateV1>(() => loadAgentDashboardState(sessions.map((session) => session.id)));
  const [expandedModeSessionId, setExpandedModeSessionId] = useState<string | null>(null);
  const [draggedDashboardSessionId, setDraggedDashboardSessionId] = useState<string | null>(null);
  const [isChatDropArmed, setIsChatDropArmed] = useState(false);
  const [dashboardDropCue, setDashboardDropCue] = useState("Drop to split chat");
  const runtime = useAgentRuntime();

  useEffect(() => { sessionsRef.current = sessions; }, [sessions]);
  useEffect(() => { dashboardStateRef.current = dashboardState; }, [dashboardState]);
  useEffect(() => {
    setDashboardState((current) => loadAgentDashboardState(sessions.map((session) => session.id), { getItem: () => JSON.stringify(current), setItem: () => undefined, removeItem: () => undefined, clear: () => undefined, key: () => null, length: 0 }));
  }, [sessions]);
  useEffect(() => { saveAgentDashboardState(dashboardState); }, [dashboardState]);
  useEffect(() => { activeSessionIdRef.current = activeSessionId; }, [activeSessionId]);
  useEffect(() => {
    const pendingSessionId = pendingNewSessionActivationRef.current;
    if (!pendingSessionId || !sessions.some((session) => session.id === pendingSessionId)) return;
    pendingNewSessionActivationRef.current = null;
    onActiveSessionIdChange(pendingSessionId);
  }, [sessions, onActiveSessionIdChange]);
  useEffect(() => {
    isAgentsWorkspaceOpenRef.current = isAgentsWorkspaceOpen;
    return () => { isAgentsWorkspaceOpenRef.current = false; };
  }, [isAgentsWorkspaceOpen]);

  useEffect(() => {
    let active = true;
    getHermesCliStatus().then((result) => {
      if (!active) return;
      setCliStatus(result);
      setConnectionState(result.status);
    });
    listHermesSlashCommandRegistry().then((registry) => {
      if (!active) return;
      setSlashCommands(registry.commands);
      setSlashCommandSource(registry.source);
    }).catch(() => {
      if (!active) return;
      setSlashCommands([]);
      setSlashCommandSource("unavailable");
    });
    return () => { active = false; };
  }, []);

  useEffect(() => {
    let active = true;
    loadHermesProfileSettings()
      .then((loaded) => {
        if (!active) return;
        setProfileSettings(loaded);
        setModelPanelDraft({ provider: loaded.modelProvider, model: loaded.modelName, reasoning: loaded.reasoningEffort });
        setModelPanelError(null);
      })
      .catch((error) => {
        if (!active) return;
        setModelPanelError(`Model settings unavailable: ${error instanceof Error ? error.message : String(error)}`);
      });
    return () => { active = false; };
  }, []);

  useEffect(() => {
    function hasActiveTextSelection() {
      const target = document.activeElement;
      if (target instanceof HTMLTextAreaElement || target instanceof HTMLInputElement) {
        return target.selectionStart !== null && target.selectionEnd !== null && target.selectionStart !== target.selectionEnd;
      }
      return Boolean(window.getSelection()?.toString());
    }

    function handleGlobalKeyDown(event: KeyboardEvent) {
      if (event.defaultPrevented) return;
      if ((event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "k") {
        event.preventDefault();
        setCommandPaletteOpen(true);
        return;
      }
      if (activeHermesRunsRef.current.size > 0 && (event.metaKey || event.ctrlKey) && event.key.toLowerCase() === "c" && !hasActiveTextSelection()) {
        event.preventDefault();
        void handleStopHermesRun("keyboard", dashboardState.focusedSessionId ?? activeSessionIdRef.current);
      }
    }
    window.addEventListener("keydown", handleGlobalKeyDown);
    return () => window.removeEventListener("keydown", handleGlobalKeyDown);
  }, [dashboardState.focusedSessionId]);

  useEffect(() => {
    if (sessions.length === 0 && (!isSending || lastPromptStartedAt === null)) return undefined;
    const interval = window.setInterval(() => setElapsedTick(Date.now()), 500);
    return () => window.clearInterval(interval);
  }, [activeSessionId, sessions.length, isSending, lastPromptStartedAt]);

  useEffect(() => {
    if (!activeCommandPanel) return;
    window.requestAnimationFrame(() => activeCommandPanelRef.current?.focus());
  }, [activeCommandPanel]);

  useEffect(() => {
    if (!pendingConfirmation) return;
    window.requestAnimationFrame(() => pendingConfirmationRef.current?.focus());
  }, [pendingConfirmation]);

  useEffect(() => { window.localStorage.setItem(SESSIONS_RAIL_WIDTH_STORAGE_KEY, String(sessionsRailWidth)); }, [sessionsRailWidth]);
  useEffect(() => { window.localStorage.setItem(SESSIONS_RAIL_COMPACT_STORAGE_KEY, String(isSessionsRailCompact)); }, [isSessionsRailCompact]);
  useEffect(() => { window.localStorage.setItem(FILE_MANAGER_WIDTH_STORAGE_KEY, String(fileManagerWidth)); }, [fileManagerWidth]);

  function updateSessionsOverflowCue() {
    const list = sessionsListRef.current;
    if (!list) {
      setShowSessionsOverflowCue(false);
      return;
    }
    const hasContentBelow = list.clientHeight > 0 && list.scrollHeight > list.clientHeight + 1;
    const isAtTop = list.scrollTop <= 4;
    setShowSessionsOverflowCue(hasContentBelow && isAtTop);
  }

  useEffect(() => {
    updateSessionsOverflowCue();
    const list = sessionsListRef.current;
    if (!list) return undefined;
    const handleScroll = () => updateSessionsOverflowCue();
    list.addEventListener("scroll", handleScroll, { passive: true });
    window.addEventListener("resize", handleScroll);
    const animationFrame = window.requestAnimationFrame(updateSessionsOverflowCue);
    const resizeObserver = typeof ResizeObserver === "undefined" ? null : new ResizeObserver(updateSessionsOverflowCue);
    resizeObserver?.observe(list);
    return () => {
      list.removeEventListener("scroll", handleScroll);
      window.removeEventListener("resize", handleScroll);
      window.cancelAnimationFrame(animationFrame);
      resizeObserver?.disconnect();
    };
  }, [sessions.length, sessionsRailWidth, isSessionsRailCompact]);

  function handleSessionsOverflowCueClick() {
    const list = sessionsListRef.current;
    if (!list) return;
    list.scrollTo({ top: Math.min(list.scrollHeight, list.clientHeight * 0.7), behavior: "smooth" });
    setShowSessionsOverflowCue(false);
  }

  const activeSession = sessions.find((session) => session.id === activeSessionId) ?? sessions[0];
  const messages = activeSession?.messages ?? [];
  const activeRepositoryId = activeSession?.linkedRepositoryId ?? "none";
  const selectedRepository = repositories.find((repository) => repository.id === activeRepositoryId);

  useEffect(() => {
    if (!activeSessionId) return;
    updateSession(activeSessionId, clearSessionNeedsReply);
  }, [activeSessionId]);

  useEffect(() => {
    if (!activeSession || !activeSession.pendingInitialPrompt || connectionState !== "online" || !runtime.canStartSessionRun(activeSession.id)) return;
    const prompt = activeSession.pendingInitialPrompt;
    const claimKey = `${activeSession.id}:${activeSession.operationRunId ?? "no-run"}:${prompt}`;
    if (claimedInitialPromptKeysRef.current.has(claimKey)) return;
    claimedInitialPromptKeysRef.current.add(claimKey);
    updateSession(activeSession.id, (session) => ({ ...session, pendingInitialPrompt: undefined, updatedAt: new Date().toISOString() }));
    window.setTimeout(() => {
      const session = sessionsRef.current.find((candidate) => candidate.id === activeSession.id);
      if (!session) {
        claimedInitialPromptKeysRef.current.delete(claimKey);
        return;
      }
      void sendHermesPrompt({ ...session, pendingInitialPrompt: undefined }, prompt);
    }, 0);
  }, [activeSession?.id, activeSession?.operationRunId, activeSession?.pendingInitialPrompt, connectionState, runtime]);

  const hermesWithPresence = useMemo(() => {
    const base = participantsById.hermes;
    return { ...base, presence: isSending ? "thinking" as const : connectionState === "online" ? "online" as const : "offline" as const };
  }, [connectionState, isSending]);

  const disabledReason = connectionState === "online" ? undefined : "Hermes CLI is not reachable. Install Hermes or set ZOID_HERMES_CLI to the Hermes executable path.";
  const contextUsedPercent = estimateContextUsed(messages);
  const compressionCount = 0;
  const activeRuntime = activeSession ? runtime.getSessionRuntime(activeSession.id) : undefined;
  const promptElapsed = activeRuntime?.currentRun
    ? elapsedBetween(activeRuntime.currentRun.startedAt, undefined, elapsedTick)
    : elapsedBetween(activeRuntime?.lastStartedAt, activeRuntime?.lastFinishedAt, elapsedTick);
  const activeTaskStatus = activeRuntime?.status ?? (connectionState === "online" ? "idle" : connectionState);
  const activeSessionStartedAt = activeSession ? Date.parse(activeSession.createdAt) : Number.NaN;
  const sessionElapsed = Number.isFinite(activeSessionStartedAt) ? elapsedTick - activeSessionStartedAt : null;
  const chatWorkspaceStyle = {
    "--sessions-rail-width": `${isSessionsRailCompact ? SESSIONS_RAIL_COMPACT_WIDTH : sessionsRailWidth}px`,
    "--file-manager-width": `${fileManagerWidth}px`,
  } as CSSProperties;
  const fileManagerRootListing = fileManagerRootPath ? fileManagerListings[fileManagerRootPath] : undefined;
  const availableModelOptions = profileSettings?.availableModels ?? defaultHermesProfileSettings.availableModels;
  const providerOptions = Array.from(new Set([...Object.keys(availableModelOptions), modelPanelDraft.provider].filter(Boolean))).map((provider) => ({ value: provider, label: provider }));
  const modelOptions = Array.from(new Set([...(availableModelOptions[modelPanelDraft.provider] ?? []), modelPanelDraft.model].filter(Boolean))).map((model) => ({ value: model, label: model }));
  function effectiveModelInfoForSession(session?: HermesChatSession) {
    return {
      modelName: session?.modelName || profileSettings?.modelName || modelPanelDraft.model || ACTIVE_MODEL,
      reasoningEffort: session?.reasoningEffort || profileSettings?.reasoningEffort || modelPanelDraft.reasoning,
    };
  }
  const activeModelInfo = effectiveModelInfoForSession(activeSession);
  const activeModelLabel = activeModelInfo.modelName;
  const activeReasoningLabel = activeModelInfo.reasoningEffort;
  const modelPanelStorageLabel = profileSettings?.storagePath ?? "Hermes profile settings";

  async function updateModelPanelSettings(nextDraft: ModelPanelDraft) {
    if (!profileSettings) {
      setModelPanelError("Model settings are still loading; try again in a moment.");
      return;
    }
    const previousDraft = modelPanelDraft;
    setModelPanelDraft(nextDraft);
    setModelPanelError(null);
    setModelPanelStatus("Updating…");
    setIsUpdatingModelPanel(true);
    try {
      const saved = await saveHermesProfileSettings({
        ...profileSettings,
        modelProvider: nextDraft.provider,
        modelName: nextDraft.model,
        reasoningEffort: nextDraft.reasoning,
      });
      setProfileSettings(saved);
      setModelPanelDraft({ provider: saved.modelProvider, model: saved.modelName, reasoning: saved.reasoningEffort });
      const targetSessionId = activeSessionIdRef.current;
      if (targetSessionId) {
        updateSession(targetSessionId, (session) => ({
          ...session,
          modelName: saved.modelName,
          reasoningEffort: saved.reasoningEffort,
          updatedAt: new Date().toISOString(),
        }));
      }
      setModelPanelStatus(`Updated · ${saved.storagePath}`);
    } catch (error) {
      setModelPanelDraft(previousDraft);
      setModelPanelError(`Update failed: ${error instanceof Error ? error.message : String(error)}`);
      setModelPanelStatus(null);
    } finally {
      setIsUpdatingModelPanel(false);
    }
  }

  function handleModelProviderChange(nextProvider: string) {
    const providerModels = availableModelOptions[nextProvider] ?? [];
    const nextModel = providerModels.includes(modelPanelDraft.model) ? modelPanelDraft.model : providerModels[0] ?? modelPanelDraft.model;
    void updateModelPanelSettings({ ...modelPanelDraft, provider: nextProvider, model: nextModel });
  }

  function handleModelNameChange(nextModel: string) {
    void updateModelPanelSettings({ ...modelPanelDraft, model: nextModel });
  }

  function handleReasoningChange(nextReasoning: string) {
    void updateModelPanelSettings({ ...modelPanelDraft, reasoning: nextReasoning });
  }

  useEffect(() => {
    const list = messageListRef.current;
    if (!list) return;
    list.scrollTo({ top: list.scrollHeight, behavior: "smooth" });
  }, [activeSessionId, messages.length, messages[messages.length - 1]?.content, messages[messages.length - 1]?.status]);

  function updateSession(sessionId: string, updater: (session: HermesChatSession) => HermesChatSession) {
    onSessionsChange((current) => current.map((session) => session.id === sessionId ? updater(session) : session));
  }

  function clearActiveHermesRunIfCurrent(sessionId: string, assistantId: string) {
    const activeRun = activeHermesRunsRef.current.get(sessionId);
    if (activeRun?.assistantId !== assistantId) return false;
    activeHermesRunsRef.current.delete(sessionId);
    return true;
  }

  useEffect(() => {
    let cancelled = false;
    void listAgentRuns().then((runs) => {
      if (cancelled) return;
      for (const run of runs) {
        runtime.markSessionRunStarted(run.sessionId, run.runId, run.runId, run.startedAt);
      }
    }).catch(() => undefined);
    return () => { cancelled = true; };
  }, []);

  useEffect(() => {
    let unlisten: (() => void) | undefined;
    void listen<AgentRunEvent>("agent-run-event", (event) => {
      const runEvent = event.payload;
      if (!runEvent?.sessionId || !runEvent.runId) return;
      const runKey = `${runEvent.sessionId}:${runEvent.runId}`;
      const lastSequence = agentRunEventSequenceRef.current.get(runKey) ?? -1;
      if (runEvent.sequence <= lastSequence) return;
      if (terminalAgentRunEventsRef.current.has(runKey) && runEvent.type === "agent-run-output") return;
      agentRunEventSequenceRef.current.set(runKey, runEvent.sequence);
      const currentRun = runtime.getSessionRuntime(runEvent.sessionId).currentRun;
      if (currentRun && currentRun.runId !== runEvent.runId) return;
      if (runEvent.type === "agent-run-started") {
        runtime.markSessionRunStarted(runEvent.sessionId, runEvent.runId, runEvent.runId, runEvent.timestamp);
        return;
      }
      if (runEvent.type === "agent-run-output" && runEvent.chunk) {
        const visibleChunk = stripHermesRunMetadata(runEvent.chunk);
        if (!visibleChunk) return;
        updateSession(runEvent.sessionId, (session) => ({
          ...session,
          updatedAt: new Date().toISOString(),
          messages: session.messages.map((message) => message.id === runEvent.runId ? { ...message, content: `${message.content}${visibleChunk}\n`, status: "streaming" } : message),
        }));
        return;
      }
      if (runEvent.type === "agent-run-completed") {
        terminalAgentRunEventsRef.current.add(runKey);
        runtime.markSessionRunFinished(runEvent.sessionId, "idle", undefined, runEvent.runId);
        updateSession(runEvent.sessionId, (session) => ({
          ...session,
          updatedAt: new Date().toISOString(),
          messages: session.messages.map((message) => message.id === runEvent.runId ? { ...message, status: "sent" } : message),
        }));
        window.setTimeout(() => runNextQueuedPrompt(runEvent.sessionId), 0);
        return;
      }
      if (runEvent.type === "agent-run-error" || runEvent.type === "agent-run-stopped") {
        terminalAgentRunEventsRef.current.add(runKey);
        const status = runEvent.type === "agent-run-stopped" ? "interrupted" : "error";
        runtime.markSessionRunFinished(runEvent.sessionId, status, runEvent.message, runEvent.runId);
        updateSession(runEvent.sessionId, (session) => ({
          ...session,
          updatedAt: new Date().toISOString(),
          messages: session.messages.map((message) => message.id === runEvent.runId ? { ...message, status: "error", error: runEvent.message ?? "Hermes run failed." } : message),
        }));
        window.setTimeout(() => runNextQueuedPrompt(runEvent.sessionId), 0);
      }
    }).then((dispose) => { unlisten = dispose; }).catch(() => undefined);
    return () => { unlisten?.(); };
  }, [runtime]);

  function openSession(sessionId: string) {
    if (suppressNextSessionClickRef.current) {
      suppressNextSessionClickRef.current = false;
      return;
    }
    onSessionsChange((current) => current.map((session) => session.id === sessionId ? clearSessionNeedsReply(session) : session));
    onActiveSessionIdChange(sessionId);
  }

  function notifyForBackgroundAgentResponse(sessionId: string, assistantMessageId: string, responseContent: string) {
    if (isAgentsWorkspaceOpenRef.current && activeSessionIdRef.current === sessionId) return;
    const currentSession = sessionsRef.current.find((session) => session.id === sessionId);
    if (!currentSession || currentSession.lastNotifiedAssistantMessageId === assistantMessageId) return;
    const notificationUpdatedAt = new Date().toISOString();
    onSessionsChange((current) => current.map((session) => session.id === sessionId ? {
      ...session,
      needsReply: true,
      lastNotifiedAssistantMessageId: assistantMessageId,
      notificationUpdatedAt,
    } : session));
    const sessionTitle = currentSession.title || "Hermes session";
    const responsePreview = agentResponsePreview(responseContent);
    void sendDesktopAgentNotification({ sessionTitle, responsePreview });
  }

  function beginRenameSession(session: HermesChatSession) {
    setEditingSessionId(session.id);
    setEditingSessionTitle(session.title);
  }

  function commitRenameSession() {
    if (!editingSessionId) return;
    const nextTitle = editingSessionTitle.replace(/\s+/g, " ").trim();
    if (nextTitle) {
      updateSession(editingSessionId, (session) => ({ ...session, title: nextTitle, updatedAt: new Date().toISOString() }));
    }
    setEditingSessionId(null);
    setEditingSessionTitle("");
  }

  function cancelRenameSession() {
    setEditingSessionId(null);
    setEditingSessionTitle("");
  }

  function handleRenameKeyDown(event: ReactKeyboardEvent<HTMLInputElement>) {
    if (event.key === "Enter") {
      event.preventDefault();
      commitRenameSession();
    }
    if (event.key === "Escape") {
      event.preventDefault();
      cancelRenameSession();
    }
  }

  function handleLinkedRepositoryChange(repositoryId: string) {
    if (activeSession) {
      updateSession(activeSession.id, (session) => ({ ...session, linkedRepositoryId: repositoryId === "none" ? undefined : repositoryId, updatedAt: new Date().toISOString() }));
    }
  }

  async function loadFileManagerPath(path?: string, options: { makeRoot?: boolean } = {}) {
    const loadingKey = path || "macos-home";
    setFileManagerLoadingPath(loadingKey);
    setFileManagerError(null);
    try {
      const listing = await listFileManagerDirectory(path);
      setFileManagerListings((current) => ({ ...current, [listing.path]: listing }));
      setExpandedFilePaths((current) => new Set(current).add(listing.path));
      if (options.makeRoot || !path || !fileManagerRootPath) {
        setFileManagerRootPath(listing.path);
      }
    } catch (error) {
      setFileManagerError(error instanceof Error ? error.message : String(error));
    } finally {
      setFileManagerLoadingPath(null);
    }
  }

  function handleFileManagerToggle() {
    if (fileManagerOpen) {
      setFileManagerOpen(false);
      return;
    }
    if (!fileManagerRootPath) void loadFileManagerPath();
    setFileManagerVisible(true);
    window.requestAnimationFrame(() => setFileManagerOpen(true));
  }

  useEffect(() => {
    if (fileManagerOpen || !fileManagerVisible) return;
    const timeout = window.setTimeout(() => setFileManagerVisible(false), 460);
    return () => window.clearTimeout(timeout);
  }, [fileManagerOpen, fileManagerVisible]);

  function handleFolderToggle(entry: FileManagerEntry) {
    if (entry.kind !== "directory") return;
    if (expandedFilePaths.has(entry.path)) {
      setExpandedFilePaths((current) => {
        const next = new Set(current);
        next.delete(entry.path);
        return next;
      });
      return;
    }
    if (!fileManagerListings[entry.path]) {
      void loadFileManagerPath(entry.path);
      return;
    }
    setExpandedFilePaths((current) => new Set(current).add(entry.path));
  }

  function renderFileManagerEntries(listingPath: string, depth = 0): ReactNode {
    const listing = fileManagerListings[listingPath];
    if (!listing) return null;
    return (
      <ul className="file-manager-list">
        {listing.entries.map((entry) => {
          const isDirectory = entry.kind === "directory";
          const isExpanded = expandedFilePaths.has(entry.path);
          const itemContent = (
            <>
              <span className="file-manager-chevron" aria-hidden="true">
                {isDirectory ? (isExpanded ? <ChevronDown size={14} /> : <ChevronRight size={14} />) : null}
              </span>
              <span className="file-manager-icon" aria-hidden="true">
                {isDirectory ? <Folder size={15} /> : <FileText size={15} />}
              </span>
              <span className="file-manager-name">{entry.name}</span>
              <span className="file-manager-meta">
                {isDirectory ? `${entry.childrenCount ?? 0} items` : entry.size ? `${Math.ceil(entry.size / 1024)} KB` : entry.kind}
              </span>
            </>
          );
          return (
            <li key={entry.path} className="file-manager-row">
              {isDirectory ? (
                <button
                  aria-expanded={isExpanded}
                  className="file-manager-item file-manager-item--folder"
                  onClick={() => handleFolderToggle(entry)}
                  style={{ "--file-manager-depth": depth } as CSSProperties}
                  title={entry.path}
                  type="button"
                >
                  {itemContent}
                </button>
              ) : (
                <div
                  className="file-manager-item"
                  style={{ "--file-manager-depth": depth } as CSSProperties}
                  title={entry.path}
                >
                  {itemContent}
                </div>
              )}
              {isDirectory && isExpanded ? <div className="file-manager-branch">{renderFileManagerEntries(entry.path, depth + 1)}</div> : null}
            </li>
          );
        })}
      </ul>
    );
  }

  function prependNewSession() {
    onSessionsChange((current) => {
      const nextSession = createSession("New session", current);
      pendingNewSessionActivationRef.current = nextSession.id;
      return [nextSession, ...current];
    });
  }

  function handleNewSession() {
    prependNewSession();

  }

  function handleSessionsRailResizeStart(event: ReactPointerEvent<HTMLButtonElement>) {
    event.preventDefault();
    const startX = event.clientX;
    const startWidth = sessionsRailWidth;

    function handlePointerMove(moveEvent: PointerEvent) {
      setSessionsRailWidth(clampSessionsRailWidth(startWidth + moveEvent.clientX - startX));
    }

    function handlePointerUp() {
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
    }

    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", handlePointerUp, { once: true });
  }

  function handleFileManagerResizeStart(event: ReactPointerEvent<HTMLButtonElement>) {
    event.preventDefault();
    event.currentTarget.setPointerCapture?.(event.pointerId);
    const startX = event.clientX;
    const startWidth = fileManagerWidth;
    let latestClientX = startX;
    setIsFileManagerResizing(true);

    function applyResize(clientX: number) {
      setFileManagerWidth(clampFileManagerWidth(startWidth - (clientX - startX)));
    }

    function handlePointerMove(moveEvent: PointerEvent) {
      moveEvent.preventDefault();
      latestClientX = moveEvent.clientX;
      if (fileManagerResizeFrameRef.current !== null) return;
      fileManagerResizeFrameRef.current = window.requestAnimationFrame(() => {
        fileManagerResizeFrameRef.current = null;
        applyResize(latestClientX);
      });
    }

    function handlePointerUp(upEvent?: PointerEvent) {
      latestClientX = upEvent?.clientX ?? latestClientX;
      if (fileManagerResizeFrameRef.current !== null) {
        window.cancelAnimationFrame(fileManagerResizeFrameRef.current);
        fileManagerResizeFrameRef.current = null;
      }
      applyResize(latestClientX);
      setIsFileManagerResizing(false);
      window.removeEventListener("pointermove", handlePointerMove);
      window.removeEventListener("pointerup", handlePointerUp);
      window.removeEventListener("pointercancel", handlePointerUp);
    }

    window.addEventListener("pointermove", handlePointerMove);
    window.addEventListener("pointerup", handlePointerUp, { once: true });
    window.addEventListener("pointercancel", handlePointerUp, { once: true });
  }

  function handleFileManagerResizeKeyDown(event: ReactKeyboardEvent<HTMLButtonElement>) {
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    event.preventDefault();
    const delta = event.shiftKey ? 40 : 16;
    setFileManagerWidth((current) => clampFileManagerWidth(current + (event.key === "ArrowLeft" ? delta : -delta)));
  }

  function handleSessionsRailResizeKeyDown(event: ReactKeyboardEvent<HTMLButtonElement>) {
    if (event.key !== "ArrowLeft" && event.key !== "ArrowRight") return;
    event.preventDefault();
    const delta = event.shiftKey ? 40 : 16;
    setSessionsRailWidth((current) => clampSessionsRailWidth(current + (event.key === "ArrowRight" ? delta : -delta)));
  }

  function trapDialogFocus(event: ReactKeyboardEvent<HTMLElement>, closeDialog: () => void) {
    if (event.key === "Escape") {
      event.preventDefault();
      closeDialog();
      return;
    }
    if (event.key !== "Tab") return;
    const focusableElements = Array.from(
      event.currentTarget.querySelectorAll<HTMLElement>("button:not(:disabled), [href], input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex='-1'])"),
    ).filter((element) => !element.hasAttribute("disabled") && element.getAttribute("aria-hidden") !== "true");
    if (focusableElements.length === 0) {
      event.preventDefault();
      event.currentTarget.focus();
      return;
    }
    const firstElement = focusableElements[0];
    const lastElement = focusableElements[focusableElements.length - 1];
    if (event.shiftKey && document.activeElement === firstElement) {
      event.preventDefault();
      lastElement.focus();
    } else if (!event.shiftKey && document.activeElement === lastElement) {
      event.preventDefault();
      firstElement.focus();
    }
  }

  function handleSessionsRailMorphToggle() {
    const workspace = chatWorkspaceRef.current;
    const prefersReducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    if (!workspace || prefersReducedMotion) {
      setIsSessionsRailCompact((current) => !current);
      return;
    }

    sessionsRailMorphAnimationsRef.current.forEach((animation) => animation.cancel());
    sessionsRailMorphAnimationsRef.current = [];
    document.querySelectorAll("[data-session-rail-morph-clone]").forEach((clone) => clone.remove());

    const previousWorkspaceRect = workspace.getBoundingClientRect();
    const previousPanelRect = workspace.querySelector<HTMLElement>("[data-session-rail-morph-panel]")?.getBoundingClientRect();
    const previousItems: SessionsRailMorphSnapshot[] = Array.from(workspace.querySelectorAll<HTMLElement>("[data-session-rail-morph-item]")).map((element) => ({
      clone: element.cloneNode(true) as HTMLElement,
      key: element.getAttribute("data-session-rail-morph-item"),
      rect: element.getBoundingClientRect(),
    }));

    workspace.classList.add("sessions-rail-morphing");

    flushSync(() => {
      setIsSessionsRailCompact((current) => !current);
    });

    const nextWorkspaceRect = workspace.getBoundingClientRect();
    const nextPanelRect = workspace.querySelector<HTMLElement>("[data-session-rail-morph-panel]")?.getBoundingClientRect();
    const nextItems = new Map(
      Array.from(workspace.querySelectorAll<HTMLElement>("[data-session-rail-morph-item]")).map((element) => [element.getAttribute("data-session-rail-morph-item"), element]),
    );
    const previousKeys = new Set(previousItems.map((item) => item.key));

    sessionsRailMorphAnimationsRef.current.push(
      workspace.animate(
        [
          { transform: `translateX(${previousWorkspaceRect.left - nextWorkspaceRect.left}px)`, filter: "blur(0px)" },
          { transform: "translateX(0)", filter: "blur(0px)" },
        ],
        SESSIONS_RAIL_MORPH_TIMING,
      ),
    );

    if (previousPanelRect && nextPanelRect) {
      const panel = workspace.querySelector<HTMLElement>("[data-session-rail-morph-panel]");
      if (panel) {
        sessionsRailMorphAnimationsRef.current.push(
          panel.animate(
            [
              {
                opacity: isSessionsRailCompact ? 0.16 : 0.94,
                transform: `translate(${previousPanelRect.left - nextPanelRect.left}px, 0) scaleX(${previousPanelRect.width / Math.max(nextPanelRect.width, 1)})`,
              },
              { opacity: isSessionsRailCompact ? 1 : 0.98, transform: "translate(0, 0) scaleX(1)" },
            ],
            SESSIONS_RAIL_MORPH_TIMING,
          ),
        );
      }
    }

    previousItems.forEach(({ clone, key, rect }) => {
      const nextElement = nextItems.get(key);
      if (nextElement) {
        const nextRect = nextElement.getBoundingClientRect();
        sessionsRailMorphAnimationsRef.current.push(
          nextElement.animate(
            [
              {
                opacity: 0.9,
                transform: `translate(${rect.left - nextRect.left}px, ${rect.top - nextRect.top}px) scale(${rect.width / Math.max(nextRect.width, 1)}, ${rect.height / Math.max(nextRect.height, 1)})`,
              },
              { opacity: 1, transform: "translate(0, 0) scale(1, 1)" },
            ],
            SESSIONS_RAIL_MORPH_TIMING,
          ),
        );
        return;
      }

      clone.removeAttribute("data-session-rail-morph-item");
      clone.setAttribute("data-session-rail-morph-clone", "true");
      clone.setAttribute("aria-hidden", "true");
      Object.assign(clone.style, {
        height: `${rect.height}px`,
        left: `${rect.left}px`,
        margin: "0",
        pointerEvents: "none",
        position: "fixed",
        top: `${rect.top}px`,
        transformOrigin: "left center",
        width: `${rect.width}px`,
        zIndex: "60",
      });
      document.body.appendChild(clone);
      const animation = clone.animate(
        [
          { opacity: 1, transform: "translateX(0) scale(1)" },
          { opacity: 0, transform: "translateX(-18px) scale(0.96)" },
        ],
        SESSIONS_RAIL_MORPH_EXIT_TIMING,
      );
      sessionsRailMorphAnimationsRef.current.push(animation);
      animation.finished.then(() => clone.remove()).catch(() => clone.remove());
    });

    nextItems.forEach((element, key) => {
      if (previousKeys.has(key)) return;
      sessionsRailMorphAnimationsRef.current.push(
        element.animate(
          [
            { opacity: 0, transform: "translateY(12px) scale(0.92)" },
            { opacity: 1, transform: "translateY(0) scale(1)" },
          ],
          { ...SESSIONS_RAIL_MORPH_TIMING, delay: 90, duration: 420 },
        ),
      );
    });

    window.setTimeout(() => workspace.classList.remove("sessions-rail-morphing"), Number(SESSIONS_RAIL_MORPH_TIMING.duration));
  }

  async function handleRollbackToMessage(messageIndex: number) {
    if (!activeSession || isSending) return;
    const messagesToKeep = activeSession.messages.slice(0, messageIndex + 1);
    const userTurnsToUndo = activeSession.messages
      .slice(messageIndex + 1)
      .filter((message) => message.role === "user")
      .length;

    if (userTurnsToUndo <= 0) return;

    const sendingSessionId = activeSession.id;
    const command = `/undo ${userTurnsToUndo}`;
    const promptStartedAt = Date.now();
    const detectedRepository = selectedRepository;
    setIsSending(true);
    setLastPromptStartedAt(promptStartedAt);

    try {
      const result = await executeHermesSlashCommand(command, detectedRepository?.path, activeSession.hermesCliSessionId, true);
      onSessionsChange((current) => current.map((session) => session.id === sendingSessionId ? {
        ...session,
        hermesCliSessionId: result.session || session.hermesCliSessionId,
        updatedAt: new Date().toISOString(),
        messages: messagesToKeep,
      } : session));
      if (result.session) setCliStatus((current) => current ? { ...current, session: result.session || current.session } : current);
      saveRecentCommand(command);
      setRecentCommands(loadRecentCommands());
    } catch (error) {
      const errorMessage: ChatMessage = {
        id: `hermes-rollback-error-${crypto.randomUUID()}`,
        role: "assistant",
        participantId: "hermes",
        content: "Rollback failed.",
        createdAt: new Date().toISOString(),
        status: "error",
        error: error instanceof Error ? error.message : String(error),
      };
      updateSession(sendingSessionId, (session) => ({ ...session, updatedAt: new Date().toISOString(), messages: [...session.messages, errorMessage] }));
    } finally {
      setLastPromptStartedAt(null);
      setIsSending(false);
    }
  }

  function handleChatStagePointerDown(event: ReactPointerEvent<HTMLDivElement>) {
    const target = event.target instanceof HTMLElement ? event.target : null;
    if (target?.closest("button, a, input, textarea, select, [role='button'], [contenteditable='true'], .message-bubble, .message-copy-button, .message-rollback-button, .message-action-button")) return;
    window.requestAnimationFrame(() => composerRef.current?.focusMessageField());
  }

  function appendCommandResult(sessionId: string, command: string, result: HermesSlashCommandExecution, assistantId: string) {
    if (result.kind === "new-session") {
      const content = result.content || "Started a new Zoid session.";
      onSessionsChange((current) => current.map((session) => session.id === sessionId ? {
        ...session,
        hermesCliSessionId: result.session || session.hermesCliSessionId,
        updatedAt: new Date().toISOString(),
        messages: session.messages.map((message) => message.id === assistantId ? { ...message, content, status: "sent" } : message),
      } : session));
      if (result.session) setCliStatus((current) => current ? { ...current, session: result.session || current.session } : current);
      saveRecentCommand(command);
      setRecentCommands(loadRecentCommands());
      prependNewSession();
      return;
    }
    if (result.kind === "close-session") {
      saveRecentCommand(command);
      setRecentCommands(loadRecentCommands());
      onArchiveSession(sessionId);
      return;
    }
    if (result.kind === "panel" && result.panel) {
      setActiveCommandPanel(result.panel);
    }
    const content = result.kind === "panel"
      ? `Opened ${result.panel ?? "command panel"}.`
      : result.content || "Command completed.";
    onSessionsChange((current) => current.map((session) => session.id === sessionId ? {
      ...session,
      hermesCliSessionId: result.session || session.hermesCliSessionId,
      updatedAt: new Date().toISOString(),
      messages: session.messages.some((message) => message.id === assistantId)
        ? session.messages.map((message) => message.id === assistantId ? { ...message, content, status: "sent" } : message)
        : [...session.messages, { id: assistantId, role: "assistant", participantId: "hermes", content, createdAt: new Date().toISOString(), status: "sent" }],
    } : session));
    notifyForBackgroundAgentResponse(sessionId, assistantId, content);
    if (result.session) setCliStatus((current) => current ? { ...current, session: result.session || current.session } : current);
    saveRecentCommand(command);
    setRecentCommands(loadRecentCommands());
  }

  async function runSlashCommand(sessionForCommand: HermesChatSession, command: string, confirmed = false) {
    const sendingSessionId = sessionForCommand.id;
    const assistantId = `hermes-command-${crypto.randomUUID()}`;
    const userMessage: ChatMessage = { id: `user-command-${crypto.randomUUID()}`, role: "user", participantId: "ziad", content: command, createdAt: new Date().toISOString(), status: "sent" };
    const assistantMessage: ChatMessage = { id: assistantId, role: "assistant", participantId: "hermes", content: "", createdAt: new Date().toISOString(), status: "streaming" };
    const sessionRepository = repositories.find((repository) => repository.id === sessionForCommand.linkedRepositoryId);
    const detectedRepository = sessionRepository ?? detectRepositoryFromPrompt(command, repositories);
    if (!runtime.tryStartSessionRun(sendingSessionId, assistantId)) {
      runtime.queuePrompt(sendingSessionId, command);
      queueHermesPrompt(sendingSessionId, command, "slash");
      return;
    }
    updateSession(sendingSessionId, (session) => ({
      ...session,
      updatedAt: new Date().toISOString(),
      messages: [...session.messages, userMessage, assistantMessage],
      linkedRepositoryId: detectedRepository?.id ?? session.linkedRepositoryId,
    }));
    activeHermesRunsRef.current.set(sendingSessionId, { sessionId: sendingSessionId, assistantId });
    setIsSending(true);
    const promptStartedAt = Date.now();
    setLastPromptStartedAt(promptStartedAt);

    try {
      const result = await executeHermesSlashCommand(command, detectedRepository?.path, sessionForCommand.hermesCliSessionId, confirmed);
      if (result.requiresConfirmation) {
        setPendingConfirmation({
          result,
          sessionId: sendingSessionId,
          assistantId,
          command,
          linkedRepositoryPath: detectedRepository?.path,
          hermesCliSessionId: sessionForCommand.hermesCliSessionId,
        });
        onSessionsChange((current) => current.map((session) => session.id === sendingSessionId ? {
          ...session,
          messages: session.messages.map((message) => message.id === assistantId ? { ...message, content: result.content || "Confirmation required.", status: "sent" } : message),
        } : session));
      } else {
        appendCommandResult(sendingSessionId, command, result, assistantId);
      }
    } catch (error) {
      const stoppedByUser = error instanceof Error && error.message.includes("stopped by the user");
      const preservedStopCopy = activeHermesRunsRef.current.get(sendingSessionId)?.assistantId === assistantId ? activeHermesRunsRef.current.get(sendingSessionId)?.stopCopy : undefined;
      onSessionsChange((current) => current.map((session) => session.id === sendingSessionId ? {
        ...session,
        updatedAt: new Date().toISOString(),
        messages: session.messages.map((message) => {
          if (message.id !== assistantId) return message;
          if (stoppedByUser && message.error === "User stopped the active run.") return message;
          return { ...message, content: stoppedByUser ? preservedStopCopy ?? "Stopped Hermes run." : message.content || "Command failed.", status: "error", error: error instanceof Error ? error.message : String(error) };
        }),
      } : session));
    } finally {
      setLastPromptStartedAt(null);
      clearActiveHermesRunIfCurrent(sendingSessionId, assistantId);
      runtime.markSessionRunFinished(sendingSessionId, "idle", undefined, assistantId);
      setIsSending(activeHermesRunsRef.current.size > 0);
      runNextQueuedPrompt(sendingSessionId);
    }
  }

  async function runPendingConfirmedCommand(pending: PendingCommandConfirmation) {
    if (!runtime.tryStartSessionRun(pending.sessionId, pending.assistantId)) {
      runtime.queuePrompt(pending.sessionId, pending.command);
      queueHermesPrompt(pending.sessionId, pending.command, "slash");
      return;
    }
    setIsSending(true);
    activeHermesRunsRef.current.set(pending.sessionId, { sessionId: pending.sessionId, assistantId: pending.assistantId });
    const promptStartedAt = Date.now();
    setLastPromptStartedAt(promptStartedAt);

    try {
      const result = await executeHermesSlashCommand(pending.command, pending.linkedRepositoryPath, pending.hermesCliSessionId, true);
      appendCommandResult(pending.sessionId, pending.command, result, pending.assistantId);
    } catch (error) {
      const stoppedByUser = error instanceof Error && error.message.includes("stopped by the user");
      const preservedStopCopy = activeHermesRunsRef.current.get(pending.sessionId)?.assistantId === pending.assistantId ? activeHermesRunsRef.current.get(pending.sessionId)?.stopCopy : undefined;
      onSessionsChange((current) => current.map((session) => session.id === pending.sessionId ? {
        ...session,
        updatedAt: new Date().toISOString(),
        messages: session.messages.map((message) => {
          if (message.id !== pending.assistantId) return message;
          if (stoppedByUser && message.error === "User stopped the active run.") return message;
          return { ...message, content: stoppedByUser ? preservedStopCopy ?? "Stopped Hermes run." : message.content || "Command failed.", status: "error", error: error instanceof Error ? error.message : String(error) };
        }),
      } : session));
    } finally {
      setLastPromptStartedAt(null);
      clearActiveHermesRunIfCurrent(pending.sessionId, pending.assistantId);
      runtime.markSessionRunFinished(pending.sessionId, "idle", undefined, pending.assistantId);
      setIsSending(activeHermesRunsRef.current.size > 0);
      runNextQueuedPrompt(pending.sessionId);
    }
  }

  async function handleStopHermesRun(source: "button" | "keyboard" = "button", scopedSessionId?: string) {
    const stopTargetSessionId = scopedSessionId ?? activeSessionId;
    const run = runtime.getSessionRuntime(stopTargetSessionId).currentRun ?? activeHermesRunsRef.current.get(stopTargetSessionId);
    if (!run) return;
    const stopCopy = source === "keyboard" ? "Stopped Hermes with Ctrl/Cmd+C." : "Stopped Hermes run.";
    const stopRunId = "runId" in run ? run.runId : run.assistantId;
    try {
      const stopped = await cancelHermesCliRun(run.sessionId, stopRunId);
      if (!stopped) return;
      activeHermesRunsRef.current.set(run.sessionId, { sessionId: run.sessionId, assistantId: stopRunId, stopCopy });
      clearActiveHermesRunIfCurrent(run.sessionId, stopRunId);
      runtime.markSessionRunFinished(run.sessionId, "interrupted", "User stopped the active run.", stopRunId);
      setIsSending(activeHermesRunsRef.current.size > 0);
      updateSession(run.sessionId, (session) => ({
        ...session,
        updatedAt: new Date().toISOString(),
        messages: session.messages.map((message) => (
          message.id === stopRunId
            ? { ...message, content: stopCopy, status: "error", error: "User stopped the active run." }
            : message
        )),
      }));
    } catch (error) {
      const detail = error instanceof Error ? error.message : String(error);
      runtime.markSessionRunFinished(run.sessionId, "error", detail);
      updateSession(run.sessionId, (session) => ({
        ...session,
        updatedAt: new Date().toISOString(),
        messages: session.messages.map((message) => (
          message.id === stopRunId
            ? { ...message, content: "Could not stop Hermes run.", status: "error", error: detail }
            : message
        )),
      }));
    }
  }

  function queueHermesPrompt(sessionId: string, content: string, kind: QueuedHermesPrompt["kind"]) {
    queuedHermesPromptsRef.current = [...queuedHermesPromptsRef.current, { sessionId, content, kind }];
  }

  function runNextQueuedPrompt(preferredSessionId?: string) {
    const queuedPrompts = queuedHermesPromptsRef.current;
    const preferredPrompt = preferredSessionId
      ? queuedPrompts.find((prompt) => prompt.sessionId === preferredSessionId && runtime.canStartSessionRun(prompt.sessionId))
      : undefined;
    const nextPrompt = preferredPrompt ?? queuedPrompts.find((prompt) => runtime.canStartSessionRun(prompt.sessionId));
    if (!nextPrompt) return;
    queuedHermesPromptsRef.current = queuedPrompts.filter((prompt) => prompt !== nextPrompt);
    runtime.dequeuePrompt(nextPrompt.sessionId);
    window.setTimeout(() => {
      const session = sessionsRef.current.find((candidate) => candidate.id === nextPrompt.sessionId);
      if (!session) return;
      if (nextPrompt.kind === "slash") {
        void runSlashCommand(session, nextPrompt.content);
        return;
      }
      void sendHermesPrompt(session, nextPrompt.content);
    }, 0);
  }

  async function sendHermesPrompt(sessionForSend: HermesChatSession, content: string) {
    const sendingSessionId = sessionForSend.id;
    const userMessage: ChatMessage = { id: `user-${crypto.randomUUID()}`, role: "user", participantId: "ziad", content, createdAt: new Date().toISOString(), status: "sent" };
    const assistantId = `hermes-${crypto.randomUUID()}`;
    const assistantMessage: ChatMessage = { id: assistantId, role: "assistant", participantId: "hermes", content: "", createdAt: new Date().toISOString(), status: "streaming" };
    const nextMessages = [...sessionForSend.messages, userMessage];
    const updatedAt = new Date().toISOString();
    const effectiveRepositoryId = sessionForSend.linkedRepositoryId;
    const currentLinkedRepository = repositories.find((repository) => repository.id === effectiveRepositoryId);
    const detectedRepository = currentLinkedRepository ?? detectRepositoryFromPrompt(content, repositories);
    if (!runtime.tryStartSessionRun(sendingSessionId, assistantId)) {
      runtime.queuePrompt(sendingSessionId, content);
      queueHermesPrompt(sendingSessionId, content, "prompt");
      return;
    }
    const modelInfoForSend = effectiveModelInfoForSession(sessionForSend);
    updateSession(sendingSessionId, (session) => ({
      ...session,
      title: session.title === "New session" ? titleFromPrompt(content) : session.title,
      linkedRepositoryId: detectedRepository?.id ?? session.linkedRepositoryId,
      modelName: modelInfoForSend.modelName,
      reasoningEffort: modelInfoForSend.reasoningEffort,
      updatedAt,
      messages: [...session.messages, userMessage, assistantMessage],
    }));

    const promptStartedAt = Date.now();
    const eventRunKey = `${sendingSessionId}:${assistantId}`;
    agentRunEventSequenceRef.current.delete(eventRunKey);
    terminalAgentRunEventsRef.current.delete(eventRunKey);
    setLastPromptStartedAt(promptStartedAt);

    setElapsedTick(promptStartedAt);
    activeHermesRunsRef.current.set(sendingSessionId, { sessionId: sendingSessionId, assistantId });
    setIsSending(true);
    let finalRuntimeStatus: "idle" | "error" | "interrupted" = "idle";
    let finalRuntimeError: string | undefined;

    try {
      const response = await sendHermesCliRunMessage(
        nextMessages.map((message) => ({ role: message.role, content: message.content })),
        detectedRepository?.path,
        sessionForSend.hermesCliSessionId,
        sendingSessionId,
        assistantId,
      );
      const responseContent = response.content || "Hermes CLI returned an empty response.";
      onSessionsChange((current) => current.map((session) => session.id === sendingSessionId ? {
        ...session,
        title: session.title === "New session" ? titleFromPrompt(content) : session.title,
        linkedRepositoryId: detectedRepository?.id ?? session.linkedRepositoryId,
        hermesCliSessionId: response.session || session.hermesCliSessionId,
        modelName: modelInfoForSend.modelName,
        reasoningEffort: modelInfoForSend.reasoningEffort,
        updatedAt: new Date().toISOString(),
        messages: session.messages.map((message) => message.id === assistantId ? { ...message, content: responseContent, status: "sent" } : message),
      } : session));
      notifyForBackgroundAgentResponse(sendingSessionId, assistantId, responseContent);
      setCliStatus((current) => current ? { ...current, session: response.session } : current);
      if (sessionForSend.operationRunId && sessionForSend.operationRepositoryId && sessionForSend.operationAction) {
        onRepositoryOperationComplete?.({
          sessionId: sendingSessionId,
          runId: sessionForSend.operationRunId,
          repositoryId: sessionForSend.operationRepositoryId,
          action: sessionForSend.operationAction,
          outcome: "needs-user",
          responseContent,
        });
      }
    } catch (error) {
      const stoppedByUser = error instanceof Error && error.message.includes("stopped by the user");
      finalRuntimeStatus = stoppedByUser ? "interrupted" : "error";
      finalRuntimeError = error instanceof Error ? error.message : String(error);
      const preservedStopCopy = activeHermesRunsRef.current.get(sendingSessionId)?.assistantId === assistantId ? activeHermesRunsRef.current.get(sendingSessionId)?.stopCopy : undefined;
      onSessionsChange((current) => current.map((session) => session.id === sendingSessionId ? {
        ...session,
        updatedAt: new Date().toISOString(),
        messages: session.messages.map((message) => {
          if (message.id !== assistantId) return message;
          if (stoppedByUser && message.error === "User stopped the active run.") return message;
          return {
            ...message,
            content: stoppedByUser ? preservedStopCopy ?? "Stopped Hermes run." : message.content || "Hermes terminal response failed.",
            status: "error",
            error: error instanceof Error ? error.message : String(error),
          };
        }),
      } : session));
      if (sessionForSend.operationRunId && sessionForSend.operationRepositoryId && sessionForSend.operationAction) {
        onRepositoryOperationComplete?.({
          sessionId: sendingSessionId,
          runId: sessionForSend.operationRunId,
          repositoryId: sessionForSend.operationRepositoryId,
          action: sessionForSend.operationAction,
          outcome: stoppedByUser ? "cancelled" : "failed",
          responseContent: error instanceof Error ? error.message : String(error),
        });
      }
    } finally {
      setLastPromptStartedAt(null);
      clearActiveHermesRunIfCurrent(sendingSessionId, assistantId);
      runtime.markSessionRunFinished(sendingSessionId, finalRuntimeStatus, finalRuntimeError, assistantId);
      setIsSending(activeHermesRunsRef.current.size > 0);
      runNextQueuedPrompt(sendingSessionId);
    }
  }

  function handleStartRuthlessCodeReview() {
    if (!activeSession || isSending) return;
    const prompt = buildRuthlessReviewerPrompt({
      repository: selectedRepository,
      activeSessionTitle: activeSession.title,
    });
    setActiveCommandPanel(null);
    void sendHermesPrompt(activeSession, prompt);
  }

  async function handleSend(content: string) {
    const targetSession = expandedSession ?? activeSession;
    if (!targetSession) return;
    const parsedSlashCommand = parseSlashCommand(content, slashCommands);
    if (activeHermesRunsRef.current.has(targetSession.id) || runtime.getSessionRuntime(targetSession.id).status === "running" || runtime.getSessionRuntime(targetSession.id).status === "needs-input") {
      queueHermesPrompt(targetSession.id, content, parsedSlashCommand ? "slash" : "prompt");
      runtime.queuePrompt(targetSession.id, content);
      return;
    }
    if (parsedSlashCommand) {
      await runSlashCommand(targetSession, content);
      return;
    }
    await sendHermesPrompt(targetSession, content);
  }

  const tiledSessions = dashboardState.tiledSessionIds.map((sessionId) => sessions.find((session) => session.id === sessionId)).filter((session): session is HermesChatSession => Boolean(session));
  const dashboardVisibleSessions = tiledSessions;
  const runningCount = runtime.activeRunCount;
  const queuedCount = Object.values(runtime.runtimeBySessionId).reduce((total, state) => total + state.queuedPrompts.length, 0);
  const needsReplyCount = sessions.filter((session) => session.needsReply || runtime.getSessionRuntime(session.id).status === "needs-input").length;
  const expandedSession = expandedModeSessionId ? sessions.find((session) => session.id === expandedModeSessionId) : undefined;

  function patchDashboard(updater: (current: AgentDashboardStateV1) => AgentDashboardStateV1) {
    setDashboardState((current) => updater(current));
  }


  function removeSessionFromDashboard(sessionId: string) {
    patchDashboard((current) => {
      const tiledSessionIds = current.tiledSessionIds.filter((id) => id !== sessionId);
      return { ...current, tiledSessionIds, primarySessionId: current.primarySessionId === sessionId ? tiledSessionIds[0] : current.primarySessionId, focusedSessionId: current.focusedSessionId === sessionId ? tiledSessionIds[0] : current.focusedSessionId };
    });
  }

  function moveDashboardSession(sessionId: string, direction: "left" | "right") {
    patchDashboard((current) => {
      const index = current.tiledSessionIds.indexOf(sessionId);
      const swapWith = direction === "left" ? index - 1 : index + 1;
      if (index < 0 || swapWith < 0 || swapWith >= current.tiledSessionIds.length) return current;
      const tiledSessionIds = [...current.tiledSessionIds];
      [tiledSessionIds[index], tiledSessionIds[swapWith]] = [tiledSessionIds[swapWith], tiledSessionIds[index]];
      return { ...current, tiledSessionIds };
    });
  }

  function sessionDashboardPriority(session: HermesChatSession) {
    const state = runtime.getSessionRuntime(session.id);
    if (state.status === "needs-input" || session.needsReply) return 0;
    if (state.status === "error") return 1;
    if (state.status === "interrupted") return 2;
    if (state.status === "running") return 3;
    if (state.queuedPrompts.length) return 4;
    return 5;
  }

  useEffect(() => {
    if (!dashboardState.autoPrioritize || dashboardState.tiledSessionIds.length < 2) return;
    const ordered = [...dashboardState.tiledSessionIds].sort((leftId, rightId) => {
      const left = sessions.find((session) => session.id === leftId);
      const right = sessions.find((session) => session.id === rightId);
      if (!left || !right) return 0;
      return sessionDashboardPriority(left) - sessionDashboardPriority(right) || Date.parse(right.updatedAt) - Date.parse(left.updatedAt);
    });
    if (ordered.join("|") === dashboardState.tiledSessionIds.join("|")) return;
    patchDashboard((current) => ({ ...current, tiledSessionIds: ordered, focusedSessionId: ordered[0] ?? current.focusedSessionId, primarySessionId: current.primarySessionId && ordered.includes(current.primarySessionId) ? current.primarySessionId : ordered[0] }));
  }, [dashboardState.autoPrioritize, dashboardState.tiledSessionIds.join("|"), sessions, runtime.runtimeBySessionId]);

  function showActiveAgents() {
    const priority = (session: HermesChatSession) => {
      const state = runtime.getSessionRuntime(session.id);
      if (state.status === "needs-input" || session.needsReply) return 0;
      if (state.status === "error") return 1;
      if (state.status === "interrupted") return 2;
      if (state.status === "running") return 3;
      if (state.queuedPrompts.length) return 4;
      return 5;
    };
    const ordered = [...sessions].sort((left, right) => priority(left) - priority(right) || Date.parse(right.updatedAt) - Date.parse(left.updatedAt));
    patchDashboard((current) => {
      const tiledSessionIds = [...current.tiledSessionIds];
      for (const session of ordered) {
        if (tiledSessionIds.length >= AGENT_DASHBOARD_MAX_TILES) break;
        if (!tiledSessionIds.includes(session.id)) tiledSessionIds.push(session.id);
      }
      return { ...current, tiledSessionIds, focusedSessionId: tiledSessionIds[0], primarySessionId: current.primarySessionId ?? tiledSessionIds[0] };
    });
  }

  async function sendToDashboardSession(sessionId: string, prompt: string) {
    const session = sessionsRef.current.find((candidate) => candidate.id === sessionId);
    if (!session) return;
    const sessionRuntime = runtime.getSessionRuntime(sessionId);
    if (sessionRuntime.status === "running" || sessionRuntime.status === "needs-input") {
      runtime.queuePrompt(sessionId, prompt);
      queueHermesPrompt(sessionId, prompt, parseSlashCommand(prompt, slashCommands) ? "slash" : "prompt");
      return;
    }
    if (!runtime.canStartSessionRun(sessionId)) {
      runtime.queuePrompt(sessionId, prompt);
      queueHermesPrompt(sessionId, prompt, parseSlashCommand(prompt, slashCommands) ? "slash" : "prompt");
      return;
    }
    openSession(sessionId);
    const parsedSlashCommand = parseSlashCommand(prompt, slashCommands);
    if (parsedSlashCommand) {
      await runSlashCommand(session, prompt);
      return;
    }
    await sendHermesPrompt(session, prompt);
  }

  async function continueDashboardSession(sessionId: string) {
    const session = sessionsRef.current.find((candidate) => candidate.id === sessionId);
    if (!session || runtime.getSessionRuntime(sessionId).status === "running") return;
    await sendToDashboardSession(sessionId, buildContinuationBrief(session));
  }

  function draggedSessionIdFromEvent(event: ReactDragEvent<HTMLElement>) {
    return activeDashboardDragSessionIdRef.current || event.dataTransfer.getData(SESSION_DASHBOARD_DRAG_TYPE) || event.dataTransfer.getData("text/plain") || draggedDashboardSessionId;
  }

  function canDropSessionOnDashboard(sessionId: string | null) {
    return dashboardDropStatus(sessionId).canDrop;
  }

  function dashboardPointIsInside(clientX: number, clientY: number) {
    const rect = chatDashboardDropRef.current?.getBoundingClientRect();
    return Boolean(rect && rect.width > 0 && rect.height > 0 && clientX >= rect.left && clientX <= rect.right && clientY >= rect.top && clientY <= rect.bottom);
  }

  function dashboardDropInsertionIndexFromPoint(clientX: number, clientY: number, sessionId: string | null) {
    const rect = chatDashboardDropRef.current?.getBoundingClientRect();
    if (!rect || rect.width <= 0 || rect.height <= 0) return undefined;
    const validSessionIds = new Set(sessionsRef.current.map((session) => session.id));
    const currentTiles = (dashboardStateRef.current ?? dashboardState).tiledSessionIds.filter((id) => validSessionIds.has(id));
    const seededTileCount = currentTiles.length === 0 && activeSessionIdRef.current && activeSessionIdRef.current !== sessionId ? 1 : currentTiles.length;
    const tileCount = Math.max(1, Math.min(AGENT_DASHBOARD_MAX_TILES, seededTileCount));
    const xRatio = Math.max(0, Math.min(1, (clientX - rect.left) / rect.width));
    const yRatio = Math.max(0, Math.min(1, (clientY - rect.top) / rect.height));
    if (tileCount <= 1) return xRatio < 0.5 ? 0 : 1;
    if (tileCount === 2) return xRatio < 0.5 ? 0 : 2;
    if (tileCount === 3) return xRatio < 0.5 ? 1 : yRatio < 0.5 ? 1 : 3;
    return (xRatio < 0.5 ? 0 : 1) + (yRatio < 0.5 ? 0 : 2);
  }

  function dashboardDropStatus(sessionId: string | null) {
    const validSessions = sessionsRef.current;
    const currentDashboard = dashboardStateRef.current ?? dashboardState;
    const currentTiles = currentDashboard.tiledSessionIds.filter((id) => validSessions.some((session) => session.id === id));
    if (!sessionId || !validSessions.some((session) => session.id === sessionId)) return { canDrop: false, cue: "Drag a session icon" };
    if (currentTiles.includes(sessionId)) return { canDrop: true, cue: "Drop to move/focus" };
    if (currentTiles.length >= AGENT_DASHBOARD_MAX_TILES) return { canDrop: false, cue: "Max 4 panels" };
    return { canDrop: true, cue: "Drop to split chat" };
  }

  function applySessionDropToDashboard(sessionId: string | null, insertAt?: number) {
    const dropStatus = dashboardDropStatus(sessionId);
    setDashboardDropCue(dropStatus.cue);
    if (!dropStatus.canDrop) return;
    setExpandedModeSessionId(null);
    setIsChatDropArmed(false);
    setDraggedDashboardSessionId(null);
    lastDashboardNativeDragPointRef.current = null;
    activeDashboardDragSessionIdRef.current = null;
    patchDashboard((current) => applyDraggedSessionToDashboard(current, sessionId as string, activeSessionIdRef.current, sessionsRef.current.map((session) => session.id), { insertAt }));
  }

  function handleSessionIconDragStart(event: ReactDragEvent<HTMLElement>, session: HermesChatSession) {
    event.stopPropagation();
    activeDashboardDragSessionIdRef.current = session.id;
    event.dataTransfer.effectAllowed = "copy";
    event.dataTransfer.setData(SESSION_DASHBOARD_DRAG_TYPE, session.id);
    event.dataTransfer.setData("text/plain", session.id);
    setDashboardDropCue(dashboardDropStatus(session.id).cue);
    lastDashboardNativeDragPointRef.current = null;
    setDraggedDashboardSessionId(session.id);
    setIsChatDropArmed(false);
  }

  function handleSessionIconPointerDown(event: ReactPointerEvent<HTMLElement>, session: HermesChatSession) {
    if (event.button !== 0) return;
    activeDashboardDragSessionIdRef.current = session.id;
    setDashboardDropCue(dashboardDropStatus(session.id).cue);
    dashboardPointerDragRef.current = { sessionId: session.id, startX: event.clientX, startY: event.clientY, isDragging: false };
  }

  function handleSessionIconDragEnd(event?: ReactDragEvent<HTMLElement>) {
    const sessionId = activeDashboardDragSessionIdRef.current || draggedDashboardSessionId;
    const eventPoint = event && (event.clientX !== 0 || event.clientY !== 0) ? { clientX: event.clientX, clientY: event.clientY } : null;
    const fallbackPoint = eventPoint && dashboardPointIsInside(eventPoint.clientX, eventPoint.clientY) ? eventPoint : lastDashboardNativeDragPointRef.current;
    dashboardPointerDragRef.current = null;
    if (sessionId && fallbackPoint && dashboardPointIsInside(fallbackPoint.clientX, fallbackPoint.clientY) && canDropSessionOnDashboard(sessionId)) {
      applySessionDropToDashboard(sessionId, dashboardDropInsertionIndexFromPoint(fallbackPoint.clientX, fallbackPoint.clientY, sessionId));
      return;
    }
    activeDashboardDragSessionIdRef.current = null;
    lastDashboardNativeDragPointRef.current = null;
    setDraggedDashboardSessionId(null);
    setIsChatDropArmed(false);
  }

  function handleChatDashboardDragOver(event: ReactDragEvent<HTMLElement>) {
    const sessionId = draggedSessionIdFromEvent(event);
    lastDashboardNativeDragPointRef.current = { clientX: event.clientX, clientY: event.clientY };
    const dropStatus = dashboardDropStatus(sessionId);
    if (!activeDashboardDragSessionIdRef.current && !dropStatus.canDrop) return;
    event.preventDefault();
    event.dataTransfer.dropEffect = dropStatus.canDrop ? "copy" : "none";
    setDashboardDropCue(dropStatus.cue);
    setIsChatDropArmed(true);
  }

  function handleChatDashboardDragLeave(event: ReactDragEvent<HTMLElement>) {
    if (event.currentTarget.contains(event.relatedTarget as Node | null)) return;
    setIsChatDropArmed(false);
  }

  function handleChatDashboardDrop(event: ReactDragEvent<HTMLElement>) {
    const sessionId = draggedSessionIdFromEvent(event);
    if (!canDropSessionOnDashboard(sessionId)) return;
    event.preventDefault();
    applySessionDropToDashboard(sessionId, dashboardDropInsertionIndexFromPoint(event.clientX, event.clientY, sessionId));
  }

  useEffect(() => {
    function pointerIsInsideDashboard(event: PointerEvent) {
      const rect = chatDashboardDropRef.current?.getBoundingClientRect();
      return Boolean(rect && event.clientX >= rect.left && event.clientX <= rect.right && event.clientY >= rect.top && event.clientY <= rect.bottom);
    }

    function finishPointerDrag() {
      dashboardPointerDragRef.current = null;
      activeDashboardDragSessionIdRef.current = null;
      setDraggedDashboardSessionId(null);
      setIsChatDropArmed(false);
    }

    function handleWindowPointerMove(event: PointerEvent) {
      const dragState = dashboardPointerDragRef.current;
      if (!dragState) return;
      const movedFarEnough = Math.hypot(event.clientX - dragState.startX, event.clientY - dragState.startY) > 6;
      if (!dragState.isDragging && !movedFarEnough) return;
      if (!dragState.isDragging) {
        dragState.isDragging = true;
        activeDashboardDragSessionIdRef.current = dragState.sessionId;
        suppressNextSessionClickRef.current = true;
        setDraggedDashboardSessionId(dragState.sessionId);
      }
      event.preventDefault();
      const overDashboard = pointerIsInsideDashboard(event);
      setDashboardDropCue(dashboardDropStatus(dragState.sessionId).cue);
      setIsChatDropArmed(overDashboard);
    }

    function handleWindowPointerUp(event: PointerEvent) {
      const dragState = dashboardPointerDragRef.current;
      if (!dragState) return;
      if (dragState.isDragging) {
        suppressNextSessionClickRef.current = true;
        if (pointerIsInsideDashboard(event)) applySessionDropToDashboard(dragState.sessionId, dashboardDropInsertionIndexFromPoint(event.clientX, event.clientY, dragState.sessionId));
      }
      finishPointerDrag();
    }

    function handleWindowDragOver(event: DragEvent) {
      const sessionId = activeDashboardDragSessionIdRef.current;
      if (!sessionId) return;
      lastDashboardNativeDragPointRef.current = { clientX: event.clientX, clientY: event.clientY };
      const insideDashboard = dashboardPointIsInside(event.clientX, event.clientY);
      const dropStatus = dashboardDropStatus(sessionId);
      if (!insideDashboard) {
        setIsChatDropArmed(false);
        return;
      }
      event.preventDefault();
      event.stopPropagation();
      if (event.dataTransfer) event.dataTransfer.dropEffect = dropStatus.canDrop ? "copy" : "none";
      setDashboardDropCue(dropStatus.cue);
      setIsChatDropArmed(true);
    }

    function handleWindowDrop(event: DragEvent) {
      const sessionId = activeDashboardDragSessionIdRef.current;
      if (!sessionId) return;
      const point = { clientX: event.clientX, clientY: event.clientY };
      lastDashboardNativeDragPointRef.current = point;
      if (!dashboardPointIsInside(point.clientX, point.clientY)) return;
      event.preventDefault();
      event.stopPropagation();
      if (canDropSessionOnDashboard(sessionId)) applySessionDropToDashboard(sessionId, dashboardDropInsertionIndexFromPoint(point.clientX, point.clientY, sessionId));
    }

    window.addEventListener("pointermove", handleWindowPointerMove, { passive: false });
    window.addEventListener("pointerup", handleWindowPointerUp);
    window.addEventListener("pointercancel", finishPointerDrag);
    document.addEventListener("dragover", handleWindowDragOver, { capture: true });
    document.addEventListener("drop", handleWindowDrop, { capture: true });
    return () => {
      window.removeEventListener("pointermove", handleWindowPointerMove);
      window.removeEventListener("pointerup", handleWindowPointerUp);
      window.removeEventListener("pointercancel", finishPointerDrag);
      document.removeEventListener("dragover", handleWindowDragOver, { capture: true });
      document.removeEventListener("drop", handleWindowDrop, { capture: true });
    };
  }, []);

  return (
    <section aria-label="Hermes chat" className="hermes-chat-shell hermes-genm agents-sumi-e">
      <header className="hermes-topbar hermes-topbar--status-only">
        <div className="hermes-title-block">
          <p className="kana-line">代理</p>
          <h2>Hermes Agents</h2>
          <p className="hermes-reference-line">Agent command room · local Hermes runtime · fail-closed execution</p>
        </div>
        <div className="hermes-genm-ink-mark agents-ink-mark" aria-hidden="true"><span /><span /><span /></div>
        <div className="topbar-status-stack">
          <div className="connection-panel" aria-live="polite">
            <span className={`status-dot ${statusTone(connectionState)}`} aria-hidden="true" />
            <span className="connection-status-copy">
              <span>Hermes CLI {connectionState.toUpperCase()}</span>
              <span className="status-label-jp">{CONNECTION_STATE_JAPANESE[connectionState]}</span>
            </span>
          </div>
          <div className="repository-link-control repository-link-control--topbar">
            <label htmlFor="linked-repository-select"><span>Repository</span><span className="control-label-jp">接続</span></label>
            <GlobalDropdown
              disabled={repositories.length === 0}
              id="linked-repository-select"
              label="Link repository"
              onChange={handleLinkedRepositoryChange}
              options={[
                { value: "none", label: "Unlinked / 未接続" },
                ...repositories.map((repository) => ({
                  value: repository.id,
                  label: repository.name,
                  meta: `${repository.branch || "unknown"} — ${repository.path}`,
                })),
              ]}
              size="compact"
              value={selectedRepository ? activeRepositoryId : "none"}
            />
          </div>
          <button aria-label={fileManagerOpen ? "Close file manager sidebar" : "Open file manager sidebar"} aria-pressed={fileManagerOpen} className="file-manager-toggle-button" onClick={handleFileManagerToggle} title="Open file manager" type="button">
            <FolderTree size={15} strokeWidth={2.4} aria-hidden="true" />
            <span>Files</span>
            <span className="button-label-jp" aria-hidden="true">書類</span>
          </button>
        </div>
      </header>

      <div className={["chat-workspace", fileManagerVisible ? "chat-workspace--file-manager-mounted" : "", fileManagerOpen ? "chat-workspace--file-manager-open" : "", isFileManagerResizing ? "chat-workspace--file-manager-resizing" : ""].filter(Boolean).join(" ")} ref={chatWorkspaceRef} style={chatWorkspaceStyle}>
        <aside className={isSessionsRailCompact ? "sessions-rail sessions-rail--compact" : sessionsRailWidth < 160 ? "sessions-rail sessions-rail--narrow" : sessionsRailWidth < 184 ? "sessions-rail sessions-rail--medium" : "sessions-rail sessions-rail--wide"} aria-label="Opened Hermes sessions" data-session-rail-morph-panel>
          <div className="sessions-rail-header">
            <span className="sessions-rail-title">Sessions</span>
            <span className="sessions-rail-count" aria-label={`${sessions.length} sessions`}>{sessions.length}</span>
            <button aria-label={isSessionsRailCompact ? "Maximize sessions rail" : "Minimize sessions rail"} aria-pressed={isSessionsRailCompact} className="sessions-rail-morph-button" onClick={handleSessionsRailMorphToggle} title={isSessionsRailCompact ? "Maximize sessions rail" : "Minimize sessions rail"} type="button">
              {isSessionsRailCompact ? <Maximize2 size={13} strokeWidth={2.4} /> : <Minimize2 size={13} strokeWidth={2.4} />}
            </button>
          </div>
          <div className="sessions-list" onScroll={updateSessionsOverflowCue} ref={sessionsListRef} role="list">
            <div className="session-tab-row session-tab-row--new" role="listitem">
              <button aria-label="New session" className="session-tab session-new-button" data-session-rail-morph-item="new-session" onClick={handleNewSession} title={isSessionsRailCompact ? "New session" : undefined} type="button">
                <span className="session-tab-icon session-new-icon" aria-hidden="true"><Plus size={16} strokeWidth={2.6} /></span>
                <span className="session-tab-title">New session</span>
                <span className="session-tab-meta">Start a clean Hermes terminal thread</span>
              </button>
            </div>
            {sessions.map((session) => {
              const sessionRepository = repositories.find((repository) => repository.id === session.linkedRepositoryId) ?? (session.id === activeSessionId ? selectedRepository : undefined);
              const isEditingSession = editingSessionId === session.id;
              const sessionPortrait = getSessionAgentAvatar(session.id, session.portraitId);
              const portraitStyle = sessionPortraitStyle(session, sessions);
              const needsReply = Boolean(session.needsReply);
              const sessionRuntime = runtime.getSessionRuntime(session.id);
              const isSessionIdle = sessionRuntime.status === "idle";
              const isSessionRunning = sessionRuntime.status === "running";
              const hasQueuedPrompts = sessionRuntime.queuedPrompts.length > 0;
              const sessionRowClassName = [
                "session-tab-row",
                needsReply ? "session-tab-row--needs-reply" : "",
                `session-tab-row--${sessionRuntime.status}`,
                hasQueuedPrompts ? "session-tab-row--queued" : "",
              ].filter(Boolean).join(" ");
              const portraitClassName = [
                "session-tab-icon session-tab-portrait session-tab-drag-handle",
                isSessionIdle ? "session-tab-portrait--idle" : "",
              ].filter(Boolean).join(" ");

              return (
                <div
                  className={sessionRowClassName}
                  data-dashboard-drag-session={isEditingSession ? undefined : session.id}
                  draggable={!isEditingSession}
                  key={session.id}
                  onDragEnd={isEditingSession ? undefined : handleSessionIconDragEnd}
                  onDragStart={isEditingSession ? undefined : (event) => handleSessionIconDragStart(event, session)}
                  onPointerDown={isEditingSession ? undefined : (event) => handleSessionIconPointerDown(event, session)}
                  role="listitem"
                >
                  {needsReply ? (
                    <span className="session-reply-indicator" aria-label="Hermes replied and needs your reply" title="Hermes replied and needs your reply">
                      <BellDot size={13} strokeWidth={2.5} aria-hidden="true" />
                    </span>
                  ) : null}
                  {isEditingSession ? (
                    <div className="session-tab session-tab--editing" data-session-rail-morph-item={session.id} style={portraitStyle}>
                      <span className="session-tab-icon session-tab-portrait" aria-hidden="true" title={sessionPortrait.name} />
                      <input
                        aria-label={`Rename session ${session.title}`}
                        autoFocus
                        className="session-rename-input"
                        onBlur={commitRenameSession}
                        onChange={(event) => setEditingSessionTitle(event.target.value)}
                        onKeyDown={handleRenameKeyDown}
                        value={editingSessionTitle}
                      />
                      <span className="session-tab-meta">{repositoryLabel(sessionRepository)}</span>
                    </div>
                  ) : (
                    <button
                      aria-label={`Open session ${session.title}`}
                      aria-current={session.id === activeSessionId ? "page" : undefined}
                      className={session.id === activeSessionId ? "session-tab active" : "session-tab"}
                      data-dashboard-drag-session={session.id}
                      data-session-rail-morph-item={session.id}
                      draggable
                      onClick={() => openSession(session.id)}
                      onContextMenu={(event) => { event.preventDefault(); beginRenameSession(session); }}
                      onDoubleClick={() => beginRenameSession(session)}
                      onDragEnd={handleSessionIconDragEnd}
                      onDragStart={(event) => handleSessionIconDragStart(event, session)}
                      onPointerDown={(event) => handleSessionIconPointerDown(event, session)}
                      style={portraitStyle}
                      title={isSessionsRailCompact ? `${session.title} · drag to split chat` : "Double-click/right-click to rename · drag to split chat"}
                      type="button"
                    >
                      <span
                        aria-hidden="true"
                        className={portraitClassName}
                        data-dashboard-drag-session={session.id}
                        draggable
                        onDragEnd={handleSessionIconDragEnd}
                        onDragStart={(event) => handleSessionIconDragStart(event, session)}
                        onPointerDown={(event) => handleSessionIconPointerDown(event, session)}
                        title={`${sessionPortrait.name} · drag to split chat`}
                      />
                      <span className="session-tab-title">{session.title}</span>
                      <span className="session-tab-meta">{repositoryLabel(sessionRepository)}</span>
                    </button>
                  )}
                  {!isSessionsRailCompact ? (
                    <div className="session-row-controls">
                      {sessionRuntime.status !== "idle" ? <span className={`session-runtime-chip session-runtime-chip--${sessionRuntime.status}`}>{sessionRuntime.status}</span> : null}
                      {hasQueuedPrompts ? <span className="session-runtime-chip session-runtime-chip--queued">Q{sessionRuntime.queuedPrompts.length}</span> : null}
                      <button className="session-continue-button" aria-label={`Continue ${session.title}`} disabled={isSessionRunning || connectionState !== "online"} onClick={() => void continueDashboardSession(session.id)} type="button">{isSessionRunning ? "Running" : "Continue"}</button>
                      <button aria-label={`Archive session ${session.title}`} className="archive-session-button" onClick={() => onArchiveSession(session.id)} title="Archive session" type="button">
                        <Archive size={14} strokeWidth={2.4} aria-hidden="true" />
                      </button>
                    </div>
                  ) : null}
                </div>
              );
            })}
          </div>
          {showSessionsOverflowCue ? (
            <button
              aria-label="More sessions below"
              className="sessions-overflow-cue"
              onClick={handleSessionsOverflowCueClick}
              title="More sessions below"
              type="button"
            >
              <ChevronDown size={16} strokeWidth={2.8} aria-hidden="true" />
            </button>
          ) : null}
          {!isSessionsRailCompact ? (
            <button
              aria-label="Drag to resize Sessions rail"
              aria-orientation="vertical"
              aria-valuemax={SESSIONS_RAIL_MAX_WIDTH}
              aria-valuemin={SESSIONS_RAIL_MIN_WIDTH}
              aria-valuenow={sessionsRailWidth}
              className="sessions-rail-resize-handle"
              onKeyDown={handleSessionsRailResizeKeyDown}
              onPointerDown={handleSessionsRailResizeStart}
              role="separator"
              title="Drag to resize Sessions rail"
              type="button"
            />
          ) : null}
        </aside>

        <div
          className={isChatDropArmed ? "chat-main-pane chat-main-pane--dashboard chat-main-pane--drop-armed" : "chat-main-pane chat-main-pane--dashboard"}
          data-drop-cue={dashboardDropCue}
          onDragLeave={handleChatDashboardDragLeave}
          onDragOver={handleChatDashboardDragOver}
          onDrop={handleChatDashboardDrop}
          ref={chatDashboardDropRef}
        >
          <div className="agent-monitor-bar" aria-label="Agent monitor controls">
            <div className="agent-monitor-status-cluster" aria-label="Agent dashboard status">
              <span><b className="stat-number-accent">{dashboardState.tiledSessionIds.length}</b> tiled</span>
              <span><b className="stat-number-accent">{runningCount}</b> running</span>
              <span><b className="stat-number-accent">{needsReplyCount}</b> needs reply</span>
              <span><b className="stat-number-accent">{queuedCount}</b> queued</span>
            </div>
            <div className="agent-monitor-command-cluster">
              <button type="button" aria-pressed={dashboardState.autoPrioritize} onClick={() => patchDashboard((current) => ({ ...current, autoPrioritize: !current.autoPrioritize }))}>Auto-prioritize</button>
              <button type="button" onClick={showActiveAgents}>Active agents</button>
              <button type="button" onClick={() => patchDashboard((current) => ({ ...current, tiledSessionIds: [], primarySessionId: undefined, focusedSessionId: undefined }))}>Clear</button>
            </div>
          </div>
          {expandedSession ? (
            <div className="agent-expanded-chat">
              <button type="button" onClick={() => setExpandedModeSessionId(null)}>Back to dashboard · {runningCount} running</button>
              <div className="chat-stage" onPointerDown={handleChatStagePointerDown}>
                <div className="message-list" ref={messageListRef} role="log" aria-live="polite" aria-label="Hermes conversation messages">
                  {expandedSession.messages.map((message, index) => {
                    const userTurnsAfterMessage = expandedSession.messages.slice(index + 1).filter((item) => item.role === "user").length;
                    return <MessageBubble key={message.id} canRollback={userTurnsAfterMessage > 0 && !isSending} message={message} onRollback={() => void handleRollbackToMessage(index)} participant={message.participantId === "hermes" ? hermesWithPresence : participantsById[message.participantId]} />;
                  })}
                </div>
              </div>
            </div>
          ) : dashboardVisibleSessions.length === 0 ? (
            <div className="chat-stage" onPointerDown={handleChatStagePointerDown}>
              <div className="message-list" ref={messageListRef} role="log" aria-live="polite" aria-label="Hermes conversation messages">
                {messages.map((message, index) => {
                  const userTurnsAfterMessage = messages.slice(index + 1).filter((item) => item.role === "user").length;
                  return <MessageBubble key={message.id} canRollback={userTurnsAfterMessage > 0 && !isSending} message={message} onRollback={() => void handleRollbackToMessage(index)} participant={message.participantId === "hermes" ? hermesWithPresence : participantsById[message.participantId]} />;
                })}
              </div>
            </div>
          ) : (
            <div className={`agent-monitor-grid agent-monitor-grid--${dashboardState.layoutMode} agent-monitor-grid--count-${dashboardVisibleSessions.length}${draggedDashboardSessionId ? " agent-monitor-grid--drag-active" : ""}`}>
              {dashboardVisibleSessions.map((session) => {
                const panelRuntime = runtime.getSessionRuntime(session.id);
                const isPanelRunning = panelRuntime.status === "running" || panelRuntime.status === "needs-input";
                const panelQueueOnly = !isPanelRunning && runningCount >= MAX_ACTIVE_AGENT_RUNS;
                const panelDisabled = connectionState !== "online";
                const panelDisabledReason = connectionState !== "online" ? disabledReason : undefined;
                const panelModelInfo = effectiveModelInfoForSession(session);
                return (
                  <AgentMonitorPanel
                    key={session.id}
                    session={session}
                    runtimeState={panelRuntime}
                    repository={repositories.find((repository) => repository.id === session.linkedRepositoryId)}
                    isPrimary={(dashboardState.primarySessionId ?? dashboardVisibleSessions[0]?.id) === session.id}
                    isFocused={(dashboardState.focusedSessionId ?? activeSessionId) === session.id}
                    disabled={panelDisabled}
                    disabledReason={panelDisabledReason}
                    contextUsedPercent={estimateContextUsed(session.messages)}
                    modelLabel={panelModelInfo.modelName}
                    queueOnly={panelQueueOnly}
                    slashCommands={slashCommands}
                    slashCommandSource={slashCommandSource}
                    onFocus={(sessionId) => { openSession(sessionId); patchDashboard((current) => ({ ...current, focusedSessionId: sessionId })); }}
                    onSend={sendToDashboardSession}
                    onStop={(sessionId) => void handleStopHermesRun("button", sessionId)}
                    onContinue={continueDashboardSession}
                    onExpand={(sessionId) => { openSession(sessionId); patchDashboard((current) => ({ ...current, focusedSessionId: sessionId })); setExpandedModeSessionId(sessionId); }}
                    onRemoveFromDashboard={removeSessionFromDashboard}
                    onMakePrimary={(sessionId) => patchDashboard((current) => ({ ...current, primarySessionId: sessionId, focusedSessionId: sessionId }))}
                    onMove={moveDashboardSession}
                  />
                );
              })}
            </div>
          )}
        </div>
        {fileManagerVisible ? (
          <aside className={`file-manager-sidebar ${fileManagerOpen ? "file-manager-sidebar--open" : "file-manager-sidebar--closed"}`} aria-hidden={!fileManagerOpen} aria-label="macOS Finder file manager" inert={!fileManagerOpen}>
            <button
              aria-label="Drag to resize Finder sidebar"
              aria-orientation="vertical"
              aria-valuemax={FILE_MANAGER_MAX_WIDTH}
              aria-valuemin={FILE_MANAGER_MIN_WIDTH}
              aria-valuenow={fileManagerWidth}
              className="file-manager-resize-handle"
              onKeyDown={handleFileManagerResizeKeyDown}
              onPointerDown={handleFileManagerResizeStart}
              role="separator"
              title="Resize Finder sidebar"
              type="button"
            />
            <header className="file-manager-header">
              <div>
                <strong>Finder</strong>
                <span>{fileManagerRootListing?.path ?? "macOS home folder"}</span>
              </div>
              <button aria-label="Close file manager" onClick={() => setFileManagerOpen(false)} type="button"><X size={14} /></button>
            </header>
            <div className="file-manager-toolbar">
              <button onClick={() => void loadFileManagerPath(fileManagerRootListing?.path, { makeRoot: true })} type="button">Refresh</button>
            </div>
            {fileManagerError ? <p className="file-manager-error" role="alert">{fileManagerError}</p> : null}
            <div className="file-manager-tree" aria-busy={fileManagerLoadingPath !== null}>
              {fileManagerRootListing ? renderFileManagerEntries(fileManagerRootListing.path) : <p className="file-manager-empty">Loading Finder folders…</p>}
            </div>
          </aside>
        ) : null}
        {(expandedSession || dashboardState.tiledSessionIds.length === 0) ? (
          <ChatComposer ref={composerRef} disabled={connectionState !== "online"} disabledReason={disabledReason} isSending={isSending} contextUsedPercent={contextUsedPercent} modelLabel={activeModelLabel} slashCommands={slashCommands} slashCommandSource={slashCommandSource} onSend={handleSend} onStop={() => void handleStopHermesRun("button", expandedSession?.id)} />
        ) : null}
      </div>

      <footer className="chat-stats-strip" aria-label="Hermes session stats">
        <span><b>Status</b> {renderNumberAccents(activeTaskStatus)} · <b>Task</b> {renderNumberAccents(formatElapsed(promptElapsed))} · <b>Session</b> {renderNumberAccents(formatElapsed(sessionElapsed))}</span>
        <span><b>Context</b> <span className="stat-number-accent">{contextUsedPercent}%</span> · <span className="stat-number-accent">{compressionCount}</span> compressions</span>
        <span className="chat-stats-model-section">
          <span className="chat-stats-model-copy"><b>Model</b> {renderNumberAccents(activeModelLabel)} · Reasoning {renderNumberAccents(activeReasoningLabel)} · Codex {renderNumberAccents(`${CODEX_USAGE_TODAY} / ${CODEX_USAGE_WEEKLY}`)}</span>
          <button
            aria-label="Change model and reasoning"
            aria-haspopup="dialog"
            className="chat-stats-model-button"
            onClick={() => setActiveCommandPanel("model")}
            title={`Change model and reasoning · ${modelPanelDraft.provider} · reasoning ${activeReasoningLabel}`}
            type="button"
          >
            Tune
          </button>
        </span>
        <span><b>Hermes ID</b> {renderNumberAccents(cliStatus?.session ?? activeSession?.id ?? "most-recent-hermes-cli-session")}</span>
      </footer>

      <CommandPalette
        open={commandPaletteOpen}
        commands={slashCommands}
        recentCommands={recentCommands}
        onClose={() => setCommandPaletteOpen(false)}
        onInsertCommand={(command) => {
          composerRef.current?.insertText(command);
          setCommandPaletteOpen(false);
        }}
        onRunCommand={(command) => {
          setCommandPaletteOpen(false);
          if (!activeSession) return;
          if (activeHermesRunsRef.current.has(activeSession.id) || runtime.getSessionRuntime(activeSession.id).status === "running" || runtime.getSessionRuntime(activeSession.id).status === "needs-input") {
            queueHermesPrompt(activeSession.id, command, "slash");
            runtime.queuePrompt(activeSession.id, command);
            return;
          }
          void runSlashCommand(activeSession, command);
        }}
      />

      {activeCommandPanel ? (
        <div className="command-palette-backdrop" role="presentation" onMouseDown={(event) => { if (event.target === event.currentTarget) setActiveCommandPanel(null); }}>
          <section aria-label={`${COMMAND_PANEL_COPY[activeCommandPanel].title} command panel`} className={`zoid-native-command-panel ${activeCommandPanel === "model" ? "zoid-native-command-panel--model" : ""}`} role="dialog" aria-modal="true" onKeyDown={(event) => trapDialogFocus(event, () => setActiveCommandPanel(null))} ref={activeCommandPanelRef} tabIndex={-1}>
            <header className="command-panel-header">
              <div>
                <span className="command-panel-kicker">{activeCommandPanel === "model" ? "モデル設定" : "Native Zoid command surface"}</span>
                <strong>{COMMAND_PANEL_COPY[activeCommandPanel].title}</strong>
              </div>
              <button aria-label="Close command panel" onClick={() => setActiveCommandPanel(null)} type="button">Close</button>
            </header>
            {activeCommandPanel === "model" ? (
              <div className="model-command-panel">
                <section className="model-command-current" aria-label="Current model selection">
                  <article>
                    <span>Provider</span>
                    <strong title={modelPanelDraft.provider}>{modelPanelDraft.provider}</strong>
                  </article>
                  <article>
                    <span>Model</span>
                    <strong title={activeModelLabel}>{activeModelLabel}</strong>
                  </article>
                  <article>
                    <span>Reasoning</span>
                    <strong>{activeReasoningLabel}</strong>
                  </article>
                </section>
                <section className="model-command-controls" aria-labelledby="model-command-controls-title">
                  <div className="model-command-section-heading">
                    <span>01</span>
                    <div>
                      <h3 id="model-command-controls-title">Choose runtime defaults</h3>
                      <p>{COMMAND_PANEL_COPY.model.body}</p>
                    </div>
                  </div>
                  <div className="model-command-grid" aria-busy={isUpdatingModelPanel}>
                    <label className="model-command-field">
                      <span>Provider</span>
                      <GlobalDropdown disabled={!profileSettings || isUpdatingModelPanel} label="Model provider" onChange={handleModelProviderChange} options={providerOptions} size="compact" value={modelPanelDraft.provider} />
                      <small>Source account or local provider.</small>
                    </label>
                    <label className="model-command-field">
                      <span>Model</span>
                      <GlobalDropdown disabled={!profileSettings || isUpdatingModelPanel || modelOptions.length === 0} label="Model" onChange={handleModelNameChange} options={modelOptions} size="compact" value={modelPanelDraft.model} />
                      <small>Provider-aware list; saved immediately.</small>
                    </label>
                    <label className="model-command-field">
                      <span>Reasoning</span>
                      <GlobalDropdown disabled={!profileSettings || isUpdatingModelPanel} label="Reasoning effort" onChange={handleReasoningChange} options={REASONING_OPTIONS} size="compact" value={modelPanelDraft.reasoning} />
                      <small>Controls default thinking budget.</small>
                    </label>
                  </div>
                </section>
                <section className="model-command-contract" aria-label="Persistence behavior">
                  <span>02</span>
                  <div>
                    <strong title={modelPanelStorageLabel}>Saved to {modelPanelStorageLabel}</strong>
                    <p>Changes persist to Hermes profile settings immediately. Existing CLI sessions may require a new Zoid-launched session before runtime model changes take effect.</p>
                  </div>
                </section>
                {modelPanelStatus ? <p className="model-command-status" role="status">{modelPanelStatus}</p> : null}
                {modelPanelError ? <p className="model-command-error" role="alert">{modelPanelError}</p> : null}
              </div>
            ) : <p>{COMMAND_PANEL_COPY[activeCommandPanel].body}</p>}
            {activeCommandPanel === "agents" ? (
              <div className="ruthless-reviewer-card">
                <div>
                  <span className="ruthless-reviewer-kicker">Ruthless subagent</span>
                  <strong>Line-by-line implementation review</strong>
                  <p>Launches a scoped reviewer that starts from git status/diff, checks changed code for fake wiring or no-op behavior, and reports only Required fixes, proof, access violations, and blocking questions.</p>
                </div>
                <dl>
                  <div><dt>Tools</dt><dd>terminal + file only</dd></div>
                  <div><dt>Side effects</dt><dd>read-only review; no edits, commits, deploys, cron, web, or messaging</dd></div>
                  <div><dt>Target</dt><dd>{selectedRepository ? selectedRepository.path : "linked repository required or reviewer reports blocked"}</dd></div>
                </dl>
                <button disabled={!activeSession || isSending} onClick={handleStartRuthlessCodeReview} type="button">
                  {isSending ? "Hermes busy" : "Run ruthless review"}
                </button>
              </div>
            ) : null}
            {activeCommandPanel !== "model" ? <small>Command behavior is sourced from Hermes; this panel keeps the interaction inside Zoid.</small> : null}
          </section>
        </div>
      ) : null}

      {pendingConfirmation ? (
        <div className="zoid-command-confirm-backdrop" role="presentation">
          <section aria-label="Confirm command" className="zoid-command-confirm" role="dialog" aria-modal="true" onKeyDown={(event) => trapDialogFocus(event, () => setPendingConfirmation(null))} ref={pendingConfirmationRef} tabIndex={-1}>
            <header>
              <strong>Confirm command</strong>
              <span>{pendingConfirmation.result.scope === "global-hermes" ? "Global runtime action" : "Current session action"}</span>
            </header>
            <p>{pendingConfirmation.result.content ?? "This command needs confirmation before Zoid runs it."}</p>
            <code>{pendingConfirmation.result.command}</code>
            <div className="zoid-command-confirm-actions">
              <button autoFocus type="button" onClick={() => setPendingConfirmation(null)}>Cancel</button>
              <button
                className="zoid-command-confirm-run"
                type="button"
                onClick={() => {
                  const pending = pendingConfirmation;
                  setPendingConfirmation(null);
                  void runPendingConfirmedCommand(pending);
                }}
              >
                Run command
              </button>
            </div>
          </section>
        </div>
      ) : null}
    </section>
  );
}
