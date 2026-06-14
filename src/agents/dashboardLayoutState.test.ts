import { strict as assert } from "node:assert";
import { AGENT_DASHBOARD_MAX_TILES, AGENT_DASHBOARD_STORAGE_KEY, applyDraggedSessionToDashboard, loadAgentDashboardState, sanitizeAgentDashboardState, saveAgentDashboardState } from "./dashboardLayoutState";

const valid = ["a", "b", "c", "d", "e"];

assert.deepEqual(sanitizeAgentDashboardState({ version: 99, tiledSessionIds: ["a"] }, valid).tiledSessionIds, [], "unknown versions fall back");
assert.deepEqual(sanitizeAgentDashboardState({ version: 1, tiledSessionIds: ["a", "missing", "a", "b", "c", "d", "e"], primarySessionId: "missing", focusedSessionId: "b", layoutMode: "quad", autoPrioritize: true }, valid), {
  version: 1,
  tiledSessionIds: ["a", "b", "c", "d"],
  primarySessionId: "a",
  focusedSessionId: "b",
  layoutMode: "quad",
  autoPrioritize: true,
}, "state is deduped, capped, and missing sessions are removed");
assert.equal(sanitizeAgentDashboardState({ version: 1, tiledSessionIds: valid, layoutMode: "bad" }, valid).tiledSessionIds.length, AGENT_DASHBOARD_MAX_TILES, "tiles are capped at four");
assert.equal(sanitizeAgentDashboardState({ version: 1, tiledSessionIds: ["a", "b"], layoutMode: "quad" }, valid).layoutMode, "split-2", "removed manual layout choices should be ignored and derived from tile count");

assert.deepEqual(applyDraggedSessionToDashboard({ version: 1, tiledSessionIds: [], layoutMode: "auto", autoPrioritize: true }, "b", "a", valid), {
  version: 1,
  tiledSessionIds: ["a", "b"],
  primarySessionId: "a",
  focusedSessionId: "b",
  layoutMode: "split-2",
  autoPrioritize: false,
}, "dropping a session onto the single main chat creates a two-panel split with the main chat as primary");
assert.deepEqual(applyDraggedSessionToDashboard({ version: 1, tiledSessionIds: ["a", "b"], primarySessionId: "a", focusedSessionId: "b", layoutMode: "split-2", autoPrioritize: false }, "c", "a", valid).tiledSessionIds, ["a", "b", "c"], "dropping into two open chats grows the dashboard to three panels");
assert.equal(applyDraggedSessionToDashboard({ version: 1, tiledSessionIds: ["a", "b"], primarySessionId: "a", focusedSessionId: "b", layoutMode: "split-2", autoPrioritize: false }, "c", "a", valid).layoutMode, "focus-stack", "three dropped chats use the primary-plus-secondary stack layout");
assert.equal(applyDraggedSessionToDashboard({ version: 1, tiledSessionIds: ["a", "b", "c"], primarySessionId: "a", focusedSessionId: "c", layoutMode: "focus-stack", autoPrioritize: false }, "d", "a", valid).layoutMode, "quad", "dropping into three open chats grows to a four-panel quad");
assert.deepEqual(applyDraggedSessionToDashboard({ version: 1, tiledSessionIds: ["a", "b", "c", "d"], primarySessionId: "a", focusedSessionId: "d", layoutMode: "quad", autoPrioritize: false }, "e", "a", valid).tiledSessionIds, ["a", "b", "c", "d"], "drag-drop never exceeds the supported four visible panels");
assert.deepEqual(applyDraggedSessionToDashboard({ version: 1, tiledSessionIds: [], layoutMode: "auto", autoPrioritize: false }, "a", "a", valid, { insertAt: 0 }).tiledSessionIds, ["a"], "dropping the active single chat into empty dashboard space should still materialize a tile instead of no-oping");
assert.deepEqual(applyDraggedSessionToDashboard({ version: 1, tiledSessionIds: [], layoutMode: "auto", autoPrioritize: false }, "b", "a", valid, { insertAt: 0 }).tiledSessionIds, ["b", "a"], "dropping a new session on the left side of a single chat should place it before the active chat");
assert.deepEqual(applyDraggedSessionToDashboard({ version: 1, tiledSessionIds: [], layoutMode: "auto", autoPrioritize: false }, "b", "a", valid, { insertAt: 1 }).tiledSessionIds, ["a", "b"], "dropping a new session on the right side of a single chat should place it after the active chat");
assert.deepEqual(applyDraggedSessionToDashboard({ version: 1, tiledSessionIds: ["a", "b"], primarySessionId: "a", focusedSessionId: "b", layoutMode: "split-2", autoPrioritize: false }, "c", "a", valid, { insertAt: 0 }).tiledSessionIds, ["c", "a", "b"], "two-window drops on the left side should insert before the left panel");
assert.deepEqual(applyDraggedSessionToDashboard({ version: 1, tiledSessionIds: ["a", "b", "c"], primarySessionId: "a", focusedSessionId: "c", layoutMode: "focus-stack", autoPrioritize: false }, "d", "a", valid, { insertAt: 1 }).tiledSessionIds, ["a", "d", "b", "c"], "three-window drops should map cursor position into the four-panel order instead of only appending");
assert.deepEqual(applyDraggedSessionToDashboard({ version: 1, tiledSessionIds: ["a", "b", "c"], primarySessionId: "a", focusedSessionId: "c", layoutMode: "focus-stack", autoPrioritize: false }, "d", "a", valid, { insertAt: 3 }).tiledSessionIds, ["a", "b", "c", "d"], "three-window bottom/right empty-area drops should append as the fourth panel");
assert.deepEqual(applyDraggedSessionToDashboard({ version: 1, tiledSessionIds: ["a", "b", "c"], primarySessionId: "a", focusedSessionId: "c", layoutMode: "focus-stack", autoPrioritize: false }, "b", "a", valid, { insertAt: 3 }).tiledSessionIds, ["a", "c", "b"], "dropping an already tiled session on empty dashboard space should move/focus it instead of silently rejecting the drop");

const storage = new Map<string, string>();
const fakeStorage = {
  get length() { return storage.size; },
  clear: () => storage.clear(),
  getItem: (key: string) => storage.get(key) ?? null,
  key: (index: number) => Array.from(storage.keys())[index] ?? null,
  removeItem: (key: string) => { storage.delete(key); },
  setItem: (key: string, value: string) => { storage.set(key, value); },
} satisfies Storage;
storage.set(AGENT_DASHBOARD_STORAGE_KEY, "not-json");
assert.deepEqual(loadAgentDashboardState(valid, fakeStorage).tiledSessionIds, [], "corrupt localStorage falls back");
saveAgentDashboardState({ version: 1, tiledSessionIds: ["a"], primarySessionId: "a", focusedSessionId: "a", layoutMode: "split-2", autoPrioritize: false }, fakeStorage);
assert.equal(loadAgentDashboardState(valid, fakeStorage).layoutMode, "auto", "saved manual layout mode is migrated back to automatic single-tile layout");
