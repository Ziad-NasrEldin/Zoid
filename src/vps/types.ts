export type HostingerVpsState =
  | "running"
  | "starting"
  | "stopping"
  | "stopped"
  | "creating"
  | "initial"
  | "error"
  | "suspending"
  | "unsuspending"
  | "suspended"
  | "destroying"
  | "destroyed"
  | "recreating"
  | "restoring"
  | "recovery"
  | "stopping_recovery"
  | "unknown";

export type HostingerVirtualMachine = {
  id: string;
  hostname: string;
  state: HostingerVpsState | string;
  plan: string | null;
  primaryIp: string | null;
  location: string | null;
  actionsLock: string | null;
  cpus: number | null;
  memoryMb: number | null;
  diskGb: number | null;
};

export type HostingerVpsActionLog = {
  id: string;
  virtualMachineId: string;
  action: "start" | "stop" | "restart" | string;
  state: string;
  createdAt: string;
  providerActionId: string | null;
  message: string;
};

export type HostingerVpsOverview = {
  tokenPresent: boolean;
  servers: HostingerVirtualMachine[];
  actions: HostingerVpsActionLog[];
  lastSyncedAt: string | null;
  lastError: string | null;
  cachePath: string;
  updatedAt: string;
};

export type HostingerVpsOperationResult = {
  ok: boolean;
  message: string;
  action: HostingerVpsActionLog;
  overview: HostingerVpsOverview;
};
