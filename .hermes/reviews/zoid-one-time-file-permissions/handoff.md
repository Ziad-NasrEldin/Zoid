# Feature Handoff: Zoid one-time file permission reuse

## Original request

Page Feedback for `/` Code repository Edit button: "why does everytime i open the edit menu it shows me a confimation message to allow zoid 25 to access my files and folders it should allow permenantely apply the same funcitonality on all requests that asks for permissions"

## Implementation summary

- Scoped fix in `src-tauri/src/lib.rs` for GitHub default-branch edit/list actions.
- When a repository already has persisted remote metadata, `list_remote_branches` and `update_default_branch` no longer re-touch the local repository folder before using GitHub/`gh` APIs.
- Added `remember_file_permission_path_without_touch` so remote-metadata paths are recorded in a separate persistent `remembered_paths` marker list without filesystem reads that can re-open macOS folder permission prompts.
- `touched_paths` remains reserved for paths actually accessed successfully; direct filesystem actions still go through `remember_file_permission_path` and must prove access before they are treated as touched.
- Added `repository_id_from_raw_path` and used it for remote-metadata update fallback results, so successful default-branch updates do not canonicalize/read the local path when remote metadata is already provided.
- Existing direct filesystem actions still retain the one-time warm/marker behavior after real access.
- Added regression coverage that remote-metadata branch actions do not emit local access errors for an unreadable/missing path, isolate `HERMES_HOME`, store the path under `remembered_paths` only, do not let a remote-only remembered path satisfy direct filesystem access, and do not canonicalize/read local paths after a successful remote update.

## Changed files

- `src-tauri/src/lib.rs`: permission marker helper, separate remote-known vs touched marker state, non-touch raw repository id helper, remote-metadata branch action guards, Rust regression tests.

## How to test

- Open Zoid 25 > Code > repository card > Edit default branch for a repo with `remoteUrl`; the branch list/update should use GitHub remote metadata and should not request folder access repeatedly.
- Scan/clone paths should still persist their selected folders once through the existing marker after real filesystem access.

## Tests run

- `cargo test --manifest-path src-tauri/Cargo.toml --lib branch_remote_metadata_paths_do_not_retouch_local_folder_permissions -- --test-threads=1`: PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib --bins -- --test-threads=1`: PASS, 42 Rust tests passed.
- `npm run build && npm run tauri:build`: PASS, TypeScript/Vite build passed and packaged `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Installed app relaunch: copied bundle to `/Applications/Zoid 25.app`, launched `/Applications/Zoid 25.app/Contents/MacOS/zoid`, process observed as PID 54267.
- Screenshot: `/tmp/zoid25-permission-fix-final.png` captured the relaunched installed app.
- Earlier `npm run test:frontend && npm run test:rust`: BLOCKED before Rust by pre-existing frontend scaffold guard: `Settings must expose archived agent sessions: JSON.stringify(remainingArchivedSessions)`.

## Prior review fixes addressed

- Fixed marker pollution: remote-known paths are now stored in `remembered_paths`, not `touched_paths`.
- Fixed test isolation: the new regression test uses `env_lock()` and a temporary `HERMES_HOME`.
- Added regression coverage that remote-only remembered paths do not satisfy direct filesystem access.
- Fixed successful-update path: remote-metadata update fallback uses raw path id generation instead of `repository_id(...canonicalize...)`.
- Added regression coverage for fake `gh repo edit` success with missing local path.

## Git info

- Branch: current working tree, dirty before this scoped fix.
- Diff base: existing local working tree contains many unrelated modified/untracked Zoid files; review should focus only the scoped `src-tauri/src/lib.rs` permission changes and tests.

## Reviewer focus areas

- Confirm remote-metadata branch actions avoid local filesystem permission prompts while preserving persistent marker behavior.
- Confirm direct filesystem actions still access real selected folders and do not falsely claim access.
- Confirm tests cover both non-touch remote path behavior and persisted-marker behavior.

## Fix cycle notes

Third review request after addressing the second critique about fallback `repository_id` canonicalization.
