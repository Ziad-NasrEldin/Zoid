import { AlertTriangle, Bot, CalendarClock, CheckCircle2, ExternalLink, RefreshCw, ShieldCheck } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { getMavoidSocialOverview, listMavoidSocialPosts, manageMavoidSocialAutomation, runMavoidBufferHealthCheck, validateMavoidMediaUrl } from "./socialClient";
import { canRetryBufferSchedule, deriveMavoidSocialStatusLabel, formatPlatformList } from "./socialViewModel";
import type { MavoidSocialOverview, MavoidSocialPost } from "./types";

type LoadState = "idle" | "loading" | "ready" | "error";

type RhythmStep = {
  time: string;
  title: string;
  label: string;
  state: string;
  detail: string;
};

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
  return neutralizeProviderCopy(value?.replace(/_/g, " ") ?? "waiting");
}

function formatBytes(bytes: number | null | undefined): string {
  if (!bytes) return "unknown size";
  if (bytes < 1024) return `${bytes} B`;
  if (bytes < 1024 * 1024) return `${Math.round(bytes / 1024)} KB`;
  return `${(bytes / (1024 * 1024)).toFixed(1)} MB`;
}

function openExternal(url: string | null | undefined) {
  if (!url || !/^https:\/\//i.test(url)) return;
  window.open(url, "_blank", "noopener,noreferrer");
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
        <div className={`social-provider-card social-provider-card--${overview?.overallStatus ?? "unknown"}`}>
          <ShieldCheck aria-hidden="true" size={20} />
          <span>Provider read-back</span>
          <strong>{overview ? deriveMavoidSocialStatusLabel(overview) : loadState}</strong>
          <small>{neutralizeProviderCopy(overview?.activeBlocker) || "Provider state comes from the local social workspace."}</small>
        </div>
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
        <button disabled={Boolean(busyAction)} onClick={() => automation("pause_monitor")} type="button">
          <CalendarClock aria-hidden="true" size={16} /> Pause monitor
        </button>
        <button disabled={Boolean(busyAction)} onClick={() => automation("resume_monitor")} type="button">
          <CalendarClock aria-hidden="true" size={16} /> Resume monitor
        </button>
        <button disabled={!selectedPostHasSafeMedia || Boolean(busyAction)} onClick={validateSelectedMedia} type="button"><ShieldCheck aria-hidden="true" size={16} /> Validate media</button>
        <button disabled={!canOpenExternal(overview?.latestReportPath)} onClick={() => openExternal(overview?.latestReportPath)} type="button"><ExternalLink aria-hidden="true" size={16} /> Latest report</button>
      </div>

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

        <main className="social-panel social-detail" aria-label="Selected post detail">
          {selectedPost ? (
            <>
              <nav className="social-section-tabs" aria-label="Content dashboard sections">
                <span>Overview</span><span>Queue</span><span>Post detail</span><span>Media</span><span>Reports</span>
              </nav>
              <div className="social-detail-heading"><span>{selectedPost.postDate} · {selectedPost.slotType.replace(/_/g, " ")}</span><h3>{neutralizeProviderCopy(selectedPost.title)}</h3><small>{neutralizeProviderCopy(selectedPost.topicOrNewsItem)}</small></div>
              <p className="social-caption">{neutralizeProviderCopy(selectedPost.caption)}</p>
              <div className="social-gate" role="status"><AlertTriangle aria-hidden="true" size={18} /> {retryState.ok ? "Schedule/retry available after confirmation." : `Schedule/retry locked: ${retryState.reason}`}</div>
              <div className="social-action-row">
                <button disabled={!retryState.ok} onClick={() => window.confirm("Retry provider scheduling for this approved post?") && setMessage("Schedule retry must be executed through the guarded backend action.")} type="button">Retry schedule</button>
                <button disabled type="button" title="Evidence is mandatory before resolving a post manually.">Manual resolution requires evidence</button>
                <button disabled={!canOpenExternal(selectedPost.review?.reportPath)} onClick={() => openExternal(selectedPost.review?.reportPath)} type="button">Open review report</button>
              </div>

              <section className="social-detail-section" aria-label="Design previews">
                <div className="social-panel-heading"><span>Design preview</span><strong>{selectedPost.mediaAssets.length}</strong><small>Generated visual assets, not text-only fallbacks.</small></div>
                <div className="social-media-gallery">
                  {selectedPost.mediaAssets.map((asset, index) => {
                    const source = canOpenExternal(asset.publicUrl) ? asset.publicUrl : null;
                    return (
                      <article className="social-media-card" key={`${asset.path}-${asset.publicUrl ?? "local"}`}>
                        <div className="social-media-preview">
                          {source ? <img alt={`${neutralizeValue(selectedPost.title)} design ${index + 1}`} loading="lazy" src={source} /> : <div className="social-media-fallback">Local-only asset metadata; preview requires a validated public HTTPS image URL.</div>}
                        </div>
                        <span>{asset.validationStatus} · {asset.provider ?? "local"}</span>
                        <strong>{asset.publicUrl ?? asset.path}</strong>
                        <small>{asset.contentType ?? "unknown type"} · {asset.width ?? "?"}×{asset.height ?? "?"} · {formatBytes(asset.bytes)} · checked {safeLabel(asset.validatedAt)}</small>
                        {asset.temporary ? <small className="social-warning">Temporary media host — replace with durable owned media before production scheduling.</small> : null}
                        <div className="social-action-row">
                          <button disabled={!source} onClick={() => openExternal(asset.publicUrl)} type="button">Open public media URL</button>
                          <button disabled={!source} onClick={() => validateMediaUrl(asset.publicUrl)} type="button">Validate media</button>
                        </div>
                      </article>
                    );
                  })}
                </div>
              </section>

              <section className="social-detail-section" aria-label="Review report">
                <div className="social-panel-heading"><span>Review</span><strong>{selectedPost.review?.verdict ?? "missing"}</strong><small>{safeLabel(selectedPost.review?.reviewer, "No reviewer recorded")}</small></div>
                <dl className="social-kv"><dt>Approved at</dt><dd>{safeLabel(selectedPost.review?.approvedAt)}</dd><dt>Report path</dt><dd>{safePathLabel(selectedPost.review?.reportPath)}</dd><dt>Required fixes</dt><dd>{selectedPost.review?.requiredFixes.length ? selectedPost.review.requiredFixes.join(" · ") : "No open required fixes"}</dd></dl>
              </section>

              <section className="social-detail-section" aria-label="Provider platform state">
                <div className="social-panel-heading"><span>Platform state</span><strong>{selectedPost.bufferPosts.length}</strong><small>Read-back, channel, and provider IDs stay visible before any completion claim.</small></div>
                <div className="social-platform-grid">
                  {selectedPost.bufferPosts.map((item) => (
                    <article className="social-platform-card" key={`${item.platform}-${item.bufferId ?? item.channelId ?? "pending"}`}>
                      <span>{item.platform}</span><strong>{safeLabel(item.channelDisplayName, item.platform)}</strong>
                      <small>State: {displayStatus(item.state)}</small>
                      <small>Provider post id: {safeLabel(item.bufferId)}</small>
                      <small>Channel: {safeLabel(item.channelId)}</small>
                      <small>Local: {safeLabel(item.scheduledAtLocal)}</small>
                      <small>UTC: {safeLabel(item.scheduledAtUtc)}</small>
                      <small>Read-back: {safeLabel(item.readBackVerifiedAt)}</small>
                      {item.lastErrorCode ? <small className="social-warning">{item.lastErrorCode}: {neutralizeProviderCopy(item.lastErrorMessage)}</small> : null}
                      {item.publishedUrl ? <button onClick={() => openExternal(item.publishedUrl)} type="button">Open published URL</button> : null}
                    </article>
                  ))}
                  {selectedPost.bufferPosts.length === 0 ? <p className="social-empty">No provider platform records yet.</p> : null}
                </div>
              </section>

              <section className="social-detail-section" aria-label="Reports and event history">
                <div className="social-panel-heading"><span>Reports + events</span><strong>{selectedPost.reports.length + selectedPost.events.length}</strong><small>Replayable audit trail with no secrets.</small></div>
                <div className="social-report-list">
                  {selectedPost.reports.map((report) => <button disabled={!canOpenExternal(report.path)} key={`${report.kind}-${report.path}`} onClick={() => openExternal(report.path)} type="button"><span>{safeLabel(report.kind)}</span><strong>{neutralizeValue(report.label)}</strong><small>{report.createdAt ?? safePathLabel(report.path)}</small></button>)}
                  {selectedPost.reports.length === 0 ? <p className="social-empty">No reports exist for this post yet.</p> : null}
                </div>
                <ol className="social-event-list">
                  {selectedPost.events.map((event) => <li key={`${event.timestamp}-${event.eventType}`}><time>{event.timestamp}</time><strong>{safeLabel(event.eventType.replace(/_/g, " "))}</strong><span>{neutralizeValue(event.message)}</span><small>{safeLabel(event.actor)} · {event.severity} · {safePathLabel(event.evidencePath)}</small></li>)}
                  {selectedPost.events.length === 0 ? <li>No provider events or publishing history recorded yet.</li> : null}
                </ol>
              </section>
            </>
          ) : <p>No post selected.</p>}
        </main>

        <aside className="social-panel social-automation-panel" aria-label="Automation state">
          <div className="social-panel-heading"><span>Hermes cron</span><strong>{overview?.automation.creatorState ?? "unknown"}</strong><small>creator {overview?.automation.creatorJobId ?? "—"}</small></div>
          <dl className="social-kv"><dt>Creator next run</dt><dd>{overview?.automation.creatorNextRunAt ?? "—"}</dd><dt>Monitor state</dt><dd>{overview?.automation.monitorState ?? "unknown"}</dd><dt>Monitor job</dt><dd>{overview?.automation.monitorJobId ?? "—"}</dd><dt>Cooldown next run</dt><dd>{overview?.automation.cooldownNextRunAt ?? "—"}</dd><dt>Provider endpoint</dt><dd>{overview?.bufferEndpoint ? "configured" : "—"}</dd><dt>HTTP status</dt><dd>{overview?.bufferHealth.httpStatus ? `HTTP ${overview.bufferHealth.httpStatus}` : "—"}</dd><dt>Last probe</dt><dd>{overview?.bufferHealth.lastCheckedAt ?? "—"}</dd><dt>Credentials</dt><dd>{overview?.bufferHealth.credentialsPresent.bufferAccessToken ? "Access token present" : "Access token missing"} · {overview?.bufferHealth.credentialsPresent.bufferOrganizationId ? "Organization present" : "Organization missing"}</dd><dt>Latest report</dt><dd>{safePathLabel(overview?.latestReportPath)}</dd></dl>
        </aside>
      </div>
    </section>
  );
}
