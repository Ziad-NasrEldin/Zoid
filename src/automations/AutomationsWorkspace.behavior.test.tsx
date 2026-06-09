import { strict as assert } from "node:assert";
import { Window } from "happy-dom";
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { AutomationsWorkspace } from "./AutomationsWorkspace";
import type { AutomationCronJob, AutomationList } from "./types";

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
  PointerEvent: window.PointerEvent,
  MouseEvent: window.MouseEvent,
  KeyboardEvent: window.KeyboardEvent,
  Event: window.Event,
  requestAnimationFrame: window.requestAnimationFrame.bind(window),
  cancelAnimationFrame: window.cancelAnimationFrame.bind(window),
});

function cronJob(overrides: Partial<AutomationCronJob>): AutomationCronJob {
  return {
    jobId: "cron_1",
    name: "Morning executive brief",
    schedule: "0 9 * * *",
    repeat: "forever",
    deliver: "origin",
    nextRunAt: "2026-06-09T09:00:00.000Z",
    lastRunAt: "2026-06-08T09:00:00.000Z",
    lastStatus: "ok",
    lastDeliveryError: null,
    enabled: true,
    state: "enabled",
    pausedAt: null,
    pausedReason: null,
    script: null,
    noAgent: false,
    skills: ["briefing"],
    promptPreview: "Prepare a concise brief",
    enabledToolsets: ["web"],
    protected: false,
    protectionReason: null,
    ...overrides,
  };
}

function automationList(jobs: AutomationCronJob[] = [cronJob({})]): AutomationList {
  return {
    jobs,
    watchers: [{ id: "watcher:calendar", name: "Calendar watcher", state: "running", source: "Hermes", lastSeenAt: "2026-06-08T10:00:00.000Z", lastStatus: "ok", detail: "observed" }],
    watcherSourceStatus: "available",
    count: jobs.length,
    refreshedAt: "2026-06-08T10:00:00.000Z",
    hermesCommand: "hermes cron list --json",
    activeProfile: "default",
  };
}

async function settle() {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
}

async function click(element: Element) {
  await act(async () => {
    element.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true }) as unknown as Event);
  });
}

async function keyDown(element: Element, key: string, shiftKey = false) {
  await act(async () => {
    element.dispatchEvent(new window.KeyboardEvent("keydown", { bubbles: true, cancelable: true, key, shiftKey }) as unknown as Event);
  });
}

async function renderAutomations(currentList = automationList(), options: { keepRemovedJobInReadBack?: boolean } = {}): Promise<{ container: HTMLDivElement; root: Root; calls: Array<{ cmd: string; args: unknown }>; statuses: string[] }> {
  const calls: Array<{ cmd: string; args: unknown }> = [];
  const statuses: string[] = [];
  let list = currentList;

  mockIPC((cmd, args) => {
    calls.push({ cmd, args });
    if (cmd === "list_hermes_automations") return list;
    if (cmd === "manage_hermes_cron_job") {
      const action = (args as { action: string }).action;
      const jobId = (args as { jobId?: string; job_id?: string }).jobId ?? (args as { job_id?: string }).job_id;
      if (action === "remove" && !options.keepRemovedJobInReadBack) list = automationList(list.jobs.filter((job) => job.jobId !== jobId));
      if (action === "pause") list = automationList(list.jobs.map((job) => job.jobId === jobId ? { ...job, enabled: false, state: "paused", lastStatus: "paused" } : job));
      if (action === "resume") list = automationList(list.jobs.map((job) => job.jobId === jobId ? { ...job, enabled: true, state: "enabled", lastStatus: "ok" } : job));
      return list;
    }
    throw new Error(`Unexpected command: ${cmd}`);
  });

  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const root = createRoot(container);
  await act(async () => root.render(<AutomationsWorkspace onStatusChange={(status) => statuses.push(status)} />));
  await settle();
  return { container, root, calls, statuses };
}

function buttonByText(container: HTMLElement, text: string): HTMLButtonElement {
  const button = [...container.querySelectorAll<HTMLButtonElement>("button")].find((item) => item.textContent?.includes(text));
  assert.ok(button, `button should exist: ${text}`);
  return button;
}

async function runTests() {
  const protectedJob = cronJob({ jobId: "cron_protected", name: "System heartbeat", protected: true, protectionReason: "System protected", enabled: true, lastStatus: "ok" });
  const failedScriptJob = cronJob({ jobId: "cron_failed", name: "Script ingestion", lastStatus: "failed", noAgent: true, script: "scripts/import.py", nextRunAt: "2026-06-10T09:00:00.000Z" });
  const { container, root, calls, statuses } = await renderAutomations(automationList([protectedJob, failedScriptJob]));

  assert.ok(container.querySelector(".automation-sumi-e"), "Automations should opt into the scoped Brain-derived design system");
  assert.ok(container.querySelector(".automation-ink-clock"), "Automations hero should have its own clock/ritual mark");
  assert.ok(container.textContent?.includes("Provider-owned schedules · protected system jobs · watcher state is read-only"), "hero should disclose source-of-truth constraints");
  assert.ok(container.textContent?.includes("Blind spots patched"), "edge-case safety panel should render");
  assert.ok(container.textContent?.includes("Protected jobs cannot be removed from the UI."), "protected removal guard should be visible");
  assert.equal(statuses[statuses.length - 1], "blocked", "failed cron job should make nav status blocked");

  const removeButtons = [...container.querySelectorAll<HTMLButtonElement>("button")].filter((button) => button.textContent?.includes("Remove"));
  assert.equal(removeButtons.length, 2, "both jobs should render remove controls");
  assert.equal(removeButtons[0].disabled, true, "protected job remove button must be disabled");
  assert.equal(removeButtons[1].disabled, false, "unprotected job can request removal");

  const scriptTab = buttonByText(container, "Script-only");
  assert.equal(scriptTab.getAttribute("aria-pressed"), "false", "filters should use pressed-button semantics, not tabs");
  await click(scriptTab);
  await settle();
  assert.equal(scriptTab.getAttribute("aria-pressed"), "true", "active filter should expose aria-pressed");
  assert.ok(container.textContent?.includes("1 of 2 shown"), "script filter should narrow visible jobs");
  assert.equal(container.querySelectorAll("[role='tablist'], [role='tab']").length, 0, "filter and section labels should not fake tab semantics");

  removeButtons[1].focus();
  await click(removeButtons[1]);
  await settle();
  assert.ok(container.textContent?.includes("Remove cron job?"), "remove action should open branded confirmation");
  assert.equal(document.activeElement?.textContent, "Cancel", "modal should move focus to the first confirmation control");
  const dialog = container.querySelector<HTMLElement>(".automation-confirm-panel");
  assert.ok(dialog, "dialog should be queryable for keyboard handling");
  await keyDown(dialog, "Tab", true);
  assert.equal(document.activeElement?.textContent, "Remove", "Shift+Tab on first modal control should wrap to last control");
  await keyDown(dialog, "Escape");
  await settle();
  assert.equal(container.querySelector(".automation-confirm-panel"), null, "Escape should close confirmation dialog");
  assert.equal(document.activeElement, removeButtons[1], "closing dialog should restore focus to triggering button");

  await click(removeButtons[1]);
  await settle();
  const confirmRemoveButton = container.querySelector<HTMLButtonElement>(".automation-confirm-actions .automation-danger-button");
  assert.ok(confirmRemoveButton, "confirm remove button should render inside modal");
  await click(confirmRemoveButton);
  await settle();
  assert.ok(calls.some((call) => call.cmd === "manage_hermes_cron_job" && (call.args as { action?: string }).action === "remove"), "remove should call Hermes manager");
  assert.ok(container.textContent?.includes("Removed “Script ingestion”. Hermes provider read-back refreshed."), "remove status should mention provider read-back");
  const remainingJobTitles = [...container.querySelectorAll(".automation-job-card h4")].map((heading) => heading.textContent);
  assert.ok(!remainingJobTitles.includes("Script ingestion"), "removed cron job card should disappear after provider read-back");

  await act(async () => root.unmount());
  clearMocks();

  const readbackFailureJob = cronJob({ jobId: "cron_sticky", name: "Sticky remove", protected: false });
  const secondRender = await renderAutomations(automationList([readbackFailureJob]), { keepRemovedJobInReadBack: true });
  const stickyRemove = [...secondRender.container.querySelectorAll<HTMLButtonElement>("button")].find((button) => button.textContent?.includes("Remove"));
  assert.ok(stickyRemove, "sticky remove button should render");
  await click(stickyRemove);
  await settle();
  const stickyConfirm = secondRender.container.querySelector<HTMLButtonElement>(".automation-confirm-actions .automation-danger-button");
  assert.ok(stickyConfirm, "sticky confirm button should render");
  await click(stickyConfirm);
  await settle();
  assert.ok(secondRender.container.querySelector(".automation-confirm-panel"), "failed read-back should keep modal open");
  assert.ok(secondRender.container.textContent?.includes("Hermes read-back still includes this cron job after remove."), "failed read-back should surface inside the modal");
  await act(async () => secondRender.root.unmount());
  clearMocks();
}

await runTests();
