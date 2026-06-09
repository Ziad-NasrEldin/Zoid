# Agent Response Notifications Plan

## Goal

Add Zoid/Hermes notification support when an agent finishes responding in a session that is not currently open:

1. Show a design-system-aligned "needs reply" indicator above/on each affected session box in the Sessions rail.
2. Trigger macOS desktop notifications from the Tauri app.
3. Send an email notification to `ziad.ahmed.25.25.25@gmail.com` containing a summarized version of the conversation.

## Current context / assumptions

- Target app: `/Users/ziadnasreldin/Zoid`.
- App is a Tauri v2 + React/Vite desktop app.
- Hermes chat UI is mainly in `src/agents/AgentsHermesScreen.tsx`.
- Sessions are currently persisted in browser `localStorage` from `src/App.tsx` using `zoid25:hermes-sessions`.
- Existing global Agents navigation already has a red notification dot concept via:
  - `src/App.tsx`: `hasHermesWaitingNotification`
  - `src/App.css`: `.session-notification-dot`
- Per-session session-card unread/reply state does not appear to exist yet.
- macOS notification support is not currently registered; `src-tauri/Cargo.toml` only has opener, dialog, window-state plugins.
- Email cannot safely be sent from frontend code. It should be done in Rust/Tauri with SMTP credentials stored outside the bundled frontend.
- Email provider/config is an implementation blocker unless existing SMTP credentials already exist. Plan should support env/config-driven SMTP and fail visibly but non-blockingly if not configured.

## Proposed approach

Create a single notification pipeline at the point where Hermes changes an assistant message from `streaming` to `sent` or completes a slash-command response.

The pipeline should classify a notification as needed only when:

- The completed response belongs to a session whose id is not the current `activeSessionId` at completion time, OR the user has navigated away before the response returns.
- The response is a successful assistant response that needs user attention.
- Do not notify for the initial welcome message, active visible session responses, stopped runs, archive/delete actions, or empty internal panel events unless they produce user-facing assistant content.

The state model should be explicit, not inferred only from message text:

- Add per-session notification metadata to `HermesChatSession`, such as:
  - `needsReply?: boolean`
  - `lastNotifiedAssistantMessageId?: string`
  - `lastReadAssistantMessageId?: string`
  - optional `notificationUpdatedAt?: string`
- Clear `needsReply` when the user opens that session.
- Keep the existing global Agents nav dot but derive it from any session with `needsReply === true`.
- Add a per-session badge/icon in each `.session-tab-row`, positioned above/on the session box and styled with existing Kujo/Zoid tokens.

For desktop notifications:

- Add Tauri notification plugin support.
- Request/check notification permission from the frontend before first send.
- Display title/body like:
  - Title: `Hermes replied in <session title>`
  - Body: first compact line of response or summary preview.
- Clicking notification can be a follow-up enhancement if supported cleanly; core requirement is alerting.

For email notifications:

- Add a backend Tauri command such as `send_agent_response_email_notification`.
- Generate a safe deterministic conversation summary locally first, without a second paid model call:
  - session title
  - latest user prompt
  - latest assistant response short summary/excerpt
  - conversation status: "Hermes replied and may need your reply"
  - include no secrets; truncate large messages.
- Use SMTP through backend Rust, with config loaded from env vars or a local Zoid notification settings file outside frontend bundle.
- Send to fixed recipient `ziad.ahmed.25.25.25@gmail.com` by default, but keep the recipient in a constant/config field so it is testable.
- Email sending must be best-effort: if it fails, do not break the chat response; record a visible non-blocking status/log.

## Step-by-step plan

### 1. Add live-state verification gate before implementation

Before implementing, verify current repo state so stale assumptions do not pass review:

- Run `git status --short` in `/Users/ziadnasreldin/Zoid`.
- Confirm `src/agents/AgentsHermesScreen.tsx`, `src/App.tsx`, `src/App.css`, `src-tauri/Cargo.toml`, `src-tauri/src/lib.rs`, and `src-tauri/capabilities/default.json` still match the notification architecture above.
- Confirm existing tests still include `src/scaffold.test.ts` notification-dot checks.
- If session model or notification dot implementation has changed, update this plan before coding.

### 2. Model per-session "needs reply" state

Update `src/agents/AgentsHermesScreen.tsx`:

- Extend `HermesChatSession` with notification metadata:
  - `needsReply?: boolean`
  - `lastNotifiedAssistantMessageId?: string`
  - `notificationUpdatedAt?: string`
- Ensure any type guard in `src/App.tsx` still accepts existing stored sessions without these fields.
- Add helper functions near session utilities:
  - `markSessionNeedsReply(session, assistantMessageId, now)`
  - `clearSessionNeedsReply(session)`
  - `shouldNotifyForAgentResponse(sessionId, assistantMessageId, activeSessionId)`
- When `onActiveSessionIdChange(session.id)` is triggered from a session tab, clear `needsReply` for that session before/alongside switching.
- Also clear on initial mount if the active session already had stale `needsReply`.

### 3. Trigger notifications when Hermes actually responds

Update `sendHermesPrompt`, `appendCommandResult`, and confirmed slash-command completion paths in `src/agents/AgentsHermesScreen.tsx`:

- Capture the sending session id and assistant message id.
- At response completion, compare against current active session via a ref, not a stale closure:
  - maintain `activeSessionIdRef.current = activeSessionId` in an effect.
- If completed session id differs from `activeSessionIdRef.current`, mark that session as `needsReply`.
- Avoid duplicate notifications by checking `lastNotifiedAssistantMessageId`.
- Notify only after the assistant message is set to `status: "sent"`.
- Do not notify on `status: "error"` unless a later requirement asks for error notifications.

### 4. Add per-session visual indicator above the session box

Update `src/agents/AgentsHermesScreen.tsx` session rendering:

- For each session tab row, compute `const needsReply = Boolean(session.needsReply)`.
- Render a semantic indicator near/above the session box, not only in the portrait icon:
  - Example class: `session-reply-indicator`
  - Include accessible label: `aria-label="Hermes replied and needs your reply"`
  - Use an icon from `lucide-react` such as `BellDot`, `MessageCircleReply`, or `CircleAlert`.
- Ensure compact rail mode still shows the indicator clearly.
- Keep archive button usable and avoid overlap.

Update `src/App.css`:

- Style `.session-reply-indicator` using existing design tokens:
  - `var(--kujo-red)`, `var(--kujo-paper)`, `var(--kujo-ink)`, `var(--kujo-blue-soft)`.
  - hard black border / paper fill to match session boxes.
  - no generic floating web badge look.
- Position it above the session card edge, for example top-right over `.session-tab-row`, with z-index inside the rail.
- Add compact-mode positioning.
- Preserve existing `.session-notification-dot` global nav behavior.

### 5. Keep global Agents notification synchronized

Update `src/App.tsx`:

- Replace/confirm `hasHermesWaitingNotification` derives from `hermesSessions.some((session) => session.needsReply)`.
- Ensure switching into the Agents workspace does not clear all session indicators; only opening a specific session clears that session.
- If there is currently logic based on "latest assistant message after latest user message", either remove it or align it with explicit `needsReply` state.

### 6. Add macOS desktop notification support

Frontend changes:

- Add dependency: `@tauri-apps/plugin-notification`.
- Add a small client wrapper, likely `src/agents/agentNotifications.ts`, with:
  - `ensureDesktopNotificationPermission()`
  - `sendDesktopAgentNotification({ sessionTitle, responsePreview })`
- Import and call the wrapper from `AgentsHermesScreen.tsx` only when a background-session response completes.
- Gracefully no-op in tests/browser environments where Tauri plugin APIs are unavailable.

Rust/Tauri changes:

- Add dependency in `src-tauri/Cargo.toml`: `tauri-plugin-notification = "2"`.
- Register plugin in `src-tauri/src/lib.rs`:
  - `.plugin(tauri_plugin_notification::init())`
- Add notification permission to `src-tauri/capabilities/default.json`.
- Confirm macOS bundle identifier remains `com.mavoid.zoid25` in `src-tauri/tauri.conf.json`.

### 7. Add email notification backend

Rust dependencies to evaluate/add in `src-tauri/Cargo.toml`:

- Preferred: `lettre` with rustls TLS support for SMTP.
- Use `serde` structs already available for command payloads.

Add structs and command in `src-tauri/src/lib.rs`:

- `AgentResponseEmailNotificationRequest`
- `AgentResponseEmailNotificationResult`
- `send_agent_response_email_notification(request) -> Result<..., String>`

SMTP config lookup order:

1. Environment variables, for dev/test:
   - `ZOID_NOTIFY_SMTP_HOST`
   - `ZOID_NOTIFY_SMTP_PORT`
   - `ZOID_NOTIFY_SMTP_USERNAME`
   - `ZOID_NOTIFY_SMTP_PASSWORD`
   - `ZOID_NOTIFY_EMAIL_FROM`
   - optional `ZOID_NOTIFY_EMAIL_TO`, defaulting to `ziad.ahmed.25.25.25@gmail.com`
2. Optional local config file under app config dir, only if project already has a convention for app-local settings.

Security rules:

- Never expose SMTP password to frontend.
- Never store SMTP secrets in `localStorage`.
- Never commit real SMTP credentials.
- Email failures should return a structured error and be shown/logged as non-blocking status.

Register command in `tauri::generate_handler!`.

Add frontend client function in `src/agents/hermesClient.ts` or `src/agents/agentNotifications.ts`:

- `sendAgentResponseEmailNotification(payload)` invoking the Tauri command.

### 8. Conversation summary content for email

Implement frontend summary/excerpt builder first unless Rust needs the raw messages:

- Function: `buildAgentResponseEmailSummary(session, assistantMessageId)`.
- Summary should be deterministic and bounded:
  - `Session: <title>`
  - `Latest user message:` first 800-1200 chars of the most recent user message before the assistant response.
  - `Hermes response summary:` first 1200-1800 chars of the assistant response, collapsed whitespace, with code blocks reduced if too long.
  - `Context:` 3-5 bullet style snippets from recent messages.
- Do not use another LLM unless explicitly approved later.
- Avoid sending full huge conversations by default.
- Add a note: "Open Zoid to continue the session."

Email subject:

- `Zoid: Hermes replied in <session title>`

### 9. Notification preferences / safety controls

Minimum first version:

- Hard-enable for background-session responses only.
- Email recipient fixed/defaulted to `ziad.ahmed.25.25.25@gmail.com`.
- If SMTP not configured, desktop/session badge still work.

Optional but recommended in same implementation if small:

- Add settings toggles in Profile/Safety section (`src/App.tsx`) for:
  - Desktop notifications enabled
  - Email notifications enabled
  - Email recipient
- If adding settings is too large, defer toggles and keep env-driven email for v1.

### 10. Tests / validation

Frontend tests:

- Update/add tests in `src/scaffold.test.ts` for structural checks:
  - `HermesChatSession` includes `needsReply` metadata.
  - `AgentsHermesScreen` renders `session-reply-indicator` only for sessions with `needsReply`.
  - Clicking/opening a session clears that session indicator.
  - Existing global Agents nav notification dot still exists and does not inherit boxed session icon styling.
- Add or update focused behavior test if feasible:
  - `src/agents/AgentsHermesScreen.notifications.test.tsx`
  - Simulate background session response completion and assert `needsReply` is set.

Rust tests:

- Add unit tests for email payload summary/sanitization if summary is built in Rust.
- Add command config tests with env vars using existing `env_lock()` style in `src-tauri/src/lib.rs` tests.
- Do not send real email in unit tests; mock/validate message construction only.

Manual/app validation:

- Run `npm run test:frontend`.
- Run `npm run test:rust`.
- Run `npm run build`.
- Launch with `/Applications/Zoid 25.app` or `com.mavoid.zoid25` after rebuild, not `open -a Zoid`.
- Create two sessions.
- Send a prompt in session A, switch to session B before Hermes responds.
- Confirm session A shows the new indicator above/on its session box.
- Confirm global Agents nav dot remains visible if outside Agents workspace.
- Confirm opening session A clears only session A indicator.
- Confirm macOS prompts/allows notification permission and displays a desktop notification.
- With SMTP env vars configured, confirm one real email is received at `ziad.ahmed.25.25.25@gmail.com` with bounded summary content.
- If SMTP unavailable, confirm UI remains functional and records non-blocking notification failure.

### 11. Feature critique gate before completion

Because this changes a Zoid/Hermes app feature, do not mark implementation complete until the critique workflow passes:

- Create `.hermes/reviews/agent-response-notifications/handoff.md` with:
  - exact scope
  - changed files
  - notification trigger rules
  - manual verification evidence
  - test outputs
  - known SMTP/macOS permission limitations
- Trigger/wait for the separate critique-agent review.
- Fix all Required fixes.
- Re-review until verdict is `APPROVED`.

## Files likely to change

- `src/agents/AgentsHermesScreen.tsx`
  - session notification state model
  - background response detection
  - per-session indicator rendering
  - desktop/email notification calls
- `src/agents/hermesClient.ts`
  - new Tauri invoke wrapper for email notification, unless placed in a new file
- `src/agents/agentNotifications.ts` (new)
  - desktop notification permission/send wrapper
  - email summary builder/client wrapper
- `src/App.tsx`
  - global `hasHermesWaitingNotification` derivation and active-session clearing flow
  - optional Profile/Safety notification preferences
- `src/App.css`
  - session reply indicator design-system styling and compact-mode support
- `src/scaffold.test.ts`
  - structural notification tests
- `src/agents/AgentsHermesScreen.notifications.test.tsx` (possible new focused test)
- `src-tauri/Cargo.toml`
  - `tauri-plugin-notification`
  - `lettre` or selected SMTP dependency
- `src-tauri/src/lib.rs`
  - notification plugin registration
  - email notification Tauri command
  - tests for config/message construction
- `src-tauri/capabilities/default.json`
  - notification plugin permissions
- `.hermes/reviews/agent-response-notifications/handoff.md`
  - required critique handoff after implementation

## Risks, tradeoffs, and open questions

- SMTP credentials are required for real email delivery. Without them, implementation can only verify payload construction and graceful failure.
- macOS notification behavior depends on app permission state and bundle identity; the first run may require granting notification permission in System Settings.
- A deterministic excerpt is safer and cheaper than an LLM-generated summary, but less polished. A model-generated summary can be added later if explicitly approved.
- If many sessions finish close together, emails could spam. Add a duplicate guard per assistant message in v1; consider throttling later.
- If the user has a session visible but the app is backgrounded, current requirement says "session not open"; v1 should notify only for non-active session. A future setting can also notify when app window is unfocused.
- Email content may include sensitive chat details. Keep summaries bounded, avoid secrets where possible, and do not include full raw transcripts by default.

## Definition of done

- Background-session Hermes replies set a per-session `needsReply` indicator above/on the session box.
- Opening that session clears the indicator.
- Global Agents notification remains consistent with per-session unread/reply state.
- macOS desktop notification fires for qualifying responses.
- Email command exists, uses secure backend SMTP config, and sends bounded summaries to `ziad.ahmed.25.25.25@gmail.com` when SMTP is configured.
- Missing SMTP config does not break chat.
- Frontend, Rust, build, and in-app smoke checks pass.
- Feature critique review verdict is `APPROVED`.
