# Feature Handoff: Compact Hermes session avatar fill

## Original request

Page Feedback for `/` at `tauri://localhost`: `this avatar icon should take the whole window/box not just the iinner box please` for `span.session-tab-icon.session-tab-portrait` inside the Hermes sessions rail.

## Implementation summary

- In compact sessions rail mode, the portrait span now stretches absolutely to the full session-tab square instead of rendering as a smaller 34px inner avatar box.
- The new compact portrait rule removes the translucent inner background/backdrop/box-shadow so the avatar image itself occupies the full box.
- New scaffold source guard asserts compact portraits use `position:absolute`, `inset:0`, `width/height:100%`, and `place-self:stretch`.
- This is scoped to compact `.session-tab-icon.session-tab-portrait`; the New Session plus icon keeps the smaller icon box.

## Changed files

- `src/App.css`: added compact portrait full-box override.
- `src/scaffold.test.ts`: added source guard for full-box compact portrait avatars.

## How to test

- `npm run build`
- `npm run tauri:build`
- Replace `/Applications/Zoid 25.app` with the built bundle and launch `/Applications/Zoid 25.app`.
- In Hermes Agents workspace with compact sessions rail, session avatar portraits should fill the full square boxes; there should be no smaller translucent inner portrait box.

## Tests run

- `npm run test:frontend`: FAIL, blocked by existing unrelated scaffold guard before this change: `Hermes session tabs need deterministic visibly distinct historical-figure portrait treatment: SESSION_FIGURE_PORTRAITS` because `src/agents/sessionPortraits.ts` does not contain that stale symbol.
- `npm run build`: PASS.
- `npm run tauri:build`: PASS, bundle created at `src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Installed/relaunched `/Applications/Zoid 25.app`: PASS, running process `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- Native screenshot `/tmp/zoid-front-process-zoid.png`: PASS, Hermes sessions rail visible and compact session portraits visually fill their square boxes.

## Git info

- Branch: not checked out for a clean branch in this handoff.
- Commit SHA: not committed.
- Diff base: working tree is already broadly dirty from unrelated Zoid work; intended scoped files are `src/App.css` and `src/scaffold.test.ts`.

## Frontend/backend/database notes

- Frontend/CSS only.
- No backend commands changed.
- No database changes.

## Reviewer focus areas

- Check only the scoped compact session avatar fill change, not unrelated existing dirty tree work.
- Confirm the New Session plus icon still keeps the intended icon-box treatment.
- Confirm the added scaffold guard protects against the reported small-inner-box regression.

## Fix cycle notes

Initial review request.
