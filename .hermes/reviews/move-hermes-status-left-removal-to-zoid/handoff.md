# Feature Handoff: Move Hermes status-left removal to Zoid 25

## Original request

"ok please delete the edit of the hermes desktop and apply it to zoid 25"

Context: the previous request was to remove the oversized/useless element on the left of the Hermes status. The change was accidentally made/checked in Hermes Desktop context; it needed to be removed from Hermes Desktop and applied/verified in Zoid 25.

## Implementation summary

- Reverted the mistaken Hermes Desktop chat/session-rail/composer files back to their repository baseline:
  - `apps/desktop/src/app/chat/index.tsx`
  - `apps/desktop/src/app/chat/composer/index.tsx`
  - `apps/desktop/src/store/layout.ts`
  - `apps/desktop/src/styles.css`
- Verified those Hermes Desktop files no longer show as modified.
- In Zoid 25, the Agents > Hermes topbar now removes the oversized left title block (`AGENTS / HERMES TERMINAL` + large `Hermes Agent`) and reclaims that space for the status/repository controls.
- Rebuilt and reinstalled `/Applications/Zoid 25.app` from the current Zoid bundle, relaunched it, and screenshot-verified the native Agents page.

## Changed files

Zoid 25 source currently contains the intended UI state in:

- `src/agents/AgentsHermesScreen.tsx`: topbar uses `hermes-topbar hermes-topbar--status-only`; no left title block is rendered.
- `src/App.css`: removed `section-kicker` and large `.hermes-topbar h2` styles; topbar is compact and right/status-oriented.
- `src/scaffold.test.ts`: asserts the oversized left title block stays absent and the status-only topbar remains.

Hermes Desktop cleanup was performed by restoring the mistaken scoped files; unrelated existing dirty Hermes Desktop/TUI files were left untouched.

## How to test

From `/Users/ziadnasreldin/Zoid`:

- `npm run test:frontend`
- `npm run build`
- `npm run tauri:build`
- Replace `/Applications/Zoid 25.app` with `src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Launch `/Applications/Zoid 25.app`, click `Agents`, and verify:
  - the topbar no longer shows `AGENTS / HERMES TERMINAL` or the large `Hermes Agent` title;
  - the topbar shows only the compact Hermes CLI status and repository-link controls;
  - the message list/composer/stats remain visible.

## Tests run

- Hermes Desktop scoped restore check: PASS — `git status --short -- apps/desktop/src/app/chat/index.tsx apps/desktop/src/app/chat/composer/index.tsx apps/desktop/src/store/layout.ts apps/desktop/src/styles.css` returned no modified files.
- `npm run test:frontend`: PASS.
- `npm run build`: PASS (`tsc && vite build`; 37 modules transformed).
- `npm run tauri:build`: PASS; bundle produced at `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Installed app refresh: PASS — copied bundle to `/Applications/Zoid 25.app` and relaunched.
- Native process check: PASS — running process `/Applications/Zoid 25.app/Contents/MacOS/zoid` observed.
- Native screenshot check: PASS — after clicking Agents, screenshot `/tmp/zoid25-agents-click-coordinate.png` shows the oversized `AGENTS / HERMES TERMINAL` / `Hermes Agent` topbar block absent; topbar contains compact `HERMES CLI ONLINE` and repository-link controls.

## Git info

- Zoid branch: `main`.
- Current Zoid HEAD: `424be61 fix: make desktop shell responsive`.
- Hermes Desktop repo still has unrelated dirty files outside the restored scoped chat/session/composer files; they were intentionally not touched.

## Frontend/backend/database notes

- Frontend/UI only.
- Backend/native command bridge not changed.
- Database not changed.

## Reviewer focus areas

- Confirm the wrong Hermes Desktop files were restored and no longer carry the mistaken edit.
- Confirm Zoid 25 active source/native installed app shows the status-only Agents topbar.
- Confirm the cleanup did not remove the actual Hermes CLI status or repository-link controls.

## Fix cycle notes

Initial review request.
