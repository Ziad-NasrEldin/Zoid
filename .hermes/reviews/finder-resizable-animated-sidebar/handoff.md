# Finder resizable animated sidebar handoff

## Scope
Implemented Finder sidebar polish in Zoid 25 Agent/Hermes screen:

- Finder sidebar width is manually resizable from its left edge.
- Finder width is clamped and persisted in localStorage.
- Finder resize handle exposes separator semantics and supports ArrowLeft/ArrowRight keyboard resizing.
- Finder open/close layout now uses a CSS grid-column transition plus panel enter animation.
- Removed the useless `Up` toolbar button; the Finder toolbar now keeps only `Refresh`.
- Added/updated tests for resize handle, no-Up toolbar, CSS variable mutation, and smooth-open CSS hooks.
- Fixed CommandPalette input highlight reset because the current frontend test suite exposed it while verifying this work.

## Main paths touched
- `src/agents/AgentsHermesScreen.tsx`
  - Added Finder width constants/storage.
  - Added `clampFileManagerWidth`, `getInitialFileManagerWidth`.
  - Added `fileManagerWidth` state and persisted CSS variable.
  - Added `handleFileManagerResizeStart` pointer-drag handler.
  - Added `.file-manager-resize-handle` button.
  - Removed Finder `Up` button rendering.

- `src/App.css`
  - Finder grid uses `--file-manager-width`.
  - Added `.file-manager-resize-handle` styles.
  - Added `file-manager-panel-enter` animation.
  - Added `grid-template-columns` transition on `.chat-workspace`.
  - Preserves narrow-layout behavior and hides resize handle on single-column mobile layout.

- `src/scaffold.test.ts`
  - Added static assertions for Finder resize/open animation hooks.
  - Added guard that `>Up</button>` is not rendered.

- `src/agents/AgentsHermesScreen.file-manager.test.tsx`
  - Updated behavior test to assert no Up button, Refresh remains, resize handle exists, and drag changes `--file-manager-width`.

- `src/agents/CommandPalette.tsx`
  - Reset highlighted command synchronously in input handler to satisfy existing behavior test.

## Verification already run
- `tsx src/agents/CommandPalette.behavior.test.tsx && tsx src/agents/AgentsHermesScreen.file-manager.test.tsx && npm run build` passed.
- `npm run tauri:build` passed and bundled `Zoid 25.app`.
- Installed bundle to `/Applications/Zoid 25.app`, relaunched via `open -b com.mavoid.zoid25`, and verified process is running.
- Browser/local app check confirmed:
  - Finder opened.
  - resize handle exists.
  - toolbar is `["Refresh"]` with no Up.
  - pointer drag changed `--file-manager-width` from `336px` to `436px`.
  - sidebar animation name is `file-manager-panel-enter`.
  - workspace transition is `grid-template-columns 0.42s cubic-bezier(0.16, 1, 0.3, 1)`.

## Known caveat
Full `npm run test:frontend` is currently blocked by an unrelated scaffold assertion: `Apple Notes Brain sync setup must not use native select controls`. The Finder-specific tests and build pass.

## Review request
Please critique the implementation against the user request:
1. Finder sidebar can be manually resized by dragging from its side.
2. No extra drop/dropdown surface was added.
3. Useless Up button removed or otherwise handled.
4. Finder opening feels smoother, not rough.

Focus on functional regressions, accessibility, persistence, layout interaction with chat/composer/sessions rail, and whether the implementation actually satisfies the request. Return verdict APPROVED or REQUIRED_FIXES with concrete required fixes only.