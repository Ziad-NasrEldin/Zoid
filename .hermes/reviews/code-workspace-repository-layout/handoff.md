# Code workspace repository layout handoff

Scope: Page Feedback for `/` Code workspace at tauri://localhost. User disliked that scan folder/clone repository controls took too much space while repository list was constrained and had its own scroll. Repository list should be main component; scan folder and clone are secondary controls.

Changed:
- `src/code/CodeWorkspace.tsx`
  - Moved Repository list before actions in DOM inside `.code-repository-layout`.
  - Wrapped scan/clone controls in a secondary `<aside className="repo-control-grid">`.
  - Added explicit modifier classes `repo-action-panel--scan` and `repo-action-panel--clone`.
- `src/App.css`
  - Code shell now uses `grid-template-rows: auto minmax(0, 1fr)`.
  - Header hero reduced substantially: smaller min/title/text/ink-mark/padding.
  - Added `.code-repository-layout` as list-first two-column layout: list gets `minmax(0, 1fr)`, secondary actions get `minmax(260px, 320px)`.
  - Scan/clone cards made denser: one-column side controls, lower padding/gaps/font/button sizes.
  - Repository list panel gets `min-height: min(620px, calc(100vh - 230px))` and card list remains `overflow: visible`; the page shell owns scrolling, not an internal repository-list scroller.
  - Responsive breakpoint collapses `.code-repository-layout` to one column.
- `src/scaffold.test.ts`
  - Added source guards for list-priority layout and compact action classes.

Verification already run:
- `npm run build` passed.
- `npm run test:frontend` passed.
- Browser smoke at `http://127.0.0.1:1420/` passed; accessibility tree shows Code workspace with Repository list before Repository actions.
- Browser geometry probe: `.repository-card-list`/`.repository-list-panel` overflowY is `visible`, `.code-workspace-shell` overflowY is `auto`, and `.code-repository-layout` first child is `repository-list-panel`.

Review focus:
- Does this actually address the UX complaint: list is primary, controls no longer dominate, and scrolling is page-level rather than trapped inside the repo list?
- Check responsive behavior and whether action sidebar is still too wide/tall at ~900px content width.
- Ensure no regression in scan/clone/default branch controls or accessibility labels.

Dirty tree note: repo has unrelated existing modifications/untracked review artifacts outside this scope. Review only the files above for this feature.