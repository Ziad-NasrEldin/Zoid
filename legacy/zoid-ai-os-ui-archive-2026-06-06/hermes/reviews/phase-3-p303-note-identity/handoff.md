# Feature Handoff: Phase 3 P3.03 Note Identity and Frontmatter Metadata

## Original request

Continue autonomous Zoid Phase 3 tracker work after P3.01–P3.02:

- P3.03 Database: note identity/index metadata with stable frontmatter ID and conflict state.
- Use small TDD-oriented slices, verify locally, run lean critique, update tracker/review docs, and commit clean approved slice.

## Implementation summary

- Added backend helpers for stable local Markdown note identity:
  - derive a deterministic `note_<hash>` ID from the safe relative path when frontmatter is missing;
  - preserve existing `zoid_id` frontmatter across title/body edits;
  - derive title from frontmatter, then first Markdown heading, then filename fallback;
  - derive slug from frontmatter or slugified title;
  - compute a deterministic body digest.
- Added frontmatter writer that inserts or updates Zoid identity fields (`zoid_id`, `title`, `slug`) while preserving existing frontmatter lines and note body.
- Made generated frontmatter scalar output YAML-safe for accepted titles/IDs/slugs by double-quoting and escaping values; added reader support for the same escaped double-quoted scalar form.
- Added note identity metadata upsert into SQLite:
  - writes/updates the `notes` row;
  - writes/updates the `knowledge_index_entries` `markdown_frontmatter` row;
  - fails closed when the same stable note ID appears at a different active path and marks the existing row `conflicted` / `duplicate_id` without overwriting the duplicate path.
- Added validation for note IDs, titles, slugs, and safe Markdown relative paths.
- Stabilized an existing P2.29 cancel bridge regression by replacing a timing-based `sleep 10` command with a sentinel-file wait that never releases during the test. This preserves the cancel/kill assertion and avoids timeout race flakiness.

Known limitation: this is backend helper/test coverage for P3.03. It does not implement full P3.04 note CRUD, P3.05 filesystem scanner/indexer, P3.10 Tauri commands, or frontend UI.

## Changed files

- `src-tauri/src/lib.rs`: added note identity/frontmatter helper structs/functions, YAML-safe scalar write/read helpers, and SQLite note/index upsert helper.
- `src-tauri/src/tests.rs`: added P3.03 RED/GREEN tests covering stable identity, YAML-safe round-trip, unsafe path/ID rejection, index metadata upsert, and duplicate-ID conflict state; stabilized the existing P2.29 cancel regression command.

## How to test

From `/Users/ziadnasreldin/Zoid`:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p303 -- --nocapture
```

```bash
cargo test --manifest-path src-tauri/Cargo.toml p229_run_bridge_cancel_kills_active_process_writes_log_and_rejects_terminal_mutation -- --nocapture
```

```bash
npm run verify:local && git diff --check
```

## Tests run

- Focused P3.03 after final fixes: `cargo test --manifest-path src-tauri/Cargo.toml p303 -- --nocapture`: PASS, 3 tests passed.
- P2.29 stabilization check: `cargo test --manifest-path src-tauri/Cargo.toml p229_run_bridge_cancel_kills_active_process_writes_log_and_rejects_terminal_mutation -- --nocapture`: PASS, 1 test passed.
- Full local gate: `npm run verify:local && git diff --check`: PASS.
  - Rust: 137 passed, 0 failed, 1 ignored guarded P2.32 real-DB harness.
  - Frontend tests: PASS.
  - Frontend build: PASS.
  - Final marker: `PASS: local push verification passed (--skip-package)`.

## Review / fix cycle

- Initial critique: REQUIRED FIXES because the frontmatter writer emitted unquoted plain scalars that could be invalid YAML for common accepted titles such as `Meeting: Client`.
- Fix 1: changed frontmatter scalar output to double-quoted escaped values and added test coverage for colon-space and embedded quote output.
- Second critique: REQUIRED FIXES because the parser did not unescape the writer's double-quoted scalar output, so escaped titles did not round-trip.
- Fix 2: added `parse_yaml_scalar_value` / `unescape_yaml_double_quoted_scalar` and a round-trip assertion for `Meeting: Client "A"`.
- Final lean critique verdict: APPROVED.

## Git info

- Branch: `main`
- Latest committed baseline before this slice: `d9caa9f feat: add phase 3 notes files schema`
- Current state before commit: uncommitted P3.03 changes plus this handoff/critique directory.

## Reviewer focus areas

1. Stable identity behavior: generated IDs are deterministic and frontmatter `zoid_id` is preserved across edits.
2. Validation/path safety: note IDs/slugs/relative paths fail closed without overclaiming full filesystem safety.
3. Duplicate conflict behavior: duplicate IDs at different active paths mark existing rows conflicted and reject the duplicate upsert.
4. Knowledge index metadata: frontmatter rows are upserted correctly and scoped to `note` / `markdown_frontmatter`.
5. Scope control: this does not pretend note CRUD/scanner/UI is complete.
