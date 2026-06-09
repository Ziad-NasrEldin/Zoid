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
    review: { verdict: "APPROVED", reviewer: "independent reviewer", reportPath: "/tmp/review.md", requiredFixes: ["Keep footer clear"], approvedAt: "2026-06-09T12:40:00Z" },
    mediaAssets: [
      { path: "/tmp/proof-1.png", publicUrl: "https://files.catbox.moe/9tix1y.png", contentType: "image/png", bytes: 99945, width: 1080, height: 1350, validatedAt: "2026-06-09T12:39:00Z", provider: "catbox", temporary: true, validationStatus: "valid" },
      { path: "/tmp/proof-2.png", publicUrl: "https://d.uguu.se/XiBkwaaa.png", contentType: "image/png", bytes: 99945, width: 1080, height: 1350, validatedAt: "2026-06-09T12:39:00Z", provider: "uguu", temporary: true, validationStatus: "valid" },
    ],
    bufferPosts: [
      { bufferId: null, platform: "instagram", channelId: "ig-1", channelDisplayName: "Instagram", scheduledAtUtc: null, scheduledAtLocal: null, state: "not_created", readBackVerifiedAt: null, publishedUrl: null, lastErrorCode: "RATE_LIMIT_EXCEEDED", lastErrorMessage: "24h cooldown" },
      { bufferId: "fb-post-1", platform: "facebook", channelId: "fb-1", channelDisplayName: "Facebook", scheduledAtUtc: "2026-06-10T07:00:00Z", scheduledAtLocal: "2026-06-10 10:00 Africa/Cairo", state: "scheduled", readBackVerifiedAt: "2026-06-09T12:50:00Z", publishedUrl: null, lastErrorCode: null, lastErrorMessage: null },
    ],
    reports: [
      { label: "Manifest", path: "/tmp/manifest.json", kind: "generation", createdAt: "2026-06-09T12:37:00Z" },
      { label: "Review report", path: "/tmp/review.md", kind: "review", createdAt: "2026-06-09T12:40:00Z" },
    ],
    events: [
      { timestamp: "2026-06-09T12:39:00Z", actor: "hermes", eventType: "review_approved", message: "Independent reviewer approved rendered PNGs.", severity: "success", evidencePath: "/tmp/review.md" },
      { timestamp: "2026-06-09T12:41:00Z", actor: "buffer", eventType: "rate_limited", message: "Provider returned 24h rate limit.", severity: "warning", evidencePath: "/tmp/buffer.json" },
    ],
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
    if (cmd === "mavoid_social_validate_media_url") return { url: "https://files.catbox.moe/9tix1y.png", ok: true, httpStatus: 200, contentType: "image/png", bytes: 99945, message: "Direct image URL is valid." };
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
  assert.ok(rendered.container.textContent?.includes("10:00"), "daily intel publish slot should be visible as a time lane");
  assert.ok(rendered.container.textContent?.includes("18:00"), "evening publish slot should be visible as a time lane");
  assert.ok(rendered.container.textContent?.includes("Social operations command room"), "hero should be compact and tool-agnostic");
  assert.ok(rendered.container.textContent?.includes("コンテンツ運用"), "hero eyebrow should be Japanese and tool-agnostic");
  assert.ok(!rendered.container.textContent?.includes("MaVoid · Buffer social automation"), "old English eyebrow should not render on the content page");
  assert.ok(!rendered.container.textContent?.includes("Buffer"), "visible content page wording should be provider/tool agnostic");
  assert.ok(rendered.container.textContent?.includes("Publishing pipeline proof"), "post titles should neutralize provider names in visible UI");
  assert.ok(rendered.container.textContent?.includes("Rate-limited · 24h cooldown"), "rate limit status should be visible");
  assert.ok(rendered.container.textContent?.includes("12fd35ec77e2"), "8:00 creator job id should be visible");
  assert.ok(rendered.container.textContent?.includes("https://files.catbox.moe/9tix1y.png"), "public direct media URL should be visible");
  assert.equal(rendered.container.querySelectorAll(".social-media-preview img").length, 2, "post detail should render actual design/image previews for every media asset");
  assert.equal(rendered.container.querySelector(".social-media-preview img")?.getAttribute("src"), "https://files.catbox.moe/9tix1y.png", "first rendered preview should use the validated public direct media URL");
  assert.ok(rendered.container.textContent?.includes("Review report"), "post detail should show review/report artifacts");
  assert.ok(rendered.container.textContent?.includes("APPROVED"), "post detail should show reviewer verdict");
  assert.ok(rendered.container.textContent?.includes("Keep footer clear"), "post detail should show reviewer required fixes/history");
  assert.ok(rendered.container.textContent?.includes("Instagram") && rendered.container.textContent?.includes("Facebook"), "post detail should show per-platform provider status cards");
  assert.ok(rendered.container.textContent?.includes("fb-post-1"), "post detail should show provider post ids when present");
  assert.ok(rendered.container.textContent?.includes("Independent reviewer approved rendered PNGs."), "post detail should show event history");
  assert.ok(rendered.container.textContent?.includes("HTTP 429"), "provider health card should show HTTP status clearly");
  assert.ok(rendered.container.textContent?.includes("Access token present") && rendered.container.textContent?.includes("Organization present"), "provider health card should show secret-safe credential booleans");
  assert.ok(rendered.container.textContent?.includes("Temporary media host"), "media hosting section should warn about temporary public URLs");
  assert.ok(rendered.container.textContent?.includes("Latest report"), "dashboard should expose latest report metadata/action area");
  assert.ok(rendered.container.textContent?.includes("Schedule/retry locked"), "schedule retry should be visibly locked when gates fail");
  assert.ok(rendered.container.textContent?.includes("Manual resolution requires evidence"), "manual posted action should be evidence-gated");
  assert.ok(rendered.container.querySelector("button")?.textContent !== undefined, "dashboard should keep actionable controls rendered");
  assert.ok([...rendered.container.querySelectorAll("button")].some((button) => button.textContent?.includes("Pause monitor")), "monitor pause control should render");
  assert.ok([...rendered.container.querySelectorAll("button")].some((button) => button.textContent?.includes("Validate media")), "media validation action should render");
  assert.ok(rendered.calls.includes("mavoid_social_get_overview"), "dashboard should read overview through Tauri command");

  const css = readFileSync("src/App.css", "utf8");
  const socialCss = css.slice(css.indexOf("/* MaVoid social operations dashboard */"));
  assert.ok(socialCss.includes("--social-ink: var(--sumi-ink)"), "social page should inherit sumi-e ink token instead of custom brown/yellow palette");
  assert.ok(socialCss.includes("--social-paper: var(--sumi-paper)"), "social page should inherit sumi-e paper token");
  assert.ok(socialCss.includes("height: 100vh"), "social page should own a viewport-height scroll surface inside the fixed Zoid shell");
  assert.ok(socialCss.includes("overflow-y: auto"), "social page should be vertically scrollable instead of clipping below the viewport");
  assert.ok(socialCss.includes(".social-toolbar button::before") && socialCss.includes("inset: auto 10px 6px"), "toolbar buttons should use the shared ink-rule button affordance, not generic pills");
  assert.ok(socialCss.includes("margin-top: clamp(18px, 2.4vw, 34px)"), "provider read-back card should sit lower in the hero instead of crowding the top edge");
  assert.ok(socialCss.includes(".social-automation-panel .social-kv") && socialCss.includes("grid-template-columns: 1fr"), "automation state rows should stack labels and values to prevent narrow-panel text overlap");
  assert.ok(socialCss.includes(".social-automation-panel .social-kv dd") && socialCss.includes("line-height: 1.35"), "automation state values should have readable line-height when wrapping");
  assert.ok(!socialCss.includes("rgba(255, 252, 242"), "social page must not use the rejected warm yellow card wash");
  assert.ok(!socialCss.includes("rgba(255, 241, 225"), "social page must not use the rejected yellow/orange rate-limit wash");
  assert.ok(!socialCss.includes("rgba(236,228,209"), "social hero must not use the rejected yellow background gradient");

  const healthButton = [...rendered.container.querySelectorAll("button")].find((button) => button.textContent?.includes("Check provider API"));
  assert.ok(healthButton, "provider health button should render");
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
