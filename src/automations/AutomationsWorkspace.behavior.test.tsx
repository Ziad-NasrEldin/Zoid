import { strict as assert } from "node:assert";
import { Window } from "happy-dom";
import { act as reactAct } from "react";
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


async function act(callback: () => void | Promise<void>) {
  await reactAct(async () => {
    await callback();
    await Promise.resolve();
    await new Promise((resolve) => setTimeout(resolve, 0));
  });
}
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

type RenderOptions = {
  keepRemovedJobInReadBack?: boolean;
  manageHandler?: (action: string, jobId: string | undefined, currentList: AutomationList) => AutomationList | Promise<AutomationList>;
};

async function renderAutomations(currentList = automationList(), options: RenderOptions = {}): Promise<{ container: HTMLDivElement; root: Root; calls: Array<{ cmd: string; args: unknown }>; statuses: string[] }> {
  const calls: Array<{ cmd: string; args: unknown }> = [];
  const statuses: string[] = [];
  let list = currentList;

  mockIPC((cmd, args) => {
    calls.push({ cmd, args });
    if (cmd === "list_hermes_automations") return list;
    if (cmd === "manage_hermes_cron_job") {
      const action = (args as { action: string }).action;
      const jobId = (args as { jobId?: string; job_id?: string }).jobId ?? (args as { job_id?: string }).job_id;
      if (options.manageHandler) {
        return Promise.resolve(options.manageHandler(action, jobId, list)).then((nextList) => {
          list = nextList;
          return list;
        });
      }
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

  assert.ok(container.querySelector(".automation-sumi-e"), "Automations should opt into the scoped Brain-derived sumi-e design system");
  assert.ok(container.querySelector(".automation-ink-clock"), "Automations hero should have its own clock/ritual mark");
  assert.ok(container.textContent?.includes("Provider-owned schedules · protected system jobs · watcher state is read-only"), "hero should disclose source-of-truth constraints");
  assert.ok(container.textContent?.includes("Blind spots patched"), "edge-case safety panel should render");
  assert.ok(container.textContent?.includes("Protected jobs cannot be removed from the UI."), "protected removal guard should be visible");
  assert.equal(statuses[statuses.length - 1], "blocked", "failed cron job should make nav status blocked");

  let removeButtons = [...container.querySelectorAll<HTMLButtonElement>("button")].filter((button) => button.textContent?.includes("Remove"));
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

  removeButtons = [...container.querySelectorAll<HTMLButtonElement>("button")].filter((button) => button.textContent?.includes("Remove"));
  assert.equal(removeButtons.length, 1, "script filter should leave only the visible script-job remove control in DOM queries");
  removeButtons[0].focus();
  await click(removeButtons[0]);
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
  assert.equal(document.activeElement, removeButtons[0], "closing dialog should restore focus to triggering button");

  await click(removeButtons[0]);
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

  mockIPC(() => {
    throw new TypeError("Cannot read properties of undefined (reading 'invoke')");
  });
  const bridgeContainer = document.createElement("div");
  document.body.replaceChildren(bridgeContainer);
  const bridgeRoot = createRoot(bridgeContainer);
  await act(async () => bridgeRoot.render(<AutomationsWorkspace />));
  await settle();
  assert.ok(bridgeContainer.textContent?.includes("Hermes desktop bridge is unavailable in this preview"), "browser-preview bridge errors should render as truthful product copy, not raw JavaScript exceptions");
  assert.ok(!bridgeContainer.textContent?.includes("Cannot read properties of undefined"), "raw invoke exceptions should not leak into the Automations layout");
  await act(async () => bridgeRoot.unmount());
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

  const earlySecondsJob = cronJob({ jobId: "cron_seconds", name: "Unix seconds ritual", nextRunAt: "1717933200", lastRunAt: "1717933200", script: null, noAgent: false });
  const laterIsoJob = cronJob({ jobId: "cron_later", name: "Later ISO ritual", nextRunAt: "2026-06-10T09:00:00.000Z", script: null, noAgent: false });
  const timeRender = await renderAutomations(automationList([laterIsoJob, earlySecondsJob]));
  const nextRunTitle = timeRender.container.querySelector(".automation-next-run-card strong");
  assert.equal(nextRunTitle?.textContent, "Unix seconds ritual", "next ritual should use parsed chronological time, not lexicographic string order");
  assert.ok(timeRender.container.textContent?.includes("2024"), "Unix-second timestamps should render as their real year, not 1970");
  const scriptSummary = [...timeRender.container.querySelectorAll(".automation-summary-card")].find((card) => card.textContent?.includes("Script-only"));
  assert.equal(scriptSummary?.querySelector("strong")?.textContent, "0", "zero script-only jobs should render as 0, not unknown dash");
  await act(async () => timeRender.root.unmount());
  clearMocks();

  const blankNameRender = await renderAutomations({
    ...automationList([cronJob({ jobId: "cron_blank_name", name: "   " })]),
    watchers: [{ id: "watcher:blank", name: "", state: "running", source: "Hermes", lastSeenAt: null, lastStatus: null, detail: null }],
  });
  assert.ok(blankNameRender.container.textContent?.includes("cron_blank_name"), "blank cron job names should fall back to job id in visible headings");
  assert.ok(blankNameRender.container.textContent?.includes("watcher:blank"), "blank watcher names should fall back to watcher id in visible headings");
  await click(buttonByText(blankNameRender.container, "Run now"));
  await settle();
  assert.ok(blankNameRender.container.textContent?.includes("This will run “cron_blank_name” now"), "confirm copy should not render empty quoted job names");
  await act(async () => blankNameRender.root.unmount());
  clearMocks();

  const runRender = await renderAutomations(automationList([cronJob({ jobId: "cron_run", name: "Manual run ritual" })]));
  await click(buttonByText(runRender.container, "Run now"));
  await settle();
  const runDialog = runRender.container.querySelector<HTMLElement>(".automation-confirm-panel");
  assert.ok(runDialog, "run now should open confirmation dialog");
  assert.equal(runDialog.getAttribute("aria-labelledby"), "automation-confirm-title", "dialog should be labelled by its visible title");
  assert.equal(runDialog.getAttribute("aria-describedby"), "automation-confirm-description", "dialog should describe its side-effect warning");
  assert.ok(runRender.container.textContent?.includes("may trigger external side effects"), "run confirmation should warn about external side effects");
  const runConfirm = runRender.container.querySelector<HTMLButtonElement>(".automation-confirm-actions .automation-primary-button");
  assert.ok(runConfirm, "run confirm button should render");
  await click(runConfirm);
  await settle();
  assert.ok(runRender.calls.some((call) => call.cmd === "manage_hermes_cron_job" && (call.args as { action?: string }).action === "run"), "run now should call Hermes manager with action run");
  await act(async () => runRender.root.unmount());
  clearMocks();

  let releasePause!: (value: AutomationList) => void;
  const concurrentFirst = cronJob({ jobId: "cron_concurrent_1", name: "Concurrent one", enabled: true, state: "enabled" });
  const concurrentSecond = cronJob({ jobId: "cron_concurrent_2", name: "Concurrent two", enabled: true, state: "enabled" });
  const concurrentRender = await renderAutomations(automationList([concurrentFirst, concurrentSecond]), {
    manageHandler: () => new Promise<AutomationList>((resolve) => { releasePause = resolve; }),
  });
  const pauseButtons = [...concurrentRender.container.querySelectorAll<HTMLButtonElement>(".automation-action-row button")].filter((button) => button.textContent?.includes("Pause"));
  assert.equal(pauseButtons.length, 2, "two pause controls should render for concurrency guard test");
  await click(pauseButtons[0]);
  await settle();
  assert.equal(concurrentRender.calls.filter((call) => call.cmd === "manage_hermes_cron_job").length, 1, "first action should start exactly one Hermes call");
  assert.ok([...concurrentRender.container.querySelectorAll<HTMLButtonElement>(".automation-action-row button")].every((button) => button.disabled), "all mutating job controls should disable while any action is in flight");
  await click(pauseButtons[1]);
  await settle();
  assert.equal(concurrentRender.calls.filter((call) => call.cmd === "manage_hermes_cron_job").length, 1, "disabled second-job control must not start a concurrent Hermes action");
  await act(async () => releasePause(automationList([{ ...concurrentFirst, enabled: false, state: "paused", lastStatus: "paused" }, concurrentSecond])));
  await settle();
  assert.ok(concurrentRender.container.textContent?.includes("Paused “Concurrent one”. Hermes provider read-back refreshed."), "serialized action should complete with provider read-back message");
  await act(async () => concurrentRender.root.unmount());
  clearMocks();
}

await runTests();
