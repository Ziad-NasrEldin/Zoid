# Critique Report: P1.14 Backend generic event writer/reader

## Verdict

APPROVED

## Summary

Reviewed the current P1.14 backend event writer/reader changes against the handoff and requested focus areas. The implementation adds internal generic event create/read/list helpers over the existing SQLite `events` and `event_targets` tables, validates metadata JSON before insert, redacts summaries and metadata before persistence, stores targets, reads/list targets deterministically, and wraps create + target insert + readback in a SQLite savepoint for atomic rollback. Focused event tests and clippy both passed. I found no required fixes.

## What was changed

- Added internal event repository models in `src-tauri/src/lib.rs`: `EventTargetInput`, `EventTargetRecord`, `EventCreateInput`, `EventRecord`, and `EventListFilter`.
- Reworked existing `write_event` to route through `create_event_record`, preserving existing event writes while adding generic event API semantics.
- Added `create_event_record`, `read_event_record`, `list_event_records`, `read_event_targets`, list-limit normalization, and improved `next_event_id` generation.
- Mapped P1.14 API terms onto the existing schema without a migration: `action_type` uses `events.type`, and `outcome` uses `events.severity`.
- Validates `metadata_json` before opening the create savepoint or inserting event/target rows.
- Redacts summary text and metadata JSON before insertion; metadata remains valid JSON after redaction.
- Uses a SQLite savepoint around event insert, target inserts, and final readback; failures roll back partial rows.
- Lists events by `rowid desc` for deterministic newest-insertion-first ordering and reads targets with `primary` relation first, then stable lexical ordering.
- Added focused regression tests for redaction/target ordering/list filtering, invalid metadata pre-insert behavior, savepoint rollback on target failure, and rapid-event ordering.
- Added `Docs/2026-06-01-zoid-subagent-orchestration.md` as a process artifact. No frontend files or new Tauri commands were changed in the current working tree.

## Required fixes

None.

## Improvements

| ID | Priority | Area | Suggestion | Why it matters |
|----|----------|------|------------|----------------|
| I-1 | Low | Backend/API hardening | Consider replacing `insert or ignore` for `event_targets` with explicit duplicate detection or documented idempotent semantics when this internal API becomes externally exposed. | The current behavior is inherited from prior `write_event` behavior and is acceptable for P1.14, but future callers may expect duplicate target input to be rejected rather than silently collapsed. |
| I-2 | Low | DB/schema | Consider adding a future migration with `check (json_valid(metadata_json))` on `events.metadata_json` to match newer core tables. | The application validates and redacts metadata before persistence, so this is not a current blocker, but a schema-level guard would provide defense in depth against future raw SQL writes. |

## Tests performed

- Inspected handoff: `.hermes/reviews/p1-14-backend-event-writer/handoff.md`.
- Inspected implementation in `src-tauri/src/lib.rs`, including event models, `write_event`, `create_event_record`, `read_event_record`, `list_event_records`, `read_event_targets`, metadata redaction/validation helpers, event ID generation, and focused tests.
- Inspected SQLite event schema in `src-tauri/migrations/0001_foundation.sql` and `src-tauri/migrations/0002_event_schema_backfill.sql`.
- Checked Tauri command exposure in `src-tauri/src/lib.rs`: only existing `get_foundation_status` is in `tauri::generate_handler![...]`; no new event commands were added.
- Checked working tree status: app-code changes are limited to `src-tauri/src/lib.rs`; `.hermes/reviews/p1-14-backend-event-writer/` and `Docs/2026-06-01-zoid-subagent-orchestration.md` are untracked review/process artifacts.
- `cargo test --manifest-path src-tauri/Cargo.toml event --lib`: PASS. 7 passed, 0 failed, 56 filtered out.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --lib --tests -- -D warnings`: PASS. Finished without warnings.

## Tests still needed

None for P1.14. Broader full-suite/release verification can still be run by the orchestrator if desired, but the required focused event tests and clippy gate passed.

## Dev-agent instructions

- No required fixes. Proceed with acceptance/merge for P1.14.
- Keep the event repository internal until the separate frontend/Tauri command scope is intentionally implemented in a later slice.
- If a future slice exposes generic event writes to broader callers, define duplicate-target behavior explicitly and consider a schema-level JSON validity constraint for `events.metadata_json`.
