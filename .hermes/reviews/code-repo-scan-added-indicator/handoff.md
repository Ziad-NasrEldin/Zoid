# Handoff: Code repo scan added indicator

## User request
User reported there is no visual indication when scanning a folder adds repositories to the Code repository list. Add clear feedback when repos are added.

## Scope
Review only this focused change:
- `src/code/CodeWorkspace.tsx`
- `src/App.css`
- `src/scaffold.test.ts`

The repo has many unrelated dirty/untracked files from ongoing Zoid work; do not review or attribute unrelated changes.

## Implementation summary
- Added `repositoryScanFeedback` state with tone/label/message.
- Added `recentlyAddedRepositoryIds` state that clears after ~4.2s.
- `handleScanFolder` now:
  - immediately shows an info status while scanning.
  - compares detected repos against existing repo IDs.
  - shows success copy when new repos were added, info copy when scan found only existing/no repos, and error copy on scan failure.
  - stores newly added IDs so new cards get a temporary highlight class.
- Added visible scan feedback in two places:
  - directly inside the Scan folder panel beneath the scan button, so feedback is visible immediately where the user clicked.
  - inside the Repository list header area, so the list also shows the scan result.
- Added `.repository-card--just-added` animation/highlight for newly added repo cards.
- Converted `.repository-list-panel` to a vertical flex layout so optional feedback banners do not break the list area.
- Updated `src/scaffold.test.ts` source guards for the new scan feedback and just-added card classes.

## Verification performed
- `npm run build` — passed.
- `npm run test:frontend` — passed.
- Browser dev server `http://127.0.0.1:1420/` — HTTP 200.
- Browser mocked Tauri internals for `plugin:dialog|open` and `scan_github_repositories`, then clicked Choose folder and Scan selected folder.
- Browser snapshot confirmed:
  - selected folder value updated.
  - Scan panel displayed status: `REPOS ADDED` / `1 new repository added to the list: visual-scan-repo-2.`
  - Repository list displayed the same status and the new repo appeared.
- Visual screenshot inspection confirmed the green success indicator is clearly visible directly under the scan button.
- DOM style probe confirmed:
  - `.repo-action-feedback--success` visible with green background/inset accent.
  - `.repo-scan-feedback--success` visible in repository list.
- `npm run tauri:build` — passed.
- Replaced `/Applications/Zoid 25.app` with the built bundle and relaunched.
- Installed app running confirmed: `/Applications/Zoid 25.app/Contents/MacOS/zoid` PID 53083, window name `Zoid 25`.

## Notes
- Browser verification used mocked Tauri internals because normal browser preview has no native Tauri invoke/dialog bridge. Native bundle was built and relaunched afterward.
- The card highlight is intentionally temporary; the success banners remain after the scan result.
