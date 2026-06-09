# Handoff: repository agent actions + hybrid runbooks

Feature slug: repository-agent-actions-runbooks

## User request

Add three buttons to each Code repository card:
- Run localhost
- Deploy staging
- Deploy production

Each action should connect to a new Hermes session in Agents. The workflow should improve over iterations by remembering a per-repository/per-action runbook. User approved the hybrid model: canonical operation memory in Zoid/Hermes-profile app data, optional repo-local export to `.hermes/runbooks/<action>.md`.

## Implemented slice

Frontend/session routing:
- Added repository action buttons to every repo card in `src/code/CodeWorkspace.tsx`.
- Buttons show per-action memory state: New, Running, Learning/Learned, Review, Blocked, Fix needed.
- Clicking an action creates a new Hermes session, links the repository, switches to Agents, and auto-sends a structured operation prompt when Hermes is online.
- Session metadata tracks `operationRunId`, `operationAction`, `operationRepositoryId`, and `pendingInitialPrompt`.

Hybrid runbook scaffold:
- Added `src/code/repositoryOperations.ts` with operation actions, profile/run types, storage keys, prompt generation, profile lookup, status labels, `RUNBOOK_UPDATE` extraction, outcome inference, and runbook merge.
- Canonical operation profiles/runs are persisted under Zoid local storage keys as the current app-data scaffold:
  - `zoid25:repository-operation-profiles`
  - `zoid25:repository-operation-runs`
- Prompt explicitly treats Zoid app data as canonical and repo-local `.hermes/runbooks/<action>.md` as explicit export-only.
- Prompt includes `RUNBOOK_UPDATE` output contract and safety rules.

Learning/completion tracking:
- Agents screen reports operation completion back to App after Hermes response success/failure/cancel.
- App infers `blocked`, `needs-user`, or `failed` from normal CLI responses that contain those signals instead of always marking learned.
- App extracts `RUNBOOK_UPDATE` and appends it into `RepositoryOperationProfile.runbookMarkdown` for future prompts.
- Profiles move from running to learned/needs-review/blocked/broken and adjust confidence.
- Runs record finishedAt, resolved outcome, and responseContent.

Safety behavior:
- Localhost prompt allows automatic local server setup/verification but says not to deploy.
- Staging prompt allows staging deployment, requires tests/build where appropriate, and forbids secret disclosure.
- Production prompt allows preparation but explicitly requires confirmation before irreversible production actions, migrations, destructive commands, env overwrites, force pushes, or resets.

Styling:
- Added compact operation strip styling in `src/App.css`, with production visually marked as higher-risk.

Tests/guards:
- Updated `src/scaffold.test.ts` to guard action labels and operation strip.
- Added `src/code/repositoryOperations.test.ts` for runbook extraction, outcome inference, and merge behavior.
- Added repository operation test to `npm run test:frontend`.
- `npm run build` passed.
- `npm run test:frontend` passed.

## Key files changed

- `src/code/repositoryOperations.ts` new
- `src/code/repositoryOperations.test.ts` new
- `src/code/CodeWorkspace.tsx`
- `src/App.tsx`
- `src/App.css`
- `src/sessionState.ts`
- `src/agents/sessionState.ts`
- `src/agents/AgentsHermesScreen.tsx`
- `src/scaffold.test.ts`
- `package.json`
- `.hermes/plans/2026-06-09_102824-repository-agent-actions-runbooks.md`

## Review focus

Please review for Required fixes only:
1. Does the Code card action UI actually trigger the correct Agents session with linked repo/action metadata?
2. Does auto-send risk repeated duplicate sends from `pendingInitialPrompt`?
3. Are production safety boundaries strong enough in the generated prompt?
4. Is runbook learning now real enough for this slice: extract `RUNBOOK_UPDATE`, merge into profile, and use it in future prompts?
5. Does outcome inference avoid marking blocked/failed/needs-user responses as learned?
6. Are session type duplicates (`src/sessionState.ts` and `src/agents/sessionState.ts`) handled consistently?
7. Any TypeScript/runtime issues hidden by current tests?

## Known limitation

The canonical operation store is currently browser localStorage-backed under Zoid keys, not yet moved to Rust/Tauri app-data JSON commands. The feature still keeps repo-local export explicit and uses the stored profile/runbook for the next prompt. If Rust app-data persistence is required in this slice rather than next slice, mark as Required fix.
