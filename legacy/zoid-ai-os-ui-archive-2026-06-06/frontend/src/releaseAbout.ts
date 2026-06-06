export type LogRetentionRecord = {
  scope: string;
  retention_days: number;
  max_total_bytes: number;
  enabled: boolean;
  updated_at?: string;
};

export type LogRetentionSettingView = {
  scope: string;
  retentionDays: number;
  maxTotalBytes: number;
  enabled: boolean;
  summary: string;
};

export type LogCleanupRunRecord = {
  id: string;
  scope: string;
  dry_run: boolean;
  files_considered: number;
  files_deleted: number;
  bytes_deleted: number;
  status: string;
  error_message?: string | null;
  created_at: string;
};

export type ReleaseAboutView = {
  appName: string;
  version: string;
  build: string;
  packaging: string;
  signing: string;
  notarization: string;
  safeDiagnostics: string[];
};

export type ReleaseHardeningState =
  | { mode: "loading" }
  | { mode: "error"; error: string }
  | { mode: "ready"; retention: LogRetentionSettingView[]; about: ReleaseAboutView; migrationGuidance: ReturnType<typeof buildMigrationFailureGuidance>; cleanupResult?: LogCleanupRunRecord };

export type ReleaseHardeningInvoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

export function buildLogRetentionSettingsView(settings: LogRetentionRecord[]): LogRetentionSettingView[] {
  return settings.map((setting) => ({
    scope: setting.scope,
    retentionDays: setting.retention_days,
    maxTotalBytes: setting.max_total_bytes,
    enabled: setting.enabled,
    summary: `${setting.enabled ? "Enabled" : "Disabled"}: keep ${setting.retention_days} day(s), cap ${Math.round(setting.max_total_bytes / 1024)} KiB`,
  }));
}

export function buildReleaseAboutView(input: { version?: string; build?: string; packaged?: boolean; signingConfigured?: boolean; notarizationConfigured?: boolean }): ReleaseAboutView {
  return {
    appName: "Zoid",
    version: input.version || "development",
    build: input.build || "local",
    packaging: input.packaged ? "Tauri package metadata configured" : "Development build",
    signing: input.signingConfigured ? "Signing configured by environment/certificate" : "Unsigned local build; no certificate secret is bundled",
    notarization: input.notarizationConfigured ? "Notarization configured by environment" : "Notarization not configured for local builds",
    safeDiagnostics: [
      "No raw secrets are displayed in About or diagnostics.",
      "App data is stored in ~/Library/Application Support/Zoid and ~/Zoid.",
      "Log retention can be reviewed before cleanup with dry-run.",
    ],
  };
}

export function buildMigrationFailureGuidance(errorMessage: string) {
  const redacted = errorMessage.replace(/(token|password|secret)\s*[=:]\s*\S+/gi, "$1=[REDACTED]");
  return {
    title: "Database migration could not finish safely",
    message: redacted,
    actions: ["Quit Zoid", "Back up ~/Library/Application Support/Zoid", "Review migration backups", "Relaunch after resolving the blocker"],
  };
}

export async function loadReleaseHardeningState(invokeBridge: ReleaseHardeningInvoke): Promise<ReleaseHardeningState> {
  try {
    const retention = await invokeBridge<LogRetentionRecord[]>("list_log_retention_settings_command");
    return {
      mode: "ready",
      retention: buildLogRetentionSettingsView(retention),
      about: buildReleaseAboutView({ version: "0.1.0", build: "local", packaged: true }),
      migrationGuidance: buildMigrationFailureGuidance("Migration failure details are redacted before display."),
    };
  } catch (error) {
    const message = error instanceof Error ? error.message : typeof error === "string" ? error : "native bridge unavailable";
    return { mode: "error", error: buildMigrationFailureGuidance(message).message };
  }
}

export async function dryRunLogCleanup(invokeBridge: ReleaseHardeningInvoke, scope = "default"): Promise<LogCleanupRunRecord> {
  return invokeBridge<LogCleanupRunRecord>("cleanup_logs_command", { scope, dryRun: true });
}
