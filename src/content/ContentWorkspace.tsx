import { AlertTriangle, CalendarClock, CheckCircle2, Megaphone, ShieldCheck, UploadCloud } from "lucide-react";
import { useEffect, useMemo, useState } from "react";
import {
  approveSpecialistGate,
  blockedVerificationRecords,
  CONTENT_WORKSPACE_STORAGE_KEY,
  createDefaultContentWorkspaceState,
  createLocalScheduleIntent,
  omnisocialsActionCopy,
  parseContentWorkspaceState,
  pieceScheduleGateSummary,
  recordFailClosedSocialAction,
  type ContentWorkspaceState,
  type SocialPlatform,
} from "./contentModel";

function getInitialContentState(): ContentWorkspaceState {
  if (typeof window === "undefined") return createDefaultContentWorkspaceState();
  return parseContentWorkspaceState(window.localStorage.getItem(CONTENT_WORKSPACE_STORAGE_KEY));
}

export function ContentWorkspace() {
  const [state, setState] = useState<ContentWorkspaceState>(getInitialContentState);
  const selectedPiece = useMemo(
    () => state.pieces.find((piece) => piece.id === state.selectedPieceId) ?? state.pieces[0],
    [state.pieces, state.selectedPieceId],
  );
  const selectedPlan = state.plans.find((plan) => plan.id === selectedPiece?.planId);
  const selectedAssets = state.mediaAssets.filter((asset) => asset.pieceId === selectedPiece?.id);
  const blockedRecords = blockedVerificationRecords(state.verifications);

  useEffect(() => {
    window.localStorage.setItem(CONTENT_WORKSPACE_STORAGE_KEY, JSON.stringify(state));
  }, [state]);

  function selectPiece(pieceId: string) {
    setState((current) => ({ ...current, selectedPieceId: pieceId }));
  }

  function failClosed(actionType: "upload" | "schedule" | "publish") {
    if (!selectedPiece) return;
    setState((current) => recordFailClosedSocialAction(current, selectedPiece.id, selectedPiece.platforms[0] ?? "linkedin", actionType));
  }

  function approveGate() {
    if (!selectedPiece) return;
    setState((current) => approveSpecialistGate(current, selectedPiece.id));
  }

  function createIntent(platform: SocialPlatform) {
    if (!selectedPiece) return;
    setState((current) => createLocalScheduleIntent(current, selectedPiece.id, platform, true));
  }

  if (!selectedPiece) {
    return <section className="content-workspace" aria-label="Content workspace"><p>No content drafts yet.</p></section>;
  }

  return (
    <section className="content-workspace" aria-label="Content OmniSocials workspace">
      <header className="content-hero">
        <div>
          <p className="content-eyebrow">Content · OmniSocials</p>
          <h2>Buffer alternative, draft-first</h2>
          <p>
            Plan MaVoid social content, hold review gates, and record schedule intent without silently publishing through Buffer or any external social API.
          </p>
        </div>
        <div className={`content-provider-card content-provider-card--${state.omnisocials.state}`}>
          <ShieldCheck aria-hidden="true" size={22} />
          <span>Provider state</span>
          <strong>{state.omnisocials.state.replace(/_/g, " ")}</strong>
          <small>{state.omnisocials.statusNote}</small>
        </div>
      </header>

      <div className="content-workspace-grid">
        <aside className="content-left-panel" aria-label="Content plans and drafts">
          <div className="content-panel-heading">
            <span>Active plan</span>
            <strong>{selectedPlan?.title ?? "Unplanned"}</strong>
            <small>{selectedPlan?.pillar ?? "No pillar"}</small>
          </div>
          <div className="content-piece-list" role="list" aria-label="Draft pieces">
            {state.pieces.map((piece) => (
              <button
                className={piece.id === selectedPiece.id ? "content-piece-card active" : "content-piece-card"}
                key={piece.id}
                onClick={() => selectPiece(piece.id)}
                type="button"
              >
                <span>{piece.status.replace(/_/g, " ")}</span>
                <strong>{piece.title}</strong>
                <small>{piece.platforms.join(" · ")}</small>
              </button>
            ))}
          </div>
        </aside>

        <main className="content-center-panel" aria-label="Selected content piece">
          <div className="content-detail-header">
            <Megaphone aria-hidden="true" size={24} />
            <div>
              <p>{selectedPiece.status.replace(/_/g, " ")}</p>
              <h3>{selectedPiece.title}</h3>
            </div>
          </div>
          <article className="content-draft-card">
            <span>Draft copy</span>
            <p>{selectedPiece.bodyMarkdown}</p>
          </article>
          <div className="content-gate-strip" role="status">
            <AlertTriangle aria-hidden="true" size={18} />
            <span>{pieceScheduleGateSummary(selectedPiece, state.reviewGates, state.schedules)}</span>
          </div>
          <div className="content-action-row" aria-label="Content actions">
            <button onClick={approveGate} type="button"><CheckCircle2 aria-hidden="true" size={17} /> Approve specialist gate</button>
            <button onClick={() => createIntent("linkedin")} type="button"><CalendarClock aria-hidden="true" size={17} /> Record LinkedIn intent</button>
            <button onClick={() => failClosed("publish")} type="button"><UploadCloud aria-hidden="true" size={17} /> Test publish block</button>
          </div>
          <section className="content-assets" aria-label="Media references">
            <h4>Media / evidence references</h4>
            {selectedAssets.map((asset) => (
              <div className="content-asset-row" key={asset.id}>
                <span>{asset.assetKind}</span>
                <strong>{asset.storageRef}</strong>
                <small>{asset.altText}</small>
              </div>
            ))}
          </section>
        </main>

        <aside className="content-right-panel" aria-label="OmniSocials integration evidence">
          <div className="content-panel-heading">
            <span>Buffer replacement boundary</span>
            <strong>OmniSocials</strong>
            <small>{omnisocialsActionCopy(state.omnisocials)}</small>
          </div>
          <div className="content-fail-actions">
            <button onClick={() => failClosed("upload")} type="button">Fail-closed upload</button>
            <button onClick={() => failClosed("schedule")} type="button">Fail-closed schedule</button>
            <button onClick={() => failClosed("publish")} type="button">Fail-closed publish</button>
          </div>
          <div className="content-evidence-list" role="list" aria-label="Blocked verification records">
            {blockedRecords.slice(0, 5).map((record) => (
              <article className="content-evidence-card" key={record.id} role="listitem">
                <span>{record.outcome}</span>
                <strong>{record.actionType} · {record.platform}</strong>
                <small>{record.failureReport}</small>
              </article>
            ))}
          </div>
        </aside>
      </div>
    </section>
  );
}
