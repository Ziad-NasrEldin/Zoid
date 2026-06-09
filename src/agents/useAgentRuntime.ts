import { invoke } from "@tauri-apps/api/core";
import { useCallback, useMemo, useRef, useState } from "react";

export type AgentRunStatus = "idle" | "running" | "needs-input" | "error" | "interrupted";
export type AgentRunHandle = { runId: string; sessionId: string; startedAt: string; assistantId?: string; processId?: number; backendHandle?: string };
export type AgentSessionRuntimeState = {
  sessionId: string;
  status: AgentRunStatus;
  currentRun?: AgentRunHandle;
  queuedPrompts: string[];
  lastStartedAt?: string;
  lastFinishedAt?: string;
  lastError?: string;
  wasRunningBeforeClose?: boolean;
};
export type StartAgentRunInput = { sessionId: string; prompt: string; hermesSessionId?: string; cwd?: string; metadata?: Record<string, string> };
export type StartAgentRunResult = { runId: string; sessionId: string; startedAt: string };
export type AgentRunEvent = {
  type: "agent-run-started" | "agent-run-output" | "agent-run-error" | "agent-run-needs-input" | "agent-run-completed" | "agent-run-stopped";
  runId: string;
  sessionId: string;
  timestamp: string;
  sequence: number;
  channel?: "stdout" | "stderr" | "system";
  chunk?: string;
  message?: string;
  exitCode?: number;
};

export const MAX_ACTIVE_AGENT_RUNS = 4;

export function startAgentRun(input: StartAgentRunInput): Promise<StartAgentRunResult> {
  const runId = `agent-run-${crypto.randomUUID()}`;
  return invoke("send_hermes_cli_run_message", {
    messages: [{ role: "user", content: input.prompt }],
    linkedRepository: input.cwd,
    hermesSession: input.hermesSessionId,
    sessionId: input.sessionId,
    runId,
  }).then(() => ({ runId, sessionId: input.sessionId, startedAt: new Date().toISOString() }));
}

export function stopAgentRun(input: { sessionId?: string; runId: string }): Promise<{ runId: string; stopped: boolean; reason?: "running" | "already-completed" | "already-stopped" | "unknown-run" }> {
  return invoke<boolean>("cancel_hermes_cli_run", { sessionId: input.sessionId, runId: input.runId }).then((stopped) => ({ runId: input.runId, stopped, reason: stopped ? "running" : "unknown-run" }));
}

export type BackendAgentRunSnapshot = { sessionId: string; runId: string; startedAt: string; status: "running" | "stopping"; pid?: number };

export function listAgentRuns(): Promise<Array<AgentRunHandle & { status: "running" | "stopping"; pid?: number }>> {
  return invoke<BackendAgentRunSnapshot[]>("list_hermes_cli_runs").then((runs) => runs.map((run) => ({
    runId: run.runId,
    sessionId: run.sessionId,
    startedAt: run.startedAt,
    status: run.status,
    pid: run.pid,
  })));
}

export function emptyRuntimeState(sessionId: string): AgentSessionRuntimeState {
  return { sessionId, status: "idle", queuedPrompts: [] };
}

export function reconcileAgentRunEvent(state: Record<string, AgentSessionRuntimeState>, event: AgentRunEvent): Record<string, AgentSessionRuntimeState> {
  const current = state[event.sessionId] ?? emptyRuntimeState(event.sessionId);
  if (current.currentRun && current.currentRun.runId !== event.runId) return state;
  const terminal = event.type === "agent-run-completed" || event.type === "agent-run-error" || event.type === "agent-run-stopped";
  return {
    ...state,
    [event.sessionId]: {
      ...current,
      status: event.type === "agent-run-error" ? "error" : event.type === "agent-run-needs-input" ? "needs-input" : terminal ? "idle" : "running",
      currentRun: terminal ? undefined : current.currentRun ?? { runId: event.runId, sessionId: event.sessionId, startedAt: event.timestamp },
      lastStartedAt: current.lastStartedAt ?? event.timestamp,
      lastFinishedAt: terminal ? event.timestamp : current.lastFinishedAt,
      lastError: event.type === "agent-run-error" ? event.message ?? "Agent run failed" : current.lastError,
    },
  };
}

export function useAgentRuntime() {
  const [runtimeBySessionId, setRuntimeBySessionId] = useState<Record<string, AgentSessionRuntimeState>>({});
  const runtimeRef = useRef(runtimeBySessionId);
  const activeSessionIdsRef = useRef<Set<string>>(new Set());
  runtimeRef.current = runtimeBySessionId;

  const activeRunCount = useMemo(() => Object.values(runtimeBySessionId).filter((state) => state.status === "running" || state.status === "needs-input").length, [runtimeBySessionId]);
  const getSessionRuntime = useCallback((sessionId: string) => runtimeRef.current[sessionId] ?? emptyRuntimeState(sessionId), []);
  const canStartSessionRun = useCallback((sessionId: string) => !activeSessionIdsRef.current.has(sessionId) && activeSessionIdsRef.current.size < MAX_ACTIVE_AGENT_RUNS, []);

  const markSessionRunStarted = useCallback((sessionId: string, runId: string, assistantId = runId, startedAt = new Date().toISOString()) => {
    activeSessionIdsRef.current.add(sessionId);
    setRuntimeBySessionId((current) => ({ ...current, [sessionId]: { ...(current[sessionId] ?? emptyRuntimeState(sessionId)), status: "running", currentRun: { runId, sessionId, assistantId, startedAt }, lastStartedAt: startedAt, lastError: undefined } }));
  }, []);

  const tryStartSessionRun = useCallback((sessionId: string, runId: string, assistantId = runId) => {
    if (!canStartSessionRun(sessionId)) return false;
    markSessionRunStarted(sessionId, runId, assistantId);
    return true;
  }, [canStartSessionRun, markSessionRunStarted]);

  const markSessionRunFinished = useCallback((sessionId: string, status: AgentRunStatus = "idle", error?: string, runId?: string) => {
    const currentRun = runtimeRef.current[sessionId]?.currentRun;
    if (runId && currentRun && currentRun.runId !== runId) return;
    activeSessionIdsRef.current.delete(sessionId);
    setRuntimeBySessionId((current) => {
      const previous = current[sessionId] ?? emptyRuntimeState(sessionId);
      if (runId && previous.currentRun && previous.currentRun.runId !== runId) return current;
      return { ...current, [sessionId]: { ...previous, status, currentRun: undefined, lastFinishedAt: new Date().toISOString(), lastError: error } };
    });
  }, []);

  const queuePrompt = useCallback((sessionId: string, prompt: string) => {
    setRuntimeBySessionId((current) => {
      const previous = current[sessionId] ?? emptyRuntimeState(sessionId);
      return { ...current, [sessionId]: { ...previous, queuedPrompts: [...previous.queuedPrompts, prompt] } };
    });
  }, []);
  const dequeuePrompt = useCallback((sessionId: string) => {
    const prompt = runtimeRef.current[sessionId]?.queuedPrompts[0];
    if (!prompt) return undefined;
    setRuntimeBySessionId((current) => {
      const previous = current[sessionId] ?? emptyRuntimeState(sessionId);
      return { ...current, [sessionId]: { ...previous, queuedPrompts: previous.queuedPrompts.slice(1) } };
    });
    return prompt;
  }, []);
  return { runtimeBySessionId, activeRunCount, getSessionRuntime, canStartSessionRun, tryStartSessionRun, markSessionRunStarted, markSessionRunFinished, queuePrompt, dequeuePrompt };
}
