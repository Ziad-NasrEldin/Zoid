# Critique Report: Hermes Command Parity

## Verdict

APPROVED

## Scope Reviewed

- Handoff: `.hermes/reviews/hermes-command-parity/handoff.md`
- Backend bridge: `src-tauri/src/lib.rs`
- Frontend command bridge/UI: `src/agents/hermesClient.ts`, `src/agents/hermesCommands.ts`, `src/agents/slashCommandParser.ts`, `src/agents/recentCommands.ts`, `src/agents/CommandPalette.tsx`, `src/agents/ChatComposer.tsx`, `src/agents/AgentsHermesScreen.tsx`
- Styling/tests: `src/App.css`, `src/scaffold.test.ts`
- Validation gate: `npm run test:rust && npm run test:frontend && npm run build`

## Findings

The previously blocking validation failures have been fixed in the current repository state. The implementation satisfies the requested Hermes command parity architecture: Zoid imports Hermes' live Python `COMMAND_REGISTRY`, exposes Tauri commands to list and execute slash commands, feeds the same registry into both the slash composer and command palette, treats unknown/non-wrapped slash commands as Hermes chat/session executions, provides Zoid-native panel behavior for high-value command groups, and gates dangerous/global commands behind a Zoid confirmation flow.

| Requirement | Result | Notes |
|---|---:|---|
| Do not duplicate a static Hermes command list in Zoid | PASS | `src-tauri/src/lib.rs` imports `from hermes_cli.commands import COMMAND_REGISTRY`; no static Zoid-side Hermes command table was found in the reviewed implementation. |
| Load live Hermes registry | PASS | `load_hermes_slash_commands_inner` detects a Hermes source root, runs Python against the source registry, serializes registry entries, annotates Zoid behavior, and sorts commands. |
| Slash composer and command palette share the same registry | PASS | `AgentsHermesScreen.tsx` loads `listHermesSlashCommands()` into `slashCommands`, then passes the same array to `ChatComposer` and `CommandPalette`. |
| Unknown/non-wrapped slash commands forward to Hermes chat/session execution | PASS | `parseSlashCommand` recognizes any slash-prefixed token. Backend fallback runs `hermes chat --query <slash command>` via `hermes_chat_args(...)` when a command is not handled natively/no-op/confirmation. |
| High-value commands may return Zoid-native panels | PASS | `classify_zoid_command_behavior` maps model/tools/skills/cron/agents/profile/history/usage/debug/browser groups to `native-panel`; frontend opens `activeCommandPanel` with Zoid-native modal copy. |
| Dangerous/global commands require Zoid confirmation | PASS | Backend returns `kind: "confirmation"`/`requires_confirmation` for sensitive commands and destructive forms. Frontend stores the exact pending command context and the confirm button executes that same command with `confirmed = true`. |
| `/clear` and `/quit` semantics | PASS | `/new`, `/reset`, and `/clear` return `new-session`; `/quit`, `/exit`, and `/q` return `close-session`, with frontend session creation/archive behavior wired. |
| TUI-only visual commands no-op | PASS | `/redraw`, `/skin`, `/statusbar`, and `/indicator` classify as `noop` and return `Not needed in Zoid.` |
| `/restart` status/restart/status with confirmation | PASS | `/restart` is confirmation-gated and, once confirmed, runs gateway status, restart, and status, returning combined output scoped to `global-hermes`. |
| Tests/build | PASS | Full combined gate passed in this re-review. |

## Verification Run

Command run from `/Users/ziadnasreldin/Zoid`:

```sh
npm run test:rust && npm run test:frontend && npm run build
```

Result: PASS.

- Rust: 41 passed, 0 failed.
- Frontend: `tsx src/scaffold.test.ts && tsx src/ui/GlobalDropdown.behavior.test.tsx` passed.
- Build: `tsc && vite build` passed.
- Only warning observed: Vite chunk-size warning for the generated frontend bundle.
- Rust emitted existing dead-code warnings for `apply_profile_runtime_args` and `prompt_with_enabled_profile_context`; these do not block approval.

## Notes

- I did not edit source files during this re-review.
- I updated this critique report only.
- Remaining product depth items noted in the handoff, such as richer model/history/settings panels, are acceptable follow-up work rather than blockers for the stated command parity requirements.
