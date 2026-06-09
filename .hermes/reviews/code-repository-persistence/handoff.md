# Feature Handoff: Code repository persistence

## Original request

"in the code page in zoid 25, the repositories that were scanned do not persist throughout sessions, please fix"

## Implementation summary

- Root cause: Code repositories were held only in React component state (`useState([])`), so scanned/cloned repository records and the selected linked repository were lost when the Tauri app session restarted.
- Added localStorage-backed initializers for the Code repository list and linked repository selection.
- Added a repository shape guard so corrupt/non-array storage data falls back safely instead of crashing app startup.
- Added persistence effects for repository list and linked repository selection.
- Extended the scaffold regression test to require repository persistence keys, initializers, and storage writes.

## Changed files

- `src/App.tsx`: initializes repositories and linked repository selection from localStorage; persists changes back to localStorage; validates stored repository records.
- `src/scaffold.test.ts`: adds regression checks for scanned repository/session persistence.

## Scope Boundary / Dirty Working Tree Handling

Intended fix files only:

- `src/App.tsx`
- `src/scaffold.test.ts`

The repo already had broader dirty/untracked Zoid Code/Hermes work before this focused persistence fix, including `src-tauri/src/lib.rs`, `src/App.css`, `src/agents/*`, `src/code/`, and existing `.hermes/reviews/*` artifacts. I did not revert/stash/clean unrelated work.

Review the isolated fix with:

```bash
git diff -- src/App.tsx src/scaffold.test.ts
```

## How to test

- Run `npm run test:frontend` to verify the regression guard.
- Run `npm run build` to verify TypeScript/build.
- Run `npm run test:rust` to ensure native repository commands still pass.
- Run `npm run tauri:build`, copy the new bundle to `/Applications/Zoid 25.app`, relaunch, and confirm the installed app starts from `/Applications`.

## Tests run

- `npm run test:frontend`: PASS after implementation. RED first failed on missing `REPOSITORIES_STORAGE_KEY`.
- `npm run build`: PASS.
- `npm run test:rust`: PASS, 9 tests passed.
- `npm run tauri:build`: PASS, built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Installed app replacement/relaunch: PASS, running process `/Applications/Zoid 25.app/Contents/MacOS/zoid` PID observed.
- Screenshot: `/tmp/zoid25-code-persistence.png` captured installed app after relaunch. It showed the installed app running; current remembered workspace was Agents, with Code navigation visible.

## Git info

- Branch: `main`
- Commit SHA: not committed
- Diff base: working tree, scoped diff command above

## Frontend/backend/database notes

- Frontend: persistence lives in `src/App.tsx` localStorage keys `zoid25:code-repositories` and `zoid25:linked-repository-id`.
- Backend: no backend/database changes for this fix.
- Tauri/localStorage: relies on the installed Tauri WebView persisted localStorage, matching the existing last-workspace persistence pattern.

## Reviewer focus areas

- Confirm repository persistence is scoped and safe in the already-dirty repo.
- Confirm corrupt localStorage does not crash startup.
- Confirm this fix preserves the existing Code and Agents repository-linking surfaces.

## Fix cycle notes

Initial handoff; critique pending.
