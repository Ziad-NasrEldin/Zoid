import type { ReactElement } from "react";

import {
  createHistoryTimelineViewModel,
  type HistoryTimelineInput,
  type HistoryTimelineViewModel,
} from "./historyTimelineViewModel";

export interface HistoryTimelineProps extends HistoryTimelineInput {
  className?: string;
  onLoadMore?: (cursor: NonNullable<HistoryTimelineViewModel["nextCursor"]>) => void;
}

export function HistoryTimeline(props: HistoryTimelineProps): ReactElement {
  const { className, onLoadMore, ...input } = props;
  const viewModel = createHistoryTimelineViewModel(input);
  const classes = ["history-timeline", className].filter(Boolean).join(" ");

  return (
    <section className={classes} aria-busy={viewModel.isLoading} aria-label={viewModel.title}>
      <header className="history-timeline__header">
        <div>
          <p className="history-timeline__eyebrow">History</p>
          <h2>{viewModel.title}</h2>
        </div>
        <span className="history-timeline__status">{viewModel.paginationLabel}</span>
      </header>

      {viewModel.isLoading ? <HistoryTimelineNotice label="Loading history..." /> : null}
      {viewModel.errorMessage ? <HistoryTimelineNotice label={viewModel.errorMessage} tone="error" /> : null}
      {viewModel.emptyMessage ? <HistoryTimelineNotice label={viewModel.emptyMessage} /> : null}

      {viewModel.entries.length > 0 ? (
        <ol className="history-timeline__list">
          {viewModel.entries.map((entry) => (
            <li className={`history-timeline__entry history-timeline__entry--${entry.tone}`} key={entry.id}>
              <article>
                <header className="history-timeline__entry-header">
                  <time dateTime={entry.timestamp}>{entry.timeLabel}</time>
                  <span>{entry.outcomeLabel}</span>
                </header>
                <h3>{entry.actionLabel}</h3>
                <p>{entry.summary}</p>
                <dl className="history-timeline__meta">
                  <div>
                    <dt>Source</dt>
                    <dd>{entry.sourceLabel}</dd>
                  </div>
                  <div>
                    <dt>Matched</dt>
                    <dd>{entry.matchedLabels.join(", ") || "None"}</dd>
                  </div>
                  <div>
                    <dt>Targets</dt>
                    <dd>{entry.targetLabels.join(", ") || "None"}</dd>
                  </div>
                </dl>
              </article>
            </li>
          ))}
        </ol>
      ) : null}

      {viewModel.hasNextPage && viewModel.nextCursor ? (
        <footer className="history-timeline__footer">
          <button type="button" onClick={() => onLoadMore?.(viewModel.nextCursor!)} disabled={!onLoadMore}>
            Load older history
          </button>
        </footer>
      ) : null}
    </section>
  );
}

function HistoryTimelineNotice({ label, tone = "neutral" }: { label: string; tone?: "neutral" | "error" }): ReactElement {
  return <p className={`history-timeline__notice history-timeline__notice--${tone}`}>{label}</p>;
}
