# Complete Profile Page — Critique Handoff

## Feature slug
complete-profile-page

## User request
Implement a complete Zoid 25 profile/preferences page using selected Codex app/profile/preferences and Hermes desktop/Hermes Agent findings. Include editable Hermes memory and Hermes soul, user name/preferences, profile/personal settings, and run a separate line-by-line review after implementation.

## Scope implemented
- Expanded Settings page from a simple archive/settings form into a complete Profile, Memory & Soul page.
- Added Codex-inspired preferences:
  - user identity/onboarding role/timezone/communication style
  - model/provider/reasoning defaults
  - access/sandbox mode and approval mode
  - trusted projects/default workdir
- Added Hermes-inspired preferences:
  - Hermes memory and soul text areas
  - personality/response mode
  - toolsets, MCP servers, plugins, enabled skills
  - gateway platforms, notifications, voice preferences
  - toggles for memory/user profile/auto-context/web/browser/terminal/file/cron/checkpoints/redaction
- Added persistent profile settings client with native Tauri invoke path and localStorage fallback.
- Expanded Rust `HermesProfileSettings` and default values with serde default/camelCase compatibility.
- Added scaffold assertions for the complete profile page and native persistence fields.
- Added CSS for complete profile sections, overview cards, toggles, responsive grids, and active profile summary.

## Files to review line-by-line
- `src/App.tsx`
  - `SettingsArchive` and profile page rendering logic.
- `src/App.css`
  - `.profile-*` styles and interaction with existing Settings/archive styles.
- `src/agents/hermesProfileClient.ts`
  - Type shape, defaults, sanitizer, Tauri load/save, localStorage fallback.
- `src-tauri/src/lib.rs`
  - `HermesProfileSettings`, defaults, load/save helpers/commands, tests.
- `src-tauri/Cargo.toml`
  - `serde_json` dependency.
- `src/scaffold.test.ts`
  - profile page coverage assertions.

## Verification already run
- `npm run build` — passed.
- `cargo check` in `src-tauri` — passed.
- `npm test` — passed:
  - frontend scaffold test passed.
  - Rust tests passed: 15 passed, 0 failed.

## Required review focus
1. TypeScript correctness and stale/unsafe casts in `SettingsArchive`.
2. Native/browser persistence correctness and malformed localStorage handling.
3. Rust serde/default compatibility for older profile JSON.
4. Whether any profile fields accidentally look like secrets storage. They should remain preferences/config only; no credentials.
5. UX completeness: page should be visually usable, responsive, and not bury Save.
6. Line-by-line review for regressions caused by broad existing file changes.
7. Produce verdict: APPROVED or Required fixes. If required fixes exist, list exact file/line/patch guidance.
