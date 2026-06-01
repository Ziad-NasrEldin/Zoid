# Feature Handoff: Deterministic release DMG verification

## Original request

User asked: "can you run verify release please after each stop ... try again and continue where you left off"

While resuming Zoid Phase 1 work, `npm run verify:release` was intentionally run. It passed Rust/frontend/Tauri app compilation but timed out during Tauri/create-dmg's Finder/AppleScript DMG prettifying step, leaving an interstitial mounted DMG and no final DMG artifact. The user wants release verification to run after stops, so the release gate needed to be deterministic in the CLI environment.

## Implementation summary

- Changed `scripts/verify-local.sh` release path to build the real Tauri `.app` bundle with `tauri build --bundles app`.
- Added deterministic unsigned internal DMG creation via `hdiutil create` from a staging directory containing `Zoid.app` and an `/Applications` symlink.
- Kept the existing read-only DMG mount/inspection checks for app bundle, executable, Applications symlink, and plist identity.
- Avoided the generated create-dmg Finder/AppleScript path that hung waiting for Finder `.DS_Store` metadata.
- Updated README and release docs to state the internal release DMG is deterministic hdiutil packaging from the verified Tauri app bundle, not Finder-prettified Tauri DMG layout.

## Changed files

- `scripts/verify-local.sh`: release gate now builds `.app`, creates deterministic DMG with `hdiutil create`, then mounts/inspects it.
- `README.md`: documents deterministic internal DMG release gate wording.
- `Docs/release/macos-dmg-signing-notarization.md`: documents deterministic CLI-safe DMG behavior and avoids implying Finder layout automation.
- `.hermes/reviews/release-verification-deterministic-dmg/handoff.md`: this handoff.

## How to test

- `bash -n scripts/verify-local.sh`
- `npm run verify:release`
- Expected:
  - Rust tests pass.
  - Frontend build passes.
  - Tauri builds `src-tauri/target/release/bundle/macos/Zoid.app`.
  - `hdiutil create` creates `src-tauri/target/release/bundle/dmg/Zoid_0.1.0_aarch64.dmg` without Finder/AppleScript.
  - Script mounts the DMG read-only and verifies `Zoid.app`, `/Applications` symlink, `CFBundleName=Zoid`, `CFBundleIdentifier=com.mavoid.zoid`, and `CFBundleExecutable=zoid`.

## Tests run

- Initial `npm run verify:release`: FAILED/TIMED OUT after Tauri/create-dmg `Running bundle_dmg.sh`; Rust tests and frontend build had passed first. Left `/Volumes/dmg.avwLW8` interstitial mount, which was detached with `hdiutil detach -force`.
- `bash -n scripts/verify-local.sh`: PASS.
- Cleaned stale interstitial artifacts: `rm -f src-tauri/target/release/bundle/macos/rw.*.Zoid_*.dmg`.
- `npm run verify:release`: PASS.
  - Rust tests: 19 passed.
  - Frontend build: PASS.
  - Tauri app bundle build: PASS.
  - Deterministic DMG created: PASS.
  - DMG mounted read-only: PASS.
  - DMG Applications symlink present: PASS.
  - DMG contents verified: PASS.
  - Artifact app: `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid.app`
  - Artifact dmg: `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/dmg/Zoid_0.1.0_aarch64.dmg`

## Git info

- Branch: `main`
- Commit SHA, if committed: not committed yet
- Diff base: `HEAD` / `7a9841e Add visible user folder foundation`

## Frontend/backend/database notes

- Frontend: no product UI changes.
- Backend/native app: no Rust app behavior changes.
- Database: no schema/data changes.
- Release tooling: internal DMG is structurally verified but not Finder-prettified, signed, notarized, or public-distribution-ready.

## Reviewer focus areas

- Whether replacing Tauri's DMG bundling in `verify:release` with deterministic `hdiutil create` is acceptable for internal release verification.
- Shell safety: cleanup behavior, app path/DMG path, architecture/version naming, required commands.
- Verification fidelity: script still mounts and inspects a real DMG artifact rather than only checking file existence.
- Docs truthfulness: no claim that this is a signed/notarized or polished public distribution DMG.

## Fix cycle notes

Initial review returned `APPROVED` with optional follow-ups:

- trap-protect temporary DMG staging cleanup;
- require and validate the `/Applications` symlink target;
- optionally clarify deterministic wording.

Follow-ups implemented before finalizing:

- Added `trap cleanup_stage EXIT` around the temporary DMG staging directory and cleared it before the later mounted-DMG cleanup trap is installed.
- Added `readlink` prerequisite and now require the mounted DMG `/Applications` symlink to point exactly to `/Applications`.
- Re-ran `bash -n scripts/verify-local.sh && npm run verify:release`: PASS; Rust tests 19 passed, frontend build passed, Tauri app bundle built, deterministic DMG created, mounted read-only, Applications symlink verified, contents verified.

Ready for re-review.
