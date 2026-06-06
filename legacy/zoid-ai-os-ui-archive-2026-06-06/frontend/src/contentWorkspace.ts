export type ContentPlanRecord = { id: string; title: string; pillar: string; status: string; owner_actor_type: string; metadata_json: string };
export type ContentPieceRecord = { id: string; plan_id: string; title: string; body_markdown: string; status: string; platforms_json: string; required_gate: string; metadata_json: string };
export type MediaAssetRecord = { id: string; piece_id: string; asset_kind: string; storage_ref: string; alt_text: string; metadata_json: string };
export type ContentReviewGateRecord = { id: string; piece_id: string; gate_type: string; status: string; evidence_summary: string };
export type ContentScheduleRecord = { id: string; piece_id: string; platform: string; scheduled_for: string; status: string; confirmation_id?: string | null };
export type ContentVerificationRecord = { id: string; piece_id?: string | null; schedule_id?: string | null; platform: string; action_type: string; outcome: string; provider_status?: string | null; failure_report?: string | null };
export type OmniSocialsStatusRecord = { state: string; platform: string; credential_ref?: string | null; status_note: string };

export type ContentWorkspaceState =
  | { mode: "loading" }
  | { mode: "error"; error: string }
  | {
      mode: "ready";
      plans: ContentPlanRecord[];
      pieces: ContentPieceRecord[];
      mediaAssets: MediaAssetRecord[];
      reviewGates: ContentReviewGateRecord[];
      schedules: ContentScheduleRecord[];
      verifications: ContentVerificationRecord[];
      omnisocials: OmniSocialsStatusRecord;
      selectedPieceId: string | null;
      actionStatus?: string | null;
    };

export function parsePlatforms(platformsJson: string): string[] {
  try {
    const parsed = JSON.parse(platformsJson || "[]");
    return Array.isArray(parsed) ? parsed.filter((item): item is string => typeof item === "string") : [];
  } catch {
    return [];
  }
}

export function pieceScheduleGateSummary(piece: ContentPieceRecord, gates: ContentReviewGateRecord[], schedules: ContentScheduleRecord[]): string {
  const pieceGates = gates.filter((gate) => gate.piece_id === piece.id && gate.gate_type === piece.required_gate);
  const hasApprovedGate = piece.required_gate === "none" || pieceGates.some((gate) => gate.status === "approved");
  const hasSchedule = schedules.some((schedule) => schedule.piece_id === piece.id && schedule.status === "intent");
  if (hasSchedule) return "Schedule intent recorded locally — no external publish implied.";
  if (!hasApprovedGate) return "Schedule blocked until specialist review is approved.";
  return "Review gate passed; schedule still needs human confirmation.";
}

export function omnisocialsActionCopy(status: OmniSocialsStatusRecord): string {
  if (status.state === "connected" && status.credential_ref) {
    return "Connection metadata exists, but Phase 5 still records manual/fail-closed action evidence before any external write.";
  }
  return "Upload, schedule, and publish are fail-closed because OmniSocials is not configured.";
}

export function blockedVerificationRecords(records: ContentVerificationRecord[]): ContentVerificationRecord[] {
  return records.filter((record) => record.outcome === "blocked" || record.outcome === "failed");
}

export type ContentWorkspaceDesignScreen = {
  id: string;
  title: string;
  group: "Command" | "Planning" | "Production" | "Review" | "Operations" | "Integrations";
  intent: string;
  primaryMetric: string;
  secondaryMetric: string;
  keyRegions: string[];
  surfaceKind: "dashboard" | "brand" | "wizard" | "editor" | "calendar" | "pipeline" | "detail" | "override" | "queue" | "report" | "modal" | "recovery" | "mapping" | "library" | "agents" | "mirror";
};

export type ContentWorkspaceDesignView = {
  sourceProjectId: string;
  sourceProjectTitle: string;
  sampleNotice: string;
  designSystem: {
    name: "Apple-design-analysis";
    primary: "#0066cc";
    canvas: "#ffffff";
    parchment: "#f5f5f7";
    ink: "#1d1d1f";
    radiusCard: "18px";
    radiusPill: "9999px";
    disallowedUiShadow: true;
  };
  screens: ContentWorkspaceDesignScreen[];
  navigation: string[];
};

export function buildContentWorkspaceDesignView(): ContentWorkspaceDesignView {
  return {
    sourceProjectId: "2534809720873389640",
    sourceProjectTitle: "Zoid macOS Desktop Sitemap",
    sampleNotice: "Visual implementation of the Stitch Content Workspace. These operational cards are design-copy only; native content state remains fail-closed in preview and no external publishing is implied.",
    designSystem: {
      name: "Apple-design-analysis",
      primary: "#0066cc",
      canvas: "#ffffff",
      parchment: "#f5f5f7",
      ink: "#1d1d1f",
      radiusCard: "18px",
      radiusPill: "9999px",
      disallowedUiShadow: true,
    },
    navigation: ["Dashboard", "Campaigns", "Calendar", "Pipeline", "Reviews", "Automation", "Evidence", "Settings"],
    screens: [
      {
        id: "autonomous-campaign-dashboard",
        title: "Autonomous Campaign Dashboard",
        group: "Command",
        intent: "Executive command surface for active campaigns, confidence, agent progress, and next actions.",
        primaryMetric: "4 active campaigns",
        secondaryMetric: "82% automation confidence",
        keyRegions: ["Campaign health hero", "agent run strip", "risk queue", "next scheduled slot"],
        surfaceKind: "dashboard",
      },
      {
        id: "brand-management-mavoid",
        title: "Brand Management - MaVoid",
        group: "Planning",
        intent: "Brand system manager for voice, pillars, account defaults, and reusable content constraints.",
        primaryMetric: "6 voice rules",
        secondaryMetric: "3 active markets",
        keyRegions: ["Brand profile", "voice guardrails", "pillar matrix", "market/account defaults"],
        surfaceKind: "brand",
      },
      {
        id: "new-campaign-wizard",
        title: "New Campaign Wizard",
        group: "Planning",
        intent: "Step-based campaign creation flow from objective and audience through channels and approval mode.",
        primaryMetric: "5-step setup",
        secondaryMetric: "Draft-first launch",
        keyRegions: ["Objective stepper", "audience fields", "platform selection", "approval policy"],
        surfaceKind: "wizard",
      },
      {
        id: "advanced-campaign-editor",
        title: "Advanced Campaign Editor",
        group: "Planning",
        intent: "Dense editor for strategy, cadence, creative constraints, and automation policy overrides.",
        primaryMetric: "12 editable controls",
        secondaryMetric: "Manual confirmation required",
        keyRegions: ["Strategy canvas", "cadence controls", "content rules", "automation limits"],
        surfaceKind: "editor",
      },
      {
        id: "content-slot-calendar",
        title: "Content Slot Calendar",
        group: "Production",
        intent: "Calendar planning board for daily slots, platform windows, collisions, and held publish intents.",
        primaryMetric: "18 planned slots",
        secondaryMetric: "0 live publishes",
        keyRegions: ["Week grid", "platform chips", "collision panel", "slot inspector"],
        surfaceKind: "calendar",
      },
      {
        id: "todays-content-pipeline",
        title: "Today's Content Pipeline",
        group: "Production",
        intent: "Kanban-style pipeline for today’s planned content from brief through draft, review, adapt, schedule.",
        primaryMetric: "7 pieces today",
        secondaryMetric: "2 need review",
        keyRegions: ["Pipeline columns", "piece cards", "review badges", "delivery readiness"],
        surfaceKind: "pipeline",
      },
      {
        id: "content-piece-detail-adaptations",
        title: "Content Piece Detail & Adaptations",
        group: "Production",
        intent: "Single content-piece detail with core draft, platform adaptations, media references, and schedule gates.",
        primaryMetric: "4 adaptations",
        secondaryMetric: "1 blocked gate",
        keyRegions: ["Draft preview", "platform tabs", "media evidence", "gate summary"],
        surfaceKind: "detail",
      },
      {
        id: "content-editor-override-flow",
        title: "Content Editor / Override Flow",
        group: "Production",
        intent: "Focused editor for human edits, reasoned overrides, and version comparison before approval.",
        primaryMetric: "3 override reasons",
        secondaryMetric: "Version diff visible",
        keyRegions: ["Editor body", "override drawer", "diff rail", "save as draft controls"],
        surfaceKind: "override",
      },
      {
        id: "approval-needed-queue",
        title: "Approval-Needed Queue",
        group: "Review",
        intent: "Review queue collecting low-confidence drafts, probation campaigns, and pieces requiring specialist approval.",
        primaryMetric: "5 approvals waiting",
        secondaryMetric: "Specialist review mode",
        keyRegions: ["Approval list", "risk reasons", "review actions", "decision notes"],
        surfaceKind: "queue",
      },
      {
        id: "dry-test-report-mavoid-daily",
        title: "Dry Test Report - MaVoid Daily",
        group: "Review",
        intent: "Preflight report showing what would be generated, scheduled, and blocked before a run is allowed.",
        primaryMetric: "9 checks passed",
        secondaryMetric: "2 blocked externally",
        keyRegions: ["Check summary", "generated outputs", "blocked writes", "operator signoff"],
        surfaceKind: "report",
      },
      {
        id: "run-now-modal",
        title: "Run Now Modal",
        group: "Operations",
        intent: "Run confirmation modal for immediate dry-run or controlled execution with scope and safety checks.",
        primaryMetric: "Scoped run",
        secondaryMetric: "No surprise publishing",
        keyRegions: ["Run scope", "safety checklist", "affected pieces", "confirmation CTA"],
        surfaceKind: "modal",
      },
      {
        id: "recovery-failure-center",
        title: "Recovery / Failure Center",
        group: "Operations",
        intent: "Failure triage center for blocked provider actions, stale runs, retry options, and recovery notes.",
        primaryMetric: "3 recoverable issues",
        secondaryMetric: "Fail-closed state",
        keyRegions: ["Failure ledger", "retry policy", "provider diagnostics", "recovery timeline"],
        surfaceKind: "recovery",
      },
      {
        id: "omnisocials-account-mappings",
        title: "OmniSocials & Account Mappings",
        group: "Integrations",
        intent: "Integration/account mapping panel for platforms, credentials, brands, and safe unavailable states.",
        primaryMetric: "8 platform mappings",
        secondaryMetric: "Credentials server-side",
        keyRegions: ["Platform table", "brand mappings", "credential state", "fail-closed copy"],
        surfaceKind: "mapping",
      },
      {
        id: "evidence-artifact-library",
        title: "Evidence & Artifact Library",
        group: "Integrations",
        intent: "Library of briefs, captures, media, review notes, run logs, and proof artifacts linked to content pieces.",
        primaryMetric: "24 artifacts",
        secondaryMetric: "Linked to pieces",
        keyRegions: ["Artifact grid", "filters", "preview inspector", "link history"],
        surfaceKind: "library",
      },
      {
        id: "agent-execution-notifications",
        title: "Agent Execution & Notifications",
        group: "Operations",
        intent: "Agent execution monitor for planner, research, copy, designer, reviewer, publisher, verifier, and recovery profiles.",
        primaryMetric: "8 agent profiles",
        secondaryMetric: "Notifications routed",
        keyRegions: ["Agent timeline", "profile statuses", "notification rules", "handoff state"],
        surfaceKind: "agents",
      },
      {
        id: "campaign-automation-mirror",
        title: "Campaign Automation Mirror",
        group: "Operations",
        intent: "Read-only mirror of automations that replaced Hermes cron, showing schedule rhythm, last run, and next run.",
        primaryMetric: "11 mirrored automations",
        secondaryMetric: "Zoid-owned execution",
        keyRegions: ["Automation list", "schedule rhythm", "last output", "manual controls"],
        surfaceKind: "mirror",
      },
    ],
  };
}


export type ContentFlowScreenId =
  | "dashboard"
  | "brand-management"
  | "new-campaign"
  | "campaign-editor"
  | "slot-calendar"
  | "today-pipeline"
  | "piece-detail"
  | "piece-editor"
  | "approval-queue"
  | "dry-test-report"
  | "run-now-modal"
  | "recovery-center"
  | "omnisocials-mappings"
  | "evidence-library"
  | "agent-execution"
  | "automation-mirror";

export type ContentFlowSection = "Dashboard" | "Campaigns" | "Calendar" | "Pipeline" | "Approvals" | "Runs" | "Library" | "Settings";

export type ContentFlowScreen = {
  id: ContentFlowScreenId;
  stitchTitle: string;
  section: ContentFlowSection;
  type: "primary" | "settings" | "wizard" | "editor" | "detail" | "queue" | "report" | "modal" | "operations" | "library" | "mirror";
  renderMode: "primary-screen" | "modal-overlay";
  routeLabel: string;
  purpose: string;
  entryPoints: string[];
  outgoing: ContentFlowScreenId[];
};

export type ContentWorkspaceFlow = {
  defaultScreen: ContentFlowScreenId;
  sections: ContentFlowSection[];
  screens: ContentFlowScreen[];
  allScreensVisibleAtOnce: false;
};

export function buildContentWorkspaceFlow(): ContentWorkspaceFlow {
  const sections: ContentFlowSection[] = ["Dashboard", "Campaigns", "Calendar", "Pipeline", "Approvals", "Runs", "Library", "Settings"];
  const screens: ContentFlowScreen[] = [
    { id: "dashboard", stitchTitle: "Autonomous Campaign Dashboard", section: "Dashboard", type: "primary", renderMode: "primary-screen", routeLabel: "Content > Dashboard", purpose: "Command center for active campaigns, confidence, today status, risk queue, and next run.", entryPoints: ["Content sidebar", "Breadcrumb back"], outgoing: ["campaign-editor", "new-campaign", "today-pipeline", "approval-queue", "recovery-center", "run-now-modal"] },
    { id: "brand-management", stitchTitle: "Brand Management - MaVoid", section: "Settings", type: "settings", renderMode: "primary-screen", routeLabel: "Content > Settings > Brands > MaVoid", purpose: "Manage voice, pillars, allowed claims, account defaults, markets, and platform constraints.", entryPoints: ["Settings", "Wizard brand step", "Campaign editor constraints"], outgoing: ["omnisocials-mappings", "campaign-editor", "new-campaign"] },
    { id: "new-campaign", stitchTitle: "New Campaign Wizard", section: "Campaigns", type: "wizard", renderMode: "primary-screen", routeLabel: "Content > Campaigns > New", purpose: "Step-based campaign creation from goal and audience through dry-test before activation.", entryPoints: ["Dashboard New Campaign", "Campaigns Create"], outgoing: ["brand-management", "campaign-editor", "dry-test-report", "dashboard"] },
    { id: "campaign-editor", stitchTitle: "Advanced Campaign Editor", section: "Campaigns", type: "editor", renderMode: "primary-screen", routeLabel: "Content > Campaigns > MaVoid Daily > Editor", purpose: "Edit strategy, cadence, templates, agent assignments, adaptation rules, and approval thresholds.", entryPoints: ["Dashboard active campaign", "Wizard completion", "Automation mirror"], outgoing: ["slot-calendar", "today-pipeline", "dry-test-report", "brand-management", "automation-mirror"] },
    { id: "slot-calendar", stitchTitle: "Content Slot Calendar", section: "Calendar", type: "primary", renderMode: "primary-screen", routeLabel: "Content > Calendar", purpose: "Calendar grid for slots by day, platform, campaign, approval state, and conflicts.", entryPoints: ["Calendar tab", "Campaign editor cadence", "Dashboard next slot"], outgoing: ["piece-detail", "new-campaign", "approval-queue", "recovery-center"] },
    { id: "today-pipeline", stitchTitle: "Today's Content Pipeline", section: "Pipeline", type: "primary", renderMode: "primary-screen", routeLabel: "Content > Pipeline > Today", purpose: "Production board from brief to draft, design, review, adaptation, scheduled, and blocked.", entryPoints: ["Dashboard summary", "Pipeline tab", "Calendar day"], outgoing: ["piece-detail", "approval-queue", "recovery-center"] },
    { id: "piece-detail", stitchTitle: "Content Piece Detail & Adaptations", section: "Pipeline", type: "detail", renderMode: "primary-screen", routeLabel: "Content > Pieces > Daily LinkedIn Launch", purpose: "Inspect one piece, source brief, platform adaptations, artifacts, run history, and approvals.", entryPoints: ["Pipeline card", "Calendar slot", "Evidence link", "Approval item"], outgoing: ["piece-editor", "approval-queue", "evidence-library", "run-now-modal"] },
    { id: "piece-editor", stitchTitle: "Content Editor / Override Flow", section: "Pipeline", type: "editor", renderMode: "primary-screen", routeLabel: "Content > Pieces > Daily LinkedIn Launch > Edit", purpose: "Edit copy, override AI output, compare versions, adapt per platform, and record manual reason.", entryPoints: ["Piece detail edit", "Approval override", "Pipeline quick action"], outgoing: ["piece-detail", "approval-queue"] },
    { id: "approval-queue", stitchTitle: "Approval-Needed Queue", section: "Approvals", type: "queue", renderMode: "primary-screen", routeLabel: "Content > Approvals", purpose: "Review low-confidence, risky, blocked, or special-category content decisions.", entryPoints: ["Dashboard risk queue", "Pipeline blocked", "Detail approval banner", "Dry test failure"], outgoing: ["piece-detail", "piece-editor", "today-pipeline", "dashboard"] },
    { id: "dry-test-report", stitchTitle: "Dry Test Report - MaVoid Daily", section: "Runs", type: "report", renderMode: "primary-screen", routeLabel: "Content > Runs > Dry Test > MaVoid Daily", purpose: "Preview generated outputs, scheduling decisions, checks, and blocked external writes before activation.", entryPoints: ["Wizard final step", "Campaign editor dry test", "Run modal dry-run"], outgoing: ["campaign-editor", "approval-queue", "piece-detail", "agent-execution"] },
    { id: "run-now-modal", stitchTitle: "Run Now Modal", section: "Runs", type: "modal", renderMode: "modal-overlay", routeLabel: "Content > Runs > Run Now Modal", purpose: "Confirm controlled run or dry-run scope before automation starts.", entryPoints: ["Dashboard next run", "Campaign editor run", "Piece detail schedule", "Recovery retry"], outgoing: ["dry-test-report", "agent-execution", "dashboard"] },
    { id: "recovery-center", stitchTitle: "Recovery / Failure Center", section: "Runs", type: "operations", renderMode: "primary-screen", routeLabel: "Content > Runs > Recovery", purpose: "Diagnose failed runs, blocked integrations, agent failures, missing approvals, and invalid mappings.", entryPoints: ["Dashboard failure", "Agent failure", "Dry test failure", "Automation mirror failure"], outgoing: ["run-now-modal", "omnisocials-mappings", "piece-editor", "evidence-library"] },
    { id: "omnisocials-mappings", stitchTitle: "OmniSocials & Account Mappings", section: "Settings", type: "settings", renderMode: "primary-screen", routeLabel: "Content > Settings > OmniSocials", purpose: "Map brands/campaigns/platforms to social accounts and show fail-closed integration status.", entryPoints: ["Settings", "Brand defaults", "Recovery integration failure", "Campaign warning"], outgoing: ["brand-management", "campaign-editor", "dashboard"] },
    { id: "evidence-library", stitchTitle: "Evidence & Artifact Library", section: "Library", type: "library", renderMode: "primary-screen", routeLabel: "Content > Library", purpose: "Search generated drafts, design captures, run logs, review decisions, dry-test evidence, and artifacts.", entryPoints: ["Library tab", "Piece artifacts", "Dry test outputs", "Recovery logs"], outgoing: ["piece-detail", "dry-test-report", "agent-execution"] },
    { id: "agent-execution", stitchTitle: "Agent Execution & Notifications", section: "Runs", type: "operations", renderMode: "primary-screen", routeLabel: "Content > Runs > MaVoid Daily > Execution", purpose: "Monitor planner, research, copy, designer, reviewer, publisher, verifier, and recovery agent steps.", entryPoints: ["Run modal confirm", "Dashboard active run", "Automation history", "Recovery failed run"], outgoing: ["recovery-center", "piece-detail", "evidence-library", "automation-mirror"] },
    { id: "automation-mirror", stitchTitle: "Campaign Automation Mirror", section: "Runs", type: "mirror", renderMode: "primary-screen", routeLabel: "Content > Runs > Automation Mirror", purpose: "Mirror recurring content automation schedules, last run, next run, manual override, and probation status.", entryPoints: ["Dashboard automation", "Campaign editor automation", "Automations workspace mirror"], outgoing: ["campaign-editor", "agent-execution", "recovery-center", "run-now-modal"] },
  ];
  return { defaultScreen: "dashboard", sections, screens, allScreensVisibleAtOnce: false };
}

export type ContentWorkspaceRefinementChecklistItem = {
  screenId: ContentFlowScreenId;
  pageFeelsLike: "finished-product-screen";
  visibleRegions: string[];
  primaryAction: string;
  visualPolishNotes: string[];
};

export function buildContentWorkspaceRefinementChecklist(): ContentWorkspaceRefinementChecklistItem[] {
  return [
    { screenId: "dashboard", pageFeelsLike: "finished-product-screen", visibleRegions: ["executive hero", "confidence metrics", "run timeline", "risk queue", "campaign shortcuts"], primaryAction: "Open controlled run", visualPolishNotes: ["compact hero keeps active screen above fold", "cards use varied scale instead of equal placeholders"] },
    { screenId: "brand-management", pageFeelsLike: "finished-product-screen", visibleRegions: ["brand identity", "voice rules", "pillar matrix", "market defaults", "account mapping link"], primaryAction: "Update brand constraints", visualPolishNotes: ["rules are grouped by decision area", "brand tone appears as usable product controls"] },
    { screenId: "new-campaign", pageFeelsLike: "finished-product-screen", visibleRegions: ["wizard progress", "objective card", "audience/platform fields", "approval policy", "dry-test checkpoint"], primaryAction: "Continue campaign setup", visualPolishNotes: ["step cards show current setup path", "final dry-run checkpoint is visually distinct"] },
    { screenId: "campaign-editor", pageFeelsLike: "finished-product-screen", visibleRegions: ["strategy sidebar", "editable policy canvas", "cadence rail", "automation limits", "dry-run actions"], primaryAction: "Run dry test", visualPolishNotes: ["editor feels dense and operational", "inspector separates safety from strategy"] },
    { screenId: "slot-calendar", pageFeelsLike: "finished-product-screen", visibleRegions: ["week calendar", "platform bands", "collision states", "slot inspector", "approval warnings"], primaryAction: "Open selected slot", visualPolishNotes: ["calendar uses large tappable day blocks", "blocked slots have readable but restrained emphasis"] },
    { screenId: "today-pipeline", pageFeelsLike: "finished-product-screen", visibleRegions: ["kanban columns", "piece cards", "gate badges", "delivery readiness", "blocked lane"], primaryAction: "Open content piece", visualPolishNotes: ["pipeline columns have varied statuses", "blocked/review lanes do not dominate the page"] },
    { screenId: "piece-detail", pageFeelsLike: "finished-product-screen", visibleRegions: ["draft preview", "platform adaptations", "artifact evidence", "review gate summary", "schedule controls"], primaryAction: "Edit or override draft", visualPolishNotes: ["detail page reads as one selected object", "evidence and actions stay in the inspector"] },
    { screenId: "piece-editor", pageFeelsLike: "finished-product-screen", visibleRegions: ["editor canvas", "version diff", "override reason", "platform tabs", "review submission"], primaryAction: "Save override draft", visualPolishNotes: ["editing surface gets the largest visual weight", "diff rail makes the override workflow explicit"] },
    { screenId: "approval-queue", pageFeelsLike: "finished-product-screen", visibleRegions: ["approval backlog", "risk reasons", "decision controls", "review notes", "queue filters"], primaryAction: "Open review item", visualPolishNotes: ["queue rows show why action is needed", "decision buttons are secondary to item context"] },
    { screenId: "dry-test-report", pageFeelsLike: "finished-product-screen", visibleRegions: ["check summary", "generated output preview", "blocked writes", "operator signoff", "follow-up routes"], primaryAction: "Resolve dry-test issues", visualPolishNotes: ["report reads like a preflight artifact", "blocked writes are clear without implying failure panic"] },
    { screenId: "run-now-modal", pageFeelsLike: "finished-product-screen", visibleRegions: ["modal title", "scope checklist", "safety guardrails", "affected pieces", "confirmation actions"], primaryAction: "Confirm dry run", visualPolishNotes: ["modal overlays without replacing the current screen", "copy stays controlled-run and fail-closed"] },
    { screenId: "recovery-center", pageFeelsLike: "finished-product-screen", visibleRegions: ["failure ledger", "provider diagnostics", "retry policy", "recovery timeline", "evidence link"], primaryAction: "Retry controlled run", visualPolishNotes: ["recovery reasons are triaged by urgency", "integration failure path is visibly fail-closed"] },
    { screenId: "omnisocials-mappings", pageFeelsLike: "finished-product-screen", visibleRegions: ["platform table", "brand mappings", "credential state", "server-side warning", "safe fallback actions"], primaryAction: "Review account mapping", visualPolishNotes: ["integration table is readable and conservative", "credentials are never implied to be exposed"] },
    { screenId: "evidence-library", pageFeelsLike: "finished-product-screen", visibleRegions: ["artifact grid", "filters", "preview inspector", "link history", "run evidence"], primaryAction: "Open linked artifact", visualPolishNotes: ["artifacts have distinct types and hierarchy", "preview area prevents the grid from feeling generic"] },
    { screenId: "agent-execution", pageFeelsLike: "finished-product-screen", visibleRegions: ["agent timeline", "profile statuses", "notification rules", "handoff states", "failure links"], primaryAction: "Inspect agent run", visualPolishNotes: ["agent list reads as a sequence", "publisher step is visibly disabled/fail-closed"] },
    { screenId: "automation-mirror", pageFeelsLike: "finished-product-screen", visibleRegions: ["automation list", "schedule rhythm", "last output", "manual controls", "probation state"], primaryAction: "Open automation run", visualPolishNotes: ["mirror looks read-only but actionable", "manual controls are scoped and safe"] },
  ];
}
