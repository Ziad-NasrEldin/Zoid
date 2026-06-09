import { strict as assert } from "node:assert";
import { Window } from "happy-dom";
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { CommandPalette } from "./CommandPalette";
import type { HermesSlashCommand } from "./hermesCommands";

const window = new Window({ url: "http://127.0.0.1:1420" }) as unknown as Window & typeof globalThis;
const document = window.document as Document;

Object.assign(globalThis, {
  IS_REACT_ACT_ENVIRONMENT: true,
  window,
  document,
  HTMLElement: window.HTMLElement,
  HTMLButtonElement: window.HTMLButtonElement,
  HTMLInputElement: window.HTMLInputElement,
  Node: window.Node,
  MouseEvent: window.MouseEvent,
  KeyboardEvent: window.KeyboardEvent,
  requestAnimationFrame: window.requestAnimationFrame.bind(window),
});

const HTMLElementConstructor = (window as any).HTMLElement as { prototype: { scrollIntoView?: () => void } };
const EventConstructor = (window as any).Event as new (type: string, init?: EventInit) => Event;
if (!HTMLElementConstructor.prototype.scrollIntoView) {
  HTMLElementConstructor.prototype.scrollIntoView = () => undefined;
}

const commands: HermesSlashCommand[] = [
  { name: "queue", aliases: ["q"], description: "Queue a prompt", category: "Session", argsHint: "<prompt>", subcommands: [], cliOnly: false, gatewayOnly: false, zoidBehavior: "forward" },
  { name: "steer", aliases: [], description: "Steer the running turn", category: "Session", argsHint: "<prompt>", subcommands: [], cliOnly: false, gatewayOnly: false, zoidBehavior: "forward" },
  { name: "usage", aliases: [], description: "Show usage", category: "Info", argsHint: "", subcommands: [], cliOnly: false, gatewayOnly: false, zoidBehavior: "native-panel", panel: "usage" },
];

async function keydown(element: Element, key: string, options: KeyboardEventInit = {}) {
  await act(async () => {
    element.dispatchEvent(new window.KeyboardEvent("keydown", { key, bubbles: true, cancelable: true, ...options }) as unknown as Event);
  });
}

async function input(element: HTMLInputElement, value: string) {
  await act(async () => {
    element.value = value;
    element.dispatchEvent(new EventConstructor("input", { bubbles: true }));
  });
}

async function settle() {
  await act(async () => {
    await Promise.resolve();
  });
}

type RenderResult = {
  container: HTMLDivElement;
  root: Root;
  inserted: string[];
  ran: string[];
  closed: number;
};

async function renderPalette(): Promise<RenderResult> {
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const root = createRoot(container);
  const inserted: string[] = [];
  const ran: string[] = [];
  let closed = 0;

  await act(async () => {
    root.render(
      <CommandPalette
        commands={commands}
        onClose={() => { closed += 1; }}
        onInsertCommand={(command) => inserted.push(command)}
        onRunCommand={(command) => ran.push(command)}
        open
        recentCommands={[]}
      />,
    );
  });

  return { container, root, inserted, ran, get closed() { return closed; } } as RenderResult;
}

async function runTests() {
  const render = await renderPalette();
  const inputEl = render.container.querySelector<HTMLInputElement>('input[aria-label="Search commands"]');
  assert.ok(inputEl, "command palette search should render");
  assert.equal(inputEl.getAttribute("role"), "combobox", "search input should expose combobox semantics for active option navigation");
  assert.equal(render.container.querySelector('[aria-selected="true"]')?.textContent?.includes("/queue"), true, "first command should start highlighted");

  await keydown(inputEl, "ArrowDown");
  await settle();
  assert.equal(render.container.querySelector('[aria-selected="true"]')?.textContent?.includes("/steer"), true, "ArrowDown should move the highlighted command");

  await keydown(inputEl, "ArrowUp");
  await settle();
  assert.equal(render.container.querySelector('[aria-selected="true"]')?.textContent?.includes("/queue"), true, "ArrowUp should move the highlighted command back");

  await keydown(inputEl, "ArrowUp");
  await settle();
  assert.equal(render.container.querySelector('[aria-selected="true"]')?.textContent?.includes("/usage"), true, "ArrowUp on the first command should wrap to the last command");

  await input(inputEl, "steer");
  await settle();
  assert.equal(render.container.querySelector('[aria-selected="true"]')?.textContent?.includes("/steer"), true, "filtering should reset the highlight to the first matching command");

  await keydown(inputEl, "Enter");
  await settle();
  assert.deepEqual(render.inserted, ["/steer "], "Enter should insert commands that need args");

  await keydown(inputEl, "Enter", { metaKey: true });
  await settle();
  assert.deepEqual(render.ran, ["/steer"], "Cmd/Ctrl+Enter should run the highlighted command immediately");

  await keydown(inputEl, "Escape");
  await settle();
  assert.equal(render.closed, 1, "Escape should close the palette");

  await act(async () => render.root.unmount());
}

await runTests();
