import { buildNoteWorkspaceView, createInitialNoteForm, noteBridgeCommands, validateNoteForm, type NoteRecord } from "./noteViewModel";

function assert(condition: unknown, message: string): asserts condition { if (!condition) throw new Error(message); }
function assertEqual<T>(actual: T, expected: T, message: string) { if (actual !== expected) throw new Error(`${message}: expected ${String(expected)}, got ${String(actual)}`); }

const note: NoteRecord = { id: "note-1", title: "Visible Note", slug: "visible-note", relative_path: "Notes/visible-note.md", status: "active", conflict_state: "clean", body_digest: "abc", metadata_json: '{"source":"test"}', markdown: "# Visible Note\n\nBody" };

assertEqual(noteBridgeCommands.create, "create_markdown_note_command", "create command matches backend");
assertEqual(noteBridgeCommands.list, "list_notes_command", "list command matches backend");
assertEqual(noteBridgeCommands.edit, "edit_markdown_note_command", "edit command matches backend");

const initial = createInitialNoteForm();
assertEqual(initial.relative_path, "Notes/untitled.md", "initial note path should be a backend-safe visible path");

const valid = validateNoteForm({ title: " Note ", body_markdown: " Body ", relative_path: "Notes/note.md", metadata_json: "{}" });
assert(valid.ok, "valid note form should pass");
assertEqual(valid.value.title, "Note", "title is trimmed");
assertEqual(valid.value.body_markdown, "Body", "markdown is trimmed");

const invalid = validateNoteForm({ title: " ", body_markdown: "", relative_path: "../escape.md", metadata_json: '{"apiToken":"secret"}' });
assert(!invalid.ok, "invalid note form should fail");
assert(invalid.errors.title?.includes("required"), "title required");
assert(invalid.errors.body_markdown?.includes("required"), "body required");
assert(invalid.errors.relative_path?.includes("safe"), "path safety enforced");
assert(invalid.errors.metadata_json?.includes("secret"), "secret metadata rejected");

const loading = buildNoteWorkspaceView({ mode: "loading", selectedNoteId: null });
assertEqual(loading.items.length, 0, "loading view must not fake notes");
const error = buildNoteWorkspaceView({ mode: "error", selectedNoteId: null, error: "native unavailable" });
assertEqual(error.statusLabel, "Note data unavailable", "error status explicit");
assert(error.detail.kind === "error" && error.detail.copy.includes("native unavailable"), "bridge error surfaced");
const empty = buildNoteWorkspaceView({ mode: "ready", selectedNoteId: null, notes: [], conflicts: [] });
assertEqual(empty.statusLabel, "No notes", "empty state truthful");
const populated = buildNoteWorkspaceView({ mode: "ready", selectedNoteId: "note-1", notes: [note], conflicts: [] });
assertEqual(populated.statusLabel, "1 note", "real note count shown");
assert(populated.detail.kind === "note" && populated.detail.note.markdown.includes("Body"), "selected note detail uses real markdown");
assert(populated.conflictsCopy.includes("No note conflicts"), "conflict state rendered truthfully");

console.log("noteViewModel tests passed");
