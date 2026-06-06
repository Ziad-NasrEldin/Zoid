# Feature Handoff: P1.05 Database core schema

## Original request

Phase 1 / Database: Add core schema tables needed for foundation: workspaces, settings, events, entity_links, logs/files references, action confirmations/policies where needed.

Continuation context: user asked to continue where prior phase-by-phase Zoid implementation left off, carefully, with verify:release after each stop.

## Implementation summary

- Added P1.05 core SQLite schema migration.
- Added core tables for app settings, integration statuses, entity links, log references, file references, action policies, and confirmation decisions.
- Preserved existing workspaces/events/event_targets foundation tables from earlier migrations.
- Registered migration version 3 and updated migration-version tests.
- Added migration tests for fresh DB creation, v2-to-v3 idempotent upgrade, non-secret schema constraints, table columns, and indexes.
- Fixed review-required gaps:
  - SQLite foreign keys are now enabled on foundation database connections and migration/test paths.
  - `action_policies` is seeded from the existing centralized action-policy matrix so `confirmation_decisions` can reference known categories.
  - Added regression tests for FK enforcement and valid `send_email` confirmation decision insert.

## Changed files

- `src-tauri/migrations/0003_core_schema_p105.sql`: new P1.05 schema migration.
- `src-tauri/src/lib.rs`: migration registration, FK enablement, action policy seeding, schema/version tests, FK/confirmation tests.

## How to test

From `/Users/ziadnasreldin/Zoid`:

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run verify:local`
- `npm run verify:release` before stopping/reporting per current user request.

Expected behavior:

- Migration version reaches 3.
- Fresh and upgraded DBs contain P1.05 core schema tables.
- Re-running migrations is idempotent.
- FK enforcement rejects orphan `event_targets` and orphan `confirmation_decisions.action_category`.
- Seeded `send_email` policy supports a valid `confirmation_decisions` row.
- No raw secret-bearing columns are added to settings/integration tables.

## Tests run

Implementation subagent TDD evidence:

- Red run after tests were added: `cargo test p105 -- --nocapture` failed before migration implementation because migration version was still 2.
- Red/fix loop for review gaps: new policy-seeding test failed before seeding with `QueryReturnedNoRows` for `send_email`.
- Green focused runs:
  - `cargo test p105 -- --nocapture`: PASS, 2 passed after migration.
  - `cargo test migrations_ --lib`: PASS, 5 passed after FK/seed fix.
  - `cargo test open_foundation_database_enables_foreign_keys --lib`: PASS.
- Full Rust tests from implementer: `cargo test`: PASS, 24 passed.

Parent verification:

- `npm run verify:local`: PASS.
  - Rust tests: 24 passed, 0 failed.
  - Frontend build: PASS.

Independent reviews:

- Spec review after initial P1.05 implementation: PASS.
- Quality review after initial P1.05 implementation: REQUEST_CHANGES for FK enforcement and policy seeding.
- Spec re-review after fixes: PASS.
- Quality re-review after fixes: APPROVED.

## Git info

- Branch: main
- Commits:
  - `9473ae0 Add P1.05 core database schema migration`
  - `d8ac612 Fix SQLite FK enforcement and policy seeding`
- Related prior release-gate fix: `4793f72 Fix deterministic release DMG verification`

## Frontend/backend/database notes

- Frontend routes/components: no frontend changes for this slice.
- Backend/native: SQLite connection handling now enables FK enforcement.
- Database tables/migrations:
  - `schema_migrations`, `workspaces`, `events`, `event_targets` from prior migrations.
  - New P1.05 tables: `app_settings`, `integration_statuses`, `entity_links`, `log_references`, `file_references`, `action_policies`, `confirmation_decisions`.
- Security/privacy:
  - Integration/status settings use non-secret config/credential references, not raw tokens/passwords/API keys.
  - Tests assert no forbidden secret-like settings/integration columns were introduced.

## Reviewer focus areas

- Migration idempotence and upgrade safety.
- SQLite FK enforcement and runtime connection correctness.
- Action policy seed consistency with `evaluate_action_policy`.
- No raw secret storage introduced by settings/integration tables.
- Whether scope correctly remains schema/foundation-only without premature repository/UI work.

## Fix cycle notes

- First quality review found two Required fixes:
  1. SQLite FK declarations were not enforced because `PRAGMA foreign_keys = ON` was missing.
  2. `action_policies` was empty while `confirmation_decisions.action_category` had an FK.
- Fix commit `d8ac612` addressed both.
- Re-review verdicts after the fix:
  - Spec: PASS.
  - Quality: APPROVED.
