# Zoid Phase 2 Velocity Operating Model

Date: 2026-06-02

## Why this exists

P2.03-P2.06 proved the backend foundation is moving correctly but too slowly. The bottleneck is fixed review/verification overhead paid for every small tracker checkbox, plus a large `src-tauri/src/lib.rs` monolith that makes every backend slice harder to inspect.

This document changes the operating model for the rest of Phase 2: grouped backend batches, risk-banded gates, lean reviews, and incremental monolith reduction.

## Parent/orchestrator ownership

The parent Hermes session owns:

- repo status and conflict checks;
- tracker interpretation and task boundaries;
- file-ownership boundaries for subagents;
- central verification commands;
- final critique handoff/report validation;
- tracker updates;
- commits;
- concise user reporting.

Subagent summaries are evidence, not truth, until the parent re-checks files/tests/status.

## Risk bands

### Band A — Foundation-critical

Examples:

- SQLite migrations and schema ownership constraints;
- repository primitives that persist durable state;
- secret/log redaction or leak prevention;
- process execution/cancellation/log persistence;
- Tauri bridge commands that mutate durable data.

Gate:

- TDD/focused tests;
- direct DB/repository invariant tests where relevant;
- one combined read-only reviewer;
- one combined re-review only if fixes were required;
- final critique approval;
- focused + full relevant verification before commit.

### Band B — Backend service wrappers/query composition

Examples:

- task/detail/list service wrappers over already-reviewed repositories;
- notification/history query services over existing event/entity-link records;
- CLI profile truthful configured/unconfigured helpers.

Gate:

- focused tests;
- one lean combined review for the whole batch;
- final critique for the batch, not each checkbox;
- full Rust suite before commit.

### Band C — UI/bridge glue

Examples:

- Tauri command registration;
- Today/Inbox/History UI surfaces using reviewed services.

Gate:

- focused unit/component or command tests;
- app/browser/native smoke where feasible;
- critique per user-visible batch, not per component.

### Band D — Tracker/docs/process-only

Examples:

- tracker updates;
- handoff/report documents;
- operating-model docs.

Gate:

- no standalone critique unless bundled with code.

## Batching rule

Do not pay the full review tax for one tiny checkbox when related work shares the same files and domain.

Preferred Phase 2 batches:

1. **Backend query/service batch**
   - P2.07 History/Event query model.
   - P2.08 Task create/list/detail/update service.
   - P2.15 Notification creation/query service basics where it reuses P2.06 primitives.
   - P2.16 History query service basics where it reuses P2.07 primitives.

2. **Agent execution batch**
   - P2.09 CLI profile config.
   - P2.10 command/session runner.
   - P2.11 cleanup/failure handling.
   - P2.12 redacted raw log references.
   - P2.13 run lifecycle service.

3. **Review/bridge batch**
   - P2.14 manual ReviewRecord service.
   - P2.17-P2.19 Tauri commands/events for tasks/runs/reviews/notifications/history.

4. **Frontend vertical batch**
   - P2.20-P2.27 Today/task/run/review/inbox/history UI basics.

5. **Verification batch**
   - P2.28-P2.35 backend/UI/manual/verify/review gates.

## Review rule

For small backend/database batches:

- One combined reviewer covers spec, quality, security/redaction, DB integrity.
- If fixes are required, fix with regression tests and run one combined re-review.
- Then run one final critique for the whole batch.
- More reviews require a high-risk reason.

Expected review count:

- clean batch: 2 reviews total;
- batch with fixes: 3 reviews total.

## Monolith reduction rule

`src-tauri/src/lib.rs` should shrink incrementally, but refactors must not stall Phase 2.

Priority order:

1. Move large test modules out of `lib.rs` into dedicated test modules when no behavior changes are needed.
2. Extract new Phase 2 service/query code into dedicated modules instead of adding more to the monolith.
3. Move existing repositories only when a batch touches them heavily and tests can prove no behavior change.
4. Avoid parallel implementers editing `src-tauri/src/lib.rs`; use one writer plus parallel read-only discovery/review lanes.

## Current next batch

Start with the backend query/service batch:

- P2.07 History/Event query model.
- P2.08 Task create/list/detail/update service.
- P2.15 Notification creation/query service basics if it can be thin over P2.06.
- P2.16 History query service basics if it can be thin over P2.07.

Commit boundary: one commit for the batch after final critique approval and tracker update.
