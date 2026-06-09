# Settings accepted sumi-e restore handoff

Scope: restore the Settings page back to the previously designed Brain-derived sumi-e version after the operational/cobalt shell had replaced it.

User request: "restore the settings page too as well" in the context of restoring the old accepted sumi-e design, not redesigning from scratch.

Files in scope:
- `src/App.tsx`
  - Restored Settings root shell class to `settings-sumi-e`.
  - Restored hero title to `Profile, Memory & Soul`.
  - Restored explanatory hero copy.
  - Restored `.settings-ink-mark` DOM motif.
  - Kept the existing Settings tabs/archive safety behavior.
- `src/App.css`
  - Restored the old accepted sumi-e sizing/treatment for Settings hero, kana, title, reference line, ink mark, profile card, and paper/ink/red-seal palette.
  - Removed the inactive `.settings-operational-shell` override block so the old operational/cobalt treatment does not linger in the active Settings styling path.

Design invariants to verify:
- Root class: `settings-archive-shell profile-page-shell profile-page-shell--organized settings-sumi-e`.
- No active `settings-operational-shell` / `settings-control-room` shell.
- Hero title: `Profile, Memory & Soul`.
- `.settings-ink-mark` exists and renders with ink/red-seal mark.
- Settings uses paper/ink/red-seal palette and Brain-derived serif typography.
- Operational/cobalt compact header is not active.
- Existing Settings archive safety/modal behavior remains intact.

Verification already run:
- `npm run test:frontend && npm run build` passed.
- Browser computed styles on live Settings page verified:
  - root class is `settings-sumi-e`.
  - heading is `Profile, Memory & Soul`.
  - `.settings-ink-mark` exists.
  - title font size ~76.8px.
  - ink mark width ~166px.
  - no horizontal overflow.
- Browser visual inspection showed old sumi-e Settings surface: Japanese kana, large serif title, ink mark, red-seal/paper palette, profile card, and no operational/cobalt/control-room shell.

Review request: confirm this restores the old Settings sumi-e design while preserving Settings behavior and archive safety. Return APPROVED or REQUIRED_FIXES with exact evidence.