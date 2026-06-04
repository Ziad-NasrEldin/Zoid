import { createInitialNoteForm, noteBridgeCommands, validateNoteForm, type NoteConflictRecord, type NoteFormDraft, type NoteRecord, type NoteScanResult, type NoteValidationErrors, type NoteWorkspaceState } from "./noteViewModel";

export type NoteBridgeInvoke = <T = unknown>(command: string, args?: Record<string, unknown>) => Promise<T>;
export type NoteBridgeUiState = { form: NoteFormDraft; formErrors: NoteValidationErrors; state: NoteWorkspaceState };

export function createInitialNoteBridgeState(): NoteBridgeUiState {
  return { form: createInitialNoteForm(), formErrors: {}, state: { mode: "loading", selectedNoteId: null } };
}

function bridgeError(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "unknown native note bridge error";
}

function selectVisible(notes: NoteRecord[], requested: string | null) {
  return requested && notes.some((note) => note.id === requested) ? requested : null;
}

export function formDraftForNote(note: NoteRecord): NoteFormDraft {
  return { title: note.title, body_markdown: note.markdown, relative_path: note.relative_path, metadata_json: note.metadata_json || "{}" };
}

export async function refreshNotesFromBridge(invoke: NoteBridgeInvoke, options: { selectedNoteId: string | null }): Promise<NoteWorkspaceState> {
  try {
    const [notes, conflicts] = await Promise.all([
      invoke<NoteRecord[]>(noteBridgeCommands.list, { request: { status: "active", include_markdown: true, limit: 50 } }),
      invoke<NoteConflictRecord[]>(noteBridgeCommands.conflicts),
    ]);
    return { mode: "ready", selectedNoteId: selectVisible(notes, options.selectedNoteId), notes, conflicts, scan: null };
  } catch (error) {
    return { mode: "error", selectedNoteId: options.selectedNoteId, error: bridgeError(error) };
  }
}

export async function selectNoteThroughBridge(invoke: NoteBridgeInvoke, noteId: string): Promise<NoteWorkspaceState> {
  try {
    const note = await invoke<NoteRecord>(noteBridgeCommands.read, { noteId });
    const refreshed = await refreshNotesFromBridge(invoke, { selectedNoteId: note.id });
    if (refreshed.mode !== "ready" || refreshed.notes.some((candidate) => candidate.id === note.id)) return refreshed;
    return { ...refreshed, selectedNoteId: note.id, notes: [note, ...refreshed.notes] };
  } catch (error) {
    return { mode: "error", selectedNoteId: noteId, error: bridgeError(error) };
  }
}

export async function createNoteThroughBridge(invoke: NoteBridgeInvoke, form: NoteFormDraft): Promise<NoteBridgeUiState> {
  const validation = validateNoteForm(form);
  if (!validation.ok) return { form, formErrors: validation.errors, state: { mode: "error", selectedNoteId: null, error: "Note form has validation errors. No native command was called." } };
  try {
    const created = await invoke<NoteRecord>(noteBridgeCommands.create, { request: validation.value });
    return { form: createInitialNoteForm(), formErrors: {}, state: await refreshNotesFromBridge(invoke, { selectedNoteId: created.id }) };
  } catch (error) {
    return { form, formErrors: {}, state: { mode: "error", selectedNoteId: null, error: bridgeError(error) } };
  }
}

export async function editNoteThroughBridge(invoke: NoteBridgeInvoke, noteId: string, form: NoteFormDraft): Promise<NoteBridgeUiState> {
  const validation = validateNoteForm(form);
  if (!validation.ok) return { form, formErrors: validation.errors, state: { mode: "error", selectedNoteId: noteId, error: "Note form has validation errors. No native command was called." } };
  try {
    const edited = await invoke<NoteRecord>(noteBridgeCommands.edit, { noteId, request: { markdown: validation.value.body_markdown } });
    return { form: formDraftForNote(edited), formErrors: {}, state: await refreshNotesFromBridge(invoke, { selectedNoteId: edited.id }) };
  } catch (error) {
    return { form, formErrors: {}, state: { mode: "error", selectedNoteId: noteId, error: bridgeError(error) } };
  }
}

export async function scanNotesThroughBridge(invoke: NoteBridgeInvoke, selectedNoteId: string | null): Promise<NoteWorkspaceState> {
  try {
    const scan = await invoke<NoteScanResult>(noteBridgeCommands.scan);
    const refreshed = await refreshNotesFromBridge(invoke, { selectedNoteId });
    return refreshed.mode === "ready" ? { ...refreshed, scan } : refreshed;
  } catch (error) {
    return { mode: "error", selectedNoteId, error: bridgeError(error) };
  }
}

export async function trashNoteThroughBridge(invoke: NoteBridgeInvoke, noteId: string): Promise<NoteWorkspaceState> {
  try {
    await invoke<NoteRecord>(noteBridgeCommands.trash, { noteId });
    return await refreshNotesFromBridge(invoke, { selectedNoteId: null });
  } catch (error) {
    return { mode: "error", selectedNoteId: noteId, error: bridgeError(error) };
  }
}
