# Sidebar collapse blue rail handoff

## Feature
Make the far-left hamburger button toggle the main editorial sidebar between expanded and compact states. On minimize, the white primary sidebar should collapse into the blue rail and each navigation item should become a unique branded icon.

## Relevant files
- `src/App.tsx`
- `src/App.css`
- `src/scaffold.test.ts`

## Implemented behavior
- Added `isSidebarCollapsed` state on the Zoid shell.
- Far-left hamburger is now a real `<button>` with `Minimize sidebar` / `Maximize sidebar` labels and pressed state.
- Expanded state keeps the original blue rail + white editorial sidebar.
- Collapsed state changes the shell grid to blue rail + main workspace, hides the white sidebar, and renders `Compact section navigation` inside the blue rail.
- Compact nav has one unique lucide icon per section:
  - Today: `CalendarDays`
  - Projects: `FolderKanban`
  - Agents: `Bot`
  - Code: `Code2`
  - Content: `Megaphone`
  - Automations: `Repeat2`
  - Settings: `Settings`
- Compact nav preserves active section styling and status dots using Zoid's blue/white/yellow/green/red brand language.
- Expanded sidebar rows also show subtle matching section icons so the morph is visually connected.

## Verification already run
- `npm run test:frontend` → pass
- `npm run build` → pass
- `curl -I --max-time 5 http://127.0.0.1:1420/` → HTTP 200
- Browser smoke at `http://127.0.0.1:1420/`:
  - Initial state shows button `Minimize sidebar` and full white primary nav.
  - Clicking hamburger changes label to `Maximize sidebar` and shows `Compact section navigation` with icon-only nav in blue rail.
  - Clicking hamburger again restores full white primary nav.

## Review focus
- Check that the collapse/morph behavior specifically satisfies the user's request for the white sidebar to compact into the blue rail.
- Check accessibility/keyboard labels and active state.
- Check for regressions from the existing dirty tree; this task should be scoped to sidebar collapse files above.
