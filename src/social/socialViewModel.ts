import type { MavoidSocialCounts, MavoidSocialOverview, MavoidSocialPost } from "./types";

export type { MavoidSocialOverview, MavoidSocialPost } from "./types";

export function summarizeMavoidSocialPosts(posts: MavoidSocialPost[]): MavoidSocialCounts {
  return posts.reduce<MavoidSocialCounts>(
    (summary, post) => {
      summary.totalPosts += 1;
      if (post.status === "review_requested" || post.status === "request_changes" || post.review?.verdict === "REQUEST_CHANGES") summary.needsReview += 1;
      if (post.status === "approved" || post.status === "media_hosted") summary.readyToSchedule += 1;
      if (post.status === "scheduled_verified") summary.scheduledVerified += 1;
      if (post.status === "posted") summary.posted += 1;
      if (["rate_limited", "media_blocked", "buffer_failed", "failed_closed"].includes(post.status)) summary.blocked += 1;
      return summary;
    },
    { totalPosts: 0, needsReview: 0, readyToSchedule: 0, scheduledVerified: 0, posted: 0, blocked: 0 },
  );
}

export function deriveMavoidSocialStatusLabel(overview: MavoidSocialOverview): string {
  if (overview.bufferHealth.rateLimited) {
    return `Rate-limited · ${overview.bufferHealth.rateLimitWindow ?? "cooldown"} cooldown`;
  }
  const labels: Record<MavoidSocialOverview["overallStatus"], string> = {
    healthy: "Healthy",
    rate_limited: "Rate-limited",
    needs_review: "Needs review",
    media_blocked: "Media blocked",
    ready_to_schedule: "Ready to schedule",
    scheduled_verified: "Scheduled verified",
    posted: "Posted",
    failed_closed: "Failed closed",
    paused: "Paused",
    unknown: "Unknown",
  };
  return labels[overview.overallStatus];
}

export function canRetryBufferSchedule(overview: MavoidSocialOverview, post: MavoidSocialPost): { ok: boolean; reason: string } {
  if (overview.bufferHealth.rateLimited) {
    return { ok: false, reason: `Provider is rate-limited${overview.bufferHealth.rateLimitWindow ? ` for ${overview.bufferHealth.rateLimitWindow}` : ""}. Wait for cooldown before retrying.` };
  }
  if (!overview.bufferHealth.ok) return { ok: false, reason: "Provider health is not verified." };
  if (post.review?.verdict !== "APPROVED") return { ok: false, reason: "Reviewer approval is required before scheduling." };
  if (!post.mediaAssets.some((asset) => asset.publicUrl && asset.validationStatus === "valid")) {
    return { ok: false, reason: "A valid public direct media URL is required before scheduling." };
  }
  if (post.bufferPosts.some((bufferPost) => bufferPost.state === "scheduled" || bufferPost.state === "posted")) {
    return { ok: false, reason: "This post already has provider state. Verify read-back for status clarity, but do not retry scheduling because duplicate provider posts could be created." };
  }
  return { ok: true, reason: "Ready to retry scheduling." };
}

export function formatPlatformList(post: MavoidSocialPost): string {
  return post.platforms.map((platform) => platform[0].toUpperCase() + platform.slice(1)).join(" · ");
}
