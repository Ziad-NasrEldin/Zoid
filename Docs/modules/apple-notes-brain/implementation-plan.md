# Apple Notes Brain Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. Feature is not complete until feature-critique-workflow produces `.hermes/reviews/apple-notes-brain/handoff.md` and the critique report verdict is APPROVED.

**Goal:** Build Apple Notes Brain into Zoid 25: a built-in second-brain module with protected Apple Notes 2-way sync, local Brain metadata, Task Candidates, and clarifying-agent handoff before execution.

**Architecture:** Apple Notes remains the human capture surface. Zoid stores a local Brain mirror and owns extracted structure, task state, clarifying sessions, agent briefs, and references. The Rust/Tauri backend talks to macOS Notes through AppleScript/JXA wrappers and persists a sync ledger; React renders the Brain workspace and sends only explicit user-approved Apple Notes write-back commands.

**Tech Stack:** Tauri 2, Rust, React, TypeScript, local JSON persistence under Hermes/Zoid profile storage, macOS `osascript` against Notes.app, existing Hermes CLI bridge for clarifying-agent sessions.

---

## Resolved Product Decisions

These are no longer open questions. Do not re-ask unless implementation proves a hard blocker.

1. Apple Notes support is built into Zoid Brain.
2. V1 supports 2-way sync only for a dedicated user-approved Apple Notes folder named `Zoid Brain`.
3. Other selected Apple Notes folders are read-only imports by default.
4. Onboarding offers to create `Zoid Brain` automatically, but never auto-assumes an existing folder is safe for 2-way sync.
5. Default 2-way sync includes only human note title/body.
6. Zoid-only metadata stays in Zoid: summaries, extracted tasks, task candidates, ambiguity score, clarifying questions, agent briefs, session state, related references, embeddings/index data.
7. AI output writes back to Apple Notes only through explicit user actions.
8. One source note with multiple possible tasks creates separate Task Candidates linked to the same source note.
9. No automatic hard deletes. Apple Notes deletion marks `missingInApple`; Zoid deletion archives/unlinks locally. Actual Apple Notes delete from Zoid requires explicit confirmation.
10. Agents can read synced notes and draft changes, but cannot directly overwrite, delete, or mass-edit Apple Notes in v1.
11. Task/work status lives in Zoid by default, not Apple Notes.
12. Conflict rule: never silently overwrite. If Apple Notes and Zoid both changed since the last sync snapshot, stop and show conflict resolution.

## Important macOS Reality

Apple Notes has no clean public cloud/API sync endpoint for this. V1 must be macOS-local and use the installed Notes app via `osascript`/JXA/AppleScript.

Verified planning check on this machine:

```bash
sdef /System/Applications/Notes.app
```

The Notes scripting dictionary exposes `account`, `folder`, `note`, `body`, `creation date`, and `modification date`, so local Notes automation is feasible.

## Storage Model

Store Brain data in one Zoid-owned JSON file first. Do not add SQLite yet unless JSON performance becomes a real blocker.

Backend storage path:

```text
~/.hermes/zoid-brain.json
```

Use this shape:

```ts
type BrainStore = {
  version: 1;
  sources: AppleNotesSource[];
  notes: BrainNote[];
  extractions: BrainExtraction[];
  taskCandidates: TaskCandidate[];
  clarificationSessions: BrainClarificationSession[];
  conflicts: BrainSyncConflict[];
  updatedAt: string;
};
```

Core types:

```ts
type AppleNotesSource = {
  id: string;
  sourceType: "appleNotes";
  accountName: string;
  folderName: string;
  syncMode: "twoWay" | "readOnly" | "ignored";
  enabled: boolean;
  createdByZoid: boolean;
  lastSyncedAt: string | null;
  lastError: string | null;
};

type BrainNote = {
  id: string;
  sourceType: "appleNotes";
  sourceId: string;
  appleNoteId: string;
  title: string;
  body: string;
  sourceFolder: string;
  accountName: string;
  appleCreatedAt: string | null;
  appleModifiedAt: string | null;
  zoidModifiedAt: string | null;
  importedAt: string;
  lastSyncedAt: string | null;
  lastSyncedTitle: string;
  lastSyncedBody: string;
  lastSyncedHash: string;
  currentHash: string;
  syncStatus: "synced" | "changedInApple" | "changedInZoid" | "conflict" | "missingInApple" | "writeFailed" | "unlinked";
  archived: boolean;
};

type BrainExtraction = {
  id: string;
  noteId: string;
  summary: string;
  topics: string[];
  entities: string[];
  references: string[];
  decisions: string[];
  openQuestions: string[];
  ambiguityScore: number;
  extractedAt: string;
  extractor: "localHeuristic" | "hermes";
};

type TaskCandidate = {
  id: string;
  noteId: string;
  title: string;
  extractedDescription: string;
  status: "needsReview" | "needsClarification" | "readyForAgent" | "sentToAgent" | "done" | "rejected" | "merged";
  priorityGuess: "low" | "normal" | "high";
  readinessScore: number;
  clarificationSessionId: string | null;
  createdAt: string;
  updatedAt: string;
};

type BrainClarificationSession = {
  id: string;
  noteId: string;
  taskCandidateIds: string[];
  status: "draft" | "questioning" | "briefReady" | "sentToAgent" | "archived";
  transcript: Array<{ role: "user" | "assistant"; content: string; createdAt: string }>;
  resolvedBrief: string;
  openQuestions: string[];
  hermesSessionId: string | null;
  createdAt: string;
  updatedAt: string;
};

type BrainSyncConflict = {
  id: string;
  noteId: string;
  appleTitle: string;
  appleBody: string;
  zoidTitle: string;
  zoidBody: string;
  detectedAt: string;
  resolvedAt: string | null;
  resolution: "keepApple" | "keepZoid" | "manualMerge" | "saveBoth" | null;
};
```

## Build Order

### Task 1: Add Brain glossary to durable context

**Objective:** Keep the domain language stable before code starts.

**Files:**
- Already modified: `CONTEXT.md`

**Work:**
- Confirm `Brain Note`, `Task Candidate`, and `Apple Notes Brain` exist in `CONTEXT.md`.
- Keep this file implementation-light.

**Verify:**

```bash
grep -n "Apple Notes Brain\|Brain Note\|Task Candidate" CONTEXT.md
```

Expected: all three terms appear.

---

### Task 2: Create Brain module docs and acceptance checklist

**Objective:** Put the full product scope in a module folder so implementers do not lose the decisions above.

**Files:**
- Create: `Docs/modules/apple-notes-brain/implementation-plan.md` — this file.
- Create later if needed: `Docs/modules/apple-notes-brain/acceptance-checklist.md`

**Acceptance checklist content:**
- `Zoid Brain` folder can be created from Zoid.
- `Zoid Brain` folder syncs title/body both ways.
- Read-only folders import but do not write.
- Conflict detection stops overwrite.
- Deletions do not hard-delete automatically.
- Task Candidates remain Zoid-owned.
- Clarifying session can produce an agent-ready brief.
- Agents cannot write Apple Notes without explicit user action.

**Verify:**

```bash
find Docs/modules/apple-notes-brain -maxdepth 1 -type f -print
```

Expected: implementation plan and acceptance checklist exist.

---

### Task 3: Add Rust Brain data types

**Objective:** Define backend models before adding Apple Notes automation.

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Implementation:**
Add serializable Rust structs mirroring the storage model:
- `BrainStore`
- `AppleNotesSource`
- `BrainNote`
- `BrainExtraction`
- `TaskCandidate`
- `BrainClarificationSession`
- `BrainSyncConflict`
- enums as string-backed fields for easy frontend compatibility

Use `#[serde(rename_all = "camelCase")]` on all structs.

**Tests:**
Add tests near existing Rust tests:
- default empty store serializes/deserializes
- `TaskCandidate` status round-trips
- `BrainNote.sync_status` round-trips

**Verify:**

```bash
npm run test:rust
```

Expected: Rust tests pass.

---

### Task 4: Add local Brain storage helpers

**Objective:** Persist Zoid Brain state safely with backups.

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Implementation:**
Add helpers following the existing provider/profile storage pattern:

```rust
fn brain_storage_path() -> Result<PathBuf, String> {
    Ok(hermes_profile_home()?.join("zoid-brain.json"))
}

fn load_brain_store_inner() -> Result<BrainStore, String> { /* read or default */ }

fn save_brain_store_inner(store: &BrainStore) -> Result<(), String> { /* backup then pretty JSON write */ }
```

Use existing `backup_file` style before writes.

**Tests:**
- missing file returns default store
- saving then loading preserves sources/notes/candidates
- malformed JSON returns a readable error

**Verify:**

```bash
npm run test:rust
```

Expected: all Rust tests pass.

---

### Task 5: Add Apple Notes script runner boundary

**Objective:** Create one safe backend boundary for all Notes automation.

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Implementation:**
Add:

```rust
const APPLE_NOTES_TIMEOUT_SECONDS: u64 = 30;

fn run_apple_notes_script(script: &str) -> Result<String, String> {
    let mut command = Command::new("osascript");
    command.arg("-l").arg("JavaScript").arg("-e").arg(script);
    let (success, stdout, stderr) = run_command_with_timeout(&mut command, Duration::from_secs(APPLE_NOTES_TIMEOUT_SECONDS))?;
    if success { Ok(stdout) } else { Err(format!("Apple Notes automation failed: {}", stderr.trim())) }
}
```

Rules:
- Use JXA for JSON-friendly output.
- Never interpolate user text directly into script source without JSON escaping.
- Any write/delete command must be a dedicated backend function with explicit action name.

**Tests:**
- add a pure escaping helper test
- do not require Notes.app in unit tests

**Verify:**

```bash
npm run test:rust
```

Expected: tests pass without launching Notes.

---

### Task 6: Implement Apple Notes folder list and setup commands

**Objective:** Let Zoid discover Notes folders and create the protected `Zoid Brain` folder.

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Register commands in `tauri::generate_handler![...]`

**Commands:**

```rust
#[tauri::command]
fn list_apple_notes_folders() -> Result<Vec<AppleNotesFolder>, String>

#[tauri::command]
fn ensure_zoid_brain_folder() -> Result<AppleNotesSource, String>
```

**Behavior:**
- `list_apple_notes_folders` returns accounts/folders only.
- `ensure_zoid_brain_folder` creates or finds a folder named `Zoid Brain` and saves it as a source with `syncMode = twoWay` and `createdByZoid` set accurately.
- Existing folders are not automatically made 2-way except the explicit `ensure` action.

**Manual verification command:**
Use a local dev-only command or temporary test path to call `ensure_zoid_brain_folder` from the app.

**Verify:**

```bash
npm run test:rust
npm run build
```

Expected: Rust tests and frontend build pass.

---

### Task 7: Implement Apple Notes read import

**Objective:** Import notes from configured Apple Notes sources into Zoid Brain.

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Commands:**

```rust
#[tauri::command]
fn sync_apple_notes_sources() -> Result<BrainStore, String>

#[tauri::command]
fn load_brain_store() -> Result<BrainStore, String>
```

**Behavior:**
- Pull all notes from enabled sources.
- Match existing notes by Apple Notes ID/name where possible.
- Store title/body/source folder/account/modified date/hash.
- For read-only folders, only pull from Apple Notes.
- For 2-way folder, pull changes but do not push yet.
- Missing Apple note marks local note `missingInApple`; it does not delete local data.

**Tests:**
Add pure sync-planning tests that do not call Notes:
- unchanged note -> `synced`
- Apple-only changed note -> `changedInApple` and updates local if Zoid unchanged
- Apple-missing note -> `missingInApple`

**Verify:**

```bash
npm run test:rust
npm run build
```

---

### Task 8: Implement 2-way push and conflict planner

**Objective:** Safely push Zoid edits back to Apple Notes without silent overwrite.

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Commands:**

```rust
#[tauri::command]
fn update_brain_note(note_id: String, title: String, body: String) -> Result<BrainStore, String>

#[tauri::command]
fn push_brain_note_to_apple(note_id: String) -> Result<BrainStore, String>

#[tauri::command]
fn resolve_brain_sync_conflict(conflict_id: String, resolution: String, title: Option<String>, body: Option<String>) -> Result<BrainStore, String>
```

**Rules:**
- Only sources with `syncMode = twoWay` can push.
- Before push, fetch the current Apple title/body/hash.
- If Apple hash differs from `lastSyncedHash` and Zoid hash also differs, create a conflict and stop.
- If only Zoid changed, write title/body to Apple Notes.
- If only Apple changed, pull into Zoid.
- If both changed, require conflict resolution.

**Conflict resolutions:**
- keep Apple
- keep Zoid
- manual merge
- save both

**Tests:**
- conflict when both sides changed
- no conflict when only Zoid changed
- read-only source refuses push
- resolved conflict updates snapshots

**Verify:**

```bash
npm run test:rust
npm run build
```

---

### Task 9: Add Apple Notes delete/archive safety commands

**Objective:** Preserve data by default.

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Commands:**

```rust
#[tauri::command]
fn archive_brain_note(note_id: String) -> Result<BrainStore, String>

#[tauri::command]
fn unlink_brain_note(note_id: String) -> Result<BrainStore, String>

#[tauri::command]
fn delete_apple_note_from_zoid(note_id: String, confirmed: bool) -> Result<BrainStore, String>
```

**Rules:**
- Archive/unlink never touches Apple Notes.
- `delete_apple_note_from_zoid` requires `confirmed = true` and only affects linked Apple note.
- Frontend must show branded centered confirmation before calling confirmed delete.

**Tests:**
- unconfirmed delete rejects
- archive keeps Apple ID but marks archived
- unlink removes active source link but preserves local body

**Verify:**

```bash
npm run test:rust
```

---

### Task 10: Add frontend Brain types and client

**Objective:** Give React a typed API boundary.

**Files:**
- Create: `src/brain/types.ts`
- Create: `src/brain/brainClient.ts`

**Implementation:**
`types.ts` mirrors backend camelCase fields.

`brainClient.ts` exports:

```ts
export function loadBrainStore(): Promise<BrainStore>;
export function listAppleNotesFolders(): Promise<AppleNotesFolder[]>;
export function ensureZoidBrainFolder(): Promise<AppleNotesSource>;
export function syncAppleNotesSources(): Promise<BrainStore>;
export function updateBrainNote(noteId: string, title: string, body: string): Promise<BrainStore>;
export function pushBrainNoteToApple(noteId: string): Promise<BrainStore>;
export function resolveBrainSyncConflict(...): Promise<BrainStore>;
export function archiveBrainNote(noteId: string): Promise<BrainStore>;
export function unlinkBrainNote(noteId: string): Promise<BrainStore>;
export function deleteAppleNoteFromZoid(noteId: string, confirmed: boolean): Promise<BrainStore>;
```

**Verify:**

```bash
npm run build
```

Expected: TypeScript passes.

---

### Task 11: Add Brain navigation workspace

**Objective:** Make Brain first-class in Zoid navigation.

**Files:**
- Modify: `src/App.tsx`
- Create: `src/brain/BrainWorkspace.tsx`
- Modify: `src/App.css`

**Implementation:**
- Add `Brain` to `ActiveWorkspace`.
- Add nav item with calm label, for example: `Brain`, meta `Notes sync`.
- Render `BrainWorkspace` for active workspace.
- Keep UI aligned with Zoid design: editorial lists, ruled divisions, blue architecture, no fake metrics.

**Acceptance:**
- Brain appears in sidebar.
- Opening Brain shows truthful empty state if Apple Notes is not set up.
- No native `<select>`; use `GlobalDropdown` for folder/sync mode choices.

**Verify:**

```bash
npm run test:frontend
npm run build
```

---

### Task 12: Build Brain setup screen

**Objective:** Let user create/link the dedicated Apple Notes folder and add read-only folders.

**Files:**
- Modify: `src/brain/BrainWorkspace.tsx`
- Modify: `src/App.css`

**UI sections:**
- Header: `Apple Notes Brain`
- Source status: connected/not connected/sync error
- Primary action: `Create Zoid Brain folder`
- Secondary action: `Choose existing folder for read-only import`
- Source list with sync mode badges: `2-way`, `read-only`, `ignored`

**Rules:**
- Auto-create is allowed only when user clicks the setup action.
- Existing folders require explicit choice.
- Default mode for non-`Zoid Brain` folder is `readOnly`.

**Frontend tests:**
Add checks in `src/scaffold.test.ts` for:
- `Brain` navigation item
- `Apple Notes Brain` copy
- `Create Zoid Brain folder` action
- no one-off native select for sync modes

**Verify:**

```bash
npm run test:frontend
npm run build
```

---

### Task 13: Build Brain Inbox and note detail editor

**Objective:** Show imported notes and allow Zoid-side edits for 2-way synced notes.

**Files:**
- Modify: `src/brain/BrainWorkspace.tsx`
- Modify: `src/App.css`

**UI:**
- Left list: notes sorted by sync status then modified date.
- Status badges: synced, changed in Apple Notes, changed in Zoid, conflict, missing in Apple Notes, write failed, unlinked.
- Detail pane:
  - title input
  - body textarea
  - source metadata
  - `Save in Zoid`
  - `Push to Apple Notes`
  - `Pull latest`
  - `Archive / unlink`

**Rules:**
- Read-only source notes can be viewed, extracted, and used as context but not pushed.
- `Push to Apple Notes` disabled for read-only sources.
- Missing Apple note shows recovery actions: archive in Zoid or recreate later; do not auto-recreate.

**Verify:**

```bash
npm run test:frontend
npm run build
```

---

### Task 14: Build conflict resolution UI

**Objective:** Make conflicts safe and obvious.

**Files:**
- Modify: `src/brain/BrainWorkspace.tsx`
- Modify: `src/App.css`

**UI:**
- Side-by-side Apple vs Zoid versions.
- Actions:
  - Keep Apple version
  - Keep Zoid version
  - Merge manually
  - Save both as separate notes
- Clear warning: `Zoid will not overwrite Apple Notes automatically.`

**Rules:**
- Conflict badge blocks push until resolved.
- Manual merge requires user-edited title/body before applying.

**Verify:**

```bash
npm run test:frontend
npm run build
```

---

### Task 15: Add local extraction engine for summaries and Task Candidates

**Objective:** Make Brain useful even before full LLM extraction.

**Files:**
- Create: `src/brain/brainExtraction.ts`
- Modify: `src/brain/types.ts`
- Modify: `src/brain/BrainWorkspace.tsx`

**Implementation:**
Start with deterministic heuristics:
- split note body by lines/checklist bullets
- detect action lines containing `todo`, `fix`, `build`, `write`, `ask`, `check`, `implement`, `need`, `should`
- detect question lines ending with `?`
- produce summary from title + first meaningful line
- ambiguity score rises when task lines lack project/app/object/done condition

**Rules:**
- Extraction creates Zoid-only metadata.
- Extraction never writes back to Apple Notes.
- Multiple detected tasks become separate Task Candidates linked to source note.

**Tests:**
- Create test file: `src/brain/brainExtraction.test.ts`
- Add to `test:frontend` script or existing `src/scaffold.test.ts` runner.
- Test one note with 4 mixed tasks creates 4 Task Candidates.
- Test question extraction.
- Test ambiguity scoring.

**Verify:**

```bash
npx tsx src/brain/brainExtraction.test.ts
npm run test:frontend
npm run build
```

---

### Task 16: Add Task Candidate review lane

**Objective:** Keep scribbles from becoming executable work too early.

**Files:**
- Modify: `src/brain/BrainWorkspace.tsx`
- Modify: `src/App.css`

**UI:**
Tabs inside Brain:
- Sources
- Inbox
- Task Candidates
- Questions
- References
- Sessions

Task Candidate actions:
- ignore/reject
- merge
- clarify
- mark ready for agent
- view source note

**Rules:**
- No candidate auto-starts an agent.
- Every candidate keeps source note link visible.

**Verify:**

```bash
npm run test:frontend
npm run build
```

---

### Task 17: Add Clarifying Session creation

**Objective:** Convert scribbles into clean agent briefs before execution.

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/agents/hermesClient.ts`
- Modify: `src/brain/BrainWorkspace.tsx`
- Modify: `src/agents/AgentsHermesScreen.tsx` only if needed to accept prefilled sessions

**Backend command option A:**
Use existing `send_hermes_cli_message` from frontend with a generated prompt.

**Preferred v1 behavior:**
- Create a `BrainClarificationSession` locally first.
- Open or create a Hermes chat session with a prefilled prompt:

```text
You are preparing a Zoid Brain note for execution. Ask only critical clarifying questions. The source note may be messy. Do not start implementation. Convert the note into an agent-ready brief only after the goal, scope, output, and done condition are clear.

Source note title: ...
Source note body: ...
Task candidates: ...
```

**Rules:**
- Clarifying agent asks questions first.
- It does not execute code/research/design until user explicitly starts an execution session.
- Final output is a resolved brief stored in Zoid.

**Verify:**

```bash
npm run test:frontend
npm run build
```

Manual: start a clarification from a messy note and confirm it opens with source context.

---

### Task 18: Add agent-ready brief and execution handoff

**Objective:** Let the user convert a clarified task into an agent session cleanly.

**Files:**
- Modify: `src/brain/BrainWorkspace.tsx`
- Modify: `src/agents/AgentsHermesScreen.tsx` if deep linking/prefill is needed

**Behavior:**
- `Create agent brief` stores:
  - original note excerpt
  - selected task candidates
  - clarification answers
  - linked references
  - acceptance criteria
  - out-of-scope
  - recommended agent type
- `Start Agent Session` opens Agents with that brief.

**Rules:**
- The execution session is separate from clarification.
- Agents still cannot write Apple Notes directly.

**Verify:**
Manual E2E:
1. Create messy note in `Zoid Brain` Apple Notes folder.
2. Sync into Zoid.
3. Extract candidates.
4. Start clarification.
5. Produce brief.
6. Start agent session with the brief.

---

### Task 19: Add explicit Apple Notes write-back actions

**Objective:** Support useful write-back without polluting notes automatically.

**Files:**
- Modify: `src-tauri/src/lib.rs`
- Modify: `src/brain/BrainWorkspace.tsx`

**Actions:**
- Append summary to Apple Note
- Create Apple Note from brief/session
- Write task checklist back to Apple Notes

**Rules:**
- Each action opens branded confirmation.
- Action preview shows exact text to be written.
- Write-back goes through conflict check first.
- No agent can trigger this without user click.

**Verify:**
Manual: append a summary to a test note, sync, confirm it appears in Apple Notes.

---

### Task 20: Add manual E2E script/checklist

**Objective:** Make release verification repeatable because Apple Notes automation cannot be fully unit-tested.

**Files:**
- Create: `Docs/modules/apple-notes-brain/manual-verification.md`

**Checklist:**
1. Launch `/Applications/Zoid 25.app` or dev app.
2. Open Brain.
3. Create `Zoid Brain` Apple Notes folder.
4. Create note in Apple Notes.
5. Sync in Zoid.
6. Edit title/body in Zoid and push.
7. Confirm Apple Notes updates.
8. Edit same note in Apple Notes and sync back.
9. Create conflict by editing both sides before sync; verify conflict UI blocks overwrite.
10. Delete note in Apple Notes; verify Zoid marks missing, not deleted.
11. Archive/unlink in Zoid; verify Apple note remains.
12. Extract multiple Task Candidates from one note.
13. Start clarifying session and confirm it asks questions before execution.
14. Confirm no AI summary/task status is written back automatically.

**Verify:**
Run through the checklist and record results in the review handoff.

---

### Task 21: Wire tests and build gates

**Objective:** Ensure this feature does not regress the app.

**Files:**
- Modify: `package.json` if adding `src/brain/brainExtraction.test.ts` to `test:frontend`
- Modify: `src/scaffold.test.ts`
- Add Rust tests in `src-tauri/src/lib.rs`

**Commands:**

```bash
npm run test:frontend
npm run test:rust
npm run test
npm run build
npm run tauri:build
```

Expected:
- frontend tests pass
- Rust tests pass
- full test script passes
- production frontend build passes
- Tauri build passes or any system-code-signing blocker is documented honestly

---

### Task 22: Run feature critique gate

**Objective:** Satisfy Zoid delivery rule before calling the feature complete.

**Files:**
- Create: `.hermes/reviews/apple-notes-brain/handoff.md`
- Critique output: `.hermes/reviews/apple-notes-brain/critique-report.md`

**Handoff must include:**
- product decisions
- exact files changed
- commands run and outputs
- manual Apple Notes E2E evidence
- conflict/deletion safety evidence
- known limitations

**Required gate:**
- Run critique-agent review.
- Fix every Required fix.
- Re-review until verdict is `APPROVED`.

---

## Backend Command Summary

Register these in `tauri::generate_handler![...]`:

```rust
commands::load_brain_store,
commands::list_apple_notes_folders,
commands::ensure_zoid_brain_folder,
commands::sync_apple_notes_sources,
commands::update_brain_note,
commands::push_brain_note_to_apple,
commands::resolve_brain_sync_conflict,
commands::archive_brain_note,
commands::unlink_brain_note,
commands::delete_apple_note_from_zoid,
```

## Frontend File Summary

Create:

```text
src/brain/types.ts
src/brain/brainClient.ts
src/brain/brainExtraction.ts
src/brain/brainExtraction.test.ts
src/brain/BrainWorkspace.tsx
Docs/modules/apple-notes-brain/acceptance-checklist.md
Docs/modules/apple-notes-brain/manual-verification.md
```

Modify:

```text
CONTEXT.md
src/App.tsx
src/App.css
src/scaffold.test.ts
src/agents/hermesClient.ts
src/agents/AgentsHermesScreen.tsx
src-tauri/src/lib.rs
package.json
```

## Non-Goals for V1

- No full-library 2-way sync.
- No cloud/iCloud API integration.
- No silent background mass editing of Apple Notes.
- No automatic task-status write-back to Apple Notes.
- No automatic hard delete propagation.
- No agent direct-write permission to Apple Notes.
- No fake second-brain metrics or simulated sync status.

## Final Acceptance Criteria

The feature is accepted only when:

1. Zoid can create/use a dedicated `Zoid Brain` Apple Notes folder.
2. Notes in that folder sync title/body both ways.
3. Selected non-Brain folders import read-only.
4. Zoid detects conflicts and blocks silent overwrite.
5. Zoid preserves data on deletes/missing notes.
6. Zoid extracts multiple Task Candidates from one note and links them to the source note.
7. Clarifying sessions ask questions before starting execution.
8. Agent sessions receive the source note + resolved brief context.
9. Agents cannot write Apple Notes without explicit user confirmation.
10. Tests/build pass, manual Apple Notes E2E is documented, and critique verdict is APPROVED.
