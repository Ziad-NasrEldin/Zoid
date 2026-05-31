# Feature Handoff: Local verification release gate

## Original request

User said GitHub CI/CD is disabled for their GitHub account and asked to find another solution:

> i cant run anything related to CI CD in my giithub accounts its disabled for me, so find something else
> find another solution

## Implementation summary

Replaced the unavailable GitHub Actions-based gate with a local-first verification and release workflow.

Main decisions:

- Removed the active GitHub Actions workflow from main because GitHub Actions/CI/CD is unavailable for this account and every workflow run failed with `startup_failure`, including a minimal smoke workflow.
- Added `scripts/verify-local.sh` as the source-of-truth local release gate.
- Added `scripts/install-git-hooks.sh` to install a local-only pre-push hook that runs the fast gate.
- Added package scripts for local verification and hook installation.
- Updated README and release docs to stop presenting GitHub Actions as the working path and to document local verification instead.

The workflow is now:

- Fast normal-development/local gate: `npm run verify:local`
- Fast push gate: `npm run verify:push`
- Full DMG/package release gate: `npm run verify:release`
- Optional local hook: `npm run hooks:install`

This split prevents normal verification from repeatedly building/mounting the macOS DMG and showing the Finder drag-to-Applications installer window. DMG generation is now explicit via `npm run verify:release` only.

## Changed files

- `.github/workflows/ci.yml`: deleted; GitHub Actions is not available for this account, so this is not an active/usable gate.
- `scripts/verify-local.sh`: new local verification/release script.
- `scripts/install-git-hooks.sh`: new local pre-push hook installer.
- `package.json`: added `verify:local`, `verify:release`, `verify:push`, and `hooks:install` scripts. `verify:local` and `verify:push` run the fast non-packaging gate; `verify:release` runs full DMG/package verification.
- `README.md`: updated to document local verification as source of truth.
- `Docs/release/macos-dmg-signing-notarization.md`: updated release operations for local-first verification, disabled GitHub CI/CD, and future signing/notarization.
- `.hermes/reviews/local-verification-release-gate/handoff.md`: this handoff.

## Local scripts

### `scripts/verify-local.sh`

Supports:

- `--skip-package`: Rust tests + frontend build only.
- `--install`: force `npm ci`.
- `--no-install`: skip dependency installation even if `node_modules` is absent.
- `--help`: usage output.

Full mode (used by `npm run verify:release`) checks:

- `npm` present.
- `cargo` present.
- Tauri CLI present through `node_modules/.bin/tauri` or PATH.
- `hdiutil` present for packaging verification.
- `/usr/libexec/PlistBuddy` present for plist verification.
- Runs `npm run test:rust`.
- Runs `npm run build`.
- Runs `npm run tauri:build`.
- Verifies `.app` and `.dmg` artifacts exist.
- Mounts DMG read-only at a temporary mount point.
- Verifies mounted `Zoid.app`, optional `Applications` symlink, `CFBundleName=Zoid`, `CFBundleIdentifier=com.mavoid.zoid`, `CFBundleExecutable=zoid`, and executable app binary.
- Uses cleanup traps for EXIT/INT/TERM and attempts robust detach cleanup.

### `scripts/install-git-hooks.sh`

- Installs `.git/hooks/pre-push` locally.
- Hook runs `scripts/verify-local.sh --skip-package`.
- Adds a managed marker so repeated installs update cleanly.
- Backs up unmanaged existing hooks with timestamp before replacement.

## How to test

From `/Users/ziadnasreldin/Zoid`:

```sh
bash -n scripts/verify-local.sh scripts/install-git-hooks.sh
npm run verify:local
npm run verify:push
# Only when intentionally producing/checking a new DMG:
npm run verify:release
```

Optional hook install:

```sh
npm run hooks:install
```

## Tests run

Subagent implementation verification:

- `bash -n scripts/verify-local.sh scripts/install-git-hooks.sh`: PASS.
- Temp fake hook install test: PASS.
  - Existing unmanaged hook backed up.
  - Second install detected managed hook and updated without another backup.
- `npm run verify:push`: PASS.
  - Rust tests: 3 passed.
  - Frontend build: passed.
- `npm run verify:local`: PASS.
  - Rust tests: 3 passed.
  - Frontend build: passed.
  - Tauri release build: passed.
  - DMG mounted and inspected successfully.

Parent verification after `verify:local` / `verify:release` split:

- `bash -n scripts/verify-local.sh scripts/install-git-hooks.sh`: PASS.
- `npm run verify:local`: PASS.
- `npm run verify:local` PASS lines:
  - `PASS: npm dependencies present; skipped npm ci`
  - `PASS: tauri CLI found at node_modules/.bin/tauri`
  - `PASS: Rust tests passed`
  - `PASS: frontend build passed`
  - `PASS: local push verification passed (--skip-package)`
- `npm run verify:release`: not re-run after the split to avoid unnecessarily rebuilding/mounting the DMG after the installed-app prompt complaint. The same full package path passed immediately before the split, and remains available only for intentional DMG refreshes.

Earlier full release verification before the split:
  - `PASS: npm dependencies present; skipped npm ci`
  - `PASS: tauri CLI found at node_modules/.bin/tauri`
  - `PASS: Rust tests passed`
  - `PASS: frontend build passed`
  - `PASS: Tauri package build passed`
  - `PASS: app artifact exists: /Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid.app`
  - `PASS: DMG artifact exists: /Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/dmg/Zoid_0.1.0_aarch64.dmg`
  - `PASS: DMG mounted read-only for inspection`
  - `PASS: DMG Applications symlink present`
  - `PASS: DMG contents verified`
  - `PASS: local release verification passed`
- Rust test summary from final local run:
  - `test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out`
- Artifacts from final local run:
  - App: `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid.app`
  - DMG: `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/dmg/Zoid_0.1.0_aarch64.dmg`

## Subagent review results

Spec review:

- Initial spec review: PASS.
- Re-review after fixes: PASS.

Quality review:

- Initial quality review: REQUEST_CHANGES.
- Required changes:
  1. Safer DMG cleanup traps and partial-mount cleanup.
  2. Avoid silently overwriting unmanaged local pre-push hooks.
  3. Correct misleading `--no-install` runtime message.
- Fixes applied:
  1. Separate EXIT/INT/TERM traps and more robust detach cleanup.
  2. Managed hook marker plus timestamped backup for unmanaged hooks.
  3. Accurate `--no-install` skip message.
- Quality re-review verdict: APPROVED.
- Remaining minor non-blocking notes:
  - `--skip-package` still checks Tauri CLI even though it only runs Rust tests and frontend build.
  - EXIT cleanup can run after INT/TERM cleanup; cleanup is idempotent enough.

## Git info

- Branch: `main`
- Base before this local-gate slice: `141f496 fix: make CI triggers explicit for GitHub Actions`
- Working tree before final commit:
  - `D .github/workflows/ci.yml`
  - `M Docs/release/macos-dmg-signing-notarization.md`
  - `M README.md`
  - `M package.json`
  - `?? scripts/`
  - `?? .hermes/reviews/local-verification-release-gate/`

## Frontend/backend/database notes

- Frontend runtime code unchanged.
- Native/Tauri runtime code unchanged.
- Database schema unchanged.
- This is release/verification tooling and documentation only.

## Reviewer focus areas

- Verify removing GitHub Actions is appropriate given account-level CI/CD disablement.
- Verify local scripts actually replace the missing CI/CD gate.
- Verify DMG/app inspection remains strong enough for local releases.
- Verify hook installation is local-only and safe for existing hooks.
- Verify docs do not imply unsigned DMGs are public-distribution ready.
- Verify no secrets or credentials are introduced.

## Known limitations / risks

- GitHub-hosted CI/CD remains unavailable for this account; this solution intentionally does not rely on GitHub Actions.
- The local pre-push hook is opt-in per checkout and must be installed with `npm run hooks:install`.
- Local hooks can be bypassed with `git push --no-verify`; `npm run verify:local` remains the explicit release gate.
- Current DMGs remain unsigned/non-notarized and are internal artifacts only.
- Normal `verify:local` deliberately skips packaging; run `npm run verify:release` only when intentionally refreshing/rechecking the DMG artifact.
- A temporary remote diagnostic branch `ci-smoke-diagnosis` still exists from earlier CI troubleshooting and should be deleted only with explicit approval.
