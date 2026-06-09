# Feature Handoff: Composer + Menu

## Original request

User page feedback for `/` at `.hermes-chat-shell > .chat-workspace > .chat-main-pane > .chat-composer`:

When clicking the `+` button, provide options for:
- attach a file, whatever its format
- use slash command from Hermes Agent
- edit agent settings, starting with temperature and possible other settings
- token usage in this session

During grill/planning, resolved scope:
- Hybrid interaction: compact popover first, deeper panels for settings/usage.
- Attachments: multiple files, chips above composer, per-file choices.
- Default attachment behavior: send as context where possible, smart extraction for text-like files, honest unsupported/too-large states.
- Slash commands: searchable picker; normal select inserts into composer; Cmd/Ctrl+Enter can run immediately only when sending is allowed.
- Agent settings: separate Session overrides and Default profile.
- Token usage: actionable panel with usage details and cleanup/compact/start-new/copy report actions, with disabled/honest states where backend wiring is missing.
- While Hermes is responding: allow viewing/next-message prep; risky live setting changes disabled/queued.
- V1 visible menu: only four actions, implemented via internal action registry.

## Implementation summary

- Replaced disabled `+` placeholder with an active composer actions popover.
- Added four top-level actions with subtitles and subtle live badges/status:
  - Attach files
  - Slash commands
  - Agent settings
  - Session usage
- Added hidden multi-file input, attachment tray, and attachment chips above the composer.
- Added per-file chip actions: `Send as context`, `Extract text`, `Upload only`, plus remove.
- Added attachment context building:
  - text/code/log-like files are read and clipped into prompt context;
  - binary/non-text files are represented as honest metadata until backend file ingestion is connected;
  - too-large files are marked and not silently treated as processed.
- Added searchable slash-command panel with `/plan`, `/grill-me-nate`, `/handoff`, `/zoom-out` seed commands.
- Added agent settings panel with Session overrides (temperature, max output tokens) and Default profile summary.
- Added session usage panel with context percentage, unavailable token estimates, contributor hint, remove attachments, disabled compact/start-new actions, and copy usage report.
- Kept backend gaps honest with disabled controls/copy that says Hermes wiring/profile support is required.
- Passed current context percent/model from `AgentsHermesScreen` into `ChatComposer`.
- Added scaffold test guards for the new composer surfaces and styling.

## Changed files

- `src/agents/ChatComposer.tsx`: main implementation of action registry, popover, attachment chips/actions/context, slash picker, settings panel, and usage panel.
- `src/agents/AgentsHermesScreen.tsx`: passes context/model status into composer.
- `src/App.css`: composer popover/panel/chip/usage styling and input/select focus support.
- `src/scaffold.test.ts`: regression string guards for the requested composer menu surfaces.

## How to test

1. Open Zoid 25, go to Agents/Hermes.
2. Click composer `+`.
3. Expected popover rows:
   - Attach files
   - Slash commands
   - Agent settings
   - Session usage
4. Click Slash commands:
   - searchable dialog opens;
   - selecting `/plan` inserts `/plan ` into composer.
5. Click Agent settings:
   - Session overrides and Default profile sections are visible;
   - temperature/max-output controls are present;
   - full profile settings button is disabled with honest Hermes wiring copy.
6. Click Session usage:
   - context usage/actions panel opens;
   - compact/start-new actions are disabled until Hermes wiring exists;
   - remove attachments/copy report actions are present.
7. Attach files:
   - multiple files can be selected;
   - chips render above composer;
   - each chip has Send as context / Extract text / Upload only / remove.

## Tests run

- `npm run test`: PASS
  - Frontend scaffold checks passed.
  - Rust tests passed serially: 15 passed, 0 failed.
- `npm run build`: PASS
  - TypeScript + Vite production build passed.
  - Vite warned that one chunk is larger than 500 kB; pre-existing/non-blocking warning.
- `npm run tauri:build`: PASS
  - Built release binary and `Zoid 25.app` bundle.
- Browser verification at `http://127.0.0.1:1420/`: PASS
  - Agents screen loaded.
  - `+` popover displayed the four requested actions.
  - Agent settings badge displays `Requires wiring` rather than implying editable settings are live.
  - Browser console showed 0 errors.
- Installed native app replacement/relaunch: PASS
  - Replaced `/Applications/Zoid 25.app` from `src-tauri/target/release/bundle/macos/Zoid 25.app`.
  - Running process verified: `/Applications/Zoid 25.app/Contents/MacOS/zoid` PID 69338.
  - Native screenshot captured: `/tmp/zoid25-composer-native-final.png`; it shows Zoid 25 running from the installed app.
  - Attempt to click the native `+` via AppleScript coordinate automation was blocked by macOS Assistive Access, so the open-menu visual proof is browser/DOM-based while installed-app launch is native-verified.

## Git info

- Branch: `main` ahead of `origin/main` by 4 before this handoff.
- Commit SHA: not committed by this handoff.
- Working tree note: repo already had broad pre-existing dirty/untracked changes before this feature. Intended feature files are listed above; do not treat the entire dirty tree as this feature.

## Frontend/backend/database notes

- Frontend routes/components: Hermes Agents composer only.
- Backend endpoints/services: no new Tauri/Rust command added in this slice.
- Database tables/migrations: none.
- Backend gaps intentionally shown as disabled/honest UI states:
  - real Hermes slash-command registry connection;
  - exact token input/output metrics;
  - true compaction/start-new-from-summary control;
  - persistent Hermes profile settings mutation;
  - binary file ingestion beyond metadata.

## Reviewer focus areas

- Spec compliance for all four requested + menu actions.
- UX truthfulness: disabled/unwired actions must not pretend backend support exists.
- Attachment behavior: multiple files, chips, per-file actions, default context behavior.
- Interaction safety while disabled/offline/sending.
- CSS/layout: popover and panels should not break composer geometry.
- Scoped dirty tree: review only intended files for this feature unless broader existing changes are relevant.

## Fix cycle notes

First critique verdict was `REQUIRED_FIXES`; all required fixes were addressed:
- Agent settings temperature/max-token controls are now disabled while Hermes settings wiring is unavailable, and the panel copy says the values are shown as shell-only/not applied yet.
- While Hermes is responding, the textarea remains editable for next-message draft prep; Send stays disabled until the current response completes.
- Slash command options now handle keyboard `Cmd/Ctrl+Enter` explicitly in `onKeyDown`, while normal Enter still inserts.
- Re-ran `npm run test && npm run build`: PASS after fixes.
