# Code workspace repository-list scroll handoff

Scope: Follow-up to Page Feedback for `/` Code workspace. User clarified: “i cant even scroll in the repositories list, pls fix”. Previous change made list primary but removed direct list scrolling. This slice restores direct repository-list scrolling while keeping scan/clone secondary.

Changed:
- `src/App.css`
  - `.repository-list-panel` is now a large fixed work-surface region: `height: min(680px, calc(100vh - 230px)); min-height: 360px; overflow: hidden; display: flex; flex-direction: column;`.
  - `.repository-card-list` is now the scrollable list body: `flex: 1 1 auto; min-height: 0; overflow-x: hidden; overflow-y: auto; align-content: start;`.
  - Added styled WebKit scrollbar for `.repository-card-list` so the scroll affordance is visible and fits the sumi-e Code styling.
- `src/scaffold.test.ts`
  - Updated source guards to require the large scrollable repository-list body (`height: min(...)`, `overflow-y: auto`, and scrollbar rule), not the earlier visible-overflow assumption.
- Skill maintenance:
  - Updated `tauri-desktop-feature-development` reference `zoid-code-repository-list-priority-layout.md` with the clarified nuance: when many repositories exist, the main repository list should be directly scrollable, but large enough not to feel cramped.

Verification already run:
- `npm run build` passed.
- `npm run test:frontend` passed.
- Browser smoke with 24 seeded repositories:
  - `.repository-card-list` computed `overflowY: auto`.
  - `scrollHeight: 6304`, `clientHeight: 295` at the small browser viewport.
  - Programmatically setting `scrollTop = 99999` moved to `scrollTop: 6009`, proving the repository list scrolls directly.
  - `.code-repository-layout` first child remains `repository-list-panel`.

Review focus:
- Does the fix satisfy the follow-up complaint: user can directly scroll the repository list?
- Does the list remain the primary surface rather than reverting to tiny/cramped internal scrolling?
- Check responsive/narrow behavior and no regressions to scan/clone/default-branch controls.

Dirty tree note: repo has unrelated pre-existing modifications/untracked review artifacts outside this scope. Review only the files above for this slice.