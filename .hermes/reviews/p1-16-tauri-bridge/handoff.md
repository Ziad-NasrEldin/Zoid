# P1.16 Tauri Bridge Handoff

Feature: P1.16 Tauri bridge commands for foundation status, workspace registry, settings, event read/write, and policy preview.

Repo: /Users/ziadnasreldin/Zoid
Branch: main

## Original tracker requirement

`P1.16 Tauri bridge: commands for foundation status, workspace registry, settings, event read/write, policy preview.`

## Scope boundaries

- Backend/Tauri bridge only.
- No React/frontend changes.
- No new schema/migrations.
- No fake external integrations, OAuth checks, provider checks, or consequential action execution.
- Existing redaction, settings, event writer, workspace registry, and action policy internals are reused.

## Changed files

- `src-tauri/src/lib.rs`

## Command surface

Existing command preserved:
- `get_foundation_status`

New Tauri commands added and registered in `tauri::generate_handler![...]`:
- `get_workspace_registry`
- `read_local_preference`
- `list_local_preferences`
- `upsert_local_preference`
- `read_integration_status_command`
- `list_integration_statuses_command`
- `upsert_integration_status_command`
- `create_event`
- `read_event`
- `list_events`
- `preview_action_policy`

## Behavior summary

Workspace registry:
- Returns the existing seeded/canonical workspace registry records.
- Test verifies all 14 workspaces and canonical order.

Settings bridge:
- Local preference read/list/upsert through existing settings service.
- Integration status read/list/upsert through existing integration status service.
- Rejects invalid JSON and obvious secret-like keys/material/credential refs before persistence.
- Does not store secrets outside Keychain; credential refs remain references only.

Event bridge:
- Event create/read/list uses existing P1.14 event repository.
- Event create validates metadata JSON before persistence.
- Event create redacts summary/metadata before storage.
- Event create supports targets/entity links through existing event target inputs.
- Event list stays deterministic and bounded by existing normalized event-list behavior.
- Public event create bridge now enforces input bounds before persistence:
  - targets: 25
  - summary: 4096 bytes
  - metadata_json: 16384 bytes
  - small fields: 256 bytes
  - source/actor_id/target entity_id: 512 bytes
- Over-limit errors are normalized and do not echo submitted values.

Policy preview:
- `preview_action_policy` accepts either explicit category or request dimensions.
- Uses existing evaluator/classifier.
- Does not open DB, write events/logs, or execute anything.

Command registration regression:
- Tests keep a command-name list and also parse `include_str!("lib.rs")` to verify every P1.16 command is present in the actual `generate_handler` block.

## TDD / review history

Implementer RED evidence:
- `cargo test --manifest-path src-tauri/Cargo.toml tauri_bridge --lib`
- Failed initially because command surface and wrapper request/functions did not exist.

Review fix RED evidence:
- `cargo test --manifest-path src-tauri/Cargo.toml tauri_bridge --lib`
- Failed initially because `parse_generate_handler_command_names` and over-limit bridge validation were not implemented.

Independent spec review:
- PASS.

Independent quality/security review:
- Initial verdict: REQUEST_CHANGES.
- Required fixes:
  1. Bound public event write API.
  2. Strengthen marker-only command registration test.
- Fix lane completed both.
- Re-review verdict: APPROVED.

## Parent verification already run

- `cargo test --manifest-path src-tauri/Cargo.toml tauri_bridge --lib`
  - 7 passed, 0 failed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --lib --tests -- -D warnings`
  - passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`
  - 75 passed, 0 failed.

## Reviewer focus areas

Please independently verify:
- All required P1.16 commands are present and registered.
- Bridge commands do not claim fake integrations or execute protected actions.
- Settings/event write paths do not persist secrets and reject invalid JSON.
- Event create bounds are sufficient and enforced before persistence.
- Policy preview is read-only.
- Tests meaningfully cover the bridge and registration behavior.

## Expected critique output

Write report to:

`.hermes/reviews/p1-16-tauri-bridge/critique-report.md`

Use verdict `APPROVED` only if no Required fixes remain.
