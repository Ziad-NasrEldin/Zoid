import { AlertTriangle, Bot, CalendarClock, CheckCircle2, ExternalLink, RefreshCw, ShieldCheck } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import { getMavoidSocialOverview, listMavoidSocialPosts, manageMavoidSocialAutomation, runMavoidBufferHealthCheck } from "./socialClient";
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

  return (
    <section className="social-dashboard social-ink-command social-sumi-e" aria-label="MaVoid social operations dashboard">
      <header className="social-hero social-ink-hero">
        <div className="social-hero-copy">
          <p className="social-eyebrow kana-line">マヴォイド・バッファ自動投稿</p>
          <h2>Ink control room</h2>
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
        <button disabled={busyAction === "health"} onClick={runHealthCheck} type="button"><ExternalLink aria-hidden="true" size={16} /> Check Buffer API</button>
        <button disabled={Boolean(busyAction)} onClick={() => automation("run_creator")} type="button"><Bot aria-hidden="true" size={16} /> Run 8:00 creator</button>
        <button disabled={Boolean(busyAction)} onClick={() => automation(overview?.automation.creatorEnabled ? "pause_creator" : "resume_creator")} type="button">
          <CalendarClock aria-hidden="true" size={16} /> {overview?.automation.creatorEnabled ? "Pause creator" : "Resume creator"}
        </button>
      </div>

      <div className="social-grid">
        <aside className="social-panel social-post-list" aria-label="Posts">
          <div className="social-panel-heading"><span>Publishing queue</span><strong>{overview?.counts.totalPosts ?? posts.length}</strong><small>{overview?.workspacePath ?? "Loading workspace"}</small></div>
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
              <div className="social-detail-heading"><span>{selectedPost.slotType.replace(/_/g, " ")}</span><h3>{neutralizeProviderCopy(selectedPost.title)}</h3><small>{neutralizeProviderCopy(selectedPost.topicOrNewsItem)}</small></div>
              <p className="social-caption">{neutralizeProviderCopy(selectedPost.caption)}</p>
              <div className="social-gate" role="status"><AlertTriangle aria-hidden="true" size={18} /> {retryState.reason}</div>
              <div className="social-media-list">
                {selectedPost.mediaAssets.map((asset) => (
                  <article className="social-media-card" key={`${asset.path}-${asset.publicUrl ?? "local"}`}>
                    <span>{asset.validationStatus}</span>
                    <strong>{asset.publicUrl ?? asset.path}</strong>
                    <small>{asset.contentType ?? "unknown type"} · {asset.width ?? "?"}×{asset.height ?? "?"}</small>
                  </article>
                ))}
              </div>
            </>
          ) : <p>No post selected.</p>}
        </main>

        <aside className="social-panel social-automation-panel" aria-label="Automation state">
          <div className="social-panel-heading"><span>Hermes cron</span><strong>{overview?.automation.creatorState ?? "unknown"}</strong><small>creator {overview?.automation.creatorJobId ?? "—"}</small></div>
          <dl className="social-kv"><dt>Creator next run</dt><dd>{overview?.automation.creatorNextRunAt ?? "—"}</dd><dt>Monitor state</dt><dd>{overview?.automation.monitorState ?? "unknown"}</dd><dt>Cooldown next run</dt><dd>{overview?.automation.cooldownNextRunAt ?? "—"}</dd><dt>Provider endpoint</dt><dd>{overview?.bufferEndpoint ? "configured" : "—"}</dd></dl>
        </aside>
      </div>
    </section>
  );
}
