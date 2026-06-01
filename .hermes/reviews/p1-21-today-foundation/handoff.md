# Feature Handoff: P1.21 Today foundation UI

## Original request

Continue Zoid from `/private/tmp/zoid-session-handoff-2026-06-01-p1-21-continuation.md` using the Zoid-wide subagent workflow. Current tracker item:

`P1.21 Frontend integration: Today foundation/widgets from real local state or truthful empty/unconfigured states.`

User also asked to spawn more agents as needed, capped at 9.

## Implementation summary

- Added a dedicated Today foundation view for the `today` workspace.
- Today now builds its hero, metrics, and widget state from `get_foundation_status` data when native status is available.
- Browser/checking states are explicitly labeled as preview/native-only unavailable states.
- Tasks, runs, inbox, integrations, paths, policy, and safeguard readiness show truthful empty/unconfigured/native-only states instead of fabricated activity.
- Added pure Today view-model helper and focused TS assertions.
- Added active `aria-current` state to registry chip buttons.
- Added `test:frontend` and wired it into `verify-local.sh` so the Today view-model regression runs before frontend build.

## Changed files

- `src/App.tsx`: renders the Today foundation overview, metrics, real native path/status data, truthful empty widgets, and active registry chip accessibility state.
- `src/App.css`: adds Today dashboard/metric/widget layout classes.
- `src/todayFoundation.ts`: pure view-model builder for Today foundation native/checking/preview states.
- `src/todayFoundation.test.ts`: no-framework TS assertions for native counts, partial safeguard readiness, keychain truthfulness, policy formatting, checking state, and preview non-simulation.
- `package.json`: adds `test:frontend` script and `tsx` dev dependency.
- `package-lock.json`: locks `tsx` dependency.
- `scripts/verify-local.sh`: runs `npm run test:frontend` before `npm run build`.

## How to test

From `/Users/ziadnasreldin/Zoid`:

- `npm run test:frontend`
- `npm run build`
- `npm run verify:local`

For browser smoke:

- Start or reuse `npm run dev -- --host 127.0.0.1`.
- Open `http://127.0.0.1:1420/` or the Vite-selected port.
- Select Today.
- In browser preview, expect copy that says native-only data is unavailable and no simulated tasks/runs/inbox/completions are shown.
- In the packaged Tauri app, Today should show native `get_foundation_status` counts/paths/safeguard state.

## Tests run

- `npx tsx src/todayFoundation.test.ts`: PASS.
- `npm run build`: PASS.
- Combined command `npx tsx src/todayFoundation.test.ts && npm run build`: PASS.
- Independent subagent final spec re-review reran `npx tsx src/todayFoundation.test.ts` and `npm run build`: PASS.
- Independent subagent final quality re-review reran `npx tsx src/todayFoundation.test.ts` and `npm run build`: PASS.

## Git info

- Branch: `main`
- Commit SHA: not committed yet at handoff creation.
- Diff base: current `main` HEAD before P1.21 commit.

## Frontend/backend/database notes

- Frontend route/component: main app shell in `src/App.tsx`; Today-specific pure logic in `src/todayFoundation.ts`.
- Backend endpoint/command used: existing Tauri `get_foundation_status` only.
- Database: no schema changes.
- Native source-of-truth: `FoundationStatus` returned by the existing Tauri command.
- Browser preview: explicitly labeled UI-only preview; no fake local task/run/inbox/provider data.

## Reviewer focus areas

- Confirm no P1.22 settings shell or P1.23 confirmation UI primitive scope was implemented beyond displaying existing policy status copy.
- Confirm Today native view uses real `FoundationStatus` fields and preview states remain truthful.
- Confirm added frontend test coverage is appropriate and `verify-local.sh` integration is acceptable for P1.21.
- Check accessibility of registry chips and empty/unavailable state copy.

## Fix cycle notes

- First spec review: PASS.
- First quality review: APPROVED with non-blocking recommendations.
- Parent fixed active registry chip accessibility with `aria-current`.
- Parent expanded Today view-model assertions for checking state, safeguard readiness, keychain status, sample policy formatting, and preview no-ready-status behavior.
- Final spec re-review: PASS.
- Final quality re-review: APPROVED.
