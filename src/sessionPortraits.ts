export type SessionAgentAvatar = {
  id: string;
  name: string;
  asset: string;
  accent: string;
  paper: string;
  focalPoint: string;
};

export const SESSION_AGENT_AVATARS = [
  { id: "agent-avatar-01", name: "Agent Avatar 01", asset: "/agent-avatars/06111fa874ffbdd38cd79bf40466d8e4.jpg", accent: "#0d0a0a", paper: "#f7f5f4", focalPoint: "50% 44%" },
  { id: "agent-avatar-02", name: "Agent Avatar 02", asset: "/agent-avatars/0f1b53fca862629b3c26d60443613607.jpg", accent: "#8d2f45", paper: "#f8f4ea", focalPoint: "50% 42%" },
  { id: "agent-avatar-03", name: "Agent Avatar 03", asset: "/agent-avatars/148a067dbe7ef0167a0cb5befe843d58.jpg", accent: "#2f6f73", paper: "#efe6d1", focalPoint: "50% 45%" },
  { id: "agent-avatar-04", name: "Agent Avatar 04", asset: "/agent-avatars/39a21335e76840d70c2f7eacbb86721f.jpg", accent: "#936f2e", paper: "#f4eadc", focalPoint: "50% 43%" },
  { id: "agent-avatar-05", name: "Agent Avatar 05", asset: "/agent-avatars/5a7f5d298b7e3105f9d8342e494b21b3.jpg", accent: "#5c4a8b", paper: "#e8dfec", focalPoint: "50% 44%" },
  { id: "agent-avatar-06", name: "Agent Avatar 06", asset: "/agent-avatars/5b1748ef17b6b664045d1f5ad3efe292.jpg", accent: "#c77d2f", paper: "#e9dcc4", focalPoint: "50% 42%" },
  { id: "agent-avatar-07", name: "Agent Avatar 07", asset: "/agent-avatars/9c73917cce1d6977578952a41a098e4a.jpg", accent: "#111111", paper: "#dfe8df", focalPoint: "50% 44%" },
  { id: "agent-avatar-08", name: "Agent Avatar 08", asset: "/agent-avatars/b36ef9ea2212a53ea1e58fa72ccb878f.jpg", accent: "#b4453f", paper: "#f8f4ea", focalPoint: "50% 43%" },
  { id: "agent-avatar-09", name: "Agent Avatar 09", asset: "/agent-avatars/ccffb4ce0dc183f0f74d5f5558509ffd.jpg", accent: "#6f7f38", paper: "#efe6d1", focalPoint: "50% 45%" },
  { id: "agent-avatar-10", name: "Agent Avatar 10", asset: "/agent-avatars/d7537962b1936cf182e27a75c0a79b67.jpg", accent: "#5f3333", paper: "#f4eadc", focalPoint: "50% 44%" },
  { id: "agent-avatar-11", name: "Agent Avatar 11", asset: "/agent-avatars/e1033d8e1ecb6a1d9f047d218e630500.jpg", accent: "#0d0a0a", paper: "#f7f5f4", focalPoint: "50% 42%" },
  { id: "agent-avatar-12", name: "Agent Avatar 12", asset: "/agent-avatars/e7289233c3d817e0bb93c97cf2ab4e38.jpg", accent: "#2f6f73", paper: "#e8dfec", focalPoint: "50% 45%" },
  { id: "agent-avatar-13", name: "Agent Avatar 13", asset: "/agent-avatars/e9790f185c409c1e29eb5a01a8ecc167.jpg", accent: "#936f2e", paper: "#e9dcc4", focalPoint: "50% 43%" },
  { id: "agent-avatar-14", name: "Agent Avatar 14", asset: "/agent-avatars/fc001c2ca99c37cb76c0daccdf780073.jpg", accent: "#8d2f45", paper: "#dfe8df", focalPoint: "50% 44%" },
] as const satisfies readonly SessionAgentAvatar[];

export function hashSessionAvatarIndex(sessionId: string) {
  let hash = 2166136261;
  for (let index = 0; index < sessionId.length; index += 1) {
    hash ^= sessionId.charCodeAt(index);
    hash = Math.imul(hash, 16777619);
  }
  return Math.abs(hash) % SESSION_AGENT_AVATARS.length;
}

export function getSessionAgentAvatarById(portraitId: string | undefined) {
  return SESSION_AGENT_AVATARS.find((avatar) => avatar.id === portraitId);
}

export function getSessionAgentAvatar(sessionId: string, portraitId?: string) {
  return getSessionAgentAvatarById(portraitId) ?? SESSION_AGENT_AVATARS[hashSessionAvatarIndex(sessionId)];
}

export function chooseUniqueSessionAgentAvatarId(existingPortraitIds: readonly (string | undefined)[], sessionId: string) {
  const usedPortraitIds = new Set(existingPortraitIds.filter((value): value is string => Boolean(value)));
  const availableAvatars = SESSION_AGENT_AVATARS.filter((avatar) => !usedPortraitIds.has(avatar.id));
  const pool = availableAvatars.length > 0 ? availableAvatars : SESSION_AGENT_AVATARS;
  return pool[hashSessionAvatarIndex(sessionId) % pool.length].id;
}
