export type AutomationAction = "pause" | "resume" | "run" | "remove";

export type AutomationStatusKind = "ok" | "error" | "paused" | "unknown";

export type AutomationFilter = "all" | "running" | "paused" | "failed" | "script";

export type AutomationCronJob = {
  jobId: string;
  name: string;
  schedule: string;
  repeat: string;
  deliver: string;
  nextRunAt: string | null;
  lastRunAt: string | null;
  lastStatus: string | null;
  lastDeliveryError: string | null;
  enabled: boolean;
  state: string;
  pausedAt: string | null;
  pausedReason: string | null;
  script: string | null;
  noAgent: boolean;
  skills: string[];
  promptPreview: string;
  enabledToolsets: string[];
  protected: boolean;
  protectionReason: string | null;
};

export type HermesWatcher = {
  id: string;
  name: string;
  state: "running" | "paused" | "failed" | "unknown";
  source: string;
  lastSeenAt: string | null;
  lastStatus: string | null;
  detail: string | null;
};

export type AutomationList = {
  jobs: AutomationCronJob[];
  watchers: HermesWatcher[];
  watcherSourceStatus: "available" | "unavailable" | "empty";
  count: number;
  refreshedAt: string;
  hermesCommand: string | null;
  activeProfile: string;
};
