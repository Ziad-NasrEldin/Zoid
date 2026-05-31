# Spike C: Native macOS Services Feasibility

Date: 2026-05-31
Plan: `/Users/ziadnasreldin/Zoid/Docs/2026-05-31-zoid-implementation-plan-v1.md`
Result: Partial

## Goal

Prove key native macOS integrations before building dependent Zoid features.

Required capabilities:
- Keychain write/read/delete
- native notification with click/open route if feasible
- EventKit permission/read upcoming/create event after explicit confirmation
- reveal file in Finder/open file behavior
- app support and visible folder creation

## Summary

Spike C partially passed.

Passed:
- Swift/macOS toolchain availability
- app support folder creation
- visible Zoid folder creation
- Finder reveal/open file behavior
- basic native notification via `osascript`

Not fully proven:
- Keychain write/read/delete was blocked by execution guard and not retried.
- EventKit permission/read/create did not complete in CLI/temp app context.
- notification click/deep-link route was not proven because no Zoid app URL scheme is registered yet.

## Environment Discovery

Commands:

```bash
xcrun swift --version 2>&1 || swift --version 2>&1
command -v terminal-notifier || true
command -v security
command -v osascript
command -v open
```

Output summary:

```text
swift-driver version: 1.148.6 Apple Swift version 6.3.2
Target: arm64-apple-macosx26.0
/usr/bin/security
/usr/bin/osascript
/usr/bin/open
```

`terminal-notifier` was not installed.

## Prototype Artifacts

Temporary artifacts:

```text
/tmp/zoid-spike-c/eventkit_probe.swift
/tmp/zoid-spike-c/EventKitProbeApp/Contents/Info.plist
/tmp/zoid-spike-c/EventKitProbeApp/Contents/MacOS/EventKitProbe
/tmp/zoid-spike-c/reveal-open-test.txt
/tmp/zoid-spike-c/custom-url-open.out
```

Created expected Zoid folders:

```text
/Users/ziadnasreldin/Library/Application Support/Zoid/SpikeC
/Users/ziadnasreldin/Zoid/SpikeC
```

## Requirement Findings

### Keychain write/read/delete

Attempted flow:

```bash
security add-generic-password / find-generic-password / delete-generic-password
```

Tool result:

```text
BLOCKED: User denied this command. The user has NOT consented to this action. Do NOT retry this command, do NOT rephrase it, and do NOT attempt the same outcome via a different command.
```

Result: Not verified.

Impact:
- Keychain remains an unresolved blocker for credential storage.
- Must be rerun with explicit user approval or implemented/tested from the native app/Tauri command context.
- Do not implement OAuth/API credential storage until this is proven.

### App support and visible folder creation

Command:

```bash
mkdir -p "$HOME/Library/Application Support/Zoid/SpikeC" "$HOME/Zoid/SpikeC"
printf checks...
```

Output:

```text
app_support_exists=yes
visible_folder_exists=yes
```

Result: Pass.

### Reveal file in Finder / open file behavior

Command:

```bash
TMP=/tmp/zoid-spike-c/reveal-open-test.txt
printf ... > "$TMP"
open -R "$TMP"
open "$TMP"
```

Output:

```text
temp_file=/tmp/zoid-spike-c/reveal-open-test.txt exists=yes
open_reveal_exit=0
open_file_exit=0
```

Result: Pass.

### Native notification and route/open behavior

Command:

```bash
osascript -e 'display notification "Native notification probe from Zoid Spike C" with title "Zoid Spike C" subtitle "macOS notification" sound name "Glass"'
open 'file:///tmp/zoid-spike-c/reveal-open-test.txt'
open 'zoid://spike-c/notification-click'
```

Output:

```text
osascript_display_notification_exit=0
open_file_url_route_exit=0
custom_url_open_exit=1 output=No application knows how to open URL zoid://spike-c/notification-click ... kLSApplicationNotFoundErr
```

Result:
- Basic native notification: Pass.
- File URL open route: Pass.
- Custom `zoid://` route: Not configured/proven.
- Notification click callback: Not proven with `osascript`; needs app bundle registration and notification delegate/Tauri plugin path.

### EventKit permission/read/create dry-run

Temp Swift probe:

```text
/tmp/zoid-spike-c/eventkit_probe.swift
```

Temp app bundle shell:

```text
/tmp/zoid-spike-c/EventKitProbeApp/Contents/Info.plist
/tmp/zoid-spike-c/EventKitProbeApp/Contents/MacOS/EventKitProbe
```

Commands:

```bash
xcrun swift /tmp/zoid-spike-c/eventkit_probe.swift
swiftc /tmp/zoid-spike-c/eventkit_probe.swift -o /tmp/zoid-spike-c/EventKitProbeApp/Contents/MacOS/EventKitProbe
/tmp/zoid-spike-c/EventKitProbeApp/Contents/MacOS/EventKitProbe
```

Output:

```text
permission_wait=timed_out
permission_granted=false
authorization_status=0
upcoming_events_7d_count=blocked_no_permission
create_event_dry_run_constructed=blocked_no_permission
create_event_saved=false
```

Result:
- Swift EventKit imports/compiles: Pass.
- Runtime permission/read/create: Blocked/unproven.
- No real calendar event was created.

Interpretation:
- `authorization_status=0` means not determined.
- Permission callback timed out in CLI/temp bundle context.
- EventKit must be tested from a properly bundled signed macOS/Tauri app with the required calendar usage descriptions and foreground UI.

## Unsupported / Risks

- Keychain not proven because the guarded command was denied.
- EventKit permission/read/create not proven in CLI context.
- Notification click route/custom URL not proven.
- `osascript` notification success does not prove in-app notification click handling.

## Stack Decision Impact

Tauri + native layer remains plausible for:
- app support folder management
- visible `~/Zoid` folder management
- Finder reveal/open file behavior
- basic native notifications

Before dependent features, Zoid still needs focused native-app-context verification for:
- Keychain write/read/delete
- EventKit permission/read/create from real bundled app
- notification click/deep-link routing with registered app URL scheme or Tauri notification/deep-link plugin

Until those are proven:
- Credential storage is blocked/unproven.
- Apple Calendar integration is blocked/unproven.
- Native notification click routing is blocked/unproven.
