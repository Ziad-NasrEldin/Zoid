# Sidebar Liwan-Style Morph Handoff

## Scope
Apply the Liwan Portal reference morph behavior to Zoid 25 only. Liwan was reference-only; no Liwan source/data was changed.

Target: `/Users/ziadnasreldin/Zoid`

## Implementation
- Enhanced the existing Zoid 25 hamburger/sidebar collapse behavior.
- Added a Liwan-style FLIP/WAAPI morph path in `src/App.tsx`:
  - `handleSidebarMorphToggle`
  - shared morph anchors between full sidebar rows and compact rail icons
  - `data-sidebar-morph-panel` on the white sidebar panel
  - `flushSync` state flip to measure before/after geometry
  - cancels previous animations and removes morph clones before a new toggle
  - respects `prefers-reduced-motion`
- Fixed critique R1: morph anchors are now scoped to only the currently visible set:
  - expanded state exposes full `.nav-row` items
  - collapsed state exposes compact `.rail-nav-item` items
  - browser DOM probe confirms 7 morph keys and no duplicates in both states
- Updated `src/App.css`:
  - slower/eased shell/sidebar transitions using `cubic-bezier(0.16, 1, 0.3, 1)`
  - `sidebar-morphing` will-change hints
  - rail/sidebar transform origins for smoother row-to-icon motion
- Updated `src/scaffold.test.ts` static checks to require the morph infrastructure and visible-set scoping.

## Files Changed
- `src/App.tsx`
- `src/App.css`
- `src/scaffold.test.ts`

Additional build fix touched existing Zoid repository UI code:
- `src/code/CodeWorkspace.tsx`
  - restored/kept repository search state used by existing UI
  - removed the forbidden repository status panel while keeping TypeScript clean

## AI Course Platform Cleanup
Removed the mistaken auth-card Liwan morph review directory from:
`/Users/ziadnasreldin/Documents/GitHub/AI Course Platform/.hermes/reviews/auth-card-liwan-morph`

Targeted-reverted the mistaken AI Course auth morph edits; verification search found no remaining project-specific strings for:
- `Confirm password`
- `Passwords do not match`
- `auth-card-liwan-morph`
- `widthDelta`
- `signup-to-login link morphs`

Note: the AI Course checkout has untracked source/test files, so cleanup used targeted reversal and artifact deletion instead of destructive git reset.

## Verification
From `/Users/ziadnasreldin/Zoid`:

- `npm run test:frontend` — PASS
- `npm run build` — PASS, Vite large chunk warning only
- `npm test` — PASS
  - frontend scaffold test PASS
  - Rust/Tauri tests PASS, 9 passed

Local server:
- Restarted stale Zoid Vite server on `127.0.0.1:1420`
- Started tracked process: `proc_e2b799f71088`
- HTTP probe: `http://127.0.0.1:1420/` returned `200`

Browser verification:
- Loaded Zoid 25 locally.
- Expanded state DOM probe: 7 `data-sidebar-morph-item` keys, all full sidebar `.nav-row`, no duplicates.
- Clicked `Minimize sidebar`: compact blue rail showed 7 icon buttons; `.zoid25-shell` had `sidebar-collapsed`; DOM probe showed 7 compact `.rail-nav-item` morph keys, no duplicates.
- Clicked `Maximize sidebar`: full white sidebar restored; no console messages/errors.

## Critique History
First re-review: `REQUEST_CHANGES`
- R1 duplicate morph keys targeted hidden/wrong elements
- R2 frontend test failed due stale `Auto saved` assertion
- R3 token-only coverage missed visible-target mapping

Fixes applied:
- visible-state-scoped morph anchors
- scaffold test reconciled with current no-`Auto saved` topbar policy
- added static coverage for scoped morph anchor expressions
- build/type cleanup in `CodeWorkspace.tsx`

## Notes
- The working tree already contained many unrelated Zoid changes before this task. Review should focus only on the morph additions/fixes listed above.
- Non-blocking build warning remains: large JS chunk >500 KB.
