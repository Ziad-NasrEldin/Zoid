export type SocialPlatform = "linkedin" | "instagram" | "tiktok" | "x";
export type ContentPlanStatus = "active" | "archived";
export type ContentPieceStatus = "draft" | "review_ready" | "approved" | "scheduled" | "blocked" | "archived";
export type ReviewGateStatus = "pending" | "approved" | "rejected";
export type ScheduleStatus = "intent" | "blocked" | "failed" | "cancelled";
export type IntegrationState = "not_configured" | "needs_permission" | "connected" | "error" | "disabled_by_policy";
export type SocialActionType = "upload" | "schedule" | "publish";
export type VerificationOutcome = "passed" | "blocked" | "failed" | "manual";

export type ContentPlanRecord = {
  id: string;
  title: string;
  pillar: string;
  status: ContentPlanStatus;
  ownerActorType: "human" | "agent";
};

export type ContentPieceRecord = {
  id: string;
  planId: string;
  title: string;
  bodyMarkdown: string;
  status: ContentPieceStatus;
  platforms: SocialPlatform[];
  requiredGate: "none" | "specialist_review";
};

export type MediaAssetRecord = {
  id: string;
  pieceId: string;
  assetKind: "image" | "video" | "document" | "link";
  storageRef: string;
  altText: string;
};

export type ContentReviewGateRecord = {
  id: string;
  pieceId: string;
  gateType: "specialist_review";
  status: ReviewGateStatus;
  evidenceSummary: string;
};

export type ContentScheduleRecord = {
  id: string;
  pieceId: string;
  platform: SocialPlatform;
  scheduledFor: string;
  status: ScheduleStatus;
  confirmationId?: string | null;
};

export type ContentVerificationRecord = {
  id: string;
  pieceId?: string | null;
  scheduleId?: string | null;
  platform: SocialPlatform | "omnisocials";
  actionType: SocialActionType | "review" | "validation";
  outcome: VerificationOutcome;
  providerStatus?: IntegrationState | null;
  failureReport?: string | null;
};

export type OmniSocialsStatusRecord = {
  state: IntegrationState;
  provider: "omnisocials";
  platform: "omnisocials";
  credentialRef?: string | null;
  statusNote: string;
};

export type ContentWorkspaceState = {
  plans: ContentPlanRecord[];
  pieces: ContentPieceRecord[];
  mediaAssets: MediaAssetRecord[];
  reviewGates: ContentReviewGateRecord[];
  schedules: ContentScheduleRecord[];
  verifications: ContentVerificationRecord[];
  omnisocials: OmniSocialsStatusRecord;
  selectedPieceId: string;
};

export const CONTENT_WORKSPACE_STORAGE_KEY = "zoid25:content-workspace-state";

export function createDefaultContentWorkspaceState(): ContentWorkspaceState {
  return {
    plans: [
      { id: "plan-mavoid-daily", title: "MaVoid Daily Authority Rhythm", pillar: "Enterprise automation + operational systems", status: "active", ownerActorType: "human" },
    ],
    pieces: [
      {
        id: "piece-linkedin-buffer-alt",
        planId: "plan-mavoid-daily",
        title: "Why Buffer is not enough for enterprise ops content",
        bodyMarkdown: "Draft-first post. Human review required before any scheduling intent. External publishing is disabled until OmniSocials is configured and reviewed.",
        status: "review_ready",
        platforms: ["linkedin", "instagram"],
        requiredGate: "specialist_review",
      },
    ],
    mediaAssets: [
      { id: "asset-diagram-ref", pieceId: "piece-linkedin-buffer-alt", assetKind: "link", storageRef: "local://evidence/buffer-alternative-diagram", altText: "Draft visual reference for Buffer alternative workflow" },
    ],
    reviewGates: [
      { id: "gate-specialist-1", pieceId: "piece-linkedin-buffer-alt", gateType: "specialist_review", status: "pending", evidenceSummary: "Specialist designer/reviewer approval has not been recorded yet." },
    ],
    schedules: [],
    verifications: [
      { id: "verify-default-block", pieceId: "piece-linkedin-buffer-alt", platform: "omnisocials", actionType: "publish", outcome: "blocked", providerStatus: "not_configured", failureReport: "Default OmniSocials state is not_configured; Zoid does not call Buffer or any external social API." },
    ],
    omnisocials: {
      provider: "omnisocials",
      platform: "omnisocials",
      state: "not_configured",
      credentialRef: null,
      statusNote: "OmniSocials is the Buffer alternative integration surface. Credentials are server-side only; no external writes are enabled in this slice.",
    },
    selectedPieceId: "piece-linkedin-buffer-alt",
  };
}

export function parseContentWorkspaceState(value: string | null): ContentWorkspaceState {
  if (!value) return createDefaultContentWorkspaceState();
  try {
    const parsed = JSON.parse(value) as Partial<ContentWorkspaceState>;
    const defaults = createDefaultContentWorkspaceState();
    return {
      ...defaults,
      ...parsed,
      plans: Array.isArray(parsed.plans) ? parsed.plans.filter(isPlan) : defaults.plans,
      pieces: Array.isArray(parsed.pieces) ? parsed.pieces.filter(isPiece) : defaults.pieces,
      mediaAssets: Array.isArray(parsed.mediaAssets) ? parsed.mediaAssets.filter(isMediaAsset) : defaults.mediaAssets,
      reviewGates: Array.isArray(parsed.reviewGates) ? parsed.reviewGates.filter(isReviewGate) : defaults.reviewGates,
      schedules: Array.isArray(parsed.schedules) ? parsed.schedules.filter(isSchedule) : defaults.schedules,
      verifications: Array.isArray(parsed.verifications) ? parsed.verifications.filter(isVerification) : defaults.verifications,
      omnisocials: isOmniSocialsStatus(parsed.omnisocials) ? parsed.omnisocials : defaults.omnisocials,
      selectedPieceId: typeof parsed.selectedPieceId === "string" ? parsed.selectedPieceId : defaults.selectedPieceId,
    };
  } catch {
    return createDefaultContentWorkspaceState();
  }
}

function isPlan(value: unknown): value is ContentPlanRecord {
  const item = value as Partial<ContentPlanRecord>;
  return Boolean(item && typeof item.id === "string" && typeof item.title === "string" && item.status === "active");
}
function isPiece(value: unknown): value is ContentPieceRecord {
  const item = value as Partial<ContentPieceRecord>;
  return Boolean(item && typeof item.id === "string" && typeof item.title === "string" && Array.isArray(item.platforms));
}
function isMediaAsset(value: unknown): value is MediaAssetRecord {
  const item = value as Partial<MediaAssetRecord>;
  return Boolean(item && typeof item.id === "string" && typeof item.pieceId === "string" && typeof item.storageRef === "string" && item.storageRef.trim().length > 0);
}
function isReviewGate(value: unknown): value is ContentReviewGateRecord {
  const item = value as Partial<ContentReviewGateRecord>;
  return Boolean(item && typeof item.id === "string" && typeof item.pieceId === "string" && ["pending", "approved", "rejected"].includes(String(item.status)));
}
function isSchedule(value: unknown): value is ContentScheduleRecord {
  const item = value as Partial<ContentScheduleRecord>;
  return Boolean(item && typeof item.id === "string" && typeof item.pieceId === "string" && typeof item.scheduledFor === "string");
}
function isVerification(value: unknown): value is ContentVerificationRecord {
  const item = value as Partial<ContentVerificationRecord>;
  return Boolean(item && typeof item.id === "string" && typeof item.actionType === "string" && ["passed", "blocked", "failed", "manual"].includes(String(item.outcome)));
}
function isOmniSocialsStatus(value: unknown): value is OmniSocialsStatusRecord {
  const item = value as Partial<OmniSocialsStatusRecord>;
  return Boolean(item && item.provider === "omnisocials" && item.platform === "omnisocials" && typeof item.statusNote === "string");
}

export function pieceScheduleGateSummary(piece: ContentPieceRecord, gates: ContentReviewGateRecord[], schedules: ContentScheduleRecord[]): string {
  const requiredGateApproved = piece.requiredGate === "none" || gates.some((gate) => gate.pieceId === piece.id && gate.gateType === "specialist_review" && gate.status === "approved");
  const hasIntent = schedules.some((schedule) => schedule.pieceId === piece.id && schedule.status === "intent");
  if (hasIntent) return "Local schedule intent recorded — no Buffer/API publish implied.";
  if (!requiredGateApproved) return "Blocked until specialist review is approved.";
  return "Review gate passed; scheduling still needs human confirmation.";
}

export function validateScheduleIntent(piece: ContentPieceRecord, mediaAssets: MediaAssetRecord[], gates: ContentReviewGateRecord[], platform: SocialPlatform, confirmed: boolean): { ok: boolean; reason: string } {
  const needsMedia = platform === "instagram" || platform === "tiktok";
  const hasMedia = mediaAssets.some((asset) => asset.pieceId === piece.id && (asset.assetKind === "image" || asset.assetKind === "video"));
  if (needsMedia && !hasMedia) return { ok: false, reason: `${platform} requires an image or video asset reference before a local schedule intent.` };
  if (piece.requiredGate !== "none" && !gates.some((gate) => gate.pieceId === piece.id && gate.status === "approved")) return { ok: false, reason: "Specialist review gate must be approved before a schedule intent." };
  if (!confirmed) return { ok: false, reason: "Human confirmation is required before recording a schedule intent." };
  return { ok: true, reason: "Allowed as a local intent only; no external Buffer/OmniSocials write occurs." };
}

export function recordFailClosedSocialAction(state: ContentWorkspaceState, pieceId: string, platform: SocialPlatform, actionType: SocialActionType): ContentWorkspaceState {
  const nextRecord: ContentVerificationRecord = {
    id: `verify-${Date.now()}`,
    pieceId,
    platform,
    actionType,
    outcome: "blocked",
    providerStatus: state.omnisocials.state,
    failureReport: "Blocked: OmniSocials is not configured. Zoid did not call Buffer, OmniSocials, or any external publishing API.",
  };
  return { ...state, verifications: [nextRecord, ...state.verifications] };
}

export function approveSpecialistGate(state: ContentWorkspaceState, pieceId: string): ContentWorkspaceState {
  return {
    ...state,
    reviewGates: state.reviewGates.map((gate) => gate.pieceId === pieceId ? { ...gate, status: "approved", evidenceSummary: "Approved by specialist reviewer in local Zoid workflow." } : gate),
  };
}

export function createLocalScheduleIntent(state: ContentWorkspaceState, pieceId: string, platform: SocialPlatform, confirmed: boolean): ContentWorkspaceState {
  const piece = state.pieces.find((item) => item.id === pieceId);
  if (!piece) return state;
  const validation = validateScheduleIntent(piece, state.mediaAssets, state.reviewGates, platform, confirmed);
  if (!validation.ok) {
    return {
      ...state,
      verifications: [{ id: `verify-${Date.now()}`, pieceId, platform, actionType: "schedule", outcome: "blocked", providerStatus: state.omnisocials.state, failureReport: validation.reason }, ...state.verifications],
    };
  }
  const schedule: ContentScheduleRecord = {
    id: `schedule-${Date.now()}`,
    pieceId,
    platform,
    scheduledFor: new Date(Date.now() + 24 * 60 * 60 * 1000).toISOString(),
    status: "intent",
    confirmationId: `human-confirmed-${Date.now()}`,
  };
  return {
    ...state,
    schedules: [schedule, ...state.schedules],
    pieces: state.pieces.map((item) => item.id === pieceId ? { ...item, status: "scheduled" } : item),
    verifications: [{ id: `verify-${Date.now() + 1}`, pieceId, scheduleId: schedule.id, platform, actionType: "validation", outcome: "manual", providerStatus: state.omnisocials.state, failureReport: "Human-confirmed local schedule intent only; external publishing still disabled." }, ...state.verifications],
  };
}

export function blockedVerificationRecords(records: ContentVerificationRecord[]): ContentVerificationRecord[] {
  return records.filter((record) => record.outcome === "blocked" || record.outcome === "failed");
}

export function omnisocialsActionCopy(status: OmniSocialsStatusRecord): string {
  if (status.state === "connected" && status.credentialRef) return "Connected metadata exists, but Zoid still requires review and confirmation before any provider write.";
  return "OmniSocials is the Buffer alternative. Upload, schedule, and publish actions fail closed until a reviewed credential/provider slice is configured.";
}
