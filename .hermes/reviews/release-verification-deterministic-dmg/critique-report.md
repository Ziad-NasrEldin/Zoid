# Critique Report: Deterministic release DMG verification

Verdict: APPROVED

## Scope reviewed

- `scripts/verify-local.sh`
- `README.md`
- `Docs/release/macos-dmg-signing-notarization.md`
- `.hermes/reviews/release-verification-deterministic-dmg/handoff.md`

I did not edit application code or release tooling. This report is the only file written.

## Checks run

- Read the handoff, previous critique report, changed release script section, README release-gate section, and macOS release docs.
- `git diff -- scripts/verify-local.sh README.md Docs/release/macos-dmg-signing-notarization.md .hermes/reviews/release-verification-deterministic-dmg/handoff.md`
- `git status --short`
- `bash -n scripts/verify-local.sh && npm pkg get scripts.verify:release scripts.tauri:build`
- `shellcheck scripts/verify-local.sh`
- `npm run verify:release`

## Verification results

`npm run verify:release` passed in this re-review.

Observed successful release-gate output included:

- npm dependency check skipped because dependencies are present.
- Tauri CLI found at `node_modules/.bin/tauri`.
- Rust tests passed: 19 passed, 0 failed.
- Frontend build passed.
- `tauri build --bundles app` completed and produced `src-tauri/target/release/bundle/macos/Zoid.app`.
- Deterministic DMG creation completed without Finder AppleScript.
- Generated app artifact:
  - `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid.app`
- Generated DMG artifact:
  - `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/dmg/Zoid_0.1.0_aarch64.dmg`
- DMG mounted read-only for inspection.
- `/Applications` symlink was present in the mounted DMG.
- DMG contents verified.
- Local release verification passed.

## Summary

The follow-up changes address the prior optional review recommendations that mattered for hygiene and verification strictness:

- `DMG_STAGE` is now protected by an `EXIT` trap while the staging directory is live.
- The staging directory is explicitly cleaned after successful DMG creation, and the stage cleanup trap is cleared before the later mounted-DMG cleanup trap is installed.
- `readlink` is now a required command for release packaging.
- The mounted DMG must contain an `Applications` symlink, and its target must be exactly `/Applications`.

The deterministic release verification path remains appropriate for the stated purpose: a local, CLI-safe internal release gate that builds the real Tauri `.app`, packages it into a real DMG with `hdiutil create`, mounts the DMG read-only, and inspects the app bundle and key metadata without invoking Tauri/create-dmg Finder/AppleScript layout automation.

## Spec compliance

Approved.

- `npm run verify:release` still resolves to `scripts/verify-local.sh`.
- The release path still runs dependency checks, Rust tests, frontend build, Tauri app bundling, DMG creation, and mounted-DMG inspection.
- The Tauri invocation is limited to `--bundles app`, which avoids the previously hanging Finder/AppleScript DMG prettifying path while still building the real application bundle.
- The script creates the internal DMG at `src-tauri/target/release/bundle/dmg/Zoid_<version>_<arch>.dmg`.
- The script verifies the generated DMG by mounting it, not merely by checking for file existence.
- The README and release documentation correctly describe the artifact as unsigned, non-notarized, internal-only, and not a public distribution artifact.

No spec-blocking issues found.

## Shell safety

Approved.

Positive findings:

- The script uses `set -euo pipefail`.
- Changed path variables are quoted consistently.
- Required release commands now include `hdiutil`, `ditto`, `readlink`, and `/usr/libexec/PlistBuddy`.
- `DMG_STAGE` cleanup is trap-protected during staging and DMG creation.
- The stage cleanup trap is cleared before installing the mounted-DMG cleanup trap, avoiding trap overwrite confusion.
- Mounted-DMG cleanup remains protected for `EXIT`, `INT`, and `TERM`.
- `hdiutil attach` uses `-readonly -nobrowse -mountpoint`, which is suitable for noninteractive inspection.
- Existing stale output at the target DMG path is removed before creating a fresh artifact.

`shellcheck scripts/verify-local.sh` reports only two pre-existing `SC1007` warnings for the `CDPATH= cd ...` idiom on lines 22-23. These warnings are unrelated to the deterministic DMG changes and are not release-blocking.

No required shell-safety fixes remain.

## Verification fidelity

Approved.

The current release gate verifies meaningful structure and metadata:

- Confirms the Tauri-built `Zoid.app` exists and is a directory.
- Copies the app bundle into a staging directory with `ditto`.
- Creates a compressed DMG with `hdiutil create`.
- Mounts the generated DMG read-only.
- Confirms `Zoid.app` exists inside the mounted DMG.
- Confirms `Contents/Info.plist` exists.
- Confirms `Contents/MacOS/zoid` exists and is executable.
- Requires the mounted `Applications` entry to be a symlink.
- Requires the mounted `Applications` symlink target to be exactly `/Applications`.
- Verifies key bundle identity values:
  - `CFBundleName=Zoid`
  - `CFBundleIdentifier=com.mavoid.zoid`
  - `CFBundleExecutable=zoid`

Known and acceptable limitation: this internal gate does not verify signing, notarization, stapling, Gatekeeper acceptance, or Finder-prettified public DMG layout. The documentation accurately states that public macOS distribution still requires Developer ID signing and notarization.

## Docs truthfulness

Approved.

The docs truthfully state that:

- `verify:release` builds a Tauri `.app`, creates a deterministic/internal DMG, and inspects it.
- The generated DMG is unsigned and non-notarized.
- The artifact is acceptable for internal smoke checks and artifact review only.
- Public macOS distribution requires Developer ID signing and notarization.
- The deterministic internal DMG intentionally avoids Finder/AppleScript layout automation.

The word “deterministic” is used in the operational sense of deterministic CLI-controlled packaging, not as a claim of bit-for-bit reproducible DMG bytes. The docs do not overclaim reproducible-build semantics.

## Required fixes

None.

## Optional follow-ups

None required for this change set.

If the project later wants public distribution or reproducible-build guarantees, that should be handled as separate work: add signing/notarization/Gatekeeper validation for public releases, or add reproducibility normalization and checksum proof for bit-for-bit deterministic artifact claims.
