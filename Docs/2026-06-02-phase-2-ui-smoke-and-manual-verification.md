# Phase 2 UI smoke and manual verification notes

Date: 2026-06-02
Repo: `/Users/ziadnasreldin/Zoid`

## P2.31 — UI smoke/E2E where feasible

### Automated/browser smoke performed

A Vite browser-preview smoke was feasible without adding a new E2E stack.

Evidence:

- Existing port 5173 was occupied by another app:
  - `node /Users/ziadnasreldin/Documents/GitHub/leadra/node_modules/.bin/vite --host 127.0.0.1 --port 5173`
- Started this repo on port 5174:
  - tracked Hermes process: `proc_897d6f2404a5`
  - command: `npm run dev -- --port 5174 --strictPort`
- HTTP probe passed:
  - `curl -I --max-time 5 http://127.0.0.1:5174/`
  - result: `HTTP/1.1 200 OK`
- Browser render smoke passed:
  - URL: `http://127.0.0.1:5174/`
  - title: `Zoid`
  - Today workspace rendered.
  - Tasks workspace rendered after clicking the Tasks sidebar item.
  - Browser console showed Vite/React informational messages only; no JavaScript errors.

### Native create-task -> start-run E2E feasibility

A full browser-driven E2E for `create task -> start CLI run -> see output -> notification/history` was **not feasible in this checkout/session** without adding tooling because:

- No Playwright/WebDriver/Tauri-driver spec harness is present in the repo.
- `package.json` has no E2E/smoke script beyond unit/model/frontend tests and Tauri build scripts.
- Browser preview has no Tauri native `invoke` bridge, so native commands fail truthfully instead of being simulated.
- The Tasks preview displayed the expected native-only blocker: `Cannot read properties of undefined (reading 'invoke')`.

Covered by automated model/bridge checks instead:

- `npm run test:frontend` includes task bridge integration, linked panels, clean session, run controls, manual review/inbox/history panel model tests.
- `npm run test:rust` includes native bridge/task/run/review/notification/history persistence and lifecycle coverage.

## P2.32 — Manual verification

The full manual macOS flow was **not completed by the agent** because the available automation in this session can render browser preview but cannot drive the native Tauri desktop UI end-to-end.

Required manual/human or future automation flow:

1. Launch native app with `npm run tauri:dev` or packaged `.app`.
2. Create a task through the Tasks workspace.
3. Start a CLI run from task detail.
4. Confirm clean session output appears as cards/status.
5. Confirm notification/inbox attention card and task/run history records appear.
6. Restart the app.
7. Confirm task/run/review/notification/history persistence after restart.

Recommended follow-up to make P2.32 fully automatable:

- Add a Tauri/WebDriver-compatible E2E harness or a dedicated native UI smoke script.
- Keep browser preview smoke separate from native E2E because preview cannot access Tauri commands.

## Commands that passed during this closeout

- `npm run tauri -- --version`
- `npm run`
- `curl -I --max-time 5 http://127.0.0.1:5174/`
- Browser render smoke of Today and Tasks workspaces at `http://127.0.0.1:5174/`
