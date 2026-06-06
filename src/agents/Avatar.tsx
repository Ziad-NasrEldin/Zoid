import type { ChatParticipant } from "./types";

type AvatarProps = {
  participant: ChatParticipant;
  size?: "sm" | "md" | "lg";
  showPresence?: boolean;
};

export function Avatar({ participant, size = "md", showPresence = false }: AvatarProps) {
  return (
    <span
      aria-label={`${participant.displayName} avatar`}
      className={`chat-avatar chat-avatar--${size} chat-avatar--${participant.id}`}
    >
      {participant.avatarUrl ? (
        <img alt="" src={participant.avatarUrl} />
      ) : (
        <span aria-hidden="true">{participant.initials}</span>
      )}
      {showPresence ? (
        <span aria-hidden="true" className={`avatar-presence avatar-presence--${participant.presence}`} />
      ) : null}
    </span>
  );
}
