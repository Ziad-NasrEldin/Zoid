# automation-cron-parser-fake-email-delete handoff

## Change
Fixed Zoid Automations cron-list parsing so Hermes `cron list --all` stdout blocks cannot be mis-parsed as cron jobs.

## Root cause
The parser treated any line shaped like `<token> [<status>]` as a job header. Hermes job stdout from MaVoid monitor contained JSON like `"email_retries": []`, which includes ` [`, so Zoid created a fake job card with id/name derived from `email_retries`. Removing it failed because no real Hermes job existed.

## Implementation
- Added strict `is_hermes_cron_job_id`: only 12-character ASCII hex ids are accepted as job headers.
- Parser now rejects stdout JSON array/object lines as job headers.
- Updated cron action fixture IDs to valid hex.
- Added regression test for `"email_retries": []` in job stdout.

## Proof run
- `npm run test:frontend` passed.
- `npm run test:rust` passed: 72 passed, 1 ignored.
- `npm run build` passed.
- `npm run tauri:build` passed.
- Installed `/Applications/Zoid 25.app` from release bundle and relaunched.
- UI smoke: Automations now shows 4 real jobs and no fake `email_retries` card.
- Disposable real job E2E cleanup: created Hermes cron job `email_retries` id `0c16af274c5f`, removed it with Hermes, verified it no longer appears by id in `hermes cron list --all`. Final app refresh shows 4 jobs.

## Files changed
- `/Users/ziadnasreldin/Zoid/src-tauri/src/lib.rs`

## Notes for reviewer
Review that the id validation is appropriate for Hermes cron IDs and does not reject legitimate ids. Also verify the action fixture update still covers pause/resume/run/remove refresh behavior.