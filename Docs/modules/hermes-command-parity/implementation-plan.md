# Hermes Command Parity Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. This is a Zoid 25 feature and must pass the feature-critique-workflow before completion.

**Goal:** Make every default Hermes slash command work inside Zoid 25 as a native-feeling Zoid command surface, with Hermes as the command source of truth.

**Architecture:** Zoid should not duplicate Hermes slash commands. The Tauri backend will load the live Hermes command registry from the installed Hermes source/CLI, expose typed command metadata to React, route command execution through the active Zoid session's Hermes CLI bridge, and let React render Zoid-native autocomplete, command palette, confirmations, and panels. Hermes remains the behavioral reference; Zoid owns presentation.

**Tech Stack:** Tauri v2, Rust backend commands, React/TypeScript frontend, existing `hermes` CLI bridge, existing Zoid session state, existing `GlobalDropdown`, Hermes command registry at `hermes_cli/commands.py`.

---

## Confirmed Product Decisions

1. Support **all Hermes slash commands exactly**, including aliases and subcommands.
2. Do **not** duplicate a static command list in Zoid.
3. Slash commands affect the **current Zoid session/tab** unless Hermes command semantics are global.
4. `/model` changes only the current Zoid session.
5. `/tools` and `/toolsets` change Hermes globally/profile-wide, matching Hermes behavior.
6. `/clear` starts a new Zoid/Hermes session.
7. `/history` shows both Zoid local history and Hermes `state.db` history in one panel.
8. `/save` saves both Zoid transcript format and Hermes transcript format.
9. `/restart` restarts Hermes gateway and shows gateway status before/after.
10. TUI visual-only commands like `/redraw`, `/skin`, `/statusbar` are accepted but treated as not needed in Zoid.
11. Command autocomplete uses Zoid visual style, but content comes from Hermes.
12. Commands work in both the chat composer slash dropdown and a command palette.
13. Aliases and subcommand autocomplete must work.
14. Recent commands must be remembered.
15. Zoid adds branded confirmation for dangerous commands.
16. The UI should feel native to Zoid with no Hermes branding in command surfaces.
17. When Hermes adds a command, Zoid should auto-update from the Hermes registry.
18. Command support must be testable without launching the full app, then verified in-app before completion.

---

## Current Code Facts

- Zoid already spawns the Hermes CLI from Rust:
  - `src/agents/hermesClient.ts`
  - `src-tauri/src/lib.rs`
- Current message flow:
  - React calls `invoke("send_hermes_cli_message")`
  - Rust finds `hermes`
  - Rust runs `hermes chat --quiet --source desktop --query ...`
- `ChatComposer.tsx` currently has a hardcoded `slashCommands` array. This must be replaced with registry-backed data.
- The Tauri backend already exposes Hermes automation/profile/provider commands; extending it for command registry and command execution fits the existing architecture.

---

## Command Scope Rules

| Command Type | Examples | Zoid Behavior |
|---|---|---|
| Current session | `/new`, `/reset`, `/retry`, `/undo`, `/title`, `/compress`, `/goal`, `/queue`, `/steer`, `/model` | Affect active Zoid session/tab only |
| Global Hermes/profile | `/tools`, `/toolsets`, `/skills`, `/reload-skills`, `/reload-mcp`, `/cron`, `/curator`, `/kanban`, `/plugins`, `/restart`, `/sethome`, `/update` | Delegate to Hermes/global behavior with Zoid confirmations where needed |
| Native Zoid panels | `/model`, `/tools`, `/skills`, `/cron`, `/agents`, `/profile`, `/history`, `/usage`, `/debug`, `/browser` | Open Zoid panel/modal where available; fallback to Hermes text output |
| File/clipboard | `/copy`, `/paste`, `/image`, `/save` | Use Zoid/Tauri native capabilities and keep Hermes semantics |
| Visual TUI-only | `/redraw`, `/skin`, `/statusbar`, `/indicator`, `/busy` if not applicable | Accept command, show small no-op response: “Not needed in Zoid.” |
| Exit | `/quit`, `/exit`, `/q` | Close/end the active Zoid session/tab only |

---

## Dangerous Command Confirmation Policy

Zoid must show branded confirmation before executing:

- `/yolo`
- `/rollback`
- `/snapshot restore`
- `/stop`
- `/restart`
- `/cron remove`
- `/sessions delete`
- `/profile delete`
- `/update`
- `/uninstall` or any equivalent Hermes CLI destructive path if exposed
- Any Hermes command metadata marked destructive/approval-required once available

Confirmation must include:

- command string
- command scope: current session vs global Hermes profile/gateway
- expected side effect
- primary action label matching the command, not generic “OK”
- cancel as safe default

---

## Implementation Tasks

### Task 1: Add shared TypeScript command types

**Objective:** Define the frontend contract for Hermes command metadata, command execution results, and recent-command entries.

**Files:**
- Create: `src/agents/hermesCommands.ts`
- Modify: `src/agents/hermesClient.ts`

**Steps:**
1. Create `HermesSlashCommand` type with:
   - `name: string`
   - `aliases: string[]`
   - `description: string`
   - `category: string`
   - `argsHint?: string`
   - `subcommands: string[]`
   - `cliOnly: boolean`
   - `gatewayOnly: boolean`
   - `zoidBehavior: "native-panel" | "forward" | "noop" | "confirm-forward"`
   - `panel?: "model" | "tools" | "skills" | "cron" | "agents" | "profile" | "history" | "usage" | "debug" | "browser"`
2. Create `HermesSlashCommandExecution` result type with:
   - `kind: "text" | "panel" | "confirmation" | "new-session" | "close-session" | "error"`
   - `content?: string`
   - `session?: string`
   - `panel?: HermesSlashCommand["panel"]`
   - `requiresConfirmation?: boolean`
3. Add client functions:
   - `listHermesSlashCommands()` → `invoke("list_hermes_slash_commands")`
   - `executeHermesSlashCommand(command, session, linkedRepository, confirmed)` → `invoke("execute_hermes_slash_command")`

**Verification:**
- Run `npm run build` after later frontend wiring.
- TypeScript should reject unknown panel names and invalid execution kinds.

---

### Task 2: Add Rust command metadata structs

**Objective:** Mirror the frontend command metadata contract in Rust.

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Steps:**
1. Add `HermesSlashCommand` Rust struct with `Serialize`.
2. Add `HermesSlashExecutionResult` Rust struct with `Serialize`.
3. Add `ZoidCommandBehavior` enum or string helpers.
4. Add helper `classify_zoid_command_behavior(command_name: &str) -> (String, Option<String>)`.
5. Map native panels:
   - `model`, `reasoning` → `model`
   - `tools`, `toolsets` → `tools`
   - `skills`, `skill`, `reload-skills` → `skills`
   - `cron` → `cron`
   - `agents`, `tasks`, `background`, `queue`, `steer` → `agents`
   - `profile` → `profile`
   - `history`, `resume` → `history`
   - `usage`, `insights`, `status` → `usage`
   - `debug` → `debug`
   - `browser` → `browser`
6. Map no-op commands:
   - `redraw`, `skin`, `statusbar`, `indicator` unless Zoid-native equivalents are added later.
7. Map confirm-forward commands using the dangerous command policy above.

**Verification:**
- Run `cargo test --manifest-path src-tauri/Cargo.toml --lib --bins -- --test-threads=1` after tests are added.

---

### Task 3: Load Hermes command registry without duplicating it

**Objective:** Make Zoid read live Hermes slash command metadata instead of maintaining a hardcoded command list.

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Preferred Implementation:**
1. Add helper `find_hermes_source_root() -> Option<PathBuf>` that checks:
   - `$ZOID_HERMES_SOURCE`
   - `~/.hermes/hermes-agent`
   - sibling/candidate paths if needed
2. Add helper that runs Python against Hermes source:
   ```bash
   python -c 'import json; from hermes_cli.commands import COMMAND_REGISTRY; ... print(json.dumps(...))'
   ```
3. Serialize each `CommandDef` field:
   - name
   - aliases
   - description
   - category
   - args_hint
   - subcommands
   - cli_only
   - gateway_only
4. If direct import fails, fallback to `hermes chat -q "/commands"` only as a degraded text source, but the main UX should show an explicit warning that structured command metadata is unavailable.
5. Do **not** add a static complete command list in Zoid.

**Better Upstream Follow-Up:**
- If Hermes does not expose JSON registry via CLI, later add a Hermes command like `hermes commands --json`. Zoid can then prefer that over Python import.

**Tests:**
- Rust test with fixture JSON proving parse/classification works.
- Rust test that missing source returns a clear unavailable state, not fake commands.

**Verification:**
- Command list includes aliases like `/q`, `/exit`, `/tasks`, `/reset` through metadata.
- Zoid does not contain a hardcoded full slash-command array after this task.

---

### Task 4: Expose `list_hermes_slash_commands` Tauri command

**Objective:** Let the frontend request live command metadata.

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/agents/hermesClient.ts`

**Steps:**
1. Add `#[tauri::command] pub async fn list_hermes_slash_commands() -> Result<Vec<HermesSlashCommand>, String>`.
2. Register it in the Tauri invoke handler near the existing Hermes commands.
3. In the command, load live Hermes metadata and apply Zoid behavior classification.
4. Sort by category then name, preserving aliases in the command object.
5. Add frontend client wrapper.

**Verification:**
- Add a temporary/manual frontend call or unit test to ensure commands are returned.
- `npm run build` must still pass.
- Rust test validates aliases and subcommands are present when source data includes them.

---

### Task 5: Replace hardcoded composer slash commands

**Objective:** Make the composer slash dropdown use live Hermes command metadata with Zoid styling.

**Files:**
- Modify: `src/agents/ChatComposer.tsx`
- Modify as needed: `src/agents/AgentsHermesScreen.tsx`

**Steps:**
1. Remove the hardcoded `slashCommands` array.
2. Accept `slashCommands: HermesSlashCommand[]` as a prop.
3. Filter by:
   - command name
   - aliases
   - category
   - description
   - args hint
   - subcommands
4. Render command rows using Zoid styling:
   - `/name`
   - aliases as subtle chips
   - category label
   - args hint
   - description
5. When a command is selected:
   - if it has args/subcommands, insert `/name ` into composer
   - if action-only, allow immediate execution through parent callback
6. Add keyboard navigation if not already present.

**Tests:**
- Extend `src/ui/GlobalDropdown.behavior.test.tsx` or create `src/agents/ChatComposer.commands.test.tsx`.
- Test filtering by alias and subcommand.
- Test selection inserts canonical command.

**Verification:**
- Start Zoid and type `/`.
- Commands shown match Hermes metadata.
- No Hermes branding appears in the dropdown.

---

### Task 6: Add command palette

**Objective:** Add a Zoid-native command palette that exposes the same Hermes command registry.

**Files:**
- Create: `src/agents/CommandPalette.tsx`
- Modify: `src/agents/AgentsHermesScreen.tsx`
- Modify: `src/App.css`

**Steps:**
1. Add `Cmd+K` / `Ctrl+K` listener scoped to the Hermes/Zoid agent workspace.
2. Render a modal/palette using Zoid visual language.
3. Search same fields as composer:
   - command name
   - aliases
   - category
   - description
   - subcommands
4. Include “Recent” section from local storage.
5. Selecting command should:
   - open native panel if `zoidBehavior === "native-panel"` and command has no required args
   - insert command into composer if args/subcommands are needed
   - run action-only command after confirmation if required
6. Escape closes palette.

**Tests:**
- Frontend test for opening with keyboard event.
- Search by alias returns canonical command.
- Recent commands render before all commands.

**Verification:**
- `Cmd+K` opens palette.
- `/tasks` appears as alias for `/agents` if provided by Hermes.
- Recent commands persist after reload.

---

### Task 7: Add recent-command storage

**Objective:** Remember recent commands safely.

**Files:**
- Create: `src/agents/recentCommands.ts`
- Modify: `src/agents/CommandPalette.tsx`
- Modify: `src/agents/ChatComposer.tsx` or parent execution path

**Steps:**
1. Store last 20 command strings in local storage under `zoid25:recent-hermes-commands`.
2. Dedupe by canonical command + args.
3. Do not store commands containing obvious secret patterns or pasted long values.
4. Save command after successful execution or panel open.
5. Show recent commands in palette.

**Tests:**
- Add simple TS test for dedupe, max length, and secret skipping.

**Verification:**
- Run command, reopen palette, command appears under Recent.

---

### Task 8: Implement slash command parser and alias resolver

**Objective:** Normalize command input before routing.

**Files:**
- Create: `src/agents/slashCommandParser.ts`
- Modify: `src/agents/AgentsHermesScreen.tsx`

**Steps:**
1. Parse a composer value that starts with `/` into:
   - raw command string
   - typed command token
   - args string
2. Resolve aliases using live command metadata.
3. Preserve original raw text for display/history.
4. Use canonical command for behavior routing.
5. Subcommands remain in args.
6. Non-slash messages continue through current `sendHermesCliMessage` flow.

**Tests:**
- `/q` resolves to `quit`.
- `/tasks` resolves to `agents`.
- `/model gpt-x` resolves canonical command and args.
- Regular user prompt is not parsed as command.

**Verification:**
- Alias commands execute as canonical behavior.

---

### Task 9: Add backend slash command execution endpoint

**Objective:** Execute slash commands through Rust with session-aware behavior.

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/agents/hermesClient.ts`

**Steps:**
1. Add `#[tauri::command] execute_hermes_slash_command(command, hermes_session, linked_repository, confirmed)`.
2. Parse canonical command on backend as a safety check.
3. If command is dangerous and `confirmed != true`, return `requiresConfirmation` result.
4. For no-op visual commands, return text: `Not needed in Zoid.`
5. For `/restart`, call `hermes gateway status`, then `hermes gateway restart`, then `hermes gateway status`, returning before/after text.
6. For forwarded commands, run through existing Hermes CLI path in the active session.
7. Ensure `/model` current-session behavior does not globally persist unless Hermes explicitly requires it; prefer session runtime args/state in Zoid.
8. Ensure `/quit` returns `close-session`, not app exit.
9. Ensure `/clear` / `/new` returns `new-session` with a fresh local session id and no old Hermes resume id.

**Tests:**
- Rust test for dangerous command requiring confirmation.
- Rust test for no-op visual command.
- Rust test for `/quit` returns close-session.
- Rust test for `/clear` returns new-session.

**Verification:**
- Executing `/help` returns useful Hermes command output.
- Executing `/redraw` returns Zoid no-op response.

---

### Task 10: Wire command execution into `AgentsHermesScreen`

**Objective:** Make slash commands run through the new command execution path instead of ordinary chat prompts.

**Files:**
- Modify: `src/agents/AgentsHermesScreen.tsx`

**Steps:**
1. Load slash commands on mount and on explicit refresh.
2. Pass commands to `ChatComposer` and `CommandPalette`.
3. In send handler:
   - if input starts with `/`, call command parser/executor
   - else keep current message flow
4. Add user message bubble for the command only if it should appear in transcript.
5. Add assistant/system-style response bubble for command output.
6. Handle `panel` result by opening the appropriate Zoid panel/modal.
7. Handle `new-session` by creating a fresh Zoid session/tab and clearing Hermes session id.
8. Handle `close-session` by closing/removing active session/tab using existing archive/close behavior.
9. Save recent command on success.

**Tests:**
- Frontend test command path does not call normal `sendHermesCliMessage`.
- Frontend test `/clear` creates new session callback.
- Frontend test `/quit` calls close/archive callback.

**Verification:**
- `/help` renders a response.
- `/clear` starts a new session.
- `/quit` closes only active Zoid session/tab.

---

### Task 11: Build branded confirmation modal

**Objective:** Add reusable Zoid confirmation for dangerous slash commands.

**Files:**
- Create: `src/ui/ZoidConfirmDialog.tsx` or reuse existing branded modal if present
- Modify: `src/agents/AgentsHermesScreen.tsx`
- Modify: `src/App.css`

**Steps:**
1. Modal props:
   - `title`
   - `description`
   - `command`
   - `scope`
   - `confirmLabel`
   - `tone: "warning" | "danger"`
2. Cancel is default focus.
3. Confirm re-runs command with `confirmed: true`.
4. Never use native `confirm()`.

**Tests:**
- Modal opens for `/yolo`.
- Cancel does not execute backend command.
- Confirm executes with `confirmed: true`.

**Verification:**
- `/restart` shows confirmation before restarting gateway.

---

### Task 12: Native `/model` current-session panel

**Objective:** Implement `/model` as a Zoid-native session-level model selector.

**Files:**
- Modify: `src/agents/AgentsHermesScreen.tsx`
- Modify or create: `src/agents/ModelCommandPanel.tsx`
- Modify: `src/agents/hermesProfileClient.ts` if needed

**Steps:**
1. Reuse existing profile/provider settings where possible for model/provider options.
2. Store selected model/provider on the active Zoid session, not globally.
3. Pass session model/provider into Hermes runtime args for that session.
4. `/model` without args opens panel.
5. `/model <name>` updates current Zoid session model if valid, otherwise shows error.
6. Footer model label reflects session override.

**Tests:**
- Selecting model updates only active session.
- Another session keeps its prior/default model.

**Verification:**
- Two Zoid sessions can have different displayed model settings.

---

### Task 13: Native `/tools` and `/toolsets` global panel

**Objective:** Implement tools/toolsets commands as a Zoid-native global Hermes profile control.

**Files:**
- Create or modify: `src/agents/ToolsCommandPanel.tsx`
- Modify: `src-tauri/src/lib.rs`

**Steps:**
1. Add backend helpers that call/read Hermes tool state, preferably via `hermes tools list` or config where available.
2. `/tools` opens panel.
3. `/toolsets` shows current toolsets and their global status.
4. Toggles persist globally/profile-wide.
5. Show note that changes may require new session/reset, matching Hermes behavior.

**Tests:**
- Panel renders disabled/loading/error states.
- Backend rejects invalid toolset names.

**Verification:**
- Toggle state matches Hermes `tools list`/config output.

---

### Task 14: Native `/history`, `/resume`, and `/save`

**Objective:** Implement combined Zoid + Hermes history and dual-format saving.

**Files:**
- Create: `src/agents/HistoryCommandPanel.tsx`
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/agents/AgentsHermesScreen.tsx`

**Steps:**
1. Add backend command to list Hermes sessions using Hermes CLI/session DB access.
2. Combine with local Zoid sessions in one panel.
3. Clearly label source as “Zoid” or “Hermes runtime” without front-facing Hermes branding emphasis.
4. `/resume` attaches active Zoid session to chosen Hermes session id.
5. `/save` exports:
   - Zoid transcript JSON/Markdown
   - Hermes transcript/export where available
6. Use Tauri file dialog or app data directory for save location.

**Tests:**
- Combined list merges both sources without duplicate crashes.
- Save creates both expected payloads via testable backend helper.

**Verification:**
- `/history` opens one panel showing both sources.
- `/save` produces both files.

---

### Task 15: Native wrappers for `/cron`, `/skills`, `/agents`, `/profile`, `/usage`, `/debug`, `/browser`

**Objective:** Route major command groups to native Zoid panels, falling back to Hermes text output where a full panel is not implemented yet.

**Files:**
- Modify existing automation/profile/agent screens where available:
  - `src/automations/AutomationsWorkspace.tsx`
  - `src/agents/AgentsHermesScreen.tsx`
  - `src/agents/hermesProfileClient.ts`
- Create focused panel components as needed.

**Steps:**
1. `/cron` opens existing Automations/Cron control plane.
2. `/skills` opens skills panel or forwards to Hermes until panel is built.
3. `/agents` and `/tasks` open active agents/tasks panel.
4. `/profile` opens active profile info/settings panel.
5. `/usage` renders stats card if parseable, otherwise text output.
6. `/debug` runs Hermes debug command and displays report output/links in a card.
7. `/browser` opens or attaches browser connection panel if available; otherwise forwards.

**Tests:**
- Each command returns expected panel id.
- Alias `/tasks` routes to agents panel.

**Verification:**
- Palette can open every native panel.

---

### Task 16: Subcommand autocomplete

**Objective:** Suggest subcommands/argument hints from Hermes metadata.

**Files:**
- Modify: `src/agents/ChatComposer.tsx`
- Modify: `src/agents/CommandPalette.tsx`

**Steps:**
1. When input is `/command ` and command has `subcommands`, show subcommand suggestions.
2. Selecting subcommand inserts `/command subcommand`.
3. Show `argsHint` when no subcommands exist.
4. Keep aliases valid but insert canonical command unless user explicitly typed alias.

**Tests:**
- `/reasoning ` suggests known levels if Hermes metadata exposes them.
- `/goal ` suggests status/pause/resume/clear if metadata exposes them.

**Verification:**
- User can complete nested command without memorizing options.

---

### Task 17: Gateway restart status flow

**Objective:** Make `/restart` restart Hermes gateway with status before/after.

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Create/modify: `src/agents/GatewayStatusPanel.tsx` if needed

**Steps:**
1. Backend runs `hermes gateway status` and captures status.
2. Backend runs `hermes gateway restart` after confirmation.
3. Backend runs `hermes gateway status` again.
4. Frontend displays before/after in a Zoid status card.
5. Failure states show stderr and next action.

**Tests:**
- Use injectable command runner or helper unit tests to simulate status/restart/status.
- Verify restart is blocked without confirmation.

**Verification:**
- `/restart` confirmation appears.
- After confirm, status before/after is visible.

---

### Task 18: Visual-only no-op commands

**Objective:** Accept TUI-specific commands without confusing users.

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/agents/AgentsHermesScreen.tsx`

**Steps:**
1. Route `/redraw`, `/skin`, `/statusbar`, `/indicator` to a no-op result.
2. Response copy: `Not needed in Zoid.`
3. Do not show errors for these commands.
4. Keep them visible in autocomplete because product decision is all commands exactly.

**Tests:**
- Each no-op returns success text, not error.

**Verification:**
- Typing `/redraw` does not break the session.

---

### Task 19: Testing pass

**Objective:** Verify command parity is stable without launching full Zoid.

**Files:**
- Modify/create tests under `src/` and Rust tests in `src-tauri/src/lib.rs` or test modules.

**Commands:**
```bash
npm run test:frontend
npm run test:rust
npm run build
npm run test
```

**Expected:**
- Frontend tests pass.
- Rust tests pass.
- Build passes.

**Coverage Required:**
- Registry loading/classification.
- Alias resolving.
- Composer autocomplete.
- Command palette search/recent commands.
- Dangerous confirmation.
- Current-session `/model` behavior.
- Global `/tools` behavior marker.
- `/clear` new session.
- `/quit` close session.
- No-op visual commands.

---

### Task 20: In-app verification and rebuild

**Objective:** Prove the feature works in the actual Zoid app, not just source/tests.

**Commands:**
```bash
npm run build
npm run tauri:build
open -b com.mavoid.zoid25
```

If bundle identifier launch is unavailable, use the project-approved launch path:
```bash
/Applications/Zoid\ 25.app/Contents/MacOS/zoid
```

**Manual Checks:**
1. Open Agents/Hermes workspace.
2. Type `/` and confirm live commands appear.
3. Search an alias like `/q` or `/tasks`.
4. Use `Cmd+K` and search `model`.
5. Run `/help`.
6. Run `/redraw` and confirm no-op message.
7. Run `/clear` and confirm a new session starts.
8. Run `/quit` and confirm only active Zoid session/tab closes.
9. Run `/restart`, cancel confirmation, then confirm in a controlled run and verify before/after status.
10. Open `/history` and confirm both Zoid and Hermes runtime history are represented.
11. Run `/save` and confirm both output formats are created.

---

### Task 21: Feature critique workflow gate

**Objective:** Complete the required Zoid quality gate before calling the feature done.

**Files:**
- Create: `.hermes/reviews/hermes-command-parity/handoff.md`
- Wait for/create critique report under `.hermes/reviews/hermes-command-parity/critique-report.md`

**Steps:**
1. Write handoff with:
   - requirements
   - changed files
   - test results
   - manual app verification results
   - known limitations, if any
2. Trigger/wait for separate critique-agent review.
3. Fix all Required fixes.
4. Re-review until verdict is `APPROVED`.
5. Only then report feature complete.

**Verification:**
- Critique report verdict is `APPROVED`.

---

## Acceptance Criteria

- No hardcoded full Hermes slash command list remains in Zoid.
- Composer `/` dropdown and `Cmd+K` palette use the same live Hermes registry.
- All Hermes aliases resolve and execute.
- Subcommands appear where Hermes metadata exposes them.
- `/model` is current-session only.
- `/tools` and `/toolsets` are global/profile-wide.
- `/clear` starts a new Zoid/Hermes session.
- `/quit`, `/exit`, `/q` close only the active Zoid session/tab.
- `/history` shows Zoid and Hermes runtime history in one panel.
- `/save` exports both Zoid and Hermes transcript formats.
- `/restart` confirms, restarts Hermes gateway, and shows status before/after.
- TUI-only visual commands are accepted as no-op, not errors.
- Recent commands persist and appear in palette.
- Dangerous commands require branded Zoid confirmation.
- UI feels native to Zoid and does not present itself as a Hermes-branded command surface.
- Tests pass and the app is rebuilt/rerun for visible verification.
- Feature critique workflow reaches `APPROVED`.

---

## Open Implementation Notes

- Prefer adding/upstreaming `hermes commands --json` if direct Python import from `hermes_cli.commands` proves brittle.
- If a command cannot be represented as a native panel in the first implementation slice, it should still forward to Hermes and return text output.
- Do not invent command behavior when Hermes rejects a command. Display Hermes output/error cleanly.
- Keep confirmation decisions backend-owned as well as frontend-rendered so direct invoke calls cannot bypass safety.
