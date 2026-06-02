import type { InboxAttentionItemView, InboxAttentionViewModel, InboxIntent } from "./inboxViewModel";

export type InboxAttentionCardProps = {
  viewModel: InboxAttentionViewModel;
  onIntent?: (intent: InboxIntent) => void;
};

const intentLabels: Record<InboxIntent["kind"], string> = {
  mark_read: "Mark read",
  dismiss: "Dismiss",
  resolve: "Resolve",
  require_action: "Needs action",
  open_route: "Open route",
};

function InboxAttentionItem({ item, onIntent }: { item: InboxAttentionItemView; onIntent?: (intent: InboxIntent) => void }) {
  return (
    <article className={`inbox-attention-card__item inbox-attention-card__item--${item.tone}`} data-notification-id={item.id}>
      <header className="inbox-attention-card__item-header">
        <div>
          <h3>{item.title}</h3>
          <p>{item.message}</p>
        </div>
        {item.isUnread ? <span aria-label="Unread notification">Unread</span> : <span>Read</span>}
      </header>
      <dl className="inbox-attention-card__meta" aria-label={`${item.title} metadata`}>
        <div>
          <dt>Type</dt>
          <dd>{item.typeLabel}</dd>
        </div>
        <div>
          <dt>Severity</dt>
          <dd>{item.severityLabel}</dd>
        </div>
        <div>
          <dt>State</dt>
          <dd>{item.stateLabel}</dd>
        </div>
        {item.actionRouteLabel ? (
          <div>
            <dt>Action route</dt>
            <dd>{item.actionRouteLabel}</dd>
          </div>
        ) : null}
        {item.linkedLabels.length > 0 ? (
          <div>
            <dt>Linked</dt>
            <dd>{item.linkedLabels.join(" · ")}</dd>
          </div>
        ) : null}
      </dl>
      <div className="inbox-attention-card__intents" aria-label={`${item.title} intents`}>
        {item.intents.map((intent) => (
          <button key={`${item.id}-${intent.kind}`} type="button" onClick={() => onIntent?.(intent)}>
            {intentLabels[intent.kind]}
          </button>
        ))}
      </div>
    </article>
  );
}

export function InboxAttentionCard({ viewModel, onIntent }: InboxAttentionCardProps) {
  return (
    <section className="inbox-attention-card" aria-busy={viewModel.state === "loading"} aria-live="polite">
      <header className="inbox-attention-card__header">
        <div>
          <h2>{viewModel.title}</h2>
          <p>{viewModel.summary}</p>
        </div>
        <strong>{viewModel.status}</strong>
      </header>
      {viewModel.error ? <p role="alert">{viewModel.error}</p> : null}
      {viewModel.items.length === 0 ? <p>{viewModel.emptyCopy ?? viewModel.summary}</p> : null}
      {viewModel.items.map((item) => (
        <InboxAttentionItem key={item.id} item={item} onIntent={onIntent} />
      ))}
    </section>
  );
}

export default InboxAttentionCard;
