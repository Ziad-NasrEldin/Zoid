# Feature Handoff: multi-panel parallel agent dashboard re-review

## Original request
User requested seamless multi-panel chat window support inside Zoid 25 Agents page: drag/tile agents/sessions from the sessions rail into the main window, auto-adjust sizing/spacing for 2, 3, and 4 windows, preserve smooth UX and clean sumi-e UI.

Continuation context: previous reviewer required fixes for streaming, queue lifecycle, expanded composer scoping, auto-prioritize, queued count, active run listing/reconciliation, plus a follow-up UI/visual hierarchy review after screenshot regression.

## Implementation summary
- Backend Hermes run execution emits Tauri lifecycle/output events while preserving final accumulated stdout/stderr response.
- Active run snapshots include backend `started_at`/`startedAt` data instead of frontend-fabricated timestamps.
- Frontend runtime supports active-run reconciliation, event-shaped run updates, single-source queued counts, terminal event hardening, and stale/duplicate sequence rejection.
- Dashboard input scoping was corrected:
  - tiled mode hides the global composer so only panel-scoped composers send prompts;
  - expanded mode now targets `expandedSession ?? activeSession` and Stop is scoped to the expanded session;
  - expanding a panel also opens/focuses that session.
- Dashboard panel UX was tightened after screenshot review:
  - wider readable sessions rail default/min widths;
  - compact topbar/status row keeps File controls visible;
  - compact no-overlap panel header/actions/composer for 2-up tiling;
  - explicit 3-panel auto layout: primary large panel plus two stacked secondary panels;
  - toolbar and panel chrome align with the existing sumi-e paper/ink/red-seal visual system.
- Queue lifecycle was hardened after final critique:
  - prompts/slash commands only append optimistic messages after a run slot is admitted;
  - capacity-full idle panel submits queue instead of disappearing;
  - completing, stopping, or erroring any run drains the next globally startable queued prompt;
  - reconciled backend terminal events also trigger queue draining.

## Changed files
- `src-tauri/src/lib.rs`: run snapshot `started_at`, event type/emission, AppHandle plumbing, stdout/stderr streaming reader threads, accumulated final output, active run listing updates.
- `src/agents/AgentsHermesScreen.tsx`: dashboard tile rendering/scoping fixes, hidden global composer while tiled, expanded-session composer/stop scoping, stream event duplicate/terminal guards, rail width constants, queue/count/reconciliation paths.
- `src/agents/useAgentRuntime.ts`: runtime state/event/reconciliation support.
- `src/agents/hermesClient.ts`: typed active run/list and run message bridge support.
- `src/App.css`: sessions rail, monitor toolbar, 1/2/3/4 panel grid, compact panel controls/composer, topbar visual hierarchy and sumi-e cleanup.

## How to test
- `npm run build`
- `cargo check` from `src-tauri`
- Local preview at `http://127.0.0.1:1420/` if existing dev server is running.
- Agents page smoke:
  - open Agents;
  - tile one session;
  - create/tile second and third sessions;
  - verify monitor count changes to 3 tiled;
  - verify 3-panel layout is primary + stacked secondaries;
  - verify global composer disappears while tiled and per-panel composers remain scoped to panels;
  - verify File button/topbar controls stay visible.

## Tests run
- `npm run build`: PASS. Vite built 1797 modules and emitted production assets.
- `npm test`: PASS. Frontend suite passed; Rust tests passed with 76 passed, 1 ignored, 0 failed.
- `cargo check` in `src-tauri`: PASS. Finished dev profile.
- Browser/local UI smoke on existing Vite server `127.0.0.1:1420`: PASS for Agents navigation, four tiled panels visible in compact short-viewport mode, hidden global composer in tiled mode, visible sessions rail, visible File/topbar controls, no monitor/main/panel x/y overflow.

## Git info
- Branch: `main`
- Commit SHA: not committed
- Diff base: current dirty worktree; repo has unrelated existing dirty/untracked files outside this focused task.

## Frontend/backend/database notes
- Frontend routes/components: Agents page only (`AgentsHermesScreen`, `AgentMonitorPanel`, `useAgentRuntime`, `hermesClient`).
- Backend endpoints/services: Tauri Hermes CLI commands/list/cancel/run registry in `src-tauri/src/lib.rs`.
- Database: none.

## Reviewer focus areas
- Confirm the six previous required findings are resolved on disk:
  1. real streaming event path exists and frontend consumes/rejects stale/duplicate/late terminal events;
  2. queue drains after any freed run slot, including success, stop/error, and reconciled backend terminal events;
  3. expanded composer/session scoping cannot send/stop the wrong session;
  4. auto-prioritize has non-empty behavior, not a no-op;
  5. queued count is single-source and not double-counted;
  6. backend active run listing includes real startedAt and Agents screen reconciles on mount.
- Inspect current visual hierarchy and sumi-e consistency in `src/App.css` and browser layout, especially 2/3/4 tiled panels, sessions rail readability, monitor toolbar wrapping, and short viewport behavior.
- Check TypeScript/Rust compile safety around Tauri event emission and process stdout/stderr thread joins.

## Fix cycle notes
- Previous focused critique verdict: `REQUEST_CHANGES` with three required fixes:
  1. expanded composer still scoped to active session;
  2. no intentional 3-panel visual hierarchy;
  3. streaming needed duplicate/terminal hardening.
- Latest fixes after that critique:
  - `handleSend` targets `expandedSession ?? activeSession`;
  - expanded/global composer Stop calls `handleStopHermesRun("button", expandedSession?.id)`;
  - added stream event sequence map and terminal run guard;
  - final response no longer overwrites already-streamed assistant content;
  - backend stopped/completed terminal events now use the next monotonic `event_sequence` instead of `0`, so frontend sequence rejection no longer drops terminal events after streamed output;
  - added `.agent-monitor-grid--count-3` primary + stacked secondary layout and responsive reset.
- Latest final review verdict: `APPROVED`.
- Latest verification: `npm run build` PASS, `npm test` PASS, `cargo check` PASS, browser smoke PASS for 4 tiled panels in compact short-viewport mode.