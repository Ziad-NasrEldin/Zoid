# Feature Handoff: P2.14 manual ReviewRecord creation service

## Original request

Continue from handoff for Zoid Phase 2. Next recommended task: **P2.14 Backend: manual ReviewRecord creation service and reviewer-profile placeholder if available**.

## Implementation summary

- Added a backend-only `review_service` module for manual ReviewRecord creation.
- Added `ManualReviewServiceCreateInput` and `create_manual_review_service`.
- The service infers review subject from input:
  - `run_id: None` creates a task-level review.
  - `run_id: Some(...)` creates an agent-run review for that run.
- If `reviewer_profile_id` is explicitly supplied, it is passed through to the repository for existing validation.
- If no reviewer is supplied, the service attaches placeholder agent profile `manual-reviewer` when that profile exists.
- If the placeholder does not exist, the service keeps `reviewer_profile_id = None`, preserving the current manual-stub behavior.
- Existing repository validation still owns task/run ownership, required fixes payload shape, reviewer profile existence, secret redaction, entity links, and review events.

## Changed files

- `src-tauri/src/lib.rs`: registers and re-exports the new review service module.
- `src-tauri/src/review_service.rs`: new thin service wrapper around the approved ReviewRecord repository.
- `src-tauri/src/tests.rs`: adds P2.14 service tests for placeholder profile usage, missing-placeholder behavior, task/run subject inference, event actor attribution, and validation preservation.

## How to test

From repo root `/Users/ziadnasreldin/Zoid`:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p214 -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml review -- --nocapture
cargo test --manifest-path src-tauri/Cargo.toml
```

Expected behavior:
- P2.14 focused tests pass.
- Existing review repository/service-related tests pass.
- Full Rust suite passes.

## Tests run

- RED before implementation: `cargo test --manifest-path src-tauri/Cargo.toml p214 -- --nocapture` — failed as expected with missing `ManualReviewServiceCreateInput` and `create_manual_review_service`.
- GREEN: `cargo test --manifest-path src-tauri/Cargo.toml p214 -- --nocapture` — PASS, 2 passed.
- `cargo fmt --manifest-path src-tauri/Cargo.toml && cargo test --manifest-path src-tauri/Cargo.toml review -- --nocapture` — PASS, 15 passed.
- Full Rust suite: `cargo test --manifest-path src-tauri/Cargo.toml` — PASS, 122 passed, 0 failed, doc-tests 0.
- Lean combined review via separate reviewer — PASS, no required fixes.

## Git info

- Branch: `main`
- Current base before commit: `d169d7f feat: add Phase 2 agent execution service`
- Working tree at handoff creation includes unstaged/untracked P2.14 changes.

## Frontend/backend/database notes

- Frontend routes/components: not touched.
- Backend services: new `src-tauri/src/review_service.rs`.
- Database tables/migrations: no migration changes. Uses existing P2.05 `review_records` and existing `agent_profiles` placeholder lookup.
- Repository/data safety: service delegates to existing `create_review_record` so validation, redaction, event creation, and entity links remain centralized.

## Reviewer focus areas

- Confirm `src-tauri/src/review_service.rs` is included/tracked and exported.
- Confirm the service is intentionally thin and does not bypass ReviewRecord repository validation.
- Confirm placeholder `manual-reviewer` behavior is optional and does not create a fake profile or require profile configuration.
- Confirm run reviews still fail if the run does not belong to `task_id` through existing repository validation.
- Confirm tests cover both available and unavailable placeholder cases.

## Fix cycle notes

Initial handoff; no critique-required fixes yet.
