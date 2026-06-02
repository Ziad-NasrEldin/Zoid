# P2.14 Manual Review Service Critique Report

Verdict: APPROVED

## Scope reviewed
- P2.14 backend manual `ReviewRecord` creation service and optional reviewer-profile placeholder behavior.
- Handoff reviewed: `.hermes/reviews/p2-14-manual-review-service/handoff.md`.
- Current git status/diff reviewed, including untracked files.

## Evidence
- Changed implementation files reviewed:
  - `src-tauri/src/lib.rs`
  - `src-tauri/src/review_service.rs`
  - `src-tauri/src/tests.rs`
- Git status at review time showed expected P2.14 changes only: modified `src-tauri/src/lib.rs`, modified `src-tauri/src/tests.rs`, untracked `.hermes/reviews/p2-14-manual-review-service/handoff.md`, and untracked `src-tauri/src/review_service.rs`.
- `src-tauri/src/lib.rs` registers and re-exports `review_service`.
- `src-tauri/src/review_service.rs` implements a thin `create_manual_review_service` wrapper that:
  - Infers task review when `run_id` is `None`.
  - Infers agent-run review when `run_id` is supplied.
  - Passes an explicit `reviewer_profile_id` through unchanged.
  - Falls back to `manual-reviewer` only when `read_agent_profile` finds that profile.
  - Delegates persistence to `create_review_record`, preserving repository validation, redaction, events, and entity links.
- Existing repository validation in `src-tauri/src/lib.rs` still enforces task/run ownership, required fixes payload shape, reviewer profile existence, and subject consistency before insert.
- Added P2.14 tests cover placeholder profile attachment, missing-placeholder `None` behavior, task/run subject inference, event actor attribution, and preservation of repository guard behavior for required fixes.

## Independent verification run
- `git diff --check` — PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml p214 -- --nocapture` — PASS: 2 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml review -- --nocapture` — PASS: 15 passed, 0 failed.
- `cargo test --manifest-path src-tauri/Cargo.toml` — PASS: 122 passed, 0 failed; doc-tests 0 passed, 0 failed.

## Findings
- The implementation satisfies the requested backend-only service shape without modifying production repository behavior.
- Optional placeholder behavior is correctly non-creating/non-requiring: if `manual-reviewer` is unavailable, manual reviews are created with `reviewer_profile_id = None`.
- Explicit reviewer profile handling remains repository-validated because the service passes the supplied id directly to `create_review_record`.
- Run ownership mismatch remains guarded by existing repository validation even though the service pre-reads the run to infer the subject id.

## Required fixes
None.
