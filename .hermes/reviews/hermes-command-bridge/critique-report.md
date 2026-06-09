# Critique Report: Hermes command bridge in Zoid chat

## Verdict

APPROVED

## Summary

The prior R1 and R2 blockers are resolved. The Zoid chat bridge now supports terminal-style Hermes CLI commands that start with standalone `hermes`, continues to route normal prompts through `hermes chat`, displays the exact terminal command used, applies linked repository workdirs, and blocks the reviewed high-risk/interactive surfaces. The scoped backend and frontend changes are covered by targeted Rust/source tests, and the relevant test/build checks pass.

## What was changed

- `src-tauri/src/lib.rs`: adds command usage rendering, shell quoting, terminal-style Hermes command parsing, direct Hermes CLI execution, linked-repository `current_dir` handling, high-risk/interactive command guards, and tests for command parsing, response prefixing, direct subcommand execution, and linked repository execution.
- `src-tauri/src/lib.rs`: R1 fix blocks `--yolo` anywhere in parsed Hermes CLI args and includes regression assertions for `hermes --yolo`, `hermes chat --yolo --query hi`, and `hermes --profile default chat --yolo --query hi`.
- `src-tauri/src/lib.rs`: R2 fix updates the expanded `HermesProfileSettings` test initializer with `..HermesProfileSettings::default()`, restoring Rust test compilation.
- `src/agents/ChatComposer.tsx`: slash-command helper text documents terminal-style Hermes commands and says Zoid prints the exact terminal command used.
- `src/scaffold.test.ts`: source-level guardrails verify the bridge terms, composer copy, frontend send path, linked repository plumbing, and Hermes CLI spawning expectations.

## Required fixes

| ID | Severity | Area | Issue | Evidence | Required fix |
|----|----------|------|-------|----------|--------------|
| None | - | - | No blocking issues found in the final re-review scope. | Targeted Rust test, full `npm test`, scoped whitespace check, and production build all pass. | None. |

## Improvements

| ID | Priority | Area | Suggestion | Why it matters |
|----|----------|------|------------|----------------|
| I1 | Medium | Test | Add explicit negative tests for non-standalone or non-leading Hermes text such as `hermes-agent tools list`, `hermesify`, and prose containing `hermes tools list` later in the message. | The source logic checks standalone leading `hermes`, but direct regression tests would lock the contract called out in the handoff. |
| I2 | Low | Security/UX | Document which Hermes subcommands are intentionally allowed to perform side effects from chat beyond the current `uninstall`, `--yolo`, and bare interactive group blocks. | The bridge exposes terminal-equivalent command execution; a clear policy will reduce ambiguity for future side-effectful command surfaces like config or cron operations. |

## Tests performed

- Read `/Users/ziadnasreldin/Zoid/.hermes/reviews/hermes-command-bridge/handoff.md`.
- Read the prior `/Users/ziadnasreldin/Zoid/.hermes/reviews/hermes-command-bridge/critique-report.md` and verified the previous R1/R2 blockers against current source.
- Inspected scoped files:
  - `/Users/ziadnasreldin/Zoid/src-tauri/src/lib.rs`
  - `/Users/ziadnasreldin/Zoid/src/agents/ChatComposer.tsx`
  - `/Users/ziadnasreldin/Zoid/src/scaffold.test.ts`
- `git diff -- src-tauri/src/lib.rs src/agents/ChatComposer.tsx src/scaffold.test.ts && git diff --check -- src-tauri/src/lib.rs src/agents/ChatComposer.tsx src/scaffold.test.ts`: PASS, exit code 0. Diff output was large/truncated, but scoped whitespace check passed.
- `cargo test hermes_prompt_can_execute_terminal_style_cli_command --manifest-path src-tauri/Cargo.toml`: PASS, 1 test passed, 14 filtered out. Confirms R1 regression test compiles and passes.
- `npm test -- --runInBand 2>/dev/null || npm test`: PASS. Frontend scaffold test passed; Rust suite passed 15/15 tests, including `hermes_profile_settings_round_trip_uses_active_profile_home`, `hermes_prompt_can_execute_terminal_style_cli_command`, `terminal_style_cli_message_runs_requested_hermes_subcommand`, `terminal_usage_is_attached_to_hermes_responses`, and `hermes_cli_message_runs_inside_linked_repository`.
- `npm run build`: PASS. `tsc` and Vite production build completed; Vite emitted only the existing chunk-size warning for a 672.05 kB JS asset.

## Tests still needed

- Optional manual desktop smoke test in `/Applications/Zoid 25.app`: send a normal prompt, `hermes tools list`, and a linked-repository prompt; confirm the assistant bubble starts with `Terminal command used:` and that linked repository usage renders as `cd <repo> && hermes ...`.
- Optional policy test expansion for non-standalone Hermes prefixes and non-leading prose, as noted in I1.

## Dev-agent instructions

1. No required fixes remain for this scope.
2. Consider adding the optional I1 regression tests in a future hardening pass.
3. If this feature is being shipped as a desktop app, perform the optional manual Zoid UI smoke test before external release.
