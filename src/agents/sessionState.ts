import type { RepositoryOperationAction } from "../code/repositoryOperations";
import type { ChatMessage } from "./types";
import { chooseUniqueSessionAgentAvatarId } from "./sessionPortraits";

export const HERMES_LEGACY_WELCOME_COPY = "Hermes is linked through the local terminal CLI. Prompts run through your configured Hermes/Codex setup.";
export const HERMES_WELCOME_COPY = "Hermes is awake. Drop the mission, the repo, or the mess — Zoid will route it through your local command deck.";

const welcomeMessage: ChatMessage = {
  id: "hermes-welcome",
  role: "assistant",
  participantId: "hermes",
  content: HERMES_WELCOME_COPY,
  createdAt: new Date().toISOString(),
  status: "sent",
};

export type HermesChatSession = {
  id: string;
  title: string;
  createdAt: string;
  updatedAt: string;
  messages: ChatMessage[];
  linkedRepositoryId?: string;
  hermesCliSessionId?: string;
  modelName?: string;
  reasoningEffort?: string;
  operationRunId?: string;
  operationAction?: RepositoryOperationAction;
  operationRepositoryId?: string;
  pendingInitialPrompt?: string;
  needsReply?: boolean;
  lastNotifiedAssistantMessageId?: string;
  notificationUpdatedAt?: string;
  portraitId?: string;
};

export type ArchivedHermesChatSession = HermesChatSession & {
  archivedAt: string;
};

export function refreshHermesWelcomeCopy(session: HermesChatSession): HermesChatSession {
  return {
    ...session,
    messages: session.messages.map((message) => (
      message.role === "assistant" && message.participantId === "hermes" && message.content === HERMES_LEGACY_WELCOME_COPY
        ? { ...message, content: HERMES_WELCOME_COPY }
        : message
    )),
  };
}

export function createSession(title = "New session", existingSessions: readonly HermesChatSession[] = []): HermesChatSession {
  const now = new Date().toISOString();
  const id = `session-${crypto.randomUUID()}`;
  return {
    id,
    title,
    createdAt: now,
    updatedAt: now,
    portraitId: chooseUniqueSessionAgentAvatarId(existingSessions.map((session) => session.portraitId), id),
    messages: [{ ...welcomeMessage, id: `hermes-welcome-${crypto.randomUUID()}`, createdAt: now }],
  };
}
