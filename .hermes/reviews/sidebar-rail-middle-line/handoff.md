# Sidebar rail middle line collapse behavior

## Request
Page Feedback for `/`: the vertical middle line in the global sidebar rail intersects icons when the full/sidebar navigation is compressed/minimized. Hide that middle line only while the sidebar is compressed/minimized; show it again when the sidebar is expanded.

## Changes made
- `src/App.css`
  - Kept `.ink-rail::before` as the expanded-state vertical middle line.
  - Added `transition: opacity 220ms ease` for a soft state change.
  - Added `.sidebar-collapsed .ink-rail::before { opacity: 0; }` so the line disappears only in collapsed mode.
- `src/scaffold.test.ts`
  - Added a source guard for the collapsed-state rail middle-line override.

## Verification already run
- `npm run test:frontend` exited 0.
- `npm run build` initially exited 0 before native packaging; a later rerun is blocked by unrelated pre-existing TypeScript unused-symbol errors in `src/agents/AgentsHermesScreen.tsx`.
- Browser at `http://127.0.0.1:1420/`:
  - Expanded: `.zoid25-shell`, rail `::before` opacity `0.62`, button `Minimize sidebar`.
  - Collapsed after clicking minimize: `.zoid25-shell sidebar-collapsed`, rail `::before` opacity `0`, button `Maximize sidebar`.
  - Re-expanded: rail `::before` opacity returns to `0.62`.

## Review focus
Confirm the CSS is scoped narrowly enough that the decorative rail line remains visible in expanded mode and is hidden only in compressed/collapsed mode, without affecting the rail glow `::after`, navigation icons, or sidebar morph behavior.
