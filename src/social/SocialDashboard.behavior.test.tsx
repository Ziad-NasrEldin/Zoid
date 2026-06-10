import { strict as assert } from "node:assert";
import { readFileSync } from "node:fs";
import { Window } from "happy-dom";
import { flushSync } from "react-dom";
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
    automation: { creatorJobId: "12fd35ec77e2", creatorEnabled: true, creatorState: "active", creatorNextRunAt: "2026-06-10T08:00:00+03:00", monitorJobId: "9562e7cb93b6", monitorEnabled: true, monitorState: "active", monitorNextRunAt: "2026-06-10T10:00:00+03:00", cooldownJobId: "a0caa25a4cf7", cooldownNextRunAt: "2026-06-10T16:45:00+03:00" },
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
      { path: "/tmp/local-only.png", publicUrl: "file:///tmp/local-only.png", contentType: "image/png", bytes: 12345, width: 1080, height: 1350, validatedAt: null, provider: "local", temporary: false, validationStatus: "unchecked" },
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
  await act(async () => element.dispatchEvent(new window.MouseEvent("click", { bubbles: true, cancelable: true }) as unknown as Event));
}

async function renderDashboard(posts: MavoidSocialPost[] = [post()]): Promise<{ container: HTMLDivElement; root: Root; calls: string[]; validatedUrls: string[]; openedUrls: string[] }> {
  const calls: string[] = [];
  const validatedUrls: string[] = [];
  const openedUrls: string[] = [];
  window.open = ((url?: string | URL) => { openedUrls.push(String(url)); return null; }) as typeof window.open;
  let currentOverview = overview();
  mockIPC((cmd, args) => {
    calls.push(cmd);
    if (cmd === "mavoid_social_get_overview") return currentOverview;
    if (cmd === "mavoid_social_list_posts") return posts;
    if (cmd === "mavoid_social_run_buffer_health_check") return currentOverview;
    if (cmd === "mavoid_social_validate_media_url") {
      validatedUrls.push(String((args as { url?: string } | undefined)?.url));
      return { url: (args as { url?: string } | undefined)?.url ?? "", ok: true, httpStatus: 200, contentType: "image/png", bytes: 99945, message: "Direct image URL is valid." };
    }
    if (cmd === "mavoid_social_open_resource") {
      openedUrls.push(String((args as { resource?: string } | undefined)?.resource));
      return null;
    }
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
  return { container, root, calls, validatedUrls, openedUrls };
}

async function runTests() {
  const rendered = await renderDashboard();
  assert.ok(rendered.container.querySelector(".social-dashboard"), "dashboard shell should render");
  assert.ok(rendered.container.querySelector(".social-ink-command"), "dashboard should use the corrected ink command-room layout");
  assert.ok(rendered.container.querySelector(".social-sumi-e"), "content page should inherit the accepted Zoid sumi-e page system");
  assert.ok(rendered.container.querySelector(".social-ink-mark"), "content page hero should use the same ink mark language as adjacent Zoid pages");
  assert.ok(rendered.container.querySelector(".social-rhythm-lane"), "redesign should show the 8/10/18 rhythm as a first-class lane");
  assert.ok(rendered.container.querySelector(".social-detail-workbench"), "selected post detail should render as a compact workbench, not a long stacked scroll column");
  assert.ok(rendered.container.querySelector(".social-media-strip"), "media previews should be visible beside the selected post summary near the top");
  assert.ok(rendered.container.querySelector(".social-detail-proof-grid"), "review and platform proof should be consolidated into one compact grid");
  assert.ok(rendered.container.querySelector(".social-report-drawer details"), "long reports and events should be collapsed behind a native details disclosure");
  assert.ok(rendered.container.querySelector(".social-automation-summary .social-automation-cards"), "automation state should be summarized as readable cards instead of a long dense key-value list");
  assert.ok(rendered.container.querySelector(".social-schedule-calendar"), "dashboard should expose a real calendar/schedule view above the post detail");
  assert.ok(rendered.container.textContent?.includes("Week schedule"), "calendar view should be clearly labeled for scheduled posts and creatives");
  assert.ok(rendered.container.textContent?.includes("Loaded 7-day schedule"), "calendar should clarify that it is a loaded seven-day schedule, not a full month calendar");
  assert.equal(rendered.container.querySelectorAll(".social-section-tabs span").length, 0, "fake tab-looking spans must not remain in the dashboard");
  assert.ok([...rendered.container.querySelectorAll(".social-section-tabs button")].length >= 4, "section navigation should use real buttons if rendered");
  assert.ok([...rendered.container.querySelectorAll(".social-section-tabs button")].every((button) => button.getAttribute("aria-controls")), "section buttons should target real sections");
  assert.ok([...rendered.container.querySelectorAll(".social-section-tabs button")].some((button) => button.getAttribute("aria-current") === "true"), "section jump controls should expose the active/current target");
  assert.ok(![...rendered.container.querySelectorAll(".social-detail > .social-action-row button")].some((button) => button.textContent?.includes("Retry schedule")), "message-only retry schedule button must be removed unless wired to a real backend action");
  assert.ok(![...rendered.container.querySelectorAll(".social-detail > .social-action-row button")].some((button) => button.textContent?.includes("Manual resolution requires evidence")), "disabled future manual-resolution button should be replaced by readable gate copy");
  assert.ok(rendered.container.textContent?.includes("Manual resolution requires evidence"), "manual resolution constraint should remain visible as non-button gate copy");
  assert.ok(rendered.container.textContent?.includes("08:00"), "creator slot should be visible as a time lane");
  assert.ok(rendered.container.textContent?.includes("10:00"), "daily intel publish slot should be visible as a time lane");
  assert.ok(rendered.container.textContent?.includes("18:00"), "evening publish slot should be visible as a time lane");
  assert.ok(rendered.container.textContent?.includes("Social operations command room"), "hero should be compact and tool-agnostic");
  assert.ok(rendered.container.textContent?.includes("コンテンツ運用"), "hero eyebrow should be Japanese and tool-agnostic");
  assert.ok(!rendered.container.textContent?.includes("MaVoid · Buffer social automation"), "old English eyebrow should not render on the content page");
  assert.ok(!/buffer/i.test(rendered.container.textContent ?? ""), "visible content page wording should be provider/tool agnostic in any casing");
  assert.ok(rendered.container.textContent?.includes("Publishing pipeline proof"), "post titles should neutralize provider names in visible UI");
  assert.equal(rendered.container.querySelector(".social-provider-card"), null, "provider read-back hero card should not render");
  assert.ok(!rendered.container.textContent?.includes("Rate-limited · 24h cooldown"), "removed provider read-back status should not be visible in the hero");
  assert.ok(rendered.container.textContent?.includes("12fd35ec77e2"), "8:00 creator job id should be visible");
  assert.ok(rendered.container.textContent?.includes("https://files.catbox.moe/9tix1y.png"), "public direct media URL should be visible");
  assert.equal(rendered.container.querySelectorAll(".social-media-preview img").length, 2, "post detail should render actual design/image previews for every safe public HTTPS media asset");
  assert.equal(rendered.container.querySelector(".social-media-preview img")?.getAttribute("src"), "https://files.catbox.moe/9tix1y.png", "first rendered preview should use the validated public direct media URL");
  assert.ok([...(rendered.container.querySelectorAll(".social-media-preview img") as NodeListOf<HTMLImageElement>)].every((image) => !image.src.startsWith("file://") && !image.alt.includes("Buffer")), "preview images must not use file:// fallbacks or provider-specific accessible names");
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
  assert.ok([...rendered.container.querySelectorAll("button")].some((button) => button.textContent?.includes("Pause monitor")), "monitor pause control should render when monitor is enabled");
  assert.ok(![...rendered.container.querySelectorAll(".social-toolbar button")].some((button) => button.textContent?.includes("Resume monitor")), "monitor toolbar should not show both pause and resume actions at the same time");
  assert.ok([...rendered.container.querySelectorAll(".social-toolbar button")].some((button) => button.textContent?.includes("Latest report")), "local latest report should render as a real opener action");
  assert.ok([...rendered.container.querySelectorAll(".social-toolbar button")].every((button) => !button.textContent?.includes("Latest report") || !button.hasAttribute("disabled")), "latest report toolbar action should only render when actually openable");
  assert.ok([...rendered.container.querySelectorAll(".social-report-list button")].every((button) => !button.hasAttribute("disabled")), "report list should not render disabled fake buttons for local paths");
  assert.ok([...rendered.container.querySelectorAll(".social-report-list button")].some((item) => item.textContent?.includes("Manifest")), "local reports should be visible as real opener actions");
  assert.ok(rendered.container.textContent?.includes("Real source:"), "review verdict should identify its local source instead of appearing as fake seeded data");
  assert.ok(rendered.container.textContent?.includes("Next automatic check:"), "platform state/report history should explain refresh frequency and next automatic check");
  assert.ok([...rendered.container.querySelectorAll("button")].some((button) => button.textContent?.includes("Validate media")), "media validation action should render");
  const toolbarValidateButton = [...rendered.container.querySelectorAll(".social-toolbar button")].find((button) => button.textContent?.includes("Validate media"));
  assert.ok(toolbarValidateButton && !toolbarValidateButton.hasAttribute("disabled"), "toolbar validation should be enabled when at least one safe HTTPS media URL exists");
  await click(toolbarValidateButton);
  await settle();
  assert.equal(rendered.validatedUrls[rendered.validatedUrls.length - 1], "https://files.catbox.moe/9tix1y.png", "toolbar validation should skip unsafe media URLs and validate the first safe HTTPS URL");
  assert.equal(rendered.container.querySelectorAll(".social-report-list button[disabled]").length, 0, "local report paths should be metadata cards instead of disabled fake buttons");
  const perAssetValidateButtons = [...rendered.container.querySelectorAll(".social-media-card button")].filter((button) => button.textContent?.includes("Validate media"));
  assert.equal(perAssetValidateButtons.length, 3, "each media asset should render a validate affordance");
  assert.equal(perAssetValidateButtons.filter((button) => !button.hasAttribute("disabled")).length, 2, "only safe public HTTPS media assets should be validatable from the UI");
  const unsafeCardButtons = [...rendered.container.querySelectorAll(".social-media-card")]
    .find((card) => card.textContent?.includes("file:///tmp/local-only.png"))
    ?.querySelectorAll("button") as NodeListOf<HTMLButtonElement> | undefined;
  assert.ok(unsafeCardButtons && [...unsafeCardButtons].every((button) => button.hasAttribute("disabled")), "unsafe media URLs must not be openable or validatable");
  await click(perAssetValidateButtons[2]);
  await settle();
  assert.equal(rendered.validatedUrls[rendered.validatedUrls.length - 1], "https://d.uguu.se/XiBkwaaa.png", "second media card should validate its own URL, not the first asset URL");
  const firstOpenButton = [...rendered.container.querySelectorAll(".social-media-card button")].find((button) => button.textContent?.includes("Open media URL") && !button.hasAttribute("disabled"));
  assert.ok(firstOpenButton, "safe media assets should expose a real open action");
  await click(firstOpenButton);
  await settle();
  assert.equal(rendered.openedUrls[rendered.openedUrls.length - 1], "https://files.catbox.moe/9tix1y.png", "open media should call the native opener command with the asset URL");
  assert.ok(rendered.calls.includes("mavoid_social_get_overview"), "dashboard should read overview through Tauri command");

  const css = readFileSync("src/App.css", "utf8");
  const socialCss = css.slice(css.indexOf("/* MaVoid social operations dashboard */"));
  assert.ok(socialCss.includes("/* Social dashboard hard redraw: single-flow non-overlap layout */"), "social page should use the full redraw block that forces a single-flow non-overlap layout");
  assert.ok(socialCss.includes("--social-ink: var(--sumi-ink)"), "social page should inherit sumi-e ink token instead of custom brown/yellow palette");
  assert.ok(socialCss.includes("--social-paper: var(--sumi-paper)"), "social page should inherit sumi-e paper token");
  assert.ok(socialCss.includes("height: 100vh"), "social page should own a viewport-height scroll surface inside the fixed Zoid shell");
  assert.ok(socialCss.includes("overflow-y: auto"), "social page should be vertically scrollable instead of clipping below the viewport");
  assert.ok(socialCss.includes(".social-toolbar button::before") && socialCss.includes("inset: auto 10px 6px"), "toolbar buttons should use the shared ink-rule button affordance, not generic pills");
  assert.ok(!socialCss.includes(".social-provider-card"), "removed provider read-back card should not leave active CSS selectors behind");
  assert.ok(socialCss.includes(".social-toolbar { display: grid") && socialCss.includes("repeat(auto-fit, minmax(180px, 1fr))"), "dashboard actions should use a deterministic responsive grid instead of ragged flex wrapping");
  assert.ok(socialCss.includes(".social-schedule-calendar") && socialCss.includes("grid-template-columns: repeat(7"), "dashboard should include a visible weekly calendar grid for scheduled posts and creatives");
  assert.ok(socialCss.includes("container-type: inline-size"), "dashboard column decisions should use container width, not only viewport width");
  assert.ok(socialCss.includes(".social-calendar-grid {\n  min-width: 0;") && socialCss.includes("overflow-x: auto") && socialCss.includes("grid-template-columns: repeat(7, minmax(136px, 1fr))"), "calendar should isolate horizontal overflow instead of bleeding under the detail grid");
  assert.ok(socialCss.includes(".social-media-preview") && socialCss.includes("height: 126px") && socialCss.includes("max-height: 126px"), "media assets should render as deterministic compact thumbnails for multi-creative posts");
  assert.ok(socialCss.includes(".social-grid {\n  display: flex;\n  flex-direction: column;"), "social queue, detail, and automation should stack as full-width panels to prevent cross-panel overlap in the shell");
  assert.ok(socialCss.includes("grid-template-columns: repeat(auto-fit, minmax(230px, 1fr))"), "publishing queue should become a compact card grid instead of a narrow rail that competes with detail width");
  assert.ok(socialCss.includes(".social-detail-workbench") && socialCss.includes("grid-template-areas: \"tabs tabs\" \"summary media\" \"proof proof\" \"reports reports\""), "selected-post details should use explicit workbench grid areas instead of source-order layout accidents");
  assert.ok(socialCss.includes(".social-detail-workbench {\n  display: flex;\n  flex-direction: column;"), "redrawn selected-post details should fall back to simple document flow rather than overlapping grid rows");
  assert.ok(socialCss.includes(".social-report-drawer summary") && socialCss.includes(".social-report-drawer details"), "long audit history should be collapsed into a readable disclosure");
  assert.ok(socialCss.includes(".social-schedule-calendar { position: relative; z-index: auto") && socialCss.includes("overflow: hidden") && socialCss.includes("margin-bottom: clamp(10px, 1.4vw, 18px)"), "calendar should reserve space and clip its own overflow instead of using z-index band-aids");
  assert.ok(socialCss.includes(".social-automation-summary") && socialCss.includes(".social-automation-cards"), "automation state should be redesigned into a summarized command panel");
  assert.ok(socialCss.includes("@container (max-width: 1100px)") && socialCss.includes(".social-automation-panel { grid-column: 1 / -1; }"), "automation state should respond to available dashboard container width before it crushes the center column");
  assert.ok(socialCss.includes(".social-automation-panel .social-kv") && socialCss.includes("grid-template-columns: 1fr"), "automation state rows should stack labels and values to prevent narrow-panel text overlap");
  assert.ok(!socialCss.includes(".social-automation-panel .social-kv { grid-template-columns: repeat(3"), "automation key/value rows must not split dt/dd pairs into an eye-draining three-column grid");
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

  const unsafeOnly = await renderDashboard([post({
    mediaAssets: [
      { path: "/tmp/local-only.png", publicUrl: "file:///tmp/local-only.png", contentType: "image/png", bytes: 12345, width: 1080, height: 1350, validatedAt: null, provider: "local", temporary: false, validationStatus: "unchecked" },
    ],
  })]);
  const unsafeOnlyToolbarValidate = [...unsafeOnly.container.querySelectorAll(".social-toolbar button")].find((button) => button.textContent?.includes("Validate media"));
  assert.ok(unsafeOnlyToolbarValidate?.hasAttribute("disabled"), "toolbar media validation should stay disabled when no safe HTTPS media URL exists");
  assert.equal(unsafeOnly.validatedUrls.length, 0, "unsafe-only render should not validate media during initial load");
  await act(async () => unsafeOnly.root.unmount());
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
