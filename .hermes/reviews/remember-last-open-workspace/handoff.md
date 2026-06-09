# Feature Handoff: Remember last open workspace

## Original request

"can you make the default page i land on wwhen first opening zoid 25 is the last page i had open before closing it please"

## Implementation summary

- Zoid 25 now initializes the active workspace from `localStorage` key `zoid25:last-active-workspace`.
- Workspace changes between the currently implemented selectable pages (`Agents`, `Code`) are persisted immediately.
- Invalid/missing stored values fall back to the prior safe default: `Code`.
- This stores the last selected page before app close, so reopening lands on the same page.
- During verification, concurrent/in-flight Code/GitHub repository integration edits changed `src/App.tsx`, `src/scaffold.test.ts`, `src/agents/AgentsHermesScreen.tsx`, and `src/code/`. I kept this review scoped to the last-workspace persistence behavior, but also restored current build/test green by ensuring the Agents repository selector passes `selectedRepository?.path` to the Hermes send path.

## Changed files for this scoped feature

- `src/App.tsx`: adds active-workspace persistence helpers, initializes state from the saved workspace, and persists changes with `useEffect`.
- `src/scaffold.test.ts`: updates scaffold assertions to require localStorage persistence instead of hard-coded `Code` default.
- `src/agents/AgentsHermesScreen.tsx`: small compatibility fix for the concurrent Code integration so the current tree still builds/tests (`selectedRepository?.path` send path and type import cleanup).

## How to test

- `npm run build`
- `npm test`
- `npm run tauri:build`
- Native installed app smoke: replace `/Applications/Zoid 25.app` with the built bundle, open from `/Applications`, click `Agents`, quit, reopen, and verify the visible native workspace is `Hermes chat`.

## Tests run

- `npm run build`: PASS — TypeScript and Vite production build passed.
- `npm test`: PASS — frontend scaffold test and Rust tests passed (`9 passed`).
- `git diff --check`: PASS.
- `npm run tauri:build`: PASS — built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Installed native smoke: PASS — copied built app to `/Applications/Zoid 25.app`, relaunched PID `51690` from `/Applications/Zoid 25.app/Contents/MacOS/zoid`; after clicking `Agents`, quit/reopen returned AX workspace name `Hermes chat` and screenshot showed Agents active.

## Git info

- Branch: current working tree branch not changed by this task.
- Commit SHA, if committed: not committed.
- Diff base, if known: current working tree has unrelated/pre-existing dirty and untracked files outside this scoped feature.

## Frontend/backend/database notes

- Frontend routes/components: `src/App.tsx` active workspace state; installed Tauri WebView persistence via localStorage.
- Backend endpoints/services: none for last-workspace persistence.
- Database tables/migrations: none.

## Reviewer focus areas

- Confirm persistence is limited to valid implemented workspaces.
- Confirm invalid/missing localStorage values still land on `Code`.
- Confirm unsupported nav items are not accidentally persisted as active workspaces.
- Confirm current in-flight Code/GitHub repository integration changes do not undermine this persistence behavior.

## Fix cycle notes

Re-review requested after native installed verification and after stabilizing build/tests against concurrent in-flight Code/GitHub repository integration edits.
