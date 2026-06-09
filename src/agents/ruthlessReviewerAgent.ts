import type { CodeRepository } from "../code/types";

export const RUTHLESS_REVIEWER_AGENT_NAME = "Ruthless code-review subagent";
export const RUTHLESS_REVIEWER_TOOLSETS = ["terminal", "file"] as const;

export type RuthlessReviewerPromptOptions = {
  repository?: Pick<CodeRepository, "name" | "path" | "branch">;
  activeSessionTitle?: string;
};

function repositoryLine(repository?: RuthlessReviewerPromptOptions["repository"]) {
  if (!repository) return "No repository is currently linked. First identify the project root from the current Zoid/Hermes session context; if it cannot be proven, report BLOCKED instead of guessing.";
  return `Repository: ${repository.name} at ${repository.path}${repository.branch ? ` on branch ${repository.branch}` : ""}.`;
}

export function buildRuthlessReviewerPrompt({ repository, activeSessionTitle }: RuthlessReviewerPromptOptions = {}) {
  return `Spawn a single leaf subagent named "${RUTHLESS_REVIEWER_AGENT_NAME}" to perform an adversarial implementation review.

Access boundary:
- Use delegate_task with toolsets exactly: ${JSON.stringify([...RUTHLESS_REVIEWER_TOOLSETS])}.
- Do not grant browser, web, memory, cronjob, messaging, design, social, or further delegation tools.
- The reviewer may read files, inspect git diff/status, and run non-mutating validation commands only.
- The reviewer must not edit files, commit, push, deploy, send messages, create cron jobs, or perform external side effects.

Scope:
- ${repositoryLine(repository)}
- Active Zoid session: ${activeSessionTitle || "current session"}.
- Review every line of code written or changed for this task, including source, backend, UI, tests, styles, config, and generated wiring that affects behavior.
- Start from git status and git diff. Include untracked files that are part of the implementation. If a file is too large or generated/vendor-only, state that boundary explicitly and review only the relevant integration points.

Ruthless review rules:
- Do not assume functionality. Verify whether each important path actually runs.
- Flag fake wiring, dead buttons, no-op state, unreachable code, placeholder copy pretending to work, missing permissions, broken async/error paths, race conditions, security/privacy leaks, and tests that only grep strings without proving behavior.
- For each questionable line/block, ask: "Should this exist, and what proves it works?"
- Distill aggressively. Nothing useless should be included.
- Separate Required fixes from Optional improvements. Required means the code is not functional, unsafe, fake, or likely to break the user flow.
- If evidence is missing, say "Not proven" instead of guessing.

Return format:
1. Verdict: APPROVED or BLOCKED.
2. Required fixes: bullets with file path, line/block, problem, proof, and exact expected correction.
3. Functional proof checked: commands/files inspected and what they proved.
4. Useless/access violations: any unnecessary files, tools, side effects, or fake code that should be removed.
5. Open questions: only questions that block correctness.`;
}
