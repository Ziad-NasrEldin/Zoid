import type { ReactElement } from "react";
import { createContentLinkedPanelsViewModel, type ContentLinkedPanelsState } from "./contentLinkedPanels";
import { HistoryTimeline } from "./historyTimeline";

export function ContentLinkedPanels({ state, onRefresh }: { state: ContentLinkedPanelsState; onRefresh?: (entityId: string) => void }): ReactElement {
  const view = createContentLinkedPanelsViewModel(state);
  return (
    <section className="native-workspace-panel content-linked-panels" aria-label={view.title}>
      <header className="content-linked-panels__header">
        <div>
          <h3>{view.title}</h3>
          <p>{view.entityId ? `Loaded for ${view.entityId}` : "Select a persisted note or indexed file to load native links/history."}</p>
        </div>
        <button type="button" disabled={!view.entityId || view.isLoading} onClick={() => view.entityId && onRefresh?.(view.entityId)}>
          Refresh links/history
        </button>
      </header>
      {view.errorMessage ? <p role="alert">{view.errorMessage}</p> : null}
      {view.isLoading ? <p>Loading native entity history and links…</p> : null}
      <div className="content-linked-panels__grid">
        <section aria-label="Content entity links">
          <h4>{view.linkPanel.title}</h4>
          {view.linkPanel.items.length === 0 ? <p>{view.linkPanel.emptyCopy}</p> : null}
          <ul>
            {view.linkPanel.items.map((item) => (
              <li key={item.id}>
                <strong>{item.title}</strong>
                <span>{item.meta}</span>
                <small>{item.summary}</small>
              </li>
            ))}
          </ul>
        </section>
        <HistoryTimeline mode="entity" primary={{ entity_type: state.entityType, entity_id: view.entityId || "unselected" }} records={state.mode === "ready" ? state.history : []} status={view.historyPanel.status} error={view.errorMessage} />
      </div>
    </section>
  );
}
