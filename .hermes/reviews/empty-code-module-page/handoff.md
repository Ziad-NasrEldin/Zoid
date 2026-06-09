# Feature Handoff: Empty Code module/page

## Original request

"please clean all the code module/page, it should be empty"

## Implementation summary

- Removed the Code Workspace flow UI from the active Code page.
- Removed the Code Workspace flow import/render path from `src/App.tsx`.
- Deleted the imported Code Workspace module files from active `src/`:
  - `src/codeWorkspaceFlow.css`
  - `src/codeWorkspaceFlow.test.ts`
  - `src/codeWorkspaceFlow.ts`
  - `src/codeWorkspaceFlowView.tsx`
- The Code sidebar item still exists and remains the default active page, but its main content area is intentionally blank/empty.
- Existing Agents/Hermes page remains reachable by clicking `Agents`.
- Updated the frontend scaffold test and `package.json` test script so the empty Code page is the expected behavior.
- Rebuilt, reinstalled, and relaunched `/Applications/Zoid 25.app` for desktop testing.

## Changed files

- `src/App.tsx`: removed `CodeWorkspaceFlow` import and replaced Code render branch with `<section aria-label="Code workspace" className="empty-code-workspace" />`.
- `src/scaffold.test.ts`: changed assertions to require an empty Code page and reject old Code Workspace flow UI strings.
- `package.json`: removed `src/codeWorkspaceFlow.test.ts` from `test:frontend` because that module was deleted.
- Deleted active Code Workspace files listed above.

## How to test

1. Launch `/Applications/Zoid 25.app`.
2. Verify the `Code` navigation row is active by default.
3. Verify the main content area to the right of the sidebar is blank/empty.
4. Verify there is no `Code Workspace`, `One guided flow`, `Native local`, or `Browser preview` content visible.
5. Click `Agents` to confirm the Hermes screen still exists.

## Tests run

- `npm run test:frontend`: PASS — scaffold test passed.
- `npm run build`: PASS — TypeScript and Vite production build passed.
- `npm run test:rust`: PASS — 4 Rust tests passed.
- `npm run tauri:build`: PASS — Tauri app bundle built successfully.
- Native reinstall/relaunch: PASS — copied fresh bundle to `/Applications/Zoid 25.app`, launched process `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- Desktop screenshot verification: PASS — `/tmp/zoid25-code-empty.png` shows `Code` active and an empty white content area, with no Code Workspace flow content visible.

## Git info

- Branch: `main`
- Commit SHA: not committed in this step.

## Frontend/backend/database notes

- Frontend: Code page is now a deliberately empty section only.
- Backend/native: no native command/database changes were made for this Code page cleanup.
- Database: not applicable.

## Reviewer focus areas

- Confirm no old Code Workspace UI/module remains in active `src/`.
- Confirm the Code page is blank in the app shell while the sidebar still works.
- Confirm tests/build pass after deleting the Code module files.
- Confirm final desktop app was actually rebuilt/reinstalled/relaunched, not only browser-verified.

## Fix cycle notes

Initial review request after implementation and native desktop verification.
