# MaVoid Social Dashboard Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. This is a Zoid 25 feature and is not complete until feature-critique-workflow produces `.hermes/reviews/mavoid-social-dashboard/handoff.md` and the critique report verdict is APPROVED.

**Goal:** Build a full MaVoid social media management dashboard inside Zoid 25: post calendar/queue, post detail preview, review gate, Buffer scheduling/read-back, media hosting status, reports, and automation controls.

**Architecture:** Zoid 25 is the control plane and local product UI. Hermes cron and the MaVoid Buffer workspace remain execution/data sources for v1; Buffer remains the publishing provider only. The Tauri backend exposes safe typed commands that read local MaVoid social artifacts, query/manage protected Hermes cron jobs, run a secret-safe Buffer health probe, validate public media URLs, and return redacted state to React. React renders a Zoid-native Social workspace with overview, queue/calendar, post detail, automation, Buffer health, media hosting, and reports panels.

**Tech Stack:** Zoid 25 Tauri v2, Rust backend commands in `src-tauri/src/lib.rs` or modules, React/TypeScript frontend, Vite, existing Zoid sumi-e UI system, existing Hermes cron bridge, local workspace `/Users/ziadnasreldin/MaVoid/social-automation-buffer`, Buffer GraphQL endpoint `https://api.buffer.com/graphql`.

---

## Confirmed Product Decisions

1. Zoid replaces the social management surface; Buffer is only the publishing provider behind the scenes.
2. Use Buffer, not OmniSocials, for new scheduling/publishing.
3. Do not call `api.omnisocials.com` for new scheduling.
4. The canonical non-iCloud workspace is `/Users/ziadnasreldin/MaVoid/social-automation-buffer`.
5. The dashboard must expose both social post management and automation management.
6. Posts are designed visual assets plus captions by default, not text-only posts.
7. Autonomous human approval can be waived for the configured daily run, but independent reviewer approval cannot be waived.
8. No post can be scheduled unless reviewer verdict is `APPROVED`.
9. No post can be scheduled unless public direct media URL validation passes.
10. No post can be marked scheduled/posted unless Buffer read-back verifies provider state.
11. Buffer rate limits must be handled conservatively; when `HTTP 429 RATE_LIMIT_EXCEEDED` appears, stop repeated polling and show cooldown state.
12. No Buffer secrets may be displayed, logged, stored in dashboard JSON, or returned to React.
13. Current daily cadence remains:
    - creator around 08:00 Africa/Cairo
    - AI Intel post at 10:00 Africa/Cairo
    - enterprise authority/carousel post at 18:00 Africa/Cairo
14. Platforms currently expected through Buffer are Instagram, Facebook, and LinkedIn. X is unavailable until connected.
15. The v1 dashboard starts read-only, then adds safe controls, then adds editing/scheduling workflows.
16. Destructive cron actions stay protected by backend policy and branded confirmation.
17. The UI must show real state only. No fake metrics, fake scheduled counts, or assumed provider success.

---

## Current Code Facts

- Zoid app root: `/Users/ziadnasreldin/Zoid`.
- Zoid package: `zoid-25`, Tauri + React/Vite.
- Existing app navigation is in `src/App.tsx`.
- Current navigation includes `Content` and `Automations`.
- Current `Content` nav meta still says `OmniSocials` and the screen has stale OmniSocials/fail-closed copy:
  - `src/content/ContentWorkspace.tsx`
  - `src/content/contentModel.ts`
  - `src/content/contentWorkspace.test.ts`
- Existing automation dashboard already uses a Tauri bridge:
  - `src/automations/AutomationsWorkspace.tsx`
  - `src/automations/automationClient.ts`
  - `src/automations/automationViewModel.ts`
  - `src/automations/types.ts`
- Existing frontend automation calls:
  - `listHermesAutomations()` invokes `list_hermes_automations`
  - `manageHermesCronJob(jobId, action)` invokes `manage_hermes_cron_job`
- Existing Rust backend command surface is in `src-tauri/src/lib.rs`.
- Current MaVoid Buffer workspace files:
  - `/Users/ziadnasreldin/MaVoid/social-automation-buffer/STATUS.json`
  - `/Users/ziadnasreldin/MaVoid/social-automation-buffer/docs/buffer-social-automation-workflow.md`
  - `/Users/ziadnasreldin/MaVoid/social-automation-buffer/docs/zoid-25-dashboard-requirements.md`
  - `/Users/ziadnasreldin/MaVoid/social-automation-buffer/dashboard-spec/dashboard-data-model.md`
  - `/Users/ziadnasreldin/MaVoid/social-automation-buffer/scripts/buffer_check.py`
  - `/Users/ziadnasreldin/MaVoid/social-automation-buffer/artifacts/proof-post-2026-06-09/`
- Current proof state from `STATUS.json`:
  - proof PNG exists
  - reviewer verdict `APPROVED`
  - preferred public URL `https://files.catbox.moe/9tix1y.png`
  - not posted because Buffer returned `HTTP 429 RATE_LIMIT_EXCEEDED`, window `24h`
- Current related Hermes cron jobs:
  - Creator: `12fd35ec77e2` — `MaVoid Daily Social Creator Scheduler — Buffer`
  - Monitor: `9562e7cb93b6` — `MaVoid Buffer Publish Monitor`
  - Cooldown resume/check: `a0caa25a4cf7`

---

## Target Product Shape

### 1. Overview

Purpose: one glance system health.

Show:
- overall state: Healthy / Rate-limited / Needs review / Media blocked / Failed closed / Paused
- today/tomorrow post slots
- Buffer health
- creator automation status
- publish monitor status
- current blocker
- next run times
- latest report links

Actions:
- refresh
- run Buffer health check
- run creator now, behind confirmation
- pause/resume creator
- pause/resume monitor
- open latest report

### 2. Queue / Calendar

Purpose: replace social platform calendar/queue screens.

Views:
- list v1
- filters v1: all, today, needs review, ready to schedule, scheduled, posted, blocked
- week/month calendar later

Rows/cards show:
- date
- slot type: AI Intel / Enterprise carousel / manual campaign
- local publish time
- UTC publish time
- title/topic/news item
- platforms
- lifecycle state
- reviewer verdict
- media URL status
- Buffer scheduled/posted state

### 3. Post Detail

Purpose: complete post manager.

Show:
- large PNG/carousel preview
- caption snapshot
- platform adaptations
- public media URLs
- review report
- Buffer API summary
- Buffer read-back verification
- platform/channel status
- event timeline
- artifact/report paths

Actions:
- rerun reviewer, later phase
- regenerate post, later phase
- edit caption, later phase
- validate public URLs
- retry Buffer schedule when healthy
- open artifact folder/report
- mark manually posted only with evidence/reason, later phase

### 4. Automation Management

Purpose: manage the social automation without leaving Zoid.

Show:
- creator job
- monitor job
- cooldown jobs
- enabled/paused
- schedule
- next run
- last run
- last status/error
- protected state

Actions:
- run now
- pause/resume
- refresh
- open last output/report
- no raw prompt by default
- no remove for protected jobs

### 5. Buffer Health

Purpose: expose provider truth safely.

Show:
- endpoint `https://api.buffer.com/graphql`
- last probe time
- HTTP status
- rate limit state/window
- credentials present booleans only
- connected channel summary when available
- cooldown state

Actions:
- probe once
- cooldown-aware resume/check

### 6. Media Hosting

Purpose: ensure Buffer-compatible public media.

Show:
- public direct URL
- provider
- temporary/durable
- HTTP status
- content type
- byte size
- validation time

Production upgrade:
- Cloudflare R2/public media domain adapter.

### 7. Reports / Audit

Purpose: explain what happened and prove state.

Show:
- generation reports
- review reports
- Buffer scheduling reports
- monitor reports
- failed-closed reports
- manual operator events

---

## Data Contracts

### TypeScript contract

Create `src/social/types.ts`:

```ts
export type SocialOverallStatus =
  | "healthy"
  | "rate_limited"
  | "needs_review"
  | "media_blocked"
  | "ready_to_schedule"
  | "scheduled_verified"
  | "posted"
  | "failed_closed"
  | "paused"
  | "unknown";

export type SocialPostStatus =
  | "planned"
  | "generating"
  | "rendered"
  | "review_requested"
  | "request_changes"
  | "approved"
  | "media_hosted"
  | "buffer_pending"
  | "scheduled_unverified"
  | "scheduled_verified"
  | "posted"
  | "rate_limited"
  | "media_blocked"
  | "buffer_failed"
  | "failed_closed"
  | "manually_resolved";

export type SocialPlatform = "instagram" | "facebook" | "linkedin" | "x";

export type MavoidSocialOverview = {
  workspacePath: string;
  overallStatus: SocialOverallStatus;
  activeBlocker: string | null;
  bufferEndpoint: string;
  bufferHealth: MavoidBufferHealth;
  automation: MavoidAutomationStatus;
  counts: {
    totalPosts: number;
    needsReview: number;
    readyToSchedule: number;
    scheduledVerified: number;
    posted: number;
    blocked: number;
  };
  nextSlots: MavoidSocialSlot[];
  latestReportPath: string | null;
  updatedAt: string;
};

export type MavoidBufferHealth = {
  ok: boolean;
  httpStatus: number | null;
  rateLimited: boolean;
  rateLimitWindow: string | null;
  credentialsPresent: {
    bufferAccessToken: boolean;
    bufferOrganizationId: boolean;
  };
  lastCheckedAt: string | null;
  message: string | null;
};

export type MavoidAutomationStatus = {
  creatorJobId: string;
  creatorEnabled: boolean;
  creatorState: string;
  creatorNextRunAt: string | null;
  monitorJobId: string;
  monitorEnabled: boolean;
  monitorState: string;
  monitorNextRunAt: string | null;
  cooldownJobId: string | null;
  cooldownNextRunAt: string | null;
};

export type MavoidSocialSlot = {
  id: string;
  date: string;
  slotType: "ai_intel" | "enterprise_carousel" | "manual_campaign";
  localPublishTime: string;
  utcPublishTime: string | null;
  status: SocialPostStatus;
};

export type MavoidSocialPost = {
  id: string;
  postDate: string;
  slotType: MavoidSocialSlot["slotType"];
  title: string;
  topicOrNewsItem: string;
  caption: string;
  platforms: SocialPlatform[];
  status: SocialPostStatus;
  review: MavoidReviewReport | null;
  mediaAssets: MavoidMediaAsset[];
  bufferPosts: MavoidBufferPost[];
  reports: MavoidReportRef[];
  events: MavoidSocialEvent[];
};

export type MavoidMediaAsset = {
  path: string;
  publicUrl: string | null;
  contentType: string | null;
  bytes: number | null;
  width: number | null;
  height: number | null;
  validatedAt: string | null;
  provider: string | null;
  temporary: boolean;
  validationStatus: "valid" | "invalid" | "unchecked";
};

export type MavoidReviewReport = {
  verdict: "APPROVED" | "REQUEST_CHANGES" | "MISSING";
  reviewer: string | null;
  reportPath: string | null;
  requiredFixes: string[];
  approvedAt: string | null;
};

export type MavoidBufferPost = {
  bufferId: string | null;
  platform: SocialPlatform;
  channelId: string | null;
  channelDisplayName: string | null;
  scheduledAtUtc: string | null;
  scheduledAtLocal: string | null;
  state: "not_created" | "scheduled" | "posted" | "failed" | "unknown";
  readBackVerifiedAt: string | null;
  publishedUrl: string | null;
  lastErrorCode: string | null;
  lastErrorMessage: string | null;
};

export type MavoidReportRef = {
  label: string;
  path: string;
  kind: "generation" | "review" | "buffer" | "monitor" | "handoff" | "other";
  createdAt: string | null;
};

export type MavoidSocialEvent = {
  timestamp: string;
  actor: "zoid" | "hermes" | "buffer" | "operator";
  eventType: string;
  message: string;
  severity: "info" | "warning" | "error" | "success";
  evidencePath: string | null;
};
```

### Tauri command contract

Add frontend client `src/social/socialClient.ts`:

```ts
import { invoke } from "@tauri-apps/api/core";
import type { MavoidSocialOverview, MavoidSocialPost } from "./types";

export function getMavoidSocialOverview(): Promise<MavoidSocialOverview> {
  return invoke<MavoidSocialOverview>("mavoid_social_get_overview");
}

export function listMavoidSocialPosts(): Promise<MavoidSocialPost[]> {
  return invoke<MavoidSocialPost[]>("mavoid_social_list_posts");
}

export function getMavoidSocialPost(postId: string): Promise<MavoidSocialPost> {
  return invoke<MavoidSocialPost>("mavoid_social_get_post", { postId, post_id: postId });
}

export function runMavoidBufferHealthCheck(): Promise<MavoidSocialOverview> {
  return invoke<MavoidSocialOverview>("mavoid_social_run_buffer_health_check");
}

export function manageMavoidSocialAutomation(action: "run_creator" | "pause_creator" | "resume_creator" | "pause_monitor" | "resume_monitor"): Promise<MavoidSocialOverview> {
  return invoke<MavoidSocialOverview>("mavoid_social_manage_automation", { action });
}
```

---

## Backend Safety Policy

Implement in Rust, not only in React:

1. Redact secrets before returning any data.
2. Never return raw Authorization headers.
3. Never read or use the iCloud automation folder as runtime source.
4. Only allow known social cron job IDs for v1 actions:
   - `12fd35ec77e2`
   - `9562e7cb93b6`
5. Treat remove/delete as unsupported in this social dashboard v1.
6. Block schedule/retry if review verdict is not `APPROVED`.
7. Block schedule/retry if public media URL validation fails.
8. Block repeated Buffer probes while an active 24h cooldown marker exists unless a future explicit override is implemented.
9. Mark scheduled/posted only from Buffer read-back evidence, not creation response alone.
10. Return truthful unavailable states when files/reports are missing.

---

## Implementation Tasks

### Task 1: Create module docs index

**Objective:** Make this plan discoverable as the formal MaVoid Social Dashboard module plan.

**Files:**
- Create: `Docs/modules/mavoid-social-dashboard/README.md`
- Existing: `Docs/modules/mavoid-social-dashboard/implementation-plan.md`

**Steps:**
1. Add README with module goal, key docs, and source workspace path.
2. Link to this implementation plan.
3. Link to `/Users/ziadnasreldin/MaVoid/social-automation-buffer/docs/zoid-25-dashboard-requirements.md`.
4. Verify with `test -f Docs/modules/mavoid-social-dashboard/README.md`.

**Commit:**
```bash
git add Docs/modules/mavoid-social-dashboard
 git commit -m "docs: add MaVoid social dashboard plan"
```

### Task 2: Add shared frontend social types

**Objective:** Define the React data contract for overview, posts, media, review, Buffer, automation, and events.

**Files:**
- Create: `src/social/types.ts`
- Test: `src/social/socialViewModel.test.ts`

**Steps:**
1. Create `src/social/types.ts` using the data contract above.
2. Export all types.
3. Create a small compile-time fixture in `src/social/socialViewModel.test.ts` importing the types.
4. Run `npm run test:frontend` after frontend scaffolding exists.

**Expected:** TypeScript compiles with no missing exports.

### Task 3: Add frontend social client

**Objective:** Add typed Tauri invoke wrappers for social overview, post list, post detail, Buffer health check, and social automation actions.

**Files:**
- Create: `src/social/socialClient.ts`
- Test: `src/social/socialClient.test.ts` if existing test style supports invoke mocking; otherwise cover through view model/component tests.

**Steps:**
1. Add `getMavoidSocialOverview()`.
2. Add `listMavoidSocialPosts()`.
3. Add `getMavoidSocialPost(postId)`.
4. Add `runMavoidBufferHealthCheck()`.
5. Add `manageMavoidSocialAutomation(action)`.
6. Use dual snake/camel argument names where current app patterns require it.

**Expected:** Client compiles and mirrors existing `automationClient.ts` style.

### Task 4: Add Rust backend types and workspace constants

**Objective:** Create the Rust data structures and constants for the MaVoid social workspace.

**Files:**
- Modify: `src-tauri/src/lib.rs`, or create `src-tauri/src/mavoid_social.rs` and wire it from `lib.rs`.
- Test: `src-tauri/src/lib.rs` unit tests or module tests.

**Constants:**
```rust
const MAVOID_SOCIAL_WORKSPACE: &str = "/Users/ziadnasreldin/MaVoid/social-automation-buffer";
const MAVOID_STATUS_PATH: &str = "/Users/ziadnasreldin/MaVoid/social-automation-buffer/STATUS.json";
const MAVOID_BUFFER_CHECK_SCRIPT: &str = "/Users/ziadnasreldin/MaVoid/social-automation-buffer/scripts/buffer_check.py";
const MAVOID_CREATOR_JOB_ID: &str = "12fd35ec77e2";
const MAVOID_MONITOR_JOB_ID: &str = "9562e7cb93b6";
```

**Steps:**
1. Add serializable Rust structs matching TypeScript shape.
2. Add helpers to read JSON files safely.
3. Add helpers to normalize missing files into safe empty states.
4. Add tests for missing workspace, missing status file, and redacted output.

**Expected:** `npm run test:rust` passes.

### Task 5: Implement read-only overview command

**Objective:** Expose `mavoid_social_get_overview` from Rust.

**Files:**
- Modify: `src-tauri/src/lib.rs` or `src-tauri/src/mavoid_social.rs`
- Modify: Tauri command registration list
- Test: Rust unit tests

**Behavior:**
1. Read `STATUS.json`.
2. Read/list relevant social artifacts if present.
3. Read Hermes cron state using existing automation list helper if reusable; otherwise call existing internal logic behind `list_hermes_automations`.
4. Derive `overallStatus`.
5. Return redacted overview.

**Derivation rules:**
- If Buffer health says rate-limited, `overallStatus = "rate_limited"`.
- If approved proof post has valid media but no Buffer post, `overallStatus = "ready_to_schedule"` only when Buffer healthy; otherwise rate-limited.
- If reviewer verdict missing or request changes, `overallStatus = "needs_review"`.
- If public URL missing/invalid, `overallStatus = "media_blocked"`.
- If no data, `overallStatus = "unknown"`.

**Expected:** Command returns proof-post status without exposing secrets.

### Task 6: Implement post list and post detail commands

**Objective:** Let React list and inspect MaVoid social posts from local artifacts/reports.

**Files:**
- Modify: backend social module
- Test: Rust unit tests

**Behavior:**
1. Scan these folders:
   - `artifacts/`
   - `runtime/Generated_Posts/`
   - `runtime/Reviews/`
   - `runtime/Buffer_Reports/`
2. Parse known `manifest.json` files.
3. For the proof post, map manifest to a `MavoidSocialPost`.
4. Include review report path and content summary.
5. Include image path and public URL.
6. If future reports exist, include them.

**Commands:**
- `mavoid_social_list_posts`
- `mavoid_social_get_post`

**Expected:** UI can show proof post detail from local files.

### Task 7: Implement Buffer health command

**Objective:** Run the standalone secret-safe Buffer probe from Zoid.

**Files:**
- Modify: backend social module
- Test: Rust unit tests with mocked command runner if available; otherwise integration-style guard tests for parser.

**Behavior:**
1. Execute:
   ```bash
   python3 /Users/ziadnasreldin/MaVoid/social-automation-buffer/scripts/buffer_check.py
   ```
2. Parse stdout JSON even when exit code is `1` for rate limit.
3. Return `MavoidBufferHealth`.
4. Never include token values.
5. If output says 429/rate-limited, store/derive cooldown state and prevent immediate repeated probes in frontend/backed policy.

**Expected:** Current machine returns `rateLimited: true`, `httpStatus: 429`, credential booleans true.

### Task 8: Implement public media URL validation command

**Objective:** Validate direct image URLs from Zoid before scheduling/retry.

**Files:**
- Modify: backend social module
- Modify: `src/social/socialClient.ts`
- Test: Rust unit tests for URL validation parser; avoid network in unit tests where possible.

**Command:**
- `mavoid_social_validate_media_url(url: String)`

**Behavior:**
1. Allow `https://` only.
2. Run a HEAD or small GET range request.
3. Return HTTP status, content type, byte size if available, and direct-image verdict.
4. Accept `image/png`, `image/jpeg`, `image/webp`.
5. Reject HTML landing pages.

**Expected:** Proof URL `https://files.catbox.moe/9tix1y.png` validates as image.

### Task 9: Implement safe social automation controls

**Objective:** Manage only known MaVoid social cron jobs from the Social dashboard.

**Files:**
- Modify: backend social module
- Modify: `src/social/socialClient.ts`
- Test: Rust tests for action allowlist/protection.

**Command:**
- `mavoid_social_manage_automation(action: String)`

**Allowed actions:**
- `run_creator`
- `pause_creator`
- `resume_creator`
- `pause_monitor`
- `resume_monitor`

**Rejected actions:**
- remove
- arbitrary job id
- prompt editing
- schedule editing in v1

**Backend mapping:**
- creator actions map to job `12fd35ec77e2`
- monitor actions map to job `9562e7cb93b6`

**Expected:** Social dashboard cannot remove or mutate unrelated Hermes jobs.

### Task 10: Build SocialWorkspace shell

**Objective:** Add a Zoid-native Social workspace shell without changing behavior yet.

**Files:**
- Create: `src/social/SocialWorkspace.tsx`
- Create: `src/social/SocialOverview.tsx`
- Create: `src/social/SocialQueue.tsx`
- Create: `src/social/SocialPostDetail.tsx`
- Create: `src/social/BufferHealthPanel.tsx`
- Create: `src/social/AutomationControlPanel.tsx`
- Create: `src/social/MediaHostingPanel.tsx`
- Create: `src/social/ReportsPanel.tsx`
- Modify: `src/App.tsx`
- Modify: `src/App.css`

**UI layout:**
- Header/status banner at top.
- Left queue/list.
- Center selected post detail and preview.
- Right operational panels.

**Design direction:**
- Use existing Zoid sumi-e/ink/red-seal style.
- Product dashboard, not generic SaaS.
- Real state labels: rate-limited, approved, media hosted, not posted.
- No fake metrics.

**Expected:** App has a Social/Content dashboard route showing loading/empty/error states.

### Task 11: Connect read-only overview and post detail

**Objective:** Populate the UI from backend read-only commands.

**Files:**
- Modify: `src/social/SocialWorkspace.tsx`
- Modify: child components
- Test: `src/social/SocialWorkspace.behavior.test.tsx`

**Behavior:**
1. Load overview and post list on mount.
2. Select first post by default.
3. Show proof post image preview.
4. Show caption from manifest.
5. Show review verdict `APPROVED`.
6. Show public media URLs.
7. Show Buffer blocker `RATE_LIMIT_EXCEEDED`.
8. Show empty state if no posts exist.
9. Show bridge unavailable state if running outside Tauri preview.

**Expected:** Read-only v1 dashboard is useful without controls.

### Task 12: Add automation controls with branded confirmation

**Objective:** Add safe run/pause/resume controls for creator/monitor.

**Files:**
- Modify: `src/social/AutomationControlPanel.tsx`
- Possibly extract shared confirmation component if existing Automations panel cannot be reused.
- Test: `src/social/SocialWorkspace.behavior.test.tsx`

**Behavior:**
1. Run creator now requires confirmation.
2. Pause/resume shows action feedback.
3. Buttons disable while action is in flight.
4. On success, reload overview.
5. On error, show actionable error message.
6. Remove/delete is not present.

**Expected:** Controls manage only MaVoid social jobs.

### Task 13: Add Buffer health/cooldown UX

**Objective:** Make Buffer rate-limit state visible and prevent noisy repeated probes.

**Files:**
- Modify: `src/social/BufferHealthPanel.tsx`
- Modify: backend health command if needed
- Test: frontend behavior test

**Behavior:**
1. Show endpoint, last checked time, HTTP status, rate-limit window.
2. Show credentials present booleans only.
3. If rate-limited, show cooldown banner.
4. Disable repeated probe button while cooldown active unless future override is added.
5. Explain that posts can be prepared/reviewed/hosted while Buffer is blocked.

**Expected:** User understands why posting is blocked without causing more 429s.

### Task 14: Add media hosting panel and validation

**Objective:** Show media readiness for Buffer and validate proof URLs.

**Files:**
- Modify: `src/social/MediaHostingPanel.tsx`
- Modify: `src/social/socialClient.ts`
- Test: frontend behavior test

**Behavior:**
1. List all media assets/public URLs.
2. Label temporary providers.
3. Validate selected URL.
4. Show content type and HTTP status.
5. Show production recommendation: durable Cloudflare R2/public media domain.

**Expected:** Dashboard distinguishes “approved media exists” from “Buffer-compatible public URL exists.”

### Task 15: Add reports/audit panel

**Objective:** Surface report paths and event history inside Zoid.

**Files:**
- Modify: `src/social/ReportsPanel.tsx`
- Add open-report/open-folder command only if needed.
- Test: frontend behavior test.

**Behavior:**
1. List available reports.
2. Show kind: generation/review/buffer/monitor/handoff.
3. Open local report path through Tauri shell/dialog if already available in app patterns.
4. Show missing reports truthfully.

**Expected:** Operator can inspect proof and blockers without browsing raw folders.

### Task 16: Replace stale OmniSocials copy

**Objective:** Remove misleading OmniSocials/fail-closed copy from the active Content/Social workspace.

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/content/ContentWorkspace.tsx` only if reused, or replace lazy import with `src/social/SocialWorkspace.tsx`.
- Modify: tests referencing OmniSocials wording.

**Behavior:**
1. Navigation meta becomes `Buffer social` or `Social Ops`.
2. Hero copy explains Buffer-backed social control plane.
3. No visible “OmniSocials” label remains in the new MaVoid social dashboard unless it appears in historical migration docs/reports.

**Expected:** Product copy matches current provider reality.

### Task 17: Frontend tests

**Objective:** Verify UI states and safety constraints.

**Files:**
- Create: `src/social/SocialWorkspace.behavior.test.tsx`
- Create/modify: `src/social/socialViewModel.test.ts`
- Modify: `package.json` test script to include new tests.

**Test cases:**
1. Renders rate-limited overview with cooldown banner.
2. Renders proof post with approved review state.
3. Shows image/public URL details.
4. Does not show Remove/Delete automation action.
5. Run creator opens confirmation.
6. Schedule/retry disabled when Buffer rate-limited.
7. Schedule/retry disabled when reviewer not approved.
8. Empty state is truthful.
9. Bridge unavailable message is truthful in web preview.

**Expected:** `npm run test:frontend` passes.

### Task 18: Rust tests

**Objective:** Verify backend parsing, redaction, allowlists, and missing-file behavior.

**Files:**
- Modify/create Rust module tests.

**Test cases:**
1. Parse `STATUS.json` into overview.
2. Map proof manifest into post detail.
3. Parse Buffer 429 JSON into `rateLimited = true`.
4. Reject arbitrary cron job action.
5. Reject remove/delete action.
6. Missing workspace returns `unknown`/empty state, not panic.
7. Secret-like token values are not included in serialized responses.

**Expected:** `npm run test:rust` passes.

### Task 19: Build and browser/Tauri verification

**Objective:** Prove the dashboard works in the actual app.

**Commands:**
```bash
npm run test:frontend
npm run test:rust
npm run build
npm run tauri:dev
```

**Manual checks:**
1. Open Social/Content workspace.
2. Confirm overview loads real MaVoid Buffer workspace state.
3. Confirm proof post preview loads.
4. Confirm caption and review report appear.
5. Confirm Buffer health shows 429/rate-limited if still blocked.
6. Confirm no secrets appear in UI or console.
7. Confirm automation controls show creator/monitor only.
8. Confirm destructive remove is unavailable.
9. Confirm console has no errors.

**Expected:** Working app UI backed by real local state.

### Task 20: Feature critique gate

**Objective:** Complete mandatory product/quality review before declaring the feature done.

**Files:**
- Create: `.hermes/reviews/mavoid-social-dashboard/handoff.md`
- Critique report path created by critique agent/watchdog.

**Handoff must include:**
- scope
- changed files
- screenshots or visual inspection notes
- commands run and outputs
- safety guarantees
- known blockers
- confirmation that no secrets were exposed
- confirmation that Buffer posting was not falsely claimed

**Process:**
1. Write handoff.
2. Trigger/wait for critique-agent review.
3. Fix all Required fixes.
4. Re-review until verdict is `APPROVED`.

**Expected:** Feature complete only when critique verdict is `APPROVED`.

---

## Acceptance Criteria

1. Zoid shows a full MaVoid social dashboard backed by `/Users/ziadnasreldin/MaVoid/social-automation-buffer`.
2. User can see current system status, Buffer health, social queue, post detail, review status, media URLs, reports, and automation jobs.
3. Proof post appears with image preview, caption, approved review, public URL, and Buffer rate-limit blocker.
4. Dashboard never exposes Buffer token values.
5. Dashboard does not claim a post is scheduled/posted without Buffer read-back evidence.
6. Dashboard disables unsafe schedule/retry when Buffer is rate-limited.
7. Dashboard prevents repeated rate-limit probe spam.
8. Dashboard can safely run/pause/resume only known MaVoid social jobs.
9. No OmniSocials copy remains in the active new social dashboard except historical/migration context.
10. Frontend tests, Rust tests, build, and Tauri/manual verification pass.
11. Feature critique workflow returns `APPROVED`.

---

## Non-Goals for v1

- Rebuilding Hermes cron scheduler inside Zoid.
- Full Buffer replacement backend from scratch.
- Multi-brand dashboard.
- Analytics from Instagram/Facebook/LinkedIn beyond provider/post state.
- Automatic Cloudflare R2 media hosting, unless explicitly pulled into this phase.
- Direct social network posting outside Buffer.
- Raw cron prompt editing.
- Cron job deletion from Social dashboard.

---

## Later Phases

1. Cloudflare R2 durable media hosting.
2. In-dashboard caption editing with saved revisions.
3. In-dashboard reviewer rerun/regeneration controls.
4. Calendar week/month views.
5. Campaign planning and content-plan topic selector.
6. Multi-brand support.
7. Buffer channel mapping UI.
8. Provider analytics and published-link harvesting.
9. Manual approval policy configuration.
10. Notification/email state viewer.
