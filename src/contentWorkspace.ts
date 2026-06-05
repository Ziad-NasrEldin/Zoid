export type ContentPlanRecord = { id: string; title: string; pillar: string; status: string; owner_actor_type: string; metadata_json: string };
export type ContentPieceRecord = { id: string; plan_id: string; title: string; body_markdown: string; status: string; platforms_json: string; required_gate: string; metadata_json: string };
export type MediaAssetRecord = { id: string; piece_id: string; asset_kind: string; storage_ref: string; alt_text: string; metadata_json: string };
export type ContentReviewGateRecord = { id: string; piece_id: string; gate_type: string; status: string; evidence_summary: string };
export type ContentScheduleRecord = { id: string; piece_id: string; platform: string; scheduled_for: string; status: string; confirmation_id?: string | null };
export type ContentVerificationRecord = { id: string; piece_id?: string | null; schedule_id?: string | null; platform: string; action_type: string; outcome: string; provider_status?: string | null; failure_report?: string | null };
export type OmniSocialsStatusRecord = { state: string; platform: string; credential_ref?: string | null; status_note: string };

export type ContentWorkspaceState =
  | { mode: "loading" }
  | { mode: "error"; error: string }
  | {
      mode: "ready";
      plans: ContentPlanRecord[];
      pieces: ContentPieceRecord[];
      mediaAssets: MediaAssetRecord[];
      reviewGates: ContentReviewGateRecord[];
      schedules: ContentScheduleRecord[];
      verifications: ContentVerificationRecord[];
      omnisocials: OmniSocialsStatusRecord;
      selectedPieceId: string | null;
      actionStatus?: string | null;
    };

export function parsePlatforms(platformsJson: string): string[] {
  try {
    const parsed = JSON.parse(platformsJson || "[]");
    return Array.isArray(parsed) ? parsed.filter((item): item is string => typeof item === "string") : [];
  } catch {
    return [];
  }
}

export function pieceScheduleGateSummary(piece: ContentPieceRecord, gates: ContentReviewGateRecord[], schedules: ContentScheduleRecord[]): string {
  const pieceGates = gates.filter((gate) => gate.piece_id === piece.id && gate.gate_type === piece.required_gate);
  const hasApprovedGate = piece.required_gate === "none" || pieceGates.some((gate) => gate.status === "approved");
  const hasSchedule = schedules.some((schedule) => schedule.piece_id === piece.id && schedule.status === "intent");
  if (hasSchedule) return "Schedule intent recorded locally — no external publish implied.";
  if (!hasApprovedGate) return "Schedule blocked until specialist review is approved.";
  return "Review gate passed; schedule still needs human confirmation.";
}

export function omnisocialsActionCopy(status: OmniSocialsStatusRecord): string {
  if (status.state === "connected" && status.credential_ref) {
    return "Connection metadata exists, but Phase 5 still records manual/fail-closed action evidence before any external write.";
  }
  return "Upload, schedule, and publish are fail-closed because OmniSocials is not configured.";
}

export function blockedVerificationRecords(records: ContentVerificationRecord[]): ContentVerificationRecord[] {
  return records.filter((record) => record.outcome === "blocked" || record.outcome === "failed");
}
