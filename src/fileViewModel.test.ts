import { buildFileWorkspaceView, createInitialFileActionDraft, defaultFileRelativePath, defaultFileRootKey, fileBridgeCommands, validateFileAction, validateFileBrowse, type FileBrowseEntry } from "./fileViewModel";

function assert(condition: unknown, message: string): asserts condition { if (!condition) throw new Error(message); }
function assertEqual<T>(actual: T, expected: T, message: string) { if (actual !== expected) throw new Error(`${message}: expected ${String(expected)}, got ${String(actual)}`); }

const entry: FileBrowseEntry = { root_key: "zoid_visible", relative_path: "Notes/source.md", display_name: "source.md", file_kind: "markdown", mime_type: "text/markdown", byte_size: 8, is_directory: false, preview_available: true };

assertEqual(fileBridgeCommands.browse, "browse_files_command", "browse command matches backend");
assertEqual(fileBridgeCommands.action, "perform_file_action_command", "action command matches backend");
assertEqual(defaultFileRootKey, "zoid_visible", "default root exists for backend root key validation");
assertEqual(defaultFileRelativePath, "Notes", "default path is non-empty because backend rejects empty relative path");

const validBrowse = validateFileBrowse("zoid_visible", "Notes");
assert(validBrowse.ok, "default browse input is valid");
const invalidBrowse = validateFileBrowse("bad", "");
assert(!invalidBrowse.ok && invalidBrowse.errors.length >= 2, "invalid browse input reports errors");

const draft = createInitialFileActionDraft();
assertEqual(draft.source_relative_path, "Notes/source.md", "initial action source is visible-root relative");
const validAction = validateFileAction({ action: "copy", source_relative_path: "Notes/source.md", destination_relative_path: "Notes/copy.md", confirmation_id: "decision-1" });
assert(validAction.ok, "copy action validates with destination");
assertEqual(validAction.value.confirmation_id, "decision-1", "confirmation ID is preserved when supplied");
const invalidAction = validateFileAction({ action: "copy", source_relative_path: "../source.md", destination_relative_path: "" });
assert(!invalidAction.ok, "unsafe action fails locally");

const loading = buildFileWorkspaceView({ mode: "loading", rootKey: "zoid_visible", relativePath: "Notes", selectedPath: null });
assertEqual(loading.items.length, 0, "loading view must not fake files");
const error = buildFileWorkspaceView({ mode: "error", rootKey: "zoid_visible", relativePath: "Notes", selectedPath: null, error: "native unavailable" });
assertEqual(error.statusLabel, "File data unavailable", "error status explicit");
const ready = buildFileWorkspaceView({ mode: "ready", rootKey: "zoid_visible", relativePath: "Notes", selectedPath: "Notes/source.md", entries: [entry], preview: null, policy: { category: "move_rename_copy_file", policy: "confirm", reason: "copy is consequential", allowed_now: false, requires_confirmation: true, requires_reviewer: false, requires_clear_task: false, human_confirmation: "required", reviewer_required: "none" }, actionResult: null, actionError: null });
assertEqual(ready.statusLabel, "1 file", "ready view shows real count");
assert(ready.policyCopy.includes("confirmation required"), "policy requirement surfaced");
assert(typeof ready.actionCopy === "string" && ready.actionCopy.includes("does not fake"), "confirmation ID is not faked");

console.log("fileViewModel tests passed");
