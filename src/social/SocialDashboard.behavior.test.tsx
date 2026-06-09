import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import { Window } from "happy-dom";
import { act } from "react";
import { createRoot, type Root } from "react-dom/client";
import { clearMocks, mockIPC } from "@tauri-apps/api/mocks";
import { SocialDashboard } from "./SocialDashboard";
import type { MavoidSocialOverview, MavoidSocialPost } from "./types";

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

function overview(overrides: Partial<MavoidSocialOverview> = {}): MavoidSocialOverview {
  return {
    workspacePath: "/Users/ziadnasreldin/MaVoid/social-automation-buffer",
    overallStatus: "rate_limited",
    activeBlocker: "Buffer HTTP 429 RATE_LIMIT_EXCEEDED window=24h",
    bufferEndpoint: "https://api.buffer.com/graphql",
    bufferHealth: { ok: false, httpStatus: 429, rateLimited: true, rateLimitWindow: "24h", credentialsPresent: { bufferAccessToken: true, bufferOrganizationId: true }, lastCheckedAt: "2026-06-09T12:40:00Z", message: "rate limited" },
    automation: { creatorJobId: "12fd35ec77e2", creatorEnabled: true, creatorState: "active", creatorNextRunAt: "2026-06-10T08:00:00+03:00", monitorJobId: "9562e7cb93b6", monitorEnabled: false, monitorState: "paused", monitorNextRunAt: null, cooldownJobId: "a0caa25a4cf7", cooldownNextRunAt: "2026-06-10T16:45:00+03:00" },
    counts: { totalPosts: 1, needsReview: 0, readyToSchedule: 0, scheduledVerified: 0, posted: 0, blocked: 1 },
    nextSlots: [],
    latestReportPath: "/tmp/report.md",
    updatedAt: "2026-06-09T12:40:00Z",
    ...overrides,
  };
}

function post(overrides: Partial<MavoidSocialPost> = {}): MavoidSocialPost {
  return {
    id: "proof-post-2026-06-09",
    postDate: "2026-06-09",
    slotType: "manual_campaign",
    title: "Buffer pipeline proof",
    topicOrNewsItem: "Buffer migration proof",
    caption: "AI automation and software for complex operations.",
    platforms: ["instagram", "facebook", "linkedin"],
    status: "rate_limited",
    review: { verdict: "APPROVED", reviewer: "independent reviewer", reportPath: "/tmp/review.md", requiredFixes: [], approvedAt: null },
    mediaAssets: [{ path: "/tmp/proof.png", publicUrl: "https://files.catbox.moe/9tix1y.png", contentType: "image/png", bytes: 99945, width: 1080, height: 1350, validatedAt: null, provider: "catbox", temporary: true, validationStatus: "valid" }],
    bufferPosts: [],
    reports: [],
    events: [],
    ...overrides,
  };
}

async function settle() {
  await act(async () => {
    await Promise.resolve();
    await Promise.resolve();
  });
}

async function click(element: Element) {
  await act(async () => element.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true }) as unknown as Event));
}

async function renderDashboard(): Promise<{ container: HTMLDivElement; root: Root; calls: string[] }> {
  const calls: string[] = [];
  let currentOverview = overview();
  mockIPC((cmd) => {
    calls.push(cmd);
    if (cmd === "mavoid_social_get_overview") return currentOverview;
    if (cmd === "mavoid_social_list_posts") return [post()];
    if (cmd === "mavoid_social_run_buffer_health_check") return currentOverview;
    if (cmd === "mavoid_social_manage_automation") {
      currentOverview = overview({ automation: { ...currentOverview.automation, creatorEnabled: false, creatorState: "paused" } });
      return currentOverview;
    }
    throw new Error(`Unexpected command: ${cmd}`);
  });
  const container = document.createElement("div");
  document.body.replaceChildren(container);
  const root = createRoot(container);
  await act(async () => root.render(<SocialDashboard />));
  await settle();
  return { container, root, calls };
}

async function runTests() {
  const rendered = await renderDashboard();
  assert.ok(rendered.container.querySelector(".social-dashboard"), "dashboard shell should render");
  assert.ok(rendered.container.querySelector(".social-ink-command"), "dashboard should use the corrected ink command-room layout");
  assert.ok(rendered.container.querySelector(".social-sumi-e"), "content page should inherit the accepted Zoid sumi-e page system");
  assert.ok(rendered.container.querySelector(".social-ink-mark"), "content page hero should use the same ink mark language as adjacent Zoid pages");
  assert.ok(rendered.container.querySelector(".social-rhythm-lane"), "redesign should show the 8/10/18 rhythm as a first-class lane");
  assert.ok(rendered.container.textContent?.includes("08:00"), "creator slot should be visible as a time lane");
  assert.ok(rendered.container.textContent?.includes("10:00"), "daily intel Buffer slot should be visible as a time lane");
  assert.ok(rendered.container.textContent?.includes("18:00"), "evening Buffer slot should be visible as a time lane");
  assert.ok(rendered.container.textContent?.includes("Ink control room"), "hero should reflect sumi-e ink control-room direction");
  assert.ok(rendered.container.textContent?.includes("マヴォイド・バッファ自動投稿"), "hero eyebrow should be Japanese, not the old English eyebrow");
  assert.ok(!rendered.container.textContent?.includes("MaVoid · Buffer social automation"), "old English eyebrow should not render on the content page");
  assert.ok(rendered.container.textContent?.includes("Rate-limited · 24h cooldown"), "rate limit status should be visible");
  assert.ok(rendered.container.textContent?.includes("12fd35ec77e2"), "8:00 creator job id should be visible");
  assert.ok(rendered.container.textContent?.includes("https://files.catbox.moe/9tix1y.png"), "public direct media URL should be visible");
  assert.ok(rendered.calls.includes("mavoid_social_get_overview"), "dashboard should read overview through Tauri command");

  const css = readFileSync("src/App.css", "utf8");
  const socialCss = css.slice(css.indexOf("/* MaVoid Buffer social dashboard */"));
  assert.ok(socialCss.includes("--social-ink: var(--sumi-ink)"), "social page should inherit sumi-e ink token instead of custom brown/yellow palette");
  assert.ok(socialCss.includes("--social-paper: var(--sumi-paper)"), "social page should inherit sumi-e paper token");
  assert.ok(socialCss.includes("height: 100vh"), "social page should own a viewport-height scroll surface inside the fixed Zoid shell");
  assert.ok(socialCss.includes("overflow-y: auto"), "social page should be vertically scrollable instead of clipping below the viewport");
  assert.ok(socialCss.includes(".social-toolbar button::before") && socialCss.includes("inset: auto 10px 6px"), "toolbar buttons should use the shared ink-rule button affordance, not generic pills");
  assert.ok(!socialCss.includes("rgba(255, 252, 242"), "social page must not use the rejected warm yellow card wash");
  assert.ok(!socialCss.includes("rgba(255, 241, 225"), "social page must not use the rejected yellow/orange rate-limit wash");
  assert.ok(!socialCss.includes("rgba(236,228,209"), "social hero must not use the rejected yellow background gradient");

  const healthButton = [...rendered.container.querySelectorAll("button")].find((button) => button.textContent?.includes("Check Buffer API"));
  assert.ok(healthButton, "Buffer health button should render");
  await click(healthButton);
  await settle();
  assert.ok(rendered.calls.includes("mavoid_social_run_buffer_health_check"), "health button should call backend Buffer check");

  const pauseButton = [...rendered.container.querySelectorAll("button")].find((button) => button.textContent?.includes("Pause creator"));
  assert.ok(pauseButton, "creator pause button should render");
  await click(pauseButton);
  await settle();
  assert.ok(rendered.calls.includes("mavoid_social_manage_automation"), "automation button should call backend manager");

  await act(async () => rendered.root.unmount());
  clearMocks();

  mockIPC(() => { throw new TypeError("Cannot read properties of undefined (reading 'invoke')"); });
  const bridgeContainer = document.createElement("div");
  document.body.replaceChildren(bridgeContainer);
  const bridgeRoot = createRoot(bridgeContainer);
  await act(async () => bridgeRoot.render(<SocialDashboard />));
  await settle();
  assert.ok(bridgeContainer.textContent?.includes("Zoid desktop bridge is unavailable in this preview"), "browser bridge failure should be user-readable");
  await act(async () => bridgeRoot.unmount());
  clearMocks();

  console.log("social dashboard behavior tests passed");
}

void runTests();
