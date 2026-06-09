# Feature Handoff: Remove Hermes terminal profile card

## Original request

"Ok i like what you did in the trrminal but i want you to remove the element on the left of the hermes status, it takes way too much space and its useless"

Follow-up context identified the unwanted region as the `TERMINAL SESSION` / `HA Hermes` / `Hermes CLI is available...` profile card in the Zoid Agents > Hermes chat screen.

## Implementation summary

- Removed the left-side active agent/profile card from the Hermes chat stage.
- Removed the `TERMINAL SESSION` copy and profile-card rendering path.
- Changed the chat stage from a two-column grid to a single full-width message area so the removed element's space is returned to the conversation.
- Removed now-unused profile card CSS rules.
- Kept the top-right Hermes CLI status panel and repository link controls intact.

## Changed files

- `src/agents/AgentsHermesScreen.tsx`: removed the `<aside className="agent-profile-card">` section, removed `connectionCopy`, removed `MessageProfile`, and simplified Hermes participant import/presence construction.
- `src/App.css`: changed `.chat-stage` to one column and removed `.agent-profile-card` / `.profile-*` CSS rules.

## How to test

1. Run `npm run build` from `/Users/ziadnasreldin/Zoid`.
2. Run `npm run test` from `/Users/ziadnasreldin/Zoid`.
3. Open the running app at `http://127.0.0.1:1420/`, click `Agents`, and verify:
   - no left-side `TERMINAL SESSION` card appears;
   - no `HA Hermes` profile card appears left of the chat;
   - no `Hermes CLI is available` profile-card paragraph appears;
   - the message list spans the chat stage width.

## Tests run

- `search_files` in `src` for `TERMINAL SESSION|agent-profile-card|profile-label|Hermes CLI is available`: PASS, zero matches.
- `npm run build`: PASS (`tsc && vite build`, 37 modules transformed).
- `npm run test:frontend`: PASS.
- `npm run test`: PASS (`test:frontend` plus Rust tests; 4 Rust tests passed).
- `curl -I --max-time 5 http://127.0.0.1:1420/`: PASS, HTTP 200.
- Browser DOM check on `http://127.0.0.1:1420/` after clicking Agents: PASS — `{ terminalSession: false, agentProfileCard: 0, profileLabel: 0, bodyHasHermesAvailable: false, chatStageColumns: "804px", messageListWidth: 804 }`.

## Git info

- Branch: current working tree, uncommitted.
- Commit SHA: not committed.
- Scoped diff for this handoff: `src/agents/AgentsHermesScreen.tsx` and `src/App.css` only.
- Note: the repository already has unrelated modified/untracked files from adjacent work (`src-tauri/src/lib.rs`, `src/App.tsx`, `src/agents/hermesClient.ts`, `src/scaffold.test.ts`, and older `.hermes/reviews/*`). This review should stay scoped to the two files above.

## Frontend/backend/database notes

- Frontend route/component: Agents > Hermes chat screen (`AgentsHermesScreen`).
- Backend/native/database: not changed.
- Desktop/native note: existing Vite dev server was already serving on `127.0.0.1:1420`; browser preview verified the rendered UI removal. Existing `/Applications/Zoid 25.app` process was observed running, but this UI-only browser-verifiable removal does not add native bridge behavior.

## Reviewer focus areas

- Confirm the exact user-rejected left-side profile card is gone, not merely restyled.
- Confirm the topbar Hermes status remains visible.
- Confirm the chat stage layout no longer reserves the removed column.
- Confirm no accidental unrelated dirty-tree changes are included in this scoped review.

## Fix cycle notes

Initial review request.
