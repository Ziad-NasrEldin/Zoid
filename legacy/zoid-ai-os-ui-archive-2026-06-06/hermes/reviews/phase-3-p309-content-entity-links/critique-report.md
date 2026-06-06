# Critique Report: P3.09 Backend Content Entity Links

## Verdict

APPROVED

## Summary

The current uncommitted diff implements a backend-only content-link wrapper over the existing `entity_links` repository for links from `note`/`file` sources to `task`/`product`/`run` targets. The implementation is scoped to `src-tauri/src/lib.rs` and `src-tauri/src/tests.rs`, keeps directionality constrained at create time, uses deterministic logical IDs for idempotence, delegates generic metadata JSON redaction to the approved entity-link service, and adds focused P3.09 tests. I found no Required fixes.

## What was changed

- Added `ContentEntityLinkCreateRequest` and `ContentEntityLinkListFilter` backend types.
- Added `create_content_entity_link_service` with create-time validation for source types, target types, source/target existence/linkability, metadata JSON validity, deterministic ID generation, and delegation to `create_entity_link`.
- Added source/target list wrappers that preserve generic filtering/sorting while constraining top-level source entities to `note`/`file` and target entities to `task`/`product`/`run`.
- Added note/file source validation: active/draft notes only at create time; indexed file references only at create time; source-side listing only requires the row to still exist.
- Added target validation: non-deleted tasks, existing agent runs, and product IDs accepted as opaque while no `products` table exists, with future-aware row existence validation if that table appears.
- Added four P3.09 Rust tests covering note links, file links, invalid direction/missing entities/secret metadata, source lifecycle query survival, idempotence, and directional queries.

## Required fixes

| ID | Severity | Area | Issue | Evidence | Required fix |
|----|----------|------|-------|----------|--------------|
| — | — | — | No Required fixes found. | Reviewed current uncommitted diff and reran focused + local verification. | — |

## Improvements

| ID | Priority | Area | Suggestion | Why it matters |
|----|----------|------|------------|----------------|
| I1 | Low | Backend validation | Consider making content list wrapper `counterpart_type` validation content-aware: source queries should only accept `task`/`product`/`run` counterpart filters, and target queries should only accept `note`/`file` counterpart filters. | Current behavior safely returns empty rows for nonsensical but generic-valid filters such as `note -> email`; stricter validation would catch caller misuse earlier. |
| I2 | Low | Tests | Strengthen the P3.09 metadata test to use an actual secret value and assert safe metadata is preserved while the secret value is absent from the returned/persisted JSON. | Existing generic tests cover redaction deeply, but the P3.09-specific test currently proves delegation only lightly. |
| I3 | Low | Tests/query lifecycle | If deleted-task or completed-run detail views are expected to remain link-queryable, add target-side lifecycle regressions mirroring the source-side trashed note/file regression. | The source-side stale-link case is now covered; target lifecycle expectations are not explicitly encoded. |

## Tests performed

- `git status --short && git diff --stat && git diff --name-only`: confirmed the reviewed uncommitted source diff is limited to `src-tauri/src/lib.rs` and `src-tauri/src/tests.rs`; review artifacts are untracked under `.hermes/reviews/phase-3-p309-content-entity-links/`.
- `git diff -- src-tauri/src/lib.rs src-tauri/src/tests.rs`: inspected the current uncommitted implementation and tests only.
- `cargo test --manifest-path src-tauri/Cargo.toml p309 -- --nocapture`: PASS, 4 passed, 0 failed.
- `npm run verify:local && git diff --check`: PASS. Rust suite 161 passed, 0 failed, 1 ignored; frontend tests passed; frontend build passed; `git diff --check` passed.

## Tests still needed

- None required for this backend-only P3.09 slice.
- Optional future tests: stricter counterpart-filter validation and target-side lifecycle behavior once product/task/run UI semantics are finalized.

## Dev-agent instructions

1. No Required fixes remain.
2. Optional: consider I1/I2/I3 in a follow-up cleanup if desired.
3. Keep this as backend-only; no Tauri bridge/frontend scope was reviewed or required for P3.09.
