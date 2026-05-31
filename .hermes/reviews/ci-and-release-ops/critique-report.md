# Critique Report: CI and macOS release operations

## Verdict

APPROVED

## Summary

The feature adds a practical GitHub Actions CI workflow for source verification and macOS Tauri packaging, plus release documentation that correctly distinguishes unsigned internal DMGs from future signed/notarized public distribution artifacts. I found no required fixes. Local build, Rust tests, Tauri packaging, workflow linting, and DMG smoke checks passed. The only meaningful remaining gap is that the new workflow has not yet run on GitHub-hosted runners because the changes are still local/uncommitted; that is an expected deployment-readiness limitation, not a source-code blocker for this slice.

## What was changed

- `.github/workflows/ci.yml`: new workflow triggered on pull requests and pushes to `main`; includes an Ubuntu `verify` job running Linux Tauri dependency install, `npm ci`, `npm run test:rust`, and `npm run build`; includes a dependent macOS `package-macos` job running `npm ci`, `npm run tauri:build`, and uploading DMG/app artifacts.
- `Docs/release/macos-dmg-signing-notarization.md`: new internal macOS release checklist documenting unsigned internal artifact handling, future Apple Developer ID signing/notarization requirements, placeholder secret names, smoke checks, and publish gates.
- `README.md`: replaces starter text with Zoid-specific development commands and a Release / CI section linking to the new release checklist.
- `.hermes/reviews/ci-and-release-ops/handoff.md`: handoff for this review.

## Required fixes

| ID | Severity | Area | Issue | Evidence | Required fix |
|----|----------|------|-------|----------|--------------|
| None | - | - | No blocking issues found. | `npm run test:rust`, `npm run build`, `npm run tauri:build`, `actionlint .github/workflows/ci.yml`, and DMG mount/identity checks all passed locally. | None. |

## Improvements

| ID | Priority | Area | Suggestion | Why it matters |
|----|----------|------|------------|----------------|
| I1 | Medium | CI | After committing/pushing, verify the workflow once on GitHub-hosted runners and record the run result/link in the release notes or handoff. | Local lint/build checks are strong, but runner images and GitHub artifact behavior can still differ from local macOS. |
| I2 | Low | CI | Consider uploading the DMG as the primary review artifact and treating the raw `.app` upload as optional, or archive the `.app` explicitly before upload if reviewers need to run it directly. | GitHub artifact zips can be less reliable for preserving macOS bundle execution semantics than distributing the generated DMG. |
| I3 | Low | CI/Test | Consider adding `cargo fmt --check` and `cargo clippy --all-targets -- -D warnings` once the project is ready to enforce style/lints in CI. | The current CI verifies tests/build/package, but does not yet catch formatting or common Rust lint regressions. |

## Tests performed

- Read handoff: `/Users/ziadnasreldin/Zoid/.hermes/reviews/ci-and-release-ops/handoff.md`.
- Inspected git state/diff:
  - `git status --short` showed `M README.md`, `?? .github/`, `?? .hermes/reviews/ci-and-release-ops/`, `?? Docs/release/`.
  - `git diff --stat` showed README changes; untracked workflow and release doc were inspected with `git diff --no-index`.
- Inspected changed files directly:
  - `.github/workflows/ci.yml`
  - `Docs/release/macos-dmg-signing-notarization.md`
  - `README.md`
  - `package.json`
  - `src-tauri/tauri.conf.json`
- Verified lockfile presence for `npm ci`:
  - `test -f package-lock.json`: present.
- Ran Rust tests:
  - `npm run test:rust`: PASS. Result: 3 tests passed, 0 failed; `src/main.rs` had 0 tests; doc-tests 0 tests.
- Ran frontend production build:
  - `npm run build`: PASS. TypeScript + Vite build completed successfully; generated production assets under `dist/`.
- Ran Tauri macOS package build:
  - `npm run tauri:build`: PASS. Built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/zoid`, bundled `Zoid.app`, and bundled `Zoid_0.1.0_aarch64.dmg`.
- Ran workflow linter:
  - Installed `actionlint` via Homebrew because it was not previously available.
  - `actionlint .github/workflows/ci.yml`: PASS with no output/errors.
- Ran DMG smoke/identity check:
  - Mounted `src-tauri/target/release/bundle/dmg/Zoid_0.1.0_aarch64.dmg` at a temporary mount point.
  - Verified mounted contents included `Zoid.app` and `Applications`.
  - Verified `CFBundleName=Zoid`.
  - Verified `CFBundleIdentifier=com.mavoid.zoid`.
  - Verified `CFBundleExecutable=zoid`.
  - Verified mounted app binary was executable.
- Checked release doc secret handling:
  - Only placeholder values were found for Apple ID, app-specific password, team ID, certificate, and certificate password.
  - No real signing credentials/certificates were observed in the changed docs or workflow.

## Tests still needed

- First real GitHub Actions run after commit/push, including artifact upload/download verification from GitHub.
- Future signed/notarized release candidate validation after Apple Developer Program credentials and Tauri signing/notarization configuration exist:
  - `codesign --verify --deep --strict --verbose=2`
  - `spctl --assess --type execute --verbose`
  - `xcrun stapler validate`

## Dev-agent instructions

1. No required fixes for this review.
2. Commit/push the workflow and docs when ready.
3. Confirm the first GitHub Actions run passes on hosted runners and that the uploaded DMG artifact can be downloaded and smoke checked.
4. Keep unsigned DMGs labeled/internal-only until Developer ID signing and notarization are configured and verified.
