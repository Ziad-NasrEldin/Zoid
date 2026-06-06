export type Phase6IntegrationState = { key: string; state: string; safe_copy: string };
export type InboxAggregateRecord = { id: string; item_type: string; title: string; detail: string; state: string; priority: string; route?: string | null };
export type CalendarRefRecord = { id: string; title: string; starts_at: string; ends_at: string; location?: string | null; notes?: string | null; state: string; confirmation_id?: string | null };
export type EmailRefRecord = { id: string; subject: string; recipients_json: string; snippet?: string | null; state: string; confirmation_id?: string | null };
export type BusinessCompanyRecord = { id: string; name: string; domain?: string | null; status: string; notes?: string | null };
export type BusinessContactRecord = { id: string; company_id?: string | null; full_name: string; email?: string | null; role?: string | null; status: string };
export type FollowUpRecord = { id: string; subject: string; due_at?: string | null; state: string; priority: string; contact_id?: string | null; company_id?: string | null; product_id?: string | null };
export type ProductRecord = { id: string; name: string; status: string; summary?: string | null; owner_contact_id?: string | null };
export type EntityLinkRecord = { id: string; source_type: string; source_id: string; target_type: string; target_id: string; relation_type: string };

export type Phase6OverviewRecord = {
  integrations: Phase6IntegrationState[];
  inbox: InboxAggregateRecord[];
  calendar: CalendarRefRecord[];
  emails: EmailRefRecord[];
  companies: BusinessCompanyRecord[];
  contacts: BusinessContactRecord[];
  follow_ups: FollowUpRecord[];
  products: ProductRecord[];
  product_links: EntityLinkRecord[];
};

export type Phase6State =
  | { mode: "loading" }
  | { mode: "error"; error: string }
  | { mode: "ready"; overview: Phase6OverviewRecord };

export type Phase6Invoke = <T>(command: string, args?: Record<string, unknown>) => Promise<T>;

export type Phase6WorkspaceView = {
  title: string;
  heroCopy: string;
  statusCopy: string;
  sections: { key: string; title: string; count: number; emptyCopy: string }[];
  blockers: string[];
};

export const phase6EmptyOverview: Phase6OverviewRecord = {
  integrations: [
    { key: "eventkit", state: "needs_permission", safe_copy: "Calendar access needs macOS permission before sync can run." },
    { key: "gmail", state: "not_configured", safe_copy: "Mail is unconfigured; send remains confirmation-gated." },
  ],
  inbox: [],
  calendar: [],
  emails: [],
  companies: [],
  contacts: [],
  follow_ups: [],
  products: [],
  product_links: [],
};

export function buildPhase6WorkspaceView(workspaceId: string, state: Phase6State): Phase6WorkspaceView {
  const overview = state.mode === "ready" ? state.overview : phase6EmptyOverview;
  const titleByWorkspace: Record<string, string> = {
    inbox: "Inbox",
    calendar: "Calendar",
    business: "Business",
    products: "Products",
  };
  const title = titleByWorkspace[workspaceId] ?? "Phase 6";
  const blockers = overview.integrations
    .filter((integration) => integration.state !== "connected")
    .map((integration) => integration.safe_copy);
  return {
    title,
    heroCopy: "Phase 6 connects attention, calendar, mail, relationships, and product context through native commands and persisted local records.",
    statusCopy: state.mode === "error" ? state.error : state.mode === "loading" ? "Loading native Phase 6 data…" : "Native Phase 6 data loaded. Consequential sends and calendar writes require confirmation.",
    blockers,
    sections: [
      { key: "inbox", title: "Attention inbox", count: overview.inbox.length, emptyCopy: "No attention items yet." },
      { key: "calendar", title: "Calendar events", count: overview.calendar.length, emptyCopy: "No confirmed local calendar events yet." },
      { key: "emails", title: "Mail drafts/search", count: overview.emails.length, emptyCopy: "No mail records; Gmail is safe while unconfigured." },
      { key: "business", title: "Companies / contacts / follow-ups", count: overview.companies.length + overview.contacts.length + overview.follow_ups.length, emptyCopy: "No business records yet." },
      { key: "products", title: "Products and links", count: overview.products.length + overview.product_links.length, emptyCopy: "No products or cross-links yet." },
    ],
  };
}

export function phase6SafeBridgeError(error: unknown) {
  const detail = error instanceof Error ? error.message : String(error);
  if (/^(confirmation_id|email_id|event_id) is required$/.test(detail)) return detail;
  if (/invoke|__TAURI|Tauri|Cannot read properties of undefined/i.test(detail)) {
    return "Phase 6 native data is only available inside the Tauri desktop app. Browser preview stays UI-only and does not simulate records.";
  }
  return "Phase 6 native bridge is unavailable. No browser preview or fallback records are simulated.";
}

export async function loadPhase6OverviewFromBridge(invoke: Phase6Invoke): Promise<Phase6State> {
  try {
    const overview = await invoke<Phase6OverviewRecord>("get_phase6_overview_command");
    return { mode: "ready", overview };
  } catch (error) {
    return { mode: "error", error: phase6SafeBridgeError(error) };
  }
}

export function assertPhase6NoSilentSend(command: string, args?: Record<string, unknown>) {
  if (command === "send_email_draft_command" && !args?.request) return false;
  return command !== "send_email_draft_command" || Boolean((args?.request as { confirmation_id?: string } | undefined)?.confirmation_id);
}

export function assertPhase6CalendarConfirmation(command: string, args?: Record<string, unknown>) {
  const calendarCommands = new Set(["create_calendar_event_command", "update_calendar_event_command", "delete_calendar_event_command"]);
  if (!calendarCommands.has(command)) return true;
  return Boolean((args?.request as { confirmation_id?: string } | undefined)?.confirmation_id);
}
