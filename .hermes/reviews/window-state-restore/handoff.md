# Feature Handoff: Window size and location restore

## Original request

"i want in zoid 25 the opened window size and location to be remembered and restored when the app is restarted"

## Implementation summary

- Added Tauri's native window-state plugin to the Zoid 25 desktop app.
- Registered the plugin in the Tauri builder so the main app window persists and restores bounds across app quit/reopen.
- The plugin restores the opened window's position and size on startup without frontend-only/localStorage behavior.
- Fixed an existing TypeScript no-unused-local build blocker in `AgentsHermesScreen` by removing an unused destructured prop while keeping the prop type intact.

## Changed files

- `src-tauri/Cargo.toml`: added `tauri-plugin-window-state = "2"`.
- `src-tauri/Cargo.lock`: locked `tauri-plugin-window-state v2.4.1` and transitive dependencies.
- `src-tauri/src/lib.rs`: registered `tauri_plugin_window_state::Builder::new().build()` in the Tauri builder.
- `src/agents/AgentsHermesScreen.tsx`: removed unused `linkedRepositoryId` destructuring to unblock `tsc`/packaged build; no behavioral change intended.

## Scope Boundary / Dirty Working Tree Handling

The repo was already dirty before this focused fix, with many unrelated modified/untracked files and review artifacts. I did not revert or clean them. Review this focused change via:

```bash
git diff -- src-tauri/Cargo.toml src-tauri/Cargo.lock src-tauri/src/lib.rs src/agents/AgentsHermesScreen.tsx
```

Only the window-state plugin registration/dependency and the build-blocking unused destructure are intended for this handoff.

## How to test

1. Run Rust tests:
   ```bash
   cargo test --manifest-path src-tauri/Cargo.toml
   ```
2. Run frontend production build:
   ```bash
   npm run build
   ```
3. Build packaged Tauri app:
   ```bash
   npm run tauri:build
   ```
4. Install/relaunch packaged app:
   ```bash
   rm -rf "/Applications/Zoid 25.app"
   ditto "src-tauri/target/release/bundle/macos/Zoid 25.app" "/Applications/Zoid 25.app"
   open -a "/Applications/Zoid 25.app"
   ```
5. Move/resize the window, quit, reopen, and verify bounds are restored.

## Tests run

- `cargo test --manifest-path src-tauri/Cargo.toml`: PASS — 11 Rust tests passed.
- `npm run build`: PASS — TypeScript and Vite production build passed; Vite emitted only the existing large-chunk warning.
- `npm run tauri:build`: PASS — built `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Installed and relaunched `/Applications/Zoid 25.app`: PASS — running process verified at `/Applications/Zoid 25.app/Contents/MacOS/zoid`.
- Native window restore smoke: PASS — set bounds to `80,90,960,640`, quit, reopened from `/Applications`, read bounds as `80,90,960,640` without resetting them.
- Screenshot captured: `/tmp/zoid-window-state-verification.png`.

## Git info

- Branch: `main`
- Commit SHA, if committed: not committed
- Diff base, if known: current dirty working tree; scoped diff command listed above

## Frontend/backend/database notes

- Frontend routes/components: no window persistence frontend state added.
- Backend/native: Tauri native plugin persists/restores window state.
- Database: not applicable.

## Reviewer focus areas

- Confirm `tauri-plugin-window-state` is the correct native persistence mechanism for Tauri 2.
- Confirm the plugin is registered early enough in the Tauri builder to restore window bounds on app startup.
- Confirm verification proves installed `/Applications/Zoid 25.app` restored position and size after restart.
- Confirm the unused-destructure TypeScript fix is harmless and only unblocks builds.

## Fix cycle notes

- Initial critique returned `REQUEST_CHANGES` because its first Rust test run observed a transient failure in `tests::hermes_cli_message_runs_inside_linked_repository` and because the repo's pre-existing dirty-tree churn made the focused diff ambiguous.
- Re-ran the focused failing Rust test immediately after critique: PASS (`1 passed; 14 filtered out`).
- Re-ran the full Rust suite immediately after critique: PASS (`15 passed; 0 failed`).
- Scope clarification: broad changes in `src-tauri/src/lib.rs`, `Cargo.toml`, `Cargo.lock`, and `AgentsHermesScreen.tsx` already existed in the dirty working tree before this focused fix. The only intended window-state implementation is the `tauri-plugin-window-state` dependency/lock entry and Tauri builder plugin registration; the only intended non-window fix is the one-line unused prop destructure removal needed for `npm run build`.
