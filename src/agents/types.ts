export type AgentId = "hermes";
export type ChatRole = "user" | "assistant" | "system";
export type AgentConnectionState = "checking" | "online" | "offline" | "unauthorized" | "error";

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

export type HermesHealth = {
  ok: boolean;
  status: AgentConnectionState extends infer S ? Exclude<S, "checking"> : never;
  message: string;
  model?: string;
};

export type HermesChatRequestMessage = {
  role: ChatRole;
  content: string;
};

export type HermesChatResponse = {
  content: string;
  model?: string;
};
