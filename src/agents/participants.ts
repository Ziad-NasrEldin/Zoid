import type { ChatParticipant } from "./types";

export const userParticipant: ChatParticipant = {
  id: "ziad",
  displayName: "Ziad Salah",
  handle: "operator/local",
  initials: "ZS",
  presence: "online",
};

export const hermesParticipant: ChatParticipant = {
  id: "hermes",
  displayName: "Hermes",
  handle: "hermes-agent",
  initials: "HA",
  presence: "offline",
};

export const participantsById = {
  ziad: userParticipant,
  hermes: hermesParticipant,
} satisfies Record<ChatParticipant["id"], ChatParticipant>;
