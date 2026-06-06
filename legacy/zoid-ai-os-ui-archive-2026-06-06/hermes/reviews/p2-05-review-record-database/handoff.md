# Feature Handoff: P2.05 ReviewRecord database

## Original request

Continue Zoid Phase 2 and implement P2.05 Database: ReviewRecord table/model with manual reviewer stub fields and links to task/run.

## Implementation summary

- Added Phase 2 migration `0007_phase2_review_records.sql`.
- Registered migration version 7 as `phase2_review_records`.
- Added backend/database-only ReviewRecord repository/model primitives.
- Added `ReviewSubjectType` and `ReviewVerdict`.
- Added `ReviewRecord` and `ReviewRecordCreateInput`.
- Added `review_records` table for task and agent-run reviews.
- Manual reviewer stub is supported through nullable `reviewer_profile_id`.
- Review records include:
  - subject type/id;
  - task_id;
  - optional run_id;
  - optional reviewer_profile_id;
  - review state;
  - verdict;
  - evidence summary;
  - required fixes JSON;
  - metadata JSON;
  - timestamps.
- Created review events:
  - `review.created`
  - `review.approved`
  - `review.required_fixes`
  - `review.blocked_insufficient_evidence`
- Event targets include review_record primary, task owner, and agent_run when present.
- Durable entity links are created for task -> review_record and agent_run -> review_record where applicable.
- Added `review_gate_satisfied_for_task` for backend gate semantics.
- Related entity reviews are rejected for now because verifiable typed related-entity support is not yet implemented. Both repository and DB schema reject them.

## Changed files

- `src-tauri/migrations/0007_phase2_review_records.sql`
  - Creates `review_records` table and indexes.
  - Restricts `subject_type` to `task` and `agent_run` until verifiable related entity support exists.
- `src-tauri/src/lib.rs`
  - Registers migration version 7.
  - Adds ReviewRecord enums/models/repository helpers.
  - Adds P2.05 tests.
- `.hermes/reviews/p2-05-review-record-database/handoff.md`
  - This handoff.

## How to test

Run from `/Users/ziadnasreldin/Zoid`:

- `cargo test --manifest-path src-tauri/Cargo.toml p205 -- --nocapture`
- `cargo test --manifest-path src-tauri/Cargo.toml review`
- `cargo test --manifest-path src-tauri/Cargo.toml`

Expected:

- P2.05 focused tests pass.
- Review-filtered tests pass.
- Full backend test suite passes.

## Tests run

Initial implementer RED:

- `cargo test --manifest-path src-tauri/Cargo.toml p205 -- --nocapture`
- Result: failed as expected before implementation.
- Missing symbols included:
  - `ReviewRecordCreateInput`
  - `ReviewSubjectType`
  - `ReviewVerdict`
  - `create_review_record`
  - `review_gate_satisfied_for_task`

Initial GREEN after implementation:

- `cargo test --manifest-path src-tauri/Cargo.toml p205 -- --nocapture`
- Result: 5 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml review`
- Result: 8 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml`
- Result: 105 passed, 0 failed.

Lean review first result: REQUEST_CHANGES.

Required fixes:

1. `review_gate_satisfied_for_task` returned true if any approved review existed, even after a later blocking review.
2. `related_entity` reviews allowed arbitrary unverifiable subject IDs.

Regression RED after adding required-fix tests:

- `cargo test --manifest-path src-tauri/Cargo.toml p205 -- --nocapture`
- Result: failed as expected.
- Failures:
  - `p205_approved_review_satisfies_gate_and_blocking_verdicts_do_not`
  - `p205_related_entity_review_is_rejected_until_verifiable_subject_support_exists`

Fixes applied:

- `review_gate_satisfied_for_task` now checks the latest review for a task and returns true only when latest state/verdict are approved.
- `validate_review_record_input` rejects `ReviewSubjectType::RelatedEntity` until verifiable subject support exists.

Lean re-review second result: REQUEST_CHANGES.

Required fix:

- DB migration still allowed direct SQL `subject_type = 'related_entity'` inserts.

Regression RED after adding DB-layer direct insert test:

- `cargo test --manifest-path src-tauri/Cargo.toml p205_schema_version_seven_has_review_records_table -- --nocapture`
- Result: failed as expected because the schema allowed direct related_entity insert.

Fix applied:

- `0007_phase2_review_records.sql` now restricts `subject_type` to `('task', 'agent_run')`.

Final local verification:

- `cargo fmt --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml p205 -- --nocapture`: 6 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml review`: 9 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 106 passed, 0 failed; doc-tests 0.

Lean re-review final result: PASS.

## Git info

- Branch: `main`
- Commit SHA: pending at handoff time.
- Diff base: current `HEAD` after P2.04 commit `1e61e9c feat: add Phase 2 agent run repository`.

## Frontend/backend/database notes

- Frontend routes/components: not touched.
- Tauri bridge commands: not touched.
- Backend/database:
  - Adds internal repository/model layer only.
  - Does not expose review commands to UI yet.
- Migration notes:
  - Fresh migrated in-memory DB now reports version 7.
  - Foreign keys link review records to tasks, agent_runs, and optional reviewer agent_profiles.
  - Related entity review support is intentionally deferred until typed/verifiable related-entity targets exist.

## Reviewer focus areas

- Confirm scope is backend/database-only.
- Confirm `review_records` migration and version 7 registration.
- Confirm manual reviewer stub works with nullable `reviewer_profile_id`.
- Confirm task/run review validation is truthful.
- Confirm required fixes and insufficient evidence cannot satisfy the review gate.
- Confirm latest blocking review invalidates a previously approved gate.
- Confirm both repository and schema reject unverifiable `related_entity` reviews.
- Confirm events and durable links are written correctly.
- Confirm no raw secrets are stored in metadata, required fixes, or event summaries.

## Fix cycle notes

Two lean review cycles occurred before final critique:

1. Fixed latest-review gate semantics and repository related_entity rejection.
2. Fixed DB-layer related_entity rejection.

All required lean-review fixes passed re-review. Final critique initially returned REQUEST_CHANGES.

Final critique required fixes:

1. DB-layer direct insert could create an agent-run review whose `run_id` belonged to a different `task_id`.
2. Repository validation accepted non-array `required_fixes_json` shapes that the DB constraint rejected.
3. DB schema allowed contradictory `state`/`verdict` pairs.

Final critique fixes applied:

- Added direct-SQL regression in `p205_schema_version_seven_has_review_records_table` proving mismatched run/task review inserts fail.
- Added SQLite triggers `trg_review_records_agent_run_task_match_insert` and `trg_review_records_agent_run_task_match_update` to reject `agent_run` reviews when `agent_runs.id/run_id` does not belong to the provided `task_id`.
- Renamed/tightened repository validation to require `required_fixes_json` to be a non-empty JSON array for `required_fixes` verdicts.
- Added repository regression proving object-shaped required fixes payloads are rejected before persistence.
- Added DB `check (state = verdict)` and direct-SQL regression proving contradictory state/verdict rows fail.

Post-final-critique-fix verification:

- `cargo fmt --manifest-path src-tauri/Cargo.toml`: passed.
- `cargo test --manifest-path src-tauri/Cargo.toml p205 -- --nocapture`: 6 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml review`: 9 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: 106 passed, 0 failed; doc-tests 0.