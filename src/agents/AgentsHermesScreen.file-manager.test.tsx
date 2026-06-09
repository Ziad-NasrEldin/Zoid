import { strict as assert } from "node:assert";
import { Window } from "happy-dom";
import { act, useState } from "react";
import { createRoot, type Root } from "react-dom/client";
import { mockIPC, clearMocks } from "@tauri-apps/api/mocks";
import { AgentsHermesScreen, createSession, type HermesChatSession } from "./AgentsHermesScreen";
import type { FileManagerDirectoryListing } from "./hermesClient";

const window = new Window({ url: "http://127.0.0.1:1420" }) as unknown as Window & typeof globalThis;
const document = window.document as Document;

Object.assign(globalThis, {
  IS_REACT_ACT_ENVIRONMENT: true,
  window,
  document,
  HTMLElement: window.HTMLElement,
  HTMLButtonElement: window.HTMLButtonElement,
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

function installMockIpc() {
  mockIPC((cmd, args) => {
    if (cmd === "check_hermes_cli") {
      return { ok: true, status: "online", message: "Hermes CLI online", session: "test-session" };
    }
    if (cmd === "list_hermes_slash_commands") return [];
    if (cmd === "list_file_manager_directory") {
      const path = (args as { path?: string | null } | undefined)?.path;
      if (!path) return listings.home;
      if (path === listings.projects.path) return listings.projects;
      if (path === listings.users.path) return listings.users;
      if (path === listings.home.path) return listings.home;
      throw new Error(`Unexpected file manager path: ${path}`);
    }
    throw new Error(`Unexpected command: ${cmd}`);
  });
}

async function click(element: Element) {
  await act(async () => {
    element.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true }) as unknown as Event);
  });
}

async function settle() {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
}

async function renderHermesScreen(): Promise<{ container: HTMLDivElement; root: Root }> {
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const root = createRoot(container);

  function Harness() {
    const [sessions, setSessions] = useState<HermesChatSession[]>(() => [createSession("Finder test")]);
    const activeSessionId = sessions[0]?.id ?? "missing";
    return (
      <AgentsHermesScreen
        activeSessionId={activeSessionId}
        onActiveSessionIdChange={() => undefined}
        onArchiveSession={() => undefined}
        onLinkedRepositoryIdChange={() => undefined}
        onSessionsChange={setSessions}
        repositories={[]}
        sessions={sessions}
      />
    );
  }

  await act(async () => {
    root.render(<Harness />);
  });
  await settle();
  return { container, root };
}

async function runTests() {
  installMockIpc();
  const { container, root } = await renderHermesScreen();

  const openButton = container.querySelector<HTMLButtonElement>('button[aria-label="Open file manager sidebar"]');
  assert.ok(openButton, "topbar should render an Open file manager sidebar button");
  await click(openButton);
  await settle();

  assert.ok(container.querySelector(".file-manager-sidebar"), "opening should render the right file manager sidebar");
  assert.match(container.textContent ?? "", /\/Users\/ziadnasreldin/, "initial root should show the macOS home path");
  assert.match(container.textContent ?? "", /Projects/, "initial root should render folder entries");

  const projectsButton = [...container.querySelectorAll<HTMLButtonElement>(".file-manager-item--folder")].find((button) => button.textContent?.includes("Projects"));
  assert.ok(projectsButton, "folder entries should be clickable");
  await click(projectsButton);
  await settle();
  assert.match(container.textContent ?? "", /Readme\.md/, "clicking a folder should expand and render nested contents");

  const expandedProjectsButton = [...container.querySelectorAll<HTMLButtonElement>(".file-manager-item--folder")].find((button) => button.textContent?.includes("Projects"));
  assert.ok(expandedProjectsButton, "expanded Projects folder row should remain clickable");
  await click(expandedProjectsButton);
  await settle();
  assert.doesNotMatch(container.textContent ?? "", /Readme\.md/, "clicking an expanded folder should collapse nested contents");

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

  const css = await import("node:fs").then(({ readFileSync }) => readFileSync(new URL("../App.css", import.meta.url), "utf8"));
  assert.ok(css.includes(".chat-workspace--file-manager-open .file-manager-sidebar { grid-column: 1; grid-row: 3;"), "narrow layout should place the file manager in the real single-column grid instead of implicit column 3");

  await act(async () => root.unmount());
  clearMocks();
}

await runTests();
