import { createInitialFileActionDraft, defaultFileRelativePath, defaultFileRootKey, fileBridgeCommands, validateFileAction, validateFileBrowse, type FileActionDraft, type FileActionRecord, type FileBrowseEntry, type FilePreviewRecord, type FileWorkspaceState, type PolicyPreviewRecord } from "./fileViewModel";

export type FileBridgeInvoke = <T = unknown>(command: string, args?: Record<string, unknown>) => Promise<T>;
export type FileBridgeUiState = { rootKey: string; relativePath: string; actionDraft: FileActionDraft; actionErrors: string[]; state: FileWorkspaceState };

export function createInitialFileBridgeState(): FileBridgeUiState {
  return { rootKey: defaultFileRootKey, relativePath: defaultFileRelativePath, actionDraft: createInitialFileActionDraft(), actionErrors: [], state: { mode: "loading", rootKey: defaultFileRootKey, relativePath: defaultFileRelativePath, selectedPath: null } };
}

function bridgeError(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "unknown native file bridge error";
}

async function loadPolicy(invoke: FileBridgeInvoke): Promise<PolicyPreviewRecord | null> {
  try {
    return await invoke<PolicyPreviewRecord>(fileBridgeCommands.policy, { request: { category: "move_rename_copy_file" } });
  } catch {
    return null;
  }
}

export async function browseFilesFromBridge(invoke: FileBridgeInvoke, options: { rootKey: string; relativePath: string; selectedPath: string | null }): Promise<FileWorkspaceState> {
  const validation = validateFileBrowse(options.rootKey, options.relativePath);
  if (!validation.ok) return { mode: "error", rootKey: options.rootKey, relativePath: options.relativePath, selectedPath: options.selectedPath, error: validation.errors.join(" ") };
  try {
    const [entries, policy] = await Promise.all([
      invoke<FileBrowseEntry[]>(fileBridgeCommands.browse, { request: { root_key: validation.rootKey, relative_path: validation.relativePath } }),
      loadPolicy(invoke),
    ]);
    const selectedPath = options.selectedPath && entries.some((entry) => entry.relative_path === options.selectedPath) ? options.selectedPath : null;
    return { mode: "ready", rootKey: validation.rootKey, relativePath: validation.relativePath, selectedPath, entries, preview: null, policy, actionResult: null, actionError: null };
  } catch (error) {
    return { mode: "error", rootKey: validation.rootKey, relativePath: validation.relativePath, selectedPath: options.selectedPath, error: bridgeError(error) };
  }
}

export async function previewFileThroughBridge(invoke: FileBridgeInvoke, state: FileWorkspaceState, relativePath: string): Promise<FileWorkspaceState> {
  try {
    const preview = await invoke<FilePreviewRecord>(fileBridgeCommands.preview, { request: { root_key: state.rootKey, relative_path: relativePath } });
    if (state.mode === "ready") return { ...state, selectedPath: relativePath, preview };
    return { mode: "ready", rootKey: state.rootKey, relativePath: state.relativePath, selectedPath: relativePath, entries: [], preview, policy: null };
  } catch (error) {
    return { mode: "error", rootKey: state.rootKey, relativePath: state.relativePath, selectedPath: relativePath, error: bridgeError(error) };
  }
}

export async function performFileActionThroughBridge(invoke: FileBridgeInvoke, current: FileBridgeUiState): Promise<FileBridgeUiState> {
  const validation = validateFileAction(current.actionDraft);
  if (!validation.ok) return { ...current, actionErrors: validation.errors, state: { mode: "error", rootKey: current.rootKey, relativePath: current.relativePath, selectedPath: current.state.selectedPath, error: "File action has validation errors. No native command was called." } };
  try {
    const result = await invoke<FileActionRecord>(fileBridgeCommands.action, { request: { action: validation.value.action, root_key: current.rootKey, source_relative_path: validation.value.source_relative_path, destination_relative_path: validation.value.destination_relative_path, confirmation_id: validation.value.confirmation_id } });
    const refreshed = await browseFilesFromBridge(invoke, { rootKey: current.rootKey, relativePath: current.relativePath, selectedPath: result.destination_relative_path ?? result.source_relative_path });
    return { ...current, actionErrors: [], state: refreshed.mode === "ready" ? { ...refreshed, actionResult: result } : refreshed };
  } catch (error) {
    const message = bridgeError(error);
    return { ...current, actionErrors: [], state: current.state.mode === "ready" ? { ...current.state, actionError: message } : { mode: "error", rootKey: current.rootKey, relativePath: current.relativePath, selectedPath: current.state.selectedPath, error: message } };
  }
}
