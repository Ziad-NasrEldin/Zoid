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

## P2.32 — Native verification

Completed through a guarded native app-support verification harness while the native Tauri app was running. The session still could not drive the macOS desktop UI by clicks because AppleScript Accessibility inspection hung and `screencapture` returned `could not create image from display`, so this is **native bridge/app-support verification**, not a visual click-through recording.

### Native launch evidence

- Command: `npm run tauri:dev`
- Vite/Tauri dev URL: `http://127.0.0.1:1420/`
- Cargo compiled and launched: `target/debug/zoid`
- Active native PID for the passing harness run: `17030`
- The earlier `tcsetattr: Inappropriate ioctl for device` came from running Tauri dev under a non-interactive tracked background process; it did not block launch.

### Native create task -> run -> output -> notification/history evidence

Passing command:

```bash
ZOID_P232_REAL_DB=1 ZOID_P232_NATIVE_PID=17030 \
  cargo test p232_native_app_support_flow_creates_run_review_notification_history_and_persists -- --ignored --nocapture
```

Observed result:

- `test tests::p232_native_app_support_flow_creates_run_review_notification_history_and_persists ... ok`
- Created verification marker: `p232-native-verification-1780440530426`
- Created task: `task_1780440530428_0000017055_00000000000000000000`
- Created run: `run_1780440530430_0000017055_00000000000000000000`
- Created session: `session_1780440530428_0000017055_00000000000000000000`
- Created review: `review_1780440530523_0000017055_00000000000000000000`
- Created notification: `notification_1780440530524_0000017055_00000000000000000001`
- Log path: `/Users/ziadnasreldin/Library/Application Support/Zoid/logs/run_1780440530430_0000017055_00000000000000000000.log`
- Database path: `/Users/ziadnasreldin/Library/Application Support/Zoid/zoid.sqlite`

The harness verified:

1. Native app PID was active before running.
2. Real app-support SQLite DB existed.
3. Task creation through native bridge helper persisted.
4. CLI run launched `/bin/sh -lc "printf ..."` and completed.
5. Streamed output reached EOF and contained `P2.32 native verification output: p232-native-verification-1780440530426`.
6. Manual review record was created and approved.
7. Notification was created and listed in active Inbox.
8. Task history contained `task.created`, `review.created`, and `notification.created`.
9. Run history contained `run.completed` and `review.created`.
10. DB was reopened and the exact task/run/review/notification IDs persisted.

### Restart persistence evidence

After the passing harness:

1. The Tauri dev process was stopped.
2. The native app was relaunched with `npm run tauri:dev`.
3. Fresh native PID: `17249`.
4. The exact verification rows were queried from `/Users/ziadnasreldin/Library/Application Support/Zoid/zoid.sqlite`.

Persisted rows after restart:

- Task status: `inbox`, title `P2.32 native verification p232-native-verification-1780440530426`.
- Run status: `completed`, output summary included `P2.32 native verification output: p232-native-verification-1780440530426`.
- Review verdict: `approved`.
- Notification state: `pending`, linked to the task/run/review IDs above.
- Event history included:
  - `task.created`
  - `run.queued`
  - `run.started`
  - `run.completed`
  - `review.created`
  - `review.approved`
  - `notification.created`
- Log file existed and contained:
  - `stdout:`
  - `P2.32 native verification output: p232-native-verification-1780440530426`

### Compatibility fix discovered by real DB verification

The first real app-support harness attempt failed because the existing local DB had a legacy `events.actor text not null` column. `create_event_record` now supports both clean current schemas and legacy local schemas by including the `actor` value only when the column exists.

## Commands that passed during this closeout

- `npm run tauri -- --version`
- `npm run`
- `curl -I --max-time 5 http://127.0.0.1:5174/`
- Browser render smoke of Today and Tasks workspaces at `http://127.0.0.1:5174/`
- `cargo test --no-run p232_native_app_support_flow_creates_run_review_notification_history_and_persists`
- `ZOID_P232_REAL_DB=1 ZOID_P232_NATIVE_PID=17030 cargo test p232_native_app_support_flow_creates_run_review_notification_history_and_persists -- --ignored --nocapture`
