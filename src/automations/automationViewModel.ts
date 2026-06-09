import type { AutomationCronJob, AutomationFilter, AutomationList, AutomationStatusKind } from "./types";

export function getAutomationStatusKind(job: AutomationCronJob): AutomationStatusKind {
  const state = job.state.toLowerCase();
  const lastStatus = (job.lastStatus ?? "").toLowerCase();
  if (!job.enabled || state.includes("pause") || state.includes("disable")) return "paused";
  if (lastStatus.includes("error") || lastStatus.includes("fail")) return "error";
  if (lastStatus === "ok" || lastStatus.includes(" ok")) return "ok";
  return "unknown";
}

export function filterAutomationJobs(jobs: AutomationCronJob[], filter: AutomationFilter, query: string): AutomationCronJob[] {
  const normalizedQuery = query.trim().toLowerCase();
  return jobs.filter((job) => {
    const kind = getAutomationStatusKind(job);
    const matchesFilter =
      filter === "all" ||
      (filter === "running" && job.enabled && kind !== "paused") ||
      (filter === "paused" && kind === "paused") ||
      (filter === "failed" && kind === "error") ||
      (filter === "script" && (job.noAgent || Boolean(job.script)));

    if (!matchesFilter) return false;
    if (!normalizedQuery) return true;
    return [
      job.jobId,
      job.name,
      job.schedule,
      job.repeat,
      job.deliver,
      job.script ?? "",
      job.lastStatus ?? "",
      job.lastDeliveryError ?? "",
      job.skills.join(" "),
      job.enabledToolsets.join(" "),
    ].join(" ").toLowerCase().includes(normalizedQuery);
  });
}

export function summarizeAutomationJobs(list: AutomationList) {
  const failedJobs = list.jobs.filter((job) => getAutomationStatusKind(job) === "error");
  const pausedJobs = list.jobs.filter((job) => getAutomationStatusKind(job) === "paused");
  const enabledJobs = list.jobs.filter((job) => job.enabled);
  const nextRunSoonest = list.jobs
    .map((job) => job.nextRunAt)
    .filter((value): value is string => Boolean(value))
    .sort()[0] ?? null;

  return {
    totalJobs: list.jobs.length,
    enabledJobs: enabledJobs.length,
    pausedJobs: pausedJobs.length,
    failedJobs: failedJobs.length,
    nextRunSoonest,
    watcherCount: list.watchers.length,
  };
}

export function deriveAutomationNavStatus(list: AutomationList | null, errorMessage: string | null): "ready" | "idle" | "blocked" {
  if (errorMessage) return "blocked";
  if (!list) return "idle";
  if (list.jobs.some((job) => getAutomationStatusKind(job) === "error") || list.watchers.some((watcher) => watcher.state === "failed")) return "blocked";
  if (list.jobs.length > 0 || list.watchers.length > 0) return "ready";
  return "idle";
}
