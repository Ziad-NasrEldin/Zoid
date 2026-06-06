# Phase 8 Hardening, Packaging, Performance, Accessibility, Release Readiness

Scope: release-readiness hardening for the existing local-first Tauri app. This phase does not add external credentials, fake integrations, or surprise signing/notarization secrets.

## Release readiness report format

Every release readiness report must include:

- Branch/commit and dirty-file summary.
- Build/package command and exact result.
- Focused backend/frontend tests and `npm run verify:local` / `npm run verify:release` output.
- Manual/native checks that were actually run.
- Artifact paths, SHA256, size, bundle name, bundle identifier, and executable check.
- Blockers/deferrals with exact cause.
- Critique verdict once review completes.

## Packaging/signing/notarization path

- Tauri package metadata lives in `src-tauri/tauri.conf.json`.
- Bundle targets are `app` and `dmg`; release verification builds the app bundle and a deterministic internal DMG.
- Local release builds are unsigned unless a macOS Developer ID certificate is available outside git.
- Signing requires a certificate in the macOS keychain and/or CI environment variables outside the repository.
- Notarization/stapling requires Apple credentials/app-specific password or API key outside git; no secret is committed.
- Entitlements are documented in `src-tauri/entitlements.plist`; they stay minimal for local app/runtime behavior.

## Hardening matrix

- Destructive file actions: existing confirmation decision gates remain required and covered by policy/file-action tests.
- Credentials/send/publish/deploy/calendar/Gmail: no fake connected state; credential material stays out of SQLite/logs/events/UI.
- Logs/events/prompts/summaries/errors: obvious token/password/secret/API-key material is redacted before persistence/display.
- Migration failures: fail closed, preserve app-support DB, create pre-migration backup before migration work, and show recovery guidance rather than continuing with partial state.
- Tauri surface: new Phase 8 commands are registered in the allowlist and covered by command-surface tests.

## Implemented Phase 8 artifacts

- Migration v13 (`src-tauri/migrations/0013_phase8_hardening_release.sql`) adds:
  - `log_retention_settings`
  - `log_cleanup_runs`
  - performance indexes over events, tasks, runs, notifications, browser captures/tabs, and cleanup runs.
- Backend `phase8_service` adds:
  - retention setting validation
  - safe direct-child `.log` cleanup with dry-run support
  - age-based retention plus `max_total_bytes` cap enforcement by deleting oldest direct-child log files until under cap
  - structured safe error mapping
  - pre-migration backup helper
  - tests for P8.27-P8.29 coverage.
- Foundation startup now creates pre-migration backups under `~/Zoid/Backups` before migrations on existing app-support DB files.
- Tauri bridge exposes:
  - `list_log_retention_settings_command`
  - `upsert_log_retention_settings_command`
  - `cleanup_logs_command`
- Frontend adds:
  - release/about/log-retention/migration-failure view-models in `src/releaseAbout.ts`
  - command-backed tests in `src/releaseAbout.test.ts`
  - `ReleaseHardeningPanel` in `src/App.tsx` with package metadata, log retention dry-run action, and migration recovery guidance.
- Native packaging metadata and entitlements are configured in:
  - `src-tauri/tauri.conf.json`
  - `src-tauri/entitlements.plist`

## Verification evidence

Latest full release verification:

- Command: `npm run verify:release && git diff --check`
- Result: PASS
- Rust: 187 passed, 0 failed, 1 ignored.
- Frontend: PASS, including `release/about/log-retention view-model tests passed` and `browserWorkspace tests passed`.
- Build: `tsc && vite build` PASS.
- Tauri app bundle: PASS.
- Deterministic DMG: PASS.
- DMG mounted read-only: PASS.
- DMG Applications symlink present: PASS.
- DMG contents verified: PASS.
- `git diff --check`: PASS/no output.

Artifact inspection:

- App: `/Users/ziadnasreldin/Zoid-phase7-plus/src-tauri/target/release/bundle/macos/Zoid.app`
- DMG: `/Users/ziadnasreldin/Zoid-phase7-plus/src-tauri/target/release/bundle/dmg/Zoid_0.1.0_aarch64.dmg`
- Bundle name: `Zoid`
- Bundle identifier: `com.mavoid.zoid`
- DMG SHA256: `0f5d34228d1374d383dd36c6a272b3bd75219236c2195fee6bb7043dd93bced4`
- DMG size: `4.7M`
- Binary check: `Zoid.app/Contents/MacOS/zoid` executable.
- Artifact secret/path scan: PASS, no raw secret-like test values and no project checkout path strings in release app binary.

Packaged launch/manual verification:

- Command pattern: launched `Zoid.app/Contents/MacOS/zoid` with isolated `HOME=/tmp/zoid-phase8-home.*`.
- Result: process stayed running until intentionally killed.
- Isolated first-launch artifacts created:
  - `Library/Application Support/Zoid/zoid.sqlite`
  - `Library/Application Support/Zoid/logs/foundation.log`
  - `Zoid/Backups/zoid-pre-migration-*.sqlite.bak`

## Performance measurements

Measured during the Phase 8 fix cycle after first critique:

- Packaged cold launch with isolated HOME: `164.7 ms` until app-support SQLite appeared; process stayed alive.
- Packaged warm launch with same isolated HOME: `1.0 ms` until existing SQLite was observed; process stayed alive.
- Workspace switching view-model measurement: `10,000` `buildWorkspaceChromeView` iterations in `2.828 ms`, average `0.000283 ms` per switch.
- Indexed event query measurement: in-memory SQLite with `20,000` event rows, `select id from events order by created_at desc,id desc limit 100` returned `100` rows in `0.042 ms`; query plan used `COVERING INDEX idx_events_created_id`.
- Frontend/workspace/log-rendering smoke memory measurement: `/usr/bin/time -l npm run test:frontend` completed in `2.48 real` seconds with `88,752,128` maximum resident set size and `23,579,360` peak memory footprint.

## Security review

- Raw secrets are still rejected/redacted in settings, logs, events, browser captures, notifications, and structured errors by existing and new tests.
- Release app binary scan found no raw secret-like test values after removing a runtime sample literal from the release binary.
- Signing/notarization secrets remain external and are not committed.
- New log cleanup only touches direct child `.log` files, skips symlinks/non-files, and supports dry-run.
- New release UI copy avoids internal provider/secret values and exposes only safe build/package information.

## UX/accessibility/performance review

- App-wide loading/error/blocker states remain explicit and no fake records are simulated in browser preview.
- `ReleaseHardeningPanel` uses labelled sections, real buttons, status badges, and list semantics.
- Migration failure guidance gives safe recovery actions and redacts sensitive text.
- Performance readiness is backed by v13 indexes, release build verification, isolated packaged cold/warm launch measurements, workspace switching view-model timing, indexed event query timing, and frontend smoke memory measurement.

## Unsupported/partial but truthful

- Signing/notarization/stapling are not configured because no Apple certificate/credentials were provided. The flow is documented and no checkbox claims notarized production distribution.
- Native notification click/open-route behavior is documented/reviewed; interactive click-through is not fully automatable in this CLI session.
- Keychain readiness remains truthful via existing readiness status; no fake credential storage is claimed.
