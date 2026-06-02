import type { FoundationStatus, WorkspaceRecord } from "./settingsStatus";

export type { WorkspaceRecord } from "./settingsStatus";

export const fallbackWorkspaces: WorkspaceRecord[] = [
  { id: "today", label: "Today", description: "Command center, attention, and current work.", position: 0 },
  { id: "tasks", label: "Tasks", description: "First-class tasks, review states, and follow-ups.", position: 1 },
  { id: "notes", label: "Notes", description: "Markdown notes with local metadata.", position: 2 },
  { id: "agents", label: "Agents", description: "CLI profiles, sessions, runs, and reviews.", position: 3 },
  { id: "code", label: "Code", description: "Repositories, Launch Gate, and git work.", position: 4 },
  { id: "content", label: "Content", description: "Planning, review, and OmniSocials publishing state.", position: 5 },
  { id: "automations", label: "Automations", description: "Visible recurring jobs and run history.", position: 6 },
  { id: "business", label: "Business", description: "Contacts, companies, follow-ups, and linked work.", position: 7 },
  { id: "products", label: "Products", description: "First-class product hubs and timelines.", position: 8 },
  { id: "files", label: "Files", description: "Local file manager and Zoid-aware attachments.", position: 9 },
  { id: "browser", label: "Browser", description: "Work webview/capture workspace.", position: 10 },
  { id: "inbox", label: "Inbox", description: "Notifications, approvals, blockers, and Gmail state.", position: 11 },
  { id: "calendar", label: "Calendar", description: "Built-in calendar with Apple Calendar integration gates.", position: 12 },
  { id: "history", label: "History", description: "Universal timeline and linked event history.", position: 13 },
];

export type WorkspaceRegistrySource = "native" | "fallback" | "checking";

export type WorkspaceRegistryView = {
  countLabel: string;
  source: WorkspaceRegistrySource;
  sourceLabel: string;
  truthCopy: string;
  workspaces: WorkspaceRecord[];
};

export type WorkspaceChromeView = {
  activeWorkspace: WorkspaceRecord | null;
  activeWorkspaceDescription: string;
  activeWorkspaceLabel: string;
  glyphs: Record<string, string>;
  registryEmptyCopy: string | null;
  sidebarEmptyCopy: string | null;
};

export const workspaceGlyphs: Record<string, string> = {
  agents: "A",
  automations: "ƒ",
  browser: "⌘",
  business: "B",
  calendar: "C",
  code: "</>",
  content: "P",
  files: "F",
  history: "H",
  inbox: "I",
  notes: "N",
  products: "R",
  tasks: "✓",
  today: "•",
};

export function formatWorkspaceCount(count: number) {
  return `${count} workspace${count === 1 ? "" : "s"}`;
}

export function sortWorkspaces(workspaces: WorkspaceRecord[]) {
  return [...workspaces].sort((a, b) => a.position - b.position);
}

export function buildWorkspaceRegistryView(status: FoundationStatus | null, statusError: string | null): WorkspaceRegistryView {
  if (status) {
    const workspaces = sortWorkspaces(status.workspaces);
    const countLabel = formatWorkspaceCount(workspaces.length);

    return {
      countLabel,
      source: "native",
      sourceLabel: "Native registry",
      truthCopy: `Rendering ${countLabel} returned by get_foundation_status. Browser preview fallback is not mixed into native data.`,
      workspaces,
    };
  }

  if (statusError) {
    const workspaces = sortWorkspaces(fallbackWorkspaces);
    const countLabel = formatWorkspaceCount(workspaces.length);

    return {
      countLabel,
      source: "fallback",
      sourceLabel: "Browser preview fallback",
      truthCopy: `Showing ${countLabel} from static browser preview data because native status is unavailable outside Tauri.`,
      workspaces,
    };
  }

  const workspaces = sortWorkspaces(fallbackWorkspaces);
  const countLabel = formatWorkspaceCount(workspaces.length);

  return {
    countLabel,
    source: "checking",
    sourceLabel: "Checking native registry",
    truthCopy: `Temporarily showing ${countLabel} from browser preview data while get_foundation_status is loading.`,
    workspaces,
  };
}

export function buildWorkspaceChromeView(registry: WorkspaceRegistryView, activeWorkspaceId: string): WorkspaceChromeView {
  const activeWorkspace = registry.workspaces.find((workspace) => workspace.id === activeWorkspaceId) ?? registry.workspaces[0] ?? null;
  const glyphs = Object.fromEntries(
    registry.workspaces.map((workspace) => [workspace.id, workspaceGlyphs[workspace.id] ?? workspace.label.slice(0, 1)]),
  );

  return {
    activeWorkspace,
    activeWorkspaceLabel: activeWorkspace?.label ?? "No workspaces registered",
    activeWorkspaceDescription: activeWorkspace?.description ?? "The native workspace registry is empty.",
    glyphs,
    registryEmptyCopy: registry.workspaces.length > 0 ? null : "The native registry returned no workspaces.",
    sidebarEmptyCopy: registry.workspaces.length > 0 ? null : "No native workspaces registered.",
  };
}
