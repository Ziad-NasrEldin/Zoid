# Feature Handoff: Code repository list search + status panel removal

## Original request

in zoid 25
## Page Feedback: /
**Viewport:** 1920×1018

### 1. repository list
**Location:** .zoid25-shell > .code-workspace-shell > .repository-list-panel > .repository-list-heading
**Source:** tauri://localhost/assets/index-CAVfDS49.js:1991:1102
**Classes:** repository-list-heading
**Position:** 443px, 566px (1442×51px)
**Context:** Repository list11 added
**Feedback:** add a search icon here that opens up a search field and all in this section/box 
this is a search feature for repositories

### 2. status
**Location:** #root > .zoid25-shell > .code-workspace-shell > .repo-status-panel
**Source:** tauri://localhost/assets/index-CAVfDS49.js:1991:1102
**Classes:** repo-status-panel
**Position:** 442px, 489px (1444×58px)
**Feedback:** remove this its useless
do these

## Implementation summary

- Added a repository-list search toggle button with Search/X icons in the repository list header.
- Opening the icon reveals an in-box search field directly inside `.repository-list-heading`.
- Repository cards are filtered by name, path, remote URL, branch, source, dirty/clean status, latest commit hash, and latest commit message.
- Header count changes to `x of y shown` while searching.
- Empty search state renders when no repositories match.
- Removed the separate `.repo-status-panel` from the Code workspace and reclaimed the row in `.code-workspace-shell`.
- Added scaffold guard coverage for repository search and status-panel removal.

## Changed files

- `src/code/CodeWorkspace.tsx`: search state/filtering, search toggle/input UI, filtered card rendering, removed repo status panel render.
- `src/App.css`: Code page grid now has no status row; repository list heading/search control styles; removed status panel CSS.
- `src/scaffold.test.ts`: required repository search strings and forbidden status panel strings.

## How to test

- Run `npm run test:frontend && npm run build`.
- Run `npm run tauri:build`.
- Install/relaunch `/Applications/Zoid 25.app` and open Code page.
- Expected: no separate repo status box between action panels and repository list; repository list header has search icon; clicking it opens a search input inside the repository list heading; typing filters repository cards.

## Tests run

- `npm run test:frontend && npm run build`: PASS. Vite chunk-size warning only.
- `npm run tauri:build`: PASS. Built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Reinstalled and relaunched `/Applications/Zoid 25.app`: PASS, process verified at `/Applications/Zoid 25.app/Contents/MacOS/zoid` (pid 32362).
- Screenshot capture attempted at `/tmp/zoid25-code-repo-search.png`; native window interaction was blocked/ambiguous by existing foreground/window state, so screenshot did not reliably show Code page.

## Git info

- Branch: not checked in this handoff.
- Commit SHA: not committed.
- Diff base: dirty working tree has many pre-existing/unrelated changes and untracked review folders; review should focus only the files listed above.

## Frontend/backend/database notes

- Frontend routes/components: `CodeWorkspace` only.
- Backend endpoints/services: none changed.
- Database tables/migrations: none changed.

## Reviewer focus areas

- Search icon/input is scoped to repository list box/header as requested.
- Filtering is useful for repositories and not just visual decoration.
- `.repo-status-panel` is not rendered and CSS no longer keeps its row.
- No regressions to scan/clone/use-for-agents behavior.

## Fix cycle notes

Initial review request.
