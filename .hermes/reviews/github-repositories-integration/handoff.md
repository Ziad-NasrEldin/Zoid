# Feature critique handoff: GitHub Repositories integration

## Feature slug
`github-repositories-integration`

## User request
Implement Zoid 25 Code module GitHub repository integration:
- scan a specific folder for GitHub/Git repositories and auto-detect them,
- clone a repository from a GitHub repository link,
- show added repositories in a list,
- view basic details for each repository,
- replace the Agents chat "Link repository" folder-location input with a dropdown sourced from the managed repository list.

## Changed files relevant to this feature
- `src/code/types.ts`
- `src/code/repositoryClient.ts`
- `src/code/CodeWorkspace.tsx`
- `src/App.tsx`
- `src/agents/AgentsHermesScreen.tsx`
- `src/App.css`
- `src/scaffold.test.ts`
- `src-tauri/src/lib.rs`

## Implementation notes
- Added React Code workspace for GitHub Repositories integration:
  - scan folder input/button,
  - clone GitHub URL + destination root inputs/button,
  - repository list cards showing name/path/branch/remote/latest commit/clean vs dirty/source,
  - "Use for Agents" action that selects a managed repository.
- Persisted managed repositories, linked repository selection, and last active workspace in localStorage so scanned/cloned repos are available after restart.
- Added Tauri frontend client wrappers:
  - `scan_github_repositories`
  - `clone_github_repository`
- Added Rust/Tauri commands and helpers:
  - recursively scan directories up to limited depth for `.git`, skipping heavy folders,
  - read repo details using `git -C`: remote URL, branch, dirty status, latest commit,
  - validate GitHub URL shape and clone into destination root,
  - pass selected repo path into Hermes CLI `current_dir` for agent prompts.
- Replaced agent manual folder input with a select/dropdown. Dropdown is disabled until repositories exist and displays managed repo options.

## Known context / caution
- The working tree already contains other uncommitted Zoid changes and review folders unrelated or adjacent to this request (sessions UI, composer sizing, last-workspace persistence, previous review folders). Review this feature in context but separate Required fixes for this feature from unrelated dirty-tree items.
- App was rebuilt and installed to `/Applications/Zoid 25.app`; active workspace was set to Code for visual verification via WebKit localStorage.

## Verification already run
- `npm run test:frontend` — passed.
- `npm run test:rust` — passed, 9 Rust tests.
- `npm run test` — passed.
- `npm run build` — passed.
- `npm run tauri:build` — passed, produced `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Installed/reopened `/Applications/Zoid 25.app`; screenshot `/tmp/zoid-code-workspace.png` shows Code workspace with Scan folder and Clone repo controls.

## Review focus
1. Does the repo scan/clone implementation meet the requested functionality without unsafe or surprising filesystem behavior?
2. Does the Code workspace surface enough basic repo details and clearly support repository selection for Agents?
3. Does the Agents dropdown correctly replace the previous folder-location input and pass the selected path to the Hermes send path?
4. Are edge cases handled well enough for v1: missing folder, non-Git folder, duplicate scan results, failed clone, dirty repo, no branch/no remote?
5. Are there Required fixes before this feature can be considered complete?
