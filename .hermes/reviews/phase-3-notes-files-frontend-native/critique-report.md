# Critique Report: Phase 3 Notes/Files frontend native integration

## Verdict

APPROVED

## Summary

The current uncommitted implementation wires the Notes and Files workspaces into the native React shell, hides the generic inspector only for those two native editor workspaces, and surfaces real bridge-driven controls instead of fabricated browser preview content. The handoff's native evidence is sufficient for this gate: it names the rebuilt/reinstalled `/Applications/Zoid.app` process, records installed-app verification, and provides two 1920x1080 native screenshots with the expected Notes/Files controls. I found no Required fixes.

## What was reviewed

- Handoff: `.hermes/reviews/phase-3-notes-files-frontend-native/handoff.md`.
- Current working tree diff/status for modified and untracked implementation files.
- Frontend shell wiring in `src/App.tsx` and native-editor layout in `src/App.css`.
- Notes bridge/view/workspace files: `src/noteBridgeIntegration.ts`, `src/noteViewModel.ts`, `src/noteWorkspace.tsx`, and tests.
- Files bridge/view/workspace files: `src/fileBridgeIntegration.ts`, `src/fileViewModel.ts`, `src/fileWorkspace.tsx`, and tests.
- Backend command signatures/registrations spot-check in `src-tauri/src/lib.rs` for invoke command names and arg shapes.

## Required fixes

| ID | Severity | Area | Issue | Evidence | Required fix |
|----|----------|------|-------|----------|--------------|
| — | — | — | No Required fixes found. | Reviewed source/diff, untracked files, handoff native evidence, screenshots' existence/dimensions, and reran frontend tests/build. | — |

## Findings

- Native UI evidence is acceptable for the completion gate. The handoff explicitly records rebuild, reinstall, cache clearing, `/Applications/Zoid.app/Contents/MacOS/zoid` process verification, and screenshots showing `Refresh real notes`, `Scan Markdown notes`, `Create Markdown note`, root/relative path inputs, `Browse real files`, and `Perform file action`. The referenced screenshot files exist at `/tmp/zoid-native-notes-editor-active.png` and `/tmp/zoid-native-files-exact.png` and are both 1920x1080 PNGs.
- `src/App.tsx` renders `NoteWorkspace` for `active?.id === "notes"` and `FileWorkspace` for `active?.id === "files"`, with the workspace IDs matching `src/workspaceRegistry.ts`.
- The Notes/Files bridge modules call the registered Tauri commands with matching command names and expected camelCase wrapper keys for Rust snake_case params (`noteId`, `request`), and they surface bridge errors as explicit UI error states rather than inventing sample data.
- `native-editor-active` is scoped to Notes/Files only. It removes the generic inspector pane that previously obscured the workspace content, while non-Notes/Files workspaces keep the existing inspector behavior.
- File consequential-action policy remains visible inside `FileWorkspace` via `preview_action_policy`; failures to load policy are shown as an explicit unavailable state rather than hidden success. The file action form passes through persisted `confirmation_id` when provided and does not fabricate one.
- Untracked implementation files are expected and complete for this slice: Notes/Files bridge integrations, view models, workspaces, and their tests are all present and included in `package.json`'s `test:frontend` script.

## Improvements

| ID | Priority | Area | Suggestion | Why it matters |
|----|----------|------|------------|----------------|
| I1 | Low | Notes UX | Consider making the edit form copy clearer that the current bridge edit path updates Markdown body only, even though the form also displays title/path/metadata fields. | Avoids user confusion until richer edit semantics are supported by the native note command surface. |
| I2 | Low | Tests | Add a lightweight component/App render test that asserts selecting Notes/Files renders the native workspace controls and applies `native-editor-active`. | Current bridge/view-model tests plus build cover behavior and typing, but a render regression would directly guard the original layout mismatch. |
| I3 | Low | Native evidence | Keep future native GUI handoffs with either OCR/accessibility output or attached screenshots in the review folder, not only `/tmp`. | Makes the visual evidence easier to audit after temporary files are cleaned up. |

## Tests performed

- `git status --short && git diff --stat && git diff -- src/App.tsx src/App.css src/noteWorkspace.tsx src/fileWorkspace.tsx src/noteBridgeIntegration.ts src/fileBridgeIntegration.ts src/noteViewModel.ts src/fileViewModel.ts`: inspected the current modified/untracked implementation shape.
- Read the handoff and relevant source/test files listed above.
- Spot-checked backend command names/signatures in `src-tauri/src/lib.rs` against frontend bridge constants.
- Verified screenshot files exist and are valid PNGs: `/tmp/zoid-native-notes-editor-active.png` and `/tmp/zoid-native-files-exact.png`, both 1920x1080 RGBA.
- `npm run test:frontend && npm run build`: PASS. All frontend tests passed; Vite built `dist/assets/index-DY2vaSQz.css` and `dist/assets/index-CR4HAcWD.js`.
- `git diff --check`: PASS.

## Tests still needed

- None required for this frontend/native integration gate. The dev handoff also records `npm run tauri:build -- --bundles app` PASS, installed `/Applications/Zoid.app` native UI screenshot verification PASS, and `npm run verify:local` PASS.

## Dev-agent instructions

1. No Required fixes remain; this slice is approved for completion/commit once the untracked implementation files and this report are included intentionally.
2. Do not drop the untracked Notes/Files source and test files during commit; they are part of the approved implementation.
3. Optional follow-up: add direct React render coverage for Notes/Files workspace visibility and preserve native screenshot/OCR evidence in the review folder for future audits.
