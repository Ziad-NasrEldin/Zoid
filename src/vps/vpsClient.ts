import { invoke } from "@tauri-apps/api/core";
import type { HostingerVpsOperationResult, HostingerVpsOverview } from "./types";

export async function getHostingerVpsOverview(): Promise<HostingerVpsOverview> {
  return invoke<HostingerVpsOverview>("hostinger_vps_get_overview");
}

export async function refreshHostingerVps(): Promise<HostingerVpsOverview> {
  return invoke<HostingerVpsOverview>("hostinger_vps_refresh");
}

export async function runHostingerVpsAction(virtualMachineId: string, action: "start" | "stop" | "restart"): Promise<HostingerVpsOperationResult> {
  return invoke<HostingerVpsOperationResult>("hostinger_vps_run_action", { virtualMachineId, action });
}
