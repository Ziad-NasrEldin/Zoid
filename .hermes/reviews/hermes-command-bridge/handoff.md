# Feature Handoff: Hermes command bridge in Zoid chat

## Original request

"i want you to link hermes agent commands with zoid 25 commands, i want the exact usage of commands used in the terminal to be also accessible from zoid 25 chat session"

## Implementation summary

- Zoid 25 chat now routes normal messages through the local Hermes CLI and prefixes responses with the exact terminal command used.
- Chat prompts that start with terminal-style Hermes commands, for example `hermes tools list` or `hermes cron list --all`, execute that Hermes CLI subcommand directly instead of being wrapped in `hermes chat`.
- Regular chat/slash prompts still run through `hermes chat --continue --quiet --source desktop --query ...`, and the rendered reply shows the terminal-equivalent invocation.
- Linked repository workdir is preserved in the displayed usage as `cd <repo> && hermes ...` and applied as the child-process current directory.
- High-risk or interactive command surfaces are blocked in the chat bridge: bare interactive groups such as `hermes tools`, `hermes setup`, `hermes model`, `hermes skills`, plus `uninstall` and `--yolo`.
- Composer slash-command help now tells the user that terminal-style Hermes commands can be typed from chat and that Zoid prints the exact terminal command used.

## Changed files

- `src-tauri/src/lib.rs`: added shell quoting, terminal usage rendering, terminal-style Hermes CLI command parsing, interactive/high-risk guards, command display in responses, and backend tests.
- `src/agents/ChatComposer.tsx`: documented the terminal-style command bridge in the Zoid slash-command panel helper copy.
- `src/scaffold.test.ts`: added source-level guardrails for the Hermes command bridge and composer command-surface copy.

## How to test

- In Zoid 25 Agents chat, send a normal prompt. Expected: Hermes responds and the assistant bubble starts with `Terminal command used:` followed by `hermes chat --continue --quiet --source desktop --query ...`.
- In Zoid 25 Agents chat, send `hermes tools list`. Expected: Zoid executes the Hermes CLI `tools list` subcommand and the assistant bubble starts with `Terminal command used:` followed by `hermes tools list`.
- With a linked repository selected, send a prompt. Expected: the assistant bubble shows `cd <linked repository> && hermes ...`, and the backend child process runs from that directory.
- Try `hermes setup` or `hermes uninstall`. Expected: Zoid rejects the command with a clear safety/interactive warning rather than launching an unsafe interactive flow from chat.

## Tests run

- `npm test -- --runInBand 2>/dev/null || npm test`: PASS. Frontend scaffold test passed; Rust test suite passed with 15 tests.
- `cargo test hermes_prompt_can_execute_terminal_style_cli_command terminal_usage_is_attached_to_hermes_responses terminal_style_cli_message_runs_requested_hermes_subcommand hermes_cli_message_runs_inside_linked_repository --manifest-path src-tauri/Cargo.toml`: FAIL due invalid cargo syntax for multiple test-name arguments; superseded by full `npm test` Rust suite passing all tests.
- `npm run build`: PASS. TypeScript and Vite production build passed; Vite emitted only the existing chunk-size warning.
- `npm run tauri:build`: PASS. Tauri release build passed and produced `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Reinstalled/relaunched `/Applications/Zoid 25.app`: PASS. Running process verified as `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- `git diff --check -- src-tauri/src/lib.rs src/agents/ChatComposer.tsx src/scaffold.test.ts`: PASS.

## Git info

- Branch: current working tree branch; not committed by this handoff.
- Commit SHA: not committed.
- Diff base: local working tree includes many unrelated pre-existing Zoid changes. Review should focus on the three files listed above for this command bridge.

## Frontend/backend/database notes

- Frontend routes/components: Agents/Hermes chat composer only.
- Backend commands: existing `send_hermes_cli_message` Tauri command now supports direct Hermes CLI subcommands and terminal usage display.
- Database: none.

## Reviewer focus areas

- Verify terminal-style detection is strict enough to only treat a prompt as a Hermes CLI command when it begins with `hermes` as a standalone token.
- Verify shell quoting in the displayed terminal usage is safe/readable for paths and prompts with spaces.
- Verify interactive/high-risk command guards are sufficient for a chat UI bridge.
- Verify normal chat prompts and slash prompts still route through `hermes chat`.
- Verify linked repository workdir still flows through frontend send path into backend `current_dir`.

## Known limitations / risks

- This is a synchronous Tauri command; long Hermes commands can keep the assistant bubble in writing state until the CLI returns or hits the existing timeout.
- The current repo is broadly dirty from multiple Zoid features; this handoff is intentionally scoped to the Hermes command bridge files.

## Fix cycle notes

- Fixed R1 from initial critique by blocking `--yolo` anywhere in parsed Hermes CLI args, not only as the first argument.
- Added Rust regression coverage for `hermes --yolo`, `hermes chat --yolo --query hi`, and `hermes --profile default chat --yolo --query hi`.
- Fixed R2 from re-review by updating the expanded `HermesProfileSettings` test initializer to use `..HermesProfileSettings::default()` so Rust tests compile after the broader settings struct expansion.
- Re-ran `cargo test hermes_prompt_can_execute_terminal_style_cli_command --manifest-path src-tauri/Cargo.toml`: PASS.
- Re-ran `npm test -- --runInBand 2>/dev/null || npm test`: PASS, 15 Rust tests passed.
- Re-ran `npm run build`: PASS, Vite chunk-size warning only.
- Re-ran `git diff --check -- src-tauri/src/lib.rs src/agents/ChatComposer.tsx src/scaffold.test.ts`: PASS.
