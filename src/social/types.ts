export type SocialOverallStatus =
  | "healthy"
  | "rate_limited"
  | "needs_review"
  | "media_blocked"
  | "ready_to_schedule"
  | "scheduled_verified"
  | "posted"
  | "failed_closed"
  | "paused"
  | "unknown";

export type SocialPostStatus =
  | "planned"
  | "generating"
  | "rendered"
  | "review_requested"
  | "request_changes"
  | "approved"
  | "media_hosted"
  | "buffer_pending"
  | "scheduled_unverified"
  | "scheduled_verified"
  | "posted"
  | "rate_limited"
  | "media_blocked"
  | "buffer_failed"
  | "failed_closed"
  | "manually_resolved";

export type SocialPlatform = "instagram" | "facebook" | "linkedin" | "x";

export type MavoidSocialOverview = {
  workspacePath: string;
  overallStatus: SocialOverallStatus;
  activeBlocker: string | null;
  bufferEndpoint: string;
  bufferHealth: MavoidBufferHealth;
  automation: MavoidAutomationStatus;
  counts: MavoidSocialCounts;
  nextSlots: MavoidSocialSlot[];
  latestReportPath: string | null;
  updatedAt: string;
};

export type MavoidSocialCounts = {
  totalPosts: number;
  needsReview: number;
  readyToSchedule: number;
  scheduledVerified: number;
  posted: number;
  blocked: number;
};

export type MavoidBufferHealth = {
  ok: boolean;
  httpStatus: number | null;
  rateLimited: boolean;
  rateLimitWindow: string | null;
  credentialsPresent: {
    bufferAccessToken: boolean;
    bufferOrganizationId: boolean;
  };
  lastCheckedAt: string | null;
  message: string | null;
};

export type MavoidAutomationStatus = {
  creatorJobId: string;
  creatorEnabled: boolean;
  creatorState: string;
  creatorNextRunAt: string | null;
  monitorJobId: string;
  monitorEnabled: boolean;
  monitorState: string;
  monitorNextRunAt: string | null;
  cooldownJobId: string | null;
  cooldownNextRunAt: string | null;
};

export type MavoidSocialSlot = {
  id: string;
  date: string;
  slotType: "ai_intel" | "enterprise_carousel" | "manual_campaign";
  localPublishTime: string;
  utcPublishTime: string | null;
  status: SocialPostStatus;
};

export type MavoidSocialPost = {
  id: string;
  postDate: string;
  slotType: MavoidSocialSlot["slotType"];
  title: string;
  topicOrNewsItem: string;
  caption: string;
  platforms: SocialPlatform[];
  status: SocialPostStatus;
  review: MavoidReviewReport | null;
  mediaAssets: MavoidMediaAsset[];
  bufferPosts: MavoidBufferPost[];
  reports: MavoidReportRef[];
  events: MavoidSocialEvent[];
};

export type MavoidMediaAsset = {
  path: string;
  publicUrl: string | null;
  contentType: string | null;
  bytes: number | null;
  width: number | null;
  height: number | null;
  validatedAt: string | null;
  provider: string | null;
  temporary: boolean;
  validationStatus: "valid" | "invalid" | "unchecked";
};

export type MavoidReviewReport = {
  verdict: "APPROVED" | "REQUEST_CHANGES" | "MISSING";
  reviewer: string | null;
  reportPath: string | null;
  requiredFixes: string[];
  approvedAt: string | null;
};

export type MavoidBufferPost = {
  bufferId: string | null;
  platform: SocialPlatform;
  channelId: string | null;
  channelDisplayName: string | null;
  scheduledAtUtc: string | null;
  scheduledAtLocal: string | null;
  state: "not_created" | "scheduled" | "posted" | "failed" | "unknown";
  readBackVerifiedAt: string | null;
  publishedUrl: string | null;
  lastErrorCode: string | null;
  lastErrorMessage: string | null;
};

export type MavoidReportRef = {
  label: string;
  path: string;
  kind: "generation" | "review" | "buffer" | "monitor" | "handoff" | "other";
  createdAt: string | null;
};

export type MavoidSocialEvent = {
  timestamp: string;
  actor: "zoid" | "hermes" | "buffer" | "operator";
  eventType: string;
  message: string;
  severity: "info" | "warning" | "error" | "success";
  evidencePath: string | null;
};

export type MavoidMediaValidation = {
  url: string;
  ok: boolean;
  httpStatus: number | null;
  contentType: string | null;
  bytes: number | null;
  message: string;
};
