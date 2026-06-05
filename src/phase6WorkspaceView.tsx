import { useState, type FormEvent, type ReactNode } from "react";
import type { Phase6Invoke, Phase6State } from "./phase6Workspace";
import { assertPhase6CalendarConfirmation, assertPhase6NoSilentSend, buildPhase6WorkspaceView } from "./phase6Workspace";

type Phase6WorkspaceProps = {
  workspaceId: string;
  state: Phase6State;
  onRefresh: () => void;
  invoke: Phase6Invoke;
};

type FormState = { status: "idle" | "saving" | "saved" | "error"; message: string };

function Phase6List({ children }: { children: ReactNode }) {
  return <div className="phase6-list">{children}</div>;
}

function initialFormState(): FormState {
  return { status: "idle", message: "" };
}

function JsonMetadata() {
  return <input name="metadata_json" placeholder="metadata JSON, e.g. {}" defaultValue="{}" />;
}

function formString(form: FormData, key: string) {
  const value = String(form.get(key) ?? "").trim();
  return value.length > 0 ? value : undefined;
}

function CalendarActions({ invoke, onRefresh }: Pick<Phase6WorkspaceProps, "invoke" | "onRefresh">) {
  const [state, setState] = useState<FormState>(initialFormState);
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const eventId = formString(form, "event_id");
    const action = String(form.get("action") ?? "create");
    const confirmationId = formString(form, "confirmation_id");
    const request = {
      title: formString(form, "title") ?? "",
      starts_at: formString(form, "starts_at") ?? "",
      ends_at: formString(form, "ends_at") ?? "",
      location: formString(form, "location") ?? null,
      notes: formString(form, "notes") ?? null,
      confirmation_id: confirmationId,
      metadata_json: formString(form, "metadata_json") ?? "{}",
    };
    setState({ status: "saving", message: "Saving calendar change…" });
    try {
      if (action === "delete") {
        if (!eventId) throw new Error("event_id is required for delete");
        if (!assertPhase6CalendarConfirmation("delete_calendar_event_command", { request })) throw new Error("confirmation_id is required");
        await invoke("delete_calendar_event_command", { eventId, request: { confirmation_id: confirmationId } });
      } else if (action === "update") {
        if (!eventId) throw new Error("event_id is required for update");
        if (!assertPhase6CalendarConfirmation("update_calendar_event_command", { request })) throw new Error("confirmation_id is required");
        await invoke("update_calendar_event_command", { eventId, request });
      } else {
        if (!assertPhase6CalendarConfirmation("create_calendar_event_command", { request })) throw new Error("confirmation_id is required");
        await invoke("create_calendar_event_command", { request });
      }
      setState({ status: "saved", message: "Calendar command completed locally." });
      onRefresh();
    } catch (error) {
      setState({ status: "error", message: error instanceof Error ? error.message : String(error) });
    }
  }
  return (
    <form className="phase6-form" onSubmit={submit}>
      <p className="eyebrow">Calendar write flow</p>
      <select name="action" defaultValue="create"><option value="create">Create</option><option value="update">Update</option><option value="delete">Delete</option></select>
      <input name="event_id" placeholder="event id for update/delete" />
      <input name="title" placeholder="title" />
      <input name="starts_at" placeholder="starts_at ISO" />
      <input name="ends_at" placeholder="ends_at ISO" />
      <input name="location" placeholder="location" />
      <input name="notes" placeholder="notes" />
      <input name="confirmation_id" placeholder="approved confirmation_id required" required />
      <JsonMetadata />
      <button type="submit">Run calendar command</button>
      {state.message ? <p className={`muted-copy ${state.status}`}>{state.message}</p> : null}
    </form>
  );
}

function MailActions({ invoke, onRefresh }: Pick<Phase6WorkspaceProps, "invoke" | "onRefresh">) {
  const [state, setState] = useState<FormState>(initialFormState);
  async function createDraft(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    setState({ status: "saving", message: "Creating draft…" });
    try {
      await invoke("create_email_draft_command", { request: {
        subject: formString(form, "subject") ?? "",
        recipients_json: formString(form, "recipients_json") ?? "[]",
        snippet: formString(form, "snippet") ?? null,
        thread_id: formString(form, "thread_id") ?? null,
        metadata_json: formString(form, "metadata_json") ?? "{}",
      } });
      setState({ status: "saved", message: "Draft created locally." });
      onRefresh();
    } catch (error) {
      setState({ status: "error", message: error instanceof Error ? error.message : String(error) });
    }
  }
  async function sendDraft(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    const form = new FormData(event.currentTarget);
    const emailId = formString(form, "email_id");
    const request = { confirmation_id: formString(form, "confirmation_id") };
    setState({ status: "saving", message: "Sending local state transition…" });
    try {
      if (!emailId) throw new Error("email_id is required");
      if (!assertPhase6NoSilentSend("send_email_draft_command", { request })) throw new Error("confirmation_id is required");
      await invoke("send_email_draft_command", { emailId, request });
      setState({ status: "saved", message: "Draft marked sent locally. No external Gmail send was performed." });
      onRefresh();
    } catch (error) {
      setState({ status: "error", message: error instanceof Error ? error.message : String(error) });
    }
  }
  return (
    <div className="phase6-actions">
      <form className="phase6-form" onSubmit={createDraft}>
        <p className="eyebrow">Gmail-safe draft flow</p>
        <input name="subject" placeholder="subject" required />
        <input name="recipients_json" placeholder='recipients JSON, e.g. ["name@example.com"]' defaultValue="[]" />
        <input name="snippet" placeholder="snippet" />
        <input name="thread_id" placeholder="thread id" />
        <JsonMetadata />
        <button type="submit">Create local draft</button>
      </form>
      <form className="phase6-form" onSubmit={sendDraft}>
        <p className="eyebrow">Confirmation-gated send state</p>
        <input name="email_id" placeholder="draft email id" required />
        <input name="confirmation_id" placeholder="approved confirmation_id required" required />
        <button type="submit">Mark draft sent locally</button>
      </form>
      {state.message ? <p className={`muted-copy ${state.status}`}>{state.message}</p> : null}
    </div>
  );
}

function BusinessActions({ invoke, onRefresh }: Pick<Phase6WorkspaceProps, "invoke" | "onRefresh">) {
  const [state, setState] = useState<FormState>(initialFormState);
  async function submit(command: string, request: Record<string, unknown>) {
    setState({ status: "saving", message: "Saving business record…" });
    try {
      await invoke(command, { request });
      setState({ status: "saved", message: "Business record saved." });
      onRefresh();
    } catch (error) {
      setState({ status: "error", message: error instanceof Error ? error.message : String(error) });
    }
  }
  return (
    <div className="phase6-actions">
      <form className="phase6-form" onSubmit={(event) => { event.preventDefault(); const form = new FormData(event.currentTarget); void submit("create_business_company_command", { name: formString(form, "name") ?? "", domain: formString(form, "domain") ?? null, notes: formString(form, "notes") ?? null, metadata_json: formString(form, "metadata_json") ?? "{}" }); }}>
        <p className="eyebrow">Create company</p><input name="name" placeholder="company name" required /><input name="domain" placeholder="domain" /><input name="notes" placeholder="notes" /><JsonMetadata /><button type="submit">Create company</button>
      </form>
      <form className="phase6-form" onSubmit={(event) => { event.preventDefault(); const form = new FormData(event.currentTarget); void submit("create_business_contact_command", { company_id: formString(form, "company_id") ?? null, full_name: formString(form, "full_name") ?? "", email: formString(form, "email") ?? null, phone: formString(form, "phone") ?? null, role: formString(form, "role") ?? null, notes: formString(form, "notes") ?? null, metadata_json: formString(form, "metadata_json") ?? "{}" }); }}>
        <p className="eyebrow">Create contact</p><input name="company_id" placeholder="company id" /><input name="full_name" placeholder="full name" required /><input name="email" placeholder="email" /><input name="phone" placeholder="phone" /><input name="role" placeholder="role" /><input name="notes" placeholder="notes" /><JsonMetadata /><button type="submit">Create contact</button>
      </form>
      <form className="phase6-form" onSubmit={(event) => { event.preventDefault(); const form = new FormData(event.currentTarget); void submit("create_follow_up_command", { subject: formString(form, "subject") ?? "", due_at: formString(form, "due_at") ?? null, priority: formString(form, "priority") ?? "normal", contact_id: formString(form, "contact_id") ?? null, company_id: formString(form, "company_id") ?? null, product_id: formString(form, "product_id") ?? null, metadata_json: formString(form, "metadata_json") ?? "{}" }); }}>
        <p className="eyebrow">Create follow-up</p><input name="subject" placeholder="subject" required /><input name="due_at" placeholder="due date" /><input name="priority" placeholder="priority" defaultValue="normal" /><input name="contact_id" placeholder="contact id" /><input name="company_id" placeholder="company id" /><input name="product_id" placeholder="product id" /><JsonMetadata /><button type="submit">Create follow-up</button>
      </form>
      {state.message ? <p className={`muted-copy ${state.status}`}>{state.message}</p> : null}
    </div>
  );
}

function ProductActions({ invoke, onRefresh }: Pick<Phase6WorkspaceProps, "invoke" | "onRefresh">) {
  const [state, setState] = useState<FormState>(initialFormState);
  async function submit(command: string, request: Record<string, unknown>) {
    setState({ status: "saving", message: "Saving product record…" });
    try {
      await invoke(command, { request });
      setState({ status: "saved", message: "Product command completed." });
      onRefresh();
    } catch (error) {
      setState({ status: "error", message: error instanceof Error ? error.message : String(error) });
    }
  }
  return (
    <div className="phase6-actions">
      <form className="phase6-form" onSubmit={(event) => { event.preventDefault(); const form = new FormData(event.currentTarget); void submit("create_product_command", { name: formString(form, "name") ?? "", status: formString(form, "status") ?? "active", summary: formString(form, "summary") ?? null, owner_contact_id: formString(form, "owner_contact_id") ?? null, metadata_json: formString(form, "metadata_json") ?? "{}" }); }}>
        <p className="eyebrow">Create product</p><input name="name" placeholder="product name" required /><input name="status" placeholder="status" defaultValue="active" /><input name="summary" placeholder="summary" /><input name="owner_contact_id" placeholder="owner contact id" /><JsonMetadata /><button type="submit">Create product</button>
      </form>
      <form className="phase6-form" onSubmit={(event) => { event.preventDefault(); const form = new FormData(event.currentTarget); void submit("link_product_entity_command", { product_id: formString(form, "product_id") ?? "", target_type: formString(form, "target_type") ?? "", target_id: formString(form, "target_id") ?? "", relation_type: formString(form, "relation_type") ?? "related", metadata_json: formString(form, "metadata_json") ?? "{}" }); }}>
        <p className="eyebrow">Link product context</p><input name="product_id" placeholder="product id" required /><input name="target_type" placeholder="target type" required /><input name="target_id" placeholder="target id" required /><input name="relation_type" placeholder="relation" defaultValue="related" /><JsonMetadata /><button type="submit">Link product</button>
      </form>
      {state.message ? <p className={`muted-copy ${state.status}`}>{state.message}</p> : null}
    </div>
  );
}

export function Phase6Workspace({ workspaceId, state, onRefresh, invoke }: Phase6WorkspaceProps) {
  const view = buildPhase6WorkspaceView(workspaceId, state);
  const overview = state.mode === "ready" ? state.overview : null;
  return (
    <>
      <article className="hero-card">
        <div>
          <p className="eyebrow">Phase 6 native workspace</p>
          <h3>{view.title}</h3>
          <p>{view.heroCopy}</p>
          <p className="muted-copy">{view.statusCopy}</p>
        </div>
        <button className="secondary-action" onClick={onRefresh} type="button">Refresh native data</button>
      </article>

      <section className="dashboard-grid phase6-grid">
        <article className="card large-card">
          <div className="card-header"><div><p className="eyebrow">Truthful integration states</p><h3>No fake connected data</h3></div></div>
          <Phase6List>{view.blockers.map((copy) => <p className="muted-copy" key={copy}>{copy}</p>)}</Phase6List>
        </article>
        {view.sections.map((section) => <article className="card" key={section.key}><p className="eyebrow">{section.title}</p><h3>{section.count} records</h3><p>{section.count === 0 ? section.emptyCopy : "Backed by native Phase 6 commands and SQLite persistence."}</p></article>)}
        <article className="card large-card">
          <p className="eyebrow">Command flows</p>
          <h3>{view.title} actions</h3>
          {workspaceId === "calendar" ? <CalendarActions invoke={invoke} onRefresh={onRefresh} /> : null}
          {workspaceId === "inbox" ? <MailActions invoke={invoke} onRefresh={onRefresh} /> : null}
          {workspaceId === "business" ? <BusinessActions invoke={invoke} onRefresh={onRefresh} /> : null}
          {workspaceId === "products" ? <ProductActions invoke={invoke} onRefresh={onRefresh} /> : null}
        </article>
        <article className="card large-card">
          <p className="eyebrow">Records</p><h3>{view.title} detail</h3>
          {!overview ? <p className="muted-copy">Native data is loading or unavailable.</p> : (
            <Phase6List>
              {workspaceId === "calendar" && overview.calendar.map((event) => <p key={event.id}><strong>{event.title}</strong> · {event.starts_at} → {event.ends_at} · {event.state} · {event.id}</p>)}
              {workspaceId === "inbox" && overview.inbox.map((item) => <p key={`${item.item_type}:${item.id}`}><strong>{item.title}</strong> · {item.item_type} · {item.state} · {item.id}</p>)}
              {workspaceId === "business" && overview.companies.map((company) => <p key={company.id}><strong>{company.name}</strong> · {company.domain ?? "no domain"} · {company.id}</p>)}
              {workspaceId === "business" && overview.contacts.map((contact) => <p key={contact.id}><strong>{contact.full_name}</strong> · {contact.email ?? "no email"} · {contact.id}</p>)}
              {workspaceId === "business" && overview.follow_ups.map((followUp) => <p key={followUp.id}><strong>{followUp.subject}</strong> · {followUp.priority} · {followUp.state} · {followUp.id}</p>)}
              {workspaceId === "products" && overview.products.map((product) => <p key={product.id}><strong>{product.name}</strong> · {product.status} · {product.id}</p>)}
              {workspaceId === "products" && overview.product_links.map((link) => <p key={link.id}><strong>{link.source_id}</strong> → {link.target_type}:{link.target_id}</p>)}
              {workspaceId !== "calendar" && workspaceId !== "business" && workspaceId !== "products" && overview.emails.map((email) => <p key={email.id}><strong>{email.subject}</strong> · {email.state} · {email.id}</p>)}
            </Phase6List>
          )}
        </article>
      </section>
    </>
  );
}
