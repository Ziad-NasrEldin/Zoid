import assert from "node:assert/strict";
import { buildRuthlessReviewerPrompt, RUTHLESS_REVIEWER_TOOLSETS } from "./ruthlessReviewerAgent";

assert.deepEqual([...RUTHLESS_REVIEWER_TOOLSETS], ["terminal", "file"], "Ruthless reviewer must only receive terminal and file toolsets");

const prompt = buildRuthlessReviewerPrompt({
  repository: { name: "Zoid", path: "/Users/ziadnasreldin/Zoid", branch: "main" },
  activeSessionTitle: "Notifications work",
});

for (const required of [
  "Spawn a single leaf subagent",
  "toolsets exactly: [\"terminal\",\"file\"]",
  "Do not grant browser, web, memory, cronjob, messaging, design, social, or further delegation tools.",
  "The reviewer must not edit files, commit, push, deploy, send messages, create cron jobs, or perform external side effects.",
  "Start from git status and git diff",
  "Should this exist, and what proves it works?",
  "fake wiring",
  "Verdict: APPROVED or BLOCKED",
]) {
  assert.ok(prompt.includes(required), `Ruthless reviewer prompt is missing required guard: ${required}`);
}

assert.ok(prompt.includes("Repository: Zoid at /Users/ziadnasreldin/Zoid on branch main."), "Prompt should include linked repository context");
assert.ok(!prompt.includes("browser,web"), "Prompt should not compactly grant forbidden toolsets");
