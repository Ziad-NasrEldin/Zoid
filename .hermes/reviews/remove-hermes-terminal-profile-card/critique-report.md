# Critique Report: Remove Hermes terminal profile card

## Verdict

APPROVED

## Scope reviewed

Reviewed the active repository at `/Users/ziadnasreldin/Zoid` for the requested Agents > Hermes UI removal: the left-side `TERMINAL SESSION` / `HA Hermes` / `Hermes CLI is available...` profile card should be removed, and the conversation/message area should reclaim the horizontal space. I kept the review scoped to `src/agents/AgentsHermesScreen.tsx` and `src/App.css` and did not edit product code.

## Findings

### Left-side Hermes profile card removal

- PASS: `src/agents/AgentsHermesScreen.tsx` no longer renders the `<aside className="agent-profile-card">` region.
- PASS: The rejected strings/classes are absent from active `src/`: `TERMINAL SESSION`, `agent-profile-card`, `profile-label`, `Hermes CLI is available`, `MessageProfile`, and `connectionCopy` returned zero search matches.
- PASS: The former profile-card-specific CSS rules (`.agent-profile-card`, `.profile-label`, `.profile-row`, `.agent-profile-card p`) have been removed from `src/App.css`.

### Message list uses the removed space

- PASS: `.chat-stage` is now a single-column grid:

```css
grid-template-columns: minmax(0, 1fr);
```

- PASS: The previous two-column layout (`minmax(170px, 250px) minmax(0, 1fr)`) and the 26px column gap reservation are gone, so the chat stage no longer reserves the left profile-card column.
- NOTE: Individual message bubbles still have their existing `max-width: min(760px, 88%)`; this limits bubble width but does not reintroduce the removed left-side column. The message list container itself occupies the single available chat-stage column.

### Top-right Hermes status remains intact

- PASS: The topbar still renders the Hermes CLI status panel via `.connection-panel`, with the status dot and `Hermes CLI {connectionState.toUpperCase()}` text.
- PASS: The repository-link controls remain in the topbar area rather than in the removed left-side profile-card region.

### Scope / dirty tree

- PASS: The scoped diff includes the intended files: `src/agents/AgentsHermesScreen.tsx` and `src/App.css`.
- NOTE: These same files also contain adjacent/concurrent Agents metrics/repository-link changes that are broader than this removal, and the repository has other unrelated dirty files as called out in the handoff. I did not treat those as blockers because this critique is scoped to whether the requested left-side profile card was removed and whether the chat stage reclaimed the column.

## Verification performed

- PASS: Searched active `src/` for rejected profile-card strings/classes; zero matches.
- PASS: Inspected `src/agents/AgentsHermesScreen.tsx` and confirmed only the message list remains inside `.chat-stage`.
- PASS: Inspected `src/App.css` and confirmed `.chat-stage` is single-column and profile-card CSS is removed.
- PASS: Ran `npm run build` from `/Users/ziadnasreldin/Zoid`; `tsc && vite build` completed successfully, with Vite transforming 37 modules and producing `dist/` assets.

## Issues / risks

- No blocking issues found for the requested UI removal.
- The implementation did not add a focused automated assertion for this exact removal, but the source inspection, zero-match search, and successful build are sufficient for this narrow UI cleanup.
- Concurrent unrelated dirty changes remain in the repo; they should be handled by their own reviews/tasks and are not grounds to reject this scoped removal.

## Conclusion

The unwanted left-side Hermes terminal/profile card is removed from active source, its CSS and identifying strings are gone, the chat stage no longer reserves the old column, and the build passes. Approved.
