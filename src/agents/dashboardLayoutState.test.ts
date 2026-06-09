import { strict as assert } from "node:assert";
import { AGENT_DASHBOARD_MAX_TILES, AGENT_DASHBOARD_STORAGE_KEY, loadAgentDashboardState, sanitizeAgentDashboardState, saveAgentDashboardState } from "./dashboardLayoutState";

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
assert.equal(loadAgentDashboardState(valid, fakeStorage).layoutMode, "split-2", "saved state round-trips");
