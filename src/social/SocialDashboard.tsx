import { AlertTriangle, Bot, CalendarClock, CheckCircle2, ExternalLink, RefreshCw, RotateCcw, ShieldCheck, Wand2, X } from "lucide-react";
import { useEffect, useMemo, useRef, useState, type ReactNode, type RefObject } from "react";
import { createPortal } from "react-dom";
import { getMavoidSocialOverview, listMavoidSocialPosts, manageMavoidSocialAutomation, openMavoidSocialResource, retryMavoidSocialDesign, runMavoidBufferHealthCheck, startMavoidSocialPostGeneration, validateMavoidMediaUrl } from "./socialClient";
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

type ContentType = {
  id: MavoidSocialPost["slotType"];
  label: string;
  description: string;
};

type PreviewAsset = {
  index: number;
};

type RedesignTarget = {
  postId: string;
  mediaPath: string;
  label: string;
} | null;

type ProviderVerificationTarget = {
  postId: string;
  mediaUrl: string | null;
  detailsResource: string | null;
} | null;

const slotTimes: Record<MavoidSocialPost["slotType"], string> = {
  ai_intel: "10:00",
  enterprise_carousel: "18:00",
  manual_campaign: "Planned",
};

const contentTypes: ContentType[] = [
  { id: "ai_intel", label: "AI Intel brief", description: "Generate the morning AI intelligence post for this date." },
  { id: "enterprise_carousel", label: "Carousel", description: "Generate an enterprise carousel package for this date." },
];

const platformLabels: Record<string, string> = { instagram: "Instagram", facebook: "Facebook", linkedin: "LinkedIn", x: "X" };
const platformIcons: Record<string, string> = { instagram: "◎", facebook: "f", linkedin: "in", x: "𝕏" };

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

function readableDateTime(value: string | null | undefined): string {
  if (!value) return "Not scheduled";
  const parsed = new Date(value);
  if (Number.isNaN(parsed.getTime())) return value;
  const now = new Date();
  const tomorrow = new Date(now);
  tomorrow.setDate(now.getDate() + 1);
  const sameDay = parsed.toDateString() === now.toDateString();
  const nextDay = parsed.toDateString() === tomorrow.toDateString();
  const day = sameDay ? "Today" : nextDay ? "Tomorrow" : parsed.toLocaleDateString(undefined, { weekday: "short", month: "short", day: "numeric" });
  const time = parsed.toLocaleTimeString(undefined, { hour: "numeric", minute: "2-digit" });
  return `${day}, ${time} Cairo`;
}

function hasUnverifiedProviderState(post: MavoidSocialPost | null): boolean {
  return post?.bufferPosts.some((item) => (item.state === "scheduled" || item.state === "posted") && !item.readBackVerifiedAt) ?? false;
}

function providerDetailsResource(post: MavoidSocialPost | null): string | null {
  if (!post) return null;
  return post.bufferPosts.find((item) => canOpenExternal(item.publishedUrl))?.publishedUrl
    ?? post.reports.find((report) => report.kind === "buffer" || report.kind === "monitor")?.path
    ?? post.review?.reportPath
    ?? null;
}

function renderScreenOverlay(overlay: ReactNode): ReactNode {
  if (typeof document === "undefined") return overlay;
  return createPortal(overlay, document.body);
}

function platformIconList(post: MavoidSocialPost) {
  return post.platforms.map((platform) => (
    <span className="social-platform-icon" aria-label={platformLabels[platform] ?? platform} key={platform} title={platformLabels[platform] ?? platform}>{platformIcons[platform] ?? platform.slice(0, 1).toUpperCase()}</span>
  ));
}

function rhythmSteps(overview: MavoidSocialOverview | null, selectedPost: MavoidSocialPost | null): RhythmStep[] {
  const entries = overview?.nextSlots?.length ? overview.nextSlots.map((slot) => ({
    time: slot.localPublishTime || slotTimes[slot.slotType],
    title: contentTypes.find((type) => type.id === slot.slotType)?.label ?? slot.slotType.replace(/_/g, " "),
    label: displayStatus(slot.status),
    state: slot.status.includes("failed") || slot.status.includes("blocked") ? "blocked" : "watching",
    detail: readableDateTime(slot.utcPublishTime ?? slot.date),
  })) : [];
  if (entries.length) return entries;
  return [
    { time: "08:00", title: "Creator + design agent", label: overview?.automation.creatorState ?? "reading", state: overview?.automation.creatorEnabled ? "active" : "paused", detail: readableDateTime(overview?.automation.creatorNextRunAt) },
    { time: "10:00", title: "Daily intel → publish", label: displayStatus(selectedPost?.status), state: overview?.bufferHealth.rateLimited ? "blocked" : "watching", detail: "Review, media, and provider checks gate scheduling." },
    { time: "18:00", title: "Evening post → publish", label: overview?.automation.monitorState ?? "monitor", state: overview?.bufferHealth.rateLimited ? "blocked" : "watching", detail: readableDateTime(overview?.automation.cooldownNextRunAt ?? overview?.automation.monitorNextRunAt) },
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
  const [quickMenuDate, setQuickMenuDate] = useState<string | null>(null);
  const [previewAsset, setPreviewAsset] = useState<PreviewAsset | null>(null);
  const [redesignTarget, setRedesignTarget] = useState<RedesignTarget>(null);
  const [providerVerificationTarget, setProviderVerificationTarget] = useState<ProviderVerificationTarget>(null);
  const [redesignNotes, setRedesignNotes] = useState("");
  const [assetStates, setAssetStates] = useState<Record<string, string>>({});

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

  useEffect(() => {
    if (!previewAsset && !redesignTarget && !providerVerificationTarget) return;
    function handleEscape(event: KeyboardEvent) {
      if (event.key !== "Escape") return;
      setPreviewAsset(null);
      setRedesignTarget(null);
      setProviderVerificationTarget(null);
    }
    window.addEventListener("keydown", handleEscape);
    return () => window.removeEventListener("keydown", handleEscape);
  }, [previewAsset, redesignTarget, providerVerificationTarget]);

  const selectedPost = useMemo(() => posts.find((post) => post.id === selectedPostId) ?? posts[0] ?? null, [posts, selectedPostId]);
  const retryState = overview && selectedPost ? canRetryBufferSchedule(overview, selectedPost) : { ok: false, reason: "No selected post." };
  const rhythm = useMemo(() => rhythmSteps(overview, selectedPost), [overview, selectedPost]);
  const weekSchedule = useMemo(() => scheduleDays(posts), [posts]);
  const summaryRef = useRef<HTMLElement | null>(null);
  const scheduleRef = useRef<HTMLElement | null>(null);
  const mediaRef = useRef<HTMLElement | null>(null);
  const platformsRef = useRef<HTMLElement | null>(null);

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
    if (hasUnverifiedProviderState(selectedPost)) {
      setProviderVerificationTarget({ postId: selectedPost?.id ?? "", mediaUrl: url ?? null, detailsResource: providerDetailsResource(selectedPost) });
      setMessage(null);
      setError(null);
      return;
    }
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

  async function verifyProviderState() {
    if (!providerVerificationTarget) return;
    setBusyAction("verify_provider_state");
    setMessage(null);
    setError(null);
    try {
      await runMavoidBufferHealthCheck();
      const [nextOverview, nextPosts] = await Promise.all([getMavoidSocialOverview(), listMavoidSocialPosts()]);
      setOverview(nextOverview);
      setPosts(nextPosts);
      const latestPost = nextPosts.find((post) => post.id === providerVerificationTarget.postId) ?? selectedPost;
      if (!hasUnverifiedProviderState(latestPost)) {
        const mediaUrl = providerVerificationTarget.mediaUrl;
        setProviderVerificationTarget(null);
        setMessage("Provider state verified for read-back. Schedule retry remains locked while provider records already exist, so duplicates are not created.");
        if (mediaUrl) {
          const result = await validateMavoidMediaUrl(mediaUrl);
          setMessage(`Provider state verified for read-back. Schedule retry remains locked while provider records already exist. Media validation: ${result.ok ? "valid" : "blocked"} · HTTP ${result.httpStatus ?? "—"} · ${result.contentType ?? "unknown type"}`);
        }
      } else {
        setMessage("Provider health check finished, but this post still has unverified provider state in local read-back. Refresh after the provider report updates before retrying schedule actions.");
      }
    } catch (err) {
      setError(bridgeErrorMessage(err));
    } finally {
      setBusyAction(null);
    }
  }

  async function generateForDate(date: string, contentType: ContentType) {
    setBusyAction(`generate_${date}_${contentType.id}`);
    setQuickMenuDate(null);
    setMessage(null);
    try {
      const result = await startMavoidSocialPostGeneration(date, contentType.id);
      setMessage(result.message || `${contentType.label} generation queued for ${date}.`);
      await refresh();
    } catch (err) {
      setError(bridgeErrorMessage(err));
    } finally {
      setBusyAction(null);
    }
  }

  async function submitRedesign() {
    if (!redesignTarget) return;
    setBusyAction(`redesign_${redesignTarget.mediaPath}`);
    setAssetStates((current) => ({ ...current, [redesignTarget.mediaPath]: "queued" }));
    try {
      const result = await retryMavoidSocialDesign(redesignTarget.postId, redesignTarget.mediaPath, redesignNotes);
      setAssetStates((current) => ({ ...current, [redesignTarget.mediaPath]: result.ok ? "working" : "failed" }));
      setMessage(result.message || "Designer agent redesign queued.");
      setRedesignTarget(null);
      setRedesignNotes("");
      await refresh();
    } catch (err) {
      setAssetStates((current) => ({ ...current, [redesignTarget.mediaPath]: "failed" }));
      setError(bridgeErrorMessage(err));
    } finally {
      setBusyAction(null);
    }
  }

  const selectedPostHasSafeMedia = selectedPost?.mediaAssets.some((asset) => canOpenExternal(asset.publicUrl)) ?? false;
  const selectedPostNeedsProviderVerification = hasUnverifiedProviderState(selectedPost);
  const validateActionLabel = selectedPostNeedsProviderVerification ? "Verify provider state" : "Validate";
  const validateActionTitle = selectedPostHasSafeMedia
    ? selectedPostNeedsProviderVerification
      ? "Verify provider read-back before validating media or retrying schedule actions."
      : "Validate the selected post's first public HTTPS media URL."
    : "No public HTTPS media URL is available on the selected post.";
  const selectedPreviewAssets = useMemo(() => (selectedPost?.mediaAssets ?? [])
    .map((asset, index) => ({ asset, index, source: canOpenExternal(asset.publicUrl) ? asset.publicUrl : null, alt: `${neutralizeValue(selectedPost?.title)} design ${index + 1}` }))
    .filter((entry) => entry.source), [selectedPost]);
  const activePreviewIndex = previewAsset ? selectedPreviewAssets.findIndex((entry) => entry.index === previewAsset.index) : -1;
  const activePreview = activePreviewIndex >= 0 ? selectedPreviewAssets[activePreviewIndex] : null;
  const hasPreviewNavigation = selectedPreviewAssets.length > 1;

  function openPreview(index: number) {
    setPreviewAsset({ index });
  }

  function movePreview(direction: -1 | 1) {
    if (!selectedPreviewAssets.length || activePreviewIndex < 0) return;
    const nextIndex = (activePreviewIndex + direction + selectedPreviewAssets.length) % selectedPreviewAssets.length;
    setPreviewAsset({ index: selectedPreviewAssets[nextIndex].index });
  }

  function replacePreviewMedia() {
    if (!selectedPost || !activePreview) return;
    setRedesignTarget({ postId: selectedPost.id, mediaPath: activePreview.asset.path, label: `Design ${activePreview.index + 1}` });
    setRedesignNotes("");
    setPreviewAsset(null);
  }

  return (
    <>
      <section className="social-dashboard social-ink-command social-sumi-e" aria-label="MaVoid social operations dashboard">
      <header className="social-hero social-ink-hero">
        <div className="social-hero-copy">
          <p className="social-eyebrow kana-line">コンテンツ運用</p>
          <h2>Social operations command room</h2>
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
        <div className="social-toolbar-group social-toolbar-group--primary" aria-label="Primary workflow actions">
          <span className="social-toolbar-group-label">Primary workflow</span>
          <button disabled={Boolean(busyAction)} onClick={() => window.confirm("Run the 08:00 creator now? This can create new post artifacts.") && automation("run_creator")} title={busyAction ? "Another dashboard action is running." : "Generate the next 08:00 creator artifacts now."} type="button"><Bot aria-hidden="true" size={16} /> Generate</button>
          <button disabled={!selectedPostHasSafeMedia || Boolean(busyAction)} onClick={validateSelectedMedia} title={validateActionTitle} type="button"><ShieldCheck aria-hidden="true" size={16} /> {validateActionLabel}</button>
          {overview?.automation.monitorEnabled ? (
            <button disabled={Boolean(busyAction)} onClick={() => automation("pause_monitor")} title={busyAction ? "Another dashboard action is running." : "Pause the publishing monitor automation."} type="button">
              <CalendarClock aria-hidden="true" size={16} /> Pause monitor
            </button>
          ) : (
            <button disabled={Boolean(busyAction)} onClick={() => automation("resume_monitor")} title={busyAction ? "Another dashboard action is running." : "Resume the publishing monitor automation."} type="button">
              <CalendarClock aria-hidden="true" size={16} /> Resume monitor
            </button>
          )}
        </div>
        <details className="social-toolbar-more">
          <summary>More</summary>
          <div className="social-toolbar-more-grid">
            <div className="social-toolbar-group" aria-label="Review and asset actions">
              <span className="social-toolbar-group-label">Review + assets</span>
              <button onClick={() => scrollToSection(mediaRef, "social-media")} title="Jump to the selected post design previews." type="button"><ExternalLink aria-hidden="true" size={16} /> Preview</button>
              <button disabled={!selectedPostHasSafeMedia || Boolean(busyAction)} onClick={validateSelectedMedia} title={validateActionTitle} type="button"><ShieldCheck aria-hidden="true" size={16} /> {selectedPostNeedsProviderVerification ? "Verify provider" : "Media"}</button>
              {selectedPost?.review?.reportPath ? <button onClick={() => void openResource(selectedPost.review?.reportPath)} title="Open the selected post review approval report." type="button"><ExternalLink aria-hidden="true" size={16} /> Approvals</button> : <span className="social-latest-report-metadata">Approvals: {safePathLabel(selectedPost?.review?.reportPath)}</span>}
            </div>
            <div className="social-toolbar-group" aria-label="Utility actions">
              <span className="social-toolbar-group-label">Utilities</span>
              <button disabled={loadState === "loading"} onClick={refresh} title={loadState === "loading" ? "Refreshing local social state." : "Reload posts, automation, and provider read-back from local state."} type="button"><RefreshCw aria-hidden="true" size={16} /> Refresh</button>
              <button disabled={Boolean(busyAction)} onClick={() => automation(overview?.automation.creatorEnabled ? "pause_creator" : "resume_creator")} title={busyAction ? "Another dashboard action is running." : overview?.automation.creatorEnabled ? "Pause the creator automation." : "Resume the creator automation."} type="button">
                <CalendarClock aria-hidden="true" size={16} /> {overview?.automation.creatorEnabled ? "Pause creator" : "Resume creator"}
              </button>
              <button disabled={busyAction === "health"} onClick={runHealthCheck} type="button" title={overview?.bufferHealth.rateLimited ? "Provider is cooling down; this performs one intentional health read-back." : "Run the real provider health check now."}><ExternalLink aria-hidden="true" size={16} /> Check provider</button>
              {overview?.latestReportPath ? <button onClick={() => void openResource(overview.latestReportPath)} title="Open the latest local report artifact." type="button"><ExternalLink aria-hidden="true" size={16} /> Latest report</button> : <span className="social-latest-report-metadata">Latest report: {safePathLabel(overview?.latestReportPath)}</span>}
            </div>
          </div>
        </details>
      </div>

      <section className="social-panel social-schedule-calendar" aria-label="Week schedule" id="social-calendar" ref={scheduleRef}>
        <div className="social-panel-heading"><span>Week schedule</span><strong>{posts.length}</strong><small>Loaded 7-day schedule from the first planned post in local state.</small></div>
        <div className="social-calendar-grid">
          {weekSchedule.map((day) => (
            <article className="social-calendar-day" key={day.date}>
              <div className="social-calendar-day-header">
                <time dateTime={day.date}>{day.label}</time>
                <button aria-expanded={quickMenuDate === day.date} aria-haspopup="dialog" className="social-calendar-quick-button" onClick={() => setQuickMenuDate((current) => current === day.date ? null : day.date)} title={`Generate or schedule content for ${day.label}`} type="button"><Wand2 size={14} aria-hidden="true" /> Generate</button>
              </div>
              {quickMenuDate === day.date ? (
                <div aria-label={`Generate content for ${day.label}`} className="social-calendar-type-menu" role="dialog">
                  <div className="social-calendar-type-menu-header">
                    <div>
                      <span>Quick generation</span>
                      <strong>{day.label}</strong>
                    </div>
                    <button aria-label={`Close generation menu for ${day.label}`} className="social-calendar-type-menu-close" onClick={() => setQuickMenuDate(null)} title="Close generation menu" type="button"><X size={14} aria-hidden="true" /></button>
                  </div>
                  <p className="social-calendar-type-menu-note">Choose the format Zoid should draft for this slot.</p>
                  <div className="social-calendar-type-menu-options">
                    {contentTypes.map((type) => (
                      <button className="social-calendar-type-option" disabled={Boolean(busyAction)} key={type.id} onClick={() => void generateForDate(day.date, type)} title={type.description} type="button">
                        <strong>{type.label}</strong><small>{type.description}</small>
                      </button>
                    ))}
                  </div>
                </div>
              ) : null}
              {day.posts.length ? day.posts.map((post) => (
                <button className={post.id === selectedPost?.id ? "social-calendar-card active" : "social-calendar-card"} key={post.id} onClick={() => setSelectedPostId(post.id)} type="button">
                  <span>{slotTimes[post.slotType]} · {displayStatus(post.status)}</span>
                  <strong>{neutralizeProviderCopy(post.title)}</strong>
                  <small className="social-platform-icons">{platformIconList(post)}<em>{post.mediaAssets.length} creative{post.mediaAssets.length === 1 ? "" : "s"}</em></small>
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
              </nav>

              <section className="social-detail-summary social-detail-hero-card" id="social-summary" ref={summaryRef}>
                <div className="social-detail-heading"><span>{selectedPost.postDate} · {selectedPost.slotType.replace(/_/g, " ")}</span><h3>{neutralizeProviderCopy(selectedPost.title)}</h3><small>{neutralizeProviderCopy(selectedPost.topicOrNewsItem)}</small></div>
                <p className="social-caption">{neutralizeProviderCopy(selectedPost.caption)}</p>
                <div className="social-gate" role="status"><AlertTriangle aria-hidden="true" size={18} /> {retryState.ok ? "Ready for scheduling after /impecabble content review approval and media validation." : hasUnverifiedProviderState(selectedPost) ? "Verify provider state before retrying schedule actions. This protects already-created provider posts from duplicate scheduling." : `Schedule or retry is locked because ${retryState.reason}. Check provider state, validate media, then retry after the blocker clears.`}</div>
              </section>

              <section className="social-detail-section social-media-strip" aria-label="Design previews" id="social-media" ref={mediaRef}>
                <div className="social-panel-heading"><span>Design preview</span><strong>{selectedPost.mediaAssets.length}</strong></div>
                <div className="social-media-gallery social-media-gallery--compact">
                  {selectedPost.mediaAssets.map((asset, index) => {
                    const source = canOpenExternal(asset.publicUrl) ? asset.publicUrl : null;
                    return (
                      <article className="social-media-card" key={`${asset.path}-${asset.publicUrl ?? "local"}`}>
                        <button className="social-media-preview social-media-thumb" disabled={!source} onClick={() => source && openPreview(index)} title={source ? "Preview design" : "Preview requires a public HTTPS image URL."} type="button">
                          {source ? <img alt={`${neutralizeValue(selectedPost.title)} design ${index + 1}`} loading="lazy" src={source} /> : <div className="social-media-fallback">Local-only asset metadata; preview requires a validated public HTTPS image URL.</div>}
                        </button>
                        <span>Design {index + 1}</span>
                        <strong>{asset.contentType ?? "image"} · {asset.width ?? "?"}×{asset.height ?? "?"} · {formatBytes(asset.bytes)}</strong>
                        {assetStates[asset.path] ? <small className={`social-asset-state social-asset-state--${assetStates[asset.path]}`}>Redesign {assetStates[asset.path]}</small> : null}
                        {asset.temporary ? <small className="social-warning">Temporary media host — replace with durable owned media before production scheduling.</small> : null}
                        <div className="social-action-row social-action-row--icons">
                          <button aria-label={`Open media URL for design ${index + 1}`} disabled={!source} onClick={() => void openExternal(asset.publicUrl)} title={source ? "Open media URL" : "No public HTTPS media URL available."} type="button"><ExternalLink size={15} aria-hidden="true" /></button>
                          <button aria-label={`${selectedPostNeedsProviderVerification ? "Verify provider state before media validation" : "Validate media"} for design ${index + 1}`} disabled={!source || Boolean(busyAction)} onClick={() => validateMediaUrl(asset.publicUrl)} title={source ? selectedPostNeedsProviderVerification ? "Verify provider read-back before validating this media URL." : "Validate this media URL" : "No public HTTPS media URL available."} type="button"><ShieldCheck size={15} aria-hidden="true" /></button>
                          <button aria-label={`Retry design ${index + 1}`} disabled={Boolean(busyAction)} onClick={() => { setRedesignTarget({ postId: selectedPost.id, mediaPath: asset.path, label: `Design ${index + 1}` }); setRedesignNotes(""); }} title="Ask the designer agent to retry this design" type="button"><RotateCcw size={15} aria-hidden="true" /></button>
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
                    {selectedPost.bufferPosts.map((item) => {
                      const verifiedState = item.bufferId || item.readBackVerifiedAt ? displayStatus(item.state) : `Local pending: ${displayStatus(item.state)}`;
                      return (
                        <article className="social-platform-card" key={`${item.platform}-${item.bufferId ?? item.channelId ?? "pending"}`}>
                          <span>{platformLabels[item.platform] ?? item.platform}</span><strong>{safeLabel(item.channelDisplayName, "Pending channel")}</strong>
                          <small>State: {verifiedState}</small>
                          {item.scheduledAtLocal || item.scheduledAtUtc ? <small>Scheduled: {readableDateTime(item.scheduledAtLocal ?? item.scheduledAtUtc)}</small> : null}
                          {item.lastErrorCode ? <small className="social-warning">{item.lastErrorCode}: {neutralizeProviderCopy(item.lastErrorMessage)}</small> : null}
                          {item.publishedUrl ? <button onClick={() => void openExternal(item.publishedUrl)} type="button">Open published URL</button> : null}
                        </article>
                      );
                    })}
                    {selectedPost.bufferPosts.length === 0 ? <p className="social-empty">No provider platform records yet.</p> : null}
                  </div>
                </article>
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
        </aside>
      </div>
      </section>

      {activePreview ? renderScreenOverlay((
        <div className="social-preview-backdrop social-preview-backdrop--full-app" onClick={() => setPreviewAsset(null)} role="presentation">
          <div className="social-preview-lightbox social-preview-lightbox--full-app" onClick={(event) => event.stopPropagation()} role="dialog" aria-modal="true" aria-label="Design preview">
            <div className="social-preview-topbar">
              <div className="social-panel-heading"><span>Design preview</span><strong>{selectedPost ? neutralizeValue(selectedPost.title) : "Selected design"}</strong><small>{displayStatus(selectedPost?.status)} · Public preview {activePreviewIndex + 1} of {selectedPreviewAssets.length}</small></div>
              <button aria-label="Close preview" className="social-preview-close" onClick={() => setPreviewAsset(null)} type="button"><X size={18} aria-hidden="true" /></button>
            </div>
            <div className="social-preview-stage">
              {hasPreviewNavigation ? <button aria-label="Previous design preview" className="social-preview-nav social-preview-nav--prev" onClick={() => movePreview(-1)} type="button">‹</button> : null}
              <img alt={activePreview.alt} src={activePreview.source ?? ""} />
              {hasPreviewNavigation ? <button aria-label="Next design preview" className="social-preview-nav social-preview-nav--next" onClick={() => movePreview(1)} type="button">›</button> : null}
            </div>
            <div className="social-preview-actionbar social-action-row">
              {selectedPost?.review?.reportPath ? <button onClick={() => void openResource(selectedPost.review?.reportPath)} title="Open the selected post review approval report." type="button"><CheckCircle2 size={15} aria-hidden="true" /> Open approval report</button> : <button disabled title="No review approval report is linked for this post." type="button"><CheckCircle2 size={15} aria-hidden="true" /> Approval report unavailable</button>}
              <button disabled={Boolean(busyAction)} onClick={replacePreviewMedia} title="Ask the designer agent to retry/replace this media asset." type="button"><RotateCcw size={15} aria-hidden="true" /> Replace media</button>
              <button onClick={() => void openExternal(activePreview.source)} title="Open this media URL using the system handler." type="button"><ExternalLink size={15} aria-hidden="true" /> Open media URL</button>
            </div>
          </div>
        </div>
      )) : null}

      {redesignTarget ? renderScreenOverlay((
        <div className="social-preview-backdrop" onClick={() => setRedesignTarget(null)} role="presentation">
          <form className="social-redesign-modal" onClick={(event) => event.stopPropagation()} onSubmit={(event) => { event.preventDefault(); void submitRedesign(); }}>
            <div className="social-panel-heading"><span>Retry design</span><strong>{redesignTarget.label}</strong><small>Optional feedback is passed to the background designer agent.</small></div>
            <textarea autoFocus onChange={(event) => setRedesignNotes(event.target.value)} placeholder="Optional: what should change?" value={redesignNotes} />
            <div className="social-action-row">
              <button disabled={Boolean(busyAction)} type="submit"><RotateCcw size={15} aria-hidden="true" /> Start redesign</button>
              <button onClick={() => setRedesignTarget(null)} type="button">Cancel</button>
            </div>
          </form>
        </div>
      )) : null}

      {providerVerificationTarget ? renderScreenOverlay((
        <div className="social-preview-backdrop" onClick={() => setProviderVerificationTarget(null)} role="presentation">
          <div className="social-redesign-modal social-provider-verification-modal" onClick={(event) => event.stopPropagation()} role="dialog" aria-modal="true" aria-label="Verify provider state">
            <div className="social-panel-heading"><span>Verify provider state</span><strong>Re-check media after provider read-back</strong><small>This post already has scheduled or posted provider records. Zoid needs a fresh provider verification before retrying schedule actions so it does not create duplicates.</small></div>
            <p className="social-action-note">Run one provider health/read-back check now, then retry the media validation when the local state shows those provider records as verified.</p>
            <div className="social-action-row">
              <button disabled={Boolean(busyAction)} onClick={() => void verifyProviderState()} type="button"><ShieldCheck size={15} aria-hidden="true" /> Verify now</button>
              {providerVerificationTarget.detailsResource ? <button onClick={() => void openResource(providerVerificationTarget.detailsResource)} type="button"><ExternalLink size={15} aria-hidden="true" /> Open provider details</button> : <button disabled title="No provider report, published URL, or review report is linked for this post." type="button">Open provider details unavailable</button>}
              <button onClick={() => setProviderVerificationTarget(null)} type="button">Cancel</button>
            </div>
          </div>
        </div>
      )) : null}
    </>
  );
}
