import {
  createContentLinkedPanelsViewModel,
  createIdleContentLinkedPanelsState,
  fileReferenceEntityId,
  loadContentLinkedPanelsFromBridge,
  type ContentLinkedPanelsInvoke,
} from "./contentLinkedPanels";

function assert(condition: unknown, message: string): asserts condition {
  if (!condition) throw new Error(message);
}

function assertEqual<T>(actual: T, expected: T, message: string) {
  if (actual !== expected) throw new Error(`${message}: expected ${String(expected)}, got ${String(actual)}`);
}

async function testLoadsHistoryAndEntityLinksWithoutFallbacks() {
  const calls: Array<{ command: string; args?: Record<string, unknown> }> = [];
  const invoke: ContentLinkedPanelsInvoke = async <T = unknown>(command: string, args?: Record<string, unknown>) => {
    calls.push({ command, args });
    if (command === "list_entity_history_command") return [] as T;
    if (command === "list_content_entity_links_by_source_command") return [{
      id: "entity_link_1",
      source_type: "note",
      source_id: "note_1",
      target_type: "task",
      target_id: "task_1",
      relation_type: "mentions",
      created_by_actor_type: "user",
      metadata_json: "{}",
    }] as T;
    throw new Error(`unexpected command ${command}`);
  };

  const state = await loadContentLinkedPanelsFromBridge(invoke, "note", "note_1");
  assertEqual(state.mode, "ready", "bridge state should be ready");
  assertEqual(calls[0].command, "list_entity_history_command", "first command should load history");
  assertEqual(calls[1].command, "list_content_entity_links_by_source_command", "second command should load links");
  assert(JSON.stringify(calls[0].args) === JSON.stringify({ request: { entity_type: "note", entity_id: "note_1", include_related: true, limit: 25 } }), "history request should use selected note id");
  assert(JSON.stringify(calls[1].args) === JSON.stringify({ request: { entity_type: "note", entity_id: "note_1" } }), "link request should use selected note id");

  const view = createContentLinkedPanelsViewModel(state);
  assertEqual(view.linkPanel.items.length, 1, "one entity link should render");
  assertEqual(view.linkPanel.items[0].title, "Mentions Task", "relation and target type should be humanized");
}

function testIdleStateIsTruthful() {
  const view = createContentLinkedPanelsViewModel(createIdleContentLinkedPanelsState("file"));
  assertEqual(view.entityId, null, "idle state has no selected entity");
  assertEqual(view.linkPanel.items.length, 0, "idle state has no fabricated links");
  assert(/Select a real item/.test(view.linkPanel.emptyCopy), "idle copy should be truthful");
}

function testFileReferenceIdMatchesBackendHashShape() {
  assertEqual(fileReferenceEntityId("visible", "Notes/demo.md"), "file_ref_2083b11f3ce17d0b", "file id should match Rust fnv1a64 shape");
}

await testLoadsHistoryAndEntityLinksWithoutFallbacks();
testIdleStateIsTruthful();
testFileReferenceIdMatchesBackendHashShape();
console.log("contentLinkedPanels tests passed");
