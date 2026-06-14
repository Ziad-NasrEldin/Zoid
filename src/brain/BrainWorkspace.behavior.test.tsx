import { strict as assert } from "node:assert";
import { Window } from "happy-dom";
import { flushSync } from "react-dom";
import { createRoot, type Root } from "react-dom/client";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { BrainWorkspace } from "./BrainWorkspace";
import type { AppleNotesSource, BrainClarificationSession, BrainNote, BrainStore, TaskCandidate } from "./types";

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
  requestAnimationFrame: window.requestAnimationFrame.bind(window),
  cancelAnimationFrame: window.cancelAnimationFrame.bind(window),
});


async function act(callback: () => void | Promise<void>) {
  let result: void | Promise<void> = undefined;
  flushSync(() => {
    result = callback();
  });
  await result;
  await Promise.resolve();
  await new Promise((resolve) => setTimeout(resolve, 0));
  flushSync(() => undefined);
}
const baseSource: AppleNotesSource = {
  id: "apple-notes:icloud:Zoid Brain",
  sourceType: "appleNotes",
  accountName: "iCloud",
  folderName: "Zoid Brain",
  syncMode: "readOnly",
  enabled: true,
  createdByZoid: true,
  lastSyncedAt: "2026-06-08T10:00:00.000Z",
  lastError: null,
};

function note(id: string, title: string, syncStatus: BrainNote["syncStatus"]): BrainNote {
  return {
    id,
    sourceType: "appleNotes",
    sourceId: baseSource.id,
    appleNoteId: `apple-${id}`,
    title,
    body: "1. Draft task",
    sourceFolder: "Zoid Brain",
    accountName: "iCloud",
    appleCreatedAt: null,
    appleModifiedAt: "2026-06-08T10:00:00.000Z",
    zoidModifiedAt: null,
    importedAt: "2026-06-08T10:00:00.000Z",
    lastSyncedAt: "2026-06-08T10:00:00.000Z",
    lastSyncedTitle: title,
    lastSyncedBody: "1. Draft task",
    lastSyncedHash: `hash-${id}`,
    currentHash: `hash-${id}`,
    syncStatus,
    archived: false,
  };
}

function candidate(id: string, noteId: string, title: string): TaskCandidate {
  return {
    id,
    noteId,
    title,
    extractedDescription: title,
    status: "needsClarification",
    priorityGuess: "normal",
    readinessScore: 0.4,
    clarificationSessionId: null,
    createdAt: "2026-06-08T10:00:00.000Z",
    updatedAt: "2026-06-08T10:00:00.000Z",
  };
}

function store(overrides: Partial<BrainStore> = {}): BrainStore {
  return {
    version: 1,
    sources: [baseSource],
    notes: [note("synced", "Synced note", "synced"), note("conflict", "Conflict note", "conflict")],
    extractions: [],
    taskCandidates: [candidate("candidate-1", "synced", "Draft launch memo")],
    clarificationSessions: [],
    conflicts: [{ id: "conflict-1", noteId: "conflict", appleTitle: "Apple", appleBody: "Apple body", zoidTitle: "Zoid", zoidBody: "Zoid body", detectedAt: "2026-06-08T10:00:00.000Z", resolvedAt: null, resolution: null }],
    updatedAt: "2026-06-08T10:00:00.000Z",
    ...overrides,
  };
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

async function renderBrainWorkspace(): Promise<{ container: HTMLDivElement; root: Root; calls: Array<{ cmd: string; args: unknown }> }> {
  const calls: Array<{ cmd: string; args: unknown }> = [];
  let currentStore = store();
  const openClarifyingSession: BrainClarificationSession = { id: "clarify-1", noteId: "synced", taskCandidateIds: ["candidate-1"], status: "questioning", transcript: [], resolvedBrief: "", openQuestions: ["What outcome should Zoid prepare?"], hermesSessionId: null, createdAt: "2026-06-08T10:00:00.000Z", updatedAt: "2026-06-08T10:00:00.000Z" };
  mockIPC((cmd, args) => {
    calls.push({ cmd, args });
    if (cmd === "load_brain_store") return currentStore;
    if (cmd === "list_apple_notes_folders") return [{ accountName: "iCloud", folderName: "Projects", id: "folder-1" }];
    if (cmd === "link_apple_notes_folder") {
      const nextSource = { ...baseSource, folderName: "Projects", syncMode: (args as { syncMode: AppleNotesSource["syncMode"] }).syncMode };
      currentStore = store({ sources: [nextSource] });
      return nextSource;
    }
    if (cmd === "sync_apple_notes_sources") {
      currentStore = store({ sources: [{ ...baseSource, lastError: "Apple Notes automation failed for Projects" }] });
      return currentStore;
    }
    if (cmd === "extract_brain_note") return currentStore;
    if (cmd === "create_brain_clarifying_session") {
      currentStore = store({ clarificationSessions: [openClarifyingSession] });
      return currentStore;
    }
    if (cmd === "answer_brain_clarifying_session") {
      currentStore = store({ clarificationSessions: [{ ...openClarifyingSession, status: "briefReady", transcript: [{ role: "assistant", content: "What outcome should Zoid prepare?", createdAt: "2026-06-08T10:00:00.000Z" }, { role: "user", content: "Ship a usable brief", createdAt: "2026-06-08T10:01:00.000Z" }], resolvedBrief: "# Agent Brief\n\nShip a usable brief", openQuestions: [], updatedAt: "2026-06-08T10:01:00.000Z" }] });
      return currentStore;
    }
    throw new Error(`Unexpected command: ${cmd}`);
  });

  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const root = createRoot(container);
  await act(async () => root.render(<BrainWorkspace />));
  await settle();
  return { container, root, calls };
}

function buttonByText(container: HTMLElement, text: string): HTMLButtonElement {
  const button = [...container.querySelectorAll<HTMLButtonElement>("button")].find((item) => item.textContent?.includes(text));
  assert.ok(button, `button should exist: ${text}`);
  return button;
}

async function runTests() {
  const { container, root, calls } = await renderBrainWorkspace();
  assert.ok(container.textContent?.includes("Apple Notes Brain loaded."));
  assert.ok(container.querySelector(".brain-sumi-e"), "Brain workspace should opt into the scoped sumi-e design-system class");
  assert.ok(container.textContent?.includes("Local import · conflict-aware extraction · Hermes waits for your command"), "Brain hero should show product-relevant operational provenance");
  assert.equal(container.querySelector(".brain-design-strip"), null, "self-referential design-system annotation strip should not render in product flow");

  const extractButtons = [...container.querySelectorAll<HTMLButtonElement>("button")].filter((button) => button.textContent?.includes("Extract tasks"));
  assert.equal(extractButtons.length, 2, "both synced and conflict notes should render extract controls");
  assert.equal(extractButtons[0].disabled, true, "conflict note sorts first and must block extraction");
  assert.ok(extractButtons[0].title.includes("Resolve the Apple/local conflict"));
  assert.equal(extractButtons[1].disabled, false, "synced note can be extracted");
  assert.ok(container.querySelector(".brain-link-panel"), "branded Apple Notes link panel should render visibly");
  assert.ok(container.textContent?.includes("Link Apple Notes folder"), "Apple Notes link panel heading should render");
  assert.ok(container.textContent?.includes("safe read/import"), "Apple Notes link panel helper should render");
  assert.ok(container.textContent?.includes("Zoid only reads/imports during sync; no delete or writeback is performed here."), "safe import boundaries should be visible");

  await click(buttonByText(container, "List folders"));
  await settle();
  assert.ok(container.textContent?.includes("Read-only import. Zoid reads Apple Notes into Brain and never writes back."), "read-only sync copy should be explicit");

  const modeTrigger = [...container.querySelectorAll<HTMLButtonElement>(".zoid-dropdown-trigger")].find((button) => button.getAttribute("aria-label") === "Apple Notes sync mode");
  assert.ok(modeTrigger, "sync mode dropdown should render inside branded link panel");
  await click(modeTrigger);
  await settle();
  const metadataOption = [...container.querySelectorAll<HTMLButtonElement>('[role="menuitemradio"]')].find((button) => button.textContent?.includes("Metadata tracking only"));
  assert.ok(metadataOption, "metadata-only sync option should render");
  await click(metadataOption);
  await settle();
  assert.ok(container.textContent?.includes("with no writeback yet"));

  await click(buttonByText(container, "Link selected folder"));
  await settle();
  const linkCall = calls.find((call) => call.cmd === "link_apple_notes_folder");
  assert.deepEqual(linkCall?.args, { accountName: "iCloud", folderName: "Projects", syncMode: "twoWay" });

  await click(buttonByText(container, "Sync now"));
  await settle();
  assert.ok(container.textContent?.includes("Synced with 1 source error"), "partial sync errors should be visible without failing closed");
  assert.ok(container.textContent?.includes("Apple Notes automation failed for Projects"));

  const checkbox = container.querySelector<HTMLInputElement>('input[type="checkbox"]');
  assert.ok(checkbox, "task candidate checkbox should render");
  await click(checkbox);
  await settle();
  await click(buttonByText(container, "Start clarifying questions"));
  await settle();
  assert.ok(calls.some((call) => call.cmd === "create_brain_clarifying_session"), "clarifying session command should be invoked");
  assert.ok(container.textContent?.includes("Current question"), "clarifying session should render the active question form");
  assert.ok(container.textContent?.includes("Nothing is sent to Hermes automatically"), "clarifying copy must make non-execution clear");

  const answerBox = container.querySelector<HTMLTextAreaElement>("textarea");
  assert.ok(answerBox, "clarifying answer textarea should render");
  await act(async () => {
    const setValue = Object.getOwnPropertyDescriptor(window.HTMLTextAreaElement.prototype, "value")?.set;
    setValue?.call(answerBox, "Ship a usable brief");
    const event = document.createEvent("Event");
    event.initEvent("input", true, false);
    answerBox.dispatchEvent(event);
  });
  await click(buttonByText(container, "Save answer"));
  await settle();
  assert.ok(calls.some((call) => call.cmd === "answer_brain_clarifying_session"), "clarifying answer command should be invoked");
  assert.ok(container.textContent?.includes("Agent brief"), "answered sessions should render a generated brief");

  await act(async () => root.unmount());
  clearMocks();
}

await runTests();
