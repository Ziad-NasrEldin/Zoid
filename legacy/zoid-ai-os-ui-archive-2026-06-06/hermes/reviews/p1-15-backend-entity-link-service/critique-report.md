# Critique Report: P1.15 Backend entity link service

## Verdict

APPROVED

## Summary
The P1.15 backend entity link service is compliant with the requested scope. The current source adds a backend-only service layer over the existing `entity_links` repository primitives, preserves prior lower-level repository behavior, and validates allowed entity types, required fields, JSON metadata, redaction, idempotent duplicate handling, id-collision behavior, and deterministic directional listing. No Tauri command or frontend scope creep was found. Focused tests and clippy both pass locally.

## What was changed
- Added backend service request/filter structs for entity link creation and directional listing in `src-tauri/src/lib.rs`.
- Added the P1.15 service API functions: `create_entity_link`, `get_entity_link`, `list_entity_links_by_source`, and `list_entity_links_by_target`.
- Added service validation for allowed entity types and required id/relation/actor fields before persistence.
- Added metadata JSON validation and redaction before insert-or-get persistence.
- Added deterministic directional source/target list queries with relation and counterpart-type filters.
- Added focused unit tests for service create/read/list, allowed type coverage, required-field validation, idempotence, id collision, metadata validation/redaction, and deterministic directional filtering.
- Left existing repository primitives available and unchanged in behavior, including existing `workspace` repository-level tests.

## Required fixes
None.

## Improvements
| ID | Priority | Area | Suggestion | Why it matters |
|----|----------|------|------------|----------------|
| I-001 | Low | Tests | Add explicit service test cases for empty service-level link `id` and empty `created_by_actor_type`, and for invalid list filter relation/counterpart values. | The implementation already validates these paths, but explicit coverage would make the required pre-persistence validation guarantees easier to audit in future refactors. |
| I-002 | Low | API design | Consider documenting that the service accepts any JSON value for `metadata_json`, not only JSON objects. | `validate_json_field` accepts all valid JSON values; documenting this avoids ambiguity for future bridge/UI callers. |

## Tests performed
- `cargo test --manifest-path src-tauri/Cargo.toml entity_link --lib`: PASS — 8 tests passed, 0 failed, 60 filtered out.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --lib --tests -- -D warnings`: PASS — completed with no warnings.

## Tests still needed
None.

## Dev-agent instructions
- No required fixes are needed for P1.15.
- If making follow-up hardening changes, keep them backend-only unless a later feature explicitly opens Tauri/frontend scope.
- Preserve the distinction between unchecked lower-level repository primitives and the validated P1.15 service API.
