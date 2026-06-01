# P1.08 Final Critique Report: Backend Settings Service

Verdict: APPROVED

## Scope reviewed

Reviewed commit `6d4b8e5 Implement backend settings service` against the P1.08 requirements:

- Local backend settings service for non-secret app/workspace preferences.
- Integration status service backed by existing SQLite tables.
- No raw secrets/tokens/passwords/API keys stored outside Keychain through the new service helpers.
- Truthful/idempotent integration status seeding from the canonical workspace registry.
- No schema surprise, frontend/Tauri command surface expansion, or external probe scope creep.

## Files/source inspected

- `src-tauri/src/lib.rs`
  - `SettingScope`, `LocalPreferenceInput`, `LocalPreferenceRecord`
  - local preference helpers and secret-validation path
  - `IntegrationStatus`, `IntegrationStatusInput`, `IntegrationStatusRecord`
  - integration status helpers and credential reference validation
  - `seed_default_integration_statuses`
  - `ensure_foundation` seeding call
  - Tauri command registration surface
- `src-tauri/migrations/0003_core_schema_p105.sql`
  - existing `app_settings` and `integration_statuses` table definitions
- P1.08 tests embedded in `src-tauri/src/lib.rs`
- Git state/stat for commit and changed files

## Tests/checks performed

From `/Users/ziadnasreldin/Zoid`:

1. `cargo test settings_service --manifest-path src-tauri/Cargo.toml --lib`
   - PASS: 2 passed, 0 failed.

2. `cargo test integration_status_service --manifest-path src-tauri/Cargo.toml --lib`
   - PASS: 3 passed, 0 failed.

3. `cargo test --manifest-path src-tauri/Cargo.toml --lib`
   - PASS: 38 passed, 0 failed.

4. `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets --all-features -- -D warnings`
   - PASS.

5. Static review checks
   - `git show --stat` confirms the relevant commit changes only `src-tauri/src/lib.rs`.
   - Migration SQL inspected; no P1.08 schema migration/change was introduced.
   - Tauri command search shows only existing `get_foundation_status` command and existing `invoke_handler(tauri::generate_handler![get_foundation_status])`.
   - Source search found no OAuth/EventKit/git/CLI/API probe implementation added for this task.

## Findings

### Secret handling

APPROVED.

The new local preference service path validates before DB writes:

- Secret-like preference keys are rejected via `is_secret_key`.
- Secret-like JSON keys/values are rejected via `validate_no_secret_json` / `json_contains_secret_like_material`.
- Invalid JSON is rejected as `RepositoryError::InvalidJson` before SQLite write.
- Tests confirm rejected secret-like local preference inputs leave `app_settings` empty.

The new integration status service path validates before DB writes:

- Secret-like `config_json` keys/values are rejected.
- Raw-looking credential refs such as `sk-...` are rejected.
- Safe Keychain-style reference strings are accepted.
- Tests confirm rejected integration inputs leave `integration_statuses` empty.

Advisory, not blocking: the lower-level generic `upsert_app_setting` / `update_app_setting` repository helpers remain generic JSON/constraint helpers and do not themselves perform secret rejection. The P1.08 service helpers do enforce the requirement, and no external command surface exposes the lower-level helpers. Future code should route user/app preference writes through the P1.08 service wrappers rather than calling generic app-setting helpers directly for secret-sensitive settings.

### Schema usage

APPROVED.

The implementation uses existing tables:

- `app_settings`
- `integration_statuses`

No new migration file or schema change is introduced by commit `6d4b8e5`. Existing P1.05 migration schema remains the backing store.

### Integration status seeding

APPROVED.

`seed_default_integration_statuses` is registry-backed and uses `insert or ignore`, making it idempotent and non-overwriting. It maps registry states truthfully:

- `not_configured` / `needs_permission` -> DB `not_configured`
- `planned` / `blocked` -> DB `disabled`

Seeded rows set `credential_ref` and `last_checked_at` to null and do not claim `connected` status. Tests verify:

- idempotent seeding count matches registry integration count,
- seeded statuses contain no connected claims,
- seeded credential refs are absent,
- explicit safe updates survive reseeding.

### Scope creep

APPROVED.

No frontend files changed. No new Tauri commands were added for settings/integration status access. Static search found no OAuth, EventKit permission prompts/checks, git CLI probing, external API checks, or process/network probe implementation introduced for this phase.

## Required fixes

None.

## Final verdict

APPROVED. P1.08 satisfies the backend settings service requirements and passes focused/final critique checks.
