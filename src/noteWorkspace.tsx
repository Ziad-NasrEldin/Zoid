import type { FormEvent, ReactNode } from "react";
import { buildNoteWorkspaceView, noteBridgeCommands, validateNoteForm, type NoteFormDraft, type NoteValidationErrors, type NoteWorkspaceState } from "./noteViewModel";

export type NoteWorkspaceProps = {
  state: NoteWorkspaceState;
  form: NoteFormDraft;
  linkedPanels?: ReactNode;
  formErrors?: NoteValidationErrors;
  onFormChange?: (form: NoteFormDraft) => void;
  onCreateNote?: (form: NoteFormDraft) => void;
  onEditNote?: (noteId: string, form: NoteFormDraft) => void;
  onSelectNote?: (noteId: string) => void;
  onRefresh?: () => void;
  onScan?: () => void;
  onTrashNote?: (noteId: string) => void;
};

function FieldError({ message }: { message?: string }) { return message ? <p role="alert">{message}</p> : null; }

export function NoteWorkspace({ state, form, linkedPanels, formErrors, onFormChange, onCreateNote, onEditNote, onSelectNote, onRefresh, onScan, onTrashNote }: NoteWorkspaceProps) {
  const view = buildNoteWorkspaceView(state);
  const selectedNote = view.detail.kind === "note" ? view.detail.note : null;
  const validation = validateNoteForm(form);
  const visibleErrors = formErrors ?? (validation.ok ? {} : validation.errors);
  const update = (patch: Partial<NoteFormDraft>) => onFormChange?.({ ...form, ...patch });
  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (!validation.ok) return;
    selectedNote ? onEditNote?.(selectedNote.id, form) : onCreateNote?.(form);
  }

  return (
    <section className="native-workspace note-workspace" aria-labelledby="note-workspace-heading">
      <header className="native-workspace-header">
        <h2 id="note-workspace-heading">Notes</h2>
        <p>{view.copy}</p>
        <p aria-live="polite">{view.statusLabel}</p>
        <button type="button" onClick={onRefresh}>Refresh real notes</button>
        <button type="button" onClick={onScan}>Scan Markdown notes</button>
      </header>

      <div className="native-workspace-grid">
      <aside className="native-workspace-panel" aria-label="Note list">
        {view.items.length === 0 ? <p>{view.detail.copy}</p> : null}
        <ul>{view.items.map((item) => <li key={item.id} aria-current={item.isSelected ? "true" : undefined}><button type="button" onClick={() => onSelectNote?.(item.id)}><strong>{item.title}</strong><span>{item.meta}</span></button></li>)}</ul>
      </aside>

      <article className="native-workspace-panel native-workspace-detail" aria-label="Note detail">
        {view.detail.kind === "note" && selectedNote ? <>
          <h3>{selectedNote.title}</h3>
          <p>{selectedNote.relative_path}</p>
          <pre>{selectedNote.markdown || "No Markdown was returned by the native bridge."}</pre>
          <h4>Metadata</h4><pre>{view.detail.metadataPreview}</pre>
          <button type="button" onClick={() => onTrashNote?.(selectedNote.id)}>Trash note</button>
        </> : <p>{view.detail.copy}</p>}
      </article>

      </div>

      <section className="native-workspace-panel" aria-label="Note conflicts"><p>{view.conflictsCopy}</p>{"scanCopy" in view ? <p>{view.scanCopy}</p> : null}</section>

      {linkedPanels}

      <section className="native-workspace-panel" aria-label="Create or edit note">
        <h3>{selectedNote ? "Edit Markdown note" : "Create Markdown note"}</h3>
        <p>Create uses {noteBridgeCommands.create}; edit uses {noteBridgeCommands.edit}. This UI does not fabricate notes outside the native bridge.</p>
        <form aria-label="Note editor" onSubmit={submit}>
          <label>Title<input value={form.title} onChange={(event) => update({ title: event.currentTarget.value })} required /></label><FieldError message={visibleErrors.title} />
          <label>Relative path<input value={form.relative_path ?? ""} onChange={(event) => update({ relative_path: event.currentTarget.value })} /></label><FieldError message={visibleErrors.relative_path} />
          <label>Markdown<textarea value={form.body_markdown} onChange={(event) => update({ body_markdown: event.currentTarget.value })} required /></label><FieldError message={visibleErrors.body_markdown} />
          <label>Metadata JSON<textarea value={form.metadata_json ?? "{}"} onChange={(event) => update({ metadata_json: event.currentTarget.value })} /></label><FieldError message={visibleErrors.metadata_json} />
          <button type="submit" disabled={!validation.ok}>{selectedNote ? "Update note" : "Create note"}</button>
        </form>
      </section>
    </section>
  );
}
