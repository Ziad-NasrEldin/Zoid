import { AlertTriangle, Bot, CalendarClock, CheckCircle2, ExternalLink, RefreshCw, ShieldCheck } from "lucide-react";
import { useEffect, useMemo, useRef, useState, type RefObject } from "react";
import { getMavoidSocialOverview, listMavoidSocialPosts, manageMavoidSocialAutomation, openMavoidSocialResource, runMavoidBufferHealthCheck, validateMavoidMediaUrl } from "./socialClient";
import { canRetryBufferSchedule, formatPlatformList } from "./socialViewModel";
import type { MavoidSocialOverview, MavoidSocialPost } from "./types";

type LoadState = "idle" | "loading" | "ready" | "error";

type RhythmStep = {
  time: string;
  title: string;
  label: string;
  state: string;
  detail: string;
};

type ScheduleDay = {
  date: string;
  label: string;
  posts: MavoidSocialPost[];
};

const slotTimes: Record<MavoidSocialPost["slotType"], string> = {
  ai_intel: "10:00",
  enterprise_carousel: "18:00",
  manual_campaign: "Planned",
};

function dateLabel(date: string): string {
  const parsed = new Date(`${date}T12:00:00`);
  if (Number.isNaN(parsed.getTime())) return date;
  return parsed.toLocaleDateString(undefined, { weekday: "short", month: "short", day: "numeric" });
}

function addDays(date: string, days: number): string {
  const parsed = new Date(`${date}T12:00:00`);
  if (Number.isNaN(parsed.getTime())) return date;
  parsed.setDate(parsed.getDate() + days);
  return parsed.toISOString().slice(0, 10);
}

function scheduleDays(posts: MavoidSocialPost[]): ScheduleDay[] {
  const sortedDates = posts.map((post) => post.postDate).filter(Boolean).sort();
  const start = sortedDates[0] ?? new Date().toISOString().slice(0, 10);
  return Array.from({ length: 7 }, (_, index) => {
    const date = addDays(start, index);
    return { date, label: dateLabel(date), posts: posts.filter((post) => post.postDate === date) };
  });
}

function bridgeErrorMessage(error: unknown): string {
  const message = error instanceof Error ? error.message : String(error);
  if (message.includes("Cannot read") || message.includes("invoke")) return "Zoid desktop bridge is unavailable in this preview. Open the native app to read local social state.";
  return message;
}

function neutralizeProviderCopy(value: string | null | undefined): string {
  return (value ?? "")
    .replace(/Buffer HTTP/gi, "Provider HTTP")
    .replace(/Buffer API/gi, "Provider API")
    .replace(/Buffer pipeline/gi, "Publishing pipeline")
    .replace(/Buffer migration/gi, "Publishing migration")
    .replace(/Buffer social automation/gi, "Social publishing automation")
    .replace(/Buffer/gi, "Provider");
}

function displayStatus(value: string | null | undefined) {
  const neutral = neutralizeProviderCopy(value?.replace(/_/g, " ") ?? "waiting").trim();
  if (/^rate limited$/i.test(neutral)) return "Rate-limited";
  return neutral;
}

function formatBytes(bytes: number | null | undefined): string {
  if (!bytes) return "unknown size";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

async function openExternal(url: string | null | undefined) {
  if (!url || !/^https:\/\//i.test(url)) return;
  await openMavoidSocialResource(url);
}

async function openResource(resource: string | null | undefined) {
  if (!resource) return;
  await openMavoidSocialResource(resource);
}

function canOpenExternal(url: string | null | undefined): boolean {
  return /^https:\/\//i.test(url ?? "");
}

function neutralizeValue(value: string | null | undefined): string {
  return neutralizeProviderCopy(value).replace(/buffer/gi, "provider");
}

function safeLabel(value: string | null | undefined, fallback = "—") {
  return neutralizeValue(value) || fallback;
}

function safePathLabel(path: string | null | undefined): string {
  if (!path) return "—";
  return canOpenExternal(path) ? path : "Local report path available";
}

function reportSourceLabel(path: string | null | undefined): string {
  if (!path) return "No report file linked";
  const parts = path.split(/[\\/]/).filter(Boolean);
  return parts.slice(-2).join("/") || path;
}

function providerRefreshCadence(overview: MavoidSocialOverview | null): string {
  if (!overview) return "Refresh reads local state now; provider checks run only on the button or Hermes monitor.";
  const providerChecked = overview.bufferHealth.lastCheckedAt ?? "not checked by provider API in this view";
  const next = overview.automation.cooldownNextRunAt ?? overview.automation.monitorNextRunAt;
  return `Provider/API checked: ${providerChecked}. Local state refreshed: ${overview.updatedAt}. Next automatic check: ${next ?? "not scheduled"}. Use Check provider API for an immediate provider re-check.`;
}

function rhythmSteps(overview: MavoidSocialOverview | null, selectedPost: MavoidSocialPost | null): RhythmStep[] {
  return [
    {
      time: "08:00",
      title: "Creator + design agent",
      label: overview?.automation.creatorState ?? "reading",
      state: overview?.automation.creatorEnabled ? "active" : "paused",
      detail: overview?.automation.creatorNextRunAt ?? "Next run comes from Hermes cron read-back.",
    },
    {
      time: "10:00",
      title: "Daily intel → publish",
      label: displayStatus(selectedPost?.status),
      state: overview?.bufferHealth.rateLimited ? "blocked" : "watching",
      detail: neutralizeProviderCopy(overview?.activeBlocker) || "Schedule only after review, media, and provider checks pass.",
    },
    {
      time: "18:00",
      title: "Evening post → publish",
      label: overview?.automation.monitorState ?? "monitor",
      state: overview?.bufferHealth.rateLimited ? "blocked" : "watching",
      detail: overview?.automation.cooldownNextRunAt ?? "Evening queue remains fail-closed without provider read-back.",
    },
  ];
}

export function SocialDashboard() {
  const [loadState, setLoadState] = useState<LoadState>("idle");
  const [overview, setOverview] = useState<MavoidSocialOverview | null>(null);
  const [posts, setPosts] = useState<MavoidSocialPost[]>([]);
  const [selectedPostId, setSelectedPostId] = useState<string | null>(null);
  const [message, setMessage] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busyAction, setBusyAction] = useState<string | null>(null);
  const [activeSection, setActiveSection] = useState("social-summary");

  async function refresh() {
    setLoadState("loading");
    setError(null);
    try {
      const [nextOverview, nextPosts] = await Promise.all([getMavoidSocialOverview(), listMavoidSocialPosts()]);
      setOverview(nextOverview);
      setPosts(nextPosts);
      setSelectedPostId((current) => current ?? nextPosts[0]?.id ?? null);
      setLoadState("ready");
    } catch (err) {
      setError(bridgeErrorMessage(err));
      setLoadState("error");
    }
  }

  useEffect(() => { void refresh(); }, []);

  const selectedPost = useMemo(() => posts.find((post) => post.id === selectedPostId) ?? posts[0] ?? null, [posts, selectedPostId]);
  const retryState = overview && selectedPost ? canRetryBufferSchedule(overview, selectedPost) : { ok: false, reason: "No selected post." };
  const rhythm = useMemo(() => rhythmSteps(overview, selectedPost), [overview, selectedPost]);
  const weekSchedule = useMemo(() => scheduleDays(posts), [posts]);
  const summaryRef = useRef<HTMLElement | null>(null);
  const scheduleRef = useRef<HTMLElement | null>(null);
  const mediaRef = useRef<HTMLElement | null>(null);
  const platformsRef = useRef<HTMLElement | null>(null);
  const reportsRef = useRef<HTMLElement | null>(null);

  function scrollToSection(ref: RefObject<HTMLElement | null>, sectionId: string) {
    setActiveSection(sectionId);
    ref.current?.scrollIntoView({ block: "start", behavior: window.matchMedia?.("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" });
  }

  async function runHealthCheck() {
    setBusyAction("health");
    setMessage(null);
    try {
      setOverview(await runMavoidBufferHealthCheck());
      setMessage("Provider health checked from the local social runtime.");
    } catch (err) {
      setError(bridgeErrorMessage(err));
    } finally {
      setBusyAction(null);
    }
  }

  async function automation(action: string) {
    setBusyAction(action);
    setMessage(null);
    try {
      setOverview(await manageMavoidSocialAutomation(action));
      setMessage(`Hermes automation action completed: ${action.replace(/_/g, " ")}.`);
    } catch (err) {
      setError(bridgeErrorMessage(err));
    } finally {
      setBusyAction(null);
    }
  }

  async function validateMediaUrl(url: string | null | undefined) {
    if (!url) {
      setMessage("No public media URL is available to validate.");
      return;
    }
    setBusyAction("validate_media");
    setMessage(null);
    try {
      const result = await validateMavoidMediaUrl(url);
      setMessage(`Media validation: ${result.ok ? "valid" : "blocked"} · HTTP ${result.httpStatus ?? "—"} · ${result.contentType ?? "unknown type"}`);
    } catch (err) {
      setError(bridgeErrorMessage(err));
    } finally {
      setBusyAction(null);
    }
  }

  async function validateSelectedMedia() {
    await validateMediaUrl(selectedPost?.mediaAssets.find((asset) => canOpenExternal(asset.publicUrl))?.publicUrl);
  }

  const selectedPostHasSafeMedia = selectedPost?.mediaAssets.some((asset) => canOpenExternal(asset.publicUrl)) ?? false;

  return (
    <section className="social-dashboard social-ink-command social-sumi-e" aria-label="MaVoid social operations dashboard">
      <header className="social-hero social-ink-hero">
        <div className="social-hero-copy">
          <p className="social-eyebrow kana-line">コンテンツ運用</p>
          <h2>Social operations command room</h2>
          <p>Three daily beats, one truthful provider read-back. Nothing here claims a post is scheduled or live until the local runtime and external state prove it.</p>
          <p className="social-reference-line">08:00 creator · 10:00 daily intel · 18:00 evening publish read-back</p>
        </div>
        <div className="social-ink-mark" aria-hidden="true"><span /><span /><span /></div>
      </header>

      <div className="social-rhythm-lane" aria-label="Daily automation rhythm">
        {rhythm.map((step) => (
          <article className={`social-rhythm-step social-rhythm-step--${step.state}`} key={step.time}>
            <time>{step.time}</time>
            <div>
              <strong>{step.title}</strong>
              <span>{step.label}</span>
              <small>{step.detail}</small>
            </div>
          </article>
        ))}
      </div>

      {error ? <div className="social-alert" role="alert"><AlertTriangle aria-hidden="true" size={18} /> {error}</div> : null}
      {message ? <div className="social-status" role="status"><CheckCircle2 aria-hidden="true" size={18} /> {message}</div> : null}

      <div className="social-toolbar" aria-label="Dashboard actions">
        <button disabled={loadState === "loading"} onClick={refresh} type="button"><RefreshCw aria-hidden="true" size={16} /> Refresh read-back</button>
        <button disabled={busyAction === "health"} onClick={runHealthCheck} type="button" title={overview?.bufferHealth.rateLimited ? "Provider is cooling down; use this only for one intentional health read-back." : undefined}><ExternalLink aria-hidden="true" size={16} /> Check provider API</button>
        <button disabled={Boolean(busyAction)} onClick={() => window.confirm("Run the 08:00 creator now? This can create new post artifacts.") && automation("run_creator")} type="button"><Bot aria-hidden="true" size={16} /> Run 8:00 creator</button>
        <button disabled={Boolean(busyAction)} onClick={() => automation(overview?.automation.creatorEnabled ? "pause_creator" : "resume_creator")} type="button">
          <CalendarClock aria-hidden="true" size={16} /> {overview?.automation.creatorEnabled ? "Pause creator" : "Resume creator"}
        </button>
        {overview?.automation.monitorEnabled ? (
          <button disabled={Boolean(busyAction)} onClick={() => automation("pause_monitor")} type="button">
            <CalendarClock aria-hidden="true" size={16} /> Pause monitor
          </button>
        ) : (
          <button disabled={Boolean(busyAction)} onClick={() => automation("resume_monitor")} type="button">
            <CalendarClock aria-hidden="true" size={16} /> Resume monitor
          </button>
        )}
        <button disabled={!selectedPostHasSafeMedia || Boolean(busyAction)} onClick={validateSelectedMedia} type="button"><ShieldCheck aria-hidden="true" size={16} /> Validate media</button>
        {overview?.latestReportPath ? <button onClick={() => void openResource(overview.latestReportPath)} type="button"><ExternalLink aria-hidden="true" size={16} /> Latest report</button> : <span className="social-latest-report-metadata">Latest report: {safePathLabel(overview?.latestReportPath)}</span>}
      </div>

      <section className="social-panel social-schedule-calendar" aria-label="Week schedule" id="social-calendar" ref={scheduleRef}>
        <div className="social-panel-heading"><span>Week schedule</span><strong>{posts.length}</strong><small>Loaded 7-day schedule from the first planned post in local state.</small></div>
        <div className="social-calendar-grid">
          {weekSchedule.map((day) => (
            <article className="social-calendar-day" key={day.date}>
              <time dateTime={day.date}>{day.label}</time>
              {day.posts.length ? day.posts.map((post) => (
                <button className={post.id === selectedPost?.id ? "social-calendar-card active" : "social-calendar-card"} key={post.id} onClick={() => setSelectedPostId(post.id)} type="button">
                  <span>{slotTimes[post.slotType]} · {displayStatus(post.status)}</span>
                  <strong>{neutralizeProviderCopy(post.title)}</strong>
                  <small>{post.id}</small>
                  <small>{formatPlatformList(post)} · {post.mediaAssets.length} creative{post.mediaAssets.length === 1 ? "" : "s"}</small>
                </button>
              )) : <p className="social-calendar-empty">No posts</p>}
            </article>
          ))}
        </div>
      </section>

      <div className="social-grid">
        <aside className="social-panel social-post-list" aria-label="Posts">
          <div className="social-panel-heading"><span>Publishing queue</span><strong>{overview?.counts.totalPosts ?? posts.length}</strong><small>{overview ? "Local social workspace" : "Loading workspace"}</small></div>
          {posts.map((post) => (
            <button className={post.id === selectedPost?.id ? "social-post-card active" : "social-post-card"} key={post.id} onClick={() => setSelectedPostId(post.id)} type="button">
              <span>{displayStatus(post.status)}</span>
              <strong>{neutralizeProviderCopy(post.title)}</strong>
              <small>{formatPlatformList(post)}</small>
            </button>
          ))}
          {posts.length === 0 && loadState !== "loading" ? <p className="social-empty">No local social posts found yet.</p> : null}
        </aside>

        <main className="social-panel social-detail social-detail-workbench" aria-label="Selected post detail">
          {selectedPost ? (
            <>
              <nav className="social-section-tabs" aria-label="Jump within selected post detail">
                <button aria-controls="social-summary" aria-current={activeSection === "social-summary" ? "true" : undefined} onClick={() => scrollToSection(summaryRef, "social-summary")} type="button">Summary</button>
                <button aria-controls="social-media" aria-current={activeSection === "social-media" ? "true" : undefined} onClick={() => scrollToSection(mediaRef, "social-media")} type="button">Media</button>
                <button aria-controls="social-platforms" aria-current={activeSection === "social-platforms" ? "true" : undefined} onClick={() => scrollToSection(platformsRef, "social-platforms")} type="button">Platforms</button>
                <button aria-controls="social-reports" aria-current={activeSection === "social-reports" ? "true" : undefined} onClick={() => scrollToSection(reportsRef, "social-reports")} type="button">Reports</button>
              </nav>

              <section className="social-detail-summary social-detail-hero-card" id="social-summary" ref={summaryRef}>
                <div className="social-detail-heading"><span>{selectedPost.postDate} · {selectedPost.slotType.replace(/_/g, " ")}</span><h3>{neutralizeProviderCopy(selectedPost.title)}</h3><small>{neutralizeProviderCopy(selectedPost.topicOrNewsItem)}</small></div>
                <p className="social-caption">{neutralizeProviderCopy(selectedPost.caption)}</p>
                <div className="social-gate" role="status"><AlertTriangle aria-hidden="true" size={18} /> {retryState.ok ? "Schedule/retry gates are clear; guarded backend retry is not exposed in this UI yet." : `Schedule/retry locked: ${retryState.reason}`}</div>
              </section>

              <section className="social-detail-section social-media-strip" aria-label="Design previews" id="social-media" ref={mediaRef}>
                <div className="social-panel-heading"><span>Design preview</span><strong>{selectedPost.mediaAssets.length}</strong><small>Generated visual assets, not text-only fallbacks.</small></div>
                <div className="social-media-gallery social-media-gallery--compact">
                  {selectedPost.mediaAssets.map((asset, index) => {
                    const source = canOpenExternal(asset.publicUrl) ? asset.publicUrl : null;
                    return (
                      <article className="social-media-card" key={`${asset.path}-${asset.publicUrl ?? "local"}`}>
                        <div className="social-media-preview social-media-thumb">
                          {source ? <img alt={`${neutralizeValue(selectedPost.title)} design ${index + 1}`} loading="lazy" src={source} /> : <div className="social-media-fallback">Local-only asset metadata; preview requires a validated public HTTPS image URL.</div>}
                        </div>
                        <span>{asset.validationStatus} · {asset.provider ?? "local"}</span>
                        <strong>{asset.publicUrl ?? asset.path}</strong>
                        <small>{asset.contentType ?? "unknown type"} · {asset.width ?? "?"}×{asset.height ?? "?"} · {formatBytes(asset.bytes)} · checked {safeLabel(asset.validatedAt)}</small>
                        {asset.temporary ? <small className="social-warning">Temporary media host — replace with durable owned media before production scheduling.</small> : null}
                        <div className="social-action-row">
                          <button disabled={!source} onClick={() => void openExternal(asset.publicUrl)} type="button">Open media URL</button>
                          <button disabled={!source} onClick={() => validateMediaUrl(asset.publicUrl)} type="button">Validate media</button>
                        </div>
                      </article>
                    );
                  })}
                </div>
              </section>

              <section className="social-detail-section social-detail-proof-grid" aria-label="Review and platform proof">
                <article className="social-proof-card" aria-label="Review report">
                  <div className="social-panel-heading"><span>Review</span><strong>{selectedPost.review?.verdict ?? "missing"}</strong><small>{safeLabel(selectedPost.review?.reviewer, "No reviewer recorded")}</small></div>
                  <dl className="social-kv"><dt>Approved at</dt><dd>{safeLabel(selectedPost.review?.approvedAt)}</dd><dt>Required fixes</dt><dd>{selectedPost.review?.requiredFixes.length ? selectedPost.review.requiredFixes.join(" · ") : "Not parsed from source; open report for full review notes."}</dd></dl>
                  <p className="social-action-note">Real source: {reportSourceLabel(selectedPost.review?.reportPath)} · verdict and approval state are read from the manifest/review file, not seeded UI text.</p>
                  {selectedPost.review?.reportPath ? <div className="social-action-row"><button onClick={() => void openResource(selectedPost.review?.reportPath)} type="button">Open review report</button></div> : <p className="social-action-note">Review report: {safePathLabel(selectedPost.review?.reportPath)}</p>}
                </article>

                <article className="social-proof-card" aria-label="Provider platform state" id="social-platforms" ref={platformsRef}>
                  <div className="social-panel-heading"><span>Platform state</span><strong>{selectedPost.bufferPosts.length}</strong><small>Read-back must exist before any completion claim.</small></div>
                  <div className="social-platform-grid social-platform-grid--compact">
                    {selectedPost.bufferPosts.map((item) => (
                      <article className="social-platform-card" key={`${item.platform}-${item.bufferId ?? item.channelId ?? "pending"}`}>
                        <span>{item.platform}</span><strong>{safeLabel(item.channelDisplayName, item.platform)}</strong>
                        <small>State: {displayStatus(item.state)}</small>
                        <small>Provider post id: {safeLabel(item.bufferId)}</small>
                        <small>Read-back: {safeLabel(item.readBackVerifiedAt)}</small>
                        {item.lastErrorCode ? <small className="social-warning">{item.lastErrorCode}: {neutralizeProviderCopy(item.lastErrorMessage)}</small> : null}
                        {item.publishedUrl ? <button onClick={() => void openExternal(item.publishedUrl)} type="button">Open published URL</button> : null}
                      </article>
                    ))}
                    {selectedPost.bufferPosts.length === 0 ? <p className="social-empty">No provider platform records yet.</p> : null}
                  </div>
                </article>
              </section>

              <section className="social-detail-section social-report-drawer" aria-label="Reports and event history" id="social-reports" ref={reportsRef}>
                <details open>
                  <summary><span>Reports + events</span><strong>{selectedPost.reports.length + selectedPost.events.length}</strong><small>Local artifacts and event history from the runtime workspace.</small></summary>
                  <p className="social-action-note">{providerRefreshCadence(overview)}</p>
                  <div className="social-report-list">
                    {selectedPost.reports.map((report) => {
                      const reportBody = <><span>{safeLabel(report.kind)}</span><strong>{neutralizeValue(report.label)}</strong><small>{report.createdAt ?? safePathLabel(report.path)}</small></>;
                      return <button key={`${report.kind}-${report.path}`} onClick={() => void openResource(report.path)} type="button">{reportBody}</button>;
                    })}
                    {selectedPost.reports.length === 0 ? <p className="social-empty">No reports exist for this post yet.</p> : null}
                  </div>
                  <ol className="social-event-list">
                    {selectedPost.events.map((event) => <li key={`${event.timestamp}-${event.eventType}`}><time>{event.timestamp}</time><strong>{safeLabel(event.eventType.replace(/_/g, " "))}</strong><span>{neutralizeValue(event.message)}</span><small>{safeLabel(event.actor)} · {event.severity} · {safePathLabel(event.evidencePath)}</small></li>)}
                    {selectedPost.events.length === 0 ? <li>No provider events or publishing history recorded yet.</li> : null}
                  </ol>
                </details>
                <p className="social-action-note">Manual resolution requires evidence and is intentionally not exposed as a disabled fake button.</p>
              </section>
            </>
          ) : <p>No post selected.</p>}
        </main>

        <aside className="social-panel social-automation-panel social-automation-summary" aria-label="Automation state">
          <div className="social-panel-heading"><span>Automation state</span><strong>{overview?.automation.creatorState ?? "unknown"}</strong><small>Only operational blockers and next actions.</small></div>
          <div className="social-automation-cards" aria-label="Automation state summary">
            <article><span>Creator</span><strong>{overview?.automation.creatorEnabled ? "Running" : "Paused"}</strong><small>Next: {overview?.automation.creatorNextRunAt ?? "—"}</small><small>ID: {overview?.automation.creatorJobId ?? "—"}</small></article>
            <article><span>Monitor</span><strong>{overview?.automation.monitorState ?? "unknown"}</strong><small>Job: {overview?.automation.monitorJobId ?? "—"}</small></article>
            <article><span>Cooldown</span><strong>{overview?.automation.cooldownNextRunAt ?? "None"}</strong><small>{overview?.bufferHealth.httpStatus ? `Last API: HTTP ${overview.bufferHealth.httpStatus}` : "API not checked"}</small></article>
          </div>
          <dl className="social-kv social-automation-health"><dt>External read-back</dt><dd>{overview?.bufferEndpoint ? "configured" : "not configured"}</dd><dt>Credentials</dt><dd>{overview?.bufferHealth.credentialsPresent.bufferAccessToken ? "Access token present" : "Access token missing"} · {overview?.bufferHealth.credentialsPresent.bufferOrganizationId ? "Organization present" : "Organization missing"}</dd><dt>Latest report</dt><dd>{safePathLabel(overview?.latestReportPath)}</dd></dl>
        </aside>
      </div>
    </section>
  );
}
