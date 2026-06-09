# GitHub Repositories Integration Implementation Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task. After implementation, run the mandatory feature-critique-workflow gate and do not mark complete until the critique verdict is APPROVED.

**Goal:** Add a Code module repository manager that can scan a selected folder for Git repositories, clone a repository URL, list added repositories with basic details, and let Agent chat link to one of those repositories via dropdown instead of manual folder path input.

**Architecture:** Keep repository detection and cloning in the Tauri Rust backend because filesystem traversal and `git` execution must stay local/native. Expose typed Tauri commands through a frontend repository client and share repository state between the Code workspace and Agents workspace at the app level so the agent dropdown always reflects the same repo list.

**Tech Stack:** React + TypeScript, Vite, Tauri commands in Rust, local `git` CLI, existing Hermes CLI bridge.

---

## Current Context / Assumptions

- Project root: `/Users/ziadnasreldin/Zoid`.
- App default workspace is currently `Code`, but `src/App.tsx` renders an empty Code page: `<section aria-label="Code workspace" className="empty-code-workspace" />`.
- Agent chat repository linking currently lives in `src/agents/AgentsHermesScreen.tsx` as a manual text input using `repositoryDraft`, `linkedRepository`, and `handleLinkRepository()`.
- Backend already accepts `linked_repository` in `src-tauri/src/lib.rs` and applies it as `Command::current_dir(...)` for Hermes CLI prompts.
- Existing smoke test is `src/scaffold.test.ts`; Rust tests live inside `src-tauri/src/lib.rs`.
- Scope is local Git repository management only, not GitHub API OAuth/account browsing.

## Product Requirements

1. **Scan folder for repositories**
   - User provides/selects a parent folder path.
   - Backend recursively or shallowly detects folders containing `.git`.
   - Detected repos are added to the app repository list without duplicates.

2. **Clone repo from link**
   - User can paste a GitHub repository link.
   - Backend runs `git clone <url>` into a configured local repositories root.
   - Cloned repo is added to the list.

3. **Repository list in Code module**
   - Code workspace shows all added/detected/cloned repos.
   - Each repo displays basic details: name, absolute path, remote URL if available, current branch, clean/dirty status, latest commit short hash/message if available.

4. **Agent session linking**
   - The existing “Link repository” control above the agent chat must become a dropdown/select populated from the app repository list.
   - Selecting a repo links its path to the Hermes CLI session workdir.
   - The footer/stats should show the selected repo name/path.

## Non-Goals / Boundaries

- No GitHub OAuth.
- No GitHub issue/PR syncing.
- No credential storage.
- No automatic background scan of the whole home directory.
- No deleting local repositories.
- No replacing the existing Hermes CLI workdir behavior; only replace the manual path input UX.

---

## Proposed Data Model

Create shared frontend type:

```ts
export type CodeRepository = {
  id: string; // stable path-derived id from backend
  name: string;
  path: string;
  remoteUrl?: string;
  branch?: string;
  dirty: boolean;
  latestCommit?: {
    hash: string;
    message: string;
  };
  addedAt: string;
  source: "scanned" | "cloned";
};
```

Create matching Rust structs with `#[serde(rename_all = "camelCase")]`.

Backend repository detail collection should use safe `git -C <repo> ...` commands:

```bash
git -C <path> remote get-url origin
git -C <path> branch --show-current
git -C <path> status --porcelain
git -C <path> log -1 --pretty=format:%h%x00%s
```

---

## Files Likely to Change

- Modify: `src/App.tsx`
- Modify: `src/App.css`
- Modify: `src/scaffold.test.ts`
- Modify: `src/agents/AgentsHermesScreen.tsx`
- Modify: `src/agents/hermesClient.ts` only if linked repo type changes are needed
- Modify: `src/agents/types.ts` or create shared repo type elsewhere
- Create: `src/code/CodeWorkspace.tsx`
- Create: `src/code/repositoryClient.ts`
- Create: `src/code/types.ts`
- Modify: `src-tauri/src/lib.rs`

Optional if code grows:
- Create: `src-tauri/src/repositories.rs` and re-export commands in `lib.rs`.

---

## Step-by-Step Plan

### Task 1: Add shared repository frontend types

**Objective:** Define a typed contract for repository records used by Code and Agents screens.

**Files:**
- Create: `src/code/types.ts`

**Steps:**
1. Create `CodeRepository` and `RepositorySource` types.
2. Keep fields camelCase to match Tauri serialization.
3. Do not add UI yet.

**Verification:**
- Run: `npm run build`
- Expected: TypeScript compiles or fails only because backend/client commands are not added yet if imported prematurely.

---

### Task 2: Add Rust repository detection helpers

**Objective:** Backend can identify Git repos and collect basic details for a given repo path.

**Files:**
- Modify: `src-tauri/src/lib.rs` or create `src-tauri/src/repositories.rs`

**Implementation notes:**
- Add `CodeRepository`, `LatestCommit`, and helper functions:
  - `is_git_repository(path: &Path) -> bool`
  - `repository_id(path: &Path) -> String`
  - `run_git(path, args) -> Option<String>`
  - `read_repository_details(path, source) -> Result<CodeRepository, String>`
- `is_git_repository` should accept both `.git` directory and `.git` file for worktrees/submodules.
- Dirty status is true when `git status --porcelain` output is non-empty.
- Missing remote/branch/latest commit should not fail the whole repo record.

**Tests:**
- Add Rust unit tests for:
  - detecting a temp folder with `.git` directory;
  - rejecting folders without `.git`;
  - stable id generation for the same path.

**Verification:**
- Run: `npm run test:rust`
- Expected: Rust tests pass.

---

### Task 3: Add folder scan Tauri command

**Objective:** Scan a selected parent folder and return detected Git repositories.

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Command:**
```rust
#[tauri::command]
pub async fn scan_github_repositories(folder: String) -> Result<Vec<CodeRepository>, String>
```

**Behavior:**
- Validate folder exists and is a directory.
- Traverse safely, skipping huge/noisy folders:
  - `.git`
  - `node_modules`
  - `target`
  - `dist`
  - `.next`
  - `Library`
- Recommended default depth: 3 levels from selected folder.
- If the selected folder itself is a repo, include it.
- Deduplicate by canonical path.
- Return sorted by repo name then path.

**Tests:**
- Temp parent folder with two child repos and one normal folder.
- Nested repo within depth should be returned.
- Nonexistent folder returns useful error.

**Verification:**
- Run: `npm run test:rust`
- Expected: scanner tests pass.

---

### Task 4: Add clone Tauri command

**Objective:** Clone a repo URL into a local root and return the cloned repository details.

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Command:**
```rust
#[tauri::command]
pub async fn clone_github_repository(repo_url: String, destination_root: String) -> Result<CodeRepository, String>
```

**Behavior:**
- Accept `https://github.com/org/repo`, `https://github.com/org/repo.git`, and `git@github.com:org/repo.git`.
- Validate the URL is GitHub-shaped for this feature.
- Derive destination folder name from repo slug.
- Create destination root if missing.
- Reject if destination folder already exists and is non-empty.
- Run `git clone <repo_url> <destination>` with timeout.
- Return `read_repository_details(destination, "cloned")`.

**Tests:**
- Unit test URL-to-folder-name parsing.
- Unit test invalid URL rejection.
- Avoid network clone in unit tests unless using a local bare repo fixture; use local fixture if practical.

**Verification:**
- Run: `npm run test:rust`
- Optional manual later during implementation: clone a small public GitHub repo into `/tmp/zoid-repo-clone-test` and verify it appears.

---

### Task 5: Register backend commands

**Objective:** Frontend can invoke repository scan and clone commands.

**Files:**
- Modify: `src-tauri/src/lib.rs`

**Steps:**
1. Add `scan_github_repositories` and `clone_github_repository` to `tauri::generate_handler![...]`.
2. Keep existing `check_hermes_cli` and `send_hermes_cli_message` commands unchanged.

**Verification:**
- Run: `npm run test:rust`
- Run: `npm run build`

---

### Task 6: Add frontend repository client

**Objective:** Provide typed functions for Code UI to call Tauri commands.

**Files:**
- Create: `src/code/repositoryClient.ts`

**Functions:**
```ts
import { invoke } from "@tauri-apps/api/core";
import type { CodeRepository } from "./types";

export function scanGithubRepositories(folder: string): Promise<CodeRepository[]> {
  return invoke<CodeRepository[]>("scan_github_repositories", { folder });
}

export function cloneGithubRepository(repoUrl: string, destinationRoot: string): Promise<CodeRepository> {
  return invoke<CodeRepository>("clone_github_repository", { repoUrl, destinationRoot });
}
```

**Verification:**
- Run: `npm run build`

---

### Task 7: Lift repository state into `App.tsx`

**Objective:** Make the repository list available to both Code and Agents screens.

**Files:**
- Modify: `src/App.tsx`
- Modify: `src/agents/AgentsHermesScreen.tsx`

**Steps:**
1. Add `const [repositories, setRepositories] = useState<CodeRepository[]>([])` in `App`.
2. Add `const [linkedRepositoryId, setLinkedRepositoryId] = useState<string>("none")` in `App`.
3. Render `<CodeWorkspace repositories={repositories} onRepositoriesChange={setRepositories} linkedRepositoryId={linkedRepositoryId} onLinkedRepositoryIdChange={setLinkedRepositoryId} />`.
4. Render `<AgentsHermesScreen repositories={repositories} linkedRepositoryId={linkedRepositoryId} onLinkedRepositoryIdChange={setLinkedRepositoryId} />`.
5. In Agents screen, compute selected repo by id and pass selected repo path to `sendHermesCliMessage`.

**Verification:**
- Run: `npm run build`
- Expected: TypeScript catches all prop wiring gaps.

---

### Task 8: Build Code workspace repository UI

**Objective:** Replace empty Code workspace with repository management UI.

**Files:**
- Create: `src/code/CodeWorkspace.tsx`
- Modify: `src/App.tsx`
- Modify: `src/App.css`

**UI sections:**
1. **Scan folder panel**
   - Input: folder path.
   - Button: “Scan folder”.
   - Status/error text.

2. **Clone repository panel**
   - Input: GitHub repo URL.
   - Input: destination root, default suggested path like `/Users/ziadnasreldin/Documents/GitHub` or blank placeholder.
   - Button: “Clone repo”.
   - Status/error text.

3. **Repository list**
   - Empty state when no repos added.
   - Card/table rows for each repo with:
     - name;
     - path;
     - remote URL;
     - branch;
     - dirty/clean;
     - latest commit hash/message;
     - source scanned/cloned;
     - button/select action: “Use for Agents”.

**State behavior:**
- Merge new scan results into existing repositories by `id`/`path`.
- If a repo is already present, update details rather than duplicate.
- On clone success, add the cloned repo and optionally select it.

**Verification:**
- Run: `npm run build`
- Run: `npm run test:frontend`

---

### Task 9: Replace Agent manual path input with repository dropdown

**Objective:** Agent chat links to repositories from the managed repo list only.

**Files:**
- Modify: `src/agents/AgentsHermesScreen.tsx`
- Modify: `src/App.css`

**Changes:**
- Remove `repositoryDraft` state and text input.
- Remove manual `handleLinkRepository()` path flow.
- Add `<select>` with:
  - `value={linkedRepositoryId}`
  - option `none` = “Unlinked”
  - one option per repository, label like `name — branch — path`
- Selection updates app-level `linkedRepositoryId`.
- `sendHermesCliMessage(..., selectedRepository?.path)` remains the only path sent to backend.
- Show linked repo detail in the topbar and footer.
- If no repositories exist, show a disabled dropdown/help text: “Add repositories in Code first.”

**Verification:**
- Run: `npm run build`
- Run: `npm run test:frontend`

---

### Task 10: Update scaffold tests for repository integration

**Objective:** Ensure future changes do not regress the Code module and agent dropdown.

**Files:**
- Modify: `src/scaffold.test.ts`

**Assertions to add/update:**
- `App.tsx` imports and renders `CodeWorkspace` instead of empty Code workspace.
- Code workspace includes scan and clone controls.
- Frontend client invokes `scan_github_repositories` and `clone_github_repository`.
- Agents screen no longer contains `linked-repository-input` or manual repository text input.
- Agents screen contains a repository `<select>` and uses selected repo path in `sendHermesCliMessage`.
- Backend registers both repository Tauri commands.

**Verification:**
- Run: `npm run test:frontend`
- Expected: passes.

---

### Task 11: Full local verification

**Objective:** Prove the implementation works end-to-end locally before review.

**Commands:**
```bash
npm run test
npm run build
```

**Manual verification checklist:**
1. Start app with `npm run tauri:dev` or existing local app workflow.
2. Open Code workspace.
3. Scan `/Users/ziadnasreldin/Zoid` or `/Users/ziadnasreldin/Documents/GitHub`.
4. Confirm detected repos appear once with basic details.
5. Clone a public GitHub test repo into a temp folder.
6. Confirm cloned repo appears in list.
7. Open Agents workspace.
8. Confirm repository link control is a dropdown, not a folder input.
9. Select a repo.
10. Send a simple Hermes prompt that depends on cwd, e.g. “what repo am I in? only answer package name/path if visible”.
11. Confirm backend sends selected repo path as Hermes CLI current working directory.

---

### Task 12: Mandatory feature critique gate

**Objective:** Satisfy the global software delivery rule before calling the feature complete.

**Files:**
- Create: `.hermes/reviews/github-repositories-integration/handoff.md`
- Create/update: `.hermes/reviews/github-repositories-integration/critique-report.md`

**Steps:**
1. Write handoff with scope, changed files, verification output, and known risks.
2. Trigger or wait for the separate critique-agent review.
3. Fix every Required fix.
4. Re-run tests/build/manual checks after fixes.
5. Re-review until verdict is `APPROVED`.

**Verification:**
- Critique report verdict is `APPROVED`.
- `npm run test` and `npm run build` pass after final fixes.

---

## Risks / Tradeoffs

- Recursive scanning can be slow if pointed at home directory; mitigate with depth limit and ignored folders.
- `git clone` requires network and credentials for private repos; private clone failures should show the raw safe error without storing credentials.
- Git worktrees and submodules can use `.git` files instead of directories; detection should support both.
- If repository state is only in React memory, the list disappears on app restart. This plan does not include persistence unless added as a follow-up. If persistence is required now, add a Tauri config/store file task before UI work.
- Folder picker UI is not included because current app uses plain path inputs. A native dialog can be added later via Tauri dialog plugin if installed/configured.

## Open Questions

1. Should repository list persist across app restarts? Recommended: yes, but implement after core scan/clone/dropdown works unless you want it in this first pass.
2. Should scan be shallow-only or recursive depth 3? Recommended: recursive depth 3 with ignored heavy folders.
3. Default clone destination: recommended `/Users/ziadnasreldin/Documents/GitHub` if it exists, otherwise require user input.

## Definition of Done

- Code workspace can scan a chosen folder and list detected Git repositories.
- Code workspace can clone a GitHub repo URL into a chosen destination and add it to the list.
- Repository cards show name, path, remote, branch, clean/dirty, and latest commit summary where available.
- Agents repository linker is a dropdown populated from the managed repository list.
- Sending an agent message uses the selected repository path as the Hermes CLI workdir.
- `npm run test` passes.
- `npm run build` passes.
- Manual local verification confirms scan, clone, list, dropdown, and Hermes workdir behavior.
- Feature critique workflow verdict is `APPROVED` after required fixes are resolved.
