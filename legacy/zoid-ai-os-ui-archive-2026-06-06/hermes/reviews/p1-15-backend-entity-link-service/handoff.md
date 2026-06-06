# Feature Handoff: P1.15 Backend entity link service

## Original request

P1.15 Backend: entity link service for tasks, notes, products, files, repos, runs, emails, events, browser captures.

Standing process: use the Zoid-wide orchestrator + subagent system, strict TDD for implementation, independent spec/quality review, final critique gate, central verification, then tracker/commit.

## Implementation summary

- Added a backend-only entity link service layer over the existing `entity_links` repository primitives and schema.
- Added service request/filter types:
  - `EntityLinkCreateRequest`
  - `EntityLinkListFilter`
- Added service APIs:
  - `create_entity_link`
  - `get_entity_link`
  - `list_entity_links_by_source`
  - `list_entity_links_by_target`
- Added explicit allowed domain entity types for P1.15:
  - `task`
  - `note`
  - `product`
  - `file`
  - `repo`
  - `run`
  - `email`
  - `event`
  - `browser_capture`
- Service-level validation rejects unsupported/empty entity types and empty ids/relation/actor fields before persistence.
- `metadata_json` is validated before persistence and redacted before storage using the existing redaction convention.
- Existing idempotent insert-or-get behavior is preserved:
  - duplicate logical tuple returns the existing record;
  - same id with a different tuple remains a constraint error.
- Added deterministic directional listing:
  - source -> targets, optionally filtered by relation and target/counterpart type;
  - target -> sources, optionally filtered by relation and source/counterpart type.
- Kept scope backend-only. No new Tauri commands and no frontend changes.
- No migration was needed because `entity_links` already exists in `0003_core_schema_p105.sql` with the required fields and indexes.

## Changed files

- `src-tauri/src/lib.rs`
  - Added entity link service request/filter types, validation helpers, service create/read/list functions, and service-level tests.

## How to test

From `/Users/ziadnasreldin/Zoid`:

```bash
cargo test --manifest-path src-tauri/Cargo.toml entity_link --lib
cargo test --manifest-path src-tauri/Cargo.toml
cargo clippy --manifest-path src-tauri/Cargo.toml --lib --tests -- -D warnings
npm run verify:local
npm run verify:release
```

Expected:

- Entity-link focused tests pass.
- Full Rust tests pass.
- Clippy passes with warnings denied.
- Local/release verification pass before final commit.

## Tests run

Implementer/subagent TDD evidence:

- RED: `cargo test --manifest-path src-tauri/Cargo.toml entity_link_service --lib`
  - FAIL as expected before implementation with missing `EntityLinkCreateRequest`, `EntityLinkListFilter`, `create_entity_link`, `get_entity_link`, `list_entity_links_by_source`, and `list_entity_links_by_target`.
- `cargo fmt --manifest-path src-tauri/Cargo.toml`: PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml entity_link --lib`: PASS, 8 passed.
- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS, 68 lib tests passed, main/doc tests passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --lib --tests -- -D warnings`: PASS.
- `git diff --check`: PASS.

Orchestrator verification so far:

- `cargo test --manifest-path src-tauri/Cargo.toml entity_link --lib`: PASS, 8 passed.
- `cargo clippy --manifest-path src-tauri/Cargo.toml --lib --tests -- -D warnings`: PASS.

Independent reviews before final critique:

- Spec compliance review: PASS.
- Quality/security review: APPROVED.
  - Minor non-blocking notes: lower-level repository primitives remain module-callable and intentionally preserve prior behavior, service validates metadata twice via service + repository path, valid JSON metadata may be any JSON value, and whitespace-only validation does not trim persisted values.

## Git info

- Branch: `main`
- Base before P1.15 work: `7bc289f` Implement P1.14 event writer framework
- Commit SHA: not committed yet at handoff creation time.

## Frontend/backend/database notes

- Frontend: no changes.
- Tauri command surface: no changes; P1.16 owns bridge commands.
- Backend:
  - New service layer for entity link creation, retrieval, and directional listing.
  - Existing lower-level repository primitives remain to preserve prior behavior and tests.
- Database:
  - No migration. Existing `entity_links` table supports P1.15.
  - Existing indexes support source/target/relation queries.

## Reviewer focus areas

- Confirm allowed entity type set matches P1.15 and does not need `workspace` at the service level.
- Confirm metadata validation/redaction happens before persistence.
- Confirm duplicate logical tuple/id collision behavior remains correct.
- Confirm source/target listing order and filters are deterministic enough for upcoming bridge/UI work.
- Confirm no Tauri/frontend scope creep.
- Confirm preserving unchecked lower-level repository primitives is acceptable for compatibility.

## Fix cycle notes

None yet. Independent spec and quality reviews both passed/approved before final critique.
