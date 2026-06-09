# Feature Handoff: Hermes session portrait blur removal

## Original request

Page Feedback for `/` at `tauri://localhost`: remove the blur effect from `span.session-tab-icon.session-tab-portrait` in the Hermes sessions rail.

## Implementation summary

- Removed the expanded Hermes session tab portrait icon blur effect from both the base rule and the later override rule.
- Kept the portrait image treatment deterministic and distinct by preserving saturation/contrast only.
- Adjusted the scaffold guard so any expanded portrait blur cannot return.

## Changed files

- `src/App.css`: removed `blur(1.6px)`, negative inset, and scale from `.session-tab-portrait::before`.
- `src/scaffold.test.ts`: updated source guard from blurred portrait treatment to sharp expanded portrait treatment and added a negative guard for the old blur string.

## How to test

- `npm run test:frontend`
- `npm run build`
- `npm run tauri:build`
- Install `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app` to `/Applications/Zoid 25.app`, relaunch `/Applications/Zoid 25.app/Contents/MacOS/zoid`, open Agents, inspect sessions rail portrait icons.

## Tests run

- `npm run test:frontend -- --unused`: PASS
- `npm run build`: PASS
- `npm run tauri:build`: PASS, with existing Rust dead-code warnings for `apply_profile_runtime_args` and `prompt_with_enabled_profile_context`.
- CSS source probe: PASS (`expanded_has_blur=False`, expanded filters are saturation/contrast only).
- Native relaunch: PASS, process running from `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- Native screenshot: PASS, Agents/Hermes sessions rail visible with portrait icons after relaunch.

## Git info

- Branch: main
- Commit SHA, if committed: not committed
- Diff base, if known: working tree already had many unrelated dirty/untracked files before this change.

## Frontend/backend/database notes

- Frontend routes/components: Hermes Agents page session rail CSS only.
- Backend endpoints/services: none.
- Database tables/migrations: none.

## Reviewer focus areas

- Verify the targeted expanded `.session-tab-icon.session-tab-portrait` no longer uses the blur filter.
- Verify the change is scoped and does not remove unrelated compact-session background styling.
- Verify scaffold guard matches the user request.

## Fix cycle notes

- Fixed review finding R1 by splitting the later override block so `.session-tab-portrait::before` stays non-blurred while compact session-tab backgrounds keep their existing styling.
- Strengthened `src/scaffold.test.ts` to fail on any `filter: blur(...)` in expanded `.session-tab-portrait::before` blocks, not only the old exact `blur(1.6px)` string.
- Re-ran `npm run test:frontend`, `npm run build`, `npm run tauri:build`, reinstalled `/Applications/Zoid 25.app`, relaunched it, and captured the Agents/Hermes rail screenshot.
