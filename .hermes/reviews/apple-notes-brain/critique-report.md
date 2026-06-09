Verdict: APPROVED

Required fixes

None. The two blockers from the previous review are addressed in the current third fix cycle.

Notes

- Timeout draining fix accepted. `run_command_with_timeout` now places timeout-managed commands into their own Unix process group with `command.process_group(0)`, sends TERM to the negative process group on timeout, waits briefly, then sends KILL to the same process group before waiting and joining stdout/stderr reader threads. This closes the inherited pipes held by descendant processes and lets partial stdout/stderr drain without waiting for long-lived grandchildren.
- The focused timeout regression test now asserts prompt return with a descendant process that keeps inherited stdout/stderr open. My re-run completed in 0.16s and preserved both partial stdout and stderr in the timeout error.
- Apple Notes merge safety fix accepted. `merge_apple_notes_raw_notes` now handles the Apple-unchanged/Zoid-changed case explicitly as `(apple_changed == false, zoid_unchanged == false)`: it preserves the local Zoid title/body/current hash, updates Apple identity/timestamps only, marks the note `changedInZoid`, and does not overwrite local edits with the unchanged Apple copy.
- Added Rust coverage proves the local title/body/current hash survive the Apple-unchanged/Zoid-changed sync, last synced Apple content remains the original Apple version, sync status becomes `changedInZoid`, and extraction is rejected while the note is not `synced`.
- Existing second-cycle improvements remain in place: frontend copy describes `twoWay` as metadata tracking only/no writeback, partial source errors are surfaced, frontend extraction is disabled for non-synced notes, backend extraction rejects stale/conflict/missing/changed statuses, and sync conflicts remain visible in the UI.
- I did not find evidence of automatic Apple Notes hard deletes or writeback in the reviewed Brain path. Apple Notes mutation still appears limited to protected `Zoid Brain` folder setup and ignored/disposable E2E test setup/cleanup.

Proof reviewed

- Read `/Users/ziadnasreldin/Zoid/.hermes/reviews/apple-notes-brain/handoff.md`.
- Read previous `/Users/ziadnasreldin/Zoid/.hermes/reviews/apple-notes-brain/critique-report.md`.
- Reviewed current relevant source/test areas:
  - `src-tauri/src/lib.rs`
  - `src/brain/BrainWorkspace.tsx`
  - `src/brain/BrainWorkspace.behavior.test.tsx`
- Key source observations:
  - `run_command_with_timeout` uses `command.process_group(0)` on Unix, tracks `process_group = child.id()`, kills `-{process_group}` with TERM then KILL on timeout, drains stdout/stderr reader threads, and includes capped partial output in the timeout error.
  - `command_timeout_drains_partial_stdout_and_stderr` now asserts elapsed time is under 1 second and checks partial stdout/stderr are present.
  - `merge_apple_notes_raw_notes` now uses explicit match cases for `(apple_changed, zoid_unchanged)`, including `(false, false)` as `changedInZoid` without replacing local title/body/current_hash.
  - `zoid_changed_note_is_preserved_when_apple_is_unchanged` covers the silent-overwrite regression and extraction blocking.
  - `extract_brain_note_in_store` still rejects every note whose `sync_status` is not `synced`.
- Targeted verification run by this review:
  - `cargo test --manifest-path src-tauri/Cargo.toml --lib command_timeout_drains_partial_stdout_and_stderr -- --test-threads=1`: PASS, 1 passed, finished in 0.16s.
  - `cargo test --manifest-path src-tauri/Cargo.toml --lib zoid_changed_note_is_preserved_when_apple_is_unchanged -- --test-threads=1`: PASS, 1 passed.
  - `cargo test --manifest-path src-tauri/Cargo.toml --lib brain_extraction_rejects_stale_missing_and_conflicted_notes -- --test-threads=1`: PASS, 1 passed.
  - `npx tsc --noEmit --pretty false`: PASS.
  - `git diff --check -- src-tauri/src/lib.rs src/brain/BrainWorkspace.tsx src/brain/BrainWorkspace.behavior.test.tsx src/brain/types.ts src/brain/brainClient.ts package.json`: PASS.
- Handoff-reported broader verification after the third fix cycle, not fully re-run by this independent review:
  - `npm run test:frontend`: PASS.
  - `npm run test:rust`: PASS, 71 passed / 0 failed / 1 ignored.
  - Ignored real Apple Notes E2E: PASS.
  - `npm run build`: PASS.
  - `npm run tauri:build`: PASS.
  - Full `git diff --check`: PASS.
  - App reinstalled/relaunched and packaged AX/window check passed.

Residual risks

- The real Apple Notes E2E remains ignored by default; I relied on the handoff’s reported third-cycle pass and did not rerun the mutation-heavy macOS Notes test in this review.
- Apple Notes folder linking/sync still primarily identifies folders by account name and folder name; duplicate folder names within one account could remain ambiguous depending on Apple Notes behavior.
- The Brain store is a JSON file under the active Hermes profile with backup-on-save, but there is no visible file lock around load-modify-save cycles, so concurrent writers could race.
- Conflict resolution/writeback remains intentionally out of scope for v1. Users can see conflicts and extraction is blocked, but resolution is not implemented in the Brain UI yet.
