# Feature Handoff: Hermes slash full registry scroll

## Original request

Wire all Hermes slash commands into Zoid 25's Hermes composer drop-up, while keeping the same screen/drop-down size and adding internal scrolling instead of expanding the UI.

## Current implementation

- `src/agents/ChatComposer.tsx`
  - The typed `/` inline drop-up now renders every matching command from the live `slashCommands` prop instead of truncating to 9.
  - Empty `/` shows the full live Hermes command list.
  - Search still filters using `commandSearchText(command)`.
  - Drop-up remains keyboard-first:
    - ArrowDown/ArrowUp update the highlighted command.
    - Tab inserts the highlighted command.
    - Escape clears the slash draft.
  - Added active-option refs and `scrollIntoView({ block: "nearest" })` so keyboard navigation follows the highlighted item inside the scroll container.
  - Added `aria-activedescendant` for active-item semantics.

- `src/App.css`
  - `.composer-slash-dropup` keeps the same max-height and uses hidden outer overflow.
  - `.composer-slash-dropup-list` uses `min-height: 0; overflow: auto;` so extra commands scroll inside the existing popover.
  - `.composer-input-column` has `min-width: 0` to preserve layout constraints.

- `src/agents/ChatComposer.slash.test.tsx`
  - Static guard now verifies no `matches.slice(0, 9)` cap exists.
  - Static guard verifies the scroll container and active-item scroll wiring exist.

- `src-tauri/src/lib.rs`
  - Improved permission warmup root-coverage path comparison by checking canonical path candidates inside `raw_path_is_covered_by_touched_root`.
  - Updated two permission warmup tests to match the current home-root coverage behavior.

- `src/agents/AgentsHermesScreen.file-manager.test.tsx`
  - Reconciled the behavior test with the scaffold guard that the removed useless `Up` toolbar button must not render; this was needed to keep the existing full frontend gate green.

## Verification already run

- `npm run test:frontend` — PASS
- `npm run test:rust` — PASS, 61 Rust tests passed
- `npm run build` — PASS
- Hermes live registry checked via Python source: `COMMAND_REGISTRY` currently exposes 71 commands in this local Hermes source checkout.

## Notes for reviewer

- The actual Zoid backend already loads the Hermes Python `COMMAND_REGISTRY`; the main bug was frontend truncation in the typed slash drop-up.
- Keep the critique focused on whether all commands can appear in the existing-sized drop-up, scroll internally, and keyboard navigation stays visible.
