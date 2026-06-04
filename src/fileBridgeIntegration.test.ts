import { browseFilesFromBridge, createInitialFileBridgeState, performFileActionThroughBridge, previewFileThroughBridge, type FileBridgeInvoke } from "./fileBridgeIntegration";
import type { FileBrowseEntry, FilePreviewRecord } from "./fileViewModel";

function assert(condition: unknown, message: string): asserts condition { if (!condition) throw new Error(message); }
function assertEqual<T>(actual: T, expected: T, message: string) { if (actual !== expected) throw new Error(`${message}: expected ${String(expected)}, got ${String(actual)}`); }

const entry: FileBrowseEntry = { root_key: "zoid_visible", relative_path: "Files/source.md", display_name: "source.md", file_kind: "markdown", mime_type: "text/markdown", byte_size: 8, is_directory: false, preview_available: true };
const preview: FilePreviewRecord = { ...entry, byte_size: 8, preview_text: "# Source", truncated: false };

function makeInvoke(responses: Record<string, unknown[]>) {
  const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
  const invoke: FileBridgeInvoke = async <T = unknown>(command: string, args?: Record<string, unknown>) => {
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
  const { invoke, calls } = makeInvoke({ browse_files_command: [[entry]], preview_action_policy: [{ category: "move_rename_copy_file", policy: "confirm", reason: "copy needs persisted confirmation", allowed_now: false, requires_confirmation: true, requires_reviewer: false, requires_clear_task: false, human_confirmation: "required", reviewer_required: "none" }] });
  const state = await browseFilesFromBridge(invoke, { rootKey: "zoid_visible", relativePath: "Files", selectedPath: null });
  assertEqual(state.mode, "ready", "browse returns ready state");
  assert(state.mode === "ready" && state.entries[0]?.relative_path === "Files/source.md", "browse uses bridge entries");
  assertEqual(calls[0]?.command, "browse_files_command", "browse uses native command");
  assertEqual((calls[0]?.args?.request as Record<string, unknown>).relative_path, "Files", "browse passes non-empty relative path");
  assertEqual(calls[1]?.command, "preview_action_policy", "browse surfaces action policy");
}

{
  const { invoke } = makeInvoke({ browse_files_command: [new Error("native file bridge unavailable")], preview_action_policy: [new Error("policy unavailable")] });
  const state = await browseFilesFromBridge(invoke, { rootKey: "zoid_visible", relativePath: "Files", selectedPath: null });
  assertEqual(state.mode, "error", "bridge failure should be explicit");
  assert(state.mode === "error" && state.error.includes("unavailable"), "bridge error surfaced");
}

{
  const initial = await browseFilesFromBridge(makeInvoke({ browse_files_command: [[entry]], preview_action_policy: [new Error("policy unavailable")] }).invoke, { rootKey: "zoid_visible", relativePath: "Files", selectedPath: null });
  const { invoke, calls } = makeInvoke({ preview_file_command: [preview] });
  const state = await previewFileThroughBridge(invoke, initial, "Files/source.md");
  assert(state.mode === "ready" && state.preview?.preview_text === "# Source", "preview uses native text");
  assertEqual(calls[0]?.command, "preview_file_command", "preview uses native command");
}

{
  const { invoke, calls } = makeInvoke({ perform_file_action_command: [new Error("confirmation_required") ] });
  const current = createInitialFileBridgeState();
  current.state = { mode: "ready", rootKey: "zoid_visible", relativePath: "Files", selectedPath: "Files/source.md", entries: [entry], preview: null, policy: null };
  current.actionDraft = { action: "copy", source_relative_path: "Files/source.md", destination_relative_path: "Files/copy.md", confirmation_id: "" };
  const next = await performFileActionThroughBridge(invoke, current);
  assertEqual(calls[0]?.command, "perform_file_action_command", "action uses native command even without confirmation id");
  assert(next.state.mode === "ready" && next.state.actionError?.includes("confirmation_required"), "required persisted confirmation is surfaced, not faked");
  assertEqual(((calls[0]?.args?.request as Record<string, unknown>).confirmation_id ?? null) as string | null, null, "blank confirmation id is sent as null");
}

{
  const { invoke, calls } = makeInvoke({ perform_file_action_command: [{ action: "copy", root_key: "zoid_visible", source_relative_path: "Files/source.md", destination_relative_path: "Files/copy.md" }], browse_files_command: [[{ ...entry, relative_path: "Files/copy.md", display_name: "copy.md" }]], preview_action_policy: [new Error("policy unavailable")] });
  const current = createInitialFileBridgeState();
  current.state = { mode: "ready", rootKey: "zoid_visible", relativePath: "Files", selectedPath: null, entries: [entry], preview: null, policy: null };
  current.actionDraft = { action: "copy", source_relative_path: "Files/source.md", destination_relative_path: "Files/copy.md", confirmation_id: "decision-1" };
  const next = await performFileActionThroughBridge(invoke, current);
  assert(next.state.mode === "ready" && next.state.actionResult?.action === "copy", "confirmed action result surfaced after refresh");
  assertEqual((calls[0]?.args?.request as Record<string, unknown>).confirmation_id, "decision-1", "persisted confirmation id is passed through when provided");
}

console.log("fileBridgeIntegration tests passed");
