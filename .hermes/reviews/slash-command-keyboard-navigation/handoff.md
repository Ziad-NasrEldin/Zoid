# Slash command keyboard navigation handoff

## User complaint
Keyboard up/down did not navigate the slash command UI even though the UI copy said it would.

## Scope changed
- `src/agents/CommandPalette.tsx`
  - Added highlighted command state for the global command palette.
  - Added ArrowDown/ArrowUp wrapping navigation from the search input.
  - Added Enter to insert the highlighted command and Cmd/Ctrl+Enter to run it.
  - Added combobox/option active-descendant semantics and scroll-into-view for long command lists.
  - Mouse hover updates the highlighted command so keyboard/mouse state stays aligned.
- `src/agents/ChatComposer.tsx`
  - Inline `/` drop-up now wraps on ArrowUp/ArrowDown instead of clamping.
  - Enter now inserts the highlighted inline slash command, matching Tab behavior.
  - Active item scrolls into view in long slash-command lists.
  - Added active-descendant/id wiring and hover highlight sync.
- `src/App.css`
  - Added active visual state for command-palette highlighted options.
- `src/agents/CommandPalette.behavior.test.tsx`
  - New happy-dom behavior test for ArrowUp/ArrowDown, wrapping, filtering reset, Enter insert, Cmd/Ctrl+Enter run, Escape close.
- `src/agents/ChatComposer.slash.test.tsx`
  - Static guard updated to require the inline keyboard/scroll/a11y wiring.
- `package.json`
  - Added `CommandPalette.behavior.test.tsx` to `test:frontend`.
- `src/agents/AgentsHermesScreen.file-manager.test.tsx`
  - Fixed a broken existing test by re-querying the Projects row before collapse after React re-render.

## Verification already run
- `npm run test:frontend` → passed.
- `npm run test:frontend && npx tsc --noEmit` → passed after critique fix.

## Critique fixes applied
- Inline slash completion now only opens while editing the command token (`/que`), not after whitespace/arguments (`/queue hello`). This prevents Enter from re-inserting the command when the user is trying to send a completed slash command.
- Added `getInlineSlashSearch` guard coverage for `/queue ` and `/queue hello`.

## Review focus
1. Check keyboard behavior correctness and accessibility semantics.
2. Check if any stale refs or duplicate IDs can happen when filtering commands.
3. Check whether Enter insertion in inline slash drop-up can conflict with normal send.
4. Check TypeScript/build compatibility.
