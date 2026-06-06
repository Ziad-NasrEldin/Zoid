import {
  buildLogRetentionSettingsView,
  buildMigrationFailureGuidance,
  buildReleaseAboutView,
  dryRunLogCleanup,
  loadReleaseHardeningState,
  type ReleaseHardeningInvoke,
} from "./releaseAbout";

const assertEqual = <T>(actual: T, expected: T, message: string) => {
  if (actual !== expected) throw new Error(`${message}: expected ${String(expected)}, got ${String(actual)}`);
};
const assertMatch = (actual: string, expected: RegExp, message: string) => {
  if (!expected.test(actual)) throw new Error(`${message}: ${actual} did not match ${expected}`);
};
const assertTrue = (condition: boolean, message: string) => {
  if (!condition) throw new Error(message);
};

const retention = buildLogRetentionSettingsView([
  { scope: "agent", retention_days: 14, max_total_bytes: 10_485_760, enabled: true },
]);
assertEqual(retention[0].scope, "agent", "scope is preserved");
assertMatch(retention[0].summary, /keep 14 day/, "summary includes retention days");

const about = buildReleaseAboutView({ version: "0.1.0", build: "local", packaged: true });
assertEqual(about.appName, "Zoid", "app name");
assertMatch(about.signing, /no certificate secret/i, "signing copy is safe");
assertTrue(about.safeDiagnostics.every((line) => !/token=|password=/i.test(line)), "safe diagnostics omit raw secrets");

const migration = buildMigrationFailureGuidance("failed token=raw-secret-value");
assertMatch(migration.message, /\[REDACTED\]/, "migration guidance redacts secret-like values");
assertTrue(!migration.message.includes("raw-secret-value"), "migration guidance omits raw secret");
assertTrue(migration.actions.length >= 3, "migration guidance has recovery actions");

const calls: Array<{command: string; args?: Record<string, unknown>}> = [];
const invoke: ReleaseHardeningInvoke = async (command, args) => {
  calls.push({ command, args });
  if (command === "list_log_retention_settings_command") {
    return [{ scope: "default", retention_days: 30, max_total_bytes: 10485760, enabled: true }] as never;
  }
  if (command === "cleanup_logs_command") {
    return { id: "cleanup-1", scope: args?.scope, dry_run: args?.dryRun, files_considered: 2, files_deleted: 0, bytes_deleted: 0, status: "completed", created_at: "now" } as never;
  }
  throw new Error("unexpected command");
};
const state = await loadReleaseHardeningState(invoke);
assertEqual(state.mode, "ready", "release hardening state loads");
assertEqual(calls[0].command, "list_log_retention_settings_command", "retention command invoked");
const cleanup = await dryRunLogCleanup(invoke, "agent");
assertEqual(cleanup.dry_run, true, "cleanup is dry-run by default in UI helper");
assertEqual(calls[1].command, "cleanup_logs_command", "cleanup command invoked");

console.log("release/about/log-retention view-model tests passed");
