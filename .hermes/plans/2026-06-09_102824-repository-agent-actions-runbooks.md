# Repository Agent Actions + Self-Improving Runbooks

## Goal

Add three action buttons to every Code repository card:

1. Run localhost
2. Deploy to staging
3. Deploy to production

Each action creates/opens a dedicated Hermes session in the Agents page, links the selected repository, sends a structured mission prompt, and uses a per-repository/per-action runbook that improves after each completed run.

## Resolved decision: hybrid learning storage

Use a hybrid storage model.

Canonical source of truth:
- Zoid app data / Hermes-profile-scoped app storage.
- Stores run history, current confidence, latest run status, session links, and canonical runbook content.

Optional repo-local export:
- `.hermes/runbooks/localhost.md`
- `.hermes/runbooks/staging.md`
- `.hermes/runbooks/production.md`

Default behavior:
- Keep canonical learning outside the repository so it does not accidentally get committed.
- Allow explicit export/sync into the repo when portability/team sharing is useful.
- Repo-local runbooks should be human-readable and safe to commit only after user approval.

## Current code context found

Relevant existing files:
- `src/code/CodeWorkspace.tsx`
  - Renders repository cards and default-branch controls.
  - Receives `repositories`, `linkedRepositoryId`, and callbacks from `App.tsx`.
- `src/agents/sessionState.ts`
  - Defines `HermesChatSession` and `createSession()`.
  - Sessions already support `linkedRepositoryId` and `hermesCliSessionId`.
- `src/agents/AgentsHermesScreen.tsx`
  - Sends prompts to Hermes CLI with the linked repository path as workdir.
  - Already supports repo detection/linking and queued prompts.
- `src/App.tsx`
  - Owns workspace state, repository state, active Hermes session, and session persistence via localStorage.
  - Currently renders Code or Agents workspaces separately.
- `src-tauri/src/lib.rs`
  - Already contains Hermes CLI command integration and app-data-style JSON persistence patterns.

## Product model

### RepositoryOperationAction

Values:
- `localhost`
- `staging`
- `production`

### RepositoryOperationProfile

One profile per `{repoId, action}`:

- `repoId`
- `repositoryPath`
- `repositoryRemoteUrl?`
- `action`
- `status`: `unknown | learning | learned | needs-review | blocked | broken`
- `confidenceScore`: 0-100
- `lastRunId?`
- `lastSessionId?`
- `lastSuccessfulRunAt?`
- `lastResultUrl?`
- `runbookMarkdown`
- `knownCommands`
- `knownIssues`
- `requiredSecrets`
- `verificationChecklist`
- `dangerousSteps`
- `updatedAt`

### RepositoryOperationRun

One run record per button click / Hermes session:

- `id`
- `repoId`
- `action`
- `sessionId`
- `hermesCliSessionId?`
- `startedAt`
- `finishedAt?`
- `outcome`: `running | success | failed | cancelled | needs-user | blocked`
- `triggeredBy`: `repo-card-button`
- `initialPrompt`
- `runbookSnapshot`
- `commandsAttempted?`
- `failureSummary?`
- `finalWorkingCommands?`
- `resultUrl?`
- `verificationSummary?`
- `nextRunbookPatch?`

## UX direction

Every repository card gets a compact action strip:

- `Run localhost`
- `Deploy staging`
- `Deploy production`

Each button should show operation memory state:
- no prior run: `New`
- running: `Running…`
- learned: `Learned`
- blocked: `Needs setup`
- failed last run: `Fix needed`

Click behavior:
- Switch to Agents workspace.
- Create a new session titled:
  - `Localhost · <repo name>`
  - `Staging deploy · <repo name>`
  - `Production deploy · <repo name>`
- Link the session to the repo via `linkedRepositoryId`.
- Add a user message containing the generated mission prompt.
- Start the Hermes CLI run, or queue it for send if a run is active.
- Card stores the session/run link so the user can jump back later.

## Mission prompt shape

### Localhost

Run the local development server for `<repo>` at `<path>`.

Rules:
- Use the prior runbook below if available.
- If the runbook fails, diagnose and update it.
- Install dependencies only when needed.
- Do not deploy.
- Start or verify the local server.
- Report exact URL/port and health-check result.
- At the end, produce a short `RUNBOOK_UPDATE` section with commands, issues, and next-run notes.

### Staging

Deploy `<repo>` to staging.

Rules:
- Use the prior staging runbook if available.
- Confirm required env/secrets are present; do not expose secret values.
- Run tests/build before deploy when appropriate.
- Deploy only to staging target.
- Verify staging URL and core health checks.
- At the end, produce `RUNBOOK_UPDATE`.

### Production

Prepare production deployment for `<repo>`.

Rules:
- Use prior production runbook if available.
- Inspect branch, diff, env requirements, migrations, tests, and deployment path.
- Do not run irreversible production actions without explicit user confirmation.
- Production deploy/migrations/destructive commands require approval.
- After approval and execution, verify production URL/E2E requirements.
- At the end, produce `RUNBOOK_UPDATE`.

## Safety model

Localhost:
- Can execute automatically after button click.
- Allowed to install deps, build, run dev server, discover ports.
- Should surface long-running process controls in Agents.

Staging:
- Can execute automatically for non-destructive steps.
- Should pause for missing secrets/provider login.
- Should verify staging URL before success.

Production:
- Should create the session and prepare the deployment plan.
- Must require explicit user approval before irreversible actions:
  - production deploy
  - production migrations
  - destructive commands
  - environment overwrites
  - force pushes / branch resets

## Implementation phases

### Phase 1 — Routing and session creation

- Extend `CodeWorkspaceProps` with an `onRepositoryOperationStart(repository, action)` callback.
- Add the three buttons to each repository card.
- In `App.tsx`, implement the callback:
  - create a Hermes session
  - set `linkedRepositoryId`
  - set `activeHermesSessionId`
  - switch `activeWorkspace` to `Agents`
  - attach a generated operation prompt
- Extend `HermesChatSession` with optional metadata:
  - `operationRunId?`
  - `operationAction?`
  - `operationRepositoryId?`

### Phase 2 — Operation store

- Add typed Tauri commands for operation profiles/runs:
  - `load_repository_operation_profiles`
  - `save_repository_operation_profile`
  - `create_repository_operation_run`
  - `finish_repository_operation_run`
  - `export_repository_operation_runbook`
- Store canonical data under Hermes profile/app data, not browser-only localStorage.
- Add frontend client wrapper, e.g. `src/code/repositoryOperationsClient.ts`.

### Phase 3 — Runbook-aware prompt generation

- Build prompts using:
  - repository metadata
  - action type
  - latest canonical runbook
  - safety policy
  - expected `RUNBOOK_UPDATE` output contract
- Show runbook confidence/status on the repo card.

### Phase 4 — Run completion and learning loop

- After Hermes response completes, detect operation sessions and extract a structured summary.
- Update canonical run record.
- Update the operation profile/runbook.
- Mark confidence/status:
  - first success = `learned`, low/medium confidence
  - repeated success = higher confidence
  - failure = `needs-review` or `broken`

### Phase 5 — Optional repo-local export

- Add `Export runbook` / `Sync runbook to repo` affordance.
- Write to `.hermes/runbooks/<action>.md` only after user action.
- Add `.gitignore` suggestion if user wants private runbooks.
- Never export secrets.

### Phase 6 — UI polish and review

- Add operation status badges to cards.
- Add “Last run” link from repo card to Agents session.
- Add confirm modal for production start if needed.
- Run feature critique workflow until approved.

## Likely files to change

Frontend:
- `src/code/CodeWorkspace.tsx`
- `src/code/types.ts`
- `src/code/repositoryOperationsClient.ts` new
- `src/agents/sessionState.ts`
- `src/agents/AgentsHermesScreen.tsx`
- `src/App.tsx`
- `src/App.css`
- `src/scaffold.test.ts`
- likely `src/agents/*.test.tsx` for session routing/queued prompt behavior

Tauri backend:
- `src-tauri/src/lib.rs`
- possibly split repository operation persistence into a new Rust module if `lib.rs` is already too large

Artifacts:
- `.hermes/reviews/repository-agent-actions-runbooks/handoff.md` during implementation

## Validation plan

Before implementation:
- Confirm the critical product behavior below.

During implementation:
- Unit/source guard tests for:
  - three buttons exist per repository card
  - callback receives correct repo/action
  - session is created with linked repo/action metadata
  - generated prompts include action-specific safety rules
  - production prompt requires approval before irreversible steps
  - operation profiles/runs hydrate and persist through Tauri commands
  - repo-local export never includes secrets

Manual/browser smoke:
- Seed repositories.
- Click each action.
- Confirm Agents opens, new session is active, repo is linked, prompt is inserted/sent.
- Confirm repo card shows running/last-run state.

Native/Tauri verification:
- `npm run build`
- frontend tests
- Rust tests where added
- `npm run tauri:build`
- relaunch app and smoke the full flow in the native app.

Review:
- Create feature handoff.
- Run feature critique workflow.
- Fix required findings until verdict is APPROVED.

## Risks / tradeoffs

- Browser-only localStorage is not enough for the learning layer; use backend/app-data persistence.
- Automatic runbook extraction from free-form agent text can be brittle. Strong output markers such as `RUNBOOK_UPDATE` are required.
- Production actions must not become one-click destructive automation by accident.
- Repo-local runbooks can leak internal deployment details if committed; export must be explicit and sanitized.
- Long-running localhost sessions need reliable STOP/cancel behavior and process tracking.
- Multiple simultaneous operation runs can create race conditions in runbook updates; update by run id and timestamp.

## Critical open decision

Question 2 — Should the button immediately send/start the Hermes mission, or only create the prepared Agents session and wait for the user to press Send?

Recommended answer:
- Localhost: immediate start.
- Staging: immediate start for non-destructive prep/deploy to staging, pausing for missing secrets/provider confirmations.
- Production: create prepared session and require an explicit confirmation before the mission can run irreversible production actions. The first click may open the session with the production plan prompt, but it should not silently deploy.

Why this matters:
- It changes the App/Agents integration: immediate execution needs a programmatic `sendHermesPrompt` path or queued initial prompt; draft-only can be implemented as session creation with an unsent composer draft/message.
- It also defines the safety contract for production.
