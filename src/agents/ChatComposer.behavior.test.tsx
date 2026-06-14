import { strict as assert } from "node:assert";
import { Window } from "happy-dom";
import { flushSync } from "react-dom";
import type { ComponentProps } from "react";
import { createRoot, type Root } from "react-dom/client";
import { ChatComposer, getInlineSlashSearch, shouldStopHermesFromCopyShortcut } from "./ChatComposer";
import type { HermesSlashCommand } from "./hermesCommands";
import { commandDisplayDescription, fallbackHermesSlashCommands, sortSlashCommandsForSearch } from "./hermesCommands";

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

assert.equal(getInlineSlashSearch("/"), "", "bare slash should open full command completion");
assert.equal(getInlineSlashSearch("/que"), "que", "typing the command token should filter completion");
assert.equal(getInlineSlashSearch("/QUEUE"), "queue", "command token search should be case-insensitive");
assert.equal(getInlineSlashSearch("/queue "), null, "completion should close after inserting a command that needs arguments");
assert.equal(getInlineSlashSearch("/queue hello"), null, "completion should stay closed while typing arguments so Enter can send");
assert.equal(getInlineSlashSearch("/queue\nhello"), null, "completion should not open for multiline drafts");
assert.equal(getInlineSlashSearch("hello /queue"), null, "completion should only open when slash starts the draft");
assert.equal(shouldStopHermesFromCopyShortcut(true, "c", false, true, 0, 0), true, "Ctrl+C with no active selection should stop the active Hermes run");
assert.equal(shouldStopHermesFromCopyShortcut(true, "c", false, true, 0, 4), false, "Ctrl+C with a selection should preserve normal copy behavior");
assert.equal(shouldStopHermesFromCopyShortcut(false, "c", false, true, 0, 0), false, "Ctrl+C should not stop when Hermes is idle");

const rankingCommands: HermesSlashCommand[] = [
  { name: "deploy", aliases: [], description: "Deployment plan helper", category: "ops", subcommands: [], cliOnly: false, gatewayOnly: false, zoidBehavior: "forward" },
  { name: "plan", aliases: ["p"], description: "Prepare an implementation plan", category: "core", argsHint: "<request>", subcommands: [], cliOnly: false, gatewayOnly: false, zoidBehavior: "forward" },
  { name: "profile", aliases: [], description: "Profile settings", category: "core", subcommands: [], cliOnly: false, gatewayOnly: false, zoidBehavior: "native-panel" },
];
assert.equal(sortSlashCommandsForSearch(rankingCommands, "pl")[0]?.name, "plan", "prefix command matches should outrank description contains matches");
assert.equal(commandDisplayDescription(rankingCommands[1]), "Draft a plan before Zoid/Hermes acts.", "core commands should use Zoid-polished visible descriptions");
assert.ok(fallbackHermesSlashCommands.some((command) => command.name === "plan" && command.argsHint), "fallback registry should keep /plan discoverable with an argument hint");

const window = new Window({ url: "http://127.0.0.1:1420" }) as unknown as Window & typeof globalThis;
const document = window.document as Document;

Object.assign(globalThis, {
  IS_REACT_ACT_ENVIRONMENT: true,
  window,
  document,
  HTMLElement: window.HTMLElement,
  HTMLButtonElement: window.HTMLButtonElement,
  HTMLTextAreaElement: window.HTMLTextAreaElement,
  HTMLInputElement: window.HTMLInputElement,
  Node: window.Node,
  KeyboardEvent: window.KeyboardEvent,
  MouseEvent: window.MouseEvent,
  Event: window.Event,
  CSS: window.CSS,
  requestAnimationFrame: window.requestAnimationFrame.bind(window),
});

const HTMLElementConstructor = (window as any).HTMLElement as { prototype: { scrollIntoView?: () => void } };
const EventConstructor = (window as any).Event as new (type: string, init?: EventInit) => Event;
if (!HTMLElementConstructor.prototype.scrollIntoView) {
  HTMLElementConstructor.prototype.scrollIntoView = () => undefined;
}

async function settle() {
  await act(async () => {
    await Promise.resolve();
  });
}

async function inputTextarea(element: HTMLTextAreaElement, value: string) {
  await act(async () => {
    element.value = value;
    element.dispatchEvent(new EventConstructor("input", { bubbles: true }));
    element.dispatchEvent(new EventConstructor("change", { bubbles: true }));
  });
}

type RenderResult = {
  container: HTMLDivElement;
  root: Root;
  sent: string[];
};

async function renderComposer(isSending: boolean, props: Partial<ComponentProps<typeof ChatComposer>> = {}): Promise<RenderResult> {
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const root = createRoot(container);
  const sent: string[] = [];

  await act(async () => {
    root.render(
      <ChatComposer
        isSending={isSending}
        onSend={(message) => { sent.push(message); }}
        onStop={() => undefined}
        {...props}
      />,
    );
  });

  return { container, root, sent };
}

async function runComponentTests() {
  const busyRender = await renderComposer(true);
  const textarea = busyRender.container.querySelector<HTMLTextAreaElement>("textarea");
  assert.ok(textarea, "composer textarea should render while Hermes is busy");
  assert.equal(textarea.disabled, false, "busy composer should stay editable");

  await inputTextarea(textarea, "continue after this");
  await settle();
  const sendButton = busyRender.container.querySelector<HTMLButtonElement>("button.composer-send");
  assert.equal(sendButton?.textContent, "QUEUE", "busy send button should become Queue when a draft exists");

  assert.ok(sendButton, "busy queue button should render");
  const form = busyRender.container.querySelector<HTMLFormElement>("form");
  assert.ok(form, "composer form should render");
  await act(async () => {
    form.dispatchEvent(new EventConstructor("submit", { bubbles: true, cancelable: true }));
  });
  await settle();
  assert.deepEqual(busyRender.sent, ["continue after this"], "submitting while busy should queue/send the draft instead of locking the composer");

  await act(async () => busyRender.root.unmount());

  const queueOnlyPanel = await renderComposer(true, { canStop: false, variant: "panel", inputLabel: "Message panel" });
  const queueOnlyButton = queueOnlyPanel.container.querySelector<HTMLButtonElement>("button.composer-send");
  const queueOnlyTextarea = queueOnlyPanel.container.querySelector<HTMLTextAreaElement>("textarea");
  assert.ok(queueOnlyButton, "queue-only panel button should render");
  assert.ok(queueOnlyTextarea, "queue-only panel textarea should render");
  assert.equal(queueOnlyButton.textContent, "WAIT", "queue-only panel with no draft must not show a fake STOP button");
  assert.equal(queueOnlyButton.disabled, true, "queue-only panel with no draft should not submit or stop anything");
  assert.equal(queueOnlyButton.classList.contains("composer-send--stop"), false, "queue-only panel should not get stop styling without an owning run");
  assert.equal(queueOnlyPanel.container.querySelector(".composer-input-label-row"), null, "panel composer should hide the visible full-size label row to preserve compact layout");
  assert.equal(queueOnlyTextarea.style.height, "34px", "panel composer should use compact 34px autosize instead of full 44px height");
  await inputTextarea(queueOnlyTextarea, "queue this panel");
  await settle();
  assert.equal(queueOnlyButton.textContent, "QUEUE", "queue-only panel with a draft should queue instead of stopping");
  assert.equal(queueOnlyButton.disabled, false, "queue-only panel with a draft should be submittable for queueing");
  await act(async () => queueOnlyPanel.root.unmount());

  const multiContainer = document.createElement("div");
  document.body.replaceChildren(multiContainer);
  const multiRoot = createRoot(multiContainer);
  const slashCommands: HermesSlashCommand[] = [
    { name: "help", aliases: [], description: "Show help", category: "core", subcommands: [], cliOnly: false, gatewayOnly: false, zoidBehavior: "forward" },
    { name: "new", aliases: ["reset"], description: "Start a new session", category: "core", argsHint: "[name]", subcommands: [], cliOnly: false, gatewayOnly: false, zoidBehavior: "forward" },
    { name: "tools", aliases: [], description: "List tools", category: "core", subcommands: [], cliOnly: false, gatewayOnly: false, zoidBehavior: "forward" },
    { name: "plan", aliases: ["p"], description: "Prepare an implementation plan", category: "core", argsHint: "<request>", subcommands: [], cliOnly: false, gatewayOnly: false, zoidBehavior: "forward" },
  ];
  await act(async () => {
    multiRoot.render(
      <>
        <ChatComposer ariaLabel="Composer A" inputLabel="Message A" slashCommands={slashCommands} onSend={() => undefined} />
        <ChatComposer ariaLabel="Composer B" inputLabel="Message B" slashCommands={slashCommands} onSend={() => undefined} />
      </>,
    );
  });
  const textareas = [...multiContainer.querySelectorAll<HTMLTextAreaElement>("textarea")];
  assert.equal(textareas.length, 2, "multiple composers should render at once for dashboard panels");
  await inputTextarea(textareas[0], "/");
  await inputTextarea(textareas[1], "/");
  await settle();
  const ids = [...multiContainer.querySelectorAll<HTMLElement>("[id]")].map((element) => element.id);
  assert.equal(new Set(ids).size, ids.length, "multiple composers must not duplicate slash/status DOM ids");
  for (const multiTextarea of textareas) {
    const controls = multiTextarea.getAttribute("aria-controls");
    const activeDescendant = multiTextarea.getAttribute("aria-activedescendant");
    assert.ok(controls && multiContainer.querySelector(`#${CSS.escape(controls)}`), "textarea aria-controls should point at its own slash command list");
    assert.ok(activeDescendant && multiContainer.querySelector(`#${CSS.escape(activeDescendant)}`), "textarea aria-activedescendant should point at its own active slash option");
  }

  await inputTextarea(textareas[0], "");
  await inputTextarea(textareas[1], "");
  await settle();

  const attachButtons = [...multiContainer.querySelectorAll<HTMLButtonElement>("button.composer-attach")];
  assert.equal(attachButtons.length, 2, "multiple composers should render independent action buttons");
  await act(async () => {
    attachButtons[0].click();
  });
  assert.equal(multiContainer.querySelectorAll(".composer-action-popover").length, 1, "first composer plus menu should open");
  await act(async () => {
    textareas[1].dispatchEvent(new EventConstructor("pointerdown", { bubbles: true }));
    textareas[1].focus();
  });
  await settle();
  assert.equal(multiContainer.querySelectorAll(".composer-action-popover").length, 0, "clicking another composer should close the old plus menu");

  await inputTextarea(textareas[0], "/");
  await settle();
  assert.equal(multiContainer.querySelectorAll(".composer-slash-dropup").length, 1, "first composer slash dropup should open from slash draft");
  await act(async () => {
    textareas[1].dispatchEvent(new EventConstructor("pointerdown", { bubbles: true }));
    textareas[1].focus();
  });
  await settle();
  assert.equal(multiContainer.querySelectorAll(".composer-slash-dropup").length, 0, "clicking another composer should close the old slash dropup without clearing the draft");
  assert.equal(textareas[0].value, "/", "closing slash completion from another composer should preserve the draft");

  await inputTextarea(textareas[0], "/he");
  await settle();
  assert.equal(multiContainer.querySelectorAll(".composer-slash-dropup").length, 1, "editing the slash draft should reopen completion after it was dismissed");
  await act(async () => {
    textareas[0].dispatchEvent(new KeyboardEvent("keydown", { key: "Escape", bubbles: true }));
  });
  await settle();
  assert.equal(multiContainer.querySelectorAll(".composer-slash-dropup").length, 0, "Escape should close slash completion");
  assert.equal(textareas[0].value, "/he", "Escape should not destroy the slash draft");

  await inputTextarea(textareas[0], "/to");
  await settle();
  await act(async () => {
    textareas[0].dispatchEvent(new KeyboardEvent("keydown", { key: "Tab", bubbles: true }));
  });
  await settle();
  assert.equal(textareas[0].value, "/tools", "selecting an inline slash command should replace the slash draft instead of appending to it");

  await inputTextarea(textareas[0], "/pl");
  await settle();
  assert.equal(multiContainer.querySelector(".composer-slash-dropup-option strong")?.textContent?.trim(), "/plan <request>", "/plan should be the first match for /pl");
  await act(async () => {
    textareas[0].dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
  });
  await settle();
  assert.equal(textareas[0].value, "/plan ", "inline /plan completion should insert a trailing space for the planning request");
  assert.equal(multiContainer.querySelectorAll(".composer-slash-dropup").length, 0, "inline completion should close after inserting /plan");

  const sentNewSession: string[] = [];
  await act(async () => {
    multiRoot.render(<ChatComposer ariaLabel="New session composer" slashCommands={slashCommands} onSend={(message) => { sentNewSession.push(message); }} />);
  });
  const newSessionTextarea = multiContainer.querySelector<HTMLTextAreaElement>("textarea");
  assert.ok(newSessionTextarea, "new-session composer should render");
  await inputTextarea(newSessionTextarea, "/new");
  await settle();
  await act(async () => {
    newSessionTextarea.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
  });
  await settle();
  assert.deepEqual(sentNewSession, ["/new"], "exact no-argument slash commands such as /new should execute on Enter instead of only inserting autocomplete text");

  const sentFromPlan: string[] = [];
  await act(async () => {
    multiRoot.render(<ChatComposer ariaLabel="Plan composer" slashCommands={slashCommands} onSend={(message) => { sentFromPlan.push(message); }} />);
  });
  const planTextarea = multiContainer.querySelector<HTMLTextAreaElement>("textarea");
  assert.ok(planTextarea, "plan composer should render");
  await inputTextarea(planTextarea, "/pl");
  await settle();
  await act(async () => {
    planTextarea.dispatchEvent(new KeyboardEvent("keydown", { key: "Tab", bubbles: true }));
  });
  await settle();
  assert.deepEqual(sentFromPlan, [], "inline /plan completion must insert only and never auto-send");
  await inputTextarea(planTextarea, "/plan add autocomplete");
  await act(async () => {
    planTextarea.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
  });
  await settle();
  assert.deepEqual(sentFromPlan, ["/plan add autocomplete"], "completed /plan draft should send normally when the user presses Enter after writing args");

  const sentUnknown: string[] = [];
  await act(async () => {
    multiRoot.render(<ChatComposer ariaLabel="Unknown composer" slashCommands={slashCommands} onSend={(message) => { sentUnknown.push(message); }} />);
  });
  const unknownTextarea = multiContainer.querySelector<HTMLTextAreaElement>("textarea");
  assert.ok(unknownTextarea, "unknown composer should render");
  await inputTextarea(unknownTextarea, "/xyz");
  await settle();
  assert.equal(multiContainer.querySelectorAll(".composer-slash-dropup").length, 1, "unknown slash drafts should keep the autocomplete surface open");
  assert.equal(multiContainer.querySelector(".composer-slash-empty")?.textContent?.includes("No command found"), true, "unknown slash drafts should show a no-match row");
  await act(async () => {
    unknownTextarea.dispatchEvent(new KeyboardEvent("keydown", { key: "Enter", bubbles: true }));
  });
  await settle();
  assert.deepEqual(sentUnknown, ["/xyz"], "Enter should send an unknown slash draft even while the no-match row is visible");

  await inputTextarea(unknownTextarea, "/xyz");
  await settle();
  await act(async () => {
    unknownTextarea.dispatchEvent(new KeyboardEvent("keydown", { key: "Tab", bubbles: true, cancelable: true }));
  });
  await settle();
  assert.equal(unknownTextarea.value, "/xyz", "Tab should not mutate unknown slash drafts with no matches");
  await act(async () => multiRoot.unmount());
}

await runComponentTests();
