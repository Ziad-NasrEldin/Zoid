# Feature Handoff: P1.12 Backend action policy evaluator

## Original request

Phase 1 / Backend-Security: Implement action policy evaluator for read/create/update/delete/send/publish/deploy/file/process actions with confirmation/review requirements.

Current session instruction: continue as orchestrator and use subagents for implementation/review.

## Implementation summary

- Added a typed backend action-policy request/evaluator surface in `src-tauri/src/lib.rs`:
  - `ActionRequest`
  - `ActionType`: read/create/update/delete/send/publish/deploy/file/process/unknown
  - `ActionScope`
  - `ActionConsequence`
  - `evaluate_action_request(&ActionRequest) -> ActionPolicyDecision`
- Expanded `ActionPolicyDecision` with explicit gate booleans:
  - `allowed_now`
  - `requires_confirmation`
  - `requires_reviewer`
  - `requires_clear_task`
  - `requires_gate`
- Generic typed requests now classify into canonical policy categories rather than only pre-baked category strings.
- Consequential actions are gated with confirmation/review/clear-task requirements as appropriate:
  - email send
  - content publish/schedule
  - deploy/redeploy/rollback
  - git commit/push/merge
  - destructive and bulk file operations
  - credential/settings/integration changes
  - process/automation execution or schedule changes
  - calendar creates/edits/deletes
  - external API writes
- Harmless local reads and clearly local/private creates can be allowed immediately.
- Unknown/unsafe actions fail closed as `BlockUntilConfirmed` with `allowed_now = false`.
- `action_policies` seeding remains generated from the Rust evaluator/category list via `seed_action_policies`; policy matrix is not duplicated in SQL.
- No new Tauri command surface.
- No frontend changes.
- No external calls/probes.

## Fix cycle summary

Initial independent reviews found classifier gaps:

1. Generic external/integration create requests could fall through to `create_local_task` and be allowed.
2. `automation schedule` could be shadowed by generic publish/schedule matching.
3. Generic `bulk` matching could shadow high-risk typed actions such as bulk email.

Fix commit `a3d2769` addressed these by:

- Moving specific high-risk typed categories ahead of generic bulk/destructive matching.
- Moving automation schedule classification before generic publish/schedule/content matching.
- Restricting generic bulk-file classification to file-like requests via `is_bulk_file_request`.
- Gating non-read external/integration-scoped actions and `remote record` targets as `external_api_write`.
- Adding regression tests for the review gaps.

Re-review results:

- Spec compliance re-review: PASS.
- Code quality/security re-review: APPROVED.

## Changed files

- `src-tauri/src/lib.rs`
  - Added typed action request/scoping/consequence enums and builder helpers.
  - Added `evaluate_action_request` and request classification logic.
  - Expanded `ActionPolicyDecision` gate booleans.
  - Added `external_api_write` canonical category.
  - Kept seeding from evaluator source of truth.
  - Added action policy tests and classifier precedence regression tests.

## How to test

From `/Users/ziadnasreldin/Zoid`:

- `cargo test action_policy --manifest-path src-tauri/Cargo.toml --lib -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- `npm run verify:local`

Expected behavior:

- Harmless local reads/creates return `allowed_now = true`.
- Sends/publishes/deploys/processes/deletes/bulk file operations/external writes are not allowed immediately.
- Unknown actions/categories fail closed.
- `automation schedule` classifies as `change_automation_schedule`.
- `bulk email` and `mass email` remain `send_email` and preserve `HumanConfirmation::Always`.
- External/integration creates classify as `external_api_write`, not `create_local_task`.
- Seeded `action_policies` rows match evaluator output.

## Tests run

Implementation subagent TDD evidence:

- RED for initial implementation:
  - `cargo test action_policy --lib`
  - Failed to compile as expected because `ActionRequest`, `ActionType`, `ActionConsequence`, and `evaluate_action_request` did not exist yet.
- GREEN initial:
  - `cargo test action_policy --lib`: PASS, 5 passed.
  - `cargo test`: PASS, 44 passed.
  - `cargo clippy --all-targets --all-features -- -D warnings`: PASS.

Review-fix TDD evidence:

- RED:
  - `cargo test action_policy_classifier -- --nocapture`
  - Failed as expected:
    - `automation schedule` classified as `publish_schedule_content`
    - external create classified as `create_local_task`
- GREEN focused:
  - `cargo test action_policy_classifier -- --nocapture`: PASS, 2 passed.
  - `cargo test action_policy -- --nocapture`: PASS, 7 passed.
- Full Rust:
  - `cargo test`: PASS, 46 passed.
- Clippy:
  - `cargo clippy --all-targets --all-features -- -D warnings`: PASS.

Parent/orchestrator verification:

- `npm run verify:local`: PASS.
  - Rust tests: 46 passed, 0 failed.
  - Frontend build: PASS.

Independent reviews:

- Initial spec review: GAP FOUND; fixed in `a3d2769`.
- Initial code quality/security review: REQUEST_CHANGES; fixed in `a3d2769`.
- Spec re-review: PASS.
  - Reviewer ran `cargo test action_policy -- --nocapture`: PASS, 7 passed.
- Code quality/security re-review: APPROVED.
  - Reviewer ran:
    - `cargo test action_policy -- --nocapture`: PASS, 7 passed.
    - `cargo clippy --all-targets --all-features -- -D warnings`: PASS.
    - git diff/status checks: clean.

## Git info

- Branch: main
- Implementation commit: `702424b Implement backend action policy evaluator`
- Fix commit: `a3d2769 Fix action policy classifier precedence`
- Diff base for feature: `d9c9629 Record P1.11 safe logging review`

## Frontend/backend/database notes

- Frontend:
  - No frontend changes.
  - Existing TypeScript/Vite build passes.
- Backend:
  - Backend-only evaluator/service helper code added in Rust.
  - No new invoke command; policy command exposure remains for later P1.16.
- Database:
  - Existing `action_policies` table used.
  - No migration changes.
  - Seeding remains idempotent and generated from Rust evaluator/category list.
  - Regression test verifies seeded rows match evaluator output.
- Security:
  - Unknown and unsafe requests fail closed.
  - Consequential actions are gated before future execution surfaces can use them.
  - This task provides the evaluator; P1.13 will implement confirmation decision records/framework that protected actions must use before execution.

## Reviewer focus areas

- Ensure typed request classification does not accidentally downgrade high-risk actions into allowed local categories.
- Ensure gate booleans are internally consistent.
- Ensure `action_policies` seeding is still from the single evaluator source and not duplicated in SQL.
- Ensure no Tauri/frontend scope creep before P1.16.

## Fix cycle notes

- Required fixes from initial reviews have been addressed and re-reviewed.
- Current re-review status: spec PASS and quality/security APPROVED.
