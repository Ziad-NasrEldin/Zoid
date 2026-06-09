import { AgentsHermesScreen, createSession, refreshHermesWelcomeCopy } from "./agents/AgentsHermesScreen";
import type { ArchivedHermesChatSession, HermesChatSession } from "./agents/AgentsHermesScreen";
import { AutomationsWorkspace } from "./automations/AutomationsWorkspace";
import { BrainWorkspace } from "./brain/BrainWorkspace";
import { CodeWorkspace } from "./code/CodeWorkspace";
import { ContentWorkspace } from "./content/ContentWorkspace";
import type { CodeRepository } from "./code/types";
import { Bot, Brain as BrainIcon, CalendarDays, Code2, FolderKanban, Megaphone, Repeat2, Save, Settings } from "lucide-react";
import type { LucideIcon } from "lucide-react";
import { useEffect, useRef, useState } from "react";
import type { FormEvent, KeyboardEvent, SetStateAction } from "react";
import { flushSync } from "react-dom";
import { defaultHermesProfileSettings, loadHermesProfileSettings, saveHermesProfileSettings, warmFilePermissions } from "./agents/hermesProfileClient";
import type { HermesProfileSettings } from "./agents/hermesProfileClient";
import { ProvidersSettings } from "./providers/ProvidersSettings";
import { listManagedProviders, type ManagedProvider } from "./providers/providerClient";
import { GlobalDropdown } from "./ui/GlobalDropdown";
import { chooseUniqueSessionAgentAvatarId, getSessionAgentAvatarById } from "./agents/sessionPortraits";

type NavigationStatus = "ready" | "idle" | "blocked";

type NavigationItem = {
  label: string;
  meta: string;
  status: NavigationStatus;
  Icon: LucideIcon;
};

type ActiveWorkspace = "Brain" | "Agents" | "Code" | "Content" | "Automations" | "Settings";

const navigationItems: NavigationItem[] = [
  { label: "Brain", meta: "Notes sync", status: "idle", Icon: BrainIcon },
  { label: "Today", meta: "Current work", status: "idle", Icon: CalendarDays },
  { label: "Projects", meta: "Build lanes", status: "idle", Icon: FolderKanban },
  { label: "Agents", meta: "Hermes chat", status: "ready", Icon: Bot },
  { label: "Code", meta: "Repos", status: "idle", Icon: Code2 },
  { label: "Content", meta: "OmniSocials", status: "idle", Icon: Megaphone },
  { label: "Automations", meta: "Routines", status: "idle", Icon: Repeat2 },
  { label: "Settings", meta: "Local app", status: "idle", Icon: Settings },
];

const statusLabel = {
  ready: "ready",
  idle: "empty",
  blocked: "blocked",
} satisfies Record<NavigationStatus, string>;

const LAST_WORKSPACE_STORAGE_KEY = "zoid25:last-active-workspace";
const REPOSITORIES_STORAGE_KEY = "zoid25:code-repositories";
const LINKED_REPOSITORY_STORAGE_KEY = "zoid25:linked-repository-id";
const HERMES_SESSIONS_STORAGE_KEY = "zoid25:hermes-sessions";
const HERMES_ARCHIVED_SESSIONS_STORAGE_KEY = "zoid25:hermes-archived-sessions";
const SIDEBAR_MORPH_TIMING: KeyframeAnimationOptions = {
  duration: 540,
  easing: "cubic-bezier(0.16, 1, 0.3, 1)",
};
const SIDEBAR_MORPH_EXIT_TIMING: KeyframeAnimationOptions = {
  duration: 240,
  easing: "cubic-bezier(0.25, 1, 0.5, 1)",
};

type SidebarMorphSnapshot = {
  clone: HTMLElement;
  key: string | null;
  rect: DOMRect;
};


function splitProfileList(value: string): string[] {
  return Array.from(new Set(value.split(/\r?\n|,/).map((item) => item.trim()).filter(Boolean)));
}

function joinProfileList(values: string[]): string {
  return Array.from(new Set(values.map((item) => item.trim()).filter(Boolean))).sort((a, b) => a.localeCompare(b)).join("\n");
}

function mergeCatalog(enabledValue: string, availableValues: string[]): string[] {
  return Array.from(new Set([...availableValues, ...splitProfileList(enabledValue)])).filter(Boolean).sort((a, b) => a.localeCompare(b));
}

function isActiveWorkspace(value: string | null): value is ActiveWorkspace {
  return value === "Brain" || value === "Agents" || value === "Code" || value === "Content" || value === "Automations" || value === "Settings";
}

function isCodeRepository(value: unknown): value is CodeRepository {
  if (typeof value !== "object" || value === null) return false;
  const repository = value as Partial<CodeRepository>;
  return (
    typeof repository.id === "string" &&
    typeof repository.name === "string" &&
    typeof repository.path === "string" &&
    typeof repository.addedAt === "string" &&
    (repository.branch === undefined || typeof repository.branch === "string") &&
    (repository.defaultBranch === undefined || typeof repository.defaultBranch === "string") &&
    (repository.remoteUrl === undefined || repository.remoteUrl === null || typeof repository.remoteUrl === "string") &&
    (repository.source === "scanned" || repository.source === "cloned") &&
    typeof repository.dirty === "boolean"
  );
}

function isHermesChatSession(value: unknown): value is HermesChatSession {
  if (typeof value !== "object" || value === null) return false;
  const session = value as Partial<HermesChatSession>;
  return (
    typeof session.id === "string" &&
    typeof session.title === "string" &&
    typeof session.createdAt === "string" &&
    typeof session.updatedAt === "string" &&
    Array.isArray(session.messages) &&
    (session.linkedRepositoryId === undefined || typeof session.linkedRepositoryId === "string") &&
    (session.portraitId === undefined || typeof session.portraitId === "string") &&
    (session.needsReply === undefined || typeof session.needsReply === "boolean") &&
    (session.lastNotifiedAssistantMessageId === undefined || typeof session.lastNotifiedAssistantMessageId === "string") &&
    (session.notificationUpdatedAt === undefined || typeof session.notificationUpdatedAt === "string")
  );
}

function isArchivedHermesChatSession(value: unknown): value is ArchivedHermesChatSession {
  return isHermesChatSession(value) && typeof (value as Partial<ArchivedHermesChatSession>).archivedAt === "string";
}

function getInitialWorkspace(): ActiveWorkspace {
  if (typeof window === "undefined") return "Code";

  const storedWorkspace = window.localStorage.getItem(LAST_WORKSPACE_STORAGE_KEY);
  return isActiveWorkspace(storedWorkspace) ? storedWorkspace : "Code";
}

function getInitialRepositories(): CodeRepository[] {
  if (typeof window === "undefined") return [];

  try {
    const storedRepositories = window.localStorage.getItem(REPOSITORIES_STORAGE_KEY);
    if (!storedRepositories) return [];
    const parsedRepositories: unknown = JSON.parse(storedRepositories);
    return Array.isArray(parsedRepositories) ? parsedRepositories.filter(isCodeRepository) : [];
  } catch {
    return [];
  }
}

function getInitialLinkedRepositoryId(): string {
  if (typeof window === "undefined") return "";
  return window.localStorage.getItem(LINKED_REPOSITORY_STORAGE_KEY) ?? "";
}

function getInitialHermesSessions(): HermesChatSession[] {
  if (typeof window === "undefined") return [createSession()];

  try {
    const storedSessions = window.localStorage.getItem(HERMES_SESSIONS_STORAGE_KEY);
    if (!storedSessions) return [createSession()];
    const parsedSessions: unknown = JSON.parse(storedSessions);
    const sessions = Array.isArray(parsedSessions) ? parsedSessions.filter(isHermesChatSession).map(refreshHermesWelcomeCopy) : [];
    const sessionsWithPortraits = sessions.reduce<HermesChatSession[]>((resolvedSessions, session) => {
      if (getSessionAgentAvatarById(session.portraitId)) return [...resolvedSessions, session];
      return [
        ...resolvedSessions,
        {
          ...session,
          portraitId: chooseUniqueSessionAgentAvatarId(resolvedSessions.map((item) => item.portraitId), session.id),
        },
      ];
    }, []);
    return sessionsWithPortraits.length > 0 ? sessionsWithPortraits : [createSession()];
  } catch {
    return [createSession()];
  }
}

function getInitialArchivedHermesSessions(): ArchivedHermesChatSession[] {
  if (typeof window === "undefined") return [];

  try {
    const storedSessions = window.localStorage.getItem(HERMES_ARCHIVED_SESSIONS_STORAGE_KEY);
    if (!storedSessions) return [];
    const parsedSessions: unknown = JSON.parse(storedSessions);
    return Array.isArray(parsedSessions) ? parsedSessions.filter(isArchivedHermesChatSession) : [];
  } catch {
    return [];
  }
}

function StatusDot({ status }: { status: NavigationItem["status"] }) {
  return <span aria-hidden="true" className={`status-dot ${status}`} />;
}

type SettingsArchiveProps = {
  archivedSessions: ArchivedHermesChatSession[];
  onRestoreSession: (sessionId: string) => void;
  onDeleteArchivedSessions: (sessionIds: string[]) => void;
  onDeleteAllArchivedSessions: () => void;
};

type StringProfileKey = {
  [K in keyof HermesProfileSettings]: HermesProfileSettings[K] extends string ? K : never
}[keyof HermesProfileSettings];

type BooleanProfileKey = {
  [K in keyof HermesProfileSettings]: HermesProfileSettings[K] extends boolean ? K : never
}[keyof HermesProfileSettings];

type ProfileTextField = {
  key: StringProfileKey;
  label: string;
  helper: string;
  multiline?: boolean;
  rows?: number;
};

function SettingsArchive({ archivedSessions, onRestoreSession, onDeleteArchivedSessions, onDeleteAllArchivedSessions }: SettingsArchiveProps) {
  const [settings, setSettings] = useState<HermesProfileSettings>(defaultHermesProfileSettings);
  const [managedProviders, setManagedProviders] = useState<ManagedProvider[]>([]);
  const [saveStatus, setSaveStatus] = useState("Loading Hermes profile settings…");
  const [profileLoadError, setProfileLoadError] = useState<string | null>(null);
  const [selectedArchivedSessionIds, setSelectedArchivedSessionIds] = useState<string[]>([]);
  const [activeSettingsSection, setActiveSettingsSection] = useState("identity");
  const [pendingArchiveDelete, setPendingArchiveDelete] = useState<{ sessionIds: string[]; label: string } | null>(null);
  const archiveDeleteCancelButtonRef = useRef<HTMLButtonElement | null>(null);
  const archiveDeleteDialogRef = useRef<HTMLElement | null>(null);
  const archiveDeletePreviousFocusRef = useRef<HTMLElement | null>(null);

  useEffect(() => {
    let active = true;
    loadHermesProfileSettings()
      .then((loaded) => {
        if (!active) return;
        setProfileLoadError(null);
        setSettings(loaded);
        setSaveStatus(`Loaded ${loaded.profile} profile settings`);
      })
      .catch((error: unknown) => {
        if (!active) return;
        const message = error instanceof Error ? error.message : String(error);
        setProfileLoadError(message);
        setSaveStatus(`Could not load profile settings: ${message}`);
      });
    return () => { active = false; };
  }, []);

  useEffect(() => {
    let active = true;
    listManagedProviders().then((loaded) => {
      if (!active) return;
      setManagedProviders(loaded);
    });
    return () => { active = false; };
  }, []);

  useEffect(() => {
    if (!pendingArchiveDelete) return undefined;
    archiveDeletePreviousFocusRef.current = document.activeElement instanceof HTMLElement ? document.activeElement : null;
    window.setTimeout(() => archiveDeleteCancelButtonRef.current?.focus(), 0);
    return () => {
      archiveDeletePreviousFocusRef.current?.focus();
      archiveDeletePreviousFocusRef.current = null;
    };
  }, [pendingArchiveDelete]);

  function updateSettings<K extends keyof HermesProfileSettings>(key: K, value: HermesProfileSettings[K]) {
    setSettings((current) => ({ ...current, [key]: value }));
  }

  function updateTextSetting(key: StringProfileKey, value: string) {
    setSettings((current) => ({ ...current, [key]: value }));
  }

  function updateBooleanSetting(key: BooleanProfileKey, value: boolean) {
    setSettings((current) => ({ ...current, [key]: value }));
  }

  function selectArchivedSession(sessionId: string, selected: boolean) {
    setSelectedArchivedSessionIds((current) => selected ? Array.from(new Set([...current, sessionId])) : current.filter((id) => id !== sessionId));
  }

  function requestArchiveDelete(sessionIds: string[], label: string) {
    if (sessionIds.length === 0) return;
    setPendingArchiveDelete({ sessionIds, label });
  }

  function confirmPendingArchiveDelete() {
    if (!pendingArchiveDelete) return;
    if (pendingArchiveDelete.sessionIds.length === archivedSessions.length) {
      onDeleteAllArchivedSessions();
    } else {
      onDeleteArchivedSessions(pendingArchiveDelete.sessionIds);
    }
    setSelectedArchivedSessionIds((current) => current.filter((id) => !pendingArchiveDelete.sessionIds.includes(id)));
    setPendingArchiveDelete(null);
  }


  function handleArchiveDeleteDialogKeyDown(event: KeyboardEvent<HTMLElement>) {
    if (event.key === "Escape") {
      event.preventDefault();
      setPendingArchiveDelete(null);
      return;
    }
    if (event.key !== "Tab") return;
    const focusable = Array.from(archiveDeleteDialogRef.current?.querySelectorAll<HTMLElement>(
      'button:not(:disabled), [href], input:not(:disabled), textarea:not(:disabled), select:not(:disabled), [tabindex]:not([tabindex="-1"])'
    ) ?? []).filter((element) => element.offsetParent !== null);
    if (focusable.length === 0) return;
    const first = focusable[0];
    const last = focusable[focusable.length - 1];
    if (event.shiftKey && document.activeElement === first) {
      event.preventDefault();
      last.focus();
    } else if (!event.shiftKey && document.activeElement === last) {
      event.preventDefault();
      first.focus();
    }
  }

  function deleteSelectedArchivedSessions() {
    requestArchiveDelete(selectedArchivedSessionIds, "selected archived sessions");
  }

  function deleteAllArchivedSessions() {
    requestArchiveDelete(archivedSessions.map((session) => session.id), "all archived sessions");
  }

  function renderTextField(field: ProfileTextField) {
    const value = String(settings[field.key] ?? "");
    return (
      <label className={field.multiline ? "profile-field profile-field--wide" : "profile-field"} key={field.key}>
        <span>{field.label}</span>
        {field.multiline ? (
          <textarea
            value={value}
            onChange={(event) => updateTextSetting(field.key, event.target.value)}
            rows={field.rows ?? 4}
          />
        ) : (
          <input value={value} onChange={(event) => updateTextSetting(field.key, event.target.value)} />
        )}
        <small>{field.helper}</small>
      </label>
    );
  }

  function renderToggle(key: BooleanProfileKey, label: string, helper: string) {
    return (
      <label className="profile-toggle" key={key}>
        <input
          checked={Boolean(settings[key])}
          onChange={(event) => updateBooleanSetting(key, event.target.checked)}
          type="checkbox"
        />
        <span><strong>{label}</strong><small>{helper}</small></span>
      </label>
    );
  }

  function renderCatalogGroup(label: string, helper: string, key: StringProfileKey, availableValues: string[]) {
    const selectedValues = splitProfileList(String(settings[key] ?? ""));
    const options = mergeCatalog(String(settings[key] ?? ""), availableValues);
    return (
      <section className="profile-catalog-card" aria-label={label} key={key}>
        <div className="profile-catalog-heading">
          <div>
            <h4>{label}</h4>
            <p>{helper}</p>
          </div>
          <span>{selectedValues.length}/{options.length} enabled</span>
        </div>
        <div className="profile-catalog-list" role="list">
          {options.length === 0 ? <p className="repo-empty-state">No {label.toLowerCase()} discovered in this Hermes profile.</p> : null}
          {options.map((option) => {
            const selected = selectedValues.includes(option);
            return (
              <label className="profile-catalog-item" key={option} role="listitem">
                <input
                  checked={selected}
                  onChange={(event) => {
                    const nextValues = event.target.checked
                      ? [...selectedValues, option]
                      : selectedValues.filter((value) => value !== option);
                    updateTextSetting(key, joinProfileList(nextValues));
                  }}
                  type="checkbox"
                />
                <span>{option}</span>
                <small>{selected ? "Enabled" : "Disabled"}</small>
              </label>
            );
          })}
        </div>
      </section>
    );
  }


  async function handleSaveProfile(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (profileLoadError) {
      setSaveStatus(`Save blocked until Hermes profile loads successfully: ${profileLoadError}`);
      return;
    }
    setSaveStatus("Saving Hermes profile settings…");
    try {
      const saved = await saveHermesProfileSettings(settings);
      setSettings(saved);
      setSaveStatus(`Saved to ${saved.storagePath}`);
    } catch (error) {
      setSaveStatus(`Save failed: ${error instanceof Error ? error.message : String(error)}`);
    }
  }

  const memoryBudgetLimit = Math.max(1, settings.memoryCharLimit + settings.userCharLimit);
  const memoryFullness = Math.min(100, Math.round(((settings.hermesMemory.length + settings.preferences.length) / memoryBudgetLimit) * 100));
  const soulChars = settings.hermesSoul.length;
  const usingBrowserProfileFallback = settings.storagePath.startsWith("zoid25:hermes-profile-settings");

  const identityFields: ProfileTextField[] = [
    { key: "userName", label: "Name", helper: "Shown to Zoid/Hermes as the human profile identity." },
    { key: "role", label: "Role", helper: "Codex-style onboarding role/work mode." },
    { key: "preferences", label: "User preferences", helper: "Actual Hermes USER.md content; editing saves back to the active Hermes profile.", multiline: true, rows: 4 },
  ];

  const soulFields: ProfileTextField[] = [
    { key: "hermesMemory", label: "Hermes memory", helper: "Stable facts, project conventions, and recurring setup notes.", multiline: true, rows: 6 },
    { key: "hermesSoul", label: "Hermes system prompt / soul", helper: "Actual Hermes agent.system_prompt from config.yaml; editing saves back to that config key.", multiline: true, rows: 6 },
    { key: "responseMode", label: "Response mode", helper: "How aggressively Hermes asks questions, proceeds, or challenges assumptions.", multiline: true, rows: 3 },
  ];

  const managedProviderModels = managedProviders.reduce<Record<string, string[]>>((models, provider) => {
    models[provider.providerId] = Array.from(new Set([provider.defaultModel, ...provider.modelOptions].filter(Boolean)));
    return models;
  }, {});
  const availableModelOptions = { ...settings.availableModels, ...managedProviderModels };
  const providerOptions = Array.from(new Set([...Object.keys(availableModelOptions), settings.modelProvider].filter(Boolean))).map((provider) => ({ value: provider, label: provider }));
  const modelOptions = Array.from(new Set([...(availableModelOptions[settings.modelProvider] ?? []), settings.modelName].filter(Boolean))).map((model) => ({ value: model, label: model }));
  const styleOptions = Object.entries(settings.styleTemplates).map(([value, label]) => ({ value, label }));
  const timezoneValues = typeof (Intl as typeof Intl & { supportedValuesOf?: (key: "timeZone") => string[] }).supportedValuesOf === "function" ? (Intl as typeof Intl & { supportedValuesOf: (key: "timeZone") => string[] }).supportedValuesOf("timeZone") : ["Africa/Cairo", "UTC"];
  const timezoneOptions = Array.from(new Set([settings.timezone, ...timezoneValues].filter(Boolean))).map((zone) => ({ value: zone, label: zone }));
  const reasoningOptions = ["off", "minimal", "low", "medium", "high", "xhigh"].map((level) => ({ value: level, label: level }));

  const operationsFields: ProfileTextField[] = [
    { key: "profileMode", label: "Profile mode", helper: "Zoid-only launch/profile context; persisted in zoid-profile-settings.json." },
    { key: "defaultWorkdir", label: "Default workdir", helper: "Zoid-only default folder hint for new agent sessions; persisted in zoid-profile-settings.json." },
    { key: "trustedProjects", label: "Trusted projects", helper: "Zoid-only trust list context, one path per line; persisted in zoid-profile-settings.json.", multiline: true, rows: 4 },
  ];

  const activeSectionMeta = [
    { id: "identity", number: "01", title: "Identity", detail: "Name, timezone, communication style, USER profile" },
    { id: "memory", number: "02", title: "Memory & soul", detail: "Hermes MEMORY.md and config system prompt" },
    { id: "models", number: "03", title: "Models", detail: "Provider, model, reasoning effort" },
    { id: "providers", number: "04", title: "Providers", detail: "Zoid-managed provider presets and validation" },
    { id: "tools", number: "05", title: "Tools", detail: "Workdir, MCP, plugins, skills, toolsets" },
    { id: "safety", number: "06", title: "Safety", detail: "Access, approvals, voice, notifications" },
    { id: "archive", number: "07", title: "Archive", detail: `${archivedSessions.length} archived sessions` },
  ];
  const currentSection = activeSectionMeta.find((section) => section.id === activeSettingsSection) ?? activeSectionMeta[0];

  return (
    <section aria-label="Settings" className="settings-archive-shell profile-page-shell profile-page-shell--organized settings-sumi-e">
      <header className="settings-archive-header profile-hero profile-hero--compact settings-hero">
        <div className="settings-hero-copy">
          <p className="kana-line">設定</p>
          <h2>Profile, Memory & Soul</h2>
          <p>Compact control center for the active Hermes profile. Hermes-backed fields save to config.yaml, MEMORY.md, USER.md, Zoid-managed providers, or archived local sessions; Zoid-only fields are labeled as launch/profile context.</p>
          <p className="settings-reference-line">Hermes profile · local memory · provider control · safe archive</p>
        </div>
        <div className="settings-ink-mark" aria-hidden="true"><span /><span /><span /></div>
        <div className="profile-hero-card" aria-label="Active profile summary">
          <span>Active profile</span>
          <strong>{settings.profile}</strong>
          <small>{settings.storagePath}</small>
          {usingBrowserProfileFallback ? <small>Browser fallback: profile text is stored in localStorage on this device and is not encrypted by Zoid.</small> : null}
        </div>
      </header>

      <form className="profile-settings-panel profile-settings-panel--complete" onSubmit={handleSaveProfile}>
        <div className="profile-settings-heading profile-settings-heading--sticky profile-settings-heading--compact">
          <div>
            <h3>{currentSection.title}</h3>
            <p>{currentSection.detail} · Last updated: {settings.updatedAt ? new Date(Number(settings.updatedAt) || settings.updatedAt).toLocaleString() : "Not saved yet"}</p>
          </div>
          <button disabled={Boolean(profileLoadError)} type="submit"><Save size={14} aria-hidden="true" /> Save profile</button>
        </div>

        <section className="profile-settings-workspace" aria-label="Organized profile settings workspace">
          <aside className="profile-settings-nav" aria-label="Settings sections">
            <div className="profile-nav-list" role="tablist" aria-orientation="horizontal">
              {activeSectionMeta.map((section) => (
                <button
                  aria-controls={`profile-section-${section.id}`}
                  aria-selected={activeSettingsSection === section.id}
                  className={activeSettingsSection === section.id ? "active" : ""}
                  id={`profile-tab-${section.id}`}
                  key={section.id}
                  onClick={() => setActiveSettingsSection(section.id)}
                  role="tab"
                  type="button"
                >
                  <span>{section.number}</span>
                  <strong>{section.title}</strong>
                  <small>{section.detail}</small>
                </button>
              ))}
            </div>

            <section className="profile-section profile-section--overview profile-section--overview-rail" aria-label="Profile overview">
              <article><span>Memory</span><strong>{memoryFullness}%</strong><small>{settings.hermesMemory.length + settings.preferences.length}/{memoryBudgetLimit} chars</small><div className="profile-meter"><i style={{ width: `${memoryFullness}%` }} /></div></article>
              <article><span>Soul</span><strong>{soulChars}</strong><small>{settings.soulCharLimit > 0 ? `${soulChars}/${settings.soulCharLimit} chars` : "system prompt chars"}</small></article>
              <article><span>Access</span><strong>{settings.accessMode}</strong><small>{settings.approvalMode} approvals</small></article>
              <article><span>Model</span><strong>{settings.modelName}</strong><small>{settings.modelProvider}</small></article>
            </section>
          </aside>

          <div className="profile-settings-content">
            {activeSettingsSection === "identity" ? (
              <section className="profile-section profile-section--active" id="profile-section-identity" aria-label="Identity and preferences" aria-labelledby="profile-tab-identity" role="tabpanel">
                <div className="profile-section-title"><p>01</p><h3>Identity & preferences</h3><span>Codex onboarding + Hermes USER profile</span></div>
                <div className="profile-grid">
                  {identityFields.map(renderTextField)}
                  <label className="profile-field"><span>Timezone</span><GlobalDropdown label="Timezone" onChange={(nextValue) => updateSettings("timezone", nextValue)} options={timezoneOptions} value={settings.timezone} /><small>Saved to Hermes config.yaml timezone and used in Zoid prompt context.</small></label>
                  <label className="profile-field"><span>Communication style template</span><GlobalDropdown label="Communication style template" onChange={(nextValue) => { updateSettings("personalityPreset", nextValue); updateSettings("communicationStyle", settings.styleTemplates[nextValue] ?? nextValue); }} options={styleOptions} value={settings.personalityPreset} /><small>Templates load from Hermes agent.personalities; selected personality saves to display.personality, while the expanded style text is Zoid-only prompt context.</small></label>
                </div>
              </section>
            ) : null}

            {activeSettingsSection === "memory" ? (
              <section className="profile-section profile-section--active" id="profile-section-memory" aria-label="Hermes memory and soul" aria-labelledby="profile-tab-memory" role="tabpanel">
                <div className="profile-section-title"><p>02</p><h3>Hermes memory & soul</h3><span>MEMORY.md plus config.yaml agent.system_prompt</span></div>
                <div className="profile-grid">{soulFields.map(renderTextField)}</div>
                <p className="profile-security-note">Memory loads from Hermes MEMORY.md, preferences from USER.md, and soul from config.yaml agent.system_prompt. Enabled notes are prepended to Zoid-started Hermes prompts. Store preferences and operating notes here — not passwords, API keys, tokens, or private credentials. Browser fallback storage uses localStorage and is device-local, browser-readable, and not encrypted by Zoid.</p>
                <div className="profile-toggle-grid">
                  {renderToggle("memoryEnabled", "Agent memory", "Inject durable Hermes notes into future sessions.")}
                  {renderToggle("userProfileEnabled", "User profile", "Inject your USER profile into future sessions.")}
                  {renderToggle("autoContextEnabled", "Auto context", "Let Zoid gather relevant local context before a run.")}
                </div>
              </section>
            ) : null}

            {activeSettingsSection === "models" ? (
              <section className="profile-section profile-section--active" id="profile-section-models" aria-label="Model and provider settings" aria-labelledby="profile-tab-models" role="tabpanel">
                <div className="profile-section-title"><p>03</p><h3>Models & reasoning</h3><span>Loaded from real Hermes config/profile storage; model/provider are used by Zoid-launched sessions</span></div>
                <div className="profile-grid">
                  <label className="profile-field"><span>Main provider</span><GlobalDropdown label="Main provider" onChange={(nextValue) => { updateSettings("modelProvider", nextValue); updateSettings("modelName", availableModelOptions[nextValue]?.[0] ?? ""); }} options={providerOptions} value={settings.modelProvider} /><small>Loaded from configured Hermes entries plus Zoid-managed providers as soon as they are saved.</small></label>
                  <label className="profile-field"><span>Main model</span><GlobalDropdown label="Main model" onChange={(nextValue) => updateSettings("modelName", nextValue)} options={modelOptions} value={settings.modelName} /><small>Used by Zoid-launched Hermes CLI sessions via --provider/--model.</small></label>
                  <label className="profile-field"><span>Reasoning effort</span><GlobalDropdown label="Reasoning effort" onChange={(nextValue) => updateSettings("reasoningEffort", nextValue)} options={reasoningOptions} value={settings.reasoningEffort} /><small>Loaded from Hermes agent.reasoning_effort and saved back to profile settings.</small></label>
                </div>
              </section>
            ) : null}

            {activeSettingsSection === "providers" ? (
              <section className="profile-section profile-section--active" id="profile-section-providers" aria-label="Provider management settings" aria-labelledby="profile-tab-providers" role="tabpanel">
                <ProvidersSettings
                  availableModels={availableModelOptions}
                  onProvidersChanged={setManagedProviders}
                  onSelectMainProvider={(providerId, modelName) => {
                    updateSettings("modelProvider", providerId);
                    updateSettings("modelName", modelName);
                  }}
                />
              </section>
            ) : null}

            {activeSettingsSection === "tools" ? (
              <section className="profile-section profile-section--active" id="profile-section-tools" aria-label="Tools integrations and projects" aria-labelledby="profile-tab-tools" role="tabpanel">
                <div className="profile-section-title"><p>05</p><h3>Tools, MCP, plugins & projects</h3><span>Discovered from the active Hermes profile. Toolsets, MCP servers, plugins, and skills render as toggleable lists instead of raw editable text fields.</span></div>
                <div className="profile-grid profile-grid--operations">{operationsFields.map(renderTextField)}</div>
                <div className="profile-catalog-grid profile-catalog-grid--control-plane">
                  {renderCatalogGroup("Toolsets", "Runtime toolsets Zoid can pass to Hermes sessions.", "toolsets", settings.availableToolsets)}
                  {renderCatalogGroup("MCP servers", "Configured Hermes MCP servers from config.yaml.", "mcpServers", settings.availableMcpServers)}
                  {renderCatalogGroup("Plugins", "Hermes plugins declared in the active profile.", "plugins", settings.availablePlugins)}
                  {renderCatalogGroup("Skills", "All Hermes skills discovered from the active Hermes CLI/profile. Toggle state is sourced from and saved to Hermes skills.disabled.", "enabledSkills", settings.availableSkills)}
                </div>
                <div className="profile-toggle-grid">
                  {renderToggle("webSearchEnabled", "Web search", "Allow live web research when needed.")}
                  {renderToggle("browserToolsEnabled", "Browser tools", "Allow browser automation and screenshots.")}
                  {renderToggle("terminalToolsEnabled", "Terminal", "Allow shell/build/test/process commands.")}
                  {renderToggle("fileToolsEnabled", "File edits", "Allow read/write/patch/search tools.")}
                  {renderToggle("cronEnabled", "Automations", "Allow Hermes cron/watchers from this profile.")}
                </div>
              </section>
            ) : null}

            {activeSettingsSection === "safety" ? (
              <section className="profile-section profile-section--active" id="profile-section-safety" aria-label="Security privacy and notifications" aria-labelledby="profile-tab-safety" role="tabpanel">
                <div className="profile-section-title"><p>06</p><h3>Safety, privacy, voice & notifications</h3><span>Approval, access, notifications, and voice controls backed by Hermes profile/config values</span></div>
                <div className="profile-mode-grid">
                  <label className="profile-field"><span>Access mode</span><GlobalDropdown label="Access mode" onChange={(nextValue) => updateSettings("accessMode", nextValue as HermesProfileSettings["accessMode"])} options={[{ value: "safe", label: "Safe/read-only" }, { value: "workspace", label: "Standard write tools" }, { value: "full", label: "Full access" }]} value={settings.accessMode} /><small>Safe/standard modes map to Hermes --toolsets for normal Zoid-launched chat; full means Zoid sends no toolset override, and this does not sandbox the filesystem path.</small></label>
                  <label className="profile-field"><span>Approval mode</span><GlobalDropdown label="Approval mode" onChange={(nextValue) => updateSettings("approvalMode", nextValue as HermesProfileSettings["approvalMode"])} options={[{ value: "manual", label: "Manual" }, { value: "smart", label: "Smart" }, { value: "off", label: "Off / YOLO" }]} value={settings.approvalMode} /><small>Saved to Hermes approvals.mode in config.yaml; new Hermes sessions consume it from config.</small></label>
                  {renderTextField({ key: "gatewayPlatforms", label: "Gateway platforms", helper: "Summarizes configured Hermes gateway platforms without exposing credentials.", multiline: true, rows: 3 })}
                  <label className="profile-field"><span>Notifications</span><GlobalDropdown label="Notifications" onChange={(nextValue) => updateSettings("notificationPreference", nextValue)} options={[{ value: "off", label: "Off" }, { value: "important", label: "Important only" }, { value: "all", label: "All completion alerts" }]} value={settings.notificationPreference} /><small>Writes Hermes display.background_process_notifications and completion bell preference.</small></label>
                  <label className="profile-field"><span>Voice / STT / TTS</span><GlobalDropdown label="Voice / STT / TTS" onChange={(nextValue) => updateSettings("voicePreference", nextValue as HermesProfileSettings["voicePreference"])} options={[{ value: "off", label: "Off" }, { value: "tts", label: "TTS only" }, { value: "voice", label: "STT + TTS" }]} value={settings.voicePreference} /><small>Writes Hermes STT and voice.auto_tts flags; keeps the configured TTS provider unchanged.</small></label>
                </div>
                <div className="profile-toggle-grid">
                  {renderToggle("checkpointsEnabled", "Filesystem checkpoints", "Snapshot files before risky edits when enabled.")}
                  {renderToggle("secretRedactionEnabled", "Secret redaction", "Keep API keys/tokens out of logs and tool context.")}
                  {renderToggle("piiRedactionEnabled", "PII redaction", "Strip or hash personal identifiers in gateway contexts.")}
                </div>
              </section>
            ) : null}

            {activeSettingsSection === "archive" ? (
              <section className="settings-archive-section profile-section profile-archive-section profile-section--active" id="profile-section-archive" aria-label="Archived agent sessions" aria-labelledby="profile-tab-archive" role="tabpanel">
                <div className="settings-archive-header settings-archive-header--compact profile-section-title">
                  <p>07</p><h2>Archived agent sessions</h2><span>Deleted agent sessions move here instead of disappearing. Restore one to send it back to Agents.</span>
                  {archivedSessions.length > 0 ? <div className="archive-bulk-actions"><button onClick={deleteSelectedArchivedSessions} disabled={selectedArchivedSessionIds.length === 0} type="button">Delete selected ({selectedArchivedSessionIds.length})</button><button onClick={deleteAllArchivedSessions} type="button">Delete all archived</button></div> : null}
                </div>
                {archivedSessions.length === 0 ? (
                  <p className="repo-empty-state">No archived sessions yet.</p>
                ) : (
                  <div className="archived-session-list" role="list">
                    {archivedSessions.map((session) => (
                      <article className="archived-session-card" key={session.id} role="listitem">
                        <label className="archive-session-select"><input checked={selectedArchivedSessionIds.includes(session.id)} onChange={(event) => selectArchivedSession(session.id, event.target.checked)} type="checkbox" /><span>Select</span></label>
                        <div>
                          <h3>{session.title}</h3>
                          <p>{session.messages.length} messages · Archived {new Date(session.archivedAt).toLocaleString()}</p>
                        </div>
                        <button onClick={() => onRestoreSession(session.id)} type="button">Restore session</button>
                        <button onClick={() => requestArchiveDelete([session.id], `“${session.title}”`)} type="button">Delete</button>
                      </article>
                    ))}
                  </div>
                )}
              </section>
            ) : null}
          </div>
        </section>

        <p className="profile-save-status" role="status">{saveStatus}</p>
      </form>
      {pendingArchiveDelete ? (
        <div className="settings-confirm-backdrop" role="presentation">
          <section aria-modal="true" aria-labelledby="settings-confirm-title" className="settings-confirm-panel" onKeyDown={handleArchiveDeleteDialogKeyDown} ref={archiveDeleteDialogRef} role="dialog">
            <p className="kana-line">確認</p>
            <h3 id="settings-confirm-title">Delete archive record?</h3>
            <p>Delete {pendingArchiveDelete.sessionIds.length} archived agent session{pendingArchiveDelete.sessionIds.length === 1 ? "" : "s"} from {pendingArchiveDelete.label}. This is permanent and will not touch active sessions.</p>
            <div className="settings-confirm-actions">
              <button onClick={() => setPendingArchiveDelete(null)} ref={archiveDeleteCancelButtonRef} type="button">Cancel</button>
              <button className="settings-confirm-danger" onClick={confirmPendingArchiveDelete} type="button">Delete</button>
            </div>
          </section>
        </div>
      ) : null}
    </section>
  );
}

export default function App() {
  const shellRef = useRef<HTMLElement | null>(null);
  const sidebarMorphAnimationsRef = useRef<Animation[]>([]);
  const [activeWorkspace, setActiveWorkspace] = useState<ActiveWorkspace>(getInitialWorkspace);
  const [repositories, setRepositories] = useState<CodeRepository[]>(getInitialRepositories);
  const [linkedRepositoryId, setLinkedRepositoryId] = useState(getInitialLinkedRepositoryId);
  const [hermesSessions, setHermesSessions] = useState<HermesChatSession[]>(getInitialHermesSessions);
  const [activeHermesSessionId, setActiveHermesSessionId] = useState(() => hermesSessions[0]?.id ?? createSession().id);
  const [archivedHermesSessions, setArchivedHermesSessions] = useState<ArchivedHermesChatSession[]>(getInitialArchivedHermesSessions);
  const [isSidebarCollapsed, setIsSidebarCollapsed] = useState(false);
  const [automationsStatus, setAutomationsStatus] = useState<NavigationStatus>("idle");
  const [startupNotice, setStartupNotice] = useState<string | null>(null);
  const hasHermesWaitingNotification = hermesSessions.some((session) => session.needsReply);

  useEffect(() => {
    warmFilePermissions(false).catch((error: unknown) => {
      setStartupNotice(`Hermes profile permission warmup failed: ${error instanceof Error ? error.message : String(error)}`);
    });
  }, []);

  useEffect(() => {
    window.localStorage.setItem(LAST_WORKSPACE_STORAGE_KEY, activeWorkspace);
  }, [activeWorkspace]);

  useEffect(() => {
    window.localStorage.setItem(REPOSITORIES_STORAGE_KEY, JSON.stringify(repositories));
  }, [repositories]);

  useEffect(() => {
    window.localStorage.setItem(LINKED_REPOSITORY_STORAGE_KEY, linkedRepositoryId);
  }, [linkedRepositoryId]);

  useEffect(() => {
    window.localStorage.setItem(HERMES_ARCHIVED_SESSIONS_STORAGE_KEY, JSON.stringify(archivedHermesSessions));
  }, [archivedHermesSessions]);

  useEffect(() => {
    window.localStorage.setItem(HERMES_SESSIONS_STORAGE_KEY, JSON.stringify(hermesSessions));
  }, [hermesSessions]);

  useEffect(() => {
    if (hermesSessions.length === 0) {
      const nextSession = createSession();
      setHermesSessions([nextSession]);
      setActiveHermesSessionId(nextSession.id);
      return;
    }

    if (!hermesSessions.some((session) => session.id === activeHermesSessionId)) {
      setActiveHermesSessionId(hermesSessions[0].id);
    }
  }, [activeHermesSessionId, hermesSessions]);

  function handleHermesSessionsChange(nextSessions: SetStateAction<HermesChatSession[]>) {
    setHermesSessions(nextSessions);
  }


  function handleArchiveHermesSession(sessionId: string) {
    const sessionToArchive = hermesSessions.find((session) => session.id === sessionId);
    if (!sessionToArchive) return;

    const archivedSession: ArchivedHermesChatSession = { ...sessionToArchive, archivedAt: new Date().toISOString() };
    const nextSessions = hermesSessions.filter((session) => session.id !== sessionId);
    const safeNextSessions = nextSessions.length > 0 ? nextSessions : [createSession()];
    setHermesSessions(safeNextSessions);
    setArchivedHermesSessions((current) => [archivedSession, ...current]);
    if (activeHermesSessionId === sessionId) {
      setActiveHermesSessionId(safeNextSessions[0].id);
    }
  }

  function handleDeleteArchivedHermesSessions(sessionIds: string[]) {
    const ids = new Set(sessionIds);
    setArchivedHermesSessions((current) => current.filter((session) => !ids.has(session.id)));
  }

  function handleDeleteAllArchivedHermesSessions() {
    setArchivedHermesSessions([]);
  }

  function handleRestoreHermesSession(sessionId: string) {
    const archivedSession = archivedHermesSessions.find((session) => session.id === sessionId);
    if (!archivedSession) return;

    const restoredSession: HermesChatSession = {
      id: archivedSession.id,
      title: archivedSession.title,
      createdAt: archivedSession.createdAt,
      updatedAt: archivedSession.updatedAt,
      messages: archivedSession.messages,
      linkedRepositoryId: archivedSession.linkedRepositoryId,
      hermesCliSessionId: archivedSession.hermesCliSessionId,
      portraitId: archivedSession.portraitId,
      needsReply: archivedSession.needsReply,
      lastNotifiedAssistantMessageId: archivedSession.lastNotifiedAssistantMessageId,
      notificationUpdatedAt: archivedSession.notificationUpdatedAt,
    };
    const restoredSessions = [restoredSession, ...hermesSessions];
    const remainingArchivedSessions = archivedHermesSessions.filter((session) => session.id !== sessionId);
    setHermesSessions(restoredSessions);
    setActiveHermesSessionId(restoredSession.id);
    setArchivedHermesSessions(remainingArchivedSessions);
    setActiveWorkspace("Agents");
  }

  function handleSidebarMorphToggle() {
    const shell = shellRef.current;
    const prefersReducedMotion = window.matchMedia("(prefers-reduced-motion: reduce)").matches;

    if (!shell || prefersReducedMotion) {
      setIsSidebarCollapsed((collapsed) => !collapsed);
      return;
    }

    sidebarMorphAnimationsRef.current.forEach((animation) => animation.cancel());
    sidebarMorphAnimationsRef.current = [];
    shell.querySelectorAll("[data-sidebar-morph-clone]").forEach((clone) => clone.remove());

    const previousShellRect = shell.getBoundingClientRect();
    const previousPanelRect = shell.querySelector<HTMLElement>("[data-sidebar-morph-panel]")?.getBoundingClientRect();
    const previousItems: SidebarMorphSnapshot[] = Array.from(shell.querySelectorAll<HTMLElement>("[data-sidebar-morph-item]")).map((element) => ({
      clone: element.cloneNode(true) as HTMLElement,
      key: element.getAttribute("data-sidebar-morph-item"),
      rect: element.getBoundingClientRect(),
    }));

    shell.classList.add("sidebar-morphing");

    flushSync(() => {
      setIsSidebarCollapsed((collapsed) => !collapsed);
    });

    const nextShellRect = shell.getBoundingClientRect();
    const nextPanelRect = shell.querySelector<HTMLElement>("[data-sidebar-morph-panel]")?.getBoundingClientRect();
    const nextItems = new Map(
      Array.from(shell.querySelectorAll<HTMLElement>("[data-sidebar-morph-item]")).map((element) => [element.getAttribute("data-sidebar-morph-item"), element]),
    );
    const previousKeys = new Set(previousItems.map((item) => item.key));

    sidebarMorphAnimationsRef.current.push(
      shell.animate(
        [
          { transform: `translateX(${previousShellRect.left - nextShellRect.left}px)`, filter: "blur(0px)" },
          { transform: "translateX(0)", filter: "blur(0px)" },
        ],
        SIDEBAR_MORPH_TIMING,
      ),
    );

    if (previousPanelRect && nextPanelRect) {
      const panel = shell.querySelector<HTMLElement>("[data-sidebar-morph-panel]");
      if (panel) {
        sidebarMorphAnimationsRef.current.push(
          panel.animate(
            [
              {
                opacity: isSidebarCollapsed ? 0.16 : 0.94,
                transform: `translate(${previousPanelRect.left - nextPanelRect.left}px, 0) scaleX(${previousPanelRect.width / Math.max(nextPanelRect.width, 1)})`,
              },
              { opacity: isSidebarCollapsed ? 1 : 0, transform: "translate(0, 0) scaleX(1)" },
            ],
            SIDEBAR_MORPH_TIMING,
          ),
        );
      }
    }

    previousItems.forEach(({ clone, key, rect }) => {
      const nextElement = nextItems.get(key);
      if (nextElement) {
        const nextRect = nextElement.getBoundingClientRect();
        sidebarMorphAnimationsRef.current.push(
          nextElement.animate(
            [
              {
                opacity: 0.9,
                transform: `translate(${rect.left - nextRect.left}px, ${rect.top - nextRect.top}px) scale(${rect.width / Math.max(nextRect.width, 1)}, ${rect.height / Math.max(nextRect.height, 1)})`,
              },
              { opacity: 1, transform: "translate(0, 0) scale(1, 1)" },
            ],
            SIDEBAR_MORPH_TIMING,
          ),
        );
        return;
      }

      clone.removeAttribute("data-sidebar-morph-item");
      clone.setAttribute("data-sidebar-morph-clone", "true");
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
        SIDEBAR_MORPH_EXIT_TIMING,
      );
      sidebarMorphAnimationsRef.current.push(animation);
      animation.finished.then(() => clone.remove()).catch(() => clone.remove());
    });

    nextItems.forEach((element, key) => {
      if (previousKeys.has(key)) return;
      sidebarMorphAnimationsRef.current.push(
        element.animate(
          [
            { opacity: 0, transform: "translateY(12px) scale(0.92)" },
            { opacity: 1, transform: "translateY(0) scale(1)" },
          ],
          { ...SIDEBAR_MORPH_TIMING, delay: 90, duration: 420 },
        ),
      );
    });

    window.setTimeout(() => shell.classList.remove("sidebar-morphing"), Number(SIDEBAR_MORPH_TIMING.duration));
  }

  const resolvedNavigationItems = navigationItems.map((item) =>
    item.label === "Automations" ? { ...item, status: automationsStatus } : item,
  );

  return (
    <main
      ref={shellRef}
      className={isSidebarCollapsed ? "zoid25-shell sidebar-collapsed" : "zoid25-shell"}
      aria-label="Zoid 25 desktop scaffold"
    >
      <aside className="blue-rail" aria-label="Global controls">
        <button
          aria-label={isSidebarCollapsed ? "Maximize sidebar" : "Minimize sidebar"}
          aria-pressed={isSidebarCollapsed}
          className="rail-menu"
          onClick={handleSidebarMorphToggle}
          title={isSidebarCollapsed ? "Maximize sidebar" : "Minimize sidebar"}
          type="button"
        >
          <span />
          <span />
          <span />
        </button>
        <button className="rail-lettermark" aria-label="Open profile settings" onClick={() => setActiveWorkspace("Settings")} type="button">Z25</button>
        <nav className="rail-nav" aria-label="Compact section navigation">
          {resolvedNavigationItems.map(({ Icon, ...item }) => (
            <button
              aria-current={item.label === activeWorkspace ? "page" : undefined}
              aria-label={`${item.label}: ${item.meta}, ${statusLabel[item.status]}`}
              className={item.label === activeWorkspace ? "rail-nav-item active" : "rail-nav-item"}
              key={item.label}
              onClick={() => {
                if (isActiveWorkspace(item.label)) setActiveWorkspace(item.label);
              }}
              data-sidebar-morph-item={isSidebarCollapsed ? item.label : undefined}
              title={`${item.label} — ${item.meta}`}
              type="button"
            >
              <Icon aria-hidden="true" size={19} strokeWidth={1.9} />
              <StatusDot status={item.status} />
              {item.label === "Agents" && hasHermesWaitingNotification ? <span className="session-notification-dot" aria-label="Agent response waiting" /> : null}
            </button>
          ))}
        </nav>
        <nav className="rail-language" aria-label="Interface language">
          <span>EN</span>
          <span>日本</span>
        </nav>
      </aside>

      <aside className="editorial-sidebar" aria-label="Primary navigation" aria-hidden={isSidebarCollapsed} data-sidebar-morph-panel>
        <header className="brand-block">
          <p className="kana-line">ゾイド</p>
          <h1>
            ZOID<span className="brand-number">25</span>
          </h1>
        </header>

        <nav className="nav-list" aria-label="Zoid 25 sections">
          {resolvedNavigationItems.map((item) => {
            const { Icon } = item;

            return (
              <button
                aria-current={item.label === activeWorkspace ? "page" : undefined}
                className={item.label === activeWorkspace ? "nav-row active" : "nav-row"}
                key={item.label}
                onClick={() => {
                  if (isActiveWorkspace(item.label)) setActiveWorkspace(item.label);
                }}
                data-sidebar-morph-item={!isSidebarCollapsed ? item.label : undefined}
                tabIndex={isSidebarCollapsed ? -1 : undefined}
                type="button"
              >
                <span className={item.label === "Agents" ? "nav-icon nav-icon--agent-session" : "nav-icon"} aria-hidden="true">
                  <Icon size={20} strokeWidth={1.8} />
                  {item.label === "Agents" && hasHermesWaitingNotification ? <span className="session-notification-dot" /> : null}
                </span>
                <span className="nav-title">{item.label}</span>
                <span className="nav-meta">{item.meta}</span>
                <span className="nav-state">
                  <StatusDot status={item.status} />
                  {statusLabel[item.status]}
                </span>
              </button>
            );
          })}
        </nav>
      </aside>

      {startupNotice ? <p className="app-startup-notice" role="status">{startupNotice}</p> : null}

      {activeWorkspace === "Brain" ? (
        <BrainWorkspace />
      ) : activeWorkspace === "Code" ? (
        <CodeWorkspace
          linkedRepositoryId={linkedRepositoryId}
          onLinkedRepositoryIdChange={setLinkedRepositoryId}
          onRepositoriesChange={setRepositories}
          repositories={repositories}
        />
      ) : activeWorkspace === "Automations" ? (
        <AutomationsWorkspace onStatusChange={setAutomationsStatus} />
      ) : activeWorkspace === "Content" ? (
        <ContentWorkspace />
      ) : activeWorkspace === "Settings" ? (
        <SettingsArchive
          archivedSessions={archivedHermesSessions}
          onDeleteAllArchivedSessions={handleDeleteAllArchivedHermesSessions}
          onDeleteArchivedSessions={handleDeleteArchivedHermesSessions}
          onRestoreSession={handleRestoreHermesSession}
        />
      ) : (
        <AgentsHermesScreen
          activeSessionId={activeHermesSessionId}
          isAgentsWorkspaceOpen={activeWorkspace === "Agents"}
          linkedRepositoryId={linkedRepositoryId}
          onActiveSessionIdChange={setActiveHermesSessionId}
          onArchiveSession={handleArchiveHermesSession}
          onLinkedRepositoryIdChange={setLinkedRepositoryId}
          onSessionsChange={handleHermesSessionsChange}
          repositories={repositories}
          sessions={hermesSessions}
        />
      )}
    </main>
  );
}
