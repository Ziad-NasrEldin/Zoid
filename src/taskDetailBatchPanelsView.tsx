import type { ChangeEvent, FormEvent, ReactElement } from "react";

import { HistoryTimeline } from "./historyTimeline";
import type { HistoryTimelineRecord } from "./historyTimelineViewModel";
import { buildInboxAttentionViewModel, type InboxDataState, type InboxNotificationRecord } from "./inboxViewModel";
import {
  createManualReviewViewModel,
  runHistoryRecordsForRun,
  type ManualReviewDraft,
  type ManualReviewState,
} from "./taskDetailBatchPanels";

export function ManualReviewPanel({
  state,
  onDraftChange,
  onSubmit,
  onClear,
}: {
  state: ManualReviewState;
  onDraftChange?: (patch: Partial<ManualReviewDraft>) => void;
  onSubmit?: () => void;
  onClear?: () => void;
}): ReactElement {
  const view = createManualReviewViewModel(state);
  const update = (patch: Partial<ManualReviewDraft>) => onDraftChange?.(patch);

  return (
    <section aria-busy={view.isBusy} aria-label="Manual review stub" className="manual-review-panel">
      <header>
        <div>
          <p>Manual review</p>
          <h4>{view.statusLabel}</h4>
        </div>
        <div>
          <button disabled={!view.canSubmit} onClick={onSubmit} type="button">Record review</button>
          <button disabled={!view.canClear} onClick={onClear} type="button">Clear review draft</button>
        </div>
      </header>

      <form onSubmit={submit(onSubmit)}>
        <label>
          Verdict
          <select value={state.draft.verdict} onChange={changeText((verdict) => update({ verdict: verdict as ManualReviewDraft["verdict"] }))}>
            <option value="approved">Approved</option>
            <option value="changes_required">Changes required</option>
            <option value="insufficient_evidence">Insufficient evidence</option>
            <option value="blocked">Blocked</option>
          </select>
        </label>
        <label>
          Reviewer profile placeholder
          <input value={state.draft.reviewerProfileId} onChange={changeText((reviewerProfileId) => update({ reviewerProfileId }))} placeholder="Optional reviewer profile id" />
        </label>
        <label>
          Evidence summary
          <textarea value={state.draft.evidenceSummary} onChange={changeText((evidenceSummary) => update({ evidenceSummary }))} rows={3} />
        </label>
        <label>
          Required fixes JSON array
          <textarea value={state.draft.requiredFixesJson} onChange={changeText((requiredFixesJson) => update({ requiredFixesJson }))} rows={2} />
        </label>
      </form>

      {view.lastReviewLabel ? <p>{view.lastReviewLabel}</p> : null}
      {view.errorMessage ? <p role="alert">{view.errorMessage}</p> : null}
      {view.validationErrors.length > 0 ? (
        <ul aria-label="Manual review validation errors">
          {view.validationErrors.map((error) => <li key={error}>{error}</li>)}
        </ul>
      ) : null}
    </section>
  );
}

export function InboxAttentionPanel({ state }: { state: InboxDataState<InboxNotificationRecord> }): ReactElement {
  const view = buildInboxAttentionViewModel(state);
  return (
    <section aria-label="Inbox attention cards" className="inbox-attention-panel">
      <header>
        <p>Inbox</p>
        <h4>{view.title} · {view.status}</h4>
      </header>
      <p>{view.summary}</p>
      {view.error ? <p role="alert">{view.error}</p> : null}
      {view.emptyCopy ? <p>{view.emptyCopy}</p> : null}
      <ul>
        {view.items.map((item) => (
          <li key={item.id} data-tone={item.tone}>
            <strong>{item.title}</strong>
            <span>{item.message}</span>
            <small>{item.typeLabel} · {item.severityLabel} · {item.stateLabel}</small>
            {item.linkedLabels.length > 0 ? <small>{item.linkedLabels.join(" · ")}</small> : null}
          </li>
        ))}
      </ul>
    </section>
  );
}

export function LinkedRunHistoryPanels({ history, runIds }: { history: HistoryTimelineRecord[]; runIds: string[] }): ReactElement | null {
  if (runIds.length === 0) return null;
  return (
    <section aria-label="Run history panels" className="run-history-panels">
      <h4>Run history</h4>
      {runIds.map((runId) => (
        <HistoryTimeline
          key={runId}
          mode="run"
          primary={{ entity_type: "run", entity_id: runId }}
          records={runHistoryRecordsForRun(history, runId)}
          status="ready"
          pageSize={10}
          includeRelated
        />
      ))}
    </section>
  );
}

function changeText(callback: (value: string) => void) {
  return (event: ChangeEvent<HTMLInputElement | HTMLTextAreaElement | HTMLSelectElement>) => callback(event.currentTarget.value);
}

function submit(onSubmit?: () => void) {
  return (event: FormEvent) => {
    event.preventDefault();
    onSubmit?.();
  };
}
