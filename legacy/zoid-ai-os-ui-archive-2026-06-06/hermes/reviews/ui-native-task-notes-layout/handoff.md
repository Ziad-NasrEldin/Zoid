# UI Native Task/Notes Layout Handoff

## Scope
User reported two messy Zoid AI OS UI pages from screenshots. The affected browser-preview pages were Tasks and Notes. Fixes must keep native backend truthfulness: browser preview must not simulate task/note records when Tauri invoke is unavailable.

## Changed files
- `src/taskWorkspace.tsx`
- `src/noteWorkspace.tsx`
- `src/taskBridgeIntegration.ts`
- `src/noteBridgeIntegration.ts`
- `src/App.css`
- `src/App.tsx` (only relevant local change for this scope: clearer `bridgeErrorReason`; file already contains other in-flight content-workspace changes)

## Implementation summary
- Reorganized Tasks and Notes native workspace headers into title/copy and action groups.
- Added unavailable-state panels for non-ready native bridge states.
- Browser preview now hides empty/misleading list/detail/editor/conflict/linked panels until the real native backend responds.
- Disabled create/scan actions in unavailable browser-preview state while keeping refresh available.
- Improved backend error copy for browser/Tauri invoke failures so users see native desktop requirement instead of raw invoke errors.
- Added shared layout CSS for native workspaces: responsive grid, wrapped actions, detail lists, form grids, selected list item styling, error styling, and mobile collapse.

## Backend/docs audit
- Checked Zoid docs: implementation plan/tracker and PRD confirm Tauri + React + TS, native Rust/Tauri commands, first-class Tasks, native Markdown Notes, and browser preview/native-only truthfulness.
- Checked Rust command surface/tests: `create_task_command`, `list_tasks_command`, `scan_markdown_notes_command`, and note/task bridge tests exist and pass.
- Frontend still calls the existing native bridge command names via `taskBridgeIntegration.ts` and `noteBridgeIntegration.ts`; no fake browser data was added.

## Verification run
- `npm run test:frontend && npm run build` passed.
- `npm run test:rust` passed: 179 passed, 0 failed, 1 ignored.
- Browser verification on `http://127.0.0.1:1420/`:
  - Tasks workspace: no horizontal overflow, organized unavailable panel, no fake list/detail/editor shown.
  - Notes workspace: no horizontal overflow, organized unavailable panel, no fake list/detail/editor/conflict/link panels shown.

## Known constraints
- Browser preview cannot verify real Tauri native data because `invoke` is unavailable outside the desktop app. This is expected and explicitly displayed.
- There are unrelated pre-existing/in-flight uncommitted Phase 4/Phase 5 files in the repo; review only this UI/layout/backend-linking scope unless you find direct regressions.

## Review request
Please critique for:
1. Layout regressions or accessibility issues in Tasks/Notes.
2. Any broken native backend wiring or mismatched command assumptions.
3. Whether hiding browser-preview fake panels preserves truthful native-only behavior.
4. Any Required fixes before approval.
