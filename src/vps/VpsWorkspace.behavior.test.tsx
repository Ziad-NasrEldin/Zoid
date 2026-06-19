import { strict as assert } from "node:assert";
import { Window } from "happy-dom";
import { flushSync } from "react-dom";
import { createRoot, type Root } from "react-dom/client";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { VpsWorkspace } from "./VpsWorkspace";
import type { HostingerVpsOverview } from "./types";

const window = new Window({ url: "http://127.0.0.1:1420" }) as unknown as Window & typeof globalThis;
const document = window.document as Document;

Object.assign(globalThis, {
  IS_REACT_ACT_ENVIRONMENT: true,
  window,
  document,
  HTMLElement: window.HTMLElement,
  HTMLButtonElement: window.HTMLButtonElement,
  Node: window.Node,
  MouseEvent: window.MouseEvent,
  Event: window.Event,
  requestAnimationFrame: window.requestAnimationFrame.bind(window),
  cancelAnimationFrame: window.cancelAnimationFrame.bind(window),
});

function overview(overrides: Partial<HostingerVpsOverview> = {}): HostingerVpsOverview {
  return {
    tokenPresent: true,
    servers: [
      {
        id: "123456",
        hostname: "mavoid-prod-vps",
        state: "running",
        plan: "KVM 2",
        primaryIp: "203.0.113.10",
        location: "eu-west",
        actionsLock: "unlocked",
        cpus: 2,
        memoryMb: 8192,
        diskGb: 100,
      },
      {
        id: "locked-vps",
        hostname: "maintenance-vps",
        state: "running",
        plan: "KVM 1",
        primaryIp: null,
        location: null,
        actionsLock: "locked",
        cpus: 1,
        memoryMb: 4096,
        diskGb: 50,
      },
    ],
    actions: [
      {
        id: "restart-123456-1",
        virtualMachineId: "123456",
        action: "restart",
        state: "accepted",
        createdAt: "2026-06-19T02:00:00Z",
        providerActionId: "provider-action-789",
        message: "Hostinger accepted restart for VPS 123456 (accepted).",
      },
    ],
    lastSyncedAt: "2026-06-19T02:00:00Z",
    lastError: null,
    cachePath: "/Users/example/.hermes/zoid-vps.json",
    updatedAt: "2026-06-19T02:00:01Z",
    ...overrides,
  };
}

async function settle() {
  await Promise.resolve();
  await Promise.resolve();
  await new Promise((resolve) => setTimeout(resolve, 0));
}

async function act(callback: () => unknown | Promise<unknown>) {
  let result: unknown | Promise<unknown>;
  flushSync(() => {
    result = callback();
  });
  await result;
  await settle();
}

async function click(element: Element) {
  await act(async () => {
    element.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true }) as unknown as Event);
  });
}

async function renderWorkspace(initialOverview = overview()): Promise<{ container: HTMLDivElement; root: Root; calls: Array<{ cmd: string; args: unknown }> }> {
  const calls: Array<{ cmd: string; args: unknown }> = [];
  let currentOverview = initialOverview;
  mockIPC((cmd, args) => {
    calls.push({ cmd, args });
    if (cmd === "hostinger_vps_get_overview") return currentOverview;
    if (cmd === "hostinger_vps_refresh") {
      currentOverview = overview({ lastSyncedAt: "2026-06-19T02:10:00Z" });
      return currentOverview;
    }
    if (cmd === "hostinger_vps_run_action") {
      currentOverview = overview({
        actions: [
          {
            id: "stop-123456-2",
            virtualMachineId: "123456",
            action: "stop",
            state: "accepted",
            createdAt: "2026-06-19T02:11:00Z",
            providerActionId: "provider-action-999",
            message: "Hostinger accepted stop for VPS 123456 (accepted).",
          },
          ...currentOverview.actions,
        ],
      });
      return { ok: true, message: "Hostinger accepted stop for VPS 123456 (accepted).", action: currentOverview.actions[0], overview: currentOverview };
    }
    throw new Error(`Unexpected command: ${cmd}`);
  });
  window.confirm = (() => true) as typeof window.confirm;
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const root = createRoot(container);
  await act(async () => root.render(<VpsWorkspace />));
  await settle();
  return { container, root, calls };
}

async function runTests() {
  const rendered = await renderWorkspace();

  assert.ok(rendered.container.querySelector(".vps-workspace"), "VPS workspace should render");
  assert.ok(rendered.calls.some((call) => call.cmd === "hostinger_vps_get_overview"), "workspace should load cached Hostinger overview through the Tauri backend");
  assert.ok(rendered.container.textContent?.includes("HOSTINGER_API_TOKEN present"), "connection status should reflect backend token presence without rendering the token value");
  assert.ok(!rendered.container.textContent?.includes("64ePEiE9"), "visible UI must never render the Hostinger token");
  assert.ok(rendered.container.textContent?.includes("mavoid-prod-vps"), "server hostname should render");
  assert.ok(rendered.container.textContent?.includes("203.0.113.10"), "server primary IP should render");
  assert.ok(rendered.container.textContent?.includes("KVM 2"), "server plan should render");
  assert.ok(rendered.container.textContent?.includes("2 vCPU"), "server CPU should render");
  assert.ok(rendered.container.textContent?.includes("8192 MB"), "server memory should render");
  assert.ok(rendered.container.textContent?.includes("100 GB"), "server disk should render");
  assert.ok(rendered.container.textContent?.includes("Action lock: unlocked"), "server action-lock backend field should be exposed in the UI");
  assert.ok(rendered.container.textContent?.includes("Provider action: provider-action-789"), "provider action id should be exposed for auditability");
  assert.ok(rendered.container.textContent?.includes("Hostinger accepted restart for VPS 123456"), "backend action message should be exposed in the action log");

  const lockedCard = [...rendered.container.querySelectorAll(".vps-server-card")].find((card) => card.textContent?.includes("maintenance-vps"));
  assert.ok(lockedCard, "locked server card should render");
  assert.ok([...lockedCard.querySelectorAll<HTMLButtonElement>("button")].every((button) => button.disabled), "locked server actions should be disabled in the UI");

  const refreshButton = [...rendered.container.querySelectorAll<HTMLButtonElement>("button")].find((button) => button.textContent?.includes("Refresh API"));
  assert.ok(refreshButton, "refresh button should render");
  await click(refreshButton);
  assert.ok(rendered.calls.some((call) => call.cmd === "hostinger_vps_refresh"), "refresh button should call Hostinger refresh backend command");

  const stopButton = [...rendered.container.querySelectorAll<HTMLButtonElement>("button")].find((button) => button.textContent?.includes("Stop") && !button.disabled);
  assert.ok(stopButton, "running server should expose enabled stop action");
  await click(stopButton);
  assert.ok(rendered.calls.some((call) => call.cmd === "hostinger_vps_run_action" && (call.args as { virtualMachineId?: string; action?: string }).virtualMachineId === "123456" && (call.args as { action?: string }).action === "stop"), "stop action should call backend with the real server id and action");
  assert.ok(rendered.container.textContent?.includes("Hostinger accepted stop for VPS 123456"), "action response message should render after backend action");

  await act(async () => rendered.root.unmount());
  clearMocks();

  mockIPC(() => {
    throw new TypeError("Cannot read properties of undefined (reading 'invoke')");
  });
  const bridgeContainer = document.createElement("div");
  document.body.replaceChildren(bridgeContainer);
  const bridgeRoot = createRoot(bridgeContainer);
  await act(async () => bridgeRoot.render(<VpsWorkspace />));
  await settle();
  assert.ok(bridgeContainer.textContent?.includes("Zoid desktop bridge is unavailable in this preview"), "browser-preview bridge errors should render as truthful copy");
  assert.ok(!bridgeContainer.textContent?.includes("Cannot read properties of undefined"), "raw invoke errors should not leak into the VPS UI");
  await act(async () => bridgeRoot.unmount());
  clearMocks();
}

runTests().catch((error) => {
  console.error(error);
  process.exit(1);
});
