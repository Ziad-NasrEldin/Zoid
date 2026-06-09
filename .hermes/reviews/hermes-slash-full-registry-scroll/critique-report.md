# Critique Report: Hermes slash full-registry scroll

## Verdict

APPROVED

## Scope reviewed

Scoped to the requested implementation and gate-support files:

- `src/agents/ChatComposer.tsx`
- `src/App.css`
- `src/agents/ChatComposer.slash.test.tsx`
- `src-tauri/src/lib.rs`
- `src/agents/AgentsHermesScreen.file-manager.test.tsx`

## Findings

- The inline typed `/` drop-up now derives `inlineSlashCommands` directly from the full live `slashCommands` prop, returning the entire registry for an empty slash search and filtering only by `commandSearchText` when text is present. I did not find the previous `matches.slice(0, 9)` truncation.
- The drop-up keeps a bounded outer size via `.composer-slash-dropup` `max-height` and `overflow: hidden`, with the internal `.composer-slash-dropup-list` using `min-height: 0` and `overflow: auto`, so the full registry scrolls inside the existing popover rather than expanding the UI.
- The deep slash panel similarly uses a fixed/bounded max height with hidden outer overflow and an internally scrolling `.slash-command-list`, preserving the existing panel footprint while allowing the full command set to be available.
- Keyboard navigation updates `highlightedSlashCommandIndex` for ArrowUp/ArrowDown and calls `scrollIntoView({ block: "nearest" })` on the active option ref whenever the highlight changes while the inline slash menu is open. This resolves the prior blocker where active keyboard selection could move outside the visible scroll viewport.
- Active item semantics are improved through `aria-activedescendant`, option `id`s, and `aria-selected` on the highlighted item.
- The frontend static guard now checks the no-truncation condition and the scroll/active-item wiring that matters for this regression.
- The reviewed support changes in `src-tauri/src/lib.rs` and `AgentsHermesScreen.file-manager.test.tsx` are relevant to keeping the current gates green and do not appear to conflict with the slash-scroll implementation.

## Verification

Ran:

```sh
npm run test:frontend
```

Result: PASS (`exit_code: 0`).

## Notes

No source code was edited. The only file changed by this review is this report, overwriting the previous critique as requested.
