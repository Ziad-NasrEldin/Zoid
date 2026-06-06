export type NoteRecord = {
  id: string;
  title: string;
  slug: string;
  relative_path: string;
  status: string;
  conflict_state: string;
  body_digest: string;
  metadata_json: string;
  markdown: string;
};

export type NoteFormDraft = {
  title: string;
  body_markdown: string;
  relative_path?: string | null;
  metadata_json?: string | null;
};

export type NoteValidationErrors = Partial<Record<keyof NoteFormDraft, string>>;
export type NoteValidationResult =
  | { ok: true; value: Required<Pick<NoteFormDraft, "title" | "body_markdown">> & { relative_path: string | null; metadata_json: string }; errors: NoteValidationErrors }
  | { ok: false; errors: NoteValidationErrors };

export type NoteWorkspaceState =
  | { mode: "loading"; selectedNoteId: string | null }
  | { mode: "error"; selectedNoteId: string | null; error: string }
  | { mode: "ready"; selectedNoteId: string | null; notes: NoteRecord[]; conflicts: NoteConflictRecord[]; scan?: NoteScanResult | null };

export type NoteScanResult = {
  scanned_files: number;
  indexed_notes: number;
  frontmatter_written: number;
  conflicted_notes: number;
  missing_notes_marked: number;
};

export type NoteConflictRecord = {
  id: string;
  title: string;
  relative_path: string;
  conflict_state: string;
  detected_relative_path?: string | null;
  stored_digest: string;
  disk_digest: string;
  metadata_json: string;
};

export const noteBridgeCommands = {
  create: "create_markdown_note_command",
  read: "read_note_command",
  list: "list_notes_command",
  edit: "edit_markdown_note_command",
  trash: "trash_markdown_note_command",
  scan: "scan_markdown_notes_command",
  conflicts: "list_note_conflicts_command",
  acceptConflict: "accept_note_conflict_command",
} as const;

const relativePathPattern = /^(?:Notes|Files)(?:\/[A-Za-z0-9][A-Za-z0-9._ -]*)*\.md$/;
const secretKeyPattern = /(?:secret|token|api[_-]?key|password|passwd|credential|private[_-]?key|authorization|bearer)/i;

export function createInitialNoteForm(): NoteFormDraft {
  return { title: "", body_markdown: "", relative_path: "Notes/untitled.md", metadata_json: "{}" };
}

export function validateNoteForm(draft: NoteFormDraft): NoteValidationResult {
  const errors: NoteValidationErrors = {};
  const title = draft.title.trim();
  const body = draft.body_markdown.trim();
  const relativePath = draft.relative_path?.trim() || null;
  const metadataJson = draft.metadata_json?.trim() || "{}";

  if (!title) errors.title = "Note title is required.";
  if (!body) errors.body_markdown = "Markdown body is required.";
  if (relativePath && (!relativePathPattern.test(relativePath) || relativePath.includes("..") || relativePath.includes("\\"))) {
    errors.relative_path = "Relative path must be a safe Markdown path under Notes or Files.";
  }

  try {
    const parsed = JSON.parse(metadataJson);
    if (parsed === null || Array.isArray(parsed) || typeof parsed !== "object") errors.metadata_json = "Metadata must be a valid JSON object.";
    if (Object.entries(parsed as Record<string, unknown>).some(([key, value]) => secretKeyPattern.test(key) || (typeof value === "string" && secretKeyPattern.test(value)))) {
      errors.metadata_json = "Metadata contains secret-looking keys or values.";
    }
  } catch {
    errors.metadata_json = "Metadata must be valid JSON.";
  }

  if (Object.keys(errors).length > 0) return { ok: false, errors };
  return { ok: true, errors: {}, value: { title, body_markdown: body, relative_path: relativePath, metadata_json: metadataJson } };
}

function newestNotes(notes: NoteRecord[]) {
  return [...notes].sort((a, b) => a.relative_path.localeCompare(b.relative_path) || a.id.localeCompare(b.id));
}

export function buildNoteWorkspaceView(state: NoteWorkspaceState) {
  if (state.mode === "loading") return { statusLabel: "Loading notes", copy: `Reading Markdown notes through ${noteBridgeCommands.list}…`, items: [], detail: { kind: "loading" as const, copy: "No browser preview notes are fabricated while loading." }, conflictsCopy: "Conflicts unavailable while loading." };
  if (state.mode === "error") return { statusLabel: "Note data unavailable", copy: "The native note bridge returned an error.", items: [], detail: { kind: "error" as const, copy: state.error }, conflictsCopy: state.error };

  const visible = newestNotes(state.notes.filter((note) => note.status !== "trashed" && note.status !== "deleted"));
  const selected = state.selectedNoteId ? visible.find((note) => note.id === state.selectedNoteId) : null;
  return {
    statusLabel: visible.length === 0 ? "No notes" : `${visible.length} note${visible.length === 1 ? "" : "s"}`,
    copy: visible.length === 0 ? `No active Markdown notes returned by ${noteBridgeCommands.list}.` : `Showing persisted Markdown notes from ${noteBridgeCommands.list}.`,
    items: visible.map((note) => ({ id: note.id, title: note.title, meta: `${note.relative_path} · ${note.conflict_state}`, isSelected: note.id === state.selectedNoteId })),
    detail: selected ? { kind: "note" as const, note: selected, metadataPreview: formatNoteMetadata(selected.metadata_json) } : state.selectedNoteId ? { kind: "missing" as const, copy: `Note ${state.selectedNoteId} was not returned by the native bridge.` } : { kind: "empty" as const, copy: visible.length === 0 ? "Scan or create a note; this view never invents sample notes." : "Select a note to view Markdown." },
    conflictsCopy: state.conflicts.length === 0 ? "No note conflicts returned." : `${state.conflicts.length} conflict${state.conflicts.length === 1 ? "" : "s"} returned by ${noteBridgeCommands.conflicts}.`,
    scanCopy: state.scan ? `Scan indexed ${state.scan.indexed_notes} notes from ${state.scan.scanned_files} files.` : "Scan has not run in this UI session.",
  };
}

export function formatNoteMetadata(metadataJson: string): string {
  try { return JSON.stringify(JSON.parse(metadataJson || "{}"), null, 2); } catch { return metadataJson; }
}
