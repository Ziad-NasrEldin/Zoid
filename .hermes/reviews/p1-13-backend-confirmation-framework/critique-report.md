# Critique Report: P1.13 Backend confirmation decision framework

## Verdict

APPROVED

## Summary
Reviewed the current backend source, migrations, focused tests, and handoff evidence for P1.13. The implementation provides confirmation decision records, SQLite persistence helpers, and a fail-closed execution guard for protected actions. Focused verification passed, including confirmation guard behavior, actor semantics, redaction, migration v4 safety, and clippy. I found no required fixes.

## What was changed
- Added backend confirmation decision types and records in `src-tauri/src/lib.rs`: decision state, actor type, actor, request, record, and execution gate result.
- Added SQLite helper functions to create, read, and list confirmation decisions with seeded action category validation, event-link validation, metadata JSON validation, and redaction before persistence.
- Added `require_policy_clearance_before_execution`, which fails closed without an `ActionPolicyDecision`, allows `allowed_now` decisions, and requires approved matching confirmation evidence for gated actions.
- Added actor satisfaction rules so system actors cannot satisfy human-confirmation policies, reviewer and clear-task requirements are independently enforced, and `reviewed_clear_task` is required to satisfy combined reviewer + clear-task requirements with one record.
- Updated fresh schema migration `src-tauri/migrations/0003_core_schema_p105.sql` so `confirmation_decisions.actor_type` has a CHECK constraint for known actor types.
- Added `src-tauri/migrations/0004_confirmation_actor_type_check.sql` to rebuild old v3 `confirmation_decisions` tables with the actor_type CHECK inside an explicit transaction, preserving valid rows and failing closed if a leftover upgrade table exists.
- Added focused backend unit/regression tests for guard behavior, actor semantics, redaction, confirmation persistence, and migration upgrade safety.
- Confirmed no new frontend or Tauri command surface was added; the command list remains `get_foundation_status` only.

## Required fixes
None.

## Improvements
| ID | Priority | Area | Suggestion | Why it matters |
|----|----------|------|------------|----------------|
| I-1 | Low | Confirmation evidence model | Consider adding explicit expiration timestamps or validity-window semantics to confirmation records when a future UI/flow creates approvals. | The current framework blocks records whose decision state is `expired`, but expiration is represented as a stored state rather than derived from time; future workflow code will need clear expiry rules to avoid stale approvals. |

## Tests performed
- `cargo test confirmation --lib`: PASS; 12 tests passed, 0 failed. Covered missing policy fail-closed, low-risk allowed_now pass, gated send-email approval flow, denied/cancelled/expired blocks, category mismatches, system actor rejection for human confirmation, reviewer/clear-task enforcement, combined `reviewed_clear_task`, redaction, persistence/listing, actor CHECK, and old-v3 upgrade preservation.
- `cargo test migration_v4 --lib`: PASS; 2 tests passed, 0 failed. Covered explicit transaction/no pre-drop checks and fail-closed behavior when `confirmation_decisions_actor_type_upgrade` already exists.
- `cargo clippy --lib --tests -- -D warnings`: PASS; completed without warnings.
- `git status --short && git diff --stat HEAD~4..HEAD`: PASS; application changes are limited to `src-tauri/src/lib.rs` and two SQLite migrations, with the review directory untracked; no frontend files or additional Tauri commands were changed.

## Tests still needed
None.

## Dev-agent instructions
- No required fixes. Proceed with merge/acceptance for P1.13.
- For future UI/API work, keep protected action execution behind `require_policy_clearance_before_execution` and ensure every protected action supplies an `ActionPolicyDecision`; do not infer permission from the presence of a confirmation record alone.
- When implementing user-facing confirmation creation, define and enforce approval freshness/expiry semantics so stale approvals are transitioned to or recorded as `expired` before guard evaluation.
