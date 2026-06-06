# Zoid Phase 2 — First Vertical Slice Spec

Date: 2026-06-02
Tracker items: P2.01, P2.02
Source tracker: `Docs/2026-06-01-zoid-implementation-tracker.md`
Source plan: `Docs/2026-05-31-zoid-implementation-plan-v1.md`

## Goal

Prove the core Zoid operating loop before broad workspace expansion:

`Today -> Task -> CLI Session -> AgentRun -> ReviewRecord -> Notification -> History`

Phase 2 must use real local data, truthful empty/unconfigured states, SQLite/app-support persistence, event-backed history, redacted logs, and feature critique approval before being called complete.

## In scope

- Task create/list/detail/update using SQLite.
- A configured local CLI profile surface with truthful configured/unconfigured states.
- CLI Session and AgentRun records for one safe local command path.
- Clean Session UI as the default output surface; raw logs are secondary/collapsible.
- Redacted raw logs persisted under app support, not inside SQLite.
- ReviewRecord manual stub for task/run review.
- Persistent in-app Notification/Inbox records for completion, blocker, failure, and attention.
- History query from events and event targets for task/run/review/notification timelines.
- App restart persistence for task, run metadata, review record, notification, and history.

## Explicitly out of scope

- Gmail, OmniSocials, GitHub/Vercel full integration.
- Apple Notes import.
- Full file manager.
- Full browser workspace.
- Full EventKit/calendar integration beyond existing spike findings.
- Autonomous content publishing.
- Mobile/private sync.
- Direct LLM API calls. Zoid runs local CLIs only in this phase.

## Entity boundary

Phase 2 should stay narrow and model only the first vertical slice.

### `tasks`

User-visible unit of work.

Required semantics:
- title and optional detail;
- status and priority;
- created/updated timestamps;
- archived/deleted handling;
- Today/list queries exclude deleted and normally exclude archived tasks;
- creating or changing meaningful status writes events.

### `agent_profiles`

Configured local CLI profile.

Required semantics:
- profile can be configured or unconfigured truthfully;
- command/config/capability metadata is stored;
- secrets are never stored as raw values; only credential/env references are allowed;
- unconfigured/missing command/cwd blocks execution before fake success can appear.

### `cli_sessions`

Session-level container for Clean Session UI and future multiple run attempts.

Required semantics:
- linked to task;
- records mode/cwd/profile context at the session level;
- owns session status summary separate from individual run attempt status;
- app restart can show historical session metadata.

### `agent_runs`

One execution attempt attached to a task/session/profile.

Required semantics:
- stores task_id, profile_id, session_id, cwd, command/profile snapshot, status, timestamps, duration, exit_code, log reference, output/error summaries, review state, metadata;
- raw stdout/stderr logs are not stored in SQLite;
- significant lifecycle transitions write events;
- retry creates a new run attempt rather than mutating terminal runs into new work.

### `review_records`

Manual reviewer stub first, with room for future reviewer profile output.

Required semantics:
- can review a task, run, or related entity;
- includes verdict/evidence/required fixes;
- reviewer_profile_id can be null for manual stubs;
- review events are written;
- review approval is required before any review-gated task is truthfully complete.

### `notifications`

Persistent Inbox/attention item.

Required semantics:
- stores type/title/message/severity/state/action route/read/dismiss/resolution timestamps;
- links to task/run/review through direct fields where useful and/or entity_links;
- read/dismissed/resolved state does not mutate the underlying task/run/review;
- native macOS notification delivery is optional and gated by verified native support.

### `events` and `event_targets`

Append-only history backbone, already present from Phase 1.

Required semantics:
- every meaningful action writes a redacted event;
- event_targets connect events to task/run/review/notification entities;
- events never contain raw logs or secrets;
- History views query events, not fake frontend state.

### `entity_links`

Durable relationship graph, already present from Phase 1.

Required semantics:
- entity_links represent stable relationships such as task -> agent_run, run -> review, task -> notification;
- event_targets represent event references and are not a replacement for durable relationships;
- direct foreign keys remain necessary for common ownership paths.

## Data flow

1. Task created
   - Insert task.
   - Write `task.created` event with task target.
   - Today shows the real task.

2. CLI run requested
   - Validate task exists and is not deleted.
   - Validate profile/cwd/command configuration.
   - Create `cli_session` and `agent_run` in `queued` or `starting`.
   - Link task/session/run.
   - Write `run.queued` or `run.started` event.

3. Run starts and streams
   - Process starts only after preflight passes.
   - Stream output into Clean Session UI cards.
   - Redact and persist raw log file under app support.
   - SQLite stores log reference/metadata only.
   - Write lifecycle events for meaningful transitions, not every output chunk.

4. Run completes, fails, cancels, or blocks
   - Update AgentRun status, completion timestamp, duration, exit code where applicable, log reference, output/error summary.
   - Write `run.completed`, `run.failed`, `run.cancelled`, or `run.blocked` event.
   - Create notification where the result needs user attention or confirms completion.

5. ReviewRecord created
   - Manual review stub creates a ReviewRecord linked to task/run.
   - Write `review.created` and final verdict event.
   - Required fixes or insufficient evidence keep the task from being complete.

6. Notification created
   - Insert persistent notification.
   - Link to relevant task/run/review.
   - Write `notification.created` event.
   - Today/Inbox shows completion, blocker, failure, review-required, or attention item.

7. History queried
   - Query events and event_targets by task/run/entity.
   - Expand durable entity_links only where needed for related timelines.
   - Show log references/summaries, never raw unredacted log bodies.

## Lifecycle state model

### TaskStatus

Values:
- `inbox`: captured but not planned/started.
- `planned`: accepted as work but not active.
- `active`: user or agent is working on it.
- `waiting`: paused for non-blocking user input, time, dependency, or external response.
- `review_required`: output exists but cannot be called done until review approval exists.
- `blocked`: cannot proceed until missing prerequisite, permission, config, evidence, or policy gate is resolved.
- `completed`: truthfully complete with required evidence and approval.
- `failed`: attempted work ended unsuccessfully.
- `cancelled`: intentionally stopped before normal completion.
- `archived`: hidden from active views but retained.
- `deleted`: soft-delete/tombstone retained for future sync/history integrity.

Important rule: a completed AgentRun does not automatically complete a task unless task completion policy and review requirements are satisfied.

### AgentRunStatus

Values:
- `queued`
- `starting`
- `running`
- `waiting_for_input`
- `review_required`
- `completed`
- `failed`
- `cancelled`
- `blocked`

Completion requires observed process exit, success result, duration, log reference, summary, completion event, and no unresolved review gate.

### ReviewState and ReviewVerdict

Review state values:
- `not_required`
- `required`
- `requested`
- `in_progress`
- `approved`
- `required_fixes`
- `blocked_insufficient_evidence`
- `failed`
- `cancelled`

Verdict values:
- `approved`
- `required_fixes`
- `blocked_insufficient_evidence`

`required_fixes` means work was reviewed and rejected. `blocked_insufficient_evidence` means the reviewer cannot verify truthfully.

### NotificationState

Values:
- `pending`
- `delivered`
- `read`
- `action_required`
- `resolved`
- `dismissed`
- `failed`

Read/dismiss/resolve affects the notification only, not the linked task/run/review.

### Failure versus blocker

Blocked means Zoid knows it cannot truthfully proceed because a prerequisite or gate is missing or denied.

Examples:
- CLI profile unconfigured;
- command not found before safe launch;
- working directory missing;
- permission or confirmation required;
- review approval required;
- insufficient evidence;
- app-support/log path unavailable before safe launch.

Failed means Zoid attempted an operation and observed an unsuccessful outcome.

Examples:
- process spawned and exited nonzero;
- timeout;
- crash/signal;
- DB/event/log persistence failed after attempting work;
- parser could not produce a truthful summary.

## Event taxonomy for Phase 2

Minimum events:
- `task.created`
- `task.updated`
- `task.status_changed`
- `task.archived`
- `task.deleted`
- `run.queued`
- `run.started`
- `run.waiting_for_input`
- `run.completed`
- `run.failed`
- `run.cancelled`
- `run.blocked`
- `run.log_reference_created`
- `review.created`
- `review.approved`
- `review.required_fixes`
- `review.blocked_insufficient_evidence`
- `notification.created`
- `notification.delivered`
- `notification.failed`
- `notification.read`
- `notification.resolved`

## P2.03-P2.07 implementation order

1. P2.03: `tasks` schema/model/repository with task events.
2. P2.04: `agent_profiles`, `cli_sessions`, `agent_runs`, log reference linkage, run lifecycle metadata.
3. P2.05: `review_records` schema/model/repository linked to task/run.
4. P2.06: `notifications` schema/model/repository and Inbox query basics.
5. P2.07: History query model using events, event_targets, and related entity_links.

## TDD acceptance targets

### P2.03 tasks

- Task create has required title and safe defaults.
- Empty/oversized title/detail is rejected.
- Invalid status/priority is rejected.
- Archive/delete removes task from active Today/list queries.
- Task create writes one `task.created` event with task target.
- Status update writes `task.status_changed`.

### P2.04 run/session

- Run cannot be created for a missing/deleted task.
- Run cannot start from an unconfigured profile.
- Session/run are linked to task.
- Lifecycle transitions reject impossible terminal mutations.
- Completed run stores duration, exit code, log reference, summary.
- Failed, cancelled, blocked, and waiting_for_input are distinct.
- Raw log content is absent from SQLite/events.

### P2.05 review

- Manual review can be created for task/run.
- Required fixes require a non-empty required_fixes payload.
- Review event is written and linked.
- Approved review can satisfy review gate.
- Insufficient evidence blocks completion.

### P2.06 notification

- Completion/blocker/failure/review-required notification can be created.
- Notification is linked to relevant task/run/review.
- Read/dismiss/resolve transitions do not mutate task/run/review.
- Notification creation writes event.
- Inbox query sorts actionable unread items by severity and time.

### P2.07 history

- Task history returns task, run, review, and notification events in deterministic order.
- Run history includes run lifecycle and linked review/notification events.
- Related entity expansion does not leak unrelated events.
- Raw log body is never returned by history queries.
- Pagination/limit ordering is deterministic.

## Verification gates

- Focused Rust/database tests for each slice.
- `cargo test --manifest-path src-tauri/Cargo.toml` after backend/database slices.
- Frontend smoke/build once UI surfaces are touched.
- `npm run verify:local` before phase-level handoff.
- Manual macOS verification for create task -> start CLI run -> stream output -> notification/history -> restart persistence.
- `.hermes/reviews/phase-2-first-vertical-slice/handoff.md` and critique report with `Verdict: APPROVED` before Phase 2 is marked complete.

## Planning review evidence

Two read-only subagent lanes reviewed this artifact's boundaries before schema work:
- data-model boundary review: PASS;
- lifecycle/state review: completed with recommended enums, transitions, failure/blocker split, and tests.
