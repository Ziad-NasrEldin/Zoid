export type FileBrowseEntry = {
  root_key: string;
  relative_path: string;
  display_name: string;
  file_kind: string;
  mime_type?: string | null;
  byte_size?: number | null;
  is_directory: boolean;
  preview_available: boolean;
};

export type FilePreviewRecord = {
  root_key: string;
  relative_path: string;
  display_name: string;
  file_kind: string;
  mime_type?: string | null;
  byte_size: number;
  preview_text: string;
  truncated: boolean;
};

export type FileActionKind = "copy" | "rename" | "move" | "trash";
export type FileActionDraft = { action: FileActionKind; source_relative_path: string; destination_relative_path?: string | null; confirmation_id?: string | null };
export type FileActionRecord = { action: string; root_key: string; source_relative_path: string; destination_relative_path?: string | null };
export type PolicyPreviewRecord = { category: string; policy: string; reason: string; allowed_now: boolean; requires_confirmation: boolean; requires_reviewer: boolean; requires_clear_task: boolean; human_confirmation: string; reviewer_required: string };

export type FileWorkspaceState =
  | { mode: "loading"; rootKey: string; relativePath: string; selectedPath: string | null }
  | { mode: "error"; rootKey: string; relativePath: string; selectedPath: string | null; error: string }
  | { mode: "ready"; rootKey: string; relativePath: string; selectedPath: string | null; entries: FileBrowseEntry[]; preview: FilePreviewRecord | null; policy: PolicyPreviewRecord | null; actionResult?: FileActionRecord | null; actionError?: string | null };

export const fileBridgeCommands = {
  browse: "browse_files_command",
  open: "open_file_reference_command",
  preview: "preview_file_command",
  action: "perform_file_action_command",
  policy: "preview_action_policy",
} as const;

export const defaultFileRootKey = "zoid_visible";
export const defaultFileRelativePath = "Notes";

export function createInitialFileActionDraft(): FileActionDraft {
  return { action: "copy", source_relative_path: "Notes/source.md", destination_relative_path: "Notes/copy.md", confirmation_id: "" };
}

export function validateFileBrowse(rootKey: string, relativePath: string) {
  const errors: string[] = [];
  const root = rootKey.trim();
  const path = relativePath.trim();
  if (root !== defaultFileRootKey) errors.push("Root key must be zoid_visible for this visible workspace slice.");
  if (!path) errors.push("Relative path is required; backend rejects an empty browse path.");
  if (path.includes("..") || path.includes("\\") || path.startsWith("/")) errors.push("Relative path must stay inside the visible root.");
  return { ok: errors.length === 0, rootKey: root, relativePath: path, errors };
}

export function validateFileAction(draft: FileActionDraft) {
  const errors: string[] = [];
  const source = draft.source_relative_path.trim();
  const destination = draft.destination_relative_path?.trim() || null;
  if (!source || source.includes("..") || source.includes("\\") || source.startsWith("/")) errors.push("Source path must be a safe visible-root relative path.");
  if (["copy", "move", "rename"].includes(draft.action) && !destination) errors.push("Destination path is required for copy, move, and rename.");
  if (destination && (destination.includes("..") || destination.includes("\\") || destination.startsWith("/"))) errors.push("Destination path must be a safe visible-root relative path.");
  return { ok: errors.length === 0, errors, value: { action: draft.action, source_relative_path: source, destination_relative_path: destination, confirmation_id: draft.confirmation_id?.trim() || null } };
}

export function buildFileWorkspaceView(state: FileWorkspaceState) {
  if (state.mode === "loading") return { statusLabel: "Loading files", copy: `Browsing ${state.rootKey}:${state.relativePath} through ${fileBridgeCommands.browse}…`, items: [], detail: { kind: "loading" as const, copy: "No browser preview files are fabricated while loading." }, policyCopy: "Action policy unavailable while loading." };
  if (state.mode === "error") return { statusLabel: "File data unavailable", copy: "The native file bridge returned an error.", items: [], detail: { kind: "error" as const, copy: state.error }, policyCopy: state.error };
  const selected = state.selectedPath ? state.entries.find((entry) => entry.relative_path === state.selectedPath) : null;
  return {
    statusLabel: state.entries.length === 0 ? "No files" : `${state.entries.length} file${state.entries.length === 1 ? "" : "s"}`,
    copy: state.entries.length === 0 ? `No entries returned by ${fileBridgeCommands.browse} for ${state.relativePath}.` : `Showing entries returned by ${fileBridgeCommands.browse} for ${state.relativePath}.`,
    items: state.entries.map((entry) => ({ id: entry.relative_path, title: entry.display_name, meta: `${entry.file_kind}${entry.byte_size == null ? "" : ` · ${entry.byte_size} bytes`}`, isSelected: entry.relative_path === state.selectedPath, canPreview: entry.preview_available && !entry.is_directory })),
    detail: state.preview ? { kind: "preview" as const, preview: state.preview } : selected ? { kind: "entry" as const, entry: selected, copy: selected.is_directory ? "Directory selected. Browse it to list children." : "File selected. Preview uses the native bridge only." } : { kind: "empty" as const, copy: state.entries.length === 0 ? "No real file entries returned." : "Select a file to preview." },
    policyCopy: state.policy ? `${state.policy.category}: ${state.policy.requires_confirmation ? "confirmation required" : "no confirmation required"}; ${state.policy.reason}` : `Action policy not loaded from ${fileBridgeCommands.policy}.`,
    actionCopy: state.actionError ?? (state.actionResult ? `${state.actionResult.action} completed for ${state.actionResult.source_relative_path}.` : "No file action has run. Confirmation IDs must come from persisted approval state; the UI does not fake them."),
  };
}
