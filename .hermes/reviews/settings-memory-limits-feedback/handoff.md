# Settings memory limits / tab animation / connection panel feedback

## User feedback
From Page Feedback `/` at viewport 1758×982:
1. Settings > Memory & soul: add a way to adjust the memory lens maximum characters currently shown as 4,100 from the overview card. User wants to increase/decrease the max memory size.
2. Settings nav tabs: clicking tabs is rough; subheading/subtext vanishes and returns abruptly. Make it smooth in both directions.
3. Agents/Hermes connection panel: the right half is empty; make the card fit its content.

## Relevant implementation changes only
The working tree already contained many unrelated edits. Review the following intended changes only:

- `src/App.tsx`
  - Added `NumberProfileKey`, `updateNumberSetting`, and `renderNumberField`.
  - In the `activeSettingsSection === "memory"` tab, added `profile-memory-budget-card` with two number inputs:
    - `memoryCharLimit` labeled “Hermes memory maximum”
    - `userCharLimit` labeled “User profile maximum”
  - Card displays combined `{memoryBudgetLimit} chars max` so it matches the overview Memory card math.

- `src-tauri/src/lib.rs`
  - `save_real_hermes_sources` now writes `memory.memory_char_limit` and `memory.user_char_limit` into Hermes `config.yaml` using the typed settings values.
  - `save_hermes_profile_settings_inner` rejects zero memory/user profile character limits.

- `src/App.css`
  - Agents sumi-e topbar status grid first column changed to `max-content`; `.agents-sumi-e .connection-panel` changed from `width: 100%` to `width: max-content; max-width: 100%`.
  - Profile tab buttons now transition background/color/shadow/transform, and tab text transitions opacity/transform/color.
  - Active profile tab panels animate in via `profile-tab-panel-enter`, with reduced-motion fallback.
  - Added styling for `profile-memory-budget-card`, `profile-grid--memory-limits`, and numeric inputs.

- `src/scaffold.test.ts`
  - Added source guards for the memory limit controls, persisted YAML keys, profile tab animation, and compact connection panel sizing.

## Verification already run
- `npm run test:frontend` passed.
- `npm run build` passed.
- `npm run test:rust` passed: 76 passed, 1 ignored.

## Review focus
Please do an adversarial critique for this feedback slice:
- Are the memory limit controls real/persisted and not display-only?
- Does the save path preserve Hermes config shape and write the intended YAML keys?
- Are zero/invalid character limits handled safely?
- Are the tab animations likely to smooth the reported vanish/reappear without breaking accessibility or reduced-motion?
- Does the connection panel shrink without causing topbar overflow at the reported viewport?
- Call out Required fixes only for regressions/blockers. Verdict must be APPROVED or CHANGES_REQUESTED.
