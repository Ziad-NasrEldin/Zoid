# Automations Cron Control Plane Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Feature completion requires `feature-critique-workflow`: create `.hermes/reviews/automations-cron-control-plane/handoff.md`, run the critique agent, fix Required issues, and re-review until APPROVED.

**Goal:** Turn the empty Automations page in Zoid 25 into a Hermes automation control plane that lists live Hermes cron jobs and watchers, shows run/failure state, and supports safe management actions.

**Architecture:** Add Automations as a first-class workspace in the React shell. The Tauri backend should read Hermes cron/watchers state through the local Hermes CLI / Hermes data, expose typed read/manage commands to the frontend, and never rebuild the scheduler or watcher engine inside Zoid. Zoid is the UI/control plane; Hermes remains the scheduler/provider of truth.

**Tech Stack:** React + TypeScript + Vite frontend, Tauri v2 Rust backend, existing Hermes CLI integration, existing Zoid Kujo/editorial CSS system.

---

## Current Context / Assumptions

- Repo: `/Users/ziadnasreldin/Zoid`
- Resolved domain boundary: “Automations” means Hermes-managed cron jobs and watchers only. It does not mean arbitrary background processes, local scripts outside Hermes, agent chats, repositories, or future non-Hermes routines.
- Resolved watcher boundary: V1 includes a Watchers section, but only populates it from Hermes-managed, inspectable watcher sources. If Hermes cannot provide watcher read-back yet, the UI must show a truthful “No watchers found / watcher source unavailable” state instead of inventing watcher status.
- Resolved layout boundary: Cron Jobs and Watchers must be separate sections/tabs, not one combined mixed list, because cron jobs have schedule/next-run/repeat fields while watchers have live monitoring/state fields.
- Resolved action confirmation boundary: Require branded in-app confirmation for Remove and Run now. Pause/Resume stay one-click because they are reversible, but they must be clearly labeled and verified through Hermes read-back.
- Resolved v1 edit boundary: V1 is operational controls only: list, refresh, pause, resume, run now, and remove. Do not add schedule/prompt editing or cron creation in V1.
- Resolved profile boundary: V1 manages only the active/default Hermes profile. Show a read-only `Profile: default`/active-profile label, but do not include profile switching or cross-profile management.
- Resolved prompt visibility boundary: V1 shows only job name, script, skills/toolsets, and prompt preview metadata. Do not display full cron prompt text in the Automations page.
- Resolved failure detail boundary: V1 shows `lastStatus` and `lastDeliveryError` only. Do not expose raw run output/logs unless Hermes provides a safe redacted summary field.
- Resolved removal history boundary: After confirmed remove, the job disappears only after verified Hermes read-back. Do not create a local “recently removed” audit/history log in V1.
- Resolved refresh boundary: V1 uses manual refresh plus automatic refresh after management actions only. Do not add interval polling in V1.
- Resolved nav health boundary: Automations nav status should derive from the latest loaded Hermes state: `ready` when jobs/watchers exist with no failures, `error/attention` when any job/watcher failed, `empty` when no cron jobs/watchers are found, and `offline` when Hermes CLI is unavailable. Do not persist this as local truth.
- Resolved protected job boundary: V1 should protect likely system/internal jobs from destructive Remove by default. Jobs with name/script markers such as `feature-critique-watchdog`, `session archive`, `watchdog`, or other internal automation markers should show Remove disabled/protected. Pause/Resume remain available with read-back; Run now still requires confirmation.
- Resolved protected enforcement boundary: Protected-job Remove blocking must be enforced in both frontend UI and Rust backend command layer. Frontend disables the action; Rust rejects direct `remove` invokes for protected jobs.
- Resolved watcher action boundary: Watchers are read-only in V1 unless Hermes exposes an explicit watcher control API/source of truth. Cron jobs get operational controls; watcher cards only show inspectable state/status/detail.
- Resolved protected marker boundary: Protected-job detection should be backend-owned in Rust with a small V1 marker list, mirrored to the frontend as `protected: true` plus `protectionReason`. Do not expose marker editing/config UI in V1.
- Resolved protected Run now boundary: `Run now` remains allowed for protected jobs, but must use branded confirmation. Protected means “do not delete system/internal automation,” not “never run it.”
- Resolved nav-load boundary: In V1, Automations nav health is computed only after the user opens or manually refreshes Automations. Before first load, keep the nav status neutral/empty to avoid background Hermes CLI calls.
- Planning interaction boundary: Only ask critical product/safety/implementation questions. For non-critical details, use the plan recommendations and document them as defaults.
- Current active workspace type is only `"Agents" | "Code" | "Settings"` in `src/App.tsx`.
- The sidebar already renders an `Automations` nav item but clicking it does nothing because only Agents/Code/Settings are routed.
- Current live Hermes cron list from this session has 4 jobs:
  - `feature-critique-watchdog` — every 2m — last status ok — script-only
  - `Obsidian Hermes Session Archive` — every 120m — last status ok — script-only
  - `MaVoid Daily Social Creator Scheduler` — `0 8 * * *` — last status error
  - `MaVoid OmniSocials Publish Monitor` — every 15m — last status ok — script-only
- Existing backend already has Hermes CLI discovery helpers in `src-tauri/src/lib.rs`: `candidate_hermes_paths`, `find_hermes_cli`, `run_command_with_timeout`.
- Existing frontend invokes Tauri commands through small client wrappers, e.g. `src/code/repositoryClient.ts`.
- Desired behavior should distinguish local UI intent from provider truth: a job is scheduled/paused/failed only if Hermes read-back says so.

## Proposed Approach

Implement the feature in three layers:

1. Backend automation adapter in Rust:
   - Discover Hermes CLI using existing helpers.
   - List cron jobs by calling Hermes cron read-only output first, with a robust fallback if CLI JSON is unavailable.
   - List watchers from Hermes-managed watcher sources if available; otherwise expose an empty watcher section with a clear “No watchers found” state until Hermes provides read-back.
   - Expose management commands for pause/resume/run/remove by job id for cron jobs, and equivalent safe actions for watchers only when Hermes exposes them.
   - Always re-list after a management action to verify provider truth.

2. Frontend data client and workspace:
   - Create `src/automations/automationClient.ts` with typed invoke wrappers.
   - Create `src/automations/AutomationsWorkspace.tsx` with loading/error/empty/list/detail states.
   - Add refresh, search/filter, status badges, and safe action buttons.

3. Shell integration and styling:
   - Add `Automations` to `ActiveWorkspace`.
   - Make sidebar/rail buttons route to Automations.
   - Change Automations nav status from `idle/empty` to `ready` when implemented.
   - Add CSS in `src/App.css` consistent with the existing Code/Settings panels.

## UX Requirements

- Header: “Automations” / “Hermes automation control plane”.
- Primary workspace layout:
  - Separate `Cron Jobs` and `Watchers` sections/tabs.
  - Do not merge cron jobs and watchers into a single mixed card list.
  - Cron cards prioritize schedule, next run, repeat, last run, and delivery.
  - Watcher cards prioritize monitoring state, source, last seen, last status, and detail.
- Summary strip/cards:
  - total jobs
  - enabled jobs
  - paused jobs
  - failed/error jobs
  - next run soonest
- Job list cards show:
  - name and job id
  - schedule
  - enabled / paused / scheduled state
  - last run time
  - next run time
  - last status: ok/error/never/unknown
  - delivery target
  - script / no-agent marker
  - skills/model/toolsets when present
  - last delivery error if present
- Filters:
  - All
  - Running/enabled
  - Paused/disabled
  - Failed
  - Script-only
- Actions per cron job:
  - Refresh all jobs
  - Pause enabled job without confirmation, then verify through Hermes read-back
  - Resume paused/disabled job without confirmation, then verify through Hermes read-back
  - Run now behind branded in-app confirmation, because it can trigger external side effects
  - Remove job behind branded in-app confirmation, not native `confirm()`.
- Empty state:
  - If Hermes is connected but no jobs exist: “No Hermes cron jobs found.”
  - If Hermes is missing: show the existing-style Hermes offline message and suggest setting `ZOID_HERMES_CLI`.
- Do not expose raw secrets, full prompts, or credential-like env values. Use prompt preview only.

## Files Likely To Change

- Modify: `/Users/ziadnasreldin/Zoid/src/App.tsx`
- Modify: `/Users/ziadnasreldin/Zoid/src/App.css`
- Create: `/Users/ziadnasreldin/Zoid/src/automations/AutomationsWorkspace.tsx`
- Create: `/Users/ziadnasreldin/Zoid/src/automations/automationClient.ts`
- Create: `/Users/ziadnasreldin/Zoid/src/automations/types.ts`
- Modify: `/Users/ziadnasreldin/Zoid/src-tauri/src/lib.rs`
- Modify: `/Users/ziadnasreldin/Zoid/src/scaffold.test.ts`
- Optional create: `/Users/ziadnasreldin/Zoid/src/automations/automationViewModel.test.ts`
- Create during critique gate: `/Users/ziadnasreldin/Zoid/.hermes/reviews/automations-cron-control-plane/handoff.md`

## Backend Data Contract

Add Rust serializable structs in `src-tauri/src/lib.rs`:

```rust
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AutomationCronJob {
    job_id: String,
    name: String,
    schedule: String,
    repeat: String,
    deliver: String,
    next_run_at: Option<String>,
    last_run_at: Option<String>,
    last_status: Option<String>,
    last_delivery_error: Option<String>,
    enabled: bool,
    state: String,
    paused_at: Option<String>,
    paused_reason: Option<String>,
    script: Option<String>,
    no_agent: bool,
    skills: Vec<String>,
    prompt_preview: String,
    enabled_toolsets: Vec<String>,
    protected: bool,
    protection_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub struct AutomationCronList {
    jobs: Vec<AutomationCronJob>,
    count: usize,
    refreshed_at: String,
    hermes_command: Option<String>,
}
```

Frontend mirror in `src/automations/types.ts`:

```ts
export type AutomationCronJob = {
  jobId: string;
  name: string;
  schedule: string;
  repeat: string;
  deliver: string;
  nextRunAt: string | null;
  lastRunAt: string | null;
  lastStatus: string | null;
  lastDeliveryError: string | null;
  enabled: boolean;
  state: string;
  pausedAt: string | null;
  pausedReason: string | null;
  script: string | null;
  noAgent: boolean;
  skills: string[];
  promptPreview: string;
  enabledToolsets: string[];
  protected: boolean;
  protectionReason: string | null;
};

export type AutomationCronList = {
  jobs: AutomationCronJob[];
  count: number;
  refreshedAt: string;
  hermesCommand: string | null;
};

export type AutomationAction = "pause" | "resume" | "run" | "remove";
```

Extend the final list response for watchers:

```ts
export type HermesWatcher = {
  id: string;
  name: string;
  state: "running" | "paused" | "failed" | "unknown";
  source: string;
  lastSeenAt: string | null;
  lastStatus: string | null;
  detail: string | null;
};

export type AutomationList = AutomationCronList & {
  watchers: HermesWatcher[];
  watcherSourceStatus: "available" | "unavailable" | "empty";
};
```

## Step-by-Step Plan

### Task 1: Add Automations workspace routing

**Objective:** Make the existing Automations nav item open a real workspace.

**Files:**
- Modify: `src/App.tsx`

**Steps:**
1. Import `AutomationsWorkspace` from `./automations/AutomationsWorkspace`.
2. Change `type ActiveWorkspace = "Agents" | "Code" | "Settings";` to include `"Automations"`.
3. Update `isActiveWorkspace` to accept `Automations`.
4. Keep Automations nav neutral/empty before first page load. After the page loads or refreshes, derive nav status from the latest returned Hermes state.
5. Update both sidebar and rail click handlers so `Automations` is routable.
6. Add render branch before Settings:
   ```tsx
   {activeWorkspace === "Automations" ? <AutomationsWorkspace /> : ...}
   ```

**Validation:**
- Run `npm run build` after implementation.
- Manual: click Automations in wide sidebar and compact rail; page changes and persists after reload.

### Task 2: Add frontend automation types and Tauri client

**Objective:** Create typed API wrappers for listing and managing Hermes cron jobs.

**Files:**
- Create: `src/automations/types.ts`
- Create: `src/automations/automationClient.ts`

**Steps:**
1. Add the TypeScript contracts from the Backend Data Contract section.
2. Add wrappers:
   ```ts
   import { invoke } from "@tauri-apps/api/core";
   import type { AutomationAction, AutomationList } from "./types";

   export function listHermesAutomations(): Promise<AutomationList> {
     return invoke<AutomationList>("list_hermes_automations");
   }

   export function manageHermesCronJob(jobId: string, action: AutomationAction): Promise<AutomationList> {
     return invoke<AutomationList>("manage_hermes_cron_job", { jobId, action });
   }
   ```

**Validation:**
- `npm run build` should type-check imports once backend command names exist.

### Task 3: Add backend read command for Hermes automations

**Objective:** Expose `list_hermes_automations` from Tauri and return live Hermes cron and watcher state.

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Steps:**
1. Reuse `find_hermes_cli()` and `run_command_with_timeout()`.
2. Add a function that runs the Hermes CLI in read-only mode for cron jobs.
3. Preferred cron implementation:
   - Try a JSON-capable command if available in current Hermes CLI. Verify by testing locally during implementation; candidate commands to check:
     - `hermes cron list --json`
     - `hermes cron list --all --json`
   - If JSON is unavailable, fall back to reading Hermes cron storage directly only if its structure is stable and profile-safe.
4. Add watcher discovery as a separate read path:
   - Only include Hermes-managed, inspectable watcher sources.
   - Do not treat every OS process or arbitrary script as a watcher.
   - If no reliable Hermes watcher read-back exists, return `watchers: []` plus a source/status message like `watcherSourceStatus: "unavailable"`.
5. Keep profile support in scope:
   - Default profile uses current Hermes profile/home.
   - Later extension can add profile selection, but v1 should reflect the default profile that launched Zoid.
6. Normalize missing fields into safe defaults.
7. Redact/avoid full prompts. Use prompt preview only.
8. Add backend-owned protected marker detection for likely internal/system jobs and return `protected` plus `protectionReason` for each cron job.
9. Register command in `tauri::generate_handler!`.

**Validation:**
- Add Rust tests using a fake `ZOID_HERMES_CLI` script that returns sample JSON.
- Test parses jobs including ok, error, paused, script-only, and empty list.
- Run `npm run test:rust`.

### Task 4: Add backend management command with provider read-back

**Objective:** Pause/resume/run/remove Hermes cron jobs safely and return verified refreshed state.

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Steps:**
1. Add accepted action enum or string validation for only: `pause`, `resume`, `run`, `remove`.
2. Reject empty job ids.
3. Run the exact Hermes command:
   - pause: `hermes cron pause <job_id>`
   - resume: `hermes cron resume <job_id>`
   - run: `hermes cron run <job_id>`
   - remove: `hermes cron remove <job_id>`
4. Use a shorter timeout than chat, e.g. 45–90 seconds.
5. If the command exits non-zero, return stderr/stdout as an error.
6. Before `remove`, re-check protected status in Rust and reject protected jobs even if the frontend somehow invokes the command directly.
7. Allow `run` for protected jobs, but keep frontend branded confirmation.
8. After success, immediately call the same list function and return the new `AutomationList`.
9. For remove, verify the job id is absent from the refreshed list before returning success; otherwise return an error.
10. Do not create/edit cron jobs in v1 unless user later asks.

**Validation:**
- Rust tests with fake Hermes script should verify the exact args passed for pause/resume/run/remove.
- Rust test for invalid action returns error.
- Rust test for protected remove returns error and does not call `hermes cron remove`.
- Rust test for protected run is allowed and calls `hermes cron run`.
- Rust test for remove read-back still containing the job returns error.

### Task 5: Build AutomationsWorkspace UI

**Objective:** Render useful automation state with refresh/filter/action flows.

**Files:**
- Create: `src/automations/AutomationsWorkspace.tsx`

**Steps:**
1. Use `useEffect` to load jobs on mount.
2. Local state:
   - `automationList`
   - `isLoading`
   - `errorMessage`
   - `busyJobId`
   - `busyAction`
   - `filter`
   - `searchQuery`
   - `pendingRemovalJob`
3. Header:
   - kana line: `自動化`
   - title: `Hermes automations`
   - description explaining it reads live Hermes cron jobs.
   - refresh button.
4. Summary cards based on derived counts.
5. Filter/search toolbar.
6. Job list cards with status badges and metadata grid.
7. Cron job action buttons:
   - Pause if enabled and not paused.
   - Resume if disabled or paused.
   - Run now always if not busy, including protected jobs, behind branded confirmation.
   - Remove opens branded confirm panel/modal only for unprotected jobs.
   - Protected jobs show disabled Remove with the backend-provided `protectionReason`.
8. Watcher cards are read-only in V1 and show source/status/detail plus a truthful unavailable/empty state when no inspectable watcher source exists.
9. Handle errors inline at top of page and per action where possible.
10. Show last refreshed time and Hermes command path if available.

**Validation:**
- Use local test data or a fake backend during development if needed.
- `npm run build` passes.
- Manual: job cards show all 4 current live jobs and the MaVoid Daily Social Creator Scheduler is visibly failed/error.

### Task 6: Add Automations styling

**Objective:** Match existing Zoid 25 visual language without a generic dashboard look.

**Files:**
- Modify: `src/App.css`

**Steps:**
1. Reuse patterns from `.code-workspace-shell`, `.code-workspace-header`, `.repository-list-panel`, `.repository-card`.
2. Add classes:
   - `.automations-workspace-shell`
   - `.automations-workspace-header`
   - `.automation-summary-grid`
   - `.automation-summary-card`
   - `.automation-toolbar`
   - `.automation-filter-tabs`
   - `.automation-job-list`
   - `.automation-job-card`
   - `.automation-status-badge--ok/error/paused/unknown`
   - `.automation-action-row`
   - `.automation-confirm-panel`
3. Use existing CSS vars: `--kujo-ink`, `--kujo-paper`, `--kujo-blue`, `--kujo-green`, `--kujo-amber`, `--kujo-red`, `--kujo-muted`.
4. Add responsive behavior under existing `@media (max-width: 1100px)` and `@media (max-width: 820px)`.

**Validation:**
- Browser/Tauri visual check at 1920×1018 and at a narrower width.
- Confirm no overflow hidden blocks action buttons.

### Task 7: Add frontend unit/logic coverage

**Objective:** Protect filtering/status derivation from regressions.

**Files:**
- Modify: `src/scaffold.test.ts`
- Optional create: `src/automations/automationViewModel.test.ts`

**Steps:**
1. If `scaffold.test.ts` is currently simple, add no-render pure helper tests by extracting helpers from `AutomationsWorkspace`:
   - `getAutomationStatusKind(job)`
   - `filterAutomationJobs(jobs, filter, query)`
   - `summarizeAutomationJobs(jobs)`
2. Test cases:
   - ok job classified as ok.
   - `lastStatus: "error"` classified as error.
   - disabled/paused job classified as paused.
   - script/no-agent jobs filter correctly.
   - search matches name, id, schedule, script, skills.

**Validation:**
- Run `npm run test:frontend`.

### Task 8: Full local verification

**Objective:** Prove the feature works locally before review.

**Commands:**
- `npm run test:frontend`
- `npm run test:rust`
- `npm run build`
- `npm run tauri:build` if time/build environment allows.

**Manual verification:**
1. Launch/rebuild Zoid so changes are visible in-app.
2. Open Automations from sidebar.
3. Verify all live Hermes cron jobs are listed.
4. Verify failed jobs are highlighted without hiding ok jobs.
5. Click Refresh and confirm the timestamp updates.
6. Test Pause/Resume on a safe disposable cron job only, or use a fake Hermes binary for destructive/action tests. Do not remove real production jobs during verification unless explicitly approved.
7. Verify Run now only on a safe disposable job unless approved.

### Task 9: Feature critique gate

**Objective:** Satisfy the global software delivery rule before calling the feature done.

**Files:**
- Create: `.hermes/reviews/automations-cron-control-plane/handoff.md`
- Review output: `.hermes/reviews/automations-cron-control-plane/critique-report.md`

**Steps:**
1. Write handoff with scope, files changed, exact commands run, and manual verification results.
2. Trigger/request the separate critique-agent review.
3. Fix all Required issues.
4. Re-run targeted tests/build.
5. Re-review until verdict is APPROVED.

## Safety Boundaries

- Zoid must not store Hermes secrets or raw prompts.
- Zoid must not implement an independent scheduler for these jobs.
- Zoid should not create cron jobs in v1; visibility and management only.
- Remove job must require branded confirmation.
- Real pause/resume/run/remove actions should use Hermes read-back before reporting success.
- Testing should avoid mutating real important jobs; use fake Hermes scripts or disposable cron jobs.

## Risks / Tradeoffs

- Hermes CLI JSON support may not exist or may differ by version. The implementation must verify the actual command and fallback safely.
- Directly reading `~/.hermes/cron` could break profile isolation if not done through Hermes helpers. Prefer CLI read-back where possible.
- `cron run` may trigger real external side effects. UI should label it clearly and verification should avoid running production jobs without approval.
- `remove` is destructive. Branded confirmation and provider read-back are mandatory.
- Multi-profile cron management is not included in v1 unless explicitly requested; default profile only.

## Resolved Defaults / No Open Non-Critical Questions

- V1 manages only the active/default Hermes profile and shows it as a read-only label.
- V1 does not create or edit cron schedules/prompts.
- Run now remains available for cron jobs, including protected jobs, but always requires branded confirmation because it may trigger external side effects.
- Watchers are read-only in V1 unless Hermes exposes an explicit watcher control API/source of truth.
- Automations nav health is computed only after opening/refreshing the page, not through background polling.
- Remaining implementation details should follow these defaults unless a critical product, safety, or technical blocker appears.

## Definition of Done

- Automations nav opens a real Automations workspace.
- The page lists live Hermes cron jobs from the default profile.
- Current job status, last run, next run, errors, script/no-agent, skills, and delivery are visible.
- Pause/resume/run/remove use Hermes commands and provider read-back.
- Remove has branded confirmation.
- Tests pass: frontend, Rust, build.
- Zoid is rebuilt/rerun so the feature is visible in-app.
- Feature critique gate is APPROVED.
