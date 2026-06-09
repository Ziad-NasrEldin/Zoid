import { existsSync, readFileSync, readdirSync } from "node:fs";
import { SESSION_AGENT_AVATARS, chooseUniqueSessionAgentAvatarId } from "./agents/sessionPortraits";

const app = readFileSync(new URL("./App.tsx", import.meta.url), "utf8");
const css = readFileSync(new URL("./App.css", import.meta.url), "utf8");
const main = readFileSync(new URL("./main.tsx", import.meta.url), "utf8");
const client = readFileSync(new URL("./agents/hermesClient.ts", import.meta.url), "utf8");
const backend = readFileSync(new URL("../src-tauri/src/lib.rs", import.meta.url), "utf8");
const hermesCommands = readFileSync(new URL("./agents/hermesCommands.ts", import.meta.url), "utf8");
const slashCommandParser = readFileSync(new URL("./agents/slashCommandParser.ts", import.meta.url), "utf8");
const commandPalette = readFileSync(new URL("./agents/CommandPalette.tsx", import.meta.url), "utf8");
const recentCommands = readFileSync(new URL("./agents/recentCommands.ts", import.meta.url), "utf8");
const screen = readFileSync(new URL("./agents/AgentsHermesScreen.tsx", import.meta.url), "utf8");
const sessionState = readFileSync(new URL("./agents/sessionState.ts", import.meta.url), "utf8");
const sessionPortraits = readFileSync(new URL("./agents/sessionPortraits.ts", import.meta.url), "utf8");
const chatComposer = readFileSync(new URL("./agents/ChatComposer.tsx", import.meta.url), "utf8");
const messageBubble = readFileSync(new URL("./agents/MessageBubble.tsx", import.meta.url), "utf8");
const viteConfig = readFileSync(new URL("../vite.config.ts", import.meta.url), "utf8");
const agentationFixed = readFileSync(new URL("./vendor/agentation-fixed.mjs", import.meta.url), "utf8");
const automationsWorkspace = readFileSync(new URL("./automations/AutomationsWorkspace.tsx", import.meta.url), "utf8");
const automationsClient = readFileSync(new URL("./automations/automationClient.ts", import.meta.url), "utf8");
const automationsTypes = readFileSync(new URL("./automations/types.ts", import.meta.url), "utf8");
const profileClient = readFileSync(new URL("./agents/hermesProfileClient.ts", import.meta.url), "utf8");
const providersSettings = readFileSync(new URL("./providers/ProvidersSettings.tsx", import.meta.url), "utf8");
const providerClient = readFileSync(new URL("./providers/providerClient.ts", import.meta.url), "utf8");
const globalDropdown = readFileSync(new URL("./ui/GlobalDropdown.tsx", import.meta.url), "utf8");
const codeWorkspace = readFileSync(new URL("./code/CodeWorkspace.tsx", import.meta.url), "utf8");
const repositoryClient = readFileSync(new URL("./code/repositoryClient.ts", import.meta.url), "utf8");
const repositoryOperations = readFileSync(new URL("./code/repositoryOperations.ts", import.meta.url), "utf8");
const agentNotifications = existsSync(new URL("./agents/agentNotifications.ts", import.meta.url)) ? readFileSync(new URL("./agents/agentNotifications.ts", import.meta.url), "utf8") : "";
const agentMonitorPanel = readFileSync(new URL("./agents/AgentMonitorPanel.tsx", import.meta.url), "utf8");
const ruthlessReviewerAgent = existsSync(new URL("./agents/ruthlessReviewerAgent.ts", import.meta.url)) ? readFileSync(new URL("./agents/ruthlessReviewerAgent.ts", import.meta.url), "utf8") : "";
const brainWorkspace = existsSync(new URL("./brain/BrainWorkspace.tsx", import.meta.url)) ? readFileSync(new URL("./brain/BrainWorkspace.tsx", import.meta.url), "utf8") : "";
const brainClient = existsSync(new URL("./brain/brainClient.ts", import.meta.url)) ? readFileSync(new URL("./brain/brainClient.ts", import.meta.url), "utf8") : "";
const brainTypes = existsSync(new URL("./brain/types.ts", import.meta.url)) ? readFileSync(new URL("./brain/types.ts", import.meta.url), "utf8") : "";
if (!/<LazyAgentsHermesScreen[\s\S]{0,520}repositories=\{repositories\}[\s\S]{0,160}sessions=\{hermesSessions\}/.test(app)) {
  throw new Error("Agents screen must receive repository catalog plus chat sessions, not a global linked repository selection");
}
if (/<LazyAgentsHermesScreen[\s\S]{0,520}(linkedRepositoryId=|onLinkedRepositoryIdChange=)/.test(app)) {
  throw new Error("Global Code repository link must not be passed into Agents chat sessions");
}

for (const requiredAgentRunCleanupInvariant of [
  "function clearActiveHermesRunIfCurrent(sessionId: string, assistantId: string)",
  "activeRun?.assistantId !== assistantId",
  "clearActiveHermesRunIfCurrent(sendingSessionId, assistantId)",
]) {
  if (!screen.includes(requiredAgentRunCleanupInvariant)) {
    throw new Error(`Agent run cleanup must be scoped to the completed assistant run: ${requiredAgentRunCleanupInvariant}`);
  }
}
if (screen.includes("activeHermesRunsRef.current.delete(sendingSessionId)") || screen.includes("activeHermesRunsRef.current.delete(pending.sessionId)")) {
  throw new Error("Agent run cleanup must not delete a session's active run without checking the assistant/run id");
}
for (const requiredAgentPanelKeyboardInvariant of [
  "role=\"group\"",
  "onKeyDown={handlePanelKeyDown}",
  "event.key === \"Enter\" || event.key === \" \"",
]) {
  if (!agentMonitorPanel.includes(requiredAgentPanelKeyboardInvariant)) {
    throw new Error(`Focusable agent monitor panel must have keyboard-equivalent group focus behavior: ${requiredAgentPanelKeyboardInvariant}`);
  }
}
if (agentMonitorPanel.includes("role=\"button\"") && agentMonitorPanel.includes("agent-monitor-composer")) {
  throw new Error("Agent monitor panel must not expose the entire composite panel as a button around nested controls");
}

for (const requiredBrainShell of ["Brain", "Notes sync", "BrainWorkspace", "Apple Notes Brain", "Create Zoid Brain folder"]) {
  if (!app.includes(requiredBrainShell) && !brainWorkspace.includes(requiredBrainShell)) {
    throw new Error(`Apple Notes Brain workspace shell is missing: ${requiredBrainShell}`);
  }
}
for (const requiredBrainClient of ["loadBrainStore", "listAppleNotesFolders", "ensureZoidBrainFolder", "syncAppleNotesSources", "extractBrainNote", "createBrainClarifyingSession", "load_brain_store", "list_apple_notes_folders", "ensure_zoid_brain_folder", "sync_apple_notes_sources", "extract_brain_note", "create_brain_clarifying_session"]) {
  if (!brainClient.includes(requiredBrainClient)) {
    throw new Error(`Apple Notes Brain typed Tauri client is missing: ${requiredBrainClient}`);
  }
}
for (const requiredBrainExtractionSurface of ["Extract tasks", "Task Candidates", "Start clarifying questions", "Clarifying Sessions", "Current question", "Copy brief", "Hermes not executed"]) {
  if (!brainWorkspace.includes(requiredBrainExtractionSurface)) {
    throw new Error(`Apple Notes Brain extraction/clarifying surface is missing: ${requiredBrainExtractionSurface}`);
  }
}
for (const requiredBrainBackend of ["extract_brain_note", "create_brain_clarifying_session", "localHeuristic", "TaskCandidate", "BrainClarificationSession"]) {
  if (!backend.includes(requiredBrainBackend)) {
    throw new Error(`Apple Notes Brain backend extraction command is missing: ${requiredBrainBackend}`);
  }
}
for (const requiredBrainType of ["BrainStore", "AppleNotesSource", "BrainNote", "BrainExtraction", "TaskCandidate", "BrainClarificationSession", "BrainSyncConflict", "AppleNotesFolder"]) {
  if (!brainTypes.includes(requiredBrainType)) {
    throw new Error(`Apple Notes Brain type contract is missing: ${requiredBrainType}`);
  }
}
if (brainWorkspace.includes("<select")) {
  throw new Error("Apple Notes Brain sync setup must not use native select controls");
}
if (!(/\.brain-sumi-e \{[^}]*grid-template-rows: auto auto auto minmax\(0, 1fr\);[^}]*grid-auto-rows: auto;[^}]*align-content: start;/.test(css) || /\.brain-workspace-shell:not\(\.brain-sumi-e\) \{[^}]*grid-template-rows: auto auto auto minmax\(0, 1fr\);[^}]*grid-auto-rows: auto;[^}]*align-content: start;/.test(css)) || !/\.brain-sumi-e \.brain-link-panel \{[^}]*position: relative;/.test(css)) {
  throw new Error("Apple Notes Brain link panel must own an auto-height row above the inbox grid instead of being buried behind later panels");
}
for (const requiredBrainSumiEScrollAndBrush of [
  "height: 100vh;",
  "overflow-y: auto;",
  "scrollbar-color: var(--brain-seal)",
  ".brain-sumi-e::-webkit-scrollbar-thumb",
  "--brain-seal: #c23a2e;",
  "--brain-seal-deep: #8f211a;",
  ".brain-sumi-e .brain-hero::before",
  ".brain-sumi-e .brain-link-panel::before",
  ".brain-sumi-e .brain-ink-mark::after",
  ".brain-sumi-e .brain-note-row { display: grid; grid-template-columns: minmax(0, 1fr) auto auto;",
  ".brain-sumi-e .brain-source-row, .brain-sumi-e .brain-note-row { grid-template-columns: minmax(0, 1fr); align-items: start;",
]) {
  if (!css.includes(requiredBrainSumiEScrollAndBrush)) {
    throw new Error(`Brain sumi-e pilot is missing scroll/red/brush design-system detail: ${requiredBrainSumiEScrollAndBrush}`);
  }
}
if (!app.includes("Zoid 25")) {
  throw new Error("Zoid 25 brand label is missing");
}

if (!app.includes('ZOID<span className="brand-number">25</span>') || !css.includes(".brand-block h1 { display: flex; gap: 0.12em") || !css.includes("white-space: nowrap")) {
  throw new Error("Sidebar brand mark must keep 25 beside ZOID with a small inline gap");
}

if (!css.includes(".kana-line { margin: 0 0 -4px; color: var(--shell-seal);") || !css.includes(".zoid25-shell .kana-line { color: var(--shell-seal); }") || !css.includes(".settings-archive-header p.kana-line { color: var(--kujo-blue); }") || !css.includes(".brain-sumi-e .brain-hero p.kana-line")) {
  throw new Error("Global shell kana subheadings must use the sumi-e seal accent while scoped page overrides remain explicit");
}

for (const requiredSumiESidebar of [
  "--shell-ink-black: var(--brain-ink-black, #0d0a0a);",
  "--shell-seal: var(--brain-seal, #c23a2e);",
  "--shell-serif-body: var(--brain-serif-body",
  ".ink-rail::before",
  "linear-gradient(180deg, var(--shell-ink-black), #191515 64%, var(--shell-seal-deep))",
  ".editorial-sidebar::before",
  ".nav-row::before",
  ".nav-list::-webkit-scrollbar-thumb",
]) {
  if (!css.includes(requiredSumiESidebar)) {
    throw new Error(`Global sidebar chrome must inherit the sumi-e design system: ${requiredSumiESidebar}`);
  }
}

for (const requiredAgentsSumiE of [
  'className="hermes-chat-shell hermes-genm agents-sumi-e"',
  "Hermes Agents",
  "Agent command room · local Hermes runtime · fail-closed execution",
  ".agents-sumi-e {",
  "--agents-seal: var(--brain-seal, #c23a2e);",
  "--agents-serif-body: var(--brain-serif-body",
  ".agents-sumi-e .chat-stage::before",
  ".agents-sumi-e .message-row--assistant .message-bubble::before",
  "@keyframes agents-ink-reveal",
]) {
  if (!screen.includes(requiredAgentsSumiE) && !css.includes(requiredAgentsSumiE)) {
    throw new Error(`Agents page must inherit the Brain sumi-e design system: ${requiredAgentsSumiE}`);
  }
}

if (!app.includes('aria-label="Primary navigation"')) {
  throw new Error("Primary navigation sidebar scaffold is missing");
}

if (!app.includes("ink-rail")) {
  throw new Error("Sumi-e ink rail is missing");
}

for (const requiredMotionFoundation of [
  "--motion-panel: 380ms;",
  "--ease-editorial: cubic-bezier(0.16, 1, 0.3, 1);",
  "@keyframes motion-paper-panel-enter",
  "@keyframes motion-seal-stamp",
  "@keyframes motion-ink-scan",
  "@keyframes motion-attention-ping",
  ".app-startup-notice--loading::before",
  ".zoid-dropdown.is-open .zoid-dropdown-trigger",
  ".nav-row:hover::after, .nav-row.active::after",
  "@media (prefers-reduced-motion: reduce)",
]) {
  if (!css.includes(requiredMotionFoundation)) {
    throw new Error(`Editorial Ink Mechanics motion foundation is missing: ${requiredMotionFoundation}`);
  }
}

for (const requiredMotionStateHook of [
  'data-state={isOpen ? "open" : "closed"}',
  'app-startup-notice app-startup-notice--loading',
]) {
  if (!app.includes(requiredMotionStateHook) && !globalDropdown.includes(requiredMotionStateHook)) {
    throw new Error(`Motion state hook is missing: ${requiredMotionStateHook}`);
  }
}

for (const forbiddenGlobalSumiELeak of ["settings-control-room", "blue-rail", "#3558a2", "rgba(53, 88, 162", "rgba(53,88,162", "#e7edfa", "#fde863"]) {
  if (app.includes(forbiddenGlobalSumiELeak) || css.includes(forbiddenGlobalSumiELeak)) {
    throw new Error(`Global sumi-e chrome must not retain old Kujoyama/control-room styling: ${forbiddenGlobalSumiELeak}`);
  }
}

for (const requiredSidebarControl of [
  "isSidebarCollapsed",
  "Minimize sidebar",
  "Maximize sidebar",
  "sidebar-collapsed",
  "Compact section navigation",
  "rail-nav-item",
  "tabIndex={isSidebarCollapsed ? -1 : undefined}",
  "handleSidebarMorphToggle",
  "SIDEBAR_MORPH_TIMING",
  "data-sidebar-morph-item",
  "data-sidebar-morph-panel",
  "flushSync",
]) {
  if (!app.includes(requiredSidebarControl)) {
    throw new Error(`Sidebar collapse control is missing: ${requiredSidebarControl}`);
  }
}

if (!app.includes("data-sidebar-morph-item={isSidebarCollapsed ? item.label : undefined}") || !app.includes("data-sidebar-morph-item={!isSidebarCollapsed ? item.label : undefined}")) {
  throw new Error("Sidebar morph items must be scoped to the visible source/destination set so duplicate keys do not target hidden rows");
}

for (const requiredInkSidebarControl of [
  'className={isSidebarCollapsed ? "rail-menu rail-menu--open" : "rail-menu rail-menu--close"}',
  ".rail-menu--close span:nth-child(1) { transform: translateY(7px) rotate(45deg); }",
  ".rail-menu--close span:nth-child(2) { opacity: 0; }",
  ".rail-menu--open span:nth-child(2) { opacity: 1; }",
  ".sidebar-collapsed .ink-rail::before { opacity: 0; }",
]) {
  if (!app.includes(requiredInkSidebarControl) && !css.includes(requiredInkSidebarControl)) {
    throw new Error(`Expanded sidebar must show an X close affordance and collapse back to hamburger: ${requiredInkSidebarControl}`);
  }
}

if (css.includes(".brand-block::after")) {
  throw new Error("Sidebar brand block must not render the black brush divider under ZOID25");
}

for (const requiredIcon of ["InkSigil", "variant=\"brain\"", "variant=\"today\"", "variant=\"projects\"", "variant=\"agents\"", "variant=\"code\"", "variant=\"content\"", "variant=\"automations\"", "variant=\"settings\""]) {
  if (!app.includes(requiredIcon)) {
    throw new Error(`Collapsed sidebar needs custom sumi-e nav sigil: ${requiredIcon}`);
  }
}

for (const requiredSigilCss of [".nav-sigil path", ".nav-sigil-seal { fill: var(--shell-seal); stroke: none; }"]) {
  if (!css.includes(requiredSigilCss)) {
    throw new Error(`Primary sidebar icons must use custom ink/seal styling: ${requiredSigilCss}`);
  }
}

if (/import \{[^}]*\b(?:Bot|CalendarDays|FolderKanban|Code2|Megaphone|Repeat2|Settings)\b/.test(app)) {
  throw new Error("Primary sidebar icons must not use the rejected generic lucide navigation icon set");
}

for (const requiredModelQuickSwitch of [
  "chat-stats-model-section",
  "chat-stats-model-button",
  "Reasoning {activeReasoningLabel}",
  "aria-haspopup=\"dialog\"",
  "onClick={() => setActiveCommandPanel(\"model\")}",
  "grid-template-columns: minmax(0, 1fr) auto",
]) {
  if (!screen.includes(requiredModelQuickSwitch) && !css.includes(requiredModelQuickSwitch)) {
    throw new Error(`Hermes footer needs a compact model/reasoning quick-switch button: ${requiredModelQuickSwitch}`);
  }
}

for (const requiredModelPanelRedesign of [
  "zoid-native-command-panel--model",
  "model-command-current",
  "Choose runtime defaults",
  "model-command-controls",
  "model-command-contract",
  "Saved to {modelPanelStorageLabel}",
  "className=\"command-panel-header\"",
  ".zoid-native-command-panel--model { width: min(780px, calc(100vw - 44px)); max-width: 780px; padding: 0; overflow: visible;",
  ".model-command-panel { display: grid; grid-template-columns: 236px minmax(0, 1fr);",
  ".model-command-current { display: grid; grid-auto-rows: min-content; grid-row: 1 / span 3; grid-template-columns: minmax(0, 1fr);",
  ".model-command-field { display: grid; grid-template-columns: 104px minmax(0, 1fr);",
]) {
  if (!screen.includes(requiredModelPanelRedesign) && !css.includes(requiredModelPanelRedesign)) {
    throw new Error(`Model controls command panel must be structured and redesigned, not a messy generic panel: ${requiredModelPanelRedesign}`);
  }
}

for (const requiredCollapseCss of [".zoid25-shell.sidebar-collapsed", ".sidebar-collapsed .editorial-sidebar", ".sidebar-collapsed .rail-nav", ".rail-nav-item.active", "sidebar-morphing", "cubic-bezier(0.16, 1, 0.3, 1)"]) {
  if (!css.includes(requiredCollapseCss)) {
    throw new Error(`Sidebar collapse styling is missing: ${requiredCollapseCss}`);
  }
}

if (!app.includes("LAST_WORKSPACE_STORAGE_KEY") || !app.includes('"zoid25:last-active-workspace"')) {
  throw new Error("App must remember the last active workspace in localStorage");
}

if (!app.includes("useState<ActiveWorkspace>(getInitialWorkspace)")) {
  throw new Error("Active page must initialize from the last stored workspace before falling back to Code");
}

if (!app.includes("window.localStorage.setItem(LAST_WORKSPACE_STORAGE_KEY, activeWorkspace)")) {
  throw new Error("Active workspace changes must be persisted for next launch");
}

for (const requiredRepositoryPersistence of [
  "REPOSITORIES_STORAGE_KEY",
  "getInitialRepositories",
  "window.localStorage.setItem(REPOSITORIES_STORAGE_KEY, JSON.stringify(repositories))",
]) {
  if (!app.includes(requiredRepositoryPersistence)) {
    throw new Error(`Scanned repositories must persist across sessions: ${requiredRepositoryPersistence}`);
  }
}

for (const forbiddenGlobalAgentsRepositoryLink of ["LINKED_REPOSITORY_STORAGE_KEY", "getInitialLinkedRepositoryId", "setLinkedRepositoryId", "onLinkedRepositoryIdChange="]) {
  if (app.includes(forbiddenGlobalAgentsRepositoryLink) || codeWorkspace.includes(forbiddenGlobalAgentsRepositoryLink)) {
    throw new Error(`Global repository-to-Agents linkage must be removed: ${forbiddenGlobalAgentsRepositoryLink}`);
  }
}

if (!app.includes("AgentsHermesScreen")) {
  throw new Error("App must still render the Hermes Agents screen");
}

for (const requiredRuthlessReviewerSurface of [
  "buildRuthlessReviewerPrompt",
  "RUTHLESS_REVIEWER_TOOLSETS = [\"terminal\", \"file\"]",
  "Do not grant browser, web, memory, cronjob, messaging, design, social, or further delegation tools.",
  "The reviewer must not edit files, commit, push, deploy, send messages, create cron jobs, or perform external side effects.",
  "Run ruthless review",
  "handleStartRuthlessCodeReview",
  "ruthless-reviewer-card",
]) {
  if (!ruthlessReviewerAgent.includes(requiredRuthlessReviewerSurface) && !screen.includes(requiredRuthlessReviewerSurface) && !css.includes(requiredRuthlessReviewerSurface)) {
    throw new Error(`Ruthless reviewer subagent wiring is missing or over-broad: ${requiredRuthlessReviewerSurface}`);
  }
}

if (!screen.includes("activeCommandPanel === \"agents\"") || !screen.includes("buildRuthlessReviewerPrompt({")) {
  throw new Error("Ruthless reviewer launcher must live inside the Agents command panel and build the guarded subagent prompt");
}

if (/RUTHLESS_REVIEWER_TOOLSETS\s*=\s*\[[^\]]*(browser|web|memory|cronjob|delegation|discord|spotify|vision)/.test(ruthlessReviewerAgent)) {
  throw new Error("Ruthless reviewer subagent must not receive unrelated toolsets");
}

if (!main.includes('import { Agentation } from "agentation"') || !main.includes("<Agentation />")) {
  throw new Error("Zoid must keep Agentation mounted globally");
}

for (const requiredAgentationDetailFix of [
  "agentationDetailFixedEntry",
  "src/vendor/agentation-fixed.mjs",
  "agentation: agentationDetailFixedEntry",
]) {
  if (!viteConfig.includes(requiredAgentationDetailFix)) {
    throw new Error(`Vite must alias Agentation to the local detail-level fix: ${requiredAgentationDetailFix}`);
  }
}

for (const requiredAgentationOutputDetail of [
  "function normalizeOutputDetail",
  "function getOutputDetailLabel",
  "**Output Detail:** ${getOutputDetailLabel(detailLevel)}",
  "const savedOutputDetail = OUTPUT_DETAIL_OPTIONS.find",
  "outputDetail: savedOutputDetail",
]) {
  if (!agentationFixed.includes(requiredAgentationOutputDetail)) {
    throw new Error(`Agentation output detail setting must visibly affect copied/sent output and sanitize persisted values: ${requiredAgentationOutputDetail}`);
  }
}

if (!app.includes("CodeWorkspace") || app.includes("empty-code-workspace")) {
  throw new Error("Code workspace must render the GitHub repositories integration, not the old empty page");
}

for (const requiredCodeSurface of ["Scan folder", "Clone repo", "Repository list", "Search repositories", "repository-search-input", "filteredRepositories", "repositoryScanFeedback", "repo-action-feedback", "repo-scan-feedback", "repository-card--just-added", "Run localhost", "Deploy staging", "Deploy production", "repository-operation-strip"]) {
  if (!codeWorkspace.includes(requiredCodeSurface) && !repositoryOperations.includes(requiredCodeSurface)) {
    throw new Error(`Code workspace is missing repository management surface: ${requiredCodeSurface}`);
  }
}

for (const requiredCodeDesignSystem of [
  "code-workspace-shell code-sumi-e",
  "code-hero-copy",
  "code-reference-line",
  "code-ink-mark",
  "Native Finder pickers · GitHub branch control · Hermes waits for your repository link",
  "--code-seal: #c23a2e;",
  "scrollbar-color: var(--code-seal)",
  ".code-workspace-header::before",
  ".code-ink-mark::after",
  ".repo-action-panel::before, .repository-list-panel::before",
  "font-family: var(--code-serif-latin)",
]) {
  if (!codeWorkspace.includes(requiredCodeDesignSystem) && !css.includes(requiredCodeDesignSystem)) {
    throw new Error(`Code workspace must use the Brain sumi-e design system detail: ${requiredCodeDesignSystem}`);
  }
}

for (const removedRepositoryStatusSurface of [
  "repo-" + "status-panel",
  "Linked to Agents:",
  "Ready to scan local GitHub repositories.",
]) {
  if (codeWorkspace.includes(removedRepositoryStatusSurface) || css.includes(removedRepositoryStatusSurface)) {
    throw new Error(`Code workspace must remove the useless repository status panel: ${removedRepositoryStatusSurface}`);
  }
}

if (!repositoryClient.includes("scan_github_repositories") || !repositoryClient.includes("clone_github_repository") || !repositoryClient.includes("list_github_branches") || !repositoryClient.includes("update_github_default_branch")) {
  throw new Error("Repository client must invoke scan, clone, branch-list, and default-branch update Tauri commands");
}

for (const requiredDefaultBranchSelection of ["listGithubBranches", "default-branch-editor", "GlobalDropdown", "handleSaveDefaultBranch", "Select a default branch", "default-branch-feedback", "defaultBranchError", "Default branch update failed", "Default branch updated to"]) {
  if (!codeWorkspace.includes(requiredDefaultBranchSelection) && !css.includes(requiredDefaultBranchSelection)) {
    throw new Error(`Default branch Edit must open a visible GitHub-backed branch selector instead of a no-op prompt: ${requiredDefaultBranchSelection}`);
  }
}

for (const requiredDefaultBranchEditVisibility of [
  "repo-meta-grid-item--editing",
  "repo-meta-action-row--editing",
  "aria-label=\"Edit default branch\"",
  "fallbackBranchOptions",
  "setEditingDefaultBranchRepositoryId(repository.id);",
  "default-branch-dropdown",
  "default-branch-save-button",
  "default-branch-cancel-button",
  ".repo-meta-grid-item--default-branch.repo-meta-grid-item--editing",
  ".repo-meta-grid dd.repo-meta-action-row--editing { display: block; overflow: visible; white-space: normal; text-overflow: clip; }",
  "grid-template-columns: minmax(0, 1fr) auto auto",
  ".default-branch-dropdown { min-width: 0; width: 100%; max-width: 100%; }",
  ".default-branch-dropdown .zoid-dropdown-menu { z-index: 120; right: auto; width: 100%;",
  ".default-branch-dropdown .zoid-dropdown-option { width: 100%; min-height: 32px; background: var(--code-paper); color: var(--code-ink-black); }",
  ".repository-card button:not(.zoid-dropdown-trigger):not(.zoid-dropdown-option)",
]) {
  if (!codeWorkspace.includes(requiredDefaultBranchEditVisibility) && !css.includes(requiredDefaultBranchEditVisibility)) {
    throw new Error(`Default branch Edit mode must be visibly expanded instead of clipped: ${requiredDefaultBranchEditVisibility}`);
  }
}

if (codeWorkspace.includes("window.prompt")) {
  throw new Error("Default branch editing must not rely on window.prompt because it is invisible/no-op in the desktop flow");
}

for (const requiredRepositoryListPriority of [
  "code-repository-layout",
  "repo-action-panel--scan",
  "repo-action-panel--clone",
  "grid-template-columns: minmax(0, 1fr) minmax(260px, 320px)",
  "grid-template-rows: auto minmax(0, 1fr)",
  "height: min(680px, calc(100vh - 230px))",
  "overflow-y: auto",
  ".repository-card-list::-webkit-scrollbar",
]) {
  if (!codeWorkspace.includes(requiredRepositoryListPriority) && !css.includes(requiredRepositoryListPriority)) {
    throw new Error(`Code workspace must prioritize a large scrollable repository list over scan/clone controls: ${requiredRepositoryListPriority}`);
  }
}

if (!repositoryClient.includes("@tauri-apps/plugin-dialog") || !repositoryClient.includes("directory: true") || !codeWorkspace.includes("handleChooseScanFolder") || !codeWorkspace.includes("handleChooseCloneDestination")) {
  throw new Error("Repository folders must be selected through the native Finder folder picker, not typed manually");
}

for (const forbiddenManualFolderInput of ["onChange={(event) => setScanFolder(event.target.value)}", "onChange={(event) => setDestinationRoot(event.target.value)}", "placeholder=\"/Users/ziadnasreldin/Documents/GitHub\""]) {
  if (codeWorkspace.includes(forbiddenManualFolderInput)) {
    throw new Error(`Repository folder selection must not use manual path typing: ${forbiddenManualFolderInput}`);
  }
}

if (!codeWorkspace.includes("readOnly") || !codeWorkspace.includes("Choose folder…") || !codeWorkspace.includes("Choose destination…")) {
  throw new Error("Selected folders should display as read-only values with Finder picker buttons");
}

if (!existsSync(new URL("./agents/participants.ts", import.meta.url))) {
  throw new Error("Hermes and user participants must be defined");
}

if (!existsSync(new URL("./agents/AgentsHermesScreen.tsx", import.meta.url))) {
  throw new Error("Agents Hermes screen must exist");
}

for (const requiredSessionPersistence of [
  "HERMES_SESSIONS_STORAGE_KEY",
  "HERMES_ARCHIVED_SESSIONS_STORAGE_KEY",
  "getInitialHermesSessions",
  "getInitialArchivedHermesSessions",
]) {
  if (!app.includes(requiredSessionPersistence) && !screen.includes(requiredSessionPersistence)) {
    throw new Error(`Hermes sessions need automatic save and archive support: ${requiredSessionPersistence}`);
  }
}

for (const removedManualSessionPersistence of ["handleSaveHermesSessions", "onSaveSessions", "Save sessions", "save-sessions-button", "Unsaved session changes"]) {
  if (app.includes(removedManualSessionPersistence) || screen.includes(removedManualSessionPersistence) || css.includes(removedManualSessionPersistence)) {
    throw new Error(`Hermes sessions should auto save without manual save UI: ${removedManualSessionPersistence}`);
  }
}

for (const requiredSessionsRailMorph of [
  "handleSessionsRailMorphToggle",
  "SESSIONS_RAIL_MORPH_TIMING",
  "flushSync",
  "data-session-rail-morph-item",
  "data-session-rail-morph-panel",
  "sessions-rail-morphing",
  "cubic-bezier(0.16, 1, 0.3, 1)",
]) {
  if (!screen.includes(requiredSessionsRailMorph) && !css.includes(requiredSessionsRailMorph)) {
    throw new Error(`Hermes sessions rail must use the same minimize/maximize morph system as the sidebar: ${requiredSessionsRailMorph}`);
  }
}

for (const requiredSessionPortraitTreatment of [
  "SESSION_AGENT_AVATARS",
  "chooseUniqueSessionAgentAvatarId",
  "portraitId",
  "hashSessionAvatarIndex",
  "getSessionAgentAvatar",
  "sessionPortraitStyle",
  "--session-portrait",
  "--session-portrait-accent",
  "--session-portrait-focal-point",
  "session-tab-portrait",
  "filter: saturate(1.12) contrast(1.18)",
  "filter: blur(2.8px) saturate(1.08) contrast(1.14)",
]) {
  if (!app.includes(requiredSessionPortraitTreatment) && !sessionPortraits.includes(requiredSessionPortraitTreatment) && !screen.includes(requiredSessionPortraitTreatment) && !css.includes(requiredSessionPortraitTreatment)) {
    throw new Error(`Hermes session tabs need unique agent avatar treatment: ${requiredSessionPortraitTreatment}`);
  }
}

if (/\.session-tab-portrait::before\s*\{[^}]*filter:\s*blur\(/s.test(css)) {
  throw new Error("Expanded Hermes session tab portrait icons must not use any blur effect.");
}

const portraitAssetCount = (sessionPortraits.match(/\/agent-avatars\//g) ?? []).length;
if (portraitAssetCount !== 14) {
  throw new Error(`Hermes session avatar pool must contain exactly 14 user-provided agent avatar assets; found ${portraitAssetCount}`);
}

const sessionFigureAssetFiles = readdirSync(new URL("../public/agent-avatars", import.meta.url)).filter((file) => file.endsWith(".jpg"));
if (sessionFigureAssetFiles.length !== 14) {
  throw new Error(`Hermes agent session avatar asset directory must contain exactly 14 JPG files; found ${sessionFigureAssetFiles.length}`);
}
const declaredAgentAvatarAssets = new Set(Array.from(sessionPortraits.matchAll(/asset: "\/agent-avatars\/([^\"]+\.jpg)"/g)).map((match) => match[1]));
if (declaredAgentAvatarAssets.size !== 14 || sessionFigureAssetFiles.some((file) => !declaredAgentAvatarAssets.has(file))) {
  throw new Error("Hermes session avatar pool must use every user-provided Agent Avatars JPG exactly once");
}

const assignedAgentAvatarIds: string[] = [];
for (let index = 0; index < SESSION_AGENT_AVATARS.length; index += 1) {
  assignedAgentAvatarIds.push(chooseUniqueSessionAgentAvatarId(assignedAgentAvatarIds, `new-session-${index}`));
}
if (new Set(assignedAgentAvatarIds).size !== SESSION_AGENT_AVATARS.length) {
  throw new Error("New Hermes sessions must receive unique agent avatar images before the pool is reused");
}

if (!screen.includes("function prependNewSession()") || !screen.includes("createSession(\"New session\", current)") || !screen.includes("pendingNewSessionActivationRef")) {
  throw new Error("New Hermes sessions must be created inside the functional session updater so rapid additions avoid reused agent avatars");
}

if (!screen.includes("prependNewSession();") || screen.includes("createSession(\"New session\", sessions)")) {
  throw new Error("Slash-command-created Hermes sessions must use the same current-state unique avatar assignment path as the rail button");
}

if (!app.includes("hasValidUnusedPortrait") || !app.includes("!usedPortraitIds.includes(session.portraitId)")) {
  throw new Error("Hermes session hydration must repair duplicate-but-valid legacy agent avatar ids before pool exhaustion");
}

if (!app.includes("portraitId: archivedSession.portraitId")) {
  throw new Error("Restored archived Hermes sessions must preserve their assigned agent avatar id");
}

for (const requiredHermesFileManager of [
  "Open file manager sidebar",
  "fileManagerOpen",
  "listFileManagerDirectory",
  "file-manager-sidebar",
  "macOS Finder file manager",
  "renderFileManagerEntries",
  "handleFolderToggle",
  "chat-workspace--file-manager-open",
]) {
  if (!screen.includes(requiredHermesFileManager) && !css.includes(requiredHermesFileManager)) {
    throw new Error(`Hermes topbar file manager sidebar is missing: ${requiredHermesFileManager}`);
  }
}

for (const requiredHermesFileManagerResize of [
  "FILE_MANAGER_MIN_WIDTH",
  "FILE_MANAGER_MAX_WIDTH",
  "FILE_MANAGER_WIDTH_STORAGE_KEY",
  "clampFileManagerWidth",
  "getInitialFileManagerWidth",
  "handleFileManagerResizeStart",
  "handleFileManagerResizeKeyDown",
  "aria-valuenow={fileManagerWidth}",
  "--file-manager-width",
  "file-manager-resize-handle",
  "Drag to resize Finder sidebar",
  "grid-template-columns 420ms cubic-bezier(0.16, 1, 0.3, 1)",
  "file-manager-panel-enter",
]) {
  if (!screen.includes(requiredHermesFileManagerResize) && !css.includes(requiredHermesFileManagerResize)) {
    throw new Error(`Hermes Finder sidebar must resize and open smoothly: ${requiredHermesFileManagerResize}`);
  }
}

if (screen.includes(">Up</button>")) {
  throw new Error("Hermes Finder sidebar must not render the useless Up toolbar button");
}

for (const requiredNativeFileManager of [
  "list_file_manager_directory",
  "FileManagerDirectoryListing",
  "FileManagerEntry",
  "list_file_manager_directory_inner",
  "commands::list_file_manager_directory",
]) {
  if (!client.includes(requiredNativeFileManager) && !backend.includes(requiredNativeFileManager)) {
    throw new Error(`Hermes file manager must use a native lazy directory listing command: ${requiredNativeFileManager}`);
  }
}

for (const removedBrandChromeSurface of ["window-controls", "macOS AI operating scaffold", "brand-subtitle"]) {
  if (app.includes(removedBrandChromeSurface)) {
    throw new Error(`Sidebar must remove useless brand/window chrome: ${removedBrandChromeSurface}`);
  }
}

for (const requiredSettingsArchiveSurface of ["SettingsArchive", "Archived agent sessions", "Restore session", "No archived sessions yet", "Delete selected", "Delete all archived", "handleDeleteArchivedHermesSessions", "handleDeleteAllArchivedHermesSessions", "setArchivedHermesSessions((current) => current.filter"]) {
  if (!app.includes(requiredSettingsArchiveSurface)) {
    throw new Error(`Settings must expose archived agent sessions: ${requiredSettingsArchiveSurface}`);
  }
}

const archivedSessionsWriterCount = (app.match(/window\.localStorage\.setItem\(HERMES_ARCHIVED_SESSIONS_STORAGE_KEY/g) ?? []).length;
if (archivedSessionsWriterCount !== 1) {
  throw new Error(`Archived Hermes sessions must have exactly one centralized localStorage writer; found ${archivedSessionsWriterCount}`);
}

for (const requiredCompleteProfileSurface of [
  "Identity & preferences",
  "Hermes memory & soul",
  "Models & reasoning",
  "Tools, MCP, plugins & projects",
  "Safety, privacy, voice & notifications",
  "memoryEnabled",
  "userProfileEnabled",
  "Memory lens limits",
  "profile-memory-budget-card",
  "renderNumberField(\"memoryCharLimit\"",
  "renderNumberField(\"userCharLimit\"",
  "accessMode",
  "approvalMode",
  "gatewayPlatforms",
]) {
  if (!app.includes(requiredCompleteProfileSurface) && !profileClient.includes(requiredCompleteProfileSurface)) {
    throw new Error(`Complete profile page is missing Codex/Hermes preference surface: ${requiredCompleteProfileSurface}`);
  }
}

for (const requiredProfilePersistence of [
  "role: string",
  "modelProvider: string",
  "toolsets: string",
  "mcpServers: string",
  "secretRedactionEnabled: boolean",
  "asString(value: unknown",
  "asBoolean(value: unknown",
  "const parsed: unknown = JSON.parse(stored)",
  "typeof parsed !== \"object\" || parsed === null",
  "save_hermes_profile_settings",
  "warm_file_permissions",
  "memory_char_limit",
  "user_char_limit",
  "Memory and user profile character limits must be greater than zero.",
  "--yolo",
  "#[serde(default, rename_all = \"camelCase\")]",
  "isRunningInTauri()",
  "throw error",
  "Save blocked until Hermes profile loads successfully",
]) {
  if (!profileClient.includes(requiredProfilePersistence) && !backend.includes(requiredProfilePersistence) && !app.includes(requiredProfilePersistence)) {
    throw new Error(`Complete profile settings must persist across browser/native restarts: ${requiredProfilePersistence}`);
  }
}

for (const requiredProviderManagement of [
  "ProvidersSettings",
  "Provider management settings",
  "PROVIDER_TEMPLATES",
  "Google Gemini",
  "gemma-3-27b-it",
  "GlobalDropdown label=\"Default model\"",
  "availableModels: Record<string, string[]>",
  "mergeLiveModels",
  "newProviderDraft(availableModels)",
  "type=\"password\"",
  "Reveal key",
  "Apply / sync",
  "list_managed_providers",
  "save_managed_provider",
  "validate_managed_provider",
  "apply_managed_provider",
  "reveal_managed_provider_key",
  "macOS Keychain",
  "zoid-providers.json",
  "GOOGLE_API_KEY",
  "GEMINI_API_KEY",
  "https://generativelanguage.googleapis.com/v1beta/models",
]) {
  if (!app.includes(requiredProviderManagement) && !providersSettings.includes(requiredProviderManagement) && !providerClient.includes(requiredProviderManagement) && !backend.includes(requiredProviderManagement)) {
    throw new Error(`Providers settings must manage API-key backed providers with model dropdowns and manual Hermes sync: ${requiredProviderManagement}`);
  }
}

if (providersSettings.includes("<select") || providerClient.includes("<select")) {
  throw new Error("Provider management must use GlobalDropdown instead of native select controls");
}

for (const staleModelName of ["openai/gpt-5.1", "gpt-5.1-mini", "gpt-4.1", "claude-sonnet-4-5", "anthropic/claude-sonnet-4\""]) {
  if (providerClient.includes(staleModelName) || profileClient.includes(staleModelName) || backend.includes(staleModelName)) {
    throw new Error(`Provider/model catalogs must not seed stale pre-current model ids: ${staleModelName}`);
  }
}

for (const requiredLiveCatalogBehavior of [
  "{home}/.hermes/hermes-agent/venv/bin/hermes",
  "model.split_once('/')",
  "push_model(models, model_provider, short_model)",
  "discover_hermes_skills()",
  "command.args([\"skills\", \"list\", \"--source\", \"all\"])",
]) {
  if (!backend.includes(requiredLiveCatalogBehavior)) {
    throw new Error(`Hermes profile settings must hydrate real current skills/models from Hermes, not four static fallbacks: ${requiredLiveCatalogBehavior}`);
  }
}

if (!app.includes("profile-catalog-grid--control-plane") || !app.includes("renderCatalogGroup(\"Skills\"") || !app.includes("renderCatalogGroup(\"Plugins\"") || !app.includes("renderCatalogGroup(\"MCP servers\"") || !app.includes("renderCatalogGroup(\"Toolsets\"")) {
  throw new Error("Tools tab must render skills, plugins, MCP servers, and toolsets as toggleable catalog lists/grids, not editable textareas");
}

if (!css.includes("grid-template-rows: 17px 50px minmax(34px, auto)") || !css.includes(".provider-form-grid .profile-field span { min-height: 17px")) {
  throw new Error("Provider editor fields must use fixed label/control/helper rows so boxes align across the whole card");
}

if (!app.includes("managedProviders") || !app.includes("availableModelOptions") || !app.includes("onSelectMainProvider") || !app.includes("listManagedProviders().then((loaded)")) {
  throw new Error("Saved Zoid-managed providers must load at Settings mount and become selectable immediately in Main provider/Main model");
}

if (!app.includes("warmFilePermissions(false)")) {
  throw new Error("Zoid must warm macOS file permissions once during app startup before prompt sends");
}

for (const requiredPermanentPermissionBehavior of [
  "remoteUrl?: string | null",
  "currentDefaultBranch?: string | null",
  "remote_url: Option<String>",
  "current_default_branch: Option<String>",
  "listGithubBranches(repository.path, repository.remoteUrl, currentDefaultBranch)",
  "updateGithubDefaultBranch(repository.path, repository.remoteUrl, trimmedDefaultBranch)",
]) {
  if (!repositoryClient.includes(requiredPermanentPermissionBehavior) && !backend.includes(requiredPermanentPermissionBehavior) && !codeWorkspace.includes(requiredPermanentPermissionBehavior)) {
    throw new Error(`GitHub branch edit must use persisted remote metadata instead of re-reading local folders and re-triggering macOS file prompts: ${requiredPermanentPermissionBehavior}`);
  }
}

if (!css.includes(".profile-section") || !css.includes(".profile-toggle-grid") || !css.includes(".profile-hero-card")) {
  throw new Error("Complete profile page needs dedicated section, toggle, and active-profile styling");
}

for (const requiredOrganizedSettingsSurface of [
  "activeSettingsSection",
  "profile-settings-workspace",
  "profile-settings-nav",
  "role=\"tablist\"",
  "role=\"tabpanel\"",
  "aria-labelledby=\"profile-tab-identity\"",
  "aria-labelledby=\"profile-tab-providers\"",
  "profile-page-shell--organized",
  "profile-settings-content",
]) {
  if (!app.includes(requiredOrganizedSettingsSurface) && !css.includes(requiredOrganizedSettingsSurface)) {
    throw new Error(`Settings page must use compact tabbed layout instead of one long scroll page: ${requiredOrganizedSettingsSurface}`);
  }
}

if (app.includes("window.confirm(`Delete") || app.includes("confirmArchiveDelete(")) {
  throw new Error("Settings archive destructive actions must use a branded in-app confirmation modal, not window.confirm");
}

for (const requiredSettingsSumiE of [
  "settings-sumi-e",
  "settings-hero-copy",
  "settings-reference-line",
  "settings-ink-mark",
  "settings-confirm-panel",
  "--settings-seal: var(--brain-seal",
  "settings-ink-reveal",
  "settings-mark-reveal",
  "scrollbar-color: var(--settings-seal)",
  "profile-tab-panel-enter",
  "transition: opacity 220ms ease",
  "prefers-reduced-motion: reduce",
]) {
  if (!app.includes(requiredSettingsSumiE) && !css.includes(requiredSettingsSumiE)) {
    throw new Error(`Settings page must inherit the Brain sumi-e design system: ${requiredSettingsSumiE}`);
  }
}

for (const requiredSettingsMotionCaveatCoverage of [
  "settings-live-row-reveal",
  "settings-confirm-rule-draw",
  ".settings-sumi-e .profile-catalog-item input",
  ".settings-sumi-e .profile-toggle input",
  ".settings-sumi-e .provider-meta-grid div",
  ".settings-sumi-e .provider-status-badge",
  ".settings-sumi-e .archive-session-select input",
  ".settings-sumi-e .repo-empty-state",
  ".settings-sumi-e .provider-card:hover .provider-status-badge",
  ".settings-sumi-e .archived-session-card:hover .archive-session-select",
  ".settings-sumi-e .provider-card--invalid",
  ".settings-sumi-e .provider-card--validated",
]) {
  if (!css.includes(requiredSettingsMotionCaveatCoverage)) {
    throw new Error(`Settings hidden/live-data motion caveat coverage is missing: ${requiredSettingsMotionCaveatCoverage}`);
  }
}

for (const requiredConditionalStateMotion of [
  ".automation-sumi-e .automation-modal-error",
  ".automation-sumi-e .automation-error-line",
  ".automation-sumi-e .repo-empty-state",
  ".social-empty",
]) {
  if (!css.includes(requiredConditionalStateMotion)) {
    throw new Error(`Animation pass must cover hidden empty/error/modal states: ${requiredConditionalStateMotion}`);
  }
}

const settingsReducedMotionBlock = css.slice(
  css.indexOf("@media (prefers-reduced-motion: reduce) {\n  .settings-sumi-e"),
  css.indexOf("/* MaVoid Buffer social dashboard */"),
);
for (const requiredSettingsReducedMotionCaveatCoverage of [
  ".settings-sumi-e .repo-empty-state",
  ".settings-sumi-e .settings-confirm-panel::before",
]) {
  if (!settingsReducedMotionBlock.includes(requiredSettingsReducedMotionCaveatCoverage)) {
    throw new Error(`Settings reduced-motion caveat coverage is missing: ${requiredSettingsReducedMotionCaveatCoverage}`);
  }
}

for (const requiredSettingsSafetyImplementation of [
  "handleSettingsTabsKeyDown",
  "ArrowLeft",
  "ArrowRight",
  "Home",
  "End",
  "tabIndex={activeSettingsSection === section.id ? 0 : -1}",
  "setSelectedArchivedSessionIds((current) => current.filter((sessionId) => archivedSessionIds.has(sessionId)))",
  "requestArchiveDelete(sessionIds: string[], label: string, deleteAll = false)",
  "currentSessionIds.length === 0",
  "pendingCoversEveryCurrentArchive",
  "currentArchivedSessionIds.every((sessionId) => pendingSessionIds.has(sessionId))",
  "aria-describedby=\"settings-confirm-description\"",
  "id=\"settings-confirm-description\"",
  "setAttribute(\"inert\", \"\")",
  "removeAttribute(\"inert\")",
]) {
  if (!app.includes(requiredSettingsSafetyImplementation)) {
    throw new Error(`Settings archive/tabs safety implementation is missing: ${requiredSettingsSafetyImplementation}`);
  }
}

for (const forbiddenSettingsSafetyRegression of [
  "pendingArchiveDelete.sessionIds.length === archivedSessions.length",
  ".settings-sumi-e .settings-archive-shell .zoid-dropdown-trigger",
  ".settings-sumi-e .settings-archive-shell .zoid-dropdown-option",
  "Settings sumi-e compact control-room correction",
  "Settings sumi-e tighter above-the-fold correction",
]) {
  if (app.includes(forbiddenSettingsSafetyRegression) || css.includes(forbiddenSettingsSafetyRegression)) {
    throw new Error(`Settings redesign has a known safety/cascade regression: ${forbiddenSettingsSafetyRegression}`);
  }
}

if (!css.includes("button:not(:disabled)::after") || !css.includes("button:disabled")) {
  throw new Error("Settings sumi-e button styling must avoid active seal decorations on disabled buttons");
}

const settingsHeroBaseIndex = css.indexOf(".settings-sumi-e .settings-hero { position: relative");
const settingsHeroMediaIndex = css.indexOf("@media (max-width: 1180px) { .settings-sumi-e .settings-hero");
if (settingsHeroBaseIndex < 0 || settingsHeroMediaIndex < 0 || settingsHeroMediaIndex < settingsHeroBaseIndex) {
  throw new Error("Settings responsive hero media queries must come after the final base hero rule");
}

if (profileClient.includes("...value,")) {
  throw new Error("Profile settings sanitizer must not broadly spread malformed fallback values over safe defaults");
}

if (!app.includes("type ActiveWorkspace") || !app.includes("\"Agents\"") || !app.includes("\"Code\"") || !app.includes("\"Settings\"")) {
  throw new Error("Settings archive requires Agents, Code, and Settings active workspaces to remain available");
}

if (!css.includes("hermes-chat-shell")) {
  throw new Error("Hermes chat shell styling is missing");
}

for (const requiredHermesGenMDesignSystem of [
  "hermes-chat-shell hermes-genm",
  "hermes-reference-line",
  "hermes-genm-ink-mark",
  "--genm-ink-black: #0d0a0a",
  "--genm-seal-red: #c23a2e",
  "--genm-serif-latin",
  ".hermes-genm .hermes-topbar",
  ".hermes-genm .chat-stage",
  "hermes-genm-reveal",
]) {
  if (!screen.includes(requiredHermesGenMDesignSystem) && !css.includes(requiredHermesGenMDesignSystem)) {
    throw new Error(`Hermes Agents page must implement the Gen-M monochrome ink design system: ${requiredHermesGenMDesignSystem}`);
  }
}

for (const requiredComposerPolish of [
  "handleMessageChange",
  "composerHeightRef",
  "playCalmingTypingSound",
  "typingAudioContextRef",
  "TYPING_SOUND_MIN_INTERVAL_MS",
  "TYPING_SOUND_VOLUME",
  "TYPING_SOUND_CLICK_VOLUME",
  "createOscillator",
  "createBiquadFilter",
  "primaryOscillator.type = \"sine\"",
  "clickOscillator.type = \"sine\"",
  "lowpass",
  "bambooWaterDropPitch",
  "templeBellOvertonePitch",
  "tatamiRoomDamping",
  "inputType === \"insertFromPaste\"",
  "isHermesCliCommandDraft",
  "composer-input-wrap--hermes-command",
  "composer-mode-strip--hermes-command",
  "RUN CLI",
  "onStop",
  "STOP",
  "Stop Hermes run (Ctrl/Cmd+C)",
  "composer-send--stop",
  "COMPOSER_MIN_HEIGHT",
  "COMPOSER_MAX_HEIGHT",
  "Math.min(Math.max(textarea.scrollHeight, COMPOSER_MIN_HEIGHT), COMPOSER_MAX_HEIGHT)",
]) {
  if (!chatComposer.includes(requiredComposerPolish)) {
    throw new Error(`Composer textarea needs command mode and auto-height behavior: ${requiredComposerPolish}`);
  }
}

if (!client.includes("cancelHermesCliMessage") || !client.includes("cancel_hermes_cli_message") || !backend.includes("cancel_active_hermes_run_inner") || !backend.includes("run_hermes_command_with_cancel") || !backend.includes("kill") || !backend.includes("-INT")) {
  throw new Error("Hermes chat runs must expose a real native Ctrl+C/stop path that interrupts the active CLI process");
}

if (!screen.includes("handleStopHermesRun") || !screen.includes("event.key.toLowerCase() === \"c\"") || !screen.includes("hasActiveTextSelection") || !screen.includes("Stopped Hermes with Ctrl/Cmd+C")) {
  throw new Error("Hermes screen must let Ctrl/Cmd+C stop a running agent without stealing normal text copy selections");
}

for (const requiredComposerCss of [
  "min-height: var(--composer-control-size)",
  "height: var(--composer-control-size)",
  "padding: 8px 14px",
  "transition: height 220ms cubic-bezier(0.22, 1, 0.36, 1)",
  "resize: none",
  "line-height: 1.35",
  ".composer-input-wrap--hermes-command textarea",
  ".composer-send--hermes-command",
  ".composer-send--stop",
  ".composer-mode-chip",
  ".composer-mode-strip",
  ".message-command-chip",
]) {
  if (!css.includes(requiredComposerCss)) {
    throw new Error(`Composer textarea needs aligned default height and smooth multiline expansion styling: ${requiredComposerCss}`);
  }
}

for (const removedPerKeystrokeMotion of ["typingEffectTimerRef", "composer-input-wrap--typing", "@keyframes composerTypingRing", "animation: composerTypingRing", "border-color 160ms ease", "box-shadow 160ms ease", "background 160ms ease"]) {
  if (chatComposer.includes(removedPerKeystrokeMotion) || css.includes(removedPerKeystrokeMotion)) {
    throw new Error(`Composer must not animate or glow on every keystroke: ${removedPerKeystrokeMotion}`);
  }
}

for (const restoredHermesTitleSurface of ["hermes-title-block", "<h2>Hermes Agents</h2>"]) {
  if (!screen.includes(restoredHermesTitleSurface)) {
    throw new Error(`Hermes topbar must keep the visible title/header: ${restoredHermesTitleSurface}`);
  }
}

for (const removedTopbarSurface of ["AGENTS / HERMES TERMINAL", "topbar-session-actions", "session-save-status", "Auto saved", "Sessions auto save"]) {
  if (screen.includes(removedTopbarSurface) || css.includes(removedTopbarSurface)) {
    throw new Error(`Hermes topbar must remove useless header/status surface: ${removedTopbarSurface}`);
  }
}

if (!screen.includes("hermes-topbar hermes-topbar--status-only") || !css.includes(".topbar-status-stack { display: grid; grid-template-columns: max-content minmax(240px, 320px) auto;")) {
  throw new Error("Hermes status topbar must keep compact connection/repository controls beside the title");
}

for (const requiredHermesFeedbackPolish of [
  "display: grid;",
  "position: relative;",
  "z-index: 70;",
  "overflow: visible;",
  "grid-column: 1 / -1;",
  "grid-row: 2;",
  "grid-template-columns: max-content minmax(320px, 1fr) minmax(148px, max-content);",
  "width: max-content;",
  "max-width: 100%;",
  "gap: 14px;",
  ".agents-sumi-e .repository-link-control--topbar .zoid-dropdown-trigger { min-height: 44px; padding-block: 0; }",
  ".agents-sumi-e .repository-link-control--topbar .zoid-dropdown {",
  "z-index: 90;",
  ".agents-sumi-e .repository-link-control--topbar .zoid-dropdown-menu {",
  "z-index: 220;",
  "max-height: min(360px, calc(100vh - 255px));",
  "overscroll-behavior: contain;",
  ".agents-sumi-e .hermes-topbar:has(.zoid-dropdown-menu)::after",
  "opacity: 0;",
  ".agents-sumi-e .chat-workspace {",
  ".agents-sumi-e .connection-panel {",
  "connection-status-copy",
  "status-label-jp",
  "CONNECTION_STATE_JAPANESE",
  "Repository",
  "代理",
  "接続",
  "Unlinked / 未接続",
  "<span>Files</span>",
  "min-width: 148px;",
  ".agents-sumi-e .file-manager-toggle-button span { display: inline; }",
  "button-label-jp",
  "書類",
  "Context</b>",
  "Time</b>",
  "Model</b>",
  "Session</b>",
  ".agents-sumi-e .chat-stats-strip b",
  "font-family: var(--agents-serif-latin);",
  ".agents-sumi-e .file-manager-toggle-button::after",
  "background: linear-gradient(90deg, transparent, currentColor 18%, currentColor 74%, transparent);",
  ".agents-sumi-e .chat-stats-strip > span:last-child { background: transparent; color: var(--agents-ink-soft); }",
  ".agents-sumi-e .sessions-overflow-cue {\n  border-color: var(--agents-ink-black);\n  border-radius: 0;",
  "box-shadow: none;",
  ".agents-sumi-e .sessions-overflow-cue::before { display: none; }",
  ".agents-sumi-e .sessions-overflow-cue:focus-visible { filter: none; }",
  "text-transform: none;",
]) {
  if (!css.includes(requiredHermesFeedbackPolish) && !screen.includes(requiredHermesFeedbackPolish)) {
    throw new Error(`Hermes page feedback polish is missing sumi-e controls/stat strip: ${requiredHermesFeedbackPolish}`);
  }
}

for (const forbiddenHermesNeutralStatColor of [
  ".agents-sumi-e .chat-stats-strip { color: #56514b",
  ".agents-sumi-e .chat-stats-strip span { color: #56514b",
  "border-color: #e8e4dd",
]) {
  if (css.includes(forbiddenHermesNeutralStatColor)) {
    throw new Error(`Hermes stats strip must not keep the old neutral palette: ${forbiddenHermesNeutralStatColor}`);
  }
}

for (const pageFeedbackLayoutPolish of [
  ".ink-rail {\n  position: relative;\n  isolation: isolate;\n  display: flex;\n  flex-direction: column;\n  align-items: center;\n  height: 100vh;",
  ".rail-language {\n  position: absolute;",
  "bottom: 24px;",
  "transform: translate3d(-50%, 0, 0);",
  "transition: opacity 320ms ease, transform 540ms cubic-bezier(0.16, 1, 0.3, 1);",
  "padding: clamp(14px, 1.5vw, 24px) clamp(16px, 3vw, 48px) 20px;",
  "min-height: clamp(240px, 22vw, 340px);",
  "grid-template-columns: minmax(0, 0.72fr) minmax(100px, 0.14fr) auto;",
  "width: clamp(92px, 11vw, 156px);",
  "automation-primary-button automation-refresh-button",
  ".automation-sumi-e .automation-header-actions .automation-refresh-button::after { content: none; }",
]) {
  if (!css.includes(pageFeedbackLayoutPolish) && !screen.includes(pageFeedbackLayoutPolish) && !automationsWorkspace.includes(pageFeedbackLayoutPolish)) {
    throw new Error(`Page feedback layout polish is missing: ${pageFeedbackLayoutPolish}`);
  }
}

for (const stalePageFeedbackLayout of [
  "min-height: clamp(460px, 38vw, 580px);",
  "width: clamp(118px, 16vw, 218px);",
  "padding: clamp(20px, 2vw, 32px) clamp(16px, 3vw, 48px) 20px;",
]) {
  if (css.includes(stalePageFeedbackLayout)) {
    throw new Error(`Page feedback layout polish must remove stale oversized rule: ${stalePageFeedbackLayout}`);
  }
}

for (const restoredSessionRailSurface of [
  "sessions-rail",
  "Opened Hermes sessions",
  "session-tab-row",
  "session-new-button",
  "session-new-icon",
  "Start a clean Hermes terminal thread",
  "Drag to resize Sessions rail",
  "isSessionsRailCompact",
  "Maximize sessions rail",
  "Minimize sessions rail",
  "zoid25:hermes-sessions-rail",
  "beginRenameSession",
  "onDoubleClick={() => beginRenameSession(session)}",
  "onContextMenu={(event) => { event.preventDefault(); beginRenameSession(session); }}",
  "session-rename-input",
  "repositoryLabel(sessionRepository)",
  "activeSession?.linkedRepositoryId",
]) {
  if (!screen.includes(restoredSessionRailSurface) && !css.includes(restoredSessionRailSurface)) {
    throw new Error(`Hermes desktop sessions rail/list UI must be restored: ${restoredSessionRailSurface}`);
  }
}

if (
  !css.includes(".archive-session-button { display: inline-grid; place-items: center;") ||
  !/\.sessions-rail--compact \.session-tab \{[^}]*grid-template-columns: 1fr;[^}]*grid-template-rows: 1fr;[^}]*place-items: center;/.test(css) ||
  !/\.sessions-rail--compact \.session-tab-icon \{[^}]*grid-row: auto;[^}]*place-self: center;[^}]*width: 30px;[^}]*height: 30px;[^}]*border: 0;/.test(css)
) {
  throw new Error("Sessions rail must use compact archive controls and one-box compact icons centered in their boxes");
}

if (!/\.sessions-rail--compact \.session-tab-icon\.session-tab-portrait \{[^}]*position: absolute;[^}]*inset: 0;[^}]*width: 100%;[^}]*height: 100%;[^}]*place-self: stretch;[^}]*backdrop-filter: none;/s.test(css)) {
  throw new Error("Compact Hermes session avatar portraits must fill the whole session button instead of sitting inside a smaller inner box");
}

if (!css.includes(".session-rename-input")) {
  throw new Error("Sessions rail must style inline session rename input");
}

if (!/\.chat-workspace \{[^}]*display: grid;[^}]*grid-template-columns: var\(--sessions-rail-width, 184px\) minmax\(0, 1fr\);[^}]*overflow: hidden;/.test(css)) {
  throw new Error("Hermes chat workspace must reserve the restored resizable sessions rail column");
}

if (!/\.chat-workspace \{[^}]*grid-template-rows: minmax\(0, 1fr\) auto;/.test(css) || !/\.chat-composer \{[^}]*grid-column: 1 \/ -1;[^}]*grid-row: 2;/.test(css)) {
  throw new Error("Hermes composer must occupy the full bottom row across both the sessions rail and chat pane");
}

if (!/<\/div>[\s\S]{0,420}<ChatComposer\s+ref=\{composerRef\}\s+disabled=\{connectionState !== "online"\}/.test(screen)) {
  throw new Error("Hermes composer must be a chat-workspace sibling of the main pane so it can span the whole row");
}

if (!screen.includes("handleChatStagePointerDown") || !screen.includes("onPointerDown={handleChatStagePointerDown}") || !screen.includes("focusMessageField")) {
  throw new Error("Clicking the Hermes chat stage must focus the message composer for immediate typing");
}

if (!messageBubble.includes("copyTextToClipboard") || !messageBubble.includes("message-copy-button") || !messageBubble.includes("${participant.displayName} message") || !css.includes("user-select: text") || !css.includes(".message-bubble-frame:hover .message-copy-button") || !css.includes("opacity: 0; pointer-events: none") || !css.includes("opacity 160ms ease") || !screen.includes(".message-bubble, .message-copy-button")) {
  throw new Error("Hermes messages must keep selectable text and reveal a smooth dependable copy icon only near hovered/focused responses without stealing selection focus");
}

for (const requiredRollbackControl of [
  "handleRollbackToMessage",
  "const command = `/undo ${userTurnsToUndo}`;",
  "executeHermesSlashCommand(command, detectedRepository?.path, activeSession.hermesCliSessionId, true)",
  "messagesToKeep",
  "userTurnsAfterMessage",
  "canRollback={userTurnsAfterMessage > 0 && !isSending}",
  "onRollback={() => void handleRollbackToMessage(index)}",
  "message-rollback-button",
  "Roll back conversation to here",
]) {
  if (!screen.includes(requiredRollbackControl) && !messageBubble.includes(requiredRollbackControl) && !css.includes(requiredRollbackControl)) {
    throw new Error(`Hermes messages must expose a point-in-time rollback button backed by the default /undo command: ${requiredRollbackControl}`);
  }
}

if (!css.includes("--composer-control-size") || !css.includes("height: var(--composer-control-size)") || !css.includes("min-height: var(--composer-control-size)")) {
  throw new Error("Composer attach, input, and send controls must share one height token");
}

if (!css.includes("padding: 8px 14px") || !css.includes("line-height: 1.35") || !css.includes("min-height: var(--composer-control-size)") || !css.includes("transition: height 220ms cubic-bezier(0.22, 1, 0.36, 1)")) {
  throw new Error("Composer textarea must align to adjacent buttons by default and expand smoothly for multiline drafts");
}

if (!sessionState.includes("Hermes is awake. Drop the mission") || !screen.includes("refreshHermesWelcomeCopy") || !app.includes("refreshHermesWelcomeCopy")) {
  throw new Error("Hermes default message must use the cooler Zoid-local command deck copy and migrate persisted legacy welcome messages");
}

if (!app.includes("nav-icon nav-icon--agent-session") || !css.includes(".nav-icon--agent-session .session-notification-dot") || app.includes('"nav-icon session-tab-icon"')) {
  throw new Error("Primary sidebar Agents icon must not inherit the session-tab boxed icon styling");
}

const composer = readFileSync(new URL("./agents/ChatComposer.tsx", import.meta.url), "utf8");
for (const requiredComposerAction of [
  "composerActions",
  "Attach files",
  "Slash commands",
  "Agent settings",
  "Session usage",
  "multiple",
  "composer-attachment-chip",
  "Send as context",
  "Extract text",
  "Upload only",
  "buildAttachmentContext",
  "Cmd/Ctrl+Enter runs",
  "Session overrides",
  "Default profile",
  "Compact/summarize session",
  "Remove attached files from context",
  "Send unlocks after the current response",
  "Hermes is offline. Draft here",
  "Send failed. Draft and attachments were preserved",
  "Based on Hermes Agent slash-command registry reference",
  "terminal-style commands such as",
  "hermes tools list",
  "keeps terminal plumbing out of the conversation",
  "No fake uploads",
  "Copy usage report",
]) {
  if (!composer.includes(requiredComposerAction) && !css.includes(requiredComposerAction)) {
    throw new Error(`Composer + menu is missing requested action surface: ${requiredComposerAction}`);
  }
}

for (const requiredComposerCss of [
  ".composer-action-popover",
  ".composer-action-row",
  ".composer-deep-panel",
  ".composer-attachment-tray",
  ".composer-status-note",
  ".composer-attach-dropzone",
  ".composer-attachment-rules",
  ".usage-meter",
]) {
  if (!css.includes(requiredComposerCss)) {
    throw new Error(`Composer + menu styling is missing: ${requiredComposerCss}`);
  }
}

for (const requiredHermesCommandParity of [
  "list_hermes_slash_commands",
  "execute_hermes_slash_command",
  "HermesSlashCommand",
  "HermesSlashCommandExecution",
  "zoidBehavior",
  "parseSlashCommand",
  "listHermesSlashCommands()",
  "executeHermesSlashCommand(",
  "CommandPalette",
  "recentCommands",
  "pendingConfirmation",
  "runPendingConfirmedCommand",
  "activeCommandPanel",
  "COMMAND_PANEL_COPY",
  "requiresConfirmation",
  "void runPendingConfirmedCommand(pending)",
]) {
  if (![backend, client, hermesCommands, slashCommandParser, commandPalette, recentCommands, screen].some((source) => source.includes(requiredHermesCommandParity))) {
    throw new Error(`Hermes slash command parity wiring is missing: ${requiredHermesCommandParity}`);
  }
}

for (const requiredHermesCommandStyle of [".command-palette", ".command-palette-section", ".zoid-command-confirm", ".zoid-command-confirm-run", ".zoid-native-command-panel"]) {
  if (!css.includes(requiredHermesCommandStyle)) {
    throw new Error(`Hermes command palette/confirmation styling is missing: ${requiredHermesCommandStyle}`);
  }
}

if (chatComposer.includes("}, [commandSearch]);")) {
  throw new Error("Slash command filtering must update when the live Hermes registry refreshes");
}

if (!recentCommands.includes("looksSensitive") || !recentCommands.includes("MAX_RECENT_COMMANDS")) {
  throw new Error("Recent slash commands must avoid sensitive payloads and stay bounded");
}

if (!backend.includes("from hermes_cli.commands import COMMAND_REGISTRY") || backend.includes("const HERMES_COMMANDS") || backend.includes("static HERMES_COMMANDS")) {
  throw new Error("Zoid must load the live Hermes command registry instead of duplicating a static command list");
}

if (!css.includes("body { margin: 0; min-width: 0;") || !css.includes("width: 100vw") || !css.includes("@media (max-width: 820px)")) {
  throw new Error("Desktop shell must be responsive and not force a fixed minimum width that hides the right side");
}

if (css.includes("min-width: 940px") || css.includes("grid-template-columns: 72px 336px minmax(0, 1fr)")) {
  throw new Error("Fixed desktop sizing must not remain because it can push the right side offscreen");
}

const cssBlockHas = (selector: string, snippets: string[]) => {
  const escapedSelector = selector.replace(/[.*+?^${}()|[\]\\]/g, "\\$&");
  const block = new RegExp(`${escapedSelector}\\s*\\{([^}]*)\\}`).exec(css)?.[1] ?? "";
  return snippets.every((snippet) => block.includes(snippet));
};

if (
  !cssBlockHas(".chat-stats-strip", [
    "display: grid",
    "grid-template-columns: max-content max-content minmax(0, 1fr) minmax(180px, 0.65fr)",
    "align-items: stretch",
  ]) ||
  !cssBlockHas(".chat-stats-strip > span", [
    "display: flex",
    "align-items: center",
    "min-height: 46px",
    "padding: 0 14px",
    "white-space: nowrap",
    "line-height: 1",
  ]) ||
  !cssBlockHas(".chat-stats-strip > span b", ["font-size: inherit", "line-height: 1"]) ||
  !cssBlockHas(".chat-stats-model-copy", ["display: inline-flex", "align-items: center", "line-height: 1"]) ||
  !cssBlockHas(".agents-sumi-e .chat-stats-strip b", [
    "display: inline-flex",
    "align-items: center",
    "font-family: inherit",
    "font-size: inherit",
    "line-height: 1",
    "letter-spacing: inherit",
  ]) ||
  !cssBlockHas(".chat-stats-strip > .chat-stats-model-section", [
    "display: grid",
    "grid-template-columns: minmax(0, 1fr) auto",
  ])
) {
  throw new Error("Hermes stats strip must keep metric text baseline-aligned inside compact cells and flexible model/session cells");
}

for (const requiredChatPolish of [
  "function stripTerminalCommandPlumbing",
  "const visibleContent = stripTerminalCommandPlumbing(message.content);",
  "hasVisibleContent",
  "shouldShowBubble",
  "border-right: 1px solid var(--kujo-ink)",
  ".chat-avatar--hermes::before",
  ".chat-avatar--hermes::after",
  ".chat-avatar--hermes > span:not(.avatar-presence)",
]) {
  if (!screen.includes(requiredChatPolish) && !messageBubble.includes(requiredChatPolish) && !css.includes(requiredChatPolish)) {
    throw new Error(`Hermes chat visual feedback polish is missing: ${requiredChatPolish}`);
  }
}

for (const removedChatPolish of [".sessions-rail::before", ".sessions-rail::after", "hermes-sigil", "hermes-sigil__core"]) {
  if (messageBubble.includes(removedChatPolish) || css.includes(removedChatPolish) || screen.includes(removedChatPolish)) {
    throw new Error(`Rejected decorative chat polish must be removed: ${removedChatPolish}`);
  }
}

if (!css.includes(".ink-rail { display: none; }") || !css.includes(".nav-list { flex-direction: row;")) {
  throw new Error("Narrow desktop layout must reclaim sidebar width for the main workspace");
}

for (const requiredMetric of [
  "Context</b> {contextUsedPercent}%",
  "{compressionCount} compressions",
  "{formatElapsed(promptElapsed)}",
  "Codex {CODEX_USAGE_TODAY} / {CODEX_USAGE_WEEKLY}",
  "{activeModelLabel}",
  "Change model and reasoning",
  "model-command-panel",
  "saveHermesProfileSettings",
  "{cliStatus?.session ?? activeSession?.id ?? \"most-recent-hermes-cli-session\"}",
]) {
  if (!screen.includes(requiredMetric)) {
    throw new Error(`Hermes stats strip is missing metric: ${requiredMetric}`);
  }
}

if (!screen.includes("<span><b>Time</b> {formatElapsed(promptElapsed)}</span>")) {
  throw new Error("Elapsed time must occupy the second Hermes stats section");
}

if (screen.includes("<span>Repository:") || screen.includes("Session: {cliStatus?.session ?? activeSession?.id ?? \"most-recent-hermes-cli-session\"} · Elapsed:")) {
  throw new Error("Hermes stats strip must not show Repository or duplicate Elapsed in the Session section");
}

for (const removedMetric of ["<span>Messages:", "<span>Bridge:", "<span>Operator:"]) {
  if (screen.includes(removedMetric)) {
    throw new Error(`Old Hermes stats strip metric must be replaced: ${removedMetric}`);
  }
}

if (!client.includes("linkedRepository") || !(screen.includes("sendHermesCliRunMessage(") || screen.includes("sendHermesCliMessage(")) || !(screen.includes("selectedRepository?.path") || screen.includes("detectedRepository?.path"))) {
  throw new Error("Selected repository path must be passed into the Hermes send path");
}

if (!screen.includes("linked-repository-select") || !screen.includes("GlobalDropdown") || !css.includes(".zoid-dropdown-trigger") || !css.includes(".zoid-dropdown-menu")) {
  throw new Error("Agent repository linking must use the global Zoid design-system dropdown");
}

if (!css.includes(".zoid-dropdown--compact .zoid-dropdown-trigger { min-height: 26px; padding: 4px 29px 4px 9px; border-radius: 0;") || css.includes(".zoid-dropdown--compact .zoid-dropdown-trigger { min-height: 26px; padding: 4px 29px 4px 9px; border-radius: 5px;")) {
  throw new Error("Compact Zoid dropdown triggers, including Link repository, must stay boxy with zero border radius");
}

if (!globalDropdown.includes("data-global-dropdown") || !globalDropdown.includes("role=\"menu\"") || !globalDropdown.includes("role=\"menuitemradio\"") || !css.includes(".zoid-dropdown-option.is-selected")) {
  throw new Error("Zoid dropdowns must use the global design-system menu component and styling");
}

if (!globalDropdown.includes("handleOptionKeyDown") || !globalDropdown.includes("ArrowDown") || !globalDropdown.includes("Escape") || !globalDropdown.includes("aria-checked") || !globalDropdown.includes("aria-disabled")) {
  throw new Error("GlobalDropdown must keep menu keyboard handling and selected/disabled accessibility states");
}

if (!existsSync(new URL("./ui/GlobalDropdown.behavior.test.tsx", import.meta.url)) || !app.includes("GlobalDropdown") || !chatComposer.includes("GlobalDropdown")) {
  throw new Error("GlobalDropdown must have behavior coverage and be reused by app/composer dropdown surfaces");
}

if (codeWorkspace.includes("<select") || screen.includes("<select") || chatComposer.includes("<select") || app.includes("<select")) {
  throw new Error("Zoid 25 surfaces must use GlobalDropdown instead of native select controls");
}

for (const requiredExplicitRepoDetection of [
  "MIN_REPOSITORY_NAME_DETECTION_LENGTH",
  "promptContainsRepositoryName",
  "promptContainsRepositoryPath",
  "[^a-z0-9_-]",
]) {
  if (!screen.includes(requiredExplicitRepoDetection)) {
    throw new Error(`Repository auto-detection must require explicit repo name/path evidence and avoid substring false positives: ${requiredExplicitRepoDetection}`);
  }
}

for (const requiredProfileFallbackScope of [
  "PROFILE_SETTINGS_STORAGE_PREFIX",
  "fallbackStorageKey(profile",
  "`${PROFILE_SETTINGS_STORAGE_PREFIX}:${profile}`",
]) {
  if (!profileClient.includes(requiredProfileFallbackScope)) {
    throw new Error(`Hermes profile localStorage fallback must be profile-scoped: ${requiredProfileFallbackScope}`);
  }
}

if (screen.includes("normalizedPrompt.includes(candidate)") || profileClient.includes('getItem("zoid25:hermes-profile-settings")')) {
  throw new Error("Repo detection/profile fallback must not use broad substring matching or shared profile storage keys");
}

if (screen.includes("<small>{repositories.length") || screen.includes("Repository: ${linkedRepository}") || css.includes(".repository-link-control small")) {
  throw new Error("Topbar repository helper/path text must be removed");
}

if (!app.includes("hasHermesWaitingNotification") || !app.includes("nav-icon--agent-session") || !app.includes("session-notification-dot") || !css.includes(".session-notification-dot")) {
  throw new Error("Agents navigation/session icon must show a notification dot when Hermes is waiting for the user without inheriting session-tab box styling");
}

for (const requiredAgentReplyNotification of [
  "needsReply?: boolean",
  "activeSessionIdRef",
  "isAgentsWorkspaceOpenRef",
  "notifyForBackgroundAgentResponse",
  "session-reply-indicator",
  "Hermes replied and needs your reply",
]) {
  if (!screen.includes(requiredAgentReplyNotification) && !sessionState.includes(requiredAgentReplyNotification)) {
    throw new Error(`Hermes background replies must set and render per-session needs-reply indicators: ${requiredAgentReplyNotification}`);
  }
}

for (const requiredNotificationClient of [
  "ensureDesktopNotificationPermission",
  "sendDesktopAgentNotification",
  "buildAgentResponseEmailSummary",
  "sendAgentResponseEmailNotification",
  "@tauri-apps/plugin-notification",
  "ziad.ahmed.25.25.25@gmail.com",
]) {
  if (!agentNotifications.includes(requiredNotificationClient)) {
    throw new Error(`Agent response notification client is missing: ${requiredNotificationClient}`);
  }
}

for (const requiredNotificationBackend of [
  "tauri_plugin_notification::init",
  "send_agent_response_email_notification",
  "AgentResponseEmailNotificationRequest",
  "ZOID_NOTIFY_SMTP_HOST",
  "ZOID_NOTIFY_SMTP_PASSWORD",
  "bounded_email_body",
  "bounded_email_header",
  "ziad.ahmed.25.25.25@gmail.com",
]) {
  if (!backend.includes(requiredNotificationBackend)) {
    throw new Error(`Agent response notification backend is missing: ${requiredNotificationBackend}`);
  }
}

for (const requiredNotificationStyle of [".session-reply-indicator", ".sessions-rail--compact .session-reply-indicator"]) {
  if (!css.includes(requiredNotificationStyle)) {
    throw new Error(`Per-session reply indicator must follow the Zoid design system: ${requiredNotificationStyle}`);
  }
}

if (!backend.includes("linked_repository") || !backend.includes("current_dir")) {
  throw new Error("Backend Hermes CLI command must receive and apply a linked repository workdir");
}

if (!backend.includes("scan_github_repositories") || !backend.includes("clone_github_repository") || !backend.includes("list_github_branches") || !backend.includes("update_github_default_branch")) {
  throw new Error("Backend must register repository scan, clone, branch-list, and default-branch update Tauri commands");
}

for (const requiredAutomationSurface of [
  "AutomationsWorkspace",
  "setAutomationsStatus",
  "list_hermes_automations",
  "manage_hermes_cron_job",
  "Hermes automations",
  "Cron Jobs",
  "Watchers",
  "Profile:",
  "Run now",
  "Remove cron job?",
  "job.protected",
]) {
  if (!app.includes(requiredAutomationSurface) && !automationsWorkspace.includes(requiredAutomationSurface) && !automationsClient.includes(requiredAutomationSurface) && !backend.includes(requiredAutomationSurface)) {
    throw new Error(`Automations control plane is missing: ${requiredAutomationSurface}`);
  }
}

for (const requiredAutomationType of ["AutomationCronJob", "HermesWatcher", "AutomationList", "protected", "protectionReason", "watcherSourceStatus"]) {
  if (!automationsTypes.includes(requiredAutomationType)) {
    throw new Error(`Automation type contract is missing: ${requiredAutomationType}`);
  }
}

for (const requiredAutomationCss of [".automations-workspace-shell", ".automation-summary-grid", ".automation-job-card", ".automation-confirm-panel", ".automation-status-badge--error"]) {
  if (!css.includes(requiredAutomationCss)) {
    throw new Error(`Automations styling is missing: ${requiredAutomationCss}`);
  }
}

if (!css.includes(".automation-sumi-e .automations-workspace-header::before") || !css.includes("bottom: 6px; height: 7px") || css.includes("automations-workspace-header::before { content: \"\"; position: absolute; left: 0; right: clamp(80px, 24vw, 340px); bottom: -1px")) {
  throw new Error("Automations header brush divider must stay inside the header so it does not overlap the status text below");
}

if (!backend.includes("protection_reason_for_job") || !backend.includes("Protected cron job cannot be removed") || !backend.includes("watcher_source_status: \"unavailable\"")) {
  throw new Error("Backend automation bridge must enforce protected removes and truthful watcher unavailability");
}

if (!client.includes("check_hermes_cli") || !client.includes("send_hermes_cli_message")) {
  throw new Error("Frontend must invoke Hermes CLI bridge commands");
}

if (!backend.includes('Command::new') || !backend.includes('"hermes"')) {
  throw new Error("Backend must spawn the Hermes CLI, not call an HTTP API");
}

for (const requiredHermesCommandBridge of ["hermes_cli_args_from_prompt", "hermes_chat_args", "command_usage", "hermes tools list", "interactive. Use a non-interactive subcommand", "starts_with(\"--yolo=\")", "argument_started"]) {
  if (!backend.includes(requiredHermesCommandBridge)) {
    throw new Error(`Hermes command bridge must keep CLI execution wired: ${requiredHermesCommandBridge}`);
  }
}

if (backend.includes("Terminal command used")) {
  throw new Error("Hermes chat responses must not print terminal command plumbing in the visible transcript");
}

for (const forbidden of ["/v1/chat/completions", "/v1/models", "API_SERVER_KEY", "ZOID_HERMES_API_KEY", "API server"]) {
  if (backend.includes(forbidden) || screen.includes(forbidden)) {
    throw new Error(`Hermes API wording/path must not remain in active bridge: ${forbidden}`);
  }
}
