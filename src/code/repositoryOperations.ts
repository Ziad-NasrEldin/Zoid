import type { ChatMessage } from "../agents/types";
import type { CodeRepository } from "./types";

export type RepositoryOperationAction = "localhost" | "staging" | "production";
export type RepositoryOperationStatus = "unknown" | "running" | "learned" | "needs-review" | "blocked" | "broken";

export type RepositoryOperationProfile = {
  repoId: string;
  repositoryPath: string;
  repositoryRemoteUrl?: string;
  action: RepositoryOperationAction;
  status: RepositoryOperationStatus;
  confidenceScore: number;
  lastSessionId?: string;
  lastRunId?: string;
  lastStartedAt?: string;
  lastSuccessfulRunAt?: string;
  lastResultUrl?: string;
  runbookMarkdown: string;
  updatedAt: string;
};

export type RepositoryOperationRun = {
  id: string;
  repoId: string;
  action: RepositoryOperationAction;
  sessionId: string;
  startedAt: string;
  finishedAt?: string;
  outcome: "running" | "success" | "failed" | "cancelled" | "needs-user" | "blocked";
  initialPrompt: string;
  runbookSnapshot: string;
  responseContent?: string;
};

export const REPOSITORY_OPERATION_PROFILES_STORAGE_KEY = "zoid25:repository-operation-profiles";
export const REPOSITORY_OPERATION_RUNS_STORAGE_KEY = "zoid25:repository-operation-runs";

export const REPOSITORY_OPERATION_LABELS: Record<RepositoryOperationAction, string> = {
  localhost: "Run localhost",
  staging: "Deploy staging",
  production: "Deploy production",
};

export function repositoryOperationKey(repoId: string, action: RepositoryOperationAction) {
  return `${repoId}:${action}`;
}

export function repositoryOperationTitle(action: RepositoryOperationAction, repository: CodeRepository) {
  if (action === "localhost") return `Localhost · ${repository.name}`;
  if (action === "staging") return `Staging deploy · ${repository.name}`;
  return `Production deploy · ${repository.name}`;
}

function repoLocalRunbookPath(action: RepositoryOperationAction) {
  return `.hermes/runbooks/${action}.md`;
}

function defaultRunbook(action: RepositoryOperationAction, repository: CodeRepository) {
  return [
    `# ${REPOSITORY_OPERATION_LABELS[action]} runbook for ${repository.name}`,
    "",
    "No successful run has been captured yet.",
    "Use this run to discover the commands, blockers, checks, and safe next-run defaults.",
  ].join("\n");
}

export function getRepositoryOperationProfile(
  profiles: Record<string, RepositoryOperationProfile>,
  repository: CodeRepository,
  action: RepositoryOperationAction,
) {
  return profiles[repositoryOperationKey(repository.id, action)] ?? {
    repoId: repository.id,
    repositoryPath: repository.path,
    repositoryRemoteUrl: repository.remoteUrl,
    action,
    status: "unknown" as const,
    confidenceScore: 0,
    runbookMarkdown: defaultRunbook(action, repository),
    updatedAt: new Date().toISOString(),
  };
}

export function buildRepositoryOperationPrompt({
  repository,
  action,
  profile,
}: {
  repository: CodeRepository;
  action: RepositoryOperationAction;
  profile: RepositoryOperationProfile;
}) {
  const actionIntro = action === "localhost"
    ? "Run the local development server for this repository."
    : action === "staging"
      ? "Deploy this repository to staging."
      : "Prepare and, only after explicit approval, deploy this repository to production.";

  const actionRules = action === "localhost"
    ? [
      "Use the prior runbook first. If it fails, diagnose and update it.",
      "Install dependencies only when needed.",
      "Do not deploy anything.",
      "Start or verify the local server and report the exact URL/port plus health-check result.",
    ]
    : action === "staging"
      ? [
        "Use the prior staging runbook first. If it fails, diagnose and update it.",
        "Confirm required env/secrets are present, but never print secret values.",
        "Run tests/build before deploy when appropriate.",
        "Deploy only to the staging target and verify the staging URL/core health checks.",
      ]
      : [
        "Use the prior production runbook first. If it fails, diagnose and update it.",
        "Inspect branch, diff, env requirements, migrations, tests, and deployment path before acting.",
        "Do not run irreversible production actions without explicit user confirmation in this session.",
        "Production deploys, production migrations, destructive commands, env overwrites, force pushes, and branch resets require approval.",
        "After any approved production action, verify the production URL and real E2E requirements before claiming success.",
      ];

  return [
    `Repository operation: ${REPOSITORY_OPERATION_LABELS[action]}`,
    "",
    actionIntro,
    "",
    "Repository:",
    `- Name: ${repository.name}`,
    `- Path: ${repository.path}`,
    `- Remote: ${repository.remoteUrl ?? "No remote detected"}`,
    `- Current branch: ${repository.branch ?? "Unknown"}`,
    `- Default branch: ${repository.defaultBranch ?? repository.branch ?? "Unknown"}`,
    `- Dirty state: ${repository.dirty ? "Dirty" : "Clean"}`,
    "",
    "Hybrid learning contract:",
    "- Treat Zoid app data as the canonical operation memory for this repo/action.",
    `- Repo-local export path, only if the user asks to sync/export: ${repoLocalRunbookPath(action)}`,
    "- Never include secrets in runbooks or output.",
    "- At the end, include an OPERATION_OUTCOME line with exactly one of: success, failed, blocked, needs-user.",
    "- Only use OPERATION_OUTCOME: success after the server/deploy is actually verified for this requested action, not because an old URL still exists.",
    "- At the end, include a RUNBOOK_UPDATE section with commands, failures, fixes, checks, URLs, and next-run notes.",
    "",
    "Rules:",
    ...actionRules.map((rule) => `- ${rule}`),
    "",
    "Prior runbook snapshot:",
    "```md",
    profile.runbookMarkdown || defaultRunbook(action, repository),
    "```",
  ].join("\n");
}

export function createRepositoryOperationUserMessage(content: string, createdAt: string): ChatMessage {
  return {
    id: `repo-operation-user-${crypto.randomUUID()}`,
    role: "user",
    participantId: "ziad",
    content,
    createdAt,
    status: "sent",
  };
}

export function operationStatusLabel(profile?: RepositoryOperationProfile) {
  if (!profile || profile.status === "unknown") return "New";
  if (profile.status === "running") return "Running";
  if (profile.status === "learned") return profile.confidenceScore >= 70 ? "Learned" : "Learning";
  if (profile.status === "needs-review") return "Review";
  if (profile.status === "blocked") return "Blocked";
  return "Fix needed";
}

export function extractRunbookUpdate(responseContent: string) {
  const markerMatch = /(?:^|\n)\s*#{0,6}\s*RUNBOOK_UPDATE\s*:?\s*(?:\n|$)/i.exec(responseContent);
  if (!markerMatch) return "";
  const start = markerMatch.index + markerMatch[0].length;
  const afterMarker = responseContent.slice(start).trim();
  const nextMarker = /\n\s*#{1,6}\s+[A-Z][A-Z0-9_ -]{2,}\s*(?:\n|$)/.exec(afterMarker);
  return (nextMarker ? afterMarker.slice(0, nextMarker.index) : afterMarker).trim();
}

export function inferRepositoryOperationOutcome(responseContent: string, fallback: RepositoryOperationRun["outcome"]) {
  const normalized = responseContent.toLowerCase();
  const explicitOutcome = /(?:^|\n)\s*OPERATION_OUTCOME\s*:\s*(success|failed|blocked|needs-user|cancelled)\b/i.exec(responseContent)?.[1] as RepositoryOperationRun["outcome"] | undefined;
  if (explicitOutcome) return explicitOutcome;
  if (/\b(blocked|missing secret|missing env|permission denied|not authenticated|needs setup)\b/.test(normalized)) return "blocked";
  if (/\b(needs user|needs your|requires confirmation|requires approval|awaiting approval|manual confirmation|required approval)\b/.test(normalized)) return "needs-user";
  if (/\b(failed|failure|error|could not|unable to|timed out|timeout)\b/.test(normalized)) return "failed";
  return fallback;
}

export function mergeRunbookUpdate(profile: RepositoryOperationProfile, responseContent: string, finishedAt: string) {
  const update = extractRunbookUpdate(responseContent);
  if (!update) return profile.runbookMarkdown;
  const previous = profile.runbookMarkdown.trim();
  return [
    previous || `# ${profile.action} runbook`,
    "",
    `## Learned update — ${finishedAt}`,
    "",
    update,
  ].join("\n").trim();
}
