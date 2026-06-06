# Critique Report: P1.05 Database core schema

## Verdict

APPROVED

## Summary

The P1.05 implementation satisfies the requested foundation database slice. The migration adds the expected core schema tables for settings, integration statuses, entity links, log/file references, action policies, and confirmation decisions while preserving the existing workspaces/events/event_targets foundation. Runtime migration registration reaches version 3, SQLite foreign-key enforcement is enabled on foundation database connections and migration paths, and action policies are seeded from the centralized policy matrix so confirmation decisions can reference known categories.

I found no blocking or required-change issues. The implementation stays appropriately schema/foundation-focused and does not introduce premature UI or repository/helper work. The local verification gate passes.

## Scope reviewed

- Handoff: `.hermes/reviews/p1-05-database-core-schema/handoff.md`
- Migration files:
  - `src-tauri/migrations/0001_foundation.sql`
  - `src-tauri/migrations/0002_event_schema_backfill.sql`
  - `src-tauri/migrations/0003_core_schema_p105.sql`
- Runtime/database/test code: `src-tauri/src/lib.rs`
- Relevant schema requirements in `Docs/2026-05-31-zoid-implementation-plan-v1.md`
- Git change range covering commits `9473ae0` and `d8ac612`

I did not edit application code. This critique report is the only file written.

## Findings

Positive findings:

- Migration version 3 is registered as `core_schema_p105` and includes `src-tauri/migrations/0003_core_schema_p105.sql`.
- Fresh migrations create the P1.05 tables:
  - `app_settings`
  - `integration_statuses`
  - `entity_links`
  - `log_references`
  - `file_references`
  - `action_policies`
  - `confirmation_decisions`
- Existing foundation tables for `schema_migrations`, `workspaces`, `events`, and `event_targets` remain in place.
- The schema uses JSON validity checks on JSON-bearing columns and check constraints for enumerated policy/status/decision fields.
- Indexes exist for the expected access paths around entity links, log/file references, and confirmation decisions.
- SQLite foreign keys are enabled in `open_foundation_database(...)` and again in `run_migrations(...)`; tests verify `PRAGMA foreign_keys = 1` and FK rejection for invalid references.
- `confirmation_decisions.action_category` references `action_policies(category)`, and the policy table is seeded from the centralized `evaluate_action_policy(...)` matrix after migrations.
- The `send_email` policy seed is specifically covered and supports a valid confirmation-decision insert.
- Settings/integration schema does not add raw secret/token/password/API-key columns; integration credentials are represented as a `credential_ref`.
- Upgrade/idempotence coverage exists for a v2-shaped database and repeated migration runs.

## Required fixes

| ID | Severity | Area | Issue | Evidence | Required fix |
|----|----------|------|-------|----------|--------------|
| — | — | — | No required fixes. | Source inspection and focused/full verification passed. | — |

## Improvements / non-blocking notes

| ID | Priority | Area | Suggestion | Why it matters |
|----|----------|------|------------|----------------|
| I1 | Low | Schema hardening | Consider adding an explicit `foreign key (workspace_key) references workspaces(id)` on `file_references.workspace_key` if future behavior intends file references to be tied only to known workspaces. | Current nullable, unconstrained `workspace_key` is acceptable for a foundation slice and future flexibility, but a FK would tighten integrity once workspace semantics are finalized. |
| I2 | Low | Test depth | Consider adding constraint-level tests for representative invalid JSON/check-constraint values in P1.05 tables. | Existing tests verify table shape, key FK paths, policy seeding, and non-secret columns; additional negative constraint tests would further lock down schema intent. |

## Checks performed

- Read handoff and changed source/migration files.
- Reviewed relevant implementation-plan schema requirements for core tables, entity links, events/event_targets, and migration rules.
- Inspected git state and commits:
  - `git status --short`
  - `git log --oneline -5`
  - `git show --stat --oneline --decorate 9473ae0 d8ac612`
  - `git diff --stat 4793f72..HEAD`
  - `git diff --name-only 4793f72..HEAD`
  - `git diff --check 4793f72..HEAD`
- Ran focused Rust checks:
  - `cargo test --manifest-path src-tauri/Cargo.toml p105 -- --nocapture`: PASS, 2 passed.
  - `cargo test --manifest-path src-tauri/Cargo.toml migrations_ --lib`: PASS, 5 passed.
  - `cargo test --manifest-path src-tauri/Cargo.toml open_foundation_database_enables_foreign_keys --lib`: PASS, 1 passed.
- Ran full Rust test suite:
  - `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 24 passed.
- Ran local verification gate:
  - `npm run verify:local`: PASS; Rust tests passed, frontend build passed, local push verification passed with `--skip-package`.

## Tests still needed

None blocking for P1.05.

Before a release stop, `npm run verify:release` remains useful if the parent workflow requires the packaged-app/DMG release gate for every phase. For this final critique pass, `npm run verify:local` and focused database tests were sufficient and passed.

## Dev-agent instructions

1. No required fixes remain for P1.05.
2. Optional follow-ups I1-I2 may be handled in a later schema hardening phase if desired.
3. Keep future repository/helper primitives for P1.06 rather than expanding P1.05 beyond the schema foundation scope.
