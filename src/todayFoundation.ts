export type TodayFoundationSource = "native" | "fallback" | "checking";

export type TodayFoundationStatus = {
  migration_version: number;
  workspace_count: number;
  event_count: number;
  visible_user: {
    root: string;
    starter_directories: string[];
  };
  secure_services: {
    redaction_ready: boolean;
    safe_logging_ready: boolean;
    action_policy_ready: boolean;
    event_writer_ready: boolean;
    keychain_status: string;
    sample_policy: {
      category: string;
      human_confirmation: string;
    };
  };
};

export type TodayFoundationInput = {
  source: TodayFoundationSource;
  sourceLabel: string;
  countLabel: string;
  status: TodayFoundationStatus | null;
};

export type TodayWidgetView = {
  title: string;
  status: string;
  copy: string;
  tone: "ready" | "blocked" | "pending";
};

export type TodayFoundationView = {
  sourceLabel: string;
  heroStatus: string;
  heroCopy: string;
  metrics: {
    registeredWorkspaces: string;
    foundationEvents: string;
    migrationVersion: string;
    starterDirectories: string;
    secureSafeguards: string;
    keychainStatus: string;
    samplePolicy: string;
  };
  widgets: {
    tasks: TodayWidgetView;
    runs: TodayWidgetView;
    inbox: TodayWidgetView;
    integrations: TodayWidgetView;
  };
};

function readyCount(status: TodayFoundationStatus) {
  return [
    status.secure_services.redaction_ready,
    status.secure_services.safe_logging_ready,
    status.secure_services.action_policy_ready,
    status.secure_services.event_writer_ready,
  ].filter(Boolean).length;
}

function confirmationLabel(value: string) {
  return value.replace(/_/g, " ");
}

export function buildTodayFoundationView(input: TodayFoundationInput): TodayFoundationView {
  if (input.status) {
    const safeguardsReady = readyCount(input.status);

    return {
      sourceLabel: input.sourceLabel,
      heroStatus: "Real local foundation",
      heroCopy: `Today is backed by get_foundation_status from the ${input.sourceLabel}. Counts, paths, migration state, and safeguards below are live local state; tasks, runs, inbox, and integrations remain empty until real providers exist.`,
      metrics: {
        registeredWorkspaces: input.countLabel,
        foundationEvents: String(input.status.event_count),
        migrationVersion: String(input.status.migration_version),
        starterDirectories: String(input.status.visible_user.starter_directories.length),
        secureSafeguards: `${safeguardsReady}/4 ready`,
        keychainStatus: input.status.secure_services.keychain_status,
        samplePolicy: `${input.status.secure_services.sample_policy.category}: ${confirmationLabel(input.status.secure_services.sample_policy.human_confirmation)} confirmation`,
      },
      widgets: {
        tasks: {
          title: "Tasks",
          status: "Empty",
          tone: "pending",
          copy: "No real local tasks are registered yet. Nothing is fabricated for Today.",
        },
        runs: {
          title: "Runs",
          status: "Empty",
          tone: "pending",
          copy: "No real agent or automation runs are available in the current local foundation state.",
        },
        inbox: {
          title: "Inbox",
          status: "Empty",
          tone: "pending",
          copy: "No local inbox feed is connected yet; approvals and blockers will appear only from real native sources.",
        },
        integrations: {
          title: "Integrations",
          status: "Unconfigured",
          tone: "blocked",
          copy: "Configured providers are not present. Integration cards show only explicit unconfigured states.",
        },
      },
    };
  }

  const isChecking = input.source === "checking";

  return {
    sourceLabel: input.sourceLabel,
    heroStatus: isChecking ? "Checking native foundation" : "Browser preview",
    heroCopy: isChecking
      ? `Waiting for get_foundation_status. Browser preview registry is visible temporarily; native-only data unavailable until the Tauri response arrives.`
      : `Running outside the native app: native-only data unavailable. This browser preview shows the static registry shell only and no simulated tasks/runs/completions/connected integrations.`,
    metrics: {
      registeredWorkspaces: input.countLabel,
      foundationEvents: "Native-only",
      migrationVersion: "Native-only",
      starterDirectories: "Native-only",
      secureSafeguards: "Native-only",
      keychainStatus: "Native-only",
      samplePolicy: "Native-only",
    },
    widgets: {
      tasks: {
        title: "Tasks",
        status: "Unavailable",
        tone: "pending",
        copy: "No simulated tasks. Real Today tasks require native local task state that is not available in preview.",
      },
      runs: {
        title: "Runs",
        status: "Unavailable",
        tone: "pending",
        copy: "No simulated runs or completions. Run history will appear only when real local execution state exists.",
      },
      inbox: {
        title: "Inbox",
        status: "Unavailable",
        tone: "pending",
        copy: "No simulated notifications, approvals, or blockers. Native inbox data is not available in preview.",
      },
      integrations: {
        title: "Integrations",
        status: "Unconfigured",
        tone: "blocked",
        copy: "No connected integrations are assumed. Provider setup remains unconfigured until explicit local configuration exists.",
      },
    },
  };
}
