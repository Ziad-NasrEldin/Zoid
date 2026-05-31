# Spike B: Browser / WebView Feasibility

Date: 2026-05-31
Plan: `/Users/ziadnasreldin/Zoid/Docs/2026-05-31-zoid-implementation-plan-v1.md`
Result: Partial

## Goal

Prove whether Zoid's Browser workspace can be a work webview/capture workspace and determine which browser-verification claims are safe for the product architecture.

## Summary

Spike B partially passed.

Feasible:
- Work-tab abstraction
- URL/title/history metadata
- persistent capture records
- screenshot references through fallback mechanisms
- HTTP status evidence through external checks
- cookie/session primitives in browser-like engines

Not proven or risky:
- Native Tauri screenshot capture as a first-class WebView API
- robust console/error capture in production Tauri WebView
- embedded login/OAuth for providers that block webviews
- actual macOS WKWebView behavior through Tauri because no Tauri app exists yet and Rust/Cargo were not installed

## Environment Discovery

Commands:

```bash
cd /Users/ziadnasreldin/Zoid
pwd
node --version
npm --version
cargo --version
rustc --version
curl --version | head -n 1
sqlite3 --version
screencapture -h 2>&1 | head -n 2
git status --short
python3 - <<'PY'
from pathlib import Path
for p in Path('.').rglob('*'):
    if p.is_file() and p.name in ('package.json','Cargo.toml','tauri.conf.json','tauri.conf.json5'):
        print(p)
PY
```

Output summary:

```text
PWD=/Users/ziadnasreldin/Zoid
node=v26.0.0
npm=11.12.1
cargo/rustc=not found
curl=8.7.1
sqlite3=3.51.0
screencapture exists
git status: fatal: not a git repository
no package.json, Cargo.toml, or tauri.conf.* found
```

Tauri package check:

```bash
npm view @tauri-apps/api version dist-tags --json
```

Output:

```json
{
  "version": "2.11.0",
  "dist-tags": {
    "next": "2.0.1",
    "latest": "2.11.0"
  }
}
```

Docs checked:

```text
https://v2.tauri.app/reference/javascript/api/namespacewebview/
https://v2.tauri.app/reference/javascript/api/namespacewebviewwindow/
https://docs.rs/tauri/latest/tauri/webview/struct.WebviewBuilder.html
https://docs.rs/tauri/latest/tauri/webview/struct.WebviewWindowBuilder.html
https://docs.rs/tauri/latest/tauri/webview/struct.PageLoadPayload.html
https://docs.rs/tauri/latest/tauri/enum.RunEvent.html
```

All returned HTTP 200 except an attempted `WebviewUrl` docs URL, which returned 404.

## Tauri API Findings

From `@tauri-apps/api@2.11.0` and Tauri Rust docs:

Supported/available concepts:
- `WebviewWindow` can open remote URLs.
- WebView options include `url`, `userAgent`, `incognito`, `devtools`, `dataDirectory`, and `dataStoreIdentifier`.
- `dataStoreIdentifier` is relevant for macOS >=14 / iOS >=17 persistence.
- Rust APIs include `on_document_title_changed`, `on_navigation`, `on_page_load`, `initialization_script`, `eval`, `eval_with_callback`, `navigate`, `cookies`, `cookies_for_url`, `clear_all_browsing_data`, and `open_devtools`.

Not found as strong first-class APIs:
- screenshot capture
- arbitrary external HTTP status capture from webview resource requests
- robust production console capture

Important limitation:
- `on_web_resource_request` is currently for the Tauri URI protocol, not a reliable external-web status capture system.

## Prototype Artifacts

Temporary prototype only:

```text
/tmp/zoid-spike-b-webview-feasibility/
```

Created:

```text
/tmp/zoid-spike-b-webview-feasibility/package.json
/tmp/zoid-spike-b-webview-feasibility/spike_b_probe.js
/tmp/zoid-spike-b-webview-feasibility/out/browser_spike.sqlite
/tmp/zoid-spike-b-webview-feasibility/out/tab_001.png
/tmp/zoid-spike-b-webview-feasibility/out/tab_002.png
/tmp/zoid-spike-b-webview-feasibility/out/probe-result.json
/tmp/zoid-spike-b-webview-feasibility/out/persistent-browser-profile/
```

Commands:

```bash
cd /tmp/zoid-spike-b-webview-feasibility
npm init -y
npm install playwright@latest --silent
npx playwright --version
node /tmp/zoid-spike-b-webview-feasibility/spike_b_probe.js
```

Output summary:

```text
Playwright Version 1.60.0

Opened first work tab:
requestedUrl: https://example.com/
finalUrl: https://example.com/
title: Example Domain
status: 200
screenshot: /tmp/zoid-spike-b-webview-feasibility/out/tab_001.png
screenshot bytes: 17717

Opened second work tab:
requestedUrl: https://github.com/login
finalUrl: https://github.com/login
title: Sign in to GitHub · GitHub
status: 200
screenshot: /tmp/zoid-spike-b-webview-feasibility/out/tab_002.png
screenshot bytes: 40074

SQLite persisted:
browser_tabs rows: 2
browser_history rows: 2
browser_captures rows: 2

Cookie/session behavior:
session cookie existed before close, did not survive reopen
persistent cookie survived reopen:
name=zoid_spike_persistent, value=persistent_ok, domain=httpbin.org

Login-heavy probe:
https://accounts.google.com/ loaded and redirected to Google sign-in
finalUrl: accounts.google.com/v3/signin/identifier...
title: Sign in - Google Accounts
status: 200

Console/pageerror probe:
console event captured in Playwright: zoid_console_probe
pageerror captured in Playwright: zoid_pageerror_probe
```

## Requirement Findings

| Requirement | Result | Evidence / Notes |
|---|---:|---|
| Open multiple work tabs or credible first tab abstraction | Pass conceptually | Tauri supports Webview/WebviewWindow labels; Playwright prototype opened two tab records. For first build, single active work tab is safer. |
| Persist URL/title/history metadata | Pass | Tauri has navigation/title/page load hooks; prototype persisted URL/title/status/history in SQLite. |
| Capture screenshot if feasible | Partial | Playwright screenshot worked. No first-class Tauri screenshot API found; use OS capture/headless verification/custom WKWebView bridge if needed. |
| Save URL/title/screenshot reference to entity | Pass | Prototype saved capture rows with screenshot paths and metadata. |
| Login-heavy cookie/session behavior | Partial | Persistent cookie survived Playwright profile reopen. Tauri has data-store APIs, but must be verified in actual macOS Tauri app. OAuth webviews remain risky. |
| Console/error capture in Tauri WebView | Partial / risky | Playwright captured console/pageerror. Tauri has eval/init script/devtools, but robust production capture is not proven. |
| Fallback evidence with screenshot + URL + HTTP status | Pass | Prototype produced screenshots and status evidence. HTTP status should come from separate HTTP client/fetch, not WebView internals alone. |

## OAuth / Embedded WebView Policy Finding

Google OAuth policies warn against embedded user-agents/webviews. Therefore:

- Zoid should not promise embedded OAuth login for Google-like providers.
- Gmail OAuth should use system-browser OAuth flow.
- Tokens/credentials should be stored through the secure credential system, not browser session scraping.

## Unsupported / Risks

- No native Tauri app exists in the repo.
- Rust/Cargo is not installed, so native Tauri compile/run was impossible in this spike.
- No first-class Tauri screenshot API was found.
- Console capture is not robust enough for Launch Gate acceptance.
- Google/OAuth embedded webview flows are policy and UX risks.
- WebView cookie/session behavior must be validated in actual macOS Tauri WKWebView with `dataStoreIdentifier`.

## Stack Decision Impact

- Tauri remains viable for a work webview/capture workspace.
- Browser wording must remain scoped: “work webview/capture workspace,” not full personal browser.
- Launch Gate browser evidence should not depend on console capture.
- Safe verification evidence can include URL, external HTTP status, screenshot, route smoke result, and manual observation note.
- Follow-up native Tauri mini-spike is required after Rust/Tauri setup to validate WKWebView screenshot strategy and persistent `dataStoreIdentifier` behavior.
