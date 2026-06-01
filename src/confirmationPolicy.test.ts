import {
  buildConfirmationPolicyView,
  type ConfirmationPolicyDecision,
  type ConfirmationPolicyMode,
} from "./confirmationPolicy";

const gatedPolicy: ConfirmationPolicyDecision = {
  category: "external_send",
  policy: "require_review",
  reviewer_required: "yes",
  human_confirmation: "always",
  reason: "External email sends can affect people outside the local workspace.",
  allowed_now: false,
  requires_confirmation: true,
  requires_reviewer: true,
  requires_clear_task: false,
};

const clearTaskPolicy: ConfirmationPolicyDecision = {
  category: "code_change",
  policy: "require_clear_task",
  reviewer_required: "maybe",
  human_confirmation: "yes",
  reason: "Code edits need an explicit task scope before they can proceed.",
  allowed_now: false,
  requires_confirmation: true,
  requires_reviewer: false,
  requires_clear_task: true,
};

const allowedPolicy: ConfirmationPolicyDecision = {
  category: "local_read",
  policy: "allow",
  reviewer_required: "none",
  human_confirmation: "none",
  reason: "Local read-only inspection is allowed without extra gate.",
  allowed_now: true,
  requires_confirmation: false,
  requires_reviewer: false,
  requires_clear_task: false,
};

const gatedView = buildConfirmationPolicyView({ mode: "native", policy: gatedPolicy });

if (gatedView.mode !== "native" || gatedView.sourceLabel !== "Native sample policy") {
  throw new Error("native confirmation view must identify the real native sample policy source");
}

if (gatedView.overallStatus !== "Review required" || gatedView.tone !== "blocked") {
  throw new Error("native confirmation view must summarize reviewer-gated policy as review required");
}

if (gatedView.reason !== gatedPolicy.reason) {
  throw new Error("native confirmation view must preserve the backend policy reason exactly");
}

if (!gatedView.requirements.some((item) => item.label === "Human confirmation" && item.status === "Required" && item.tone === "blocked")) {
  throw new Error("native confirmation view must show required human confirmation status");
}

if (!gatedView.requirements.some((item) => item.label === "Reviewer" && item.status === "Required" && item.detail === "yes")) {
  throw new Error("native confirmation view must show required reviewer status and raw requirement detail");
}

if (!gatedView.requirements.some((item) => item.label === "Clear task" && item.status === "Not required" && item.tone === "ready")) {
  throw new Error("native confirmation view must show clear-task status even when not required");
}

const clearTaskView = buildConfirmationPolicyView({ mode: "native", policy: clearTaskPolicy });
if (clearTaskView.overallStatus !== "Clear task required") {
  throw new Error("clear-task policy must be surfaced as clear task required");
}
if (!clearTaskView.requirements.some((item) => item.label === "Clear task" && item.status === "Required" && item.tone === "blocked")) {
  throw new Error("clear-task policy must expose required clear-task gate");
}

const allowedView = buildConfirmationPolicyView({ mode: "native", policy: allowedPolicy });
if (allowedView.overallStatus !== "Allowed now" || allowedView.tone !== "ready") {
  throw new Error("allowed native policy must be labeled allowed now without implying an action was run");
}
if (!allowedView.emptyActionCopy.includes("Read-only preview") || /approval|confirmation id|ready to run/i.test(allowedView.emptyActionCopy)) {
  throw new Error("confirmation primitive must not fabricate approval IDs or ready-to-run action state");
}

for (const mode of ["checking", "preview"] satisfies ConfirmationPolicyMode[]) {
  const view = buildConfirmationPolicyView({ mode, policy: null });
  const rendered = [
    view.sourceLabel,
    view.overallStatus,
    view.summary,
    view.reason,
    view.emptyActionCopy,
    ...view.requirements.map((item) => `${item.label}:${item.status}:${item.detail}`),
  ].join("\n");

  if (!/Native-only|Checking|Browser preview|unavailable outside Tauri/i.test(rendered)) {
    throw new Error("non-native confirmation view must truthfully label native-only policy data");
  }

  if (/Allowed now|Review required|approval|confirmation id|ready to run/i.test(rendered)) {
    throw new Error("non-native confirmation view must not fake policy decisions, approvals, IDs, or runnable state");
  }
}
