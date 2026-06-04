import type { FormEvent, ReactNode } from "react";
import { buildFileWorkspaceView, fileBridgeCommands, validateFileAction, type FileActionDraft, type FileWorkspaceState } from "./fileViewModel";

export type FileWorkspaceProps = {
  state: FileWorkspaceState;
  rootKey: string;
  relativePath: string;
  actionDraft: FileActionDraft;
  linkedPanels?: ReactNode;
  actionErrors?: string[];
  onBrowsePathChange?: (rootKey: string, relativePath: string) => void;
  onRefresh?: () => void;
  onSelectFile?: (relativePath: string) => void;
  onActionDraftChange?: (draft: FileActionDraft) => void;
  onPerformAction?: () => void;
};

export function FileWorkspace({ state, rootKey, relativePath, actionDraft, linkedPanels, actionErrors = [], onBrowsePathChange, onRefresh, onSelectFile, onActionDraftChange, onPerformAction }: FileWorkspaceProps) {
  const view = buildFileWorkspaceView(state);
  const validation = validateFileAction(actionDraft);
  const visibleErrors = actionErrors.length > 0 ? actionErrors : validation.errors;
  const updateAction = (patch: Partial<FileActionDraft>) => onActionDraftChange?.({ ...actionDraft, ...patch });
  function submit(event: FormEvent<HTMLFormElement>) { event.preventDefault(); if (validation.ok) onPerformAction?.(); }

  return (
    <section className="native-workspace file-workspace" aria-labelledby="file-workspace-heading">
      <header className="native-workspace-header">
        <h2 id="file-workspace-heading">Files</h2>
        <p>{view.copy}</p>
        <p aria-live="polite">{view.statusLabel}</p>
        <label>Root key<input value={rootKey} onChange={(event) => onBrowsePathChange?.(event.currentTarget.value, relativePath)} /></label>
        <label>Relative path<input value={relativePath} onChange={(event) => onBrowsePathChange?.(rootKey, event.currentTarget.value)} /></label>
        <button type="button" onClick={onRefresh}>Browse real files</button>
      </header>

      <div className="native-workspace-grid">
      <aside className="native-workspace-panel" aria-label="File list">
        {view.items.length === 0 ? <p>{view.detail.copy}</p> : null}
        <ul>{view.items.map((item) => <li key={item.id} aria-current={item.isSelected ? "true" : undefined}><button type="button" onClick={() => onSelectFile?.(item.id)}><strong>{item.title}</strong><span>{item.meta}{item.canPreview ? " · preview" : ""}</span></button></li>)}</ul>
      </aside>

      <article className="native-workspace-panel native-workspace-detail" aria-label="File detail">
        {view.detail.kind === "preview" ? <>
          <h3>{view.detail.preview.display_name}</h3>
          <p>{view.detail.preview.relative_path} · {view.detail.preview.byte_size} bytes{view.detail.preview.truncated ? " · truncated" : ""}</p>
          <pre>{view.detail.preview.preview_text}</pre>
        </> : <p>{view.detail.copy}</p>}
      </article>

      </div>

      <section className="native-workspace-panel" aria-label="File action policy"><p>{view.policyCopy}</p><p>{"actionCopy" in view ? view.actionCopy : "No action state."}</p></section>

      {linkedPanels}

      <section className="native-workspace-panel" aria-label="File actions">
        <h3>Perform file action</h3>
        <p>Actions use {fileBridgeCommands.action}. Confirmation ID is optional input from persisted approval; this UI does not generate or fake one.</p>
        <form aria-label="File action editor" onSubmit={submit}>
          <label>Action<select value={actionDraft.action} onChange={(event) => updateAction({ action: event.currentTarget.value as FileActionDraft["action"] })}><option value="copy">copy</option><option value="move">move</option><option value="rename">rename</option><option value="trash">trash</option></select></label>
          <label>Source path<input value={actionDraft.source_relative_path} onChange={(event) => updateAction({ source_relative_path: event.currentTarget.value })} /></label>
          <label>Destination path<input value={actionDraft.destination_relative_path ?? ""} onChange={(event) => updateAction({ destination_relative_path: event.currentTarget.value })} /></label>
          <label>Confirmation ID<input value={actionDraft.confirmation_id ?? ""} onChange={(event) => updateAction({ confirmation_id: event.currentTarget.value })} /></label>
          {visibleErrors.map((error) => <p role="alert" key={error}>{error}</p>)}
          <button type="submit" disabled={!validation.ok}>Run native file action</button>
        </form>
      </section>
    </section>
  );
}
