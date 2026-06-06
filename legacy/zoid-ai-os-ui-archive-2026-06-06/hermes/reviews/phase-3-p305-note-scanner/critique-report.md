# P3.05 Note Scanner/Indexer Critique Report

Verdict: APPROVED

## Summary

Re-review after the required fixes found the previous blockers resolved. The scanner now rewrites Markdown files that already have YAML frontmatter but are missing required Zoid identity fields, and it performs scanner frontmatter persistence before DB/index upsert using a temp-file + rename helper. Duplicate-ID paths are checked before rewrite, preserving the non-destructive duplicate behavior.

Verification performed:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p305_ -- --nocapture
```

Result: PASS, 4 tests passed, 0 failed.

## Required fix verification

### 1. Existing YAML frontmatter without `zoid_id` is rewritten

Satisfied.

`scan_markdown_notes_service` now calls `note_needs_identity_frontmatter_update`, which returns true when the Markdown has no frontmatter or when `zoid_id`, `title`, or `slug` is missing. For such notes, the scanner renders identity frontmatter with `write_note_identity_frontmatter`, which preserves unrelated existing YAML lines while setting Zoid-owned scalars.

Regression coverage is present in `p305_note_scanner_rewrites_existing_yaml_missing_zoid_id_preserving_custom_keys`, which verifies that a file with existing YAML gets a persisted `zoid_id`, retains title/slug/custom frontmatter/body content, and indexes the same stored ID.

### 2. Scanner DB/index upsert no longer happens before direct file writes

Satisfied.

For notes requiring identity persistence, the scanner now writes rendered frontmatter through `write_note_frontmatter_atomically` before calling `upsert_note_identity_metadata`. The helper writes to a new temp file and renames it into place, cleaning up the temp file on rename failure. This removes the previous direct truncating `fs::write` path and avoids leaving active/current DB/index rows when frontmatter persistence fails before upsert.

Duplicate handling is still non-destructive: in-memory duplicate IDs and existing indexed paths are checked before any rewrite, and duplicates are marked/conflicted without rewriting the duplicate file.

Regression coverage is present in `p305_note_scanner_write_failure_does_not_leave_active_index_row`, which verifies a scanner write failure does not leave an active note row or current knowledge-index row.

## Notes

- Scope remains backend-only (`src-tauri/src/lib.rs` plus Rust tests), with no Tauri bridge/frontend overreach.
- Focused P3.05 tests now cover the original scanner/indexer behavior plus both required-fix regressions.
- I did not edit source files during this re-review; only this critique report was updated.
