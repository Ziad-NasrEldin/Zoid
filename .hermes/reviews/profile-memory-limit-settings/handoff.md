# Profile memory limit settings fix handoff

## Scope
Fix Settings → Memory & soul → Memory lens limits so users can enter ordinary integer limits such as `2800` without browser native validation blocking save, and verify profile fields persist.

## Root cause
The number inputs used `min={1}` and `step={100}`. HTML number validity treats valid step values as `min + n*step`, so with `min=1` valid values were `1, 101, 201, ...`; `2800` was invalid and the browser showed “Enter a valid value”.

## Changes
- `src/App.tsx`
  - Changed profile number inputs to `step={1}` and added `inputMode="numeric"`.
- `src/scaffold.test.ts`
  - Added source guard for `step={1}` and `inputMode="numeric"` on the profile limit inputs.
- `src-tauri/src/lib.rs`
  - Extended `hermes_profile_settings_preserve_real_yaml_shapes` to save `memory_char_limit: 2800`, `user_char_limit: 1700`, and assert the saved Hermes `config.yaml` contains those values alongside `agent.system_prompt`/soul persistence.

## Verification run
- `npm run test:rust` — PASS, 79 passed, 1 ignored.
- `npm run build` — PASS, TypeScript + Vite production build succeeded.
- Browser smoke on existing Vite dev server `http://127.0.0.1:1420/`:
  - Opened Settings → Memory & soul.
  - Entered `2800` in Hermes memory maximum.
  - Verified DOM: `step: "1"`, `valid: true`, `validationMessage: ""`, budget displayed `4400 CHARS MAX`.
  - Entered soul text `Browser smoke soul value`, submitted save, verified fallback save status and localStorage contained `memoryCharLimit: 2800`, `userCharLimit: 1600`, and the soul text.
  - Reloaded and verified values persisted in browser fallback.
- `npm run test:frontend` — BLOCKED by an existing unrelated scaffold failure: `Composer textarea needs command mode and auto-height behavior: COMPOSER_MIN_HEIGHT`. This is outside this feature slice and appears tied to existing dirty composer changes, not the profile settings patch.

## Notes for reviewer
The repo had substantial pre-existing dirty changes before this fix. Review only the feature slice above. Do not include unrelated sidebar/chat/social diffs in this verdict unless they directly affect Settings profile memory limits.
