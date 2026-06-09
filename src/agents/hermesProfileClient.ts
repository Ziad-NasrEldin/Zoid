import { invoke } from "@tauri-apps/api/core";

export type ProfileAccessMode = "safe" | "workspace" | "full";
export type ApprovalMode = "manual" | "smart" | "off";
export type VoiceMode = "off" | "tts" | "voice";
export type NotificationMode = "off" | "important" | "all";

export type HermesProfileSettings = {
  userName: string;
  role: string;
  timezone: string;
  communicationStyle: string;
  preferences: string;
  hermesMemory: string;
  hermesSoul: string;
  personalityPreset: string;
  responseMode: string;
  modelProvider: string;
  modelName: string;
  reasoningEffort: string;
  auxiliaryModelNotes: string;
  profileMode: string;
  accessMode: ProfileAccessMode;
  approvalMode: ApprovalMode;
  defaultWorkdir: string;
  trustedProjects: string;
  toolsets: string;
  mcpServers: string;
  plugins: string;
  enabledSkills: string;
  gatewayPlatforms: string;
  notificationPreference: string;
  voicePreference: VoiceMode;
  memoryEnabled: boolean;
  userProfileEnabled: boolean;
  autoContextEnabled: boolean;
  webSearchEnabled: boolean;
  browserToolsEnabled: boolean;
  terminalToolsEnabled: boolean;
  fileToolsEnabled: boolean;
  cronEnabled: boolean;
  checkpointsEnabled: boolean;
  secretRedactionEnabled: boolean;
  piiRedactionEnabled: boolean;
  profile: string;
  storagePath: string;
  availableModels: Record<string, string[]>;
  availableSkills: string[];
  availableToolsets: string[];
  availableMcpServers: string[];
  availablePlugins: string[];
  styleTemplates: Record<string, string>;
  memoryCharLimit: number;
  userCharLimit: number;
  soulCharLimit: number;
  updatedAt: string;
};

const PROFILE_SETTINGS_STORAGE_PREFIX = "zoid25:hermes-profile-settings";

export const defaultHermesProfileSettings: HermesProfileSettings = {
  userName: "Ziad Salah",
  role: "Founder / product owner / technical operator",
  timezone: "Africa/Cairo",
  communicationStyle: "Direct, concise, practical, no fluff.",
  preferences: "",
  hermesMemory: "",
  hermesSoul: "",
  personalityPreset: "concise",
  responseMode: "Ask only for critical blockers; proceed on obvious defaults.",
  modelProvider: "openai-codex",
  modelName: "gpt-5.5",
  reasoningEffort: "medium",
  auxiliaryModelNotes: "Auto-select auxiliary models for title, compression, vision, approval, and critique unless pinned.",
  profileMode: "default",
  accessMode: "full",
  approvalMode: "off",
  defaultWorkdir: "~/Zoid",
  trustedProjects: "~/Zoid\n~/.hermes/hermes-agent",
  toolsets: "terminal\nfile\nbrowser\nweb\nskills\nmemory\nsession_search\ndelegation\ncronjob",
  mcpServers: "lean-ctx\ncodegraph\nstitch",
  plugins: "security-guidance",
  enabledSkills: "tauri-desktop-feature-development\nfeature-critique-workflow\nsubagent-driven-development\nhermes-agent",
  gatewayPlatforms: "Discord #hermes",
  notificationPreference: "important",
  voicePreference: "off",
  memoryEnabled: true,
  userProfileEnabled: true,
  autoContextEnabled: true,
  webSearchEnabled: true,
  browserToolsEnabled: true,
  terminalToolsEnabled: true,
  fileToolsEnabled: true,
  cronEnabled: true,
  checkpointsEnabled: false,
  secretRedactionEnabled: true,
  piiRedactionEnabled: false,
  profile: "default",
  storagePath: "Local browser fallback",
  availableModels: {
    "openai-codex": ["gpt-5.5", "gpt-5.4", "gpt-5.4-mini", "gpt-5.3-codex-spark"],
    openai: ["gpt-5.5", "gpt-5.5-pro", "gpt-5.4", "gpt-5.4-pro", "gpt-5.4-mini", "gpt-5.3", "gpt-5.3-mini", "gpt-5-mini"],
  },
  availableSkills: ["tauri-desktop-feature-development", "feature-critique-workflow", "subagent-driven-development", "hermes-agent"],
  availableToolsets: ["browser", "cronjob", "delegation", "file", "memory", "session_search", "skills", "terminal", "todo", "vision", "web"],
  availableMcpServers: ["lean-ctx", "codegraph", "stitch"],
  availablePlugins: ["security-guidance"],
  styleTemplates: {
    concise: "Direct, concise, practical, no fluff.",
    technical: "Technical, precise, evidence-backed, with implementation details.",
    executive: "Executive summary first, risks and decisions called out clearly.",
    teacher: "Patient, explanatory, with examples and tradeoffs.",
    creative: "Creative, exploratory, concept-first, still grounded in constraints.",
  },
  memoryCharLimit: 2500,
  userCharLimit: 1600,
  soulCharLimit: 0,
  updatedAt: "",
};

function asString(value: unknown, fallback = "") {
  return typeof value === "string" ? value : fallback;
}

function asBoolean(value: unknown, fallback: boolean) {
  return typeof value === "boolean" ? value : fallback;
}

function sanitizeSettings(value: Partial<HermesProfileSettings>, profile: string, storagePath: string): HermesProfileSettings {
  const accessMode = value.accessMode === "safe" || value.accessMode === "workspace" || value.accessMode === "full" ? value.accessMode : defaultHermesProfileSettings.accessMode;
  const approvalMode = value.approvalMode === "manual" || value.approvalMode === "smart" || value.approvalMode === "off" ? value.approvalMode : defaultHermesProfileSettings.approvalMode;
  const reasoningEffort = ["off", "minimal", "low", "medium", "high", "xhigh"].includes(asString(value.reasoningEffort)) ? asString(value.reasoningEffort) : defaultHermesProfileSettings.reasoningEffort;
  const notificationPreference = ["off", "important", "all"].includes(asString(value.notificationPreference)) ? asString(value.notificationPreference) : defaultHermesProfileSettings.notificationPreference;
  return {
    userName: asString(value.userName, defaultHermesProfileSettings.userName),
    role: asString(value.role, defaultHermesProfileSettings.role),
    timezone: asString(value.timezone, defaultHermesProfileSettings.timezone),
    communicationStyle: asString(value.communicationStyle, defaultHermesProfileSettings.communicationStyle),
    preferences: asString(value.preferences, defaultHermesProfileSettings.preferences),
    hermesMemory: asString(value.hermesMemory, defaultHermesProfileSettings.hermesMemory),
    hermesSoul: asString(value.hermesSoul, defaultHermesProfileSettings.hermesSoul),
    personalityPreset: asString(value.personalityPreset, defaultHermesProfileSettings.personalityPreset),
    responseMode: asString(value.responseMode, defaultHermesProfileSettings.responseMode),
    modelProvider: asString(value.modelProvider, defaultHermesProfileSettings.modelProvider),
    modelName: asString(value.modelName, defaultHermesProfileSettings.modelName),
    reasoningEffort,
    auxiliaryModelNotes: asString(value.auxiliaryModelNotes, defaultHermesProfileSettings.auxiliaryModelNotes),
    profileMode: asString(value.profileMode, defaultHermesProfileSettings.profileMode),
    accessMode,
    approvalMode,
    defaultWorkdir: asString(value.defaultWorkdir, defaultHermesProfileSettings.defaultWorkdir),
    trustedProjects: asString(value.trustedProjects, defaultHermesProfileSettings.trustedProjects),
    toolsets: asString(value.toolsets, defaultHermesProfileSettings.toolsets),
    mcpServers: asString(value.mcpServers, defaultHermesProfileSettings.mcpServers),
    plugins: asString(value.plugins, defaultHermesProfileSettings.plugins),
    enabledSkills: asString(value.enabledSkills, defaultHermesProfileSettings.enabledSkills),
    gatewayPlatforms: asString(value.gatewayPlatforms, defaultHermesProfileSettings.gatewayPlatforms),
    notificationPreference,
    voicePreference: value.voicePreference === "off" || value.voicePreference === "tts" || value.voicePreference === "voice" ? value.voicePreference : defaultHermesProfileSettings.voicePreference,
    memoryEnabled: asBoolean(value.memoryEnabled, defaultHermesProfileSettings.memoryEnabled),
    userProfileEnabled: asBoolean(value.userProfileEnabled, defaultHermesProfileSettings.userProfileEnabled),
    autoContextEnabled: asBoolean(value.autoContextEnabled, defaultHermesProfileSettings.autoContextEnabled),
    webSearchEnabled: asBoolean(value.webSearchEnabled, defaultHermesProfileSettings.webSearchEnabled),
    browserToolsEnabled: asBoolean(value.browserToolsEnabled, defaultHermesProfileSettings.browserToolsEnabled),
    terminalToolsEnabled: asBoolean(value.terminalToolsEnabled, defaultHermesProfileSettings.terminalToolsEnabled),
    fileToolsEnabled: asBoolean(value.fileToolsEnabled, defaultHermesProfileSettings.fileToolsEnabled),
    cronEnabled: asBoolean(value.cronEnabled, defaultHermesProfileSettings.cronEnabled),
    checkpointsEnabled: asBoolean(value.checkpointsEnabled, defaultHermesProfileSettings.checkpointsEnabled),
    secretRedactionEnabled: asBoolean(value.secretRedactionEnabled, defaultHermesProfileSettings.secretRedactionEnabled),
    piiRedactionEnabled: asBoolean(value.piiRedactionEnabled, defaultHermesProfileSettings.piiRedactionEnabled),
    profile,
    storagePath,
    availableModels: typeof value.availableModels === "object" && value.availableModels !== null ? value.availableModels as Record<string, string[]> : defaultHermesProfileSettings.availableModels,
    availableSkills: Array.isArray(value.availableSkills) ? value.availableSkills.filter((item): item is string => typeof item === "string" && item.trim().length > 0) : defaultHermesProfileSettings.availableSkills,
    availableToolsets: Array.isArray(value.availableToolsets) ? value.availableToolsets.filter((item): item is string => typeof item === "string" && item.trim().length > 0) : defaultHermesProfileSettings.availableToolsets,
    availableMcpServers: Array.isArray(value.availableMcpServers) ? value.availableMcpServers.filter((item): item is string => typeof item === "string" && item.trim().length > 0) : defaultHermesProfileSettings.availableMcpServers,
    availablePlugins: Array.isArray(value.availablePlugins) ? value.availablePlugins.filter((item): item is string => typeof item === "string" && item.trim().length > 0) : defaultHermesProfileSettings.availablePlugins,
    styleTemplates: typeof value.styleTemplates === "object" && value.styleTemplates !== null ? value.styleTemplates as Record<string, string> : defaultHermesProfileSettings.styleTemplates,
    memoryCharLimit: typeof value.memoryCharLimit === "number" && Number.isFinite(value.memoryCharLimit) && value.memoryCharLimit > 0 ? value.memoryCharLimit : defaultHermesProfileSettings.memoryCharLimit,
    userCharLimit: typeof value.userCharLimit === "number" && Number.isFinite(value.userCharLimit) && value.userCharLimit > 0 ? value.userCharLimit : defaultHermesProfileSettings.userCharLimit,
    soulCharLimit: typeof value.soulCharLimit === "number" && Number.isFinite(value.soulCharLimit) && value.soulCharLimit >= 0 ? value.soulCharLimit : defaultHermesProfileSettings.soulCharLimit,
    updatedAt: asString(value.updatedAt, defaultHermesProfileSettings.updatedAt),
  };
}

function fallbackStorageKey(profile = defaultHermesProfileSettings.profile) {
  return `${PROFILE_SETTINGS_STORAGE_PREFIX}:${profile}`;
}

function isRunningInTauri() {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

function readFallbackSettings(profile = defaultHermesProfileSettings.profile): HermesProfileSettings {
  const storagePath = fallbackStorageKey(profile);
  const stored = window.localStorage.getItem(storagePath);
  if (!stored) return sanitizeSettings({}, profile, storagePath);
  try {
    const parsed: unknown = JSON.parse(stored);
    if (typeof parsed !== "object" || parsed === null) {
      return sanitizeSettings({}, profile, storagePath);
    }
    const partial = parsed as Partial<HermesProfileSettings>;
    const nextProfile = typeof partial.profile === "string" && partial.profile.trim() ? partial.profile : profile;
    return sanitizeSettings(partial, nextProfile, fallbackStorageKey(nextProfile));
  } catch {
    return sanitizeSettings({}, profile, storagePath);
  }
}

export async function loadHermesProfileSettings(): Promise<HermesProfileSettings> {
  try {
    const loaded = await invoke<HermesProfileSettings>("load_hermes_profile_settings");
    return sanitizeSettings(loaded, loaded.profile || defaultHermesProfileSettings.profile, loaded.storagePath || "Hermes profile storage");
  } catch (error) {
    if (isRunningInTauri()) {
      throw error;
    }
    return readFallbackSettings();
  }
}

export async function saveHermesProfileSettings(settings: HermesProfileSettings): Promise<HermesProfileSettings> {
  try {
    const saved = await invoke<HermesProfileSettings>("save_hermes_profile_settings", { settings });
    return sanitizeSettings(saved, saved.profile || settings.profile, saved.storagePath || settings.storagePath);
  } catch (error) {
    if (!isRunningInTauri()) {
      const profile = settings.profile.trim() || defaultHermesProfileSettings.profile;
      const storagePath = fallbackStorageKey(profile);
      const fallback = sanitizeSettings({ ...settings, updatedAt: new Date().toISOString() }, profile, storagePath);
      window.localStorage.setItem(storagePath, JSON.stringify(fallback));
      return fallback;
    }
    throw error;
  }
}

export async function warmFilePermissions(force = false): Promise<string[]> {
  try {
    return await invoke<string[]>("warm_file_permissions", { force });
  } catch (error) {
    if (isRunningInTauri()) {
      throw error;
    }
    return [];
  }
}
