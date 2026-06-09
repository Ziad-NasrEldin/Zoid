# Feature Handoff: Agentation always-on rebuild fix

## Original request

"can you run agentation and fix it in zoid25 so whenever we rebuild the app its always there, at least fix it for now till szoid 25 is production ready"

## Implementation summary

- Added the `agentation` package to the active Zoid 25 app.
- Mounted `<Agentation />` in `src/main.tsx` unconditionally so it is present in rebuilt packaged/production Tauri bundles, not only Vite dev mode.
- Rebuilt, reinstalled, and relaunched `/Applications/Zoid 25.app`.
- Verified the production bundle contains Agentation code and the installed app screenshot shows the floating Agentation widget.
- This is intentionally temporary until Zoid 25 is production ready; the toolbar is not dev-gated in this scoped fix.

## Changed files

- `src/main.tsx`: imports and renders `Agentation` next to the app root.
- `package.json`: adds `agentation` dependency.
- `package-lock.json`: adds/resolves `agentation` package.

## How to test

- Run `npm run build` and confirm `dist/assets/*` contains Agentation strings.
- Run `npm run tauri:build`.
- Replace `/Applications/Zoid 25.app` with the fresh bundle and relaunch it.
- Confirm the running process is `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- Screenshot-check that the floating Agentation widget appears in the installed app.

## Tests run

- `npm install agentation`: PASS, added 1 package, 0 vulnerabilities.
- `npm run test:frontend`: PASS.
- `npm run build`: PASS, Vite built production assets with normal chunk-size warning.
- `grep -R "Output Detail\|Agentation\|agentation\|MCP" dist/assets dist/index.html`: PASS, production bundle contains Agentation strings.
- `npm run test:rust`: PASS, 9 Rust tests passed.
- `npm run tauri:build`: PASS, app bundle created at `src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Reinstall/relaunch `/Applications/Zoid 25.app`: PASS, running PID verified at `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- Screenshot `/tmp/zoid25-agentation.png`: PASS, installed app shows the floating Agentation widget at bottom-right.

## Git info

- Branch: main
- Commit SHA: not committed.
- Diff base: working tree.

## Frontend/backend/database notes

- Frontend: React entry point only.
- Backend: none.
- Database: none.

## Reviewer focus areas

- User explicitly requested a temporary always-present Agentation fix for rebuilds; this intentionally differs from the earlier dev-only implementation.
- Confirm the toolbar is present in packaged rebuilt app and the change does not touch unrelated app flows.
- Repo has substantial pre-existing dirty/untracked work; keep review scoped to the Agentation parts of `src/main.tsx`, `package.json`, and `package-lock.json`.
- `@tauri-apps/plugin-dialog` is pre-existing dirty work from the Code repository folder-picker slice, not part of this Agentation fix. It is imported by `src/code/repositoryClient.ts` and covered by `src/scaffold.test.ts`; removing it here would break unrelated in-flight Zoid work. The Agentation-specific package diff is only `agentation` plus its lockfile entry.

## Fix cycle notes

Initial review returned REQUEST_CHANGES asking to remove or justify `@tauri-apps/plugin-dialog`. It is justified as pre-existing unrelated dirty work required by the Code folder-picker slice; no source changes were needed for this Agentation fix cycle. Re-review should focus on whether the Agentation-specific diff is scoped and whether preserving pre-existing `@tauri-apps/plugin-dialog` is acceptable.
