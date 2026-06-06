import type { ReactElement } from "react";

import { createCleanSessionViewModel, type CleanSessionState } from "./cleanSession";

export type CleanSessionPanelProps = {
  state: CleanSessionState;
  onRefresh?: (runId: string) => void;
};

export function CleanSessionPanel({ state, onRefresh }: CleanSessionPanelProps): ReactElement {
  const view = createCleanSessionViewModel(state);
  const runId = view.runId;

  return (
    <section aria-busy={view.isLoading} aria-label={runId ? `Clean session output for ${runId}` : "Clean session output"} className="clean-session-panel">
      <header>
        <div>
          <p>Clean session</p>
          <h4>{view.statusLabel}</h4>
          <small data-tone={view.statusTone}>{view.summary}</small>
        </div>
        {runId ? <button type="button" onClick={() => onRefresh?.(runId)}>Refresh output</button> : null}
      </header>

      {view.errorMessage ? <p role="alert">{view.errorMessage}</p> : null}
      {view.cards.length === 0 ? <p>{view.emptyCopy}</p> : null}

      <ol className="clean-session-cards">
        {view.cards.map((card) => (
          <li key={card.id} data-kind={card.kind}>
            <strong>{card.title}</strong>
            <span>{card.body}</span>
          </li>
        ))}
      </ol>

      {view.nextOffset !== null ? (
        <small>{view.eof ? "Stream complete" : `Next stream offset ${view.nextOffset}`}</small>
      ) : null}
    </section>
  );
}
