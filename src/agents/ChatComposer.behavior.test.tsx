import { strict as assert } from "node:assert";
import { Window } from "happy-dom";
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { ChatComposer, getInlineSlashSearch, shouldStopHermesFromCopyShortcut } from "./ChatComposer";

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
  Event: window.Event,
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

async function renderComposer(isSending: boolean): Promise<RenderResult> {
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
}

await runComponentTests();
