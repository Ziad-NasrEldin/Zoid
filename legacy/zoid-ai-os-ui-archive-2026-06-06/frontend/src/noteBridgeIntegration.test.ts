import { createInitialNoteBridgeState, createNoteThroughBridge, editNoteThroughBridge, refreshNotesFromBridge, scanNotesThroughBridge, selectNoteThroughBridge, type NoteBridgeInvoke } from "./noteBridgeIntegration";
import type { NoteRecord } from "./noteViewModel";

function assert(condition: unknown, message: string): asserts condition { if (!condition) throw new Error(message); }
function assertEqual<T>(actual: T, expected: T, message: string) { if (actual !== expected) throw new Error(`${message}: expected ${String(expected)}, got ${String(actual)}`); }

const note: NoteRecord = { id: "note-1", title: "Bridge Note", slug: "bridge-note", relative_path: "Notes/bridge-note.md", status: "active", conflict_state: "clean", body_digest: "abc", metadata_json: "{}", markdown: "# Bridge Note" };

function makeInvoke(responses: Record<string, unknown[]>) {
  const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
  const invoke: NoteBridgeInvoke = async <T = unknown>(command: string, args?: Record<string, unknown>) => {
    calls.push({ command, args });
    const queue = responses[command] ?? [];
    if (queue.length === 0) throw new Error(`unexpected command ${command}`);
    const response = queue.shift();
    if (response instanceof Error) throw response;
    return response as T;
  };
  return { invoke, calls };
}

{
  const { invoke, calls } = makeInvoke({ list_notes_command: [[note]], list_note_conflicts_command: [[]] });
  const state = await refreshNotesFromBridge(invoke, { selectedNoteId: null });
  assertEqual(state.mode, "ready", "refresh should be ready");
  assert(state.mode === "ready" && state.notes[0]?.id === "note-1", "refresh uses bridge notes");
  assertEqual(calls[0]?.command, "list_notes_command", "refresh lists notes");
  assertEqual((calls[0]?.args?.request as Record<string, unknown>).include_markdown, true, "refresh asks backend for markdown truthfully");
}

{
  const { invoke } = makeInvoke({ list_notes_command: [new Error("native note bridge unavailable")], list_note_conflicts_command: [[]] });
  const state = await refreshNotesFromBridge(invoke, { selectedNoteId: "note-1" });
  assertEqual(state.mode, "error", "bridge failure should be explicit");
  assert(state.mode === "error" && state.error.includes("unavailable"), "bridge error surfaced");
}

{
  const created = { ...note, id: "created", title: "Created" };
  const { invoke, calls } = makeInvoke({ create_markdown_note_command: [created], list_notes_command: [[created]], list_note_conflicts_command: [[]] });
  const result = await createNoteThroughBridge(invoke, { title: " Created ", body_markdown: " Body ", relative_path: "Notes/created.md", metadata_json: "{}" });
  assertEqual(result.state.selectedNoteId, "created", "created note selected");
  assertEqual(calls[0]?.command, "create_markdown_note_command", "create uses native command");
  assertEqual((calls[0]?.args?.request as Record<string, unknown>).title, "Created", "create payload is validated");
}

{
  const edited = { ...note, markdown: "Edited body" };
  const { invoke, calls } = makeInvoke({ edit_markdown_note_command: [edited], list_notes_command: [[edited]], list_note_conflicts_command: [[]] });
  const result = await editNoteThroughBridge(invoke, "note-1", { title: "Bridge Note", body_markdown: "Edited body", relative_path: "Notes/bridge-note.md", metadata_json: "{}" });
  assertEqual(result.form.body_markdown, "Edited body", "edit hydrates returned markdown");
  assertEqual(calls[0]?.args?.noteId, "note-1", "edit passes camel-case noteId arg expected by Tauri invoke");
}

{
  const { invoke, calls } = makeInvoke({ read_note_command: [note], list_notes_command: [[note]], list_note_conflicts_command: [[]] });
  const state = await selectNoteThroughBridge(invoke, "note-1");
  assertEqual(state.selectedNoteId, "note-1", "select chooses note");
  assertEqual(calls[0]?.command, "read_note_command", "select reads native detail first");
}

{
  const { invoke, calls } = makeInvoke({ scan_markdown_notes_command: [{ scanned_files: 2, indexed_notes: 1, frontmatter_written: 0, conflicted_notes: 0, missing_notes_marked: 0 }], list_notes_command: [[note]], list_note_conflicts_command: [[]] });
  const state = await scanNotesThroughBridge(invoke, null);
  assert(state.mode === "ready" && state.scan?.scanned_files === 2, "scan result is surfaced after refresh");
  assertEqual(calls[0]?.command, "scan_markdown_notes_command", "scan uses native command");
}

{
  const { invoke, calls } = makeInvoke({ create_markdown_note_command: [note] });
  const result = await createNoteThroughBridge(invoke, { ...createInitialNoteBridgeState().form, title: " ", relative_path: "../bad.md" });
  assertEqual(result.state.mode, "error", "invalid create returns local error");
  assertEqual(calls.length, 0, "invalid create does not call bridge");
}

console.log("noteBridgeIntegration tests passed");
