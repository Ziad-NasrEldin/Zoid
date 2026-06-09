# Critique Report: Zoid file permission persistence

Verdict: REQUEST_CHANGES

## Scope

Reviewed the file-permission persistence changes described in the handoff, limited to the scoped `src-tauri/src/lib.rs` permission-marker implementation and its repository command integrations. I did not edit product source.

## What I inspected

- `.hermes/reviews/zoid-file-permission-persistence/handoff.md`
- `src-tauri/src/lib.rs`
  - `FilePermissionMarker`
  - `load_file_permission_marker`
  - `persist_file_permission_marker`
  - `touch_file_permission_path`
  - `remember_file_permission_path`
  - `warm_file_permissions_inner`
  - `read_repository_details`
  - `scan_repository_folder`
  - `clone_repository`
  - default-branch read/update/list flows using `read_repository_details`

## Findings

### Blocking issue 1: remembered paths are still touched on every repository request

Status: not resolved.

The handoff states that existing touched paths are skipped and only new paths are touched. `warm_file_permissions_inner` does this for its warmup target list, but `remember_file_permission_path` does not.

Current behavior in `src-tauri/src/lib.rs`:

```rust
fn remember_file_permission_path(path: &Path) -> Result<(), String> {
    let marker_path = file_permission_bootstrap_path()?;
    let marker = load_file_permission_marker(&marker_path);
    let mut touched_paths = marker.touched_paths.into_iter().collect::<HashSet<_>>();
    let touched_path = touch_file_permission_path(path)?;
    if touched_paths.insert(touched_path) {
        persist_file_permission_marker(&marker_path, &touched_paths)?;
    }
    Ok(())
}
```

Because `touch_file_permission_path(path)?` runs before checking whether the path is already in `touched_paths`, repeated calls to `read_repository_details`, `scan_repository_folder`, and `clone_repository` still perform the same filesystem access every time. For the user-reported default-branch Edit flow, `list_remote_branches` calls `read_repository_details`, and `update_default_branch` calls it as well. That means opening the Edit menu on an already-recorded repository still hits `touch_file_permission_path` before the code discovers the path was already remembered.

This contradicts the feature goal of stopping repeated permission confirmation and the handoff claim that "existing touched paths are skipped on later warmups" / "only touch new paths". The marker currently prevents repeated marker writes, but not repeated permission-touch access in the repository request path.

Required fix:

- Change `remember_file_permission_path` so it avoids `touch_file_permission_path` for already-recorded paths.
- Add a regression test that calls `remember_file_permission_path` twice for the same path and proves the second call does not perform the touch/access path again. One practical way is to factor the key resolution/check separately or inject a touch helper in tests; another is to make the second call operate on a path whose marker entry exists but whose access would fail if touched, and assert the second call succeeds only because it skips touching.
- Ensure the default-branch Edit/list/update path benefits from that skip, since those commands call `read_repository_details`.

### Blocking issue 2: clone destination roots that do not exist can no longer be created

Status: regression risk introduced by permission remembering order.

`clone_repository` now calls `remember_file_permission_path(&root)?` before `fs::create_dir_all(&root)`:

```rust
let root = PathBuf::from(destination_root.trim());
remember_file_permission_path(&root)?;
fs::create_dir_all(&root).map_err(|error| format!("Failed to create destination root: {error}"))?;
```

`remember_file_permission_path` calls `touch_file_permission_path`, which calls `metadata` for non-directories. If the destination root does not already exist, the command returns `Zoid could not access ...` before it reaches the existing `create_dir_all` behavior. That changes clone semantics from "create the destination root if needed" to "destination root must already exist".

Required fix:

- Preserve the existing create-if-missing clone behavior by moving permission remembering after `create_dir_all`, or by remembering/touching the nearest existing parent before creation and then recording the created root after creation.
- Add a focused regression test for cloning/setup path handling that verifies a missing destination root is created or, if the product intentionally requires pre-existing roots, update the UI/API contract and error message accordingly. Based on the existing code, this looks unintentional.

### Non-blocking note: warmup marker persistence itself is directionally correct

`warm_file_permissions_inner` correctly loads the persisted marker, builds a set of touched paths, skips known warmup targets when `force == false`, touches only new existing warmup targets, and persists a sorted marker. This addresses the original one-time-marker problem for the warmup list itself.

### Non-blocking note: missing/inaccessible paths remain truthful

Calling `remember_file_permission_path` before existence checks changes some error wording to `Zoid could not access ...`, but it still truthfully reports missing/inaccessible paths. I do not consider that a blocker, except for the clone create-before-touch regression above.

## Verification run

Command run from `/Users/ziadnasreldin/Zoid`:

```bash
npm run test:rust -- warm_file_permissions --nocapture
```

Result: PASS.

Observed output summary:

- `warm_file_permissions_persists_marker_after_first_run ... ok`
- `warm_file_permissions_records_new_paths_after_marker_exists ... ok`
- 2 passed, 0 failed, 30 filtered out
- Rust emitted one unrelated `unused_mut` warning at `src/lib.rs:1079`.

I did not rerun the full frontend/build/Tauri suite because the handoff already reports those runs and this review found source-level blockers in the scoped permission logic.

## Final verdict

REQUEST_CHANGES

The marker persistence implementation is a good start, and warmup-list persistence works in the focused tests. However, the repository request integration does not yet satisfy the core requirement: remembered repository/folder paths are still touched on every call because `remember_file_permission_path` checks the marker only after touching the filesystem. Additionally, clone destination handling now fails before creating a missing destination root. Both issues should be fixed before approval.
