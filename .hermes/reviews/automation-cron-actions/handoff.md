# Feature Handoff: automation cron actions

## Scope
Fix and verify Zoid Automations cron-job action buttons: Run now, Pause, Resume, Remove.

## Changes
- `src/automations/automationClient.ts`
  - Sends both `jobId` and `job_id` to `manage_hermes_cron_job` to avoid frontend/Rust invoke argument casing failures.
- `src/automations/AutomationsWorkspace.tsx`
  - Clears stale action success messages before each action.
  - Sets visible success feedback after run/pause/resume/remove.
  - Validates remove by checking the returned Hermes provider read-back no longer includes the removed job ID.
- `src/App.css`
  - Adds `.automation-action-status` styling for action feedback.
- `src-tauri/src/lib.rs`
  - Adds Rust smoke test `hermes_automation_actions_call_cli_and_refresh_provider_state` using a fake Hermes CLI to prove pause/resume/run/remove invoke the expected cron subcommands and refresh provider state.

## Safety/behavior requirements
- Protected cron jobs must remain non-removable in UI/backend.
- Remove must not claim success if the provider read-back still includes the job.
- Run now remains confirmation-gated.
- Remove remains confirmation-gated.

## Validation already run
- `npm run test:frontend` passed.
- `npm run build` passed.
- `npm run test:rust` passed: 67 passed, 1 ignored.
- `npm run tauri:build` passed and produced `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Installed rebuilt app into `/Applications/Zoid 25.app` and launched `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- Native UI smoke with real Hermes provider:
  - Created disposable no-agent cron job `cb32dde72974` named `Zoid Native Button Smoke Temp`.
  - In installed Zoid Automations UI, filtered to job ID.
  - Clicked Pause; `hermes cron list --all` showed `[paused]` and UI showed success read-back message.
  - Clicked Resume; `hermes cron list --all` showed `[active]`.
  - Clicked Run now and confirmed; provider read-back showed next run updated to current timestamp.
  - Clicked Remove and confirmed; `hermes cron list --all` no longer contained the job ID.
  - Removed the temporary script and confirmed no smoke jobs remain.

## Known warnings
- Build/Rust tests show pre-existing warnings for unused imports/functions.
- Vite chunk-size warning remains.

## Review request
Please inspect for regressions, incorrect Tauri invoke argument behavior, unsafe destructive flow, missing state handling, or insufficient tests. Verdict should be APPROVED only if no Required fixes remain.
