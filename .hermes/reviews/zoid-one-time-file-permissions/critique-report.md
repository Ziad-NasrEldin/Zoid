# Critique Report: Zoid one-time file permissions / persistent Hermes chat session

Status: APPROVED

## Scope reviewed

- `src-tauri/src/lib.rs` target changes for Hermes chat invocation, session-id parsing, and one-time permission warming.
- Rust test-script support in `package.json` and `src-tauri/Cargo.toml`.
- Existing frontend wiring only as needed to confirm the backend session parameter is passed/persisted.

## Findings

No target-scope blockers found.

### Persistent Hermes chat session

- First normal prompt path now builds `hermes chat --continue --quiet --source desktop --query <prompt>` when no persisted Hermes CLI session id is available.
- Subsequent prompt path now uses `hermes chat --resume <session-id> --quiet --source desktop --query <prompt>` when `hermes_session` is non-empty.
- `send_hermes_cli_message` passes frontend-provided `hermesSession` into `hermes_chat_args`, so persisted frontend session state can affect backend invocation.
- Frontend wiring still passes `activeSession.hermesCliSessionId` into `sendHermesCliMessage(...)` and stores `response.session` back into `hermesCliSessionId`.

### Session id parsing / output cleanup

- `parse_hermes_session_id` handles both `session_id:` and `Session ID:` / `session id:` style labels case-insensitively via lowercase prefix checks while slicing the original line.
- `strip_terminal_noise` filters the documented `Session ID:` and `session_id:` markers before content is returned, so the parsed session id is not lost before persistence because parsing is performed against `combined_output`, not stripped content.

### One-time file permission warming

- `warm_file_permissions_inner(false)` exits early when the persisted `zoid-file-permissions.json` marker exists.
- First-run warming still touches the intended home/profile/project targets and writes the marker with touched paths.
- Regression coverage verifies the second non-forced warm returns an empty result after marker creation.

### Rust test-script support

- `package.json` scopes `test:rust` to `cargo test --manifest-path src-tauri/Cargo.toml --lib --bins -- --test-threads=1`.
- `src-tauri/Cargo.toml` disables doctests for the mixed Tauri lib target with `doctest = false`.
- This is acceptable for the target test support scope and avoids unrelated rustdoc failures for the app crate.

## Verification run during re-review

- `npm run test:rust` — PASS
  - 20 Rust tests passed; 0 failed.
  - Includes targeted regressions for `--continue`, `--resume`, session parsing, and one-time permission marker behavior.

## Issues encountered

- `npm run build` currently fails before bundling due to TypeScript errors in untracked `src/ui/GlobalDropdown.behavior.test.tsx` involving `happy-dom` DOM type incompatibilities. This file is outside the target feature scope and is untracked in the working tree, so I am not treating it as a blocker for this review.

## Decision

APPROVED — all target-feature required fixes appear resolved, and the target Rust regression suite passes.
