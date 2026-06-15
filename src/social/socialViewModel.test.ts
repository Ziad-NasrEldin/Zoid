import { strict as assert } from "node:assert";
import {
  deriveMavoidSocialStatusLabel,
  summarizeMavoidSocialPosts,
  canRetryBufferSchedule,
  type MavoidSocialOverview,
  type MavoidSocialPost,
} from "./socialViewModel";

const overview: MavoidSocialOverview = {
  workspacePath: "/Users/ziadnasreldin/MaVoid/social-automation-buffer",
  overallStatus: "rate_limited",
  activeBlocker: "Buffer HTTP 429 RATE_LIMIT_EXCEEDED window=24h",
  bufferEndpoint: "https://api.buffer.com/graphql",
  bufferHealth: {
    ok: false,
    httpStatus: 429,
    rateLimited: true,
    rateLimitWindow: "24h",
    credentialsPresent: { bufferAccessToken: true, bufferOrganizationId: true },
    lastCheckedAt: "2026-06-09T12:40:00Z",
    message: "Too many requests from this client.",
  },
  automation: {
    creatorJobId: "12fd35ec77e2",
    creatorEnabled: true,
    creatorState: "scheduled",
    creatorNextRunAt: "2026-06-10T08:00:00+03:00",
    monitorJobId: "9562e7cb93b6",
    monitorEnabled: false,
    monitorState: "paused",
    monitorNextRunAt: null,
    cooldownJobId: "a0caa25a4cf7",
    cooldownNextRunAt: "2026-06-10T16:45:00+03:00",
  },
  counts: { totalPosts: 1, needsReview: 0, readyToSchedule: 0, scheduledVerified: 0, posted: 0, blocked: 1 },
  nextSlots: [],
  latestReportPath: "/tmp/report.md",
  updatedAt: "2026-06-09T12:40:00Z",
};

const approvedPost: MavoidSocialPost = {
  id: "proof-post-2026-06-09",
  postDate: "2026-06-09",
  slotType: "manual_campaign",
  title: "Buffer pipeline proof",
  topicOrNewsItem: "Buffer migration proof",
  caption: "Buffer pipeline proof",
  platforms: ["instagram", "facebook", "linkedin"],
  status: "rate_limited",
  review: { verdict: "APPROVED", reviewer: "independent reviewer", reportPath: "/tmp/review.md", requiredFixes: [], approvedAt: null },
  mediaAssets: [{ path: "/tmp/proof.png", publicUrl: "https://files.catbox.moe/9tix1y.png", contentType: "image/png", bytes: 99945, width: 1080, height: 1350, validatedAt: null, provider: "catbox", temporary: true, validationStatus: "valid" }],
  bufferPosts: [{ bufferId: null, platform: "instagram", channelId: null, channelDisplayName: null, scheduledAtUtc: null, scheduledAtLocal: null, state: "not_created", readBackVerifiedAt: null, publishedUrl: null, lastErrorCode: "RATE_LIMIT_EXCEEDED", lastErrorMessage: "24h" }],
  reports: [{ label: "Review report", path: "/tmp/review.md", kind: "review", createdAt: null }],
  events: [],
};

assert.equal(deriveMavoidSocialStatusLabel(overview), "Rate-limited · 24h cooldown", "rate limited overview should expose cooldown language");
assert.equal(canRetryBufferSchedule(overview, approvedPost).ok, false, "retry must be blocked while Buffer is rate-limited");
assert.match(canRetryBufferSchedule(overview, approvedPost).reason, /rate-limited/i, "retry blocker should mention rate limit");

const healthyOverview = { ...overview, overallStatus: "ready_to_schedule" as const, bufferHealth: { ...overview.bufferHealth, ok: true, httpStatus: 200, rateLimited: false, rateLimitWindow: null } };
assert.equal(canRetryBufferSchedule(healthyOverview, approvedPost).ok, true, "approved post with valid media can retry when Buffer is healthy");

const verifiedProviderPost = {
  ...approvedPost,
  bufferPosts: [{ ...approvedPost.bufferPosts[0], bufferId: "buf-verified-1", state: "scheduled" as const, readBackVerifiedAt: "2026-06-09T13:00:00Z" }],
};
const verifiedProviderRetry = canRetryBufferSchedule(healthyOverview, verifiedProviderPost);
assert.equal(verifiedProviderRetry.ok, false, "retry must stay blocked when provider state already exists, even after read-back verification");
assert.match(verifiedProviderRetry.reason, /duplicate provider posts/i, "provider-state blocker should explain duplicate prevention");

const unapproved = { ...approvedPost, review: { ...approvedPost.review!, verdict: "REQUEST_CHANGES" as const }, status: "request_changes" as const };
assert.equal(canRetryBufferSchedule(healthyOverview, unapproved).ok, false, "retry must be blocked without reviewer approval");

const counts = summarizeMavoidSocialPosts([approvedPost, unapproved]);
assert.equal(counts.totalPosts, 2, "summary counts total posts");
assert.equal(counts.needsReview, 1, "summary counts request-changes posts as needing review");
assert.equal(counts.blocked, 1, "summary counts rate-limited post as blocked");

console.log("social view model tests passed");
