# Feature Handoff: Phase 3 P3.07 File Browse/Open/Preview Service

## Original request

Continue autonomous Zoid Phase 3 tracker work with autopilot always on.

Tracker item:

- P3.07 Backend: basic file browse/open/preview service.

## Scope delivered

Backend-only safe file reference helpers:

- `browse_files_service` lists immediate safe children under a visible root relative path;
- `open_file_reference_service` returns a safe file reference record and absolute path string without launching the OS app;
- `preview_file_service` returns bounded UTF-8 preview text for previewable text/code/Markdown files;
- successful browse/open updates `file_references` metadata rows;
- successful preview updates both `file_references` and `knowledge_index_entries` with `entity_type='file'`, `source_type='file_preview'`;
- unsafe paths, missing files, binary/non-previewable files, and symlinks fail closed.

No destructive file actions, no OS launch, no Tauri command bridge, no frontend, no OCR/embeddings, and no whole-home recursive indexing were added.

## Changed files

- `src-tauri/src/lib.rs`
  - added `FileBrowseEntry`, `FileOpenRecord`, `FilePreviewRecord`.
  - added safe file path/root validation and symlink/canonical containment checks.
  - added file kind/mime/previewability helpers.
  - added `browse_files_service`, `open_file_reference_service`, `preview_file_service`.
  - added file reference and file preview index upsert helpers.
- `src-tauri/src/tests.rs`
  - added P3.07 tests for browse/open/preview happy path, unsafe/missing/binary rejection, and symlink file/dir rejection.

## Behavior details

Safe browsing:

- Accepts repo-visible relative paths only; rejects absolute paths, `..`, empty path segments, and hidden paths.
- Rejects directly browsed symlink directories; skips symlink entries when listing a normal directory.
- Lists only immediate children and sorts by relative path.
- Classifies folders as `folder`, Markdown as `markdown_note`, common image/audio/video/archive/code/document extensions, otherwise `other`.

Open:

- Verifies the target exists, is not a symlink, is under the visible root, and is a file.
- Returns metadata plus an absolute path string for future bridge/UI use.
- Does not launch Finder or any external application.

Preview:

- Verifies the target is a safe existing file.
- Allows only previewable text/code/Markdown extensions.
- Rejects NUL bytes and invalid UTF-8.
- Returns at most 4096 bytes of text and marks truncation.
- Writes a `file_preview` knowledge index row only after successful preview.

## Fix cycle notes

Initial critique returned `REQUEST_CHANGES` for:

1. preview IO/indexing not truly bounded because `fs::read` loaded full files and the index digest used the full buffer;
2. intermediate symlink components being allowed when canonicalized inside the visible root.

Fixes applied:

- Preview now reads only `FILE_PREVIEW_BYTE_LIMIT + 1` bytes through a bounded reader, truncates to 4096 bytes, and indexes/digests only bounded preview bytes.
- Path resolution now walks each relative path component with `symlink_metadata` and rejects any symlink component, not only final symlink targets.
- Added regression test for a large previewable file proving returned/indexed preview text is bounded and excludes tail content.
- Added regression test for an intermediate symlink directory component.

## Tests run

RED:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p307_ -- --nocapture
```

Result:

- Failed as expected because `browse_files_service`, `open_file_reference_service`, and `preview_file_service` did not exist.
- Required-fix RED later failed on intermediate symlink traversal before fixes.

Focused GREEN:

```bash
cargo test --manifest-path src-tauri/Cargo.toml p307_ -- --nocapture
```

Result:

- PASS, 4 passed.

Full verification:

```bash
npm run verify:local && git diff --check
```

Result:

- PASS.
- Rust: 152 passed, 0 failed, 1 ignored guarded P2.32 real-DB harness.
- Frontend tests: PASS.
- Frontend build: PASS.
- Final marker: `PASS: local push verification passed (--skip-package)`.

## Review focus requested

Please verify:

- path validation is not accidentally note-only and correctly supports directories/files;
- symlink and path escape protections are sufficient;
- preview is bounded and rejects binary/non-previewable files before indexing;
- file reference/index writes only happen on successful safe operations;
- scope remains backend-only and aligned with P3.07.
