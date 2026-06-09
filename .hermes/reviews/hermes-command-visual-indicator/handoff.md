# Hermes command visual indicator handoff

## Request
User said there was no visual indication when they wrote a Hermes command and asked for another agent to review prior work line by line with no assumptions.

## Scope
Files changed for this iteration:
- `src/agents/ChatComposer.tsx`
- `src/agents/MessageBubble.tsx`
- `src/App.css`
- `src/scaffold.test.ts`
- `src-tauri/src/lib.rs`

## Implementation
- Composer now detects drafts that begin with standalone `hermes` followed by whitespace or end-of-string.
- When detected, composer visibly switches into CLI mode:
  - label chip: `Hermes CLI command`
  - yellow/highlighted textarea state via `composer-input-wrap--hermes-command`
  - mode strip: `CLI mode armed — Zoid will run this as a terminal Hermes command and show the exact command used.`
  - send button changes to `RUN CLI` when enabled.
- Sent user message bubbles now show a `Hermes CLI command` chip when the user message starts with a standalone Hermes command.
- Placeholder now hints: `Message Hermes or type hermes tools list...`.

## Independent line-by-line review agent
A separate delegate agent reviewed the current Hermes command bridge code line by line and reported concrete findings. Required fixes applied from that review:
- Removed hidden backend reinjection of `--yolo` from `hermes_invocation_args`; chat no longer converts approval_mode `off` into a high-risk terminal invocation after safety checks.
- Extended high-risk guard to block `--yolo=<value>` forms, not only exact `--yolo`.
- Fixed command-line splitting so empty quoted arguments like `''` and `""` are preserved instead of silently dropped.
- Added regression tests for the above.

## Verification performed
- `npm test -- --runInBand 2>/dev/null || npm test`: PASS. Frontend scaffold passed and Rust suite passed 20/20.
- `npm run build`: PASS. Vite chunk-size warning only.
- `npm run tauri:build`: PASS. Built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Installed fresh app to `/Applications/Zoid 25.app` and launched it. Process verified: `/Applications/Zoid 25.app/Contents/MacOS/zoid` PID 15263.
- Browser/dev visual smoke at `http://127.0.0.1:1420/`: typed `hermes tools list`; DOM and screenshot confirmed:
  - `.composer-input-wrap--hermes-command` present
  - chip text `Hermes CLI command` present
  - strip text `CLI mode armed...` present
  - highlighted textarea background present
- `git diff --check -- src-tauri/src/lib.rs src/agents/ChatComposer.tsx src/agents/MessageBubble.tsx src/App.css src/scaffold.test.ts`: PASS.
- Focused cargo tests:
  - `cargo test hermes_prompt_can_execute_terminal_style_cli_command --manifest-path src-tauri/Cargo.toml`: PASS.
  - `cargo test terminal_style_cli_message_runs_requested_hermes_subcommand --manifest-path src-tauri/Cargo.toml`: PASS.

## Known limitations
- Native installed app launch/process was verified, but browser/dev visual smoke was used for the typed composer visual proof because browser tooling can inspect DOM and screenshots directly.
- In the smoke environment, Hermes CLI status displayed ERROR/LOCKED because the browser dev environment cannot reach the Tauri native invoke layer. This does not affect the visual indicator verification.

## Review instructions
Review line by line. No assumptions. Confirm the visual-state detection matches backend command detection as closely as practical, safety fixes do not regress terminal-equivalent behavior, and tests actually cover the behavior claimed.
