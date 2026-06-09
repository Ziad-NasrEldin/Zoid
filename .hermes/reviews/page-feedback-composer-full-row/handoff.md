# Feature Handoff: Page feedback composer full row

## Original request

## Page Feedback: /
**Viewport:** 1698×1009

### 1. <AgentsHermesScreen> <ChatComposer> form
**Location:** .hermes-chat-shell > .chat-workspace > .chat-main-pane > .chat-composer
**Source:** src/agents/ChatComposer.tsx:96:29
**React:** <AgentsHermesScreen> <ChatComposer>
**Classes:** chat-composer
**Position:** 653px, 854px (1045×121px)
**Feedback:** this section should slide to the left a little because it is intersecting with the sessiosn rail, i want the composer sectio nto take the whole row
this is in zoid 25

## Implementation summary

- Moved `<ChatComposer>` out of `.chat-main-pane` so it is a direct child of `.chat-workspace`.
- Changed `.chat-workspace` to a two-row grid: sessions rail + chat stage on row 1, composer spanning `grid-column: 1 / -1` on row 2.
- Set the sessions rail to end at row 1 above the composer, eliminating overlap/intersection.
- Added frontend scaffold regression checks for the full-row composer placement and sibling DOM structure.

## Changed files

- `src/agents/AgentsHermesScreen.tsx`: makes the composer a workspace sibling of the main pane so CSS grid can span it across the sessions rail and chat pane.
- `src/App.css`: adds chat workspace rows, positions rail/main/composer on explicit grid tracks, and preserves mobile stacking.
- `src/scaffold.test.ts`: adds regression assertions for composer full-row grid span and DOM placement.

## Scope Boundary / Dirty Working Tree Handling

Intended fix files only:

- `src/agents/AgentsHermesScreen.tsx`
- `src/App.css`
- `src/scaffold.test.ts`

Known unrelated dirty/untracked files already exist across the repo (package/Cargo files, other Zoid feature files, many `.hermes/reviews/*`, `src/code`, `src/automations`, `src/vendor`, etc.). They were not cleaned, reverted, or included in this focused approval claim.

Isolated diff command:

```bash
git diff -- src/agents/AgentsHermesScreen.tsx src/App.css src/scaffold.test.ts
```

## How to test

- `npm run test:frontend`
- `npm run build`
- Browser DOM geometry at `http://127.0.0.1:1420/` after opening Agents:
  - `.chat-composer` left/right equal `.chat-workspace` left/right.
  - `.sessions-rail` bottom equals composer top, no overlap.
- Installed native app:
  - rebuild/reinstall `/Applications/Zoid 25.app`
  - open Agents screen and screenshot-check composer spans below both sessions rail and main pane.

## Tests run

- `npm run test:frontend`: PASS.
- `npm run build`: PASS; Vite chunk-size warning only.
- Browser geometry probe: PASS. Result: `composerSpansWorkspace: true`, `railAboveComposer: true`, `gapOrOverlap: 0`; composer `left/right` matched workspace `left/right`.
- `cargo clean --manifest-path src-tauri/Cargo.toml && npm run tauri:build`: PASS. Clean rebuild was needed because the first packaged relaunch showed stale missing embedded assets (`asset not found: index.html`).
- Reinstalled `/Applications/Zoid 25.app` and verified process path: PASS (`/Applications/Zoid 25.app/Contents/MacOS/zoid`).
- Native screenshot `/tmp/zoid-agents-composer-native.png`: PASS; Agents screen loaded and composer visibly spans the whole bottom row below the sessions rail and main chat pane.
- `npm run test`: PASS; frontend scaffold test + 15 Rust tests passed.
- `git diff --check`: PASS.

## Git info

- Branch: current working tree, not committed by this task.
- Diff base: current uncommitted repo state.

## Frontend/backend/database notes

- Frontend routes/components: Agents Hermes screen layout only.
- Backend endpoints/services: none changed.
- Database tables/migrations: none.

## Reviewer focus areas

- Verify the composer is no longer nested under `.chat-main-pane` and can span the workspace row.
- Verify the rail height/row no longer intersects the composer.
- Verify mobile/narrow layout still stacks rail, main pane, and composer in order.
- Review only the isolated diff listed above because the repo has unrelated dirty work.

## Fix cycle notes

Initial handoff for critique.
