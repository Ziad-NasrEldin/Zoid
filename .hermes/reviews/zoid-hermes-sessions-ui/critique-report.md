# Critique Report: Zoid Hermes sessions UI

## Verdict

APPROVED

## Scope Reviewed

- Handoff: `.hermes/reviews/zoid-hermes-sessions-ui/handoff.md`
- Source changes relevant to the feature:
  - `src/agents/AgentsHermesScreen.tsx`
  - `src/agents/ChatComposer.tsx`
  - `src/App.css`
  - `src/App.tsx`
  - `src/scaffold.test.ts`
- Native/install evidence:
  - `/Applications/Zoid 25.app/Contents/MacOS/zoid`
  - `/tmp/zoid25-sessions.png`

## Requirements Check

| Requirement | Result | Notes |
| --- | --- | --- |
| Compact New session button in top chat header | PASS | `AgentsHermesScreen.tsx` renders a topbar `button.new-session-button` with label `New session`; CSS gives it compact 34px minimum height and small uppercase styling. |
| Separated left Sessions rail for opened sessions | PASS | `chat-workspace` is a two-column grid with an `aside.sessions-rail` labelled `Opened Hermes sessions`; CSS separates it with a right border and fixed rail column. |
| Switching sessions via rail without deleting old sessions | PASS | Sessions are held in an array. `handleNewSession` prepends a new session while preserving current sessions, and rail clicks call `setActiveSessionId(session.id)` only. Message updates target `sendingSessionId`, so in-flight responses update the original session even if active selection changes. |
| Composer attach/input/send sizing share one height token | PASS | `.hermes-chat-shell` defines `--composer-control-size: 44px`; attach, textarea, and send button heights/min-heights reference that token. `ChatComposer` uses `rows={1}` to align the textarea with the shared height. |
| Zoid 25 installed/native visible evidence | PASS | `/Applications/Zoid 25.app` exists, binary timestamp is current (`Jun 6 21:15:28 2026`), process is running from `/Applications/Zoid 25.app/Contents/MacOS/zoid`, and `/tmp/zoid25-sessions.png` exists as a 1920x1080 PNG captured at `Jun 6 21:13:48 2026`. |

## Verification Run

- `npm run test:frontend` — PASS
  - Command output: `> zoid-25@0.25.0 test:frontend` / `> tsx src/scaffold.test.ts`
- `npm run build` — PASS
  - Command output: `tsc && vite build`, 39 modules transformed, build completed successfully.
- Native/install evidence inspection — PASS
  - `/Applications/Zoid 25.app/Contents/MacOS/zoid` exists and is currently running as PID `51076`.
  - `/tmp/zoid25-sessions.png` exists and is a `PNG image data, 1920 x 1080, 8-bit/color RGBA` file.

## Findings

No blocking findings.

## Non-blocking Notes

- The rail uses `role="list"` on the container but the session buttons do not use `role="listitem"`. This is not a blocker because the buttons are still accessible controls with active state via `aria-current`, but the semantics could be tightened later.
- The sessions are currently in-memory only. That matches the stated requirement for opened-session switching and preservation during the running UI session; persistence across app relaunch was not requested.

## Final Assessment

The implementation satisfies the requested Hermes sessions UI behavior in the active Zoid 25 source and has passing focused frontend/build verification. The installed native app evidence is present. No product code changes are requested.
