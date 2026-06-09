# Hermes Command Parity Review Handoff

## Feature
Implement Hermes slash command parity in Zoid 25 so Zoid can behave like a native Hermes TUI surface while keeping Zoid-native UI.

## Product requirements
- Do not duplicate a static Hermes command list in Zoid.
- Load the live Hermes `COMMAND_REGISTRY` from the installed/source Hermes tree.
- Slash composer and command palette use the same registry.
- Unknown/non-wrapped commands forward to Hermes chat/session execution.
- High-value commands may return Zoid-native panel results.
- Dangerous/global commands require Zoid confirmation.
- `/clear` starts a new Zoid/Hermes session, `/quit` closes active tab/session, TUI-only visual commands are no-op/not-needed.
- `/restart` uses Hermes gateway status/restart/status with confirmation.

## Changed areas
- `src-tauri/src/lib.rs`
  - Added `HermesSlashCommand` and `HermesSlashExecutionResult` structs.
  - Added live registry import from `hermes_cli.commands.COMMAND_REGISTRY` via detected Hermes source root.
  - Added behavior classification for native panels/noops/confirmation-forwarding.
  - Added `list_hermes_slash_commands` and `execute_hermes_slash_command` Tauri commands and invoke registration.
  - Added slash execution handling for `/clear`, `/quit`, TUI no-ops, native panels, `/restart`, confirmations, and fallback Hermes CLI forwarding.
  - Added Rust tests for registry parsing, native Zoid session semantics, and confirmation gates.
- `src/agents/hermesCommands.ts`
  - Added frontend types and search/display helpers.
- `src/agents/hermesClient.ts`
  - Added Tauri invoke wrappers for listing/executing slash commands.
- `src/agents/slashCommandParser.ts`
  - Added parser that recognizes any slash command and resolves registry aliases when loaded.
- `src/agents/recentCommands.ts`
  - Added bounded, sensitive-filtered recent slash command persistence.
- `src/agents/CommandPalette.tsx`
  - Added Zoid-native command palette over live command registry and recents.
- `src/agents/ChatComposer.tsx`
  - Added slash command dropdown powered by live registry.
  - Fixed registry refresh dependency for filtered commands.
- `src/agents/AgentsHermesScreen.tsx`
  - Loads registry, routes slash sends to command executor, supports Cmd/Ctrl+K palette, recent commands, native command panels, and confirmation modal.
- `src/App.css`
  - Added command palette, native command panel, and confirmation modal styling.
- `src/scaffold.test.ts`
  - Added assertions for live registry, no static duplicated command list, palette/confirmation wiring, recents, and style coverage.

## Validation run
- `npm run test:rust && npm run test:frontend && npm run build` passed after final fixes.
  - Rust: 41 passed, 0 failed.
  - Frontend scaffold/dropdown tests: passed.
  - TypeScript/Vite build: passed with only existing chunk-size warning.
- Manual registry import check passed with 71 Hermes commands from `/Users/ziadnasreldin/.hermes/hermes-agent`.

## Fix cycle notes
- Fixed confirmation execution so confirmed commands execute the same pending command context instead of re-entering the confirmation path.
- Added native Zoid command panel UI for panel-classified Hermes commands instead of only writing a chat message.
- Fixed pre-existing/local regression guards that were failing after context compression: archive persistence invariant and two Rust test stability assertions.
- After the first critique reported stale failing gates, reran the full combined gate and confirmed it now passes.

## Notes / known limitations to review
- Native command panels are lightweight Zoid-native surfaces for panel-classified Hermes commands. Deeper full settings/model/history implementations can be follow-up product work.
- `/history` and `/save` are classified as native-panel/history rather than forwarded blindly.
- `/model` current-session scope is represented as a native model panel open; actual per-session model editing remains existing/future settings work.
- Hermes registry loading imports source Python directly, because no verified stable `hermes commands --json` CLI endpoint was available.

## Review request
Act as the critique-agent. Inspect the implementation against the requirements above. Return a verdict of APPROVED or REQUIRED_FIXES. If required fixes exist, list them as actionable bullets with file paths and expected behavior.
