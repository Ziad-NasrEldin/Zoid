export type AgentId = "hermes";
export type ChatRole = "user" | "assistant" | "system";
export type AgentConnectionState = "checking" | "online" | "offline" | "error";

export type ChatParticipant = {
  id: "ziad" | AgentId;
  displayName: string;
  handle: string;
  avatarUrl?: string;
  initials: string;
  presence: "online" | "offline" | "thinking";
};

export type ChatMessage = {
  id: string;
  role: Exclude<ChatRole, "system">;
  participantId: ChatParticipant["id"];
  content: string;
  createdAt: string;
  status: "sending" | "streaming" | "sent" | "error";
  error?: string;
};

export type HermesCliStatus = {
  ok: boolean;
  status: Exclude<AgentConnectionState, "checking">;
  message: string;
  command?: string;
  session: string;
};

export type HermesCliMessage = {
  role: ChatRole;
  content: string;
};

export type HermesCliResponse = {
  content: string;
  session: string;
};
