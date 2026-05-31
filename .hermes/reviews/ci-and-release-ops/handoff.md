# Feature Handoff: CI and macOS release operations

## Original request

After the Zoid local/GitHub repo and DMG packaging work was approved, the remaining non-blocking next steps were:

- Add CI for Rust/build/Tauri packaging.
- Decide/document signing/notarization before external DMG distribution.

The user then said: "do that the, you can use sub agent driven development to finsih up the work".

## Implementation summary

Used subagent-driven development:

1. CI implementer subagent added the GitHub Actions workflow.
2. Release-docs implementer subagent added macOS signing/notarization documentation and README release notes.
3. Spec compliance reviewer subagent passed the first implementation.
4. Quality reviewer subagent requested changes for missing Linux Tauri deps, missing artifact upload, sequencing, and release-doc clarity.
5. Fix subagent addressed all required quality-review blockers.
6. Spec compliance and quality reviewer subagents re-reviewed and approved.

What was built:

- Added GitHub Actions CI workflow at `.github/workflows/ci.yml`.
- Added a `verify` job for source checks:
  - Ubuntu runner.
  - Installs Linux Tauri system dependencies.
  - Uses Node 22.
  - Installs Rust stable.
  - Runs `npm ci`, `npm run test:rust`, `npm run build`.
- Added a `package-macos` job:
  - macOS runner.
  - `needs: verify` so packaging waits for source verification.
  - Runs `npm ci` and `npm run tauri:build`.
  - Uploads generated `.dmg` and `.app` artifacts with `actions/upload-artifact@v4`.
- Added internal release operations doc for macOS DMG signing/notarization.
- Updated README with development commands and Release / CI section.

## Changed files

- `.github/workflows/ci.yml`: new GitHub Actions workflow for verify + macOS package jobs.
- `Docs/release/macos-dmg-signing-notarization.md`: new internal release checklist for unsigned internal DMG checks and future signed/notarized external distribution.
- `README.md`: updated from starter template to Zoid-specific development and Release / CI notes.
- `.hermes/reviews/ci-and-release-ops/handoff.md`: this handoff.

## How to test

From `/Users/ziadnasreldin/Zoid`:

```bash
python3 - <<'PY'
from pathlib import Path
import re
s=Path('.github/workflows/ci.yml').read_text()
checks={
 'trigger_pull_request': re.search(r'^on:\n(?:.|\n)*?pull_request:', s, re.M) is not None,
 'push_main': 'push:' in s and 'main' in s,
 'verify_job': re.search(r'^  verify:', s, re.M) is not None,
 'linux_deps': 'libwebkit2gtk-4.1-dev' in s and 'patchelf' in s,
 'npm_ci': 'run: npm ci' in s,
 'rust_tests': 'run: npm run test:rust' in s,
 'frontend_build': 'run: npm run build' in s,
 'package_macos': re.search(r'^  package-macos:', s, re.M) is not None,
 'needs_verify': 'needs: verify' in s,
 'tauri_build': 'run: npm run tauri:build' in s,
 'upload_artifact': 'actions/upload-artifact@v4' in s,
}
for k,v in checks.items(): print(f'{k}={v}')
if not all(checks.values()): raise SystemExit(1)
PY

npm run test:rust
npm run build
npm run tauri:build
```

Optional DMG artifact check:

```bash
DMG='src-tauri/target/release/bundle/dmg/Zoid_0.1.0_aarch64.dmg'
MNT=$(mktemp -d /tmp/zoid-ci-release-dmg.XXXXXX)
cleanup(){ hdiutil detach "$MNT" -quiet >/dev/null 2>&1 || true; rmdir "$MNT" >/dev/null 2>&1 || true; }
trap cleanup EXIT
hdiutil attach "$DMG" -mountpoint "$MNT" -nobrowse -readonly -quiet
find "$MNT" -maxdepth 2 -print | sort
/usr/libexec/PlistBuddy -c 'Print :CFBundleName' "$MNT/Zoid.app/Contents/Info.plist"
/usr/libexec/PlistBuddy -c 'Print :CFBundleIdentifier' "$MNT/Zoid.app/Contents/Info.plist"
test -x "$MNT/Zoid.app/Contents/MacOS/zoid"
```

## Tests run

- Workflow structural check: PASS
  - `trigger_pull_request=True`
  - `push_main=True`
  - `verify_job=True`
  - `linux_deps=True`
  - `npm_ci=True`
  - `rust_tests=True`
  - `frontend_build=True`
  - `package_macos=True`
  - `needs_verify=True`
  - `tauri_build=True`
  - `upload_artifact=True`
- Release doc placeholder check: PASS
  - `APPLE_ID=<apple-id@example.com>` present.
  - `APPLE_PASSWORD=<app-specific-password>` present.
  - `APPLE_TEAM_ID=<TEAMID1234>` present.
  - `APPLE_CERTIFICATE=<base64-encoded-developer-id-application-certificate>` present.
  - `APPLE_CERTIFICATE_PASSWORD=<certificate-password>` present.
- `npm run test:rust`: PASS
  - 3 Rust tests passed, 0 failed.
- `npm run build`: PASS
  - TypeScript + Vite production build passed.
- `npm run tauri:build`: PASS
  - Built app binary.
  - Bundled `.app`.
  - Bundled `.dmg`.
- Artifact check: PASS
  - `src-tauri/target/release/bundle/macos/Zoid.app` exists.
  - `src-tauri/target/release/bundle/dmg/Zoid_0.1.0_aarch64.dmg` exists.
  - app binary size: `10409168` bytes.
  - dmg size: `3992473` bytes.
- DMG mount/identity check: PASS
  - Mounted at `/tmp/zoid-ci-release-dmg.v06ThW`.
  - Mounted volume contained `Zoid.app` and `Applications` symlink.
  - `CFBundleName=Zoid`.
  - `CFBundleIdentifier=com.mavoid.zoid`.
  - Mounted app binary executable: yes.

## Subagent review results

Spec compliance re-review:

- Verdict: PASS.
- No files modified by reviewer.
- Confirmed workflow triggers, jobs, commands, artifact upload, no signing secrets, release doc placeholders, robust cleanup, and README link/state.

Quality re-review:

- Verdict: APPROVED.
- Critical issues: none.
- Important issues: none.
- Minor notes:
  - `actionlint` was not available locally; reviewer used structural/manual checks.
  - Generic YAML parsers may treat `on:` as boolean under YAML 1.1, but GitHub Actions accepts the syntax.

## Git info

- Branch: `main`
- Base before this slice: `98b7e00 docs: add git and DMG packaging review evidence`
- Working tree before critique: modified/untracked files only in this slice:
  - `M README.md`
  - `?? .github/`
  - `?? Docs/release/`
  - `?? .hermes/reviews/ci-and-release-ops/`

## Frontend/backend/database notes

- Frontend runtime code unchanged.
- Native/Tauri runtime code unchanged.
- Database schema unchanged.
- CI and documentation only, with local native packaging verification rerun to ensure the documented workflow still matches the project.

## Reviewer focus areas

- Validate GitHub Actions syntax and job practicality.
- Validate Ubuntu dependencies for Tauri/Rust checks.
- Validate macOS packaging sequencing and artifact upload paths.
- Validate release documentation does not imply unsigned artifacts are ready for public distribution.
- Validate docs contain placeholders only, not secrets.

## Known limitations / risks

- The GitHub Actions workflow itself cannot be fully proven until committed/pushed and run on GitHub-hosted runners.
- Current DMGs remain unsigned/non-notarized and are for internal verification only.
- `actionlint` was not installed locally; structural/manual workflow checks were used.
