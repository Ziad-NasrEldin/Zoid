import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import { Window } from "happy-dom";
import { act, StrictMode, useState } from "react";
import type { ComponentProps } from "react";
import { createRoot, type Root } from "react-dom/client";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { AgentsHermesScreen, createSession, type HermesChatSession } from "./AgentsHermesScreen";
import type { FileManagerDirectoryListing } from "./hermesClient";
import type { HermesSlashCommandExecution } from "./hermesCommands";
import type { CodeRepository } from "../code/types";

const window = new Window({ url: "http://127.0.0.1:1420" }) as unknown as Window & typeof globalThis;
const document = window.document as Document;

Object.assign(globalThis, {
  IS_REACT_ACT_ENVIRONMENT: true,
  window,
  document,
  HTMLElement: window.HTMLElement,
  HTMLButtonElement: window.HTMLButtonElement,
  HTMLInputElement: window.HTMLInputElement,
  HTMLTextAreaElement: window.HTMLTextAreaElement,
  Node: window.Node,
  PointerEvent: window.PointerEvent,
  MouseEvent: window.MouseEvent,
  KeyboardEvent: window.KeyboardEvent,
  Event: window.Event,
  localStorage: window.localStorage,
  requestAnimationFrame: window.requestAnimationFrame.bind(window),
  cancelAnimationFrame: window.cancelAnimationFrame.bind(window),
});

const listings: Record<string, FileManagerDirectoryListing> = {
  home: {
    path: "/Users/ziadnasreldin",
    name: "ziadnasreldin",
    parent: "/Users",
    entries: [
      { name: "Projects", path: "/Users/ziadnasreldin/Projects", kind: "directory", hidden: false, readonly: false, childrenCount: 1 },
      { name: "notes.txt", path: "/Users/ziadnasreldin/notes.txt", kind: "file", hidden: false, readonly: false, size: 1024 },
    ],
  },
  projects: {
    path: "/Users/ziadnasreldin/Projects",
    name: "Projects",
    parent: "/Users/ziadnasreldin",
    entries: [
      { name: "Readme.md", path: "/Users/ziadnasreldin/Projects/Readme.md", kind: "file", hidden: false, readonly: false, size: 2048 },
    ],
  },
  users: {
    path: "/Users",
    name: "Users",
    parent: "/",
    entries: [
      { name: "ziadnasreldin", path: "/Users/ziadnasreldin", kind: "directory", hidden: false, readonly: false, childrenCount: 2 },
    ],
  },
};

const repositories: CodeRepository[] = [
  {
    id: "repo-1",
    name: "Zoid Repo",
    path: "/Users/ziadnasreldin/Zoid",
    remoteUrl: "git@github.com:mavoid/zoid.git",
    branch: "main",
    dirty: false,
    addedAt: "2026-06-09T00:00:00.000Z",
    source: "scanned",
  },
  {
    id: "repo-2",
    name: "Liwan Repo",
    path: "/Users/ziadnasreldin/Liwan",
    remoteUrl: "git@github.com:mavoid/liwan.git",
    branch: "develop",
    dirty: false,
    addedAt: "2026-06-09T00:05:00.000Z",
    source: "scanned",
  },
];

type IpcCall = { cmd: string; args: unknown };

type MockOptions = {
  slashResults?: HermesSlashCommandExecution[];
  sendMessageResult?: { content: string; session: string } | Promise<{ content: string; session: string }> | ((args: unknown) => { content: string; session: string } | Promise<{ content: string; session: string }>);
};

function installMockIpc(calls: IpcCall[] = [], options: MockOptions = {}) {
  const slashResults = [...(options.slashResults ?? [])];
  mockIPC((cmd, args) => {
    calls.push({ cmd, args });
    if (cmd === "check_hermes_cli") {
      return { ok: true, status: "online", message: "Hermes CLI online", session: "test-session" };
    }
    if (cmd === "list_hermes_slash_commands") {
      return [{ name: "danger", aliases: [], description: "Danger command", category: "test", subcommands: [], cliOnly: false, gatewayOnly: false, zoidBehavior: "confirm-forward" }];
    }
    if (cmd === "send_hermes_cli_message" || cmd === "send_hermes_cli_run_message") {
      return typeof options.sendMessageResult === "function" ? options.sendMessageResult(args) : options.sendMessageResult ?? { content: "Hermes response", session: "reply-session" };
    }
    if (cmd === "execute_hermes_slash_command") {
      const next = slashResults.shift();
      if (!next) throw new Error("Unexpected execute_hermes_slash_command call");
      return next;
    }
    if (cmd === "list_file_manager_directory") {
      const path = (args as { path?: string | null } | undefined)?.path;
      if (!path) return listings.home;
      if (path === listings.projects.path) return listings.projects;
      if (path === listings.users.path) return listings.users;
      if (path === listings.home.path) return listings.home;
      throw new Error(`Unexpected file manager path: ${path}`);
    }
    if (cmd === "cancel_hermes_cli_run" || cmd === "cancel_hermes_cli_message") return true;
    if (cmd === "send_agent_response_email_notification") {
      throw new Error("email notifications must not be sent by default from AgentsHermesScreen");
    }
    if (cmd === "send_desktop_agent_notification") return { ok: true };
    throw new Error(`Unexpected command: ${cmd}`);
  });
}

async function click(element: Element) {
  await act(async () => {
    element.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true }) as unknown as Event);
  });
}

async function keyDown(element: Element, key: string, options: KeyboardEventInit = {}) {
  await act(async () => {
    element.dispatchEvent(new window.KeyboardEvent("keydown", { bubbles: true, cancelable: true, key, ...options }) as unknown as Event);
  });
}

async function settle() {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
    await Promise.resolve();
    await new Promise((resolve) => window.setTimeout(resolve, 0));
    await Promise.resolve();
  });
}

function createDomEvent(type: string, bubbles = true, cancelable = false) {
  const event = document.createEvent("Event");
  event.initEvent(type, bubbles, cancelable);
  return event;
}

type MockDataTransfer = {
  dropEffect: string;
  effectAllowed: string;
  getData: (type: string) => string;
  setData: (type: string, value: string) => void;
};

function createDragEvent(type: string, dataTransfer: MockDataTransfer) {
  const event = createDomEvent(type, true, true) as Event & { dataTransfer: MockDataTransfer; relatedTarget?: EventTarget | null };
  Object.defineProperty(event, "dataTransfer", { configurable: true, value: dataTransfer });
  Object.defineProperty(event, "relatedTarget", { configurable: true, value: null });
  return event;
}

function createPointerEvent(type: string, options: { clientX: number; clientY: number; button?: number }) {
  const event = createDomEvent(type, true, true) as Event & { clientX: number; clientY: number; button: number; pointerId: number; preventDefault: () => void; defaultPrevented: boolean };
  let defaultPrevented = false;
  Object.defineProperty(event, "clientX", { configurable: true, value: options.clientX });
  Object.defineProperty(event, "clientY", { configurable: true, value: options.clientY });
  Object.defineProperty(event, "button", { configurable: true, value: options.button ?? 0 });
  Object.defineProperty(event, "pointerId", { configurable: true, value: 1 });
  Object.defineProperty(event, "defaultPrevented", { configurable: true, get: () => defaultPrevented });
  event.preventDefault = () => { defaultPrevented = true; };
  return event;
}

function createMockDataTransfer(): MockDataTransfer {
  const data = new Map<string, string>();
  return {
    dropEffect: "none",
    effectAllowed: "all",
    getData: (type: string) => data.get(type) ?? "",
    setData: (type: string, value: string) => { data.set(type, value); },
  };
}

async function inputTextarea(element: HTMLTextAreaElement, value: string) {
  await act(async () => {
    const valueSetter = Object.getOwnPropertyDescriptor(window.HTMLTextAreaElement.prototype, "value")?.set;
    valueSetter?.call(element, value);
    element.dispatchEvent(createDomEvent("input"));
    element.dispatchEvent(createDomEvent("change"));
  });
}

async function submitComposer(container: HTMLElement, message: string) {
  const textarea = container.querySelector<HTMLTextAreaElement>("textarea");
  assert.ok(textarea, "composer textarea should render");
  await inputTextarea(textarea, message);
  const form = container.querySelector<HTMLFormElement>('form[aria-label="Hermes message composer"]');
  assert.ok(form, "composer form should render");
  await act(async () => {
    form.dispatchEvent(createDomEvent("submit", true, true));
  });
  await settle();
}

function linkedRepositoryTrigger(container: HTMLElement) {
  const trigger = container.querySelector<HTMLButtonElement>("#linked-repository-select");
  assert.ok(trigger, "linked repository dropdown should render");
  return trigger;
}

function linkedRepositoryLabel(container: HTMLElement) {
  return linkedRepositoryTrigger(container).querySelector(".zoid-dropdown-value")?.textContent ?? "";
}

async function selectLinkedRepository(container: HTMLElement, label: string) {
  await click(linkedRepositoryTrigger(container));
  const option = [...container.querySelectorAll<HTMLButtonElement>(".zoid-dropdown-option")].find((button) => button.textContent?.includes(label));
  assert.ok(option, `linked repository option should render: ${label}`);
  await click(option);
  await settle();
}

async function renderHermesScreen(options: { repositories?: CodeRepository[]; sessionCount?: number; initialSessions?: HermesChatSession[]; strictMode?: boolean; onRepositoryOperationComplete?: ComponentProps<typeof AgentsHermesScreen>["onRepositoryOperationComplete"] } = {}): Promise<{ container: HTMLDivElement; root: Root }> {
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const root = createRoot(container);

  function Harness() {
    const [sessions, setSessions] = useState<HermesChatSession[]>(() => options.initialSessions ?? Array.from({ length: options.sessionCount ?? 1 }, (_, index) => createSession(index === 0 ? "Finder test" : `Overflow session ${index}`)));
    const [activeSessionId, setActiveSessionId] = useState(() => sessions[0]?.id ?? "missing");
    return (
      <AgentsHermesScreen
        activeSessionId={activeSessionId}
        onActiveSessionIdChange={setActiveSessionId}
        onArchiveSession={() => undefined}
        onRepositoryOperationComplete={options.onRepositoryOperationComplete}
        onSessionsChange={setSessions}
        repositories={options.repositories ?? []}
        sessions={sessions}
      />
    );
  }

  await act(async () => {
    root.render(options.strictMode ? <StrictMode><Harness /></StrictMode> : <Harness />);
  });
  await settle();
  return { container, root };
}

async function runFileManagerTests() {
  const calls: IpcCall[] = [];
  installMockIpc(calls);
  const { container, root } = await renderHermesScreen();

  const openButton = container.querySelector<HTMLButtonElement>('button[aria-label="Open file manager sidebar"]');
  assert.ok(openButton, "topbar should render an Open file manager sidebar button");
  const sessionsResizeHandle = container.querySelector<HTMLButtonElement>(".sessions-rail-resize-handle");
  assert.ok(sessionsResizeHandle, "expanded sessions rail should render a real resize handle");
  assert.equal(sessionsResizeHandle.getAttribute("role"), "separator", "Sessions resize handle should expose separator semantics");
  assert.equal(sessionsResizeHandle.getAttribute("aria-orientation"), "vertical", "Sessions resize handle should expose vertical orientation");
  assert.equal(sessionsResizeHandle.getAttribute("aria-valuemin"), "124", "Sessions resize handle should expose minimum width");
  const sessionsWorkspace = container.querySelector<HTMLElement>(".chat-workspace");
  assert.ok(sessionsWorkspace, "chat workspace should render before resizing tests");
  const sessionsWidthBefore = sessionsWorkspace.style.getPropertyValue("--sessions-rail-width");
  await keyDown(sessionsResizeHandle, "ArrowRight");
  const sessionsWidthAfter = sessionsWorkspace.style.getPropertyValue("--sessions-rail-width");
  assert.notEqual(sessionsWidthAfter, sessionsWidthBefore, "keyboard resizing the Sessions rail should update the real layout width variable");
  await click(openButton);
  await settle();

  const openedSidebar = container.querySelector<HTMLElement>(".file-manager-sidebar");
  assert.ok(openedSidebar, "opening should render the right file manager sidebar");
  assert.ok(openedSidebar.classList.contains("file-manager-sidebar--open"), "opened Finder sidebar should receive the visible motion state class");
  assert.equal(openedSidebar.inert, false, "opened Finder sidebar should remain interactive");
  assert.equal(container.querySelector('[role="tree"]'), null, "Finder list should not claim ARIA tree semantics without tree keyboard handling");
  assert.equal(container.querySelector('[role="treeitem"]'), null, "Finder rows should use native list semantics unless a full ARIA tree is implemented");
  assert.match(container.textContent ?? "", /\/Users\/ziadnasreldin/, "initial root should show the macOS home path");
  assert.match(container.textContent ?? "", /Projects/, "initial root should render folder entries");

  const projectsButton = [...container.querySelectorAll<HTMLButtonElement>(".file-manager-item--folder")].find((button) => button.textContent?.includes("Projects"));
  assert.ok(projectsButton, "folder entries should be clickable");
  assert.equal(projectsButton.getAttribute("aria-expanded"), "false", "collapsed folder row should expose collapsed state on the focusable button");
  await click(projectsButton);
  await settle();
  assert.match(container.textContent ?? "", /Readme\.md/, "clicking a folder should expand and render nested contents");
  assert.ok(container.querySelector(".file-manager-branch"), "expanded folder contents should be wrapped in the animated branch container");

  const expandedProjectsButton = [...container.querySelectorAll<HTMLButtonElement>(".file-manager-item--folder")].find((button) => button.textContent?.includes("Projects"));
  assert.ok(expandedProjectsButton, "expanded Projects folder row should remain clickable");
  assert.equal(expandedProjectsButton.getAttribute("aria-expanded"), "true", "expanded nested folder should expose expanded state on its button");
  await click(expandedProjectsButton);
  await settle();
  assert.doesNotMatch(container.textContent ?? "", /Readme\.md/, "clicking an expanded folder should collapse nested contents");

  const fileRows = [...container.querySelectorAll<HTMLElement>(".file-manager-item")].filter((item) => item.textContent?.includes("notes.txt"));
  assert.ok(fileRows.length > 0, "file entries should remain visible/readable after removing disabled file buttons");
  assert.equal(fileRows[0].tagName, "DIV", "non-actionable file rows should not be disabled buttons");

  const toolbarButtons = [...container.querySelectorAll<HTMLButtonElement>(".file-manager-toolbar button")];
  assert.ok(!toolbarButtons.some((button) => button.textContent === "Up"), "sidebar should not render the removed useless Up button");
  assert.ok(toolbarButtons.some((button) => button.textContent === "Refresh"), "sidebar should keep the useful Refresh action");

  const resizeHandle = container.querySelector<HTMLButtonElement>(".file-manager-resize-handle");
  assert.ok(resizeHandle, "sidebar should render a drag handle for resizing the Finder panel");
  assert.equal(resizeHandle.getAttribute("role"), "separator", "Finder resize handle should expose separator semantics");
  assert.equal(resizeHandle.getAttribute("aria-orientation"), "vertical", "Finder resize handle should expose vertical orientation");
  const workspace = container.querySelector<HTMLElement>(".chat-workspace");
  assert.ok(workspace, "chat workspace should own the Finder width CSS variable");
  const widthBefore = workspace.style.getPropertyValue("--file-manager-width");
  await act(async () => {
    resizeHandle.dispatchEvent(new window.PointerEvent("pointerdown", { bubbles: true, clientX: 500 }) as unknown as Event);
    window.dispatchEvent(new window.PointerEvent("pointermove", { bubbles: true, clientX: 440 }) as unknown as Event);
    window.dispatchEvent(new window.PointerEvent("pointerup", { bubbles: true, clientX: 440 }) as unknown as Event);
  });
  const widthAfter = workspace.style.getPropertyValue("--file-manager-width");
  assert.notEqual(widthAfter, widthBefore, "dragging the Finder resize handle should change the real layout width variable");

  const closeButton = container.querySelector<HTMLButtonElement>('.file-manager-header button[aria-label="Close file manager"]');
  assert.ok(closeButton, "Finder sidebar should expose a close button");
  await click(closeButton);
  const closingSidebar = container.querySelector<HTMLElement>(".file-manager-sidebar");
  assert.ok(closingSidebar, "closing Finder sidebar should remain mounted for exit motion");
  assert.ok(closingSidebar.classList.contains("file-manager-sidebar--closed"), "closing Finder sidebar should receive the exit motion state class");
  assert.equal(closingSidebar.inert, true, "closing Finder sidebar should be inert while exit animation runs");

  const css = readFileSync(new URL("../App.css", import.meta.url), "utf8");
  assert.ok(css.includes(".chat-workspace--file-manager-open .file-manager-sidebar { grid-column: 1; grid-row: 3;"), "narrow layout should place the file manager in the real single-column grid instead of implicit column 3");
  assert.ok(css.includes(".file-manager-sidebar { position: relative;") && css.includes("opacity: 1; pointer-events: auto; transform: translateX(0) scaleX(1);"), "base Finder sidebar style should be visible so missing/open-state timing never leaves it invisible");
  assert.ok(css.includes(".file-manager-sidebar--closed"), "Finder sidebar should keep a closing state for motion instead of disappearing immediately");
  assert.ok(css.includes(".file-manager-branch"), "Finder folder expansion should animate nested branch reveal");
  assert.ok(css.includes("textarea { resize: none; }"), "textareas should not show native resize handles in bottom-anchored composers");
  assert.ok(css.includes('grid-template-areas: "status commands layout"'), "agent monitor controls should stay in one row");
  assert.ok(css.includes(".agents-sumi-e .agent-monitor-grid--count-4 .agent-monitor-status-strip { min-height: 18px;"), "four-panel idle/status rows should stay compact");

  await act(async () => root.unmount());
  clearMocks();
}

async function runSessionsOverflowCueTests() {
  installMockIpc();
  const { container, root } = await renderHermesScreen({ sessionCount: 12 });

  const sessionsList = container.querySelector<HTMLElement>(".sessions-list");
  assert.ok(sessionsList, "sessions list should render");
  const css = readFileSync(new URL("../App.css", import.meta.url), "utf8");
  assert.ok(css.includes(".sessions-list::-webkit-scrollbar"), "sessions list should hide the native WebKit scrollbar");
  assert.ok(css.includes("scrollbar-width: none"), "sessions list should hide the native Firefox scrollbar");
  assert.ok(css.includes("@keyframes sessions-overflow-cue-flow"), "sessions overflow cue should have a flowing animation");
  assert.ok(css.includes("flex: 1 1 auto"), "sessions list should grow to the rail body instead of collapsing to zero height");
  assert.equal(container.querySelector('button[aria-label="More sessions below"]'), null, "sessions cue should be absent before a real overflow is measured");

  Object.defineProperty(sessionsList, "clientHeight", { configurable: true, value: 0 });
  Object.defineProperty(sessionsList, "scrollHeight", { configurable: true, value: 120 });
  Object.defineProperty(sessionsList, "scrollTop", { configurable: true, writable: true, value: 0 });
  await act(async () => {
    sessionsList.dispatchEvent(createDomEvent("scroll"));
  });
  assert.equal(container.querySelector('button[aria-label="More sessions below"]'), null, "zero-height lists must not trigger a false overflow cue");

  Object.defineProperty(sessionsList, "clientHeight", { configurable: true, value: 120 });
  Object.defineProperty(sessionsList, "scrollHeight", { configurable: true, value: 420 });
  Object.defineProperty(sessionsList, "scrollTop", { configurable: true, writable: true, value: 0 });
  await act(async () => {
    sessionsList.dispatchEvent(createDomEvent("scroll"));
  });
  assert.ok(container.querySelector<HTMLButtonElement>('button[aria-label="More sessions below"]'), "overflow cue should appear when sessions continue below the top of the list");

  sessionsList.scrollTop = 18;
  await act(async () => {
    sessionsList.dispatchEvent(createDomEvent("scroll"));
  });
  assert.equal(container.querySelector('button[aria-label="More sessions below"]'), null, "overflow cue should disappear as soon as the user scrolls down");

  await act(async () => root.unmount());
  clearMocks();
}

async function runDashboardDragDropTests() {
  window.localStorage.clear();
  installMockIpc();
  const initialSessions: HermesChatSession[] = [
    { ...createSession("Primary"), id: "drag-a", title: "Primary" },
    { ...createSession("Secondary"), id: "drag-b", title: "Secondary" },
    { ...createSession("Third"), id: "drag-c", title: "Third" },
    { ...createSession("Fourth"), id: "drag-d", title: "Fourth" },
  ];
  const { container, root } = await renderHermesScreen({ initialSessions });

  const mainPane = container.querySelector<HTMLElement>(".chat-main-pane--dashboard");
  assert.ok(mainPane, "dashboard chat pane should be the drop target");
  const secondaryHandle = container.querySelector<HTMLElement>('[data-dashboard-drag-session="drag-b"]');
  assert.ok(secondaryHandle, "session portrait icon should expose a drag handle");
  assert.equal(secondaryHandle.getAttribute("draggable"), "true", "drag must start from the session icon itself");

  const firstDrag = createMockDataTransfer();
  await act(async () => {
    secondaryHandle.dispatchEvent(createDragEvent("dragstart", firstDrag));
    mainPane.dispatchEvent(createDragEvent("dragover", firstDrag));
  });
  assert.equal(firstDrag.effectAllowed, "copy", "session icon drag should advertise copy semantics for tiling");
  assert.equal(firstDrag.dropEffect, "copy", "chat pane should accept the session drop as a split action");
  assert.ok(mainPane.classList.contains("chat-main-pane--drop-armed"), "chat pane should show a drop-armed state while hovering with a session icon");
  await act(async () => {
    mainPane.dispatchEvent(createDragEvent("drop", firstDrag));
  });
  await settle();

  let panels = [...container.querySelectorAll<HTMLElement>(".agent-monitor-panel")];
  assert.equal(panels.length, 2, "dropping one session onto the single main chat should split into two chat panels");
  assert.ok(container.querySelector(".agent-monitor-grid--split-2.agent-monitor-grid--count-2"), "two dropped chats should use the split-2 dashboard layout");
  assert.ok(panels[0].classList.contains("agent-monitor-panel--primary"), "the original main chat should become the primary panel");
  assert.match(container.textContent ?? "", /Secondary/, "the dropped session should become the second panel");

  const thirdHandle = container.querySelector<HTMLElement>('[data-dashboard-drag-session="drag-c"]');
  assert.ok(thirdHandle, "third session drag handle should render");
  Object.defineProperty(mainPane, "getBoundingClientRect", {
    configurable: true,
    value: () => ({ left: 100, top: 100, right: 900, bottom: 700, width: 800, height: 600, x: 100, y: 100, toJSON: () => ({}) }),
  });
  await act(async () => {
    thirdHandle.dispatchEvent(createPointerEvent("pointerdown", { clientX: 20, clientY: 220 }));
    window.dispatchEvent(createPointerEvent("pointermove", { clientX: 220, clientY: 220 }));
  });
  assert.ok(mainPane.classList.contains("chat-main-pane--drop-armed"), "pointer dragging the session icon over the chat pane should arm the real drop target");
  await act(async () => {
    window.dispatchEvent(createPointerEvent("pointerup", { clientX: 220, clientY: 220 }));
  });
  await settle();

  panels = [...container.querySelectorAll<HTMLElement>(".agent-monitor-panel")];
  assert.equal(panels.length, 3, "dropping into two open chats should split into three chat panels");
  assert.ok(container.querySelector(".agent-monitor-grid--focus-stack.agent-monitor-grid--count-3"), "three chats should use the primary plus two secondary layout");

  const fourthHandle = container.querySelector<HTMLElement>('[data-dashboard-drag-session="drag-d"]');
  assert.ok(fourthHandle, "fourth session drag handle should render");
  await act(async () => {
    fourthHandle.dispatchEvent(createPointerEvent("pointerdown", { clientX: 20, clientY: 280 }));
    window.dispatchEvent(createPointerEvent("pointermove", { clientX: 260, clientY: 260 }));
    window.dispatchEvent(createPointerEvent("pointerup", { clientX: 260, clientY: 260 }));
  });
  await settle();

  panels = [...container.querySelectorAll<HTMLElement>(".agent-monitor-panel")];
  assert.equal(panels.length, 4, "dropping into three open chats should split into four chat panels");
  assert.ok(container.querySelector(".agent-monitor-grid--quad.agent-monitor-grid--count-4"), "four chats should use the quad layout");
  const persistedDashboard = JSON.parse(window.localStorage.getItem("zoid25:agents-dashboard") ?? "{}");
  assert.deepEqual(persistedDashboard.tiledSessionIds, ["drag-a", "drag-b", "drag-c", "drag-d"], "drag/drop dashboard layout should persist all four tiled sessions in order");
  assert.equal(persistedDashboard.primarySessionId, "drag-a", "the original main chat should persist as the primary panel");

  const css = readFileSync(new URL("../App.css", import.meta.url), "utf8");
  assert.ok(css.includes("agent-dashboard-panel-enter"), "newly split panels should have a smooth enter animation");
  assert.ok(css.includes("chat-main-pane--drop-armed"), "drop target should have a visible smooth armed state");

  await act(async () => root.unmount());
  clearMocks();
}

async function runRepositoryLinkingTests() {
  const calls: IpcCall[] = [];
  installMockIpc(calls);
  const { container, root } = await renderHermesScreen({ repositories });

  assert.equal(linkedRepositoryLabel(container), "Unlinked / 未接続", "new Agents sessions should ignore the global Code repository link and start unlinked");
  await submitComposer(container, "generic task with no repository mention");

  const sendCall = calls.find((call) => call.cmd === "send_hermes_cli_run_message" || call.cmd === "send_hermes_cli_message");
  assert.ok(sendCall, "sending a normal prompt should call the Hermes CLI bridge");
  assert.equal((sendCall.args as { linkedRepository?: string }).linkedRepository, undefined, "new/unlinked sessions must not send the globally linked repository path");
  assert.equal(calls.some((call) => call.cmd === "send_agent_response_email_notification"), false, "Agents must not send email notifications by default");

  await submitComposer(container, "work in Liwan Repo on the reporting issue");
  assert.equal(linkedRepositoryLabel(container), "Liwan Repo", "mentioning a known repository should auto-link only the active session");
  const hermesSendCalls = calls.filter((call) => call.cmd === "send_hermes_cli_run_message" || call.cmd === "send_hermes_cli_message");
  const detectedSendCall = hermesSendCalls[hermesSendCalls.length - 1];
  assert.ok(detectedSendCall, "auto-detected prompt should be sent to Hermes");
  assert.equal((detectedSendCall.args as { linkedRepository?: string }).linkedRepository, repositories[1].path, "auto-detected repository path should be sent for the active session");

  const newSessionButton = container.querySelector<HTMLButtonElement>('button[aria-label="New session"]');
  assert.ok(newSessionButton, "new session button should render");
  await click(newSessionButton);
  await settle();
  assert.equal(linkedRepositoryLabel(container), "Unlinked / 未接続", "creating a new session should not inherit another session repository link");

  await selectLinkedRepository(container, "Zoid Repo");
  assert.equal(linkedRepositoryLabel(container), "Zoid Repo", "manual selection should update the active session dropdown");

  const liwanSessionButton = [...container.querySelectorAll<HTMLButtonElement>(".session-tab")].find((button) => button.getAttribute("aria-label")?.includes("Open session Finder test"));
  assert.ok(liwanSessionButton, "original auto-linked session should remain available in the sessions rail");
  await click(liwanSessionButton);
  await settle();
  assert.equal(linkedRepositoryLabel(container), "Liwan Repo", "switching sessions should restore that session's repository link");

  const zoidSessionButton = [...container.querySelectorAll<HTMLButtonElement>(".session-tab")].find((button) => button.getAttribute("aria-label")?.includes("Open session New session"));
  assert.ok(zoidSessionButton, "manually linked new session should remain available in the sessions rail");
  await click(zoidSessionButton);
  await settle();
  assert.equal(linkedRepositoryLabel(container), "Zoid Repo", "manual repository selection must not spill into other sessions");

  await act(async () => root.unmount());
  clearMocks();
}

async function runSlashConfirmationPreservationTests() {
  const calls: IpcCall[] = [];
  installMockIpc(calls, {
    slashResults: [
      { kind: "confirmation", command: "/danger", content: "Confirmation required.", requiresConfirmation: true, scope: "current-session" },
      { kind: "text", command: "/danger", content: "Danger command complete.", requiresConfirmation: false, scope: "current-session" },
    ],
  });
  const { container, root } = await renderHermesScreen();

  await submitComposer(container, "/danger");
  assert.match(container.textContent ?? "", /Confirm command/, "slash command should open a confirmation dialog");

  await submitComposer(container, "keep this message");
  assert.match(container.textContent ?? "", /keep this message/, "intervening user message should render before the command is confirmed");

  const runButton = [...container.querySelectorAll<HTMLButtonElement>("button")].find((button) => button.textContent === "Run command");
  assert.ok(runButton, "confirmation dialog should render Run command");
  await click(runButton);
  await settle();

  assert.match(container.textContent ?? "", /keep this message/, "confirmed slash command completion must not drop intervening conversation messages");
  assert.match(container.textContent ?? "", /Danger command complete\./, "confirmed slash command should update its original assistant placeholder");

  await act(async () => root.unmount());
  clearMocks();
}

function createDeferred<T>() {
  let resolve!: (value: T) => void;
  let reject!: (error: unknown) => void;
  const promise = new Promise<T>((promiseResolve, promiseReject) => {
    resolve = promiseResolve;
    reject = promiseReject;
  });
  return { promise, resolve, reject };
}

async function runQueuedSlashCommandTests() {
  const calls: IpcCall[] = [];
  const firstPrompt = createDeferred<{ content: string; session: string }>();
  installMockIpc(calls, {
    sendMessageResult: firstPrompt.promise,
    slashResults: [
      { kind: "text", command: "/danger", content: "Queued danger command complete.", requiresConfirmation: false, scope: "current-session" },
    ],
  });
  const { container, root } = await renderHermesScreen();

  await submitComposer(container, "slow normal prompt");
  assert.equal(calls.filter((call) => call.cmd === "send_hermes_cli_run_message" || call.cmd === "send_hermes_cli_message").length, 1, "first prompt should start a normal Hermes send");

  await submitComposer(container, "/danger");
  assert.equal(calls.filter((call) => call.cmd === "execute_hermes_slash_command").length, 0, "slash command should wait while another Hermes run is active");

  await act(async () => {
    firstPrompt.resolve({ content: "Slow prompt complete.", session: "reply-session" });
    await firstPrompt.promise;
  });
  await settle();
  await settle();

  assert.equal(calls.filter((call) => call.cmd === "send_hermes_cli_run_message" || call.cmd === "send_hermes_cli_message").length, 1, "queued slash command must not be downgraded into a normal prompt send");
  const slashCall = calls.find((call) => call.cmd === "execute_hermes_slash_command");
  assert.ok(slashCall, "queued slash command should execute through the slash command bridge after the active run finishes");
  assert.equal((slashCall.args as { command?: string }).command, "/danger", "queued slash command should preserve the original command text");
  assert.match(container.textContent ?? "", /Queued danger command complete\./, "queued slash command result should render in the session");

  await act(async () => root.unmount());
  clearMocks();
}

async function runParallelDashboardRuntimeTests() {
  const calls: IpcCall[] = [];
  const deferredBySession = new Map<string, ReturnType<typeof createDeferred<{ content: string; session: string }>>>();
  installMockIpc(calls, {
    sendMessageResult: (args) => {
      const sessionId = (args as { sessionId?: string }).sessionId ?? "missing";
      const deferred = createDeferred<{ content: string; session: string }>();
      deferredBySession.set(sessionId, deferred);
      return deferred.promise;
    },
  });
  const initialSessions = ["agent-a", "agent-b", "agent-c", "agent-d"].map((id) => ({ ...createSession(`Panel ${id}`), id }));
  window.localStorage.setItem("zoid25:agents-dashboard", JSON.stringify({
    version: 1,
    tiledSessionIds: initialSessions.map((session) => session.id),
    primarySessionId: initialSessions[0].id,
    focusedSessionId: initialSessions[0].id,
    layoutMode: "quad",
    autoPrioritize: false,
  }));
  const { container, root } = await renderHermesScreen({ initialSessions });

  async function submitPanel(sessionTitle: string, prompt: string) {
    const textarea = container.querySelector<HTMLTextAreaElement>(`textarea[aria-label="Prompt ${sessionTitle}"]`);
    assert.ok(textarea, `panel textarea should render for ${sessionTitle}`);
    await inputTextarea(textarea, prompt);
    await settle();
    const panel = textarea.closest(".agent-monitor-panel");
    assert.ok(panel, `panel should contain ${sessionTitle}`);
    const sendButton = [...panel.querySelectorAll<HTMLButtonElement>("button")].find((button) => button.textContent === "Send" || button.textContent === "Queue");
    assert.ok(sendButton, `panel send button should render for ${sessionTitle}`);
    assert.equal(sendButton.disabled, false, `panel send button should be enabled for ${sessionTitle}`);
    await click(sendButton);
  }

  await submitPanel("Panel agent-a", "run a");
  await submitPanel("Panel agent-b", "run b");
  await submitPanel("Panel agent-c", "run c");
  await submitPanel("Panel agent-d", "run d");
  await settle();

  const sendCalls = calls.filter((call) => call.cmd === "send_hermes_cli_run_message");
  assert.equal(sendCalls.length, 4, "four tiled panel sends should create four concurrent isolated invoke calls");
  assert.deepEqual(new Set(sendCalls.map((call) => (call.args as { sessionId?: string }).sessionId)), new Set(initialSessions.map((session) => session.id)), "concurrent invokes should carry distinct frontend session ids");
  assert.equal(container.textContent?.includes("4 running"), true, "dashboard should show four independent running sessions");

  await submitPanel("Panel agent-a", "queued a");
  await settle();
  assert.equal(calls.filter((call) => call.cmd === "send_hermes_cli_run_message").length, 4, "same-session send should queue while that session is running instead of starting a fifth invoke");
  assert.match(container.textContent ?? "", /1 queued/, "same-session queue should be visible on the owning panel");

  const stopButton = [...container.querySelectorAll<HTMLButtonElement>(".agent-monitor-panel button")].find((button) => button.textContent === "Stop" && button.closest(".agent-monitor-panel")?.textContent?.includes("Panel agent-b"));
  assert.ok(stopButton, "stop button should render for panel agent-b");
  await click(stopButton);
  await settle();
  const cancelCall = calls.find((call) => call.cmd === "cancel_hermes_cli_run");
  assert.ok(cancelCall, "stopping one panel should call the scoped cancel bridge");
  assert.equal((cancelCall.args as { sessionId?: string }).sessionId, "agent-b", "stop must be scoped to the stopped panel session");
  assert.equal(calls.filter((call) => call.cmd === "send_hermes_cli_run_message").length, 4, "stopping one run must not cancel or restart unrelated active panel sends");

  await act(async () => {
    deferredBySession.get("agent-a")?.resolve({ content: "A complete", session: "session-a" });
  });
  await settle();
  await settle();
  const afterQueueCalls = calls.filter((call) => call.cmd === "send_hermes_cli_run_message");
  assert.equal(afterQueueCalls.length, 5, "queued same-session prompt should start after that session's run finishes");
  assert.equal((afterQueueCalls[4].args as { sessionId?: string }).sessionId, "agent-a", "dequeued prompt should stay scoped to its original session");

  await act(async () => root.unmount());
  clearMocks();
}

async function runRepositoryOperationInitialPromptTests() {
  const calls: IpcCall[] = [];
  const completions: Array<Parameters<NonNullable<ComponentProps<typeof AgentsHermesScreen>["onRepositoryOperationComplete"]>>[0]> = [];
  installMockIpc(calls);
  const session: HermesChatSession = {
    ...createSession("Localhost · Zoid Repo"),
    linkedRepositoryId: "repo-1",
    operationRunId: "run-1",
    operationAction: "localhost",
    operationRepositoryId: "repo-1",
    pendingInitialPrompt: "Run localhost for Zoid Repo",
  };

  const { root } = await renderHermesScreen({
    repositories,
    initialSessions: [session],
    strictMode: true,
    onRepositoryOperationComplete: (result) => completions.push(result),
  });
  await settle();
  await settle();

  const sendCalls = calls.filter((call) => call.cmd === "send_hermes_cli_run_message" || call.cmd === "send_hermes_cli_message");
  assert.equal(sendCalls.length, 1, "StrictMode pendingInitialPrompt should send exactly once");
  const sentMessages = (sendCalls[0].args as { messages?: Array<{ content: string }> }).messages ?? [];
  assert.equal(sentMessages[sentMessages.length - 1]?.content, "Run localhost for Zoid Repo", "auto-send should send the prepared repository operation prompt");
  assert.equal(completions.length, 1, "repository operation completion should be reported once");
  assert.equal(completions[0].outcome, "needs-user", "unstructured Hermes responses should default to needs-user/needs-review, not learned success");

  await act(async () => root.unmount());
  clearMocks();
}

const agentsSource = readFileSync(new URL("./AgentsHermesScreen.tsx", import.meta.url), "utf8");
assert.ok(!agentsSource.includes("ziad.ahmed.25.25.25@gmail.com"), "Agents UI must not hardcode an email notification recipient");
assert.ok(!agentsSource.includes("sendAgentResponseEmailNotification({"), "Agents UI must not send email notifications without explicit opt-in settings");

await runFileManagerTests();
await runSessionsOverflowCueTests();
await runDashboardDragDropTests();
await runRepositoryLinkingTests();
await runSlashConfirmationPreservationTests();
await runQueuedSlashCommandTests();
await runParallelDashboardRuntimeTests();
await runRepositoryOperationInitialPromptTests();
