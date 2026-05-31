# macOS DMG signing and notarization

This document records the release-operations path for shipping Zoid as a macOS DMG.

## Current state

- CI and local release builds currently produce unsigned DMGs from Tauri.
- Unsigned and non-notarized DMGs are acceptable for internal verification, smoke testing, and artifact review.
- Unsigned and non-notarized DMGs are not final external distribution artifacts. Public macOS distribution should use a Developer ID-signed and notarized build.

## Future release checklist

1. Enroll the release owner or organization in the Apple Developer Program.
2. Create and protect a Developer ID Application certificate for the Apple Team used to ship Zoid.
3. Configure Tauri/macOS signing and notarization in CI or the release environment. Use repository or CI secrets; do not commit secret values.

   Required secret placeholders:

   - `APPLE_ID=<apple-id@example.com>`
   - `APPLE_PASSWORD=<app-specific-password>`
   - `APPLE_TEAM_ID=<TEAMID1234>`
   - `APPLE_CERTIFICATE=<base64-encoded-developer-id-application-certificate>`
   - `APPLE_CERTIFICATE_PASSWORD=<certificate-password>`

   Tauri 2 macOS bundle configuration may also need explicit signing settings such as a Developer ID signing identity, hardened runtime, entitlements, and minimum macOS version in `src-tauri/tauri.conf.json`, depending on the final release setup. Keep those settings in source only when they are non-secret; keep certificates, passwords, Apple IDs, and app-specific passwords in CI/local secret storage.

4. Build the release artifact:

   ```sh
   npm run tauri:build
   ```

5. Locate the generated DMG, typically under:

   ```text
   src-tauri/target/release/bundle/dmg/
   ```

6. For current unsigned internal artifacts, smoke check the DMG and `.app` bundle without expecting Gatekeeper, signing, or notarization validation to pass. Use a temporary mount point and cleanup trap so local checks do not leave volumes mounted. Replace the DMG path with the artifact produced by the build.

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

   test -d "$APP"
   test -x "$APP/Contents/MacOS/Zoid"
   /usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$APP/Contents/Info.plist"
   /usr/libexec/PlistBuddy -c 'Print :CFBundleShortVersionString' "$APP/Contents/Info.plist"
   ```

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
- Use CI secret storage for automated release builds.
- Use local Keychain or environment variables for one-off local release builds, and clear shell history/environment files if a secret was accidentally written there.
