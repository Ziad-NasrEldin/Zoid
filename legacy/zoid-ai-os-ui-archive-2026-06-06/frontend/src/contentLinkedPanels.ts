import { createHistoryTimelineViewModel, sanitizeMessage, type HistoryTimelineRecord, type HistoryTimelineViewModel } from "./historyTimelineViewModel";

export type ContentLinkedPanelsInvoke = <T = unknown>(command: string, args?: Record<string, unknown>) => Promise<T>;
export type ContentEntityType = "note" | "file";

export type EntityLinkRecord = {
  id: string;
  source_type: string;
  source_id: string;
  target_type: string;
  target_id: string;
  relation_type: string;
  created_by_actor_type: string;
  metadata_json: string;
};

export type ContentLinkedPanelsState =
  | { mode: "idle"; entityType: ContentEntityType; entityId: string | null }
  | { mode: "loading"; entityType: ContentEntityType; entityId: string }
  | { mode: "error"; entityType: ContentEntityType; entityId: string; error: string }
  | { mode: "ready"; entityType: ContentEntityType; entityId: string; history: HistoryTimelineRecord[]; links: EntityLinkRecord[] };

export type ContentLinkItem = { id: string; title: string; meta: string; summary: string };
export type ContentLinkedPanelsViewModel = {
  title: string;
  entityId: string | null;
  isLoading: boolean;
  errorMessage: string | null;
  linkPanel: { title: string; emptyCopy: string; items: ContentLinkItem[] };
  historyPanel: HistoryTimelineViewModel;
};

const PAGE_SIZE = 25;

export function createIdleContentLinkedPanelsState(entityType: ContentEntityType): ContentLinkedPanelsState {
  return { mode: "idle", entityType, entityId: null };
}

export function fileReferenceEntityId(rootKey: string, relativePath: string): string {
  return `file_ref_${fnv1a64Hex(`${rootKey}:${relativePath}`)}`;
}

export function fnv1a64Hex(value: string): string {
  let hash = 0xcbf29ce484222325n;
  const prime = 0x100000001b3n;
  const mask = 0xffffffffffffffffn;
  for (const char of new TextEncoder().encode(value)) {
    hash ^= BigInt(char);
    hash = (hash * prime) & mask;
  }
  return hash.toString(16).padStart(16, "0");
}

export async function loadContentLinkedPanelsFromBridge(
  invoke: ContentLinkedPanelsInvoke,
  entityType: ContentEntityType,
  entityId: string,
): Promise<ContentLinkedPanelsState> {
  try {
    const [history, links] = await Promise.all([
      invoke<HistoryTimelineRecord[]>("list_entity_history_command", {
        request: { entity_type: entityType, entity_id: entityId, include_related: true, limit: PAGE_SIZE },
      }),
      invoke<EntityLinkRecord[]>("list_content_entity_links_by_source_command", {
        request: { entity_type: entityType, entity_id: entityId },
      }),
    ]);
    return { mode: "ready", entityType, entityId, history, links };
  } catch (error) {
    return { mode: "error", entityType, entityId, error: bridgeError(error) };
  }
}

export function createContentLinkedPanelsViewModel(state: ContentLinkedPanelsState): ContentLinkedPanelsViewModel {
  const records = state.mode === "ready" ? state.history : [];
  const links = state.mode === "ready" ? state.links : [];
  const entityId = state.entityId;
  return {
    title: `${humanize(state.entityType)} activity and links`,
    entityId,
    isLoading: state.mode === "loading",
    errorMessage: state.mode === "error" ? sanitizeMessage(state.error) : null,
    linkPanel: {
      title: "Entity links",
      emptyCopy: state.mode === "ready" ? "No task, product, or run links are recorded for this item yet." : "Select a real item to load links.",
      items: links.map(linkItem),
    },
    historyPanel: createHistoryTimelineViewModel({
      mode: "entity",
      primary: { entity_type: state.entityType, entity_id: entityId || "unselected" },
      records,
      status: state.mode === "loading" ? "loading" : state.mode === "error" ? "error" : "ready",
      error: state.mode === "error" ? state.error : null,
      pageSize: PAGE_SIZE,
      includeRelated: true,
    }),
  };
}

function linkItem(link: EntityLinkRecord): ContentLinkItem {
  return {
    id: link.id,
    title: `${humanize(link.relation_type)} ${humanize(link.target_type)}`,
    meta: `${link.target_type} ${link.target_id}`,
    summary: sanitizeMessage(link.metadata_json || "{}"),
  };
}

function bridgeError(error: unknown): string {
  if (error instanceof Error) return error.message;
  if (typeof error === "string") return error;
  return "unknown content links bridge error";
}

function humanize(value: string): string {
  return value.replace(/[_-]+/g, " ").replace(/\b\w/g, (letter) => letter.toUpperCase());
}
