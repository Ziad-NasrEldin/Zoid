import { invoke } from "@tauri-apps/api/core";
import type { AutomationAction, AutomationList } from "./types";

export function listHermesAutomations(): Promise<AutomationList> {
  return invoke<AutomationList>("list_hermes_automations");
}

export function manageHermesCronJob(jobId: string, action: AutomationAction): Promise<AutomationList> {
  return invoke<AutomationList>("manage_hermes_cron_job", { jobId, job_id: jobId, action });
}
