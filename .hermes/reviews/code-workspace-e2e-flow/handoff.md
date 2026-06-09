# Feature Handoff: Code Workspace E2E flow visible in Zoid 25 desktop

## Original request

User reported the Zoid 25 desktop app was still not updated and attached a screenshot showing the old `Agents / Hermes Terminal` screen. The intended new change from a previous agent was the Code Workspace E2E flow, but it was not visible in the active app.

## Implementation summary

- Located the missing Code Workspace flow in `legacy/zoid-ai-os-ui-archive-2026-06-06/frontend/src/`.
- Imported the active flow model, React view, CSS, and regression test into the current active `src/` tree.
- Updated `App.tsx` so the sidebar can switch between `Agents` and `Code`, with `Code` as the default visible workspace so the user immediately sees the new change in the desktop app.
- Kept the existing Agents/Hermes screen reachable.
- Updated frontend test coverage to include the imported Code Workspace flow test.
- Rebuilt the Tauri app, replaced `/Applications/Zoid 25.app`, relaunched it, and screenshot-verified the native desktop app now shows `Code Workspace` / `One guided flow from repo to verified launch` with `Code` active.

## Changed files

- `src/codeWorkspaceFlow.ts`: imported Code Workspace flow state machine and routing logic from legacy archive.
- `src/codeWorkspaceFlowView.tsx`: imported guided Code Workspace UI from legacy archive.
- `src/codeWorkspaceFlow.css`: imported Code Workspace styling.
- `src/codeWorkspaceFlow.test.ts`: imported regression tests for the flow state machine.
- `src/App.tsx`: added active workspace state, clickable `Agents`/`Code` navigation, default `Code` workspace, and `CodeWorkspaceFlow` render path.
- `src/App.css`: made navigation clickable and included UI styling changes needed by concurrent Agents session-metrics/repository-link work.
- `src/scaffold.test.ts`: updated expectations for Code default, Code render path, and current Agents stats/repository controls.
- `src/agents/AgentsHermesScreen.tsx`: contains concurrent active changes for session metrics and repository linking that were present during this fix and included in verification.
- `package.json`: adds `src/codeWorkspaceFlow.test.ts` to `test:frontend`.

## How to test

1. From `/Users/ziadnasreldin/Zoid`, run:
   - `npm run test:frontend`
   - `npm run build`
   - `npm run test:rust`
   - `npm run tauri:build`
2. Replace and launch installed app:
   - `pkill -f '/Applications/Zoid 25.app/Contents/MacOS/zoid' || true`
   - `rm -rf '/Applications/Zoid 25.app'`
   - `cp -R 'src-tauri/target/release/bundle/macos/Zoid 25.app' '/Applications/Zoid 25.app'`
   - `open -a '/Applications/Zoid 25.app'`
3. Expected desktop result:
   - Sidebar `Code` row is active.
   - Main panel title shows `Code Workspace`.
   - Hero title shows `One guided flow from repo to verified launch`.
   - The old Hermes Agents screen remains reachable by clicking `Agents`.

## Tests run

- `npm run test:frontend`: PASS — scaffold and Code Workspace flow tests passed.
- `npm run build`: PASS — TypeScript and Vite production build completed.
- `npm run test:rust`: PASS — 2 Rust tests passed.
- `npm run tauri:build`: PASS — bundle produced at `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Native desktop relaunch: PASS — `/Applications/Zoid 25.app` updated and process launched from `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- Screenshot verification: PASS — `/tmp/zoid25-updated-code.png` shows Code Workspace active in the macOS desktop app.

## Git info

- Branch: `main`
- Current HEAD before commit: `51b5f49`
- Commit SHA: not committed in this step.
- Diff base: working tree changes from `51b5f49`.

## Frontend/backend/database notes

- Frontend routes/components: single-app shell; Code workspace is rendered in `App.tsx` using local React state rather than a router.
- Backend endpoints/services: no backend or Tauri command changes for Code flow import.
- Database tables/migrations: not applicable.
- Native side effects: package rebuilt and installed into `/Applications/Zoid 25.app` for user testing.

## Reviewer focus areas

- Ensure the imported legacy Code Workspace files are actually present in active source, not only legacy archive.
- Ensure default desktop view is Code Workspace and not the old Agents screen.
- Ensure sidebar switching between Code and Agents does not break the existing Agents bridge UI.
- Ensure tests cover the imported flow and active App wiring.
- Note that this is UI/state-machine integration only; real native Git/Finder/GitHub/deploy side effects are not implemented by this import and should not be represented as completed launch actions.

## Fix cycle notes

Initial handoff after implementation and native desktop verification.

### Fix cycle 1 after critique REQUEST_CHANGES

Critique required removing false native readiness/repo availability claims from the default Code workspace. Fixed `src/App.tsx` to pass `nativeMode="error"`, an explicit preview-only native error message, `repoCount={0}`, and no `actionStatus`, so the screen now says `Browser preview` and `Empty Code Workspace` rather than `Native local` / `1 repo profile available`.

Re-ran verification after this fix:

- `npm run test:frontend`: PASS — Code flow tests passed.
- `npm run build`: PASS — TypeScript and Vite build passed.
- `npm run test:rust`: PASS — 3 Rust tests passed.
- `npm run tauri:build`: PASS — packaged app rebuilt.
- Reinstalled `/Applications/Zoid 25.app`, relaunched it, and screenshot-verified `/tmp/zoid25-code-final.png` shows Code Workspace active with `Browser preview` status and `Empty Code Workspace` current step.
