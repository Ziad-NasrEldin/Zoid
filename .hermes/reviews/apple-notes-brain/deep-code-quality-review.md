# Deep Code Quality Review: Apple Notes Brain

Verdict: REQUIRED_FIXES

Overall assessment: acceptable direction but fragile. The implementation is wired end-to-end and the core Rust/TypeScript camelCase contract, Tauri command names, read-only Apple Notes import, no hard-delete behavior, local heuristic extraction, and clarification-session creation are mostly coherent. However, the current implementation is not cleanly production-safe yet: Apple Notes automation can hang or fail whole-sync without per-source recovery, sync mode semantics are incomplete/overstated for `twoWay`, generated conflict/missing statuses are not fully actionable in UI, and tests are mostly unit/string-presence checks with the only real Apple Notes E2E ignored and destructive if run. These are fixable, but I would not approve as fully functioning, clean, safe, and maintainable without the required fixes below.

## Required fixes

| ID | Severity | file:line | Issue | Evidence | Required change |
|---|---:|---|---|---|---|
| R1 | High | `src-tauri/src/lib.rs:2349-2376`, `src-tauri/src/lib.rs:1501-1517`, `src-tauri/src/lib.rs:1891-1919` | Apple Notes sync can deadlock/time out on large note/folder output because stdout/stderr are piped but not drained until after `wait_timeout`. | `run_command_with_timeout` spawns with piped stdout/stderr at lines 2353-2356, then waits for process exit at lines 2359-2366 before reading output at lines 2368-2376. Apple Notes note bodies are serialized as one JSON stdout payload at lines 1907-1917; enough notes/body content can fill the OS pipe and block `osascript`, causing a false timeout. | Replace this helper for output-heavy commands with concurrent pipe draining, temp-file output, or a proven timeout wrapper that drains stdout/stderr while waiting. Add a test or documented size bound. |
| R2 | High | `src-tauri/src/lib.rs:1933-1952` | One source failure aborts the entire sync and does not persist `lastError` for the failing source or preserve successful later sources. | `sync_apple_notes_sources_inner` loops sources, but `run_apple_notes_script`/JSON parse errors use `?` at line 1939, immediately returning before lines 1941-1948 can update any source error state and before line 1951 can save. | Handle errors per source: set that source `last_error`, continue to other enabled sources, save the store, and return either a partial-success store with error details or a structured error that still persists state. |
| R3 | Medium | `src-tauri/src/lib.rs:1922-1931`, `src/brain/BrainWorkspace.tsx:53`, `src/brain/types.ts:1` | `twoWay` sync mode is misleading/incomplete: backend treats `readOnly` and `twoWay` identically for sync; no writeback path exists and no `changedInZoid` status is produced by any reviewed path. | `apple_notes_syncable_sources` includes all enabled non-ignored sources without branching on `sync_mode` at lines 1922-1931. UI labels `twoWay` as “Two-way metadata tracking (no writeback yet)” at `BrainWorkspace.tsx:53`, but TS exposes real `twoWay` as a sync mode at `types.ts:1`. | Either rename/copy this to an explicit non-writeback tracking mode and remove two-way implication, or implement the documented two-way semantics with strict user-controlled writeback boundaries and tests. |
| R4 | Medium | `src-tauri/src/lib.rs:1785-1793`, `src-tauri/src/lib.rs:2175-2235`, `src/brain/BrainWorkspace.tsx:57-59` | Missing/conflict notes remain extractable and can produce task candidates even when Apple copy is missing or conflicted, with no UI guard. | Missing Apple notes are only marked `sync_status = "missingInApple"` at lines 1785-1793. `extract_brain_note_in_store` only rejects archived/missing local note at lines 2180-2185 and does not block `conflict`/`missingInApple`. The UI renders Extract tasks for every inbox note at `BrainWorkspace.tsx:57` regardless of sync status. | Disable or explicitly gate extraction for `conflict`/`missingInApple` unless user acknowledges they are extracting the stale local copy. Reflect this in backend validation too, not only UI. |
| R5 | Medium | `src/brain/BrainWorkspace.tsx:51` | Error copy can be untrue after partial operations. | On any sync error, UI says “Zoid did not read or write Apple Notes” at line 51. If a sync has read earlier sources and then fails on a later source, or if folder creation creates a folder before later persistence fails, this claim is not guaranteed. | Make failure copy operation-specific and truthful: e.g. “The current operation did not complete; Apple Notes write attempts are limited to explicit folder creation. Some sources may have been read before the error.” Pair with R2 partial-state reporting. |
| R6 | Medium | `src-tauri/src/lib.rs:5652-5772` | The only real Apple Notes E2E test is ignored and has unsafe cleanup guarantees. | Test is `#[ignore = "mutates macOS Notes..."]` at lines 5652-5654. Cleanup only runs at the end lines 5749-5765; any panic before cleanup can leave the disposable folder/note. | Keep it ignored by default, but wrap cleanup in a guard/drop pattern or separate cleanup helper, and document the manual command/environment needed to run it safely. Add non-Apple mock/integration tests for command-level flows. |
| R7 | Low | `src-tauri/src/lib.rs:1520-1540`, `src-tauri/src/lib.rs:1639-1691` | Apple Notes JSON/body parsing lacks schema validation and robust HTML/entity handling. | Folder parser filters empty names at lines 1520-1535, but raw notes parser simply deserializes at lines 1537-1540. Body normalization uses manual tag stripping and a small hard-coded entity map at lines 1639-1691. | Validate raw note fields after deserialization and use a proper HTML/entity decoder or clearly document limitations. Add tests for mixed-case tags, nested tags, escaped entities, and malformed/null fields returned by Notes. |
| R8 | Low | `src/scaffold.test.ts:31-55`, `src/scaffold.test.ts:36-49` | Frontend scaffold tests are brittle presence checks, not behavior tests. | Tests check string inclusion for required surfaces and command names, e.g. lines 31-55, but do not render BrainWorkspace, mock invokes, test loading/error states, selection gating, or extraction actions. | Add React behavior tests with mocked `invoke` covering load success/failure, folder listing/linking, sync error, extract disabled states, candidate selection same-note constraint, and clarifying-session creation. |

## Questions / assumptions that need user/dev confirmation

- Should `twoWay` ever write back to Apple Notes, or is it intentionally only “metadata tracking/no writeback”? Current code implements no Apple note update/delete path.
- Should extraction be allowed on notes whose source status is `missingInApple` or `conflict`, or should these be blocked until resolved?
- Is storing the Brain DB under the active Hermes profile (`hermes_profile_home()/zoid-brain.json`) the intended persistence boundary for Zoid, or should it live under app-specific data storage?
- Are Apple Notes folders expected to be top-level only? Current JXA only loops `account.folders()` and does not obviously traverse nested folders.
- I did not verify actual macOS Notes JXA execution in this review. Verification would require granting the built app/terminal automation permission and running the ignored E2E or a safer manual disposable-folder flow.

## Line-by-line review notes

### `src-tauri/src/lib.rs`

- `src-tauri/src/lib.rs:174-185`: `BrainStore` uses `#[serde(rename_all = "camelCase")]`; this matches TypeScript `taskCandidates`, `clarificationSessions`, and `updatedAt`. Verified OK.
- `src-tauri/src/lib.rs:202-214`: `AppleNotesSource` fields map cleanly to `src/brain/types.ts:15-25`. `sync_mode` is string-backed, so backend can emit invalid strings if introduced elsewhere; current constructors restrict normal paths.
- `src-tauri/src/lib.rs:216-238`: `BrainNote` camelCase contract matches `src/brain/types.ts:27-47`. Verified OK for field names and nullable timestamp fields.
- `src-tauri/src/lib.rs:240-306`: extraction/candidate/session/conflict structs map to TS lines `49-99`. Rust uses free strings where TS narrows unions; this compiles but requires backend discipline/tests.
- `src-tauri/src/lib.rs:979-1006`: Brain store path is `hermes_profile_home()/zoid-brain.json`; load defaults on missing file and save creates a backup. OK for basic persistence, but writes are not atomic; a crash during `fs::write` can corrupt the JSON store.
- `src-tauri/src/lib.rs:1495-1498`: `jxa_json_string_literal` uses `serde_json::to_string`, which is the right escaping strategy for embedding account/folder names in JXA. Verified OK.
- `src-tauri/src/lib.rs:1501-1517`: Apple Notes automation is constrained to `osascript -l JavaScript -e` with a 30-second timeout. Good boundary, but relies on the flawed output-draining helper in R1.
- `src-tauri/src/lib.rs:1520-1535`: folder JSON parsing filters empty account/folder and sorts/dedups. Reasonable.
- `src-tauri/src/lib.rs:1537-1540`: raw note JSON parsing does no post-parse filtering/validation. A malformed Notes/JXA value can persist empty IDs/titles/bodies without explicit handling.
- `src-tauri/src/lib.rs:1542-1553`: simple stable FNV-like hash is fine for change detection but not cryptographic; OK because it is not used as a security primitive.
- `src-tauri/src/lib.rs:1555-1589`: source/note IDs are stable based on sanitized account/folder and Apple note/fallback key. Fallback note ID can change if created/modified timestamps change and no Apple ID is available; UI labels no-folder-id but note fallback behavior should be documented.
- `src-tauri/src/lib.rs:1591-1615`: source upsert deduplicates by source type/account/folder and preserves `created_by_zoid`. It only updates `last_error` when new source has Some; linking a healthy source cannot clear an old error. Sync clears on success later at lines 1946-1948.
- `src-tauri/src/lib.rs:1617-1637`: invalid sync modes fall back to `readOnly`, but `enabled` uses the original `sync_mode != "ignored"`; safe in current callers because `link_apple_notes_folder_inner` validates at lines 1873-1875 and ensure passes `twoWay`.
- `src-tauri/src/lib.rs:1639-1691`: body normalization is deterministic and used before hashing/extraction. Good that hashing uses normalized body. The HTML stripping is simplistic; see R7.
- `src-tauri/src/lib.rs:1693-1794`: merge logic imports raw notes, normalizes body before `note_content_hash`, updates unchanged notes, records conflicts, and marks missing notes without deleting them. Verified no hard deletes in sync path.
- `src-tauri/src/lib.rs:1714-1717`: local-vs-Apple conflict detection checks `note.current_hash` against last synced state. There is no current UI/backend path that edits `note.body/title`, so conflicts are mostly future-proof rather than currently user-reachable except by manual JSON edits.
- `src-tauri/src/lib.rs:1719-1751`: on conflict, the local note body/title remains unchanged and `sync_status` becomes `conflict`; new Apple content is stored only in `BrainSyncConflict`. That is safe.
- `src-tauri/src/lib.rs:1785-1793`: missing Apple notes are marked `missingInApple` and not deleted. This satisfies the no-hard-delete requirement.
- `src-tauri/src/lib.rs:1796-1812`: folder listing JXA only reads accounts/folders. It does not write/delete. Top-level-only behavior is unverified.
- `src-tauri/src/lib.rs:1814-1861`: ensure Zoid Brain folder can create an Apple Notes folder. This is an intentional Apple Notes write; UI exposes a button for it. It does not create notes or delete anything.
- `src-tauri/src/lib.rs:1821`: `Notes.defaultAccount ? Notes.defaultAccount() : Notes.accounts()[0]` may be fragile depending on JXA Notes dictionary behavior; not verified live.
- `src-tauri/src/lib.rs:1863-1889`: linking validates account/folder against live list and persists source. Good user-control boundary.
- `src-tauri/src/lib.rs:1891-1919`: note sync JXA reads `note.name()`, `note.body()`, dates, and id; no write/delete here. String interpolation is safe via JSON literals at lines 1892-1893.
- `src-tauri/src/lib.rs:1922-1931`: `ignored` sources are excluded. `readOnly` and `twoWay` are otherwise identical.
- `src-tauri/src/lib.rs:1933-1952`: sync aborts on first source error and loses per-source error state. See R2.
- `src-tauri/src/lib.rs:2001-2173`: heuristic extraction is intentionally local: hashtags, URLs, capitalized entities, TODO/bullet/numbered/imperative task lines, ambiguity/open questions. This is maintainable enough for a heuristic MVP.
- `src-tauri/src/lib.rs:2131-2138`: multiple task extraction from numbered/body lines is supported; fallback creates at most one title-derived task.
- `src-tauri/src/lib.rs:2175-2235`: extraction replaces previous draft candidates and preserves sent/done candidates. It does not run Hermes and sets extractor `localHeuristic`; OK.
- `src-tauri/src/lib.rs:2246-2336`: clarification session validates same note, creates assistant questions, links candidates, and does not execute an agent. OK for prep-session behavior.
- `src-tauri/src/lib.rs:2349-2376`: shared timeout helper has the pipe-draining deadlock risk described in R1.
- `src-tauri/src/lib.rs:5119-5159`: Tauri command wrappers exist for all Brain operations and call the corresponding inner functions.
- `src-tauri/src/lib.rs:5364-5372`: invoke handler registers `load_brain_store`, `list_apple_notes_folders`, `ensure_zoid_brain_folder`, `link_apple_notes_folder`, `sync_apple_notes_sources`, `extract_brain_note`, and `create_brain_clarifying_session`. Verified direct wiring.
- `src-tauri/src/lib.rs:5652-5772`: real Apple Notes E2E exists but is ignored and can leave artifacts on panic before cleanup.
- `src-tauri/src/lib.rs:5774-5966`: unit tests cover multiple tasks, fallback, empty scribble handling, rerun replacement, clarification, serde round trips. Good coverage of pure logic.

### `src/brain/types.ts`

- `src/brain/types.ts:1-2`: frontend exposes `twoWay`, `readOnly`, `ignored` and statuses including values not currently emitted (`changedInApple`, `changedInZoid`, `writeFailed`, `unlinked`). This is acceptable as future-proofing but can mislead if UI treats them as active semantics.
- `src/brain/types.ts:4-13`: `BrainStore` matches Rust camelCase serde names. Verified OK.
- `src/brain/types.ts:15-105`: TypeScript contracts match Rust structs in reviewed fields. Narrow union types are stricter than Rust string fields, so invalid backend strings would cause UI lookup issues rather than compile-time backend errors.

### `src/brain/brainClient.ts`

- `src/brain/brainClient.ts:4-30`: invoke names and argument names match Tauri commands: `noteId` -> Rust `note_id`, `taskCandidateIds` -> `task_candidate_ids`, `accountName/folderName/syncMode` -> Rust snake_case. Tauri camelCase-to-snake mapping should handle this; command names match exactly. Verified OK.
- `src/brain/brainClient.ts:16`: `syncMode` parameter is typed as `string` rather than `BrainSyncMode`; this weakens compile-time safety.

### `src/brain/BrainWorkspace.tsx`

- `src/brain/BrainWorkspace.tsx:6-7`: labels cover all declared sync modes/statuses. If backend emits an unknown string, indexing will render undefined; constrained Rust constructors reduce but do not eliminate risk.
- `src/brain/BrainWorkspace.tsx:32-46`: async handlers clear bridge errors, set pending state, catch errors, and reset action pending. Good basic loading/error behavior.
- `src/brain/BrainWorkspace.tsx:36`: initial load failure sets “Native Apple Notes bridge is unavailable.” Load only reads local JSON; this message may be too Apple-Notes-specific if JSON parse/profile path fails.
- `src/brain/BrainWorkspace.tsx:40`: create-folder handler first updates local state with source, then reloads store. Redundant but not harmful.
- `src/brain/BrainWorkspace.tsx:41-42`: user must explicitly list folders and link selected folder. Good user-control boundary.
- `src/brain/BrainWorkspace.tsx:43`: sync error status says “failed closed”; paired error panel currently overclaims no read/write, see R5.
- `src/brain/BrainWorkspace.tsx:44`: extraction status truthfully says local heuristics and no agent executed. Good.
- `src/brain/BrainWorkspace.tsx:45-46`: candidate selection enforces same-note selection in UI, and backend also validates same note. Good.
- `src/brain/BrainWorkspace.tsx:48-53`: top copy accurately says mirror approved folders, local extraction, clarify before agent run. Link panel accurately states no delete/writeback during sync.
- `src/brain/BrainWorkspace.tsx:53`: sync mode label “Two-way metadata tracking (no writeback yet)” is better than claiming true writeback, but still confusing because backend behavior is same as read-only except source label.
- `src/brain/BrainWorkspace.tsx:57`: all notes, including conflict/missing, get an Extract tasks button. See R4.
- `src/brain/BrainWorkspace.tsx:58`: conflict panel shows Apple/Zoid snippets and unresolved count. It does not offer resolution; OK for MVP if clearly non-resolving, but conflicts remain stuck.
- `src/brain/BrainWorkspace.tsx:60`: clarifying sessions show questions and state that no execution occurs. Good UI truthfulness.

### `src/App.tsx`

- `src/App.tsx:4`: BrainWorkspace imported directly.
- `src/App.tsx:27-38`: Brain is included in the navigation model with “Notes sync”.
- `src/App.tsx:79-80`: `isActiveWorkspace` allows Brain.
- `src/App.tsx:895-897`: active Brain workspace renders `BrainWorkspace`. Verified direct routing.

### `src/App.css`

- `src/App.css:1029-1069` and `src/App.css:1127-1157`: Brain styles exist. There are duplicate/override blocks; not broken, but maintenance is less clean because two separated sections define the same selectors.
- `src/App.css:1051-1055`: note rows use two-column grid and truncate long note/conflict snippets. Usable but may hide important conflict text.
- `src/App.css:1065-1069`, `src/App.css:1155-1157`: responsive handling exists for narrower widths. OK.

### `src-tauri/capabilities/default.json`

- `src-tauri/capabilities/default.json:6-11`: only core/opener/dialog/notification permissions are listed. No extra Tauri plugin capability is needed for the custom Rust commands themselves in this file as reviewed; Apple Notes access is governed by macOS automation permissions and runtime `osascript` behavior.

### `src/scaffold.test.ts`

- `src/scaffold.test.ts:31-55`: verifies Brain shell/client/type strings exist but not behavior.
- `src/scaffold.test.ts:36-49`: confirms command name strings are present in frontend/backend, but this is brittle and can pass with nonfunctional code.
- No frontend behavior test renders `BrainWorkspace` with mocked Tauri invocations.

## Functionality proof checked

- Read from disk: `src-tauri/src/lib.rs`, `src/brain/types.ts`, `src/brain/brainClient.ts`, `src/brain/BrainWorkspace.tsx`, `src/App.tsx`, `src/App.css`, `src/scaffold.test.ts`, `src-tauri/capabilities/default.json`.
- Verified Rust serde camelCase model compatibility against TypeScript types by reading struct/type definitions.
- Verified Tauri command wrappers at `src-tauri/src/lib.rs:5119-5159` and invoke-handler registration at `src-tauri/src/lib.rs:5364-5372` against frontend invoke names in `src/brain/brainClient.ts:4-30`.
- Verified Apple Notes JXA scripts use JSON literal escaping for account/folder input and that sync reads notes without writing/deleting; only explicit folder creation writes to Apple Notes.
- Verified sync merge code normalizes body before hashing/extraction and marks missing notes instead of deleting them.
- Verified extraction/clarification paths are local and do not invoke Hermes.
- Ran targeted Rust tests: `cargo test brain_ --lib` from `/Users/ziadnasreldin/Zoid/src-tauri`. Result: 9 passed, 0 failed, 0 ignored, 58 filtered out. Warning: unused `prompt_with_enabled_profile_context`.
- Did not run the ignored real Apple Notes E2E because it mutates macOS Notes by design and requires automation permission; this remains unverified live behavior.

## Non-blocking improvements

- Split Apple Notes Brain backend into a dedicated Rust module instead of embedding it in a 7,700-line `lib.rs`.
- Replace string-backed Rust statuses/modes with enums plus `#[serde(rename_all = "camelCase")]` to align with TypeScript unions and prevent invalid values.
- Add atomic JSON store writes using write-to-temp + rename, and consider file locking if multiple windows/processes can sync.
- Add UI conflict resolution actions or explicitly label conflict panel as read-only diagnostic.
- Add a “last sync result” summary showing per-source imported/updated/missing/conflict/error counts.
- Type `brainClient.linkAppleNotesFolder` `syncMode` as `BrainSyncMode`.
- Deduplicate the two Brain CSS blocks into one maintainable section.
