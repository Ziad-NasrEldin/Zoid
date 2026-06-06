# Feature Handoff: Phase 4 Code, Repos, Launch Gate

## Original request

Resume Phase 4 closeout for Zoid AI OS: verify current Phase 4 implementation after restored files, write the review handoff, complete critique workflow until approval, and update the Phase 4 tracker with exact evidence.

## Implementation summary

- Restored/current on-disk Phase 4 files are present after prior context-compression/source-reconciliation blocker.
- Adds a lightweight native repo registry backed by SQLite `repo_profiles`.
- Adds repo-to-task/product linking through existing entity links with Phase 4 validation.
- Adds Launch Gate records and evidence records. Launch gates start blocked and only become verified after evidence exists.
- Adds GitHub/Vercel integration state surfaces that are truthful `not_configured` state only; no fake connected state and no external deploy execution.
- Adds policy preview for launch-sensitive actions such as commit/push/merge/deploy.
- Adds Tauri bridge commands for repo profile CRUD/list/link, integration-state listing, launch gate create/read/evidence/evaluate, and launch action policy preview.
- Adds Code workspace UI that reads native repo/integration/policy state and shows truthful blocked/unconfigured state.

Scope truth after first critique:

- Per user decision during closeout, Phase 4 is formally narrowed to lightweight native repo registry + truthful integration states + Launch Gate evidence/policy-preview behavior.
- Full git status/diff/read operations, external GitHub/Vercel automation, and deploy execution are explicitly deferred to a later dedicated slice.
- GitHub/Vercel are not connected or executed; state detection is local/truthful only.
- Manual/source verification is represented by source reconciliation plus focused backend tests for repo add/list/read/link and Launch Gate fail-closed behavior; full native UI add/status/diff evidence is no longer part of the narrowed Phase 4 acceptance.
- There are unrelated/unreviewed Phase 5 files currently present in the working tree: `src-tauri/migrations/0011_phase5_content_omnisocials.sql` and `src-tauri/src/phase5_service.rs`, plus Phase 5 references in `src/App.tsx`/`src-tauri/src/lib.rs`/tests. Review Phase 4 independently and flag if this still blocks closeout.

## Changed files

- `src-tauri/migrations/0010_phase4_code_repos_launch_gate.sql`: creates repo profiles, launch gates, and launch gate evidence tables/indexes.
- `src-tauri/src/phase4_service.rs`: repo profile/link/integration-state/Launch Gate service records and validations.
- `src-tauri/src/lib.rs`: registers migration v10; exposes Phase 4 Tauri commands; includes command-surface registration.
- `src-tauri/src/tests.rs`: adds focused Phase 4 schema, repo registry/link, truthful integration, launch policy, evidence/fail-closed, and command-surface tests.
- `src/App.tsx`: adds Code workspace loading/error/ready UI and native invokes for repo registry, integration state, and launch policy preview.

## How to test

From `/Users/ziadnasreldin/Zoid`:

- `cargo test --manifest-path src-tauri/Cargo.toml p4 -- --nocapture`
- `npm run verify:local`

Expected behavior:

- Phase 4 focused Rust tests pass.
- Full local verification passes Rust tests, frontend tests, and frontend build.
- Source reconciliation confirms `src-tauri/src/phase4_service.rs`, migration v10 SQL, command registration, and Code workspace UI are all present.

## Tests run

- `cargo test --manifest-path src-tauri/Cargo.toml p4 -- --nocapture`: PASS — 5 passed, 0 failed, 172 filtered out.
- `npm run verify:local`: PASS before scope-doc update — Rust tests 176 passed, 0 failed, 1 ignored; frontend test suite passed; `tsc && vite build` passed; local push verification passed with `--skip-package`.
- Source consistency check after a short wait: PASS — `RepoProfileInput` exists in `phase4_service.rs`, `list_repo_profiles_command` exists in `lib.rs`, `CodeWorkspace` exists in `App.tsx`, and migration `0010_phase4_code_repos_launch_gate.sql` exists.
- `npm run verify:local`: PASS after plan/tracker/handoff scope updates — Rust tests 176 passed, 0 failed, 1 ignored; frontend tests passed; `tsc && vite build` passed; local push verification passed with `--skip-package`.

## Git info

- Branch: `main`
- Commit SHA: not committed in this handoff.
- Current working tree at handoff time includes:
  - Modified Phase 4 files: `src-tauri/migrations/0010_phase4_code_repos_launch_gate.sql`, `src-tauri/src/lib.rs`, `src-tauri/src/phase4_service.rs`, `src-tauri/src/tests.rs`, `src/App.tsx`
  - Untracked/future Phase 5 files: `src-tauri/migrations/0011_phase5_content_omnisocials.sql`, `src-tauri/src/phase5_service.rs`

## Frontend/backend/database notes

- Frontend routes/components: Code workspace in `src/App.tsx`, selected when workspace id is `code`.
- Frontend bridge invokes: `list_repo_profiles_command`, `list_repo_integration_states_command`, `preview_launch_action_policy_command`.
- Backend commands: `add_repo_profile_command`, `list_repo_profiles_command`, `read_repo_profile_command`, `link_repo_entity_command`, `list_repo_integration_states_command`, `create_launch_gate_command`, `read_launch_gate_command`, `add_launch_gate_evidence_command`, `evaluate_launch_gate_command`, `preview_launch_action_policy_command`.
- Database tables/migrations: migration v10 `phase4_code_repos_launch_gate` creates `repo_profiles`, `launch_gates`, and `launch_gate_evidence`.

## Reviewer focus areas

- Whether Phase 4 narrowing is now explicit enough: lightweight repo registry + truthful integration state + Launch Gate evidence/policy preview only; real git status/diff/read and deploy execution deferred.
- Whether Launch Gate fails closed until evidence exists and avoids fake verification under the narrowed coarse evidence-presence gate.
- Whether commit/push/merge/deploy are policy previews only and do not execute protected actions.
- Whether frontend copy truthfully avoids fake GitHub/Vercel connected state.
- Whether current Phase 5 untracked/source changes still contaminate Phase 4 closeout or can be treated as separate pending future work.
- Whether tracker updates truthfully record exact evidence after the user-approved scope narrowing.

## Fix cycle notes

First critique returned `REQUEST_CHANGES`. Fixes applied after user decision:

- Formalized narrowed Phase 4 scope in `Docs/2026-05-31-zoid-implementation-plan-v1.md`: lightweight repo registry, truthful integration state, Launch Gate evidence/policy preview; full git status/diff/read and deploy execution deferred.
- Updated `Docs/2026-06-01-zoid-implementation-tracker.md` Phase 4 checklist with exact evidence for the narrowed scope and left P4.22 pending re-review.
- Updated this handoff to reflect the scope decision and current review focus.
