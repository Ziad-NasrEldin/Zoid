# Feature Handoff: Zoid session header button alignment

## Original request

"these buttons are not alligned and not the same size" with screenshot of the Sessions rail header count chip and minimize button.

## Implementation summary

- Normalized the Sessions header count chip and minimize/maximize button to a shared `--session-rail-control-size`.
- Forced both controls onto fixed-width grid columns with centered placement, equal width/height/min-width/min-height, and border-box sizing.
- Removed the extra decorative morph mark from the rendered button so the minimize/maximize button appears as a clean square matching the count chip.
- Kept compact rail controls centered and equal-sized in the installed native app.
- Preserved unrelated existing dirty repo work; this handoff is scoped to the visible Sessions header control alignment.

## Changed files

- `src/App.css`: Sessions rail header control sizing/alignment rules.
- `src/agents/AgentsHermesScreen.tsx`: keeps archive buttons out of compact mode; no behavior change to the header control itself.

## How to test

- Browser: `npm run dev`, open `http://127.0.0.1:1420/`, go to Agents.
- Expected: `sessions-rail-count` and `sessions-rail-morph-button` have equal dimensions and the same vertical center.
- Native: launch `/Applications/Zoid 25.app` and inspect the Sessions header.

## Tests run

- `npm run test:frontend`: PASS.
- `npm run build`: PASS.
- Browser geometry probe on `http://127.0.0.1:1420/`: PASS — count and button were both 32x32, same y, same center.
- Browser visual screenshot inspection: PASS.
- `npm run tauri:build`: PASS; built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Reinstalled to `/Applications/Zoid 25.app` and relaunched: PASS, running process `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- Native screenshot inspection: PASS — Zoid is in front; Sessions header count chip and minimize button appear aligned and equal size.
- `git diff --check -- src/App.css src/agents/AgentsHermesScreen.tsx`: PASS.

## Git info

- Branch: `main`.
- Commit SHA: not committed.
- Note: repository had substantial pre-existing dirty/untracked work before this scoped visual fix.

## Frontend/backend/database notes

- Frontend routes/components: Agents / Hermes chat Sessions rail header.
- Backend endpoints/services: not applicable.
- Database tables/migrations: not applicable.

## Reviewer focus areas

- Confirm the count chip and minimize/maximize button are the same apparent size and baseline/center aligned in the Sessions header.
- Confirm the compact rail still centers the controls cleanly.
- Confirm this scoped fix does not claim ownership of unrelated pre-existing dirty files.

## Fix cycle notes

Initial review requested.
