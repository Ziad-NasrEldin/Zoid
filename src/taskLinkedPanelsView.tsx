import type { ReactElement } from "react";

import type { CleanSessionState } from "./cleanSession";
import { CleanSessionPanel } from "./cleanSessionView";
import { HistoryTimeline } from "./historyTimeline";
import type { InboxDataState, InboxNotificationRecord } from "./inboxViewModel";
import type { ManualReviewState } from "./taskDetailBatchPanels";
import { InboxAttentionPanel, LinkedRunHistoryPanels, ManualReviewPanel } from "./taskDetailBatchPanelsView";
import {
  createTaskLinkedPanelsViewModel,
  type TaskLinkedPanelsState,
} from "./taskLinkedPanels";

export type TaskLinkedPanelsProps = {
  state: TaskLinkedPanelsState;
  cleanSessions?: Record<string, CleanSessionState>;
  inboxState?: InboxDataState<InboxNotificationRecord>;
  manualReview?: ManualReviewState;
  runControls?: ReactElement;
  onManualReviewDraftChange?: Parameters<typeof ManualReviewPanel>[0]["onDraftChange"];
  onSubmitManualReview?: () => void;
  onClearManualReview?: () => void;
  onRefresh?: (taskId: string) => void;
  onRefreshCleanSession?: (runId: string) => void;
};

export function TaskLinkedPanels({ state, cleanSessions = {}, inboxState, manualReview, runControls, onManualReviewDraftChange, onSubmitManualReview, onClearManualReview, onRefresh, onRefreshCleanSession }: TaskLinkedPanelsProps): ReactElement {
  const view = createTaskLinkedPanelsViewModel(state);
  const taskId = view.taskId;
  const linkedRuns = state.mode === "ready" ? state.runs : [];

  return (
    <section aria-busy={view.isLoading} aria-label="Linked task activity" className="task-linked-panels">
      <header>
        <div>
          <p>Task detail context</p>
          <h3>Linked run, review, and history panels</h3>
        </div>
        {taskId ? <button type="button" onClick={() => onRefresh?.(taskId)}>Refresh linked activity</button> : null}
      </header>

      {view.errorMessage ? <p role="alert">{view.errorMessage}</p> : null}
      {view.isLoading ? <p>Loading linked activity from native history.</p> : null}

      {runControls}
      {manualReview ? <ManualReviewPanel state={manualReview} onDraftChange={onManualReviewDraftChange} onSubmit={onSubmitManualReview} onClear={onClearManualReview} /> : null}
      {inboxState ? <InboxAttentionPanel state={inboxState} /> : null}

      <div className="task-linked-panels__grid">
        <LinkedSummaryPanel title={view.runPanel.title} emptyCopy={view.runPanel.emptyCopy} items={view.runPanel.items} />
        <LinkedSummaryPanel title={view.reviewPanel.title} emptyCopy={view.reviewPanel.emptyCopy} items={view.reviewPanel.items} />
      </div>

      {linkedRuns.length > 0 ? (
        <section aria-label="Clean session output cards">
          <h4>Clean session output</h4>
          {linkedRuns.map((run) => (
            <CleanSessionPanel
              key={run.id}
              state={cleanSessions[run.id] ?? { mode: "unavailable", runId: run.id, reason: "Run output has not been streamed for this task detail yet." }}
              onRefresh={onRefreshCleanSession}
            />
          ))}
        </section>
      ) : null}

      <HistoryTimeline
        mode="task"
        primary={{ entity_type: "task", entity_id: taskId || "unselected" }}
        records={state.mode === "ready" ? state.history : []}
        status={state.mode === "loading" ? "loading" : state.mode === "error" ? "error" : "ready"}
        error={state.mode === "error" ? state.error : null}
        pageSize={25}
        includeRelated
      />
      <LinkedRunHistoryPanels history={state.mode === "ready" ? state.history : []} runIds={linkedRuns.map((run) => run.id)} />
    </section>
  );
}

function LinkedSummaryPanel({
  title,
  emptyCopy,
  items,
}: {
  title: string;
  emptyCopy: string;
  items: ReturnType<typeof createTaskLinkedPanelsViewModel>["runPanel"]["items"];
}): ReactElement {
  return (
    <section aria-label={title}>
      <h4>{title}</h4>
      {items.length === 0 ? <p>{emptyCopy}</p> : null}
      <ul>
        {items.map((item) => (
          <li key={item.id} data-tone={item.tone}>
            <strong>{item.title}</strong>
            <span>{item.summary}</span>
            <small>{item.meta}</small>
          </li>
        ))}
      </ul>
    </section>
  );
}
