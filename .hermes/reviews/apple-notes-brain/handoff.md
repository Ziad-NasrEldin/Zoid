# Feature Handoff: Apple Notes Brain

## Original request

User asked to implement Apple Notes support in Zoid 25 so an Apple Notes folder can be linked to Zoid notes, extracted into a second-brain/reference layer, used as Tasks, and converted from scribbles into clarifying-question sessions before agents start execution.

## Implementation summary

- Added Apple Notes Brain as a first-class Zoid workspace.
- Added local Brain store under the active Hermes profile at `zoid-brain.json`.
- Added macOS Apple Notes JXA bridge boundaries for folder listing, protected `Zoid Brain` folder setup, arbitrary folder linking, and pull-only sync.
- V1 sync is safe: linked sources can be marked read-only/two-way metadata-tracked/ignored, but this implemented slice only reads/imports title/body into Zoid; no automatic Apple Notes writeback, overwrite, or delete is implemented.
- Added conflict-aware merge behavior: changed/missing notes are marked in Zoid, both-sided changes create `BrainSyncConflict` records, and unresolved conflicts are surfaced in the Brain UI.
- Added Apple Notes body normalization before hashing/storage/extraction so common Notes HTML/rich-text bodies become user-visible plain lines before task parsing.
- Added local heuristic extraction for synced notes: summaries, topics/references/entities, ambiguity score, separate task candidates from numbered/TODO/checkbox/task-like lines.
- Added clarifying-session preparation: selected candidates from the same source note create a questioning session and linked candidates. It prepares an agent brief but does not execute Hermes yet.
- Added Brain UI panels for Link Apple Notes folder, Sources, Brain Inbox, Sync Conflicts, Task Candidates, and Clarifying Sessions.
- Built, installed, and relaunched `/Applications/Zoid 25.app`.

Known limitations for this batch:
- Manual write-back to Apple Notes is not implemented in this batch. The UI copy says read/import-only even when a source is metadata-tracked as `twoWay`.
- Real agent-session launch from a clarified brief is not implemented in this batch.
- Full installed-app UI clicking through WebView internals is partially blocked by macOS/Stage Manager accessibility focus returning zero WebView AX children in this environment. The packaged app is rebuilt/reinstalled/running and visible, and the real Apple Notes backend/store/extraction/session flow is covered by a disposable Notes E2E test.

## Changed files

- `src-tauri/src/lib.rs`: Brain store/types, Apple Notes script runner, folder setup/list/link/sync commands, sync-mode semantics, Apple Notes body normalization, conflict records, extraction helpers, clarifying-session command, Rust tests.
- `src/brain/types.ts`: typed frontend model for Brain store, sources, notes, candidates, sessions, conflicts, Apple Notes folders.
- `src/brain/brainClient.ts`: typed Tauri invoke client for Brain commands.
- `src/brain/BrainWorkspace.tsx`: Brain workspace UI and actions.
- `src/App.tsx`: Brain workspace navigation and route rendering.
- `src/App.css`: Brain workspace styling.
- `src/scaffold.test.ts`: source guards for Brain workspace/client/backend command wiring.
- `.hermes/reviews/apple-notes-brain/handoff.md`: this review handoff.

## How to test

- Run `npm run test:frontend`.
- Run `npm run test:rust`.
- Run `npx tsc --noEmit --pretty false`.
- Run `npm run build`.
- Run `npm run tauri:build`.
- Replace `/Applications/Zoid 25.app` with `src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Launch `/Applications/Zoid 25.app`.
- Open Brain from the sidebar and verify visible panels/actions:
  - `Apple Notes Brain`
  - `Create Zoid Brain folder`
  - `Link Apple Notes folder`
  - `List folders`
  - `Sync now`
  - `Sources`
  - `Brain Inbox`
  - `Sync Conflicts`
  - `Task Candidates`
  - `Clarifying Sessions`

## Tests run

- `npm run test:rust`: PASS, 64 passed / 0 failed / 1 ignored; existing Rust dead-code warnings only. Adds ignored real macOS Notes E2E test for disposable folder/note sync.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib apple_notes_real_e2e_sync_extracts_tasks_and_persists_store -- --ignored --test-threads=1 --nocapture`: PASS, 1 passed. Created a disposable Apple Notes folder/note, listed it via the bridge, linked it, synced, verified normalized note body in `zoid-brain.json`, extracted 2 task candidates, created a questioning clarifying session, reloaded persisted store, and cleaned the test folder.
- `npm run test:frontend`: PASS.
- `npx tsc --noEmit --pretty false`: PASS.
- `npm run build`: PASS; Vite chunk-size warning only.
- `npm run tauri:build`: PASS; built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`; existing Rust dead-code warnings only.
- `git diff --check -- src-tauri/src/lib.rs src/App.tsx src/App.css src/scaffold.test.ts src/brain/types.ts src/brain/brainClient.ts src/brain/BrainWorkspace.tsx`: PASS.
- Installed/relaunched app: running process confirmed at `/Applications/Zoid 25.app/Contents/MacOS/zoid` pid 35644.
- Native visual verification after packaged reinstall: `/tmp/zoid-front.png` showed the packaged Zoid 25 app frontmost/running; follow-up Brain UI click-through is partially blocked by Stage Manager/focus/AX returning zero WebView children in this environment.

## Git info

- Branch: current local working tree, branch not asserted.
- Commit SHA: not committed.
- Diff base: current working tree against repo index.
- Note: repository already has many unrelated modified/untracked files. Intended Apple Notes Brain source files are the changed files listed above.

## Frontend/backend/database notes

- Frontend route/component: `BrainWorkspace` is rendered when active workspace is `Brain`.
- Backend commands:
  - `load_brain_store`
  - `list_apple_notes_folders`
  - `ensure_zoid_brain_folder`
  - `link_apple_notes_folder`
  - `sync_apple_notes_sources`
  - `extract_brain_note`
  - `create_brain_clarifying_session`
- Database: no DB migration; JSON store under active Hermes profile, `~/.hermes/zoid-brain.json`.
- macOS integration: JXA via `osascript -l JavaScript -e`, timeout 30 seconds.

## Reviewer focus areas

- Apple Notes safety: no automatic hard deletes, no silent overwrite, no agent writeback.
- Folder-linking flow: list/select/persist Apple Notes source with nullable folder IDs handled.
- Tauri command registration and frontend invoke argument names.
- JSON store persistence and backup behavior.
- Apple Notes body normalization before hashing/extraction/display.
- Conflict records and UI visibility for both-sided Apple/local changes.
- Extraction behavior: scribbles need clarification; multiple tasks become separate candidates.
- Clarifying-session behavior: questions/session prep only, no Hermes execution.
- UI truthfulness: no fake metrics or fake notes.
- Type drift between Rust camelCase structs and TypeScript types.
- Dirty-tree scoping given unrelated existing work.

## Fix cycle notes

Initial review request.

Re-review fix cycle:
- R1: Reproduced/fixed current frontend/typecheck failures. `npm run test:frontend`, `npx tsc --noEmit --pretty false`, and `npm run test:rust` now pass together.
- R2: Added arbitrary Apple Notes folder linking: backend `link_apple_notes_folder`, frontend List folders/select/sync mode/link controls, source persistence, nullable folder IDs handled.
- R3: Added Apple Notes HTML/rich-text normalization before note hashing/storage/extraction plus Rust coverage for div/br/li/link/entity markup.
- R4: Added `BrainSyncConflict` creation when Apple and local Zoid copies both changed, and surfaced unresolved conflicts in Brain UI with Rust coverage.
- R5: Changed TypeScript `AppleNotesFolder.id` to `string | null` to match Rust `Option<String>` and UI labels null IDs safely.
- R6: Implemented truthful sync-mode semantics. `ignored` sources are created disabled, skipped by sync selection even if legacy data has `enabled: true`, and relinking an existing source updates the selected mode. Added Rust coverage for ignored skip and mode updates.
- R7: Added and ran a real macOS Apple Notes E2E test using a disposable Notes folder/note: list folder, link source, sync actual note body through JXA, extract two task candidates, create clarifying session, reload persisted JSON store, and clean the test folder. Rebuilt, reinstalled, relaunched `/Applications/Zoid 25.app`, and confirmed the packaged app process/window is running; full installed WebView AX click-through remains partially blocked by macOS Stage Manager/focus returning zero WebView children in this environment.
- Reran `npm run test:frontend`, `npx tsc --noEmit --pretty false`, `npm run test:rust`, the ignored real Apple Notes E2E, `npm run build`, `npm run tauri:build`, and scoped `git diff --check` after the R6/R7 fixes.

Second re-review fix cycle:
- R1/R2: Reworked `run_command_with_timeout` so stdout/stderr are drained from reader threads even after timeout/kill, with capped partial output in the timeout error. Added focused Rust coverage for partial stdout/stderr draining. Split Apple Notes per-source sync result handling into `apply_apple_notes_source_sync_result`, so one failed source records `lastError` without discarding successful source imports.
- R3/R4/R5: Updated Brain UI copy so v1 `twoWay` is described as metadata tracking only, surfaced partial source sync errors in status/UI, and disabled extraction on conflict/stale/missing sync statuses. Backend extraction now rejects `changedInApple`, `changedInZoid`, `conflict`, and `missingInApple` notes before creating extraction records or task candidates.
- R6/R7/R8: Added `src/brain/BrainWorkspace.behavior.test.tsx` to cover Brain UI behavior: stale/conflict extraction disabled, sync-mode copy, partial sync errors, link arguments, and clarifying-session launch copy. Added Rust coverage for source partial errors and stale/conflict extraction rejection. Frontend test script now runs the Brain behavior test.
- Additional typecheck cleanup: removed stale ruthless reviewer icon usage after TypeScript surfaced unused imports/handlers.

Current verification after second fix cycle:
- `npx tsc --noEmit --pretty false`: PASS.
- `npm run test:frontend`: PASS, including Brain workspace behavior test.
- `npm run test:rust`: PASS, 70 passed / 0 failed / 1 ignored; existing Rust dead-code warnings only.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib apple_notes_real_e2e_sync_extracts_tasks_and_persists_store -- --ignored --test-threads=1 --nocapture`: PASS, 1 passed.
- `npm run build`: PASS; Vite chunk-size warning only.
- `npm run tauri:build`: PASS; built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`; existing Rust dead-code warnings only.
- `git diff --check`: PASS.
- Reinstalled and relaunched `/Applications/Zoid 25.app`; process confirmed at `/Applications/Zoid 25.app/Contents/MacOS/zoid` pid 80758.
- Packaged app AX/window check: PASS, System Events returned `zoid, 1, Zoid 25`.

Third review fix cycle:
- R1 follow-up: `run_command_with_timeout` now starts timeout-managed commands in a Unix process group and kills the whole process group on timeout before draining stdout/stderr. The timeout test now asserts a descendant that keeps pipes open returns in under 1 second; focused run passed in 0.16s with partial stdout/stderr preserved.
- Safety follow-up: `merge_apple_notes_raw_notes` now handles all merge cases explicitly. If Apple is unchanged but the Zoid/local note changed, it preserves local title/body/current hash, marks `changedInZoid`, and keeps extraction blocked instead of silently overwriting local edits. Added Rust coverage for this case.

Current verification after third fix cycle:
- `npx tsc --noEmit --pretty false`: PASS.
- `npm run test:frontend`: PASS.
- `npm run test:rust`: PASS, 71 passed / 0 failed / 1 ignored; existing Rust dead-code warnings only.
- `cargo test --manifest-path src-tauri/Cargo.toml --lib apple_notes_real_e2e_sync_extracts_tasks_and_persists_store -- --ignored --test-threads=1 --nocapture`: PASS, 1 passed.
- `npm run build`: PASS; Vite chunk-size warning only.
- `npm run tauri:build`: PASS; rebuilt `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`; existing Rust dead-code warnings only.
- `git diff --check`: PASS.
- Reinstalled and relaunched `/Applications/Zoid 25.app`; process confirmed at `/Applications/Zoid 25.app/Contents/MacOS/zoid` pid 83165.
- Packaged app AX/window check: PASS, System Events returned `zoid, 1, Zoid 25`.
