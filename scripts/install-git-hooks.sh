#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR=$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)
REPO_ROOT=$(CDPATH= cd -- "$SCRIPT_DIR/.." && pwd)
cd "$REPO_ROOT"

[ -d .git ] || {
  echo "ERROR: scripts/install-git-hooks.sh must be run from a Git checkout." >&2
  exit 1
}

HOOK_DIR=".git/hooks"
HOOK_PATH="$HOOK_DIR/pre-push"
MANAGED_MARKER="# Zoid-managed pre-push hook (scripts/install-git-hooks.sh)"
mkdir -p "$HOOK_DIR"

if [ -e "$HOOK_PATH" ]; then
  if grep -F "$MANAGED_MARKER" "$HOOK_PATH" >/dev/null 2>&1; then
    echo "Updating existing Zoid-managed pre-push hook at $REPO_ROOT/$HOOK_PATH"
  else
    BACKUP_PATH="$HOOK_PATH.backup.$(date +%Y%m%d%H%M%S)"
    cp -p "$HOOK_PATH" "$BACKUP_PATH"
    echo "Existing unmanaged pre-push hook backed up to $REPO_ROOT/$BACKUP_PATH"
  fi
fi

cat > "$HOOK_PATH" <<'HOOK'
#!/usr/bin/env bash
# Zoid-managed pre-push hook (scripts/install-git-hooks.sh)
set -euo pipefail

REPO_ROOT=$(git rev-parse --show-toplevel)
cd "$REPO_ROOT"

echo "Running local-only Zoid pre-push gate: scripts/verify-local.sh --skip-package"
scripts/verify-local.sh --skip-package
HOOK

chmod +x "$HOOK_PATH"

echo "Installed local-only Git pre-push hook at $REPO_ROOT/$HOOK_PATH"
echo "The hook is not committed and only affects this checkout. It runs: scripts/verify-local.sh --skip-package"
