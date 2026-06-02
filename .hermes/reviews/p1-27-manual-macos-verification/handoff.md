# Feature Handoff: P1.27 macOS manual verification

## Original request

Continue the Zoid-wide subagent workflow and complete P1.27: Manual verification: launch app locally, verify folders/DB/logs/status/settings on macOS.

## Implementation summary

- No product code changes.
- Launched the app locally with `npm run tauri:dev` from `/Users/ziadnasreldin/Zoid`.
- Verified the native debug process `target/debug/zoid` is running.
- Verified the dev frontend URL responds with HTTP 200 at `http://127.0.0.1:1420/`.
- Verified macOS visible user folders and app-support foundation artifacts were created/present.
- Verified SQLite foundation tables/counts and foundation log output.
- Verified settings storage truthfully: config directory exists; `config/settings.json` is currently a reported status path but is not created by startup; settings persistence is represented by the SQLite `app_settings` table, which exists and currently has 0 rows before user preferences are written.

## Changed files

- No product code changes.
- `.hermes/reviews/p1-27-manual-macos-verification/handoff.md`: verification evidence.
- `.hermes/reviews/p1-27-manual-macos-verification/critique-report.md`: final review evidence.
- `Docs/2026-06-01-zoid-implementation-tracker.md`: marks P1.27 complete after verification.

## How to test

From `/Users/ziadnasreldin/Zoid`:

- Start: `npm run tauri:dev`
- Verify process: `pgrep -fl 'target/debug/zoid'`
- Verify dev URL: `curl -sS -o /dev/null -w '%{http_code}\n' --max-time 5 http://127.0.0.1:1420/`
- Verify folders/files under:
  - `/Users/ziadnasreldin/Zoid`
  - `/Users/ziadnasreldin/Library/Application Support/Zoid`
- Verify database counts with `sqlite3 ~/Library/Application\ Support/Zoid/zoid.sqlite`.

## Tests run

- `pgrep -fl 'target/debug/zoid'`: PASS, returned `26106 target/debug/zoid`.
- `curl ... http://127.0.0.1:1420/`: PASS, returned `200`.
- Visible folder check: PASS for `Notes`, `Content`, `Assets`, `Exports`, `Imports`, `Backups` under `/Users/ziadnasreldin/Zoid`.
- App-support check: PASS for app-support root, `logs`, `config`, `zoid.sqlite`, and `logs/foundation.log`.
- SQLite summary: PASS — `workspaces=14`, `events=1`, `action_policies=20`, `integrations=7`, `app_settings=0`, `migration_version=4`.
- Log tail: PASS — `foundation.ready secure services checked` present.
- Spec review subagent: PASS with caveat to document missing physical `settings.json` truthfully.

## Git info

- Branch: `main`
- Current base before P1.27 docs commit: `3c0823e Add P1.26 frontend smoke coverage`
- Diff base: `HEAD`

## Frontend/backend/database notes

- Frontend/native app: `npm run tauri:dev` launched native debug app and Vite dev URL.
- Backend/native setup: Tauri setup hook calls `ensure_foundation()` on app startup.
- Database: `~/Library/Application Support/Zoid/zoid.sqlite`, migration version 4, seeded baseline counts verified.

## Reviewer focus areas

- This is verification-only; confirm no product code changes are needed.
- Confirm the settings evidence is described truthfully: config directory and SQLite settings table verified; no startup-created physical `settings.json` or default settings rows claimed.
- Confirm the macOS launch/process/folder/DB/log evidence is enough for P1.27.

## Fix cycle notes

- Initial parent check looked for the wrong visible starter folders from the workspace registry labels. Source review showed the actual `VISIBLE_DIRS` are `Notes`, `Content`, `Assets`, `Exports`, `Imports`, and `Backups`; parent reran verification against the correct directories.
