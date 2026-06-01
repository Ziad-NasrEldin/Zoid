import { buildSettingsStatusShellView, defaultIntegrationStates, type FoundationStatus } from "./settingsStatus";

const nativeStatus: FoundationStatus = {
  visible_root: "/Users/example/Zoid",
  app_support_dir: "/Users/example/Library/Application Support/Zoid",
  database_path: "/Users/example/Library/Application Support/Zoid/zoid.sqlite",
  logs_dir: "/Users/example/Library/Application Support/Zoid/logs",
  config_dir: "/Users/example/Library/Application Support/Zoid/config",
  config_path: "/Users/example/Library/Application Support/Zoid/config/settings.json",
  visible_user: {
    root: "/Users/example/Zoid",
    starter_directories: ["Tasks", "Notes"],
  },
  app_support: {
    root: "/Users/example/Library/Application Support/Zoid",
    logs_dir: "/Users/example/Library/Application Support/Zoid/logs",
    database_parent: "/Users/example/Library/Application Support/Zoid",
    database_path: "/Users/example/Library/Application Support/Zoid/zoid.sqlite",
    config_dir: "/Users/example/Library/Application Support/Zoid/config",
    config_path: "/Users/example/Library/Application Support/Zoid/config/settings.json",
  },
  migration_version: 7,
  workspace_count: 2,
  event_count: 11,
  workspaces: [],
  secure_services: {
    redaction_ready: true,
    safe_logging_ready: true,
    action_policy_ready: false,
    event_writer_ready: true,
    keychain_status: "blocked_unverified_native_keychain_not_tested",
    keychain: {
      ready: false,
      status: "blocked_unverified_native_keychain_not_tested",
      reason: "native keychain test path has not been exercised",
      credential_storage_enabled: false,
      test_path_exercised: false,
    },
    sample_policy: {
      category: "credentials",
      policy: "deny_by_default",
      reviewer_required: "security_reviewer",
      human_confirmation: "always_required",
      reason: "credential access is consequential",
    },
  },
};

const nativeView = buildSettingsStatusShellView({
  mode: "native",
  status: nativeStatus,
  integrations: defaultIntegrationStates,
});

if (nativeView.modeLabel !== "Native foundation") {
  throw new Error("native shell must identify the real foundation source");
}

if (!nativeView.paths.some((item) => item.label === "Visible root" && item.value === nativeStatus.visible_user.root)) {
  throw new Error("native shell must expose the real visible root path");
}

if (!nativeView.paths.some((item) => item.label === "SQLite DB" && item.value === nativeStatus.app_support.database_path)) {
  throw new Error("native shell must expose the real database path");
}

if (!nativeView.database.some((item) => item.label === "Foundation events" && item.value === "11")) {
  throw new Error("native shell must expose the real event count");
}

if (!nativeView.keychain.some((item) => item.label === "Reason" && item.value === nativeStatus.secure_services.keychain.reason)) {
  throw new Error("native shell must expose keychain reason truthfully");
}

if (!nativeView.safeguards.some((item) => item.label === "Action policy" && item.value === "Blocked")) {
  throw new Error("native shell must derive safeguard readiness from booleans");
}

if (nativeView.policy.category !== "credentials" || nativeView.policy.humanConfirmation !== "always required" || nativeView.policy.reason !== "credential access is consequential") {
  throw new Error("native shell must expose sample policy category, confirmation, and reason without adding confirmation UI state");
}

if (nativeView.integrations.some((integration) => /connected|ready/i.test(integration.state))) {
  throw new Error("settings shell must not invent configured integrations");
}

for (const requiredSection of [
  nativeView.paths,
  nativeView.database,
  nativeView.keychain,
  nativeView.safeguards,
  nativeView.events,
]) {
  if (requiredSection.length === 0) {
    throw new Error("native settings shell must render every P1.22 status section");
  }
}

if (!nativeView.events.some((item) => item.label === "Event writer" && item.value === "Ready")) {
  throw new Error("native shell must expose event writer readiness in the events section");
}

const checkingView = buildSettingsStatusShellView({
  mode: "checking",
  status: null,
  integrations: defaultIntegrationStates,
});

const previewView = buildSettingsStatusShellView({
  mode: "preview",
  status: null,
  integrations: defaultIntegrationStates,
});

for (const shell of [checkingView, previewView]) {
  const rendered = [
    shell.modeLabel,
    ...shell.paths.map((item) => item.value),
    ...shell.database.map((item) => item.value),
    ...shell.keychain.map((item) => item.value),
    ...shell.safeguards.map((item) => item.value),
    shell.policy.reason,
  ].join("\n");

  if (!/Native-only|Checking|Preview unavailable/.test(rendered)) {
    throw new Error("non-native shell must label unavailable native data explicitly");
  }

  if (/\/Users\/example|11|7/.test(rendered)) {
    throw new Error("non-native shell must not reuse native paths or counts");
  }
}

if (!previewView.paths.every((item) => item.value === "Preview unavailable")) {
  throw new Error("browser preview paths must say preview unavailable instead of fake local paths");
}

if (!checkingView.summary.includes("not fabricated") || !previewView.summary.includes("native-only")) {
  throw new Error("non-native summaries must make native-only/checking limitations explicit");
}

if (checkingView.mode !== "checking" || previewView.mode !== "preview") {
  throw new Error("non-native shell modes must remain distinct for UI copy");
}
