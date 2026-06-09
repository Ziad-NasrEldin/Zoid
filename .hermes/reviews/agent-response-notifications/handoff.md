# Agent Response Notifications — Critique Handoff

## Scope
Implement notifications when a Hermes agent responds in a session that is not currently open.

Requested behavior:
- Show a design-system-aligned needs-reply indicator above the inactive session box.
- Send macOS desktop notifications through Tauri notification support.
- Send email notifications to ziad.ahmed.25.25.25@gmail.com with a summarized version of the conversation.

## Implementation summary
- Added `src/agents/agentNotifications.ts` with:
  - Tauri notification permission/request/send wrapper using `@tauri-apps/plugin-notification`.
  - Bounded email summary construction.
  - Tauri invoke wrapper for backend email command.
- Updated `src/agents/AgentsHermesScreen.tsx` with:
  - `needsReply`, `lastNotifiedAssistantMessageId`, `notificationUpdatedAt` on `HermesChatSession`.
  - `activeSessionIdRef` and `notifyForBackgroundAgentResponse` so notifications fire only when the response lands in a non-open session.
  - `openSession` clears the needs-reply state when a session is opened.
  - Per-session `BellDot` indicator with label/title `Hermes replied and needs your reply`.
- Updated `src/App.tsx` session validation/restore and global Agents nav notification source to use persisted `needsReply`.
- Added design-system styling in `src/App.css` for `.session-reply-indicator` and compact session rail behavior.
- Added frontend dependency `@tauri-apps/plugin-notification`.
- Added Rust dependency `tauri-plugin-notification`, Tauri plugin initialization, and default capability permission.
- Added backend command `send_agent_response_email_notification` in `src-tauri/src/lib.rs`.
  - SMTP config is backend-only through env vars: `ZOID_NOTIFY_SMTP_HOST`, `ZOID_NOTIFY_SMTP_PORT`, `ZOID_NOTIFY_SMTP_USERNAME`, `ZOID_NOTIFY_SMTP_PASSWORD`, optional `ZOID_NOTIFY_EMAIL_FROM`, optional `ZOID_NOTIFY_EMAIL_TO`.
  - Email defaults to `ziad.ahmed.25.25.25@gmail.com`.
  - SMTP send uses `python3` + `smtplib` with secrets in child env, not frontend code or command-line args.
- Added scaffold guards in `src/scaffold.test.ts` for notification client/backend/UI requirements.

## Files intentionally touched for this feature
- `src/agents/agentNotifications.ts`
- `src/agents/AgentsHermesScreen.tsx`
- `src/App.tsx`
- `src/App.css`
- `src/scaffold.test.ts`
- `src-tauri/src/lib.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/capabilities/default.json`
- `package.json`
- `package-lock.json`

## Known repository state
The repo has a broad dirty tree with many pre-existing unrelated modified/untracked files and review directories. Do not judge unrelated files as part of this feature unless they are listed above.

## Post-critique fixes applied
- Fixed background detection to account for workspace visibility:
  - `App.tsx` passes `isAgentsWorkspaceOpen={activeWorkspace === "Agents"}`.
  - `AgentsHermesScreen.tsx` keeps `isAgentsWorkspaceOpenRef` and suppresses notifications only when the Agents workspace is open and the responding session is selected.
  - The visibility ref cleanup sets `isAgentsWorkspaceOpenRef.current = false` on unmount so an in-flight selected-session response that resolves after leaving Agents is treated as not currently open.
- Added backend command-boundary bounding:
  - `bounded_email_header` caps subject/session title.
  - `bounded_email_body` caps summary/body to 16 KiB before sending.
- Updated scaffold guards for both fixes.

## Validation already run
- `npm test -- --runInBand` passed after the post-critique fixes.
  - Frontend scaffold/behavior tests passed.
  - Rust tests passed: 66 passed, 1 ignored.
- `npm run build` passed after the post-critique fixes.
- `npm run tauri:build` passed after the post-critique fixes.
- Rebuilt app bundle was copied to `/Applications/Zoid 25.app` after the post-critique fixes.
- Launched via bundle id `com.mavoid.zoid25`.
- Screenshot evidence showed the installed app running on the Agents/Hermes workspace after relaunch.

## Review focus
Please inspect for:
- Whether background response detection is correct and cannot notify for the currently open session.
- Whether needs-reply state is cleared at the right time.
- Whether the visual indicator is positioned/accessibly labelled and follows the existing Zoid design style.
- Whether desktop notifications are using Tauri correctly.
- Whether email summary/content is bounded enough and credentials stay backend-only.
- Whether backend SMTP implementation has reliability/security issues.
- Whether adding notification dependencies/permissions is correct.

## Verdict required
Return one of: APPROVED, REQUEST_CHANGES, BLOCKED_NEEDS_HUMAN.
For REQUEST_CHANGES, include required fixes only, with exact file/line guidance where possible.
