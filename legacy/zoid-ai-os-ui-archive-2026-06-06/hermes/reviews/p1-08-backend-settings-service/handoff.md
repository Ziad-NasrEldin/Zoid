# Feature Handoff: P1.08 Backend settings service

## Original request

Phase 1 / Backend: Implement settings service for local app preferences and integration statuses without storing secrets outside Keychain.

Current user instruction: continue as orchestrator and delegate implementation/review tasks to subagents.

## Implementation summary

- Added backend settings service primitives in `src-tauri/src/lib.rs` for local non-secret app preferences.
- Added typed settings models:
  - `SettingScope`
  - `LocalPreferenceInput`
  - `LocalPreferenceRecord`
- Added service helpers for local preferences:
  - upsert/read/list/list-by-scope over existing `app_settings` repository/table.
  - JSON validation before write.
  - scope constrained to local app/workspace preferences.
  - secret-like keys and secret-like JSON values rejected before SQLite writes.
- Added backend integration-status service primitives over existing `integration_statuses` table.
- Added typed integration status models:
  - `IntegrationStatus`
  - `IntegrationStatusInput`
  - `IntegrationStatusRecord`
- Added integration status helpers:
  - upsert/read/list.
  - config JSON validation before write.
  - obvious secret-like config keys/values rejected before SQLite writes.
  - raw-looking credential refs rejected; safe Keychain-style reference strings allowed.
- Added default integration-status seeding from the P1.07 canonical workspace registry:
  - Registry `not_configured` / `needs_permission` -> DB `not_configured`.
  - Registry `planned` / `blocked` -> DB `disabled`.
  - Seeded rows do not claim connected/configured state and do not include credentials.
  - Seeding uses insert-or-ignore and does not overwrite explicit safe user/service updates.
- `ensure_foundation` now seeds default integration statuses after workspace seeding.
- No schema changes.
- No frontend changes.
- No new Tauri commands; P1.16 remains separate.
- No Keychain writes in this phase; the service only allows safe credential reference strings.
- No OAuth/EventKit/git/CLI/API probes or external calls.

## Changed files

- `src-tauri/src/lib.rs`
  - Added settings service and integration-status service models/helpers.
  - Added secret-safety validation helpers.
  - Added default integration-status seeding from canonical registry.
  - Added P1.08 tests.

## How to test

From `/Users/ziadnasreldin/Zoid`:

- `cargo test settings_service --manifest-path src-tauri/Cargo.toml --lib`
- `cargo test integration_status_service --manifest-path src-tauri/Cargo.toml --lib`
- `cargo test --manifest-path src-tauri/Cargo.toml`
- `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
- `npm run verify:local`

Expected behavior:

- Non-secret local app/workspace preferences can be saved, read, and listed.
- Invalid JSON is rejected as typed `RepositoryError::InvalidJson`.
- Secret-like preference keys/config keys/values are rejected as typed `RepositoryError::SecretRejected` before DB writes.
- Integration statuses can be safely upserted/read/listed with validated non-secret config.
- Raw-looking credential refs are rejected; safe Keychain reference labels are accepted.
- Default integration status seeding is idempotent and truthful.
- Seeded integrations do not claim connected/configured credentials.
- Explicit safe integration status updates survive reseeding.

## Tests run

Implementation subagent TDD evidence:

- RED:
  - `cargo test service --lib`
  - Failed before implementation with expected missing symbols, including:
    - `LocalPreferenceInput`
    - `SettingScope`
    - local preference upsert/read/list helpers
    - `IntegrationStatus`
    - `IntegrationStatusInput`
    - `seed_default_integration_statuses`
    - `RepositoryError::SecretRejected`
- GREEN focused:
  - `cargo test service --lib`: PASS, 5 passed.
- Full Rust:
  - `cargo test`: PASS, 38 passed.
- Clippy:
  - `cargo clippy --all-targets --all-features -- -D warnings`: PASS.

Parent/orchestrator verification:

- `npm run verify:local`: PASS.
  - Rust tests: 38 passed, 0 failed.
  - Frontend build: PASS.

Independent reviews:

- Spec compliance review: PASS.
  - Reviewer ran:
    - `cargo test settings_service --lib`: PASS, 2 passed.
    - `cargo test integration_status_service --lib`: PASS, 3 passed.
- Code quality/security review: APPROVED.
  - Reviewer ran:
    - `cargo test settings_service --lib`: PASS, 2 passed.
    - `cargo test integration_status_service --lib`: PASS, 3 passed.
    - `cargo clippy --all-targets --all-features -- -D warnings`: PASS.

## Git info

- Branch: main
- Commit: `6d4b8e5 Implement backend settings service`
- Diff base: `8b01f67 Record P1.07 workspace registry review`

## Frontend/backend/database notes

- Frontend routes/components:
  - No frontend changes.
  - Existing TypeScript build still passes.
- Backend services:
  - Added local settings service helpers.
  - Added integration status service helpers.
  - No new Tauri command surface yet.
- Database:
  - Uses existing `app_settings` and `integration_statuses` tables from P1.05.
  - No migration/schema change.
  - Default integration-status rows are seeded from canonical backend registry, idempotently.
- Security/privacy:
  - No raw secrets/tokens/passwords/API keys are intentionally stored in SQLite.
  - Obvious secret-like config and credential refs are rejected before writes.
  - Credential material itself remains out of scope for this task and would belong in Keychain/native secure storage later.

## Reviewer focus areas

- Confirm the service does not permit obvious raw secrets in `app_settings.value_json`, `integration_statuses.config_json`, or `integration_statuses.credential_ref`.
- Confirm default integration seeding is truthful and idempotent.
- Confirm explicit safe integration updates are not overwritten by reseeding.
- Confirm no accidental P1.16 command surface/frontend work was introduced.
- Confirm no external probes/permissions/OAuth/EventKit/Git/CLI checks were introduced.
- Confirm invalid JSON and secret rejection are typed errors.

## Fix cycle notes

- Initial implementation passed spec review and quality/security review.
- No required fixes are currently open.
