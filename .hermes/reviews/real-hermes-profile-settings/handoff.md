# Feature Handoff: Real Hermes profile settings wiring

## Original request

Page Feedback for `/` in Zoid 25 requested that the Profile, Memory & Soul settings stop showing fake/static data: memory/soul budget, access/model cards, communication style, timezone, model, reasoning, approvals, notifications, voice, and archived sessions should be grounded in real Hermes/Zoid state. The request also asked for dropdowns, at least 5 communication templates, removal of useless fake auxiliary model notes, clipped model text fix, and bulk/delete-all archived session controls.

## Implementation summary

- Settings page now loads real active Hermes profile sources through Tauri/Rust:
  - `~/.hermes/config.yaml` for model provider/model, reasoning effort, display personality, approval mode, memory toggles, redaction toggles, STT/TTS, gateway channel notes, completion bell.
  - `~/.hermes/memories/MEMORY.md` and `USER.md` for editable Hermes memory/user preference content.
  - `~/.hermes/zoid-profile-settings.json` remains the Zoid profile settings persistence file for Zoid-specific fields.
- Saving writes edits back to the Hermes profile files/config where applicable, not just browser/local fake state.
- Zoid-launched Hermes chat sessions now insert selected provider/model and access-mode toolset constraints into the actual Hermes CLI command for normal chat prompts. Explicit terminal-style `hermes ...` commands remain untouched.
- Replaced text inputs with grounded dropdowns for timezone, communication style template, main provider, main model, reasoning effort, notification preference, voice mode, access mode, and approval mode.
- Added 5 style templates: concise, technical, executive, teacher, creative.
- Removed the visible Auxiliary models free-text control from the UI.
- Fixed overview card strong text clipping by reducing/clamping size and increasing line-height/padding.
- Added archived-session selection checkboxes plus per-session delete, Delete selected, and Delete all archived controls.

## Changed files

- `src/App.tsx`: settings UI dropdowns, archive bulk delete state/actions, runtime copy clarifying real data sources.
- `src/App.css`: overview text clipping fix and archive bulk/delete styles.
- `src/agents/hermesProfileClient.ts`: new typed settings fields for real model/template options and voice modes.
- `src-tauri/src/lib.rs`: Hermes config/memory read/write helpers, real profile hydration/persistence, CLI runtime arg wiring.
- `src-tauri/Cargo.toml` / `src-tauri/Cargo.lock`: added `serde_yaml` for Hermes `config.yaml` parsing/writing.

## How to test

- `npm run build`
- `npm run test:frontend`
- `npm run test:rust`
- Launch/relaunch `/Applications/Zoid 25.app`, open Settings, confirm dropdowns render and archived sessions have checkbox/delete controls.
- In Settings, save profile after editing memory/user/preferences/model/access fields; expected: active Hermes profile files/config update and Zoid-launched normal chat commands include selected provider/model/access constraints.

## Tests run

- `npm run build`: PASS, Vite bundle warning only.
- `npm run test:frontend`: PASS.
- `npm run test:rust`: PASS, 22 tests.

## Git info

- Branch: current working tree, not committed.
- Commit SHA: not committed.
- Diff base: existing dirty Zoid workspace had many unrelated modified/untracked files before this slice; review should focus on the files listed above.

## Frontend/backend/database notes

- Frontend: `SettingsArchive` in `src/App.tsx`; profile client shape in `src/agents/hermesProfileClient.ts`.
- Backend: Tauri commands `load_hermes_profile_settings` / `save_hermes_profile_settings` now reconcile against Hermes profile files.
- Persistence: Hermes `config.yaml`, Hermes `memories/MEMORY.md`, Hermes `memories/USER.md`, and Zoid `zoid-profile-settings.json` under active Hermes profile home.
- Archived sessions are Zoid local app sessions stored in `localStorage` under `zoid25:hermes-archived-sessions`; bulk delete mutates that same real app state.

## Reviewer focus areas

- Check no UI surface still claims fake notes for settings that now write real files/config.
- Verify `serde_yaml` writeback is acceptable for Hermes config preservation.
- Check that normal Zoid chat runtime args are inserted before `--query`, while explicit `hermes ...` commands stay exact.
- Check archived bulk-delete controls mutate the real archived sessions array and persist to localStorage.
- Check TypeScript/Rust edge cases around unknown provider/model/style template values.

## Fix cycle notes

Re-review update:
- Fixed stale user-facing copy around runtime wiring/access/approval behavior.
- Coerced dropdown-backed defaults and sanitization for reasoning and notifications to valid enum values.
- Changed Hermes soul to load/save `agent.system_prompt` in real Hermes `config.yaml` instead of only Zoid-local JSON.
- Made voice mode persistence non-lossy: STT follows `voice`, while `tts`/`off` persist in Zoid profile preference without deleting configured TTS provider.
- Normalized explicit `hermes ...` command detection with `trim_start()` before deciding whether to apply runtime profile args.
- Re-ran `npm run build && npm run test:frontend && npm run test:rust`: PASS.
