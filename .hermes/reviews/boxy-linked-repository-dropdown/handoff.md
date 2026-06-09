# Feature Handoff: Boxy linked repository dropdown

## Original request
From Page Feedback item 5: “Shouldn't the drop-down button itself be boxy according to the design system? Right now, it's curvy a little bit. Can't you notice that? Please investigate it because I remember in the rest of the drop-down menus it was boxed. make the drop down menu button boxy”

## Implementation summary
- Fixed the global compact Zoid dropdown trigger style so compact dropdown buttons, including the topbar `Link repository` control, use `border-radius: 0` instead of the previous rounded 5px corners.
- Added a scaffold regression guard so compact dropdown triggers cannot silently return to 5px radius.
- Kept the shared `GlobalDropdown` primitive and accessibility behavior unchanged.

## Changed files
- `src/App.css`: changed `.zoid-dropdown--compact .zoid-dropdown-trigger` to `border-radius: 0`.
- `src/scaffold.test.ts`: added a source guard for boxy compact dropdown triggers.

## Tests run
- `npm run test:frontend`: PASS on rerun. First run failed from a stale/transient exact CSS guard in `ChatComposer.slash.test.tsx`, then passed after re-reading current source and rerunning without code changes.
- `npm run build`: PASS.
- `npm run tauri:build`: PASS; built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Reinstalled/relaunched `/Applications/Zoid 25.app`: PASS; process running from `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- Browser smoke on `http://127.0.0.1:1420`: PASS; computed `#linked-repository-select` `borderRadius` is `0px`, no console errors.
- Native screenshot `/tmp/zoid-dropdown-boxy.png`: PASS; Zoid app visible and topbar Link repository dropdown is square/boxy.
- `npm run test:rust`: FAIL, unrelated to this CSS-only dropdown change. Existing broader Rust tests failed in file-permission warmup cases: `permission_warmup_treats_touched_home_as_app_wide_root`, `warm_file_permissions_persists_marker_after_first_run`, `warm_file_permissions_records_new_paths_after_marker_exists`. The release build still passed.

## Reviewer focus areas
- Confirm the requested topbar Link repository dropdown is boxy.
- Confirm the fix is scoped to compact global dropdown styling and does not change dropdown behavior.
- Confirm the scaffold guard protects the design-system invariant.
- Treat Rust file-permission warmup failures as unrelated existing dirty-tree/backend state unless you find this CSS/test guard caused them.

## Notes
The repository has extensive unrelated dirty/untracked Zoid work from prior tasks. Review only the scoped dropdown styling/test guard change.
