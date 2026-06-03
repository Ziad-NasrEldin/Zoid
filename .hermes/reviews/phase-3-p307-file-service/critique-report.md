# P3.07 File Browse/Open/Preview Service Re-Review

Verdict: APPROVED

## Summary

The required fixes are present and satisfy the P3.07 backend file browse/open/preview review criteria. Preview IO is now bounded before UTF-8 conversion, returned preview construction, and knowledge-index digesting. Path resolution now walks every relative path component with `symlink_metadata` and rejects any symlink component. DB/index writes remain after successful validation/read/preview conversion for open/preview, and the implementation remains backend-only and within P3.07 scope.

## Verification Performed

- Reviewed updated handoff: `.hermes/reviews/phase-3-p307-file-service/handoff.md`.
- Reviewed changed source in `src-tauri/src/lib.rs` and P3.07 tests in `src-tauri/src/tests.rs`.
- Ran focused P3.07 tests:

```text
cargo test --manifest-path src-tauri/Cargo.toml p307_ -- --nocapture
```

Result:

```text
running 4 tests
test tests::p307_file_service_rejects_unsafe_binary_and_missing_paths_without_indexing ... ok
test tests::p307_file_service_rejects_symlinked_files_and_directories ... ok
test tests::p307_file_preview_reads_and_indexes_only_bounded_preview_bytes ... ok
test tests::p307_file_service_browses_opens_and_previews_safe_visible_files ... ok

test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 149 filtered out
```

- Ran whitespace/diff check:

```text
git diff --check
```

Result: PASS.

- Checked changed-file scope:

```text
git diff --stat && git diff --name-only
```

Result: only `src-tauri/src/lib.rs` and `src-tauri/src/tests.rs` are changed.

## Findings

### 1. Bounded preview IO/indexing — fixed

`preview_file_service` now calls `read_bounded_preview_bytes`, which opens the target file and reads through `file.take(FILE_PREVIEW_READ_LIMIT)`, where `FILE_PREVIEW_READ_LIMIT` is `FILE_PREVIEW_BYTE_LIMIT + 1`. It then truncates stored preview bytes to `FILE_PREVIEW_BYTE_LIMIT` before UTF-8 conversion, return payload construction, and indexing.

`upsert_file_preview_index` receives the bounded preview bytes and computes `content_digest` from those bytes, not from a full-file buffer. The regression test `p307_file_preview_reads_and_indexes_only_bounded_preview_bytes` verifies the returned preview and indexed `search_text` are 4096 bytes and exclude tail content.

### 2. Symlink component rejection — fixed

`resolve_file_service_existing_path` now accumulates each relative path segment from the visible root and calls `fs::symlink_metadata` at each step. If any component is a symlink, it returns a constraint error before canonical containment checks or file open/preview behavior.

The regression test `p307_file_service_rejects_symlinked_files_and_directories` now covers direct file symlinks, direct directory symlinks, and an intermediate symlink directory component.

### 3. Successful-only DB/index writes — acceptable

Open and preview writes occur after root/path validation, symlink rejection, metadata/type checks, previewability checks, bounded read, NUL rejection, and UTF-8 conversion. Failed missing, unsafe, binary/non-previewable, symlinked, or invalid preview paths do not write file preview index rows. The existing negative test verifies rejected unsafe/binary/missing preview paths leave zero `knowledge_index_entries` rows for `entity_type = 'file'`.

Browse writes metadata for safely enumerated non-symlink child entries only after the browse target itself is validated as a safe directory; symlink children are skipped.

### 4. Scope alignment — acceptable

The implementation remains backend-only service/helper/test work. I did not find any added Tauri command bridge, frontend changes, OS launch behavior, destructive file actions, OCR/embeddings, or recursive whole-home indexing. Changed files are limited to backend Rust source and tests.

## Approval Notes

No blocking issues remain from the previous review. The required bounded-preview and symlink-hardening fixes are implemented and covered by focused tests.
