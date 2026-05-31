# Critique Report: Local verification release gate

## Verdict

APPROVED

## Summary

The local-first verification/release workflow is ready to commit. GitHub Actions/CI/CD is unavailable for the account, so the repository now uses local verification gates instead. The final split correctly avoids normal verification repeatedly building/mounting the macOS DMG while preserving an explicit release command for DMG/package inspection.

## What was changed

- Removed the active GitHub Actions workflow because it is not usable on this account.
- Added `scripts/verify-local.sh` as the local verification/release gate.
- Added `scripts/install-git-hooks.sh` for an opt-in local pre-push hook.
- Updated `package.json` scripts:
  - `verify:local`: fast non-packaging gate.
  - `verify:push`: fast non-packaging gate.
  - `verify:release`: full package/DMG inspection gate.
  - `hooks:install`: local hook installer.
- Updated README and macOS release docs to document local gates, CI/CD unavailability, unsigned-DMG limitations, and the intentional DMG-only release path.
- Updated the handoff to reflect the final `verify:local` / `verify:release` split.

## Required fixes

| ID | Severity | Area | Issue | Evidence | Required fix |
|----|----------|------|-------|----------|--------------|
| None | - | - | No blocking issues found. | Focused review and local verification passed. | None. |

## Improvements

| ID | Priority | Area | Suggestion | Why it matters |
|----|----------|------|------------|----------------|
| I1 | Low | Release ops | Run `npm run verify:release` only when intentionally refreshing/checking a DMG artifact. | Avoids unnecessary Tauri DMG/Finder installer windows during normal development. |

## Tests performed

- Reviewed `package.json` script wiring.
- Reviewed `scripts/verify-local.sh` for fast vs release behavior, DMG cleanup, prerequisite checks, and artifact inspection.
- Reviewed `scripts/install-git-hooks.sh` for local-only behavior and safe handling of existing hooks.
- Reviewed `README.md` and `Docs/release/macos-dmg-signing-notarization.md` for documentation accuracy.
- Confirmed no active `.github/workflows/*.yml` / `.yaml` workflow remains.
- Confirmed scripts are executable.
- Parent verification:
  - `bash -n scripts/verify-local.sh scripts/install-git-hooks.sh`: PASS.
  - `npm run verify:local`: PASS.
  - Rust tests: 3 passed.
  - Frontend build: passed.
  - `hdiutil info` after fast verification showed no mounted Zoid images.

## Tests still needed

- `npm run verify:release` should be run only when intentionally producing or refreshing a DMG. It was not re-run for this final review to avoid the repeated DMG installer prompt the user reported; the full release path passed immediately before the command split.

## Dev-agent instructions

1. Commit the approved local workflow changes.
2. Push to `origin/main`.
3. Do not run the full DMG release gate again unless the user explicitly wants a new/rechecked DMG artifact.
