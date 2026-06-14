import { strict as assert } from "node:assert";
import { Window } from "happy-dom";
import { useState } from "react";
import { flushSync } from "react-dom";
import { createRoot, type Root } from "react-dom/client";
import { GlobalDropdown } from "./GlobalDropdown";

const window = new Window({ url: "http://127.0.0.1:1420" }) as unknown as Window & typeof globalThis;
const document = window.document as Document;

Object.assign(globalThis, {
  IS_REACT_ACT_ENVIRONMENT: true,
  window,
  document,
  HTMLElement: window.HTMLElement,
  HTMLButtonElement: window.HTMLButtonElement,
  Node: window.Node,
  PointerEvent: window.PointerEvent,
  MouseEvent: window.MouseEvent,
  KeyboardEvent: window.KeyboardEvent,
  requestAnimationFrame: window.requestAnimationFrame.bind(window),
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
async function click(element: Element) {
  await act(async () => {
    element.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true }) as unknown as Event);
  });
}

async function keydown(element: Element, key: string) {
  await act(async () => {
    element.dispatchEvent(new window.KeyboardEvent("keydown", { key, bubbles: true, cancelable: true }) as unknown as Event);
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
  changes: string[];
};

async function renderDropdown({ disabled = false }: { disabled?: boolean } = {}): Promise<RenderResult> {
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const root = createRoot(container);
  const changes: string[] = [];

  function Harness() {
    const [value, setValue] = useState("safe");
    return (
      <GlobalDropdown
        disabled={disabled}
        label="Access mode"
        onChange={(nextValue) => {
          changes.push(nextValue);
          setValue(nextValue);
        }}
        options={[
          { value: "safe", label: "Safe/read-only" },
          { value: "workspace", label: "Workspace write" },
          { value: "full", label: "Full access" },
          { value: "blocked", label: "Blocked", disabled: true },
        ]}
        value={value}
      />
    );
  }

  await act(async () => {
    root.render(<Harness />);
  });
  return { container, root, changes };
}

async function runTests() {
  const { container, root, changes } = await renderDropdown();
  const trigger = container.querySelector<HTMLButtonElement>(".zoid-dropdown-trigger");
  assert.ok(trigger, "dropdown trigger should render");
  assert.equal(trigger.getAttribute("aria-label"), "Access mode");
  assert.equal(trigger.getAttribute("aria-haspopup"), "menu");
  assert.equal(trigger.getAttribute("aria-expanded"), "false");

  await click(trigger);
  await settle();
  assert.equal(trigger.getAttribute("aria-expanded"), "true", "click should open the dropdown");
  assert.ok(container.querySelector('[role="menu"]'), "open dropdown should render a menu");
  assert.equal(container.querySelectorAll('[role="menuitemradio"]').length, 4, "menu should render all radio options");

  const workspaceOption = [...container.querySelectorAll<HTMLButtonElement>('[role="menuitemradio"]')].find((option) => option.textContent?.includes("Workspace write"));
  assert.ok(workspaceOption, "workspace option should exist");
  await click(workspaceOption);
  await settle();
  assert.deepEqual(changes, ["workspace"], "clicking an option should call onChange");
  assert.equal(container.querySelector('[role="menu"]'), null, "selecting an option should close the menu");

  await keydown(trigger, "Enter");
  await settle();
  assert.ok(container.querySelector('[role="menu"]'), "Enter on trigger should open menu");
  await keydown(document.activeElement ?? trigger, "Escape");
  await settle();
  assert.equal(container.querySelector('[role="menu"]'), null, "Escape should close menu");

  await keydown(trigger, "ArrowDown");
  await settle();
  assert.ok(container.querySelector('[role="menu"]'), "ArrowDown should open menu");
  const focusedBefore = document.activeElement?.textContent ?? "";
  await keydown(document.activeElement ?? trigger, "ArrowDown");
  await settle();
  assert.notEqual(document.activeElement?.textContent ?? "", focusedBefore, "ArrowDown in menu should move focus");

  const blockedOption = [...container.querySelectorAll<HTMLButtonElement>('[role="menuitemradio"]')].find((option) => option.textContent?.includes("Blocked"));
  assert.ok(blockedOption?.disabled, "disabled options should not be selectable");
  await click(blockedOption);
  await settle();
  assert.deepEqual(changes, ["workspace"], "disabled option should not call onChange");
  await act(async () => root.unmount());

  const disabledRender = await renderDropdown({ disabled: true });
  const disabledTrigger = disabledRender.container.querySelector<HTMLButtonElement>(".zoid-dropdown-trigger");
  assert.equal(disabledTrigger?.disabled, true, "disabled dropdown should disable its trigger");
  if (disabledTrigger) await click(disabledTrigger);
  await settle();
  assert.equal(disabledRender.container.querySelector('[role="menu"]'), null, "disabled dropdown should not open");
  await act(async () => disabledRender.root.unmount());
}

await runTests();
