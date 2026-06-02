import type { FormEvent } from "react";
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
  onSelectTask?: (taskId: string) => void;
  onRefresh?: () => void;
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
  return <p role="alert">{message}</p>;
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
        <select value={form.status} onChange={(event) => update({ status: event.currentTarget.value })}>
          {taskStatuses.map((status) => <option key={status} value={status}>{status.replace(/_/g, " ")}</option>)}
        </select>
      </label>
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

      <button type="submit" disabled={!validation.ok}>{submitLabel}</button>
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
  onSelectTask,
  onRefresh,
}: TaskWorkspaceProps) {
  const view = buildTaskWorkspaceView(state);
  const selectedTask = view.detail.kind === "task" ? view.detail.task : null;
  const currentForm = form ?? (selectedTask ? taskToForm(selectedTask) : createInitialTaskForm(""));

  return (
    <section aria-labelledby="task-workspace-heading">
      <header>
        <h2 id="task-workspace-heading">Tasks</h2>
        <p>{view.list.copy}</p>
        <p aria-live="polite">{view.list.statusLabel}</p>
        <button type="button" onClick={onRefresh}>Refresh real tasks</button>
      </header>

      <aside aria-label="Task list">
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

      <article aria-label="Task detail">
        {view.detail.kind === "task" ? (
          <>
            <h3>{view.detail.task.title}</h3>
            {view.detail.task.detail ? <p>{view.detail.task.detail}</p> : <p>No task detail was persisted.</p>}
            <dl>
              {view.detail.detailLines.map((line) => {
                const [term, ...rest] = line.split(": ");
                return rest.length > 0 ? <div key={line}><dt>{term}</dt><dd>{rest.join(": ")}</dd></div> : <div key={line}><dt>State</dt><dd>{line}</dd></div>;
              })}
            </dl>
            <h4>Metadata</h4>
            <pre>{view.detail.metadataPreview}</pre>
          </>
        ) : <p>{view.detail.copy}</p>}
      </article>

      <section aria-label="Create or edit task">
        <h3>{selectedTask ? "Edit persisted task" : "Create task"}</h3>
        <p>
          Create uses {taskBridgeCommands.create}; updates use {taskBridgeCommands.update}. The parent integration supplies the Tauri invoke calls.
        </p>
        <TaskForm
          form={currentForm}
          errors={formErrors}
          submitLabel={selectedTask ? "Update task" : "Create task"}
          onChange={onFormChange}
          onSubmit={(submittedForm) => selectedTask ? onUpdateTask?.(selectedTask.id, submittedForm) : onCreateTask?.(submittedForm)}
        />
      </section>
    </section>
  );
}
