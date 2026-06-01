export type StatusTone = "ready" | "blocked" | "pending";

export type WorkspaceRecord = {
  id: string;
  label: string;
  description: string;
  position: number;
};

export type ActionPolicyDecision = {
  category: string;
  policy: string;
  reviewer_required: string;
  human_confirmation: string;
  reason: string;
};

export type KeychainReadinessStatus = {
  ready: boolean;
  status: string;
  reason: string;
  credential_storage_enabled: boolean;
  test_path_exercised: boolean;
};

export type SecureFoundationStatus = {
  redaction_ready: boolean;
  safe_logging_ready: boolean;
  action_policy_ready: boolean;
  event_writer_ready: boolean;
  keychain: KeychainReadinessStatus;
  keychain_status: string;
  sample_policy: ActionPolicyDecision;
};

export type VisibleUserPathStatus = {
  root: string;
  starter_directories: string[];
};

export type AppSupportPathStatus = {
  root: string;
  logs_dir: string;
  database_parent: string;
  database_path: string;
  config_dir: string;
  config_path: string;
};

export type FoundationStatus = {
  visible_root: string;
  app_support_dir: string;
  database_path: string;
  logs_dir: string;
  config_dir: string;
  config_path: string;
  visible_user: VisibleUserPathStatus;
  app_support: AppSupportPathStatus;
  migration_version: number;
  workspace_count: number;
  event_count: number;
  workspaces: WorkspaceRecord[];
  secure_services: SecureFoundationStatus;
};

export type IntegrationState = {
  name: string;
  state: string;
  note: string;
};

export const defaultIntegrationStates: IntegrationState[] = [
  { name: "CLI profiles", state: "not configured", note: "Local command wiring is disabled until a real profile is added." },
  { name: "Gmail", state: "not configured", note: "Read and send flows remain unavailable until explicitly configured." },
  { name: "Apple Calendar", state: "needs permission", note: "Calendar access is gated by native app validation and permission." },
  { name: "OmniSocials", state: "not configured", note: "Publishing remains blocked without credentials and review policy." },
];

export type SettingsStatusMode = "native" | "checking" | "preview";

export type SettingsStatusItem = {
  label: string;
  value: string;
  tone?: StatusTone;
};

export type SettingsStatusPolicyView = {
  category: string;
  policy: string;
  reviewerRequired: string;
  humanConfirmation: string;
  reason: string;
};

export type SettingsStatusShellView = {
  mode: SettingsStatusMode;
  modeLabel: string;
  summary: string;
  paths: SettingsStatusItem[];
  database: SettingsStatusItem[];
  keychain: SettingsStatusItem[];
  safeguards: SettingsStatusItem[];
  policy: SettingsStatusPolicyView;
  events: SettingsStatusItem[];
  integrations: IntegrationState[];
};

export type SettingsStatusShellInput = {
  mode: SettingsStatusMode;
  status: FoundationStatus | null;
  integrations?: IntegrationState[];
};

const nativeOnly = "Native-only";
const previewUnavailable = "Preview unavailable";

function readinessLabel(ready: boolean) {
  return ready ? "Ready" : "Blocked";
}

function readinessTone(ready: boolean): StatusTone {
  return ready ? "ready" : "blocked";
}

function formatConfirmation(value: string) {
  return value.replace(/_/g, " ");
}

function nonNativeValue(mode: SettingsStatusMode) {
  return mode === "checking" ? "Native-only (checking)" : previewUnavailable;
}

function nonNativeShell(mode: Exclude<SettingsStatusMode, "native">, integrations: IntegrationState[]): SettingsStatusShellView {
  const value = nonNativeValue(mode);
  const label = mode === "checking" ? "Checking native foundation" : "Browser preview";

  return {
    mode,
    modeLabel: label,
    summary: mode === "checking"
      ? "Waiting for get_foundation_status. Native-only settings and readiness are not fabricated while loading."
      : "Running outside the native app. Paths, database counts, keychain, policy, event writer, and logging readiness are native-only.",
    paths: [
      { label: "Visible root", value },
      { label: "App support", value },
      { label: "Logs", value },
      { label: "SQLite DB", value },
      { label: "Config", value },
    ],
    database: [
      { label: "Migration version", value: nativeOnly },
      { label: "Foundation events", value: nativeOnly },
    ],
    keychain: [
      { label: "Status", value: nativeOnly, tone: "pending" },
      { label: "Reason", value: mode === "checking" ? "Checking native keychain readiness" : "Preview unavailable outside Tauri" },
      { label: "Credential storage", value: nativeOnly },
      { label: "Test path exercised", value: nativeOnly },
    ],
    safeguards: [
      { label: "Redaction", value: nativeOnly, tone: "pending" },
      { label: "Safe logging", value: nativeOnly, tone: "pending" },
      { label: "Action policy", value: nativeOnly, tone: "pending" },
      { label: "Event writer", value: nativeOnly, tone: "pending" },
    ],
    policy: {
      category: nativeOnly,
      policy: nativeOnly,
      reviewerRequired: nativeOnly,
      humanConfirmation: nativeOnly,
      reason: mode === "checking" ? "Checking native sample policy" : "Preview unavailable outside Tauri",
    },
    events: [
      { label: "Event writer", value: nativeOnly, tone: "pending" },
      { label: "Foundation events", value: nativeOnly },
    ],
    integrations,
  };
}

export function buildSettingsStatusShellView(input: SettingsStatusShellInput): SettingsStatusShellView {
  const integrations = input.integrations ?? defaultIntegrationStates;

  if (!input.status) {
    return nonNativeShell(input.mode === "native" ? "checking" : input.mode, integrations);
  }

  const { status } = input;
  const { secure_services: secure } = status;

  return {
    mode: "native",
    modeLabel: "Native foundation",
    summary: "Rendering real local paths, database migration state, secure readiness, keychain status, policy sample, event writer, and truthful integration setup from native status.",
    paths: [
      { label: "Visible root", value: status.visible_user.root },
      { label: "Starter directories", value: status.visible_user.starter_directories.join(", ") || "—" },
      { label: "App support", value: status.app_support.root },
      { label: "Logs", value: status.app_support.logs_dir },
      { label: "SQLite DB", value: status.app_support.database_path },
      { label: "Config directory", value: status.app_support.config_dir },
      { label: "Config file", value: status.app_support.config_path },
    ],
    database: [
      { label: "Migration version", value: String(status.migration_version) },
      { label: "Foundation events", value: String(status.event_count) },
      { label: "Registered workspaces", value: String(status.workspace_count) },
    ],
    keychain: [
      { label: "Status", value: secure.keychain.status || secure.keychain_status, tone: readinessTone(secure.keychain.ready) },
      { label: "Reason", value: secure.keychain.reason || "—" },
      { label: "Credential storage", value: secure.keychain.credential_storage_enabled ? "Enabled" : "Disabled", tone: readinessTone(secure.keychain.credential_storage_enabled) },
      { label: "Test path exercised", value: secure.keychain.test_path_exercised ? "Yes" : "No", tone: readinessTone(secure.keychain.test_path_exercised) },
    ],
    safeguards: [
      { label: "Redaction", value: readinessLabel(secure.redaction_ready), tone: readinessTone(secure.redaction_ready) },
      { label: "Safe logging", value: readinessLabel(secure.safe_logging_ready), tone: readinessTone(secure.safe_logging_ready) },
      { label: "Action policy", value: readinessLabel(secure.action_policy_ready), tone: readinessTone(secure.action_policy_ready) },
      { label: "Event writer", value: readinessLabel(secure.event_writer_ready), tone: readinessTone(secure.event_writer_ready) },
    ],
    policy: {
      category: secure.sample_policy.category,
      policy: secure.sample_policy.policy,
      reviewerRequired: formatConfirmation(secure.sample_policy.reviewer_required),
      humanConfirmation: formatConfirmation(secure.sample_policy.human_confirmation),
      reason: secure.sample_policy.reason,
    },
    events: [
      { label: "Event writer", value: readinessLabel(secure.event_writer_ready), tone: readinessTone(secure.event_writer_ready) },
      { label: "Foundation events", value: String(status.event_count) },
    ],
    integrations,
  };
}
