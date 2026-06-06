# macOS DMG signing and notarization

This document records the release-operations path for shipping Zoid as a macOS DMG.

## Current state

- GitHub Actions/CI/CD is unavailable for this GitHub account: workflows fail before jobs start with `startup_failure`.
- Local verification scripts are the source of truth for build, packaging, and release-gate checks.
- Local release builds currently produce unsigned internal DMGs from the verified Tauri `.app` bundle using deterministic `hdiutil create` packaging.
- Unsigned and non-notarized DMGs are acceptable for internal verification, smoke testing, and artifact review.
- Unsigned and non-notarized DMGs are not final external distribution artifacts. Public macOS distribution should use a Developer ID-signed and notarized build.

## Local release gate

Run the full local release gate only when intentionally creating or refreshing a DMG for internal review:

```sh
npm run verify:release
```

This script runs Rust tests, the frontend build, Tauri `.app` packaging, deterministic DMG creation, and DMG/app inspection. It mounts the generated DMG read-only at a temporary mount point, verifies `Zoid.app`, checks key `Info.plist` values, verifies the app executable, and detaches the DMG during cleanup. The internal verification DMG intentionally avoids Finder/AppleScript layout automation so the CLI gate cannot hang waiting for Finder metadata.

Normal development verification deliberately skips macOS packaging:

```sh
npm run verify:local
```

For faster pre-push verification without packaging, this equivalent command is also available:

```sh
npm run verify:push
```

To install the local-only pre-push hook in this checkout:

```sh
npm run hooks:install
```

The hook is written to `.git/hooks/pre-push`, is not committed, and runs `scripts/verify-local.sh --skip-package` before push.

## Future public release checklist

1. Enroll the release owner or organization in the Apple Developer Program.
2. Create and protect a Developer ID Application certificate for the Apple Team used to ship Zoid.
3. Configure Tauri/macOS signing and notarization in the local release environment or another available release runner. Use secret storage; do not commit secret values. If GitHub Actions becomes available in the future, repository/CI secrets may be used there, but GitHub Actions is not currently an active release path for this account.

   Required secret placeholders:

   - `APPLE_ID=<apple-id@example.com>`
   - `APPLE_PASSWORD=<app-specific-password>`
   - `APPLE_TEAM_ID=<TEAMID1234>`
   - `APPLE_CERTIFICATE=<base64-encoded-developer-id-application-certificate>`
   - `APPLE_CERTIFICATE_PASSWORD=<certificate-password>`

   Tauri 2 macOS bundle configuration may also need explicit signing settings such as a Developer ID signing identity, hardened runtime, entitlements, and minimum macOS version in `src-tauri/tauri.conf.json`, depending on the final release setup. Keep those settings in source only when they are non-secret; keep certificates, passwords, Apple IDs, and app-specific passwords in local secret storage.

4. Build and verify the local release artifact:

   ```sh
   npm run verify:release
   ```

5. Locate the generated DMG, typically under:

   ```text
   src-tauri/target/release/bundle/dmg/
   ```

6. For current unsigned internal artifacts, use `npm run verify:release` as the required smoke check. It verifies the DMG and `.app` bundle without expecting Gatekeeper, signing, or notarization validation to pass.

7. For signed and notarized release candidates, validate code signing, Gatekeeper assessment, and notarization/stapling. These checks are expected to pass only after the app or DMG has been signed, notarized, and stapled as appropriate for the release flow.

   ```sh
   DMG="src-tauri/target/release/bundle/dmg/Zoid_<version>_<arch>.dmg"
   MOUNT_POINT=$(mktemp -d "${TMPDIR:-/tmp}/zoid-dmg.XXXXXX")
   cleanup() {
     hdiutil detach "$MOUNT_POINT" >/dev/null 2>&1 || true
     rmdir "$MOUNT_POINT" >/dev/null 2>&1 || true
   }
   trap cleanup EXIT

   hdiutil attach "$DMG" -readonly -nobrowse -mountpoint "$MOUNT_POINT"
   APP="$MOUNT_POINT/Zoid.app"

   codesign --verify --deep --strict --verbose=2 "$APP"
   spctl --assess --type execute --verbose "$APP"
   xcrun stapler validate "$APP"
   # or validate the distributed DMG if that is the stapled artifact:
   xcrun stapler validate "$DMG"
   ```

8. Publish only artifacts that pass build, signing, notarization, and verification gates.

## Secret handling

- Never commit Apple credentials, app-specific passwords, signing certificates, certificate passwords, or exported keychains.
- Do not rely on GitHub Actions secrets while CI/CD is disabled for this account.
- Use local Keychain, a local password manager, or environment variables for one-off local release builds, and clear shell history/environment files if a secret was accidentally written there.
