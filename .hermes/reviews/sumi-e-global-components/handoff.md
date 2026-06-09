# Sumi-e global components pass

Scope: Review and update Zoid 25 global/shared UI surfaces to follow the new sumi-e design system.

Changed areas:
- `src/App.css`
  - Added sumi-e root tokens and legacy Kujoyama aliases to ink/paper/seal values.
  - Reworked global focus ring to seal-red sumi-e treatment.
  - Reworked shared `GlobalDropdown` chrome to paper/ink/seal with squared form language.
  - Renamed/styled the old blue rail as `ink-rail`.
  - Reworked sidebar/rail/global nav/status styling toward ink wash + controlled seal accent.
  - Swapped old blue/yellow/green hex and rgba leaks in global/shared chrome to sumi-e ink/seal equivalents.
  - Removed obsolete `settings-control-room` / `settings-operational-shell` CSS overrides.
- `src/App.tsx`
  - Settings shell now renders `settings-sumi-e`.
  - Added the settings ink mark element used by the sumi-e settings design.
  - Global rail renders `ink-rail` instead of `blue-rail`.
- `src/scaffold.test.ts`
  - Added regression guard against stale global chrome leaks: `settings-control-room`, `settings-operational-shell`, `blue-rail`, old blue/yellow hexes, and old blue rgba literals.
- `src/sessionPortraits.ts`, `src/agents/sessionPortraits.ts`, `src/agents/AgentsHermesScreen.tsx`
  - Updated session portrait accent defaults from old blue to sumi-e seal/ink accents.

Verification already run:
- Search under `src` for `settings-control-room|settings-operational-shell|blue-rail|#3558a2|#294984|#e7edfa|#b8c8ea|#fde863|rgba(53, 88, 162|rgba(53,88,162` returned only the scaffold regression guard itself.
- `npm run test:frontend` exits 0.
- `npm run build` exits 0.
- Browser check at `http://127.0.0.1:1420` confirmed:
  - `hasInkRail: true`
  - `hasBlueRail: false`
  - `hasSettingsSumiE: true`
  - `hasControlRoom: false`
  - `hasOperationalShell: false`
  - `rootKujoBlue: #0d0a0a`
  - settings class: `settings-archive-shell profile-page-shell profile-page-shell--organized settings-sumi-e`
  - settings font resolves to serif stack
  - settings dropdown exists and renders paper background

Review request:
- Confirm the first critique Required fixes are resolved.
- Focus on global/shared shell surfaces only.
