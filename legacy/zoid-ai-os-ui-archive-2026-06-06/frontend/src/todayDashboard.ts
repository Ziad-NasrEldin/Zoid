export type TodayDashboardBadgeTone = "primary" | "blue" | "muted";

export type TodayDashboardItem = {
  title: string;
  meta?: string;
  tone?: TodayDashboardBadgeTone;
  action?: string;
  completed?: boolean;
};

export type TodayDashboardPanel = {
  title: string;
  eyebrow?: string;
  status?: string;
  statusTone?: TodayDashboardBadgeTone;
  copy?: string;
  items: TodayDashboardItem[];
  media?: {
    kind: "product-photo";
    label: string;
    alt: string;
  };
  progress?: number;
};

export type TodayDashboardView = {
  title: string;
  dateLabel: string;
  navigation: string[];
  actions: string[];
  designSystem: {
    name: "Apple-design-analysis";
    primary: "#0066cc";
    canvas: "#ffffff";
    parchment: "#f5f5f7";
    ink: "#1d1d1f";
    radiusCard: "18px";
    radiusPill: "9999px";
    bodySize: "17px";
    disallowedUiShadow: true;
  };
  sampleNotice: string;
  hero: {
    eyebrow: string;
    time: string;
    title: string;
    copy: string;
    tags: string[];
  };
  topPanels: TodayDashboardPanel[];
  operationPanels: TodayDashboardPanel[];
  secondaryPanels: TodayDashboardPanel[];
};

export function buildTodayDashboardView(input: { currentDateLabel?: string } = {}): TodayDashboardView {
  return {
    title: "Today",
    dateLabel: input.currentDateLabel ?? "Friday, Jun 5",
    navigation: ["Dashboard", "Analytics", "Review"],
    actions: ["Inspect", "New"],
    sampleNotice: "Craft sample for layout and component behavior only. Real native Zoid state remains in the truth panel below; the sample cards do not claim live records.",
    designSystem: {
      name: "Apple-design-analysis",
      primary: "#0066cc",
      canvas: "#ffffff",
      parchment: "#f5f5f7",
      ink: "#1d1d1f",
      radiusCard: "18px",
      radiusPill: "9999px",
      bodySize: "17px",
      disallowedUiShadow: true,
    },
    hero: {
      eyebrow: "AI Daily Brief",
      time: "8:30 AM",
      title: "Review native evidence first, then move blocked work through the right workspace.",
      copy: "The dashboard shows how Zoid should prioritize work without inventing records. Use native panels and review gates as the source of truth.",
      tags: ["Native evidence", "Review gates", "No simulation"],
    },
    topPanels: [
      {
        title: "Needs Attention",
        eyebrow: "Needs attention",
        statusTone: "muted",
        items: [
          { title: "Blocked agent run sample", meta: "Requires native run evidence", tone: "muted" },
          { title: "Automation failure sample", meta: "No provider state is fabricated", tone: "muted" },
          { title: "Content review sample", meta: "Approval gate remains visible", tone: "primary" },
        ],
      },
      {
        title: "Active Work",
        eyebrow: "Active work",
        copy: "Native run list is not connected on this sample surface.",
        status: "Bridge required",
        statusTone: "muted",
        progress: 0,
        items: [{ title: "Run-state component sample", meta: "Waits for native AgentRun rows", tone: "muted" }],
      },
    ],
    operationPanels: [
      {
        title: "Tasks",
        status: "Due",
        statusTone: "primary",
        items: [
          { title: "Task row sample", meta: "Hydrate from native Tasks" },
          { title: "Required-fix row sample", meta: "Keep blocker visible" },
          { title: "Completed row sample", meta: "Status label, not color alone", completed: true },
        ],
      },
      {
        title: "Calendar",
        items: [
          { title: "Calendar event sample", meta: "External bridge unavailable until connected", tone: "muted" },
          { title: "Scheduled intent sample", meta: "Cannot publish without provider evidence", tone: "muted" },
        ],
      },
      {
        title: "Content Queue",
        status: "Draft",
        media: {
          kind: "product-photo",
          label: "Content asset preview sample",
          alt: "Labeled design-only content preview; not a persisted native asset.",
        },
        items: [{ title: "Draft content sample", meta: "Open real Content workspace for native state", action: "Inspect", tone: "primary" }],
      },
      {
        title: "Agents Status",
        items: [
          { title: "Agent completion sample", meta: "Requires native run evidence", tone: "muted" },
          { title: "Processing queue sample", meta: "No counts without native query", tone: "muted" },
        ],
      },
    ],
    secondaryPanels: [
      {
        title: "Dirty Repos",
        items: [
          { title: "Repo attention sample", meta: "Git status must come from repo bridge", action: "Inspect", tone: "blue" },
          { title: "Build blocker sample", meta: "Failure requires command output", action: "Review", tone: "muted" },
        ],
      },
      {
        title: "Automations",
        items: [
          { title: "Automation success sample", meta: "Needs event evidence", tone: "primary" },
          { title: "Automation failure sample", meta: "Fail closed until inspected", tone: "muted" },
        ],
      },
      {
        title: "Recent Activity",
        items: [
          { title: "Agent activity sample", meta: "Native history required", tone: "muted" },
          { title: "Code change sample", meta: "Git diff required", tone: "blue" },
          { title: "Inbox sample", meta: "External provider bridge required", tone: "muted" },
        ],
      },
      {
        title: "Inbox Brief",
        status: "Sample",
        statusTone: "muted",
        items: [
          { title: "Inbox item sample", meta: "Provider data hidden until bridge is connected", tone: "muted" },
          { title: "Review item sample", meta: "Open native inbox/review evidence", tone: "muted" },
        ],
      },
    ],
  };
}
