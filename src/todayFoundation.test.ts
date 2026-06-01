import { buildTodayFoundationView } from "./todayFoundation";

const nativeView = buildTodayFoundationView({
  source: "native",
  sourceLabel: "Native registry",
  countLabel: "2 workspaces",
  status: {
    migration_version: 7,
    workspace_count: 2,
    event_count: 11,
    visible_user: {
      root: "/Users/example/Zoid",
      starter_directories: ["Tasks", "Notes"],
    },
    secure_services: {
      redaction_ready: true,
      safe_logging_ready: true,
      action_policy_ready: true,
      event_writer_ready: true,
      keychain_status: "ready",
      sample_policy: {
        category: "filesystem",
        human_confirmation: "review_required",
      },
    },
  },
});

const partialNativeView = buildTodayFoundationView({
  source: "native",
  sourceLabel: "Native registry",
  countLabel: "2 workspaces",
  status: {
    migration_version: 3,
    workspace_count: 2,
    event_count: 0,
    visible_user: {
      root: "/Users/example/Zoid",
      starter_directories: [],
    },
    secure_services: {
      redaction_ready: true,
      safe_logging_ready: false,
      action_policy_ready: true,
      event_writer_ready: false,
      keychain_status: "blocked_unverified_native_keychain_not_tested",
      sample_policy: {
        category: "credentials",
        human_confirmation: "always_required",
      },
    },
  },
});

const checkingView = buildTodayFoundationView({
  source: "checking",
  sourceLabel: "Checking native registry",
  countLabel: "14 workspaces",
  status: null,
});

const previewView = buildTodayFoundationView({
  source: "fallback",
  sourceLabel: "Browser preview fallback",
  countLabel: "14 workspaces",
  status: null,
});

const assertions: string[] = [
  nativeView.sourceLabel,
  nativeView.metrics.registeredWorkspaces,
  nativeView.metrics.foundationEvents,
  nativeView.metrics.migrationVersion,
  nativeView.metrics.secureSafeguards,
  nativeView.metrics.samplePolicy,
  nativeView.widgets.tasks.status,
  partialNativeView.metrics.secureSafeguards,
  partialNativeView.metrics.keychainStatus,
  partialNativeView.metrics.samplePolicy,
  checkingView.heroStatus,
  previewView.heroCopy,
  previewView.widgets.runs.copy,
];

if (!assertions.includes("Native registry")) {
  throw new Error("native Today view must preserve native registry source");
}

if (!assertions.includes("2 workspaces") || !assertions.includes("11") || !assertions.includes("7")) {
  throw new Error("native Today view must expose real status counts");
}

if (!assertions.includes("4/4 ready") || !assertions.includes("2/4 ready")) {
  throw new Error("native Today view must expose secure safeguard readiness from real booleans");
}

if (!assertions.includes("filesystem: review required confirmation") || !assertions.includes("credentials: always required confirmation")) {
  throw new Error("native Today view must format sample policy confirmation without changing its meaning");
}

if (!assertions.includes("blocked_unverified_native_keychain_not_tested")) {
  throw new Error("native Today view must preserve keychain readiness truthfully");
}

if (!assertions.includes("Checking native foundation")) {
  throw new Error("checking Today view must be labeled as checking, not ready");
}

if (!previewView.heroCopy.includes("native-only data unavailable") || !previewView.widgets.runs.copy.includes("No simulated runs")) {
  throw new Error("preview Today view must be explicitly truthful, not simulated");
}

for (const widget of Object.values(previewView.widgets)) {
  if (/complete|ready/i.test(widget.status)) {
    throw new Error("preview Today widgets must not claim simulated completed or ready status");
  }
}
