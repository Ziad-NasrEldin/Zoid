# Phase 7 — Browser Workspace and Advanced Widgets

Date: 2026-06-05
Worktree: `/Users/ziadnasreldin/Zoid-phase7-plus`

## Scope decision

Phase 7 follows `Docs/spikes/2026-05-31-browser-webview-spike.md` and implements a work webview/capture workspace abstraction, not a full personal browser.

Included:
- http(s) work URL records and saved tab/page state.
- metadata fallback captures with URL, title, timestamp, optional HTTP status, manual note, screenshot support flag, screenshot reference field, and entity attachments.
- capture attachment to Launch Gate, Task, Note, Product, and Content Piece targets.
- browser/widget events and SQLite persistence.
- widget visibility/order/size persistence and reset.
- truthful unsupported states for screenshot/native WebView limits and login-heavy sites.

Excluded / unsupported in this phase:
- full personal browser replacement.
- extensions, browser sync, password manager, cookie jar management, or credential storage.
- robust production console capture.
- guaranteed embedded OAuth/login-heavy behavior.
- first-class Tauri screenshot capture; the implemented capture mode is `metadata_fallback` unless a future native screenshot path is explicitly proven.

## Browser flow

1. User enters an http(s) work URL.
2. Backend stores/updates a `browser_tabs` record (`open`, `saved`, `closed`, `blocked`, or `unsupported`).
3. Capture action stores a `browser_captures` row with fallback evidence: URL, title, timestamp, HTTP status if supplied, manual note, metadata JSON, `screenshot_supported=false`, `capture_mode=metadata_fallback`.
4. Attachment action stores `browser_capture_links` and a generic `entity_links` row from `browser_capture` to the selected Launch Gate/Task/Note/Product/ContentPiece target.
5. Events are written for browser opened/updated/capture-created/capture-attached and widget config changed/reset.

## Security/privacy behavior

- Non-http(s) URLs are rejected.
- Secret-looking URL query parameters such as token/password/cookie/session/api_key are redacted before persistence.
- Title/manual note/metadata strings are passed through existing secret redaction before persistence.
- No raw cookies, auth headers, browser storage, or credential payloads are stored.
- HTTP status command is truthful: returns `None` rather than pretending WebView network inspection exists.

## Widget behavior

Allowed widget keys:
- `today_tasks`
- `active_runs`
- `blockers`
- `completions`
- `browser_captures`
- `launch_gate_evidence`
- `content_queue`

Allowed sizes: `small`, `medium`, `large`.

Widget configs are keyed by `workspace_key + profile_key + widget_key`, persist in SQLite, and reset to visible/medium/default ordering.

## Manual/source verification evidence

Automated native-command tests cover the manual requirements that can be verified without WebView click automation:
- normal work URL persists after reopen through `p711_p720_browser_capture_metadata_attachment_events_and_redaction_are_fail_closed` and `p719_p735_p736_widget_configs_validate_persist_reset_and_emit_events`.
- login-heavy sites are documented as partial/blocked because embedded provider login/OAuth behavior is not proven in Tauri.
- screenshot capture is documented as unsupported in this slice; fallback capture is verified.
- Launch Gate attachment is verified by `browser_capture_links` plus generic `entity_links` rows targeting `launch_gate`.
- widget config survives restart in the file-backed SQLite test.

## Verification commands

Passed locally after implementation:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p7 -- --nocapture
npm run test:frontend
npm run verify:local
```
