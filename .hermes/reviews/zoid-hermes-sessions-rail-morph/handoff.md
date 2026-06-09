# Feature Handoff: Zoid Hermes sessions rail morph

## Original request

add the same morphing effect that we have in zoid 25 of the sidebar and add it to the sessions rail list

## Implementation summary

- Added a FLIP-style minimize/maximize morph to the Hermes sessions rail using the same timing/easing pattern as the main sidebar.
- The sessions rail toggle now snapshots the rail panel and each session/new-session tab, flushes the compact state update, then animates matching items from previous to next geometry.
- Added reduced-motion fallback and clone cleanup for disappearing morph items.
- Added source guard coverage so the rail keeps the same morph system.

## Changed files

- `src/agents/AgentsHermesScreen.tsx`: sessions rail morph timing, refs, toggle handler, morph data attributes, and ref wiring.
- `src/App.css`: will-change styling for rail morph participants.
- `src/scaffold.test.ts`: guard that sessions rail uses sidebar-equivalent morph primitives.

## How to test

- `npm run test:frontend`
- `npm run build`
- `npm run tauri:build`
- In browser or native app, open Agents and click Minimize/Maximize sessions rail; the rail and session tabs should morph with the sidebar cubic-bezier timing rather than instantly swapping.

## Tests run

- `npm run test:frontend`: PASS
- `npm run build`: PASS
- Browser animation instrumentation on `http://127.0.0.1:1420` Agents page: PASS, both minimize and maximize produced 540ms `cubic-bezier(0.16, 1, 0.3, 1)` animations for workspace, rail panel, new-session tab, and active session tab.
- `npm run tauri:build`: PASS; rebuilt `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Reinstalled and relaunched `/Applications/Zoid 25.app`; running process verified at `/Applications/Zoid 25.app/Contents/MacOS/zoid`.

## Git info

- Branch: current working tree
- Diff base: existing dirty local repo; this handoff scopes review to the three changed files above.

## Frontend/backend/database notes

- Frontend only: React sessions rail UI and CSS.
- Backend/database: not applicable.

## Reviewer focus areas

- Check that the sessions rail uses the same timing/easing pattern as sidebar morph.
- Check compact and expanded toggles still preserve controls/accessibility.
- Check this change does not touch unrelated dirty work.
