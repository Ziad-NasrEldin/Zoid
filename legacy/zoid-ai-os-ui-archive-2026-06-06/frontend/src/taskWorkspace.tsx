import type { FormEvent, ReactNode } from "react";
import {
  buildTaskWorkspaceView,
  createInitialTaskForm,
  taskBridgeCommands,
  taskPriorities,
  taskStatuses,
  validateTaskForm,
  type TaskFormDraft,
  type TaskRecord,
  type TaskValidationErrors,
  type TaskWorkspaceState,
} from "./taskViewModel";

export type TaskWorkspaceProps = {
  state: TaskWorkspaceState;
  form?: TaskFormDraft;
  formErrors?: TaskValidationErrors;
  onFormChange?: (form: TaskFormDraft) => void;
  onCreateTask?: (form: TaskFormDraft) => void;
  onUpdateTask?: (taskId: string, form: TaskFormDraft) => void;
  onUpdateTaskStatus?: (taskId: string, status: string) => void;
  onArchiveTask?: (taskId: string) => void;
  onDeleteTask?: (taskId: string) => void;
  onNewTask?: () => void;
  onSelectTask?: (taskId: string) => void;
  onRefresh?: () => void;
  linkedPanels?: ReactNode;
};

function taskToForm(task: TaskRecord): TaskFormDraft {
  return {
    title: task.title,
    detail: task.detail ?? "",
    status: task.status,
    priority: task.priority,
    workspace_key: task.workspace_key,
    metadata_json: task.metadata_json || "{}",
  };
}

function FieldError({ message }: { message?: string }) {
  if (!message) return null;
  return <p className="field-error" role="alert">{message}</p>;
}

export function TaskForm({
  form,
  errors,
  submitLabel,
  onChange,
  onSubmit,
}: {
  form: TaskFormDraft;
  errors?: TaskValidationErrors;
  submitLabel: string;
  onChange?: (form: TaskFormDraft) => void;
  onSubmit?: (form: TaskFormDraft) => void;
}) {
  const update = (patch: Partial<TaskFormDraft>) => onChange?.({ ...form, ...patch });
  const validation = validateTaskForm(form);
  const visibleErrors = errors ?? (validation.ok ? {} : validation.errors);

  function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault();
    if (validation.ok) onSubmit?.(form);
  }

  return (
    <form aria-label="Task editor" onSubmit={submit}>
      <label>
        Title
        <input value={form.title} onChange={(event) => update({ title: event.currentTarget.value })} required />
      </label>
      <FieldError message={visibleErrors.title} />

      <label>
        Detail
        <textarea value={form.detail ?? ""} onChange={(event) => update({ detail: event.currentTarget.value })} />
      </label>
      <FieldError message={visibleErrors.detail} />

      <label>
        Status
        <select aria-describedby="task-status-bridge-note" disabled value={form.status} onChange={(event) => update({ status: event.currentTarget.value })}>
          {taskStatuses.map((status) => <option key={status} value={status}>{status.replace(/_/g, " ")}</option>)}
        </select>
      </label>
      <p id="task-status-bridge-note" className="muted-copy">Status is shown from persisted task state; status changes use the separate native status action.</p>
      <FieldError message={visibleErrors.status} />

      <label>
        Priority
        <select value={form.priority} onChange={(event) => update({ priority: event.currentTarget.value })}>
          {taskPriorities.map((priority) => <option key={priority} value={priority}>{priority}</option>)}
        </select>
      </label>
      <FieldError message={visibleErrors.priority} />

      <label>
        Workspace key
        <input value={form.workspace_key} onChange={(event) => update({ workspace_key: event.currentTarget.value })} required />
      </label>
      <FieldError message={visibleErrors.workspace_key} />

      <label>
        Metadata JSON
        <textarea value={form.metadata_json ?? "{}"} onChange={(event) => update({ metadata_json: event.currentTarget.value })} />
      </label>
      <FieldError message={visibleErrors.metadata_json} />

      <button className="primary-action" type="submit" disabled={!validation.ok}>{submitLabel}</button>
    </form>
  );
}

export function TaskWorkspace({
  state,
  form,
  formErrors,
  onFormChange,
  onCreateTask,
  onUpdateTask,
  onUpdateTaskStatus,
  onArchiveTask,
  onDeleteTask,
  onNewTask,
  onSelectTask,
  onRefresh,
  linkedPanels,
}: TaskWorkspaceProps) {
  const view = buildTaskWorkspaceView(state);
  const selectedTask = view.detail.kind === "task" ? view.detail.task : null;
  const currentForm = form ?? (selectedTask ? taskToForm(selectedTask) : createInitialTaskForm(""));
  const isInteractive = state.mode === "ready";
  const canShowTaskEditor = state.mode !== "loading";
  const unavailableCopy = "copy" in view.detail ? view.detail.copy : "Task detail is available after selecting a persisted task.";

  return (
    <section className="native-workspace task-workspace" aria-labelledby="task-workspace-heading">
      <header className="native-workspace-header">
        <div className="native-workspace-title-copy">
          <h2 id="task-workspace-heading">Tasks</h2>
          <p>{view.list.copy}</p>
          <p aria-live="polite">{view.list.statusLabel}</p>
        </div>
        <div className="native-workspace-actions">
          <button className="secondary-action" type="button" disabled={state.mode === "loading"} onClick={onNewTask}>New task</button>
          <button className="secondary-action" type="button" onClick={onRefresh}>Refresh real tasks</button>
        </div>
      </header>

      {!isInteractive ? (
        <section className="native-workspace-panel native-workspace-unavailable" aria-live="polite">
          <p className="eyebrow">Native backend</p>
          <h3>{view.detail.kind === "loading" ? "Loading persisted tasks" : "Task backend unavailable"}</h3>
          <p>{unavailableCopy}</p>
          <p className="muted-copy">No task list or detail is shown unless the real native bridge responds. The create form below still calls the native backend and never creates browser-only records.</p>
        </section>
      ) : (
        <div className="native-workspace-grid task-workspace-grid">
          <aside className="native-workspace-panel native-list-panel" aria-label="Task list">
            {view.list.items.length === 0 ? <p>{view.list.emptyCopy}</p> : null}
            <ul>
              {view.list.items.map((item) => (
                <li key={item.id} data-tone={item.tone} aria-current={item.isSelected ? "true" : undefined}>
                  <button type="button" onClick={() => onSelectTask?.(item.id)}>
                    <strong>{item.title}</strong>
                    <span>{item.meta}</span>
                  </button>
                </li>
              ))}
            </ul>
          </aside>

          <article className="native-workspace-panel native-workspace-detail" aria-label="Task detail">
            {selectedTask && "metadataPreview" in view.detail ? (
              <>
                <h3>{selectedTask.title}</h3>
                {selectedTask.detail ? <p>{selectedTask.detail}</p> : <p>No task detail was persisted.</p>}
                <dl className="native-detail-list">
                  {view.detail.detailLines.map((line) => {
                    const [term, ...rest] = line.split(": ");
                    return rest.length > 0 ? <div key={line}><dt>{term}</dt><dd>{rest.join(": ")}</dd></div> : <div key={line}><dt>State</dt><dd>{line}</dd></div>;
                  })}
                </dl>
                <form className="inline-action-form" aria-label="Update task status" onSubmit={(event) => {
                  event.preventDefault();
                  const formData = new FormData(event.currentTarget);
                  const status = String(formData.get("status") ?? selectedTask.status);
                  onUpdateTaskStatus?.(selectedTask.id, status);
                }}>
                  <label>
                    Native status action
                    <select name="status" defaultValue={selectedTask.status}>
                      {taskStatuses.map((status) => <option key={status} value={status}>{status.replace(/_/g, " ")}</option>)}
                    </select>
                  </label>
                  <button className="secondary-action" type="submit">Update status</button>
                </form>
                <div className="native-workspace-actions" aria-label="Task lifecycle actions">
                  <button className="secondary-action" type="button" onClick={() => onArchiveTask?.(selectedTask.id)}>Archive task</button>
                  <button className="secondary-action danger-action" type="button" onClick={() => onDeleteTask?.(selectedTask.id)}>Delete task</button>
                </div>
                <p className="muted-copy">Status, archive, and delete call {taskBridgeCommands.updateStatus}, {taskBridgeCommands.archive}, and {taskBridgeCommands.delete}; they are not browser-only state changes.</p>

                <h4>Metadata</h4>
                <pre>{view.detail.metadataPreview}</pre>
              </>
            ) : <p>{unavailableCopy}</p>}
          </article>
        </div>
      )}

      {selectedTask ? linkedPanels : null}

      {canShowTaskEditor ? <section className="native-workspace-panel native-editor-panel" aria-label="Create or edit task">
        <h3>{selectedTask ? "Edit persisted task" : "Create task"}</h3>
        <p>
          Create uses {taskBridgeCommands.create}; updates use {taskBridgeCommands.update}. The parent integration supplies the Tauri invoke calls; browser preview submissions fail closed instead of creating fake tasks.
        </p>
        <TaskForm
          form={currentForm}
          errors={formErrors}
          submitLabel={selectedTask ? "Update task" : "Create task"}
          onChange={onFormChange}
          onSubmit={(submittedForm) => selectedTask ? onUpdateTask?.(selectedTask.id, submittedForm) : onCreateTask?.(submittedForm)}
        />
      </section> : null}
    </section>
  );
}
