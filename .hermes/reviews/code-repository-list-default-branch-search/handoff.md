# Feature Handoff: Code repository list default branch and search row

## Original request

Page Feedback: /
Viewport: 1920×1018

1. Search repositories button: "i want this button to be right beside the “repository list” and when clicked on it animates and uses the morphing animation to expand into a search field, but i dont it to open beneath it, all in one row of space, take care not to hit the repositories number on the right"
2. Branch dt: "this should be the default branch"
3. Branch dd: "and add a button on the right side to edit the default branch of this repository make sure to github integrate it correctly"
4. Latest commit dd: "i just want a date here not exact commit"

## Implementation summary

- Moved repository search into the repository-list title row immediately after the `Repository list` heading.
- Added a morph-width inline search control so the search button expands into a same-row search field instead of rendering beneath the heading.
- Kept the repository count as the right-aligned final row item so the search field cannot occupy the count slot.
- Added `defaultBranch` and latest commit `date` to repository metadata.
- Changed the card label from `Branch` to `Default branch` and render `defaultBranch` first, falling back to current branch only when no remote default is detected.
- Added a right-side `Edit` button in the default branch row.
- Added Tauri command `update_github_default_branch` that validates a GitHub origin remote and uses GitHub CLI `gh repo edit <owner/repo> --default-branch <branch>` to update the real GitHub repository default branch, then refreshes repository details.
- Changed `Latest commit` display to show only the date; the hash/message remain only in the title tooltip.

## Changed files

- `src/code/CodeWorkspace.tsx`: inline morphing repository search UI, default branch/date rendering, edit button and command wiring.
- `src/code/repositoryClient.ts`: added `updateGithubDefaultBranch` invoke wrapper.
- `src/code/types.ts`: added `defaultBranch` and latest commit `date` fields.
- `src/App.tsx`: accepts persisted repositories with optional `defaultBranch`.
- `src/App.css`: inline search morph layout and default-branch edit row styling.
- `src-tauri/src/lib.rs`: reads remote default branch and commit date; adds GitHub default branch update command using `gh repo edit`.

## How to test

- Run `npm run build`.
- Run `npm test`.
- Run `npm run tauri:build`.
- In browser preview at `http://127.0.0.1:1420/`, verify the repository list header has `Repository list`, search button/search field, and count all in one row.
- Seed a sample repository in `localStorage` or scan a repo, then verify card metadata displays `Default branch`, an `Edit` button on that row, and latest commit as a date only.
- For live GitHub default-branch editing, use a GitHub repo where `gh auth status` is valid and the authenticated user has admin permissions; clicking Edit should call `gh repo edit owner/repo --default-branch <branch>` through Tauri and refresh the row.

## Tests run

- `npm run build`: PASS. Vite built `dist`, with existing chunk-size warning only.
- `npm run test:rust`: PASS. 9 Rust tests passed.
- `npm run test:frontend`: PASS.
- `npm test`: PASS. Frontend scaffold test plus 9 Rust tests passed.
- `npm run tauri:build`: PASS. Built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Browser preview DOM probe: PASS. Open search row measured as `Repository list` h3 at left, `repository-search-morph--open` field width 420px beside it, and count at right; sample repo showed term `Default branch`, definition `mainEdit`, and latest commit definition `2026-06-07`.
- Installed app relaunch: PASS for process identity. Replaced `/Applications/Zoid 25.app`, launched it, and verified PID `41569 /Applications/Zoid 25.app/Contents/MacOS/zoid`.

## Git info

- Branch: main (working tree contains many unrelated existing dirty/untracked files; review should be scoped to files listed above and this request only).
- Commit SHA: not committed.
- Diff base: current working tree baseline is already dirty from prior Zoid work.

## Frontend/backend/database notes

- Frontend: Code workspace repository list/card only.
- Backend/native: Tauri command `update_github_default_branch(repositoryPath, defaultBranch)` shells out to `gh repo edit` and local `git` metadata commands.
- Database: none.

## Reviewer focus areas

- Search field must remain in the same title row and not collide with the right-side count at 1920px and narrower desktop widths.
- The visible Branch label should be gone/replaced by Default branch.
- Latest commit visible text must be only a date, not hash/message.
- Default branch edit must be a real GitHub integration, not local-only state.
- Check whether error/status feedback being stored but not rendered is a pre-existing issue or a blocker for this edit flow.

## Fix cycle notes

- Initial critique verdict: CHANGES_REQUIRED.
- Required fix addressed: `CodeWorkspace` now keeps and renders `statusMessage`/`errorMessage` in an accessible `code-workspace-feedback` notice with `role="status"` or `role="alert"`, so GitHub default-branch update failures from `gh repo edit` are visible to the user.
- Added matching CSS for normal/error workspace feedback.
- Re-ran `npm run build && npm test`: PASS.
