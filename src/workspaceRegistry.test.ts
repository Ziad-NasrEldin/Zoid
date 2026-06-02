import { type FoundationStatus } from "./settingsStatus";
import {
  buildWorkspaceChromeView,
  buildWorkspaceRegistryView,
  fallbackWorkspaces,
  formatWorkspaceCount,
  workspaceGlyphs,
} from "./workspaceRegistry";

const baseStatus = {
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
  secure_services: {
    redaction_ready: true,
    safe_logging_ready: true,
    action_policy_ready: true,
    event_writer_ready: true,
    keychain_status: "ready",
    keychain: {
      ready: true,
      status: "ready",
      reason: "test path exercised",
      credential_storage_enabled: true,
      test_path_exercised: true,
    },
    sample_policy: {
      category: "filesystem",
      policy: "allow_readonly",
      reviewer_required: "none",
      human_confirmation: "review_required",
      reason: "read-only inspection",
      allowed_now: true,
      requires_confirmation: false,
      requires_reviewer: false,
      requires_clear_task: false,
    },
  },
} satisfies Omit<FoundationStatus, "workspaces" | "workspace_count"> & { workspace_count: number };

const nativeStatus: FoundationStatus = {
  ...baseStatus,
  workspace_count: 3,
  workspaces: [
    { id: "notes", label: "Notes", description: "Native notes", position: 20 },
    { id: "today", label: "Today", description: "Native today", position: 0 },
    { id: "custom", label: "Custom", description: "Native custom module", position: 10 },
  ],
};

if (formatWorkspaceCount(1) !== "1 workspace" || formatWorkspaceCount(2) !== "2 workspaces") {
  throw new Error("workspace count labels must use singular/plural copy correctly");
}

const nativeRegistry = buildWorkspaceRegistryView(nativeStatus, null);

if (nativeRegistry.source !== "native" || nativeRegistry.sourceLabel !== "Native registry") {
  throw new Error("native registry view must identify native source");
}

if (nativeRegistry.countLabel !== "3 workspaces") {
  throw new Error("native registry view must count returned workspaces, not static fallback rows");
}

if (nativeRegistry.workspaces.map((workspace) => workspace.id).join(",") !== "today,custom,notes") {
  throw new Error("native registry view must render workspaces sorted by backend position");
}

if (!nativeRegistry.truthCopy.includes("returned by get_foundation_status") || nativeRegistry.truthCopy.includes("fallback is mixed")) {
  throw new Error("native registry copy must say it is rendering backend data and not mixed fallback data");
}

const nativeChrome = buildWorkspaceChromeView(nativeRegistry, "custom");
if (nativeChrome.activeWorkspace?.id !== "custom" || nativeChrome.activeWorkspaceLabel !== "Custom") {
  throw new Error("workspace chrome must select active native workspace by id");
}
if (nativeChrome.sidebarEmptyCopy !== null || nativeChrome.registryEmptyCopy !== null) {
  throw new Error("workspace chrome must not expose empty-state copy when workspaces exist");
}
if (nativeChrome.glyphs.custom !== "C" || workspaceGlyphs.today !== "•") {
  throw new Error("workspace chrome must provide fallback and known glyphs for registry rendering");
}

const emptyNativeRegistry = buildWorkspaceRegistryView({ ...baseStatus, workspace_count: 0, workspaces: [] }, null);
const emptyNativeChrome = buildWorkspaceChromeView(emptyNativeRegistry, "missing");
if (emptyNativeRegistry.source !== "native" || emptyNativeRegistry.countLabel !== "0 workspaces") {
  throw new Error("empty native registry must remain native and expose a zero-workspace count");
}
if (emptyNativeChrome.activeWorkspace !== null || emptyNativeChrome.activeWorkspaceLabel !== "No workspaces registered") {
  throw new Error("empty native registry must not synthesize an active workspace");
}
if (emptyNativeChrome.sidebarEmptyCopy !== "No native workspaces registered." || emptyNativeChrome.registryEmptyCopy !== "The native registry returned no workspaces.") {
  throw new Error("empty native registry must expose truthful empty states for sidebar and registry list");
}

const checkingRegistry = buildWorkspaceRegistryView(null, null);
const fallbackRegistry = buildWorkspaceRegistryView(null, "Native foundation status is available inside the packaged Tauri app.");

for (const registry of [checkingRegistry, fallbackRegistry]) {
  if (registry.workspaces.length !== fallbackWorkspaces.length || registry.countLabel !== "14 workspaces") {
    throw new Error("non-native registry must render the static preview workspace shell");
  }
  if (!/preview|loading|outside Tauri/i.test(registry.truthCopy)) {
    throw new Error("non-native registry copy must truthfully explain preview/checking data");
  }
  if (/success|ready|complete|connected/i.test([registry.sourceLabel, registry.truthCopy].join("\n"))) {
    throw new Error("non-native registry copy must not claim fake success, readiness, completion, or connected state");
  }
}

const checkingChrome = buildWorkspaceChromeView(checkingRegistry, "does-not-exist");
if (checkingChrome.activeWorkspace?.id !== "today") {
  throw new Error("preview/checking registry must fall back to the first visible preview workspace for navigation");
}
