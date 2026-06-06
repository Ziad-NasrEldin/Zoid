# Feature Handoff: P1.13 Backend confirmation decision framework

## Original request

Phase 1 / Backend: Implement confirmation decision records/framework for consequential actions; do not execute protected actions without policy decision.

## Implementation summary

Implemented a backend-only confirmation framework for consequential actions:

- Added typed confirmation decision models:
  - `ConfirmationDecisionState`
  - `ConfirmationActorType`
  - `ConfirmationActor`
  - `ConfirmationDecisionRequest`
  - `ConfirmationDecisionRecord`
  - `ExecutionGateResult`
- Added SQLite service helpers:
  - `create_confirmation_decision`
  - `read_confirmation_decision`
  - `list_confirmation_decisions`
- Added execution guard:
  - `require_policy_clearance_before_execution`
- Guard requires an `ActionPolicyDecision`; missing policy fails closed.
- Low-risk `allowed_now` actions pass without confirmation records.
- Gated actions block without approved matching confirmation evidence.
- Denied/cancelled/expired decisions block.
- Human-confirmation policies cannot be approved by `system` actors.
- Reviewer-required and clear-task requirements are enforced; combined code-policy evidence uses `reviewed_clear_task` actor type and does not allow generic human/reviewer/clear-task bypass.
- Confirmation summaries and metadata are redacted before SQLite persistence, including standalone obvious tokens (`sk-*`, `ghp_*`, bearer-like material) and secret-keyed JSON values.
- Confirmation metadata JSON is validated and remains valid after redaction.
- Confirmation action category is validated against seeded `action_policies`; optional event links validate through FKs.
- Added migration v4 to safely upgrade old v3 databases where `confirmation_decisions.actor_type` lacked a CHECK constraint.
- Migration v4 is atomic (`begin immediate`/`commit`) and fail-closed for leftover upgrade table recovery instead of dropping possible preserved data.

No frontend/Tauri command surface was added; P1.16 remains separate.

## Changed files

- `src-tauri/src/lib.rs`
  - Added confirmation types, create/read/list helpers, execution guard, actor-satisfaction logic, redaction integration, migration registration, and unit/regression tests.
- `src-tauri/migrations/0003_core_schema_p105.sql`
  - Fresh schema now constrains `confirmation_decisions.actor_type` to known actor types.
- `src-tauri/migrations/0004_confirmation_actor_type_check.sql`
  - New forward migration rebuilds `confirmation_decisions` with actor_type CHECK for already-applied v3 databases.
  - Uses an explicit transaction and no pre-drop of upgrade table.

## Tests run

Parent/orchestrator verification:

- `npm run verify:local`: PASS
  - Rust tests: 59 passed, 0 failed
  - Frontend build: PASS
  - Local push verification: PASS (`--skip-package`)

Implementation/fix verification from subagents:

- `cargo test confirmation --lib`: PASS, 11 passed after first fix cycle
- `cargo test migrations --lib`: PASS, 6 passed after actor CHECK migration fix
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 56 passed after first fix cycle
- `cargo clippy --all-targets --all-features -- -D warnings`: PASS
- `cargo test migrations_upgrade_existing_v3_confirmation_decisions_actor_type_check --lib`: PASS
- `cargo test migration_v4 -- --nocapture`: PASS, 2 passed
- `cargo test migrations_ -- --nocapture`: PASS, 7 passed
- `cargo test`: PASS, 59 passed after migration atomicity fix

Independent re-review focused checks:

- `cargo test confirmation -- --nocapture`: PASS, 12 passed
- `cargo test migration_v4 -- --nocapture`: PASS, 2 passed
- `cargo test migration_v4 --lib`: PASS, 2 passed
- `cargo test migrations_upgrade_existing_v3_confirmation_decisions_actor_type_check --lib`: PASS
- `cargo test migrations_reject_invalid_confirmation_actor_type --lib`: PASS
- `cargo clippy --lib --tests -- -D warnings`: PASS
- `cargo test --lib`: PASS, 59 passed

## Git info

- Branch: `main`
- Implementation commits:
  - `40a40e2` Implement confirmation decision framework
  - `f156de8` Fix confirmation review gaps
  - `e71a8b4` Fix confirmation actor type migration upgrade
  - `03a660b` Fix migration v4 atomic rebuild

## Frontend/backend/database notes

- Frontend: no changes.
- Tauri command surface: no new command added.
- Backend:
  - New internal confirmation framework and guard for future protected actions.
  - Guard expects a policy decision from P1.12; it fails closed when missing.
- Database:
  - Uses existing `confirmation_decisions` table from migration v3.
  - Fresh v3 schema now constrains actor_type.
  - v4 safely upgrades already-applied v3 DBs to the constrained actor_type schema.
  - v4 preserves valid rows and indexes; invalid pre-existing actor types fail the migration rather than silently upgrading unsafe data.

## Review cycle notes

Independent review found and fixes addressed:

1. Standalone secret material could persist in summaries/metadata.
   - Fixed via `looks_like_secret_material` in line and JSON redaction paths.
2. `system` actor could satisfy human-confirmation policies.
   - Fixed actor satisfaction rules; system no longer satisfies human confirmation.
3. Clear-task + reviewer requirements were unsatisfiable with one confirmation record.
   - Added `reviewed_clear_task` actor type; tests prove only combined evidence passes both requirements.
4. `confirmation_decisions.actor_type` lacked a SQLite CHECK constraint.
   - Added fresh-schema constraint and v4 upgrade migration.
5. Migration v4 was initially non-atomic and could drop a leftover upgrade table.
   - Fixed with explicit transaction and fail-closed leftover-table behavior.

Latest independent re-review verdicts before final critique:

- Spec compliance: PASS
- Code quality/security: APPROVED

## Reviewer focus areas

- Confirm no protected action can pass without an `ActionPolicyDecision`.
- Confirm actor semantics cannot bypass human/reviewer/clear-task gates.
- Confirm secret redaction covers summaries and metadata JSON without breaking JSON validity.
- Confirm migration v4 handles already-applied v3 databases safely and atomically.
- Confirm no frontend/Tauri command scope creep.