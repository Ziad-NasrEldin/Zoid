# Feature Handoff: Profile catalog settings feedback

## Original request

"In this trusted project, mcp servers, plugins, and Skills should be displayed with either a list format or a grid format if they are small in number by default. They can't be displayed as an editable text field. It doesn't make sense. So first of all, fix that. Also I would like you to add a new feature. I would like you to first link up all the skills that are in Hermes because I remember Hermes have more than 100 skills and you're only showing four of them I believe. So first link them all and secondly I want you to add the feature to disable and enable a specific skill. Same with plugins, mcp servers and toolsets. I would like to be able to easily disable and enable them. did you do that ??!!"

## Implementation summary

- Settings > Tools now uses catalog/list cards for Toolsets, MCP servers, Plugins, and Skills instead of raw editable textareas.
- Each catalog item has an enabled/disabled checkbox and visible status copy.
- Hermes skills are discovered from the real Hermes CLI using `hermes skills list --source all` and fallback recursive profile/global skills directory scanning, instead of the prior small fallback set.
- Toggle persistence writes back to Hermes config shapes:
  - `agent.disabled_toolsets`
  - `skills.disabled`
  - `plugins.enabled` / `plugins.disabled`
  - per-MCP-server `enabled` flags
- Added Tauri notification plugin init/command registration because the aggregate frontend guard was failing on that existing notification surface.

## Changed files

- `src/App.tsx`: renders catalog controls for toolsets/MCP/plugins/skills and wires checkbox updates to profile settings.
- `src-tauri/src/lib.rs`: hydrates full Hermes skill inventory, computes enabled/disabled state, saves config toggles, registers notification plugin/command.
- `src/scaffold.test.ts`: contains source guards for catalog controls and real Hermes skill hydration.
- `src-tauri/Cargo.toml` / `Cargo.lock`: include notification plugin dependency already required by the app notification feature.

## How to test

- Open `/Applications/Zoid 25.app`.
- Navigate to Settings > Tools.
- Confirm Toolsets, MCP servers, Plugins, and Skills render as list/grid catalog cards, not editable textareas.
- Confirm Skills list includes the real Hermes inventory (local CLI showed 120 parsed skill rows / 127 CLI output lines), not only four seed skills.
- Toggle a skill/plugin/MCP/toolset, save, and reload to confirm state persists through the active Hermes profile config.

## Tests run

- `COLUMNS=400 hermes skills list --source all`: parsed 120 real skill rows from the Hermes CLI inventory.
- `npm run test:frontend`: PASS.
- `npm run test:rust`: PASS, 66 passed, 1 ignored.
- `npm run build`: PASS.
- `npm run tauri:build`: PASS, bundled `src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Replaced `/Applications/Zoid 25.app`, launched bundle id `com.mavoid.zoid25`, confirmed process path `/Applications/Zoid 25.app/Contents/MacOS/zoid`.

## Git info

- Branch: main
- Commit SHA: not committed in this handoff.
- Diff base: current working tree has many broader existing Zoid changes; review should scope this feature to settings catalog controls and Hermes profile config code.

## Frontend/backend/database notes

- Frontend: `App.tsx` Settings/Profile Tools section.
- Backend: Tauri profile settings load/save in `src-tauri/src/lib.rs`.
- Database: none.

## Reviewer focus areas

- Check that the user-requested resources are not rendered as editable catalog textareas.
- Check that full skill inventory discovery is real and not seeded to four skills.
- Check toggle writes preserve Hermes config/YAML shape and avoid deleting unrelated config.
- Check the notification plugin registration fix is minimal and does not affect catalog behavior.

## Fix cycle notes

Initial handoff after implementation and local verification.
