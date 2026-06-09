# Critique Report: Hermes command visual indicator

## Verdict

APPROVED

## Summary

The requested visual indication for terminal-style Hermes commands is implemented and verified. Composer and sent-user-message detection are consistent with the backend parser for the standalone lowercase `hermes` prefix followed by whitespace or end-of-string. The backend safety fixes from the prior line-by-line review are present: no hidden `--yolo` reinjection remains, `--yolo=<value>` is blocked, and empty quoted arguments are preserved. Current tests and build pass.

## What was changed

- `src/agents/ChatComposer.tsx`: adds `isHermesCliCommandDraft`, CLI-mode chip/strip/highlight class, placeholder copy, and send-button CLI styling/text branch.
- `src/agents/MessageBubble.tsx`: adds matching user-message CLI detection and a `Hermes CLI command` chip on sent user bubbles.
- `src/App.css`: adds command chip, command textarea highlight, command send styling, mode strip, and message command chip styles.
- `src/scaffold.test.ts`: extends scaffold assertions for the visual indicator CSS/strings and backend Hermes command bridge safety markers.
- `src-tauri/src/lib.rs`: parses terminal-style `hermes ...` prompts, blocks high-risk CLI arguments, preserves empty quoted args, avoids hidden `--yolo` injection, and includes focused regression tests.

## Required fixes

None.

## Improvements

| ID | Priority | Area | Suggestion | Why it matters |
|----|----------|------|------------|----------------|
| I1 | Medium | Test | Replace some `src/scaffold.test.ts` string-presence checks for the composer with component/DOM tests around command detection and visible state. | Current frontend tests prove strings/classes exist, but not that React state transitions render them for user typing. Browser smoke covers it manually, not as repeatable CI coverage. |
| I2 | Low | Maintainability | Consider sharing or centralizing the Hermes-command detection rule between composer and message bubble, or add paired test vectors for both. | The frontend currently duplicates the same rule in two files; future drift could make draft and sent-message indicators disagree. |
| I3 | Low | UX | Decide explicitly how attachments should behave when the draft is a terminal-style `hermes ...` command. | `ChatComposer` appends attachment context to the message before send; backend will parse all appended text as CLI arguments when the message starts with `hermes`. This is existing behavior and not blocking for the requested indicator, but it should be made intentional. |

## Tests performed

- Read handoff: `/Users/ziadnasreldin/Zoid/.hermes/reviews/hermes-command-visual-indicator/handoff.md`.
- Line-by-line inspected exactly these files:
  - `/Users/ziadnasreldin/Zoid/src/agents/ChatComposer.tsx`
  - `/Users/ziadnasreldin/Zoid/src/agents/MessageBubble.tsx`
  - `/Users/ziadnasreldin/Zoid/src/App.css`
  - `/Users/ziadnasreldin/Zoid/src/scaffold.test.ts`
  - `/Users/ziadnasreldin/Zoid/src-tauri/src/lib.rs`
- `git diff --check -- src-tauri/src/lib.rs src/agents/ChatComposer.tsx src/agents/MessageBubble.tsx src/App.css src/scaffold.test.ts`: PASS.
- `npm test -- --runInBand`: PASS. Frontend scaffold passed; Rust lib/bin tests passed 20/20. Note: npm emitted `Unknown cli config "--runInBand"` warning but the command completed successfully.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib`: PASS, 20/20.
- `cargo test --manifest-path src-tauri/Cargo.toml hermes_prompt_can_execute_terminal_style_cli_command`: PASS.
- `cargo test --manifest-path src-tauri/Cargo.toml terminal_style_cli_message_runs_requested_hermes_subcommand`: PASS.
- `npm run build`: PASS. Vite emitted only the existing large chunk warning.
- Browser/dev visual smoke at `http://127.0.0.1:1420/`: typed `hermes tools list` in the Hermes composer and confirmed via DOM inspection:
  - wrapper class includes `composer-input-wrap--hermes-command`
  - chip text is `Hermes CLI command`
  - strip text includes `CLI mode armed` and terminal-command copy
  - textarea background uses the yellow command gradient
  - send button was `LOCKED` because the dev browser could not reach the Tauri native Hermes CLI bridge; this matches the disabled/offline state. Code path would show `RUN CLI` only when enabled.

## Tests still needed

- Optional CI-grade frontend component test for typing `hermes tools list` and asserting the chip/strip/highlight render.
- Optional native installed-app smoke if the release gate requires proving the same UI inside the packaged Tauri app rather than the dev browser.

## Dev-agent instructions

No required fixes. If continuing polish, prioritize I1 so future regressions in the visual state are caught by automated tests rather than only scaffold string checks or manual browser smoke.
