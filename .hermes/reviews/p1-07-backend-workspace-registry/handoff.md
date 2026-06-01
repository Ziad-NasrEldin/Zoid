# Feature Handoff: P1.07 Backend workspace registry

## Original request

Phase 1 / Backend: Implement canonical workspace registry service listing all 14 workspaces with truthful availability/integration states.

Current user instruction: continue as orchestrator and delegate implementation/review tasks to subagents.

## Implementation summary

- Replaced the loose backend workspace tuple constant with a canonical static backend registry.
- Added registry/model types for workspace metadata and truthful state reporting:
  - `WorkspaceDefinition`
  - `WorkspaceRecord`
  - `WorkspaceAvailability`
  - `WorkspaceIntegration`
  - `WorkspaceIntegrationState`
- Registry lists all 14 workspaces in canonical order:
  - `today`
  - `tasks`
  - `notes`
  - `agents`
  - `code`
  - `content`
  - `automations`
  - `business`
  - `products`
  - `files`
  - `browser`
  - `inbox`
  - `calendar`
  - `history`
- Added truthful availability/integration metadata without probing or claiming unavailable integrations:
  - Gmail: `not_configured`
  - Apple Calendar: `needs_permission`
  - OmniSocials: `planned`
  - Browser webview/capture: `planned`
  - Hermes CLI / Git CLI / Automation CLI: `not_configured`
- Kept implementation backend-only. No new P1.16 Tauri command surface was added.
- Existing foundation status path now serializes enriched workspace records, while frontend remains compatible because extra fields are allowed structurally by TypeScript.
- DB seeding/listing now derives from the canonical registry rather than a separate tuple source.
- No secrets, external calls, OAuth checks, EventKit prompts, git probing, CLI probing, or integration credentials were introduced.

## Changed files

- `src-tauri/src/lib.rs`
  - Added canonical workspace registry model/state types.
  - Updated seeding/listing to use registry-backed metadata.
  - Added tests for registry order/count/truthful states and DB seeding/listing behavior.

## How to test

From `/Users/ziadnasreldin/Zoid`:

- `cargo test workspace --manifest-path src-tauri/Cargo.toml`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`
- `npm run verify:local`

Expected behavior:

- Exactly 14 unique workspaces are returned in canonical order.
- Workspace records include availability/status notes and integration state metadata.
- Guarded integrations never claim connected/ready/functional state in this phase.
- Workspace DB rows are seeded idempotently from the canonical registry.
- Frontend build remains compatible.

## Tests run

Implementation subagent TDD evidence:

- RED:
  - `cargo test canonical_workspace_registry --lib`
  - Failed before implementation with missing registry/model/function errors, including:
    - `cannot find function canonical_workspace_ids`
    - `cannot find function canonical_workspace_registry`
    - `no field availability on type &WorkspaceRecord`
    - `cannot find type WorkspaceAvailability`
    - `cannot find type WorkspaceIntegrationState`
- GREEN focused:
  - `cargo test workspace_registry --lib`: PASS, 2 passed.
  - `cargo test workspace_records_list_truthful_registry_metadata_for_foundation_ui --lib`: PASS, 1 passed.
  - `cargo test workspace_seeding_is_registry_backed_and_idempotent --lib`: PASS, 1 passed.
- Full checks from implementation subagent:
  - `cargo test --lib`: PASS, 33 passed.
  - `cargo clippy --lib -- -D warnings`: PASS.
  - `cargo test`: PASS, 33 passed.
  - `cargo clippy -- -D warnings`: PASS.

Parent/orchestrator verification:

- `npm run verify:local`: PASS.
  - Rust tests: 33 passed, 0 failed.
  - Frontend build: PASS.

Independent reviews:

- Spec compliance review: PASS.
  - Reviewer ran `cargo test workspace` under `src-tauri`: PASS, 5 passed.
- Code quality review: APPROVED.
  - Reviewer ran `cargo test workspace --manifest-path src-tauri/Cargo.toml`: PASS, 5 passed.

## Git info

- Branch: main
- Commit: `6f20870 Implement canonical workspace registry`
- Diff base: `d55af2e Record P1.06 repository primitives review`

## Frontend/backend/database notes

- Frontend routes/components:
  - No frontend code changed.
  - Existing `WorkspaceRecord` TypeScript type remains compatible with enriched backend payload because it uses a narrower structural shape.
- Backend services:
  - Canonical backend workspace registry added internally.
  - No new external integration probing/service calls.
- Database:
  - No new migration.
  - Existing `workspaces` table is seeded from canonical registry definitions.
  - Runtime listing enriches DB rows with canonical registry availability/integration metadata.
- Security/privacy:
  - No secrets read or written.
  - No OAuth/EventKit/API permission checks or prompts.
  - No integration is reported as connected/ready unless it is only a local foundation surface.

## Reviewer focus areas

- Confirm exactly 14 canonical workspaces and correct order.
- Confirm guarded integrations remain truthful and unconnected/unconfigured/planned/needs-permission.
- Confirm DB seeding/listing is registry-backed and idempotent.
- Confirm scope does not accidentally implement P1.08 settings or P1.16 Tauri command surface.
- Confirm serialization remains stable and frontend build remains compatible.

## Fix cycle notes

- Initial implementation passed spec review and quality review.
- No required fixes are currently open.
