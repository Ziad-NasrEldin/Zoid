#!/usr/bin/env bash
set -euo pipefail

usage() {
  cat <<'EOF'
Usage: scripts/verify-local.sh [--skip-package] [--install|--no-install] [--help]

Local-first verification and release gate for Zoid.

Options:
  --skip-package  Run fast push gate only: dependency check/install, Rust tests, frontend build.
  --install       Force npm ci before verification. Same as ZOID_VERIFY_INSTALL=1.
  --no-install    Do not run npm ci, even when node_modules is missing.
  --help          Show this help.

Environment:
  ZOID_VERIFY_INSTALL=1  Force npm ci.
  ZOID_VERIFY_NO_INSTALL=1  Disable npm ci.
EOF
}

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
cd "$REPO_ROOT"

SKIP_PACKAGE=0
FORCE_INSTALL=${ZOID_VERIFY_INSTALL:-0}
NO_INSTALL=${ZOID_VERIFY_NO_INSTALL:-0}

while [ "$#" -gt 0 ]; do
  case "$1" in
    --skip-package)
      SKIP_PACKAGE=1
      ;;
    --install)
      FORCE_INSTALL=1
      NO_INSTALL=0
      ;;
    --no-install)
      NO_INSTALL=1
      FORCE_INSTALL=0
      ;;
    -h|--help)
      usage
      exit 0
      ;;
    *)
      echo "ERROR: unknown option: $1" >&2
      usage >&2
      exit 2
      ;;
  esac
  shift
done

pass() {
  printf 'PASS: %s\n' "$1"
}

fail() {
  printf 'ERROR: %s\n' "$1" >&2
  exit 1
}

need_command() {
  command -v "$1" >/dev/null 2>&1 || fail "missing required command '$1'. Install it and re-run this local gate."
}

need_file() {
  [ -e "$1" ] || fail "missing required path: $1"
}

need_command npm
need_command cargo

if [ "$NO_INSTALL" != "1" ] && { [ "$FORCE_INSTALL" = "1" ] || [ ! -d node_modules ]; }; then
  echo "Running npm ci..."
  npm ci
  pass "npm dependencies installed"
elif [ "$NO_INSTALL" = "1" ]; then
  pass "skipped npm ci by request (--no-install or ZOID_VERIFY_NO_INSTALL=1)"
else
  pass "npm dependencies present; skipped npm ci"
fi

if [ -x "$REPO_ROOT/node_modules/.bin/tauri" ]; then
  pass "tauri CLI found at node_modules/.bin/tauri"
elif command -v tauri >/dev/null 2>&1; then
  pass "tauri CLI found on PATH"
else
  fail "missing Tauri CLI. Run npm ci or install @tauri-apps/cli so 'npm run tauri:build' can execute."
fi

if [ "$SKIP_PACKAGE" != "1" ]; then
  need_command hdiutil
  need_command ditto
  need_command readlink
  [ -x /usr/libexec/PlistBuddy ] || fail "missing required executable /usr/libexec/PlistBuddy"
fi

npm run test:rust
pass "Rust tests passed"

npm run build
pass "frontend build passed"

if [ "$SKIP_PACKAGE" = "1" ]; then
  pass "local push verification passed (--skip-package)"
  exit 0
fi

npm run tauri:build -- --bundles app
pass "Tauri app bundle build passed"

APP_PATH="src-tauri/target/release/bundle/macos/Zoid.app"
DMG_DIR="src-tauri/target/release/bundle/dmg"
need_file "$APP_PATH"
[ -d "$APP_PATH" ] || fail "expected app bundle is not a directory: $APP_PATH"
mkdir -p "$DMG_DIR"

VERSION=$(node -e "const fs=require('node:fs'); console.log(JSON.parse(fs.readFileSync('src-tauri/tauri.conf.json','utf8')).version)")
case "$(uname -m)" in
  arm64) ARCH="aarch64" ;;
  x86_64) ARCH="x64" ;;
  *) ARCH="$(uname -m)" ;;
esac

DMG_PATH="$DMG_DIR/Zoid_${VERSION}_${ARCH}.dmg"
DMG_STAGE=$(mktemp -d "${TMPDIR:-/tmp}/zoid-dmg-stage.XXXXXX")
cleanup_stage() {
  rm -rf "${DMG_STAGE:-}"
}
trap cleanup_stage EXIT

rm -f "$DMG_PATH"
ditto "$APP_PATH" "$DMG_STAGE/Zoid.app"
ln -s /Applications "$DMG_STAGE/Applications"
if ! hdiutil create -volname "Zoid" -fs HFS+ -srcfolder "$DMG_STAGE" -format UDZO -ov "$DMG_PATH" >/dev/null; then
  fail "failed to create deterministic DMG at $DMG_PATH"
fi
cleanup_stage
trap - EXIT
need_file "$DMG_PATH"
pass "deterministic DMG created without Finder AppleScript"
pass "app artifact exists: $REPO_ROOT/$APP_PATH"
pass "DMG artifact exists: $REPO_ROOT/$DMG_PATH"

MOUNT_POINT=$(mktemp -d "${TMPDIR:-/tmp}/zoid-dmg.XXXXXX")
DMG_ATTACHED=0
cleanup() {
  if [ -n "${MOUNT_POINT:-}" ] && [ -d "$MOUNT_POINT" ]; then
    # Always try to detach the mount point, even if hdiutil was interrupted
    # before DMG_ATTACHED was set. A detach against an unmounted temp directory
    # is harmless and helps avoid leaking a partial DMG mount.
    if [ "${DMG_ATTACHED:-0}" = "1" ] || mount | grep -F " on $MOUNT_POINT " >/dev/null 2>&1; then
      hdiutil detach "$MOUNT_POINT" >/dev/null 2>&1 || hdiutil detach -force "$MOUNT_POINT" >/dev/null 2>&1 || true
    else
      hdiutil detach "$MOUNT_POINT" >/dev/null 2>&1 || true
    fi
    rmdir "$MOUNT_POINT" >/dev/null 2>&1 || true
  fi
}
trap cleanup EXIT
trap 'cleanup; exit 130' INT
trap 'cleanup; exit 143' TERM

hdiutil attach "$DMG_PATH" -readonly -nobrowse -mountpoint "$MOUNT_POINT" >/dev/null
DMG_ATTACHED=1
pass "DMG mounted read-only for inspection"

MOUNTED_APP="$MOUNT_POINT/Zoid.app"
INFO_PLIST="$MOUNTED_APP/Contents/Info.plist"
APP_BINARY="$MOUNTED_APP/Contents/MacOS/zoid"

[ -d "$MOUNTED_APP" ] || fail "mounted DMG does not contain Zoid.app"
[ -f "$INFO_PLIST" ] || fail "mounted app is missing Contents/Info.plist"
[ -x "$APP_BINARY" ] || fail "mounted app binary is missing or not executable: Contents/MacOS/zoid"

[ -L "$MOUNT_POINT/Applications" ] || fail "DMG Applications entry is missing or is not a symlink"
[ "$(readlink "$MOUNT_POINT/Applications")" = "/Applications" ] || fail "DMG Applications symlink does not point to /Applications"
pass "DMG Applications symlink present"

plist_value() {
  /usr/libexec/PlistBuddy -c "Print :$1" "$INFO_PLIST"
}

[ "$(plist_value CFBundleName)" = "Zoid" ] || fail "CFBundleName is not Zoid"
[ "$(plist_value CFBundleIdentifier)" = "com.mavoid.zoid" ] || fail "CFBundleIdentifier is not com.mavoid.zoid"
[ "$(plist_value CFBundleExecutable)" = "zoid" ] || fail "CFBundleExecutable is not zoid"

pass "DMG contents verified"
pass "local release verification passed"
printf 'ARTIFACT app: %s\n' "$REPO_ROOT/$APP_PATH"
printf 'ARTIFACT dmg: %s\n' "$REPO_ROOT/$DMG_PATH"
