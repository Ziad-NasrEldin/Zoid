# Feature Handoff: Polish Hermes sessions rail controls

## Original request

User said the restored Zoid 25 sessions rail was not good because:

- the archive button was huge,
- sessions should be renameable by double-click or right-click,
- text below session name should show the repository the session is working in,
- compact rail icons were messed up and should use one simple outer box instead of two boxes.

## Implementation summary

- Changed expanded rail archive action from a large text button to a narrow icon-only control with accessible label/title.
- Added inline session rename mode:
  - double-click a session row to rename,
  - right-click a session row to rename,
  - Enter commits,
  - Escape cancels,
  - blur commits non-empty text.
- Added optional `linkedRepositoryId` to `HermesChatSession`.
- Repository selection now applies to the active session and sends Hermes prompts through that session's selected repository.
- Session row metadata now shows the repository label (`repo name · branch`) or `Unlinked repository` instead of message count/index noise.
- Restore/archive paths preserve `linkedRepositoryId`.
- Compact rail icons now have only one visible outer box: the outer session button keeps the border and the inner icon has no border/background in compact mode.
- Bumped compact-state storage key to avoid stale minimized/native state hiding the expanded polished rail by default.
- Updated scaffold regression checks for rename, repository metadata, compact archive controls, and one-box compact icons.

## Changed files

- `src/agents/AgentsHermesScreen.tsx`
- `src/App.tsx`
- `src/App.css`
- `src/scaffold.test.ts`

## Verification run by dev

- `npm run test:frontend`: PASS
- `npm run build`: PASS
- `npm run test:rust`: PASS, 9 tests passed
- `npm run tauri:build`: PASS
- `git diff --check -- src/App.tsx src/agents/AgentsHermesScreen.tsx src/App.css src/scaffold.test.ts`: PASS
- Browser/DOM smoke on `http://127.0.0.1:1420/`:
  - double-click opened rename input,
  - right-click/contextmenu path opened rename input,
  - session row meta showed repository text,
  - archive control had no visible text,
  - compact mode used outer button border with inner icon border `none`.
- Reinstalled and relaunched `/Applications/Zoid 25.app`: PASS, process `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- Native screenshot `/tmp/zoid25-sessions-rail-polished-final.png`: PASS, expanded sessions rail visible with small icon-only archive controls and repository labels under session names.

## Reviewer focus

- Confirm no huge archive button remains in expanded rail.
- Confirm rename interaction is implemented without nesting an input inside a button.
- Confirm per-session repository metadata displays under session names and is used for Hermes send workdir.
- Confirm compact mode has only one visible icon box.
- Keep review scoped; repo has pre-existing unrelated dirty work.
