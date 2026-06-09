import { strict as assert } from "node:assert";
import { buildContinuationBrief } from "./continuationBrief";
import type { HermesChatSession } from "./sessionState";

const base = (id: string, title: string): HermesChatSession => ({ id, title, createdAt: "2026-06-09T00:00:00.000Z", updatedAt: "2026-06-09T00:00:00.000Z", messages: [] });
const sessionA: HermesChatSession = {
  ...base("a", "Zoid dashboard"),
  messages: [
    { id: "a1", role: "user", participantId: "ziad", content: "Implement panels in /Users/ziadnasreldin/Zoid/src/agents", createdAt: "2026-06-09T00:00:00.000Z", status: "sent" },
    { id: "a2", role: "assistant", participantId: "hermes", content: "Added dashboard state. TODO test persistence.", createdAt: "2026-06-09T00:01:00.000Z", status: "sent" },
  ],
};
const sessionB: HermesChatSession = {
  ...base("b", "Other project"),
  messages: [
    { id: "b1", role: "user", participantId: "ziad", content: "Secret unrelated /Users/ziadnasreldin/OtherRepo", createdAt: "2026-06-09T00:00:00.000Z", status: "sent" },
  ],
};

const brief = buildContinuationBrief(sessionA);
assert.match(brief, /Continue this same session/, "brief instructs same-session continuation");
assert.match(brief, /Zoid dashboard/, "brief includes title");
assert.match(brief, /TODO test persistence/, "brief includes last assistant state");
assert.match(brief, /\/Users\/ziadnasreldin\/Zoid\/src\/agents/, "brief includes paths from the selected session");
assert.doesNotMatch(brief, /OtherRepo|Secret unrelated/, "brief must not include messages from another session");
assert.equal(buildContinuationBrief(sessionB).includes("Zoid dashboard"), false, "building another session remains isolated");
