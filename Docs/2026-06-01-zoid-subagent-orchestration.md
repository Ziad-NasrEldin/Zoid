# Zoid Subagent Orchestration Model

Purpose: use an orchestrator + spawned subagent system throughout Zoid development, not only for one phase.

## Operating model

- Main Hermes agent is the orchestrator.
- Subagents do focused implementation, discovery, spec review, quality/security review, and critique-prep work.
- The orchestrator owns task ordering, file ownership, merges, conflict prevention, tracker updates, final verification, commits, and user reporting.
- Subagent claims are evidence, not truth, until the orchestrator rechecks the repository and runs the relevant gates.

## File-ownership rule

Parallel implementation is allowed only when slices do not edit the same primary files.

- If two slices touch `src-tauri/src/lib.rs`, migrations, shared frontend shell files, or shared tests, serialize implementation.
- Parallel no-edit discovery/review is allowed while implementation is running.
- Review agents must not edit application code unless explicitly assigned as fix agents.

## Standard lane types

1. Implementer lane
   - Writes tests first.
   - Runs focused failing tests, implements, reruns focused tests and broader Rust/build gates.
   - Commits only when explicitly assigned and verified.

2. Spec reviewer lane
   - Reads original tracker/spec and current source.
   - Reports PASS or exact gaps.
   - Does not edit code.

3. Quality/security reviewer lane
   - Checks edge cases, data safety, migrations, redaction, authorization/confirmation gates, and maintainability.
   - Does not edit code.

4. Discovery lane
   - No application edits.
   - Maps upcoming tasks, dependencies, likely file ownership, and tests.

5. Final critique gate
   - Feature handoff under `.hermes/reviews/<feature>/handoff.md`.
   - Critique report under `.hermes/reviews/<feature>/critique-report.md`.
   - Feature is not complete until verdict is `APPROVED` or the user explicitly waives the gate.

## Central verification gates

Before any feature is reported complete:

- Re-read current source/review artifacts if subagents edited files.
- Run relevant focused tests.
- Run `npm run verify:local`.
- Run `npm run verify:release` at stopping/reporting checkpoints unless impractical; state why if skipped.
- Update `/Users/ziadnasreldin/Zoid/Docs/2026-06-01-zoid-implementation-tracker.md`.
- Commit tracker/review artifacts separately when useful.

## Current phase execution plan

- P1.14 event writer/reader: single implementer lane because it touches shared backend core/migrations/tests.
- P1.15 entity link service: no-edit discovery can run in parallel, but implementation waits until P1.14 is committed because both likely touch `src-tauri/src/lib.rs` and entity/event link code.
- P1.16 Tauri bridge: no-edit discovery can run after P1.14/P1.15 contracts are clear; implementation waits for backend repository APIs.
- Frontend tasks: can run design/audit discovery in parallel, but code edits wait until backend command contracts stabilize.

## Reporting format

Every checkpoint report separates:

- Active lanes and status.
- Commits made / not made.
- Verification output.
- Critique verdicts.
- Blockers.
- Next lanes.
