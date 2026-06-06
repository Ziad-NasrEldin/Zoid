# Feature review handoff: Hermes CLI terminal bridge correction

## User correction
User corrected the prior implementation: Hermes is not intended to be used as an API here. They are using Hermes itself from the terminal, with Codex CLI/provider configured inside Hermes. The Zoid app should link to the Hermes terminal/CLI, not local HTTP API Server endpoints.

## Scope changed
- Removed active Hermes API Server integration assumptions from frontend/backend.
- Backend now spawns the local `hermes` CLI via Rust `std::process::Command`.
- Frontend invokes Tauri commands:
  - `check_hermes_cli`
  - `send_hermes_cli_message`
- CLI lookup:
  - `ZOID_HERMES_CLI` explicit path if set
  - `hermes` on PATH
  - `$HOME/.local/bin/hermes`
  - `$HOME/.cargo/bin/hermes`
  - `/opt/homebrew/bin/hermes`
  - `/usr/local/bin/hermes`
- Message command uses:
  - `hermes chat --continue --quiet --source desktop --query <prompt>`
- This resumes the most recent Hermes CLI session instead of using the API server.
- UI copy now says Hermes CLI / terminal session, not API server.
- Source contract test rejects API Server strings/paths in the active screen/backend.
- Added timeout guard with `wait-timeout` for spawned Hermes commands.

## Changed files in this correction
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`
- `src-tauri/src/lib.rs`
- `src/agents/types.ts`
- `src/agents/hermesClient.ts`
- `src/agents/AgentsHermesScreen.tsx`
- `src/scaffold.test.ts`

## Verification run
- `npm test` passed.
- `npm run tauri:build` passed and bundled `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Real Hermes CLI smoke command passed:
  - `hermes chat --continue --quiet --source desktop --query "Reply with exactly: ZOID_TERMINAL_LINK_OK"`
  - output included `ZOID_TERMINAL_LINK_OK`.
- Packaged app launched and process was observed, then killed.

## Known notes
- The Hermes CLI smoke output included existing Hermes warning `Warning: Unknown toolsets: hermes`; backend strips warning/session metadata from stdout before displaying the response.
- This is still a request/response CLI bridge, not a full embedded PTY terminal. It links to Hermes CLI and sends prompts through it. If the product must show a literal terminal emulator with live streaming output and interactive approval prompts, that is a bigger PTY/WebSocket task.

## Reviewer focus
1. Confirm no active API Server dependency remains for the Hermes chat path.
2. Confirm command spawning is safe enough for local-only v1 and does not hardcode secrets.
3. Confirm `--continue` behavior is acceptable for linking current/latest Hermes terminal session.
4. Identify Required fixes only if blockers remain.
