import type { ChangeEvent, FormEvent, ReactElement } from "react";

import {
  createRunControlsViewModel,
  type RunControlsDraft,
  type RunControlsState,
} from "./runControls";

export type RunControlsPanelProps = {
  state: RunControlsState;
  onDraftChange?: (patch: Partial<RunControlsDraft>) => void;
  onStart?: () => void;
  onCancel?: () => void;
  onClear?: () => void;
};

export function RunControlsPanel({ state, onDraftChange, onStart, onCancel, onClear }: RunControlsPanelProps): ReactElement {
  const view = createRunControlsViewModel(state);
  const update = (patch: Partial<RunControlsDraft>) => onDraftChange?.(patch);

  return (
    <section aria-busy={view.isBusy} aria-label="Run controls" className="run-controls-panel">
      <header>
        <div>
          <p>Run controls</p>
          <h4>{view.statusLabel}</h4>
          <small data-tone={view.statusTone}>{view.commandPreview || "Configure a safe command to run for this task."}</small>
        </div>
        <div>
          <button disabled={!view.canStart} onClick={onStart} type="button">Start run</button>
          <button disabled={!view.canCancel} onClick={onCancel} type="button">Cancel run</button>
          <button disabled={!view.canClear} onClick={onClear} type="button">Clear status</button>
        </div>
      </header>

      <form onSubmit={handleSubmit(onStart)}>
        <label>
          Profile
          <input value={state.draft.profileId} onChange={changeText((profileId) => update({ profileId }))} placeholder="default" />
        </label>
        <label>
          Working directory
          <input value={state.draft.cwd} onChange={changeText((cwd) => update({ cwd }))} placeholder="/Users/example/Zoid" />
        </label>
        <label>
          Command arguments
          <textarea value={state.draft.argvText} onChange={changeText((argvText) => update({ argvText }))} placeholder={"npm\nrun\ntest:frontend"} rows={3} />
        </label>
        <label>
          Stdin, optional
          <textarea value={state.draft.stdin} onChange={changeText((stdin) => update({ stdin }))} rows={2} />
        </label>
        <label>
          Timeout ms
          <input inputMode="numeric" value={state.draft.timeoutMsText} onChange={changeText((timeoutMsText) => update({ timeoutMsText }))} />
        </label>
        <label>
          Metadata JSON
          <textarea value={state.draft.metadataJson} onChange={changeText((metadataJson) => update({ metadataJson }))} rows={2} />
        </label>
      </form>

      {view.lastMessage ? <p>{view.lastMessage}</p> : null}
      {view.errorMessage ? <p role="alert">{view.errorMessage}</p> : null}
      {view.validationErrors.length > 0 ? (
        <ul aria-label="Run control validation errors">
          {view.validationErrors.map((error) => <li key={error}>{error}</li>)}
        </ul>
      ) : null}
    </section>
  );
}

function changeText(callback: (value: string) => void) {
  return (event: ChangeEvent<HTMLInputElement | HTMLTextAreaElement>) => callback(event.currentTarget.value);
}

function handleSubmit(onStart?: () => void) {
  return (event: FormEvent) => {
    event.preventDefault();
    onStart?.();
  };
}
