# Settings visual inconsistencies handoff

Scope: Fix visual inconsistencies reported on Zoid Settings page screenshot. This pass intentionally touches only the Settings page first-fold visual system in `src/App.css` plus one short Settings reference-line copy string in `src/App.tsx`.

Important dirty-tree note: the repo already had broad unrelated dirty/untracked work before this task. Do not review the entire `git diff` as this task's scope. Review the Settings-specific edits listed below.

User complaint / observed problems:
- Settings hero was oversized; settings controls started too far below the fold.
- Active profile summary card stretched vertically with mostly empty space and clipped/truncated content.
- Settings tab row looked cramped/inconsistent and could clip the last tab on narrower content widths.
- Overview metric cards retained older dashboard/card rhythm instead of the sumi-e Settings visual rhythm.
- Fields/dropdowns still looked more utilitarian than the new sumi-e Settings surface.
- Reference line could truncate after compacting.

Implemented changes:
- `src/App.css`
  - Reduced `.settings-sumi-e` top padding so Settings first fold starts higher.
  - Reworked `.settings-sumi-e .settings-hero` from three equal visual zones into compact `copy + card` layout.
  - Moved ink mark to a quiet absolute accent; adjusted `settings-mark-reveal` keyframes so the transform does not fight absolute centering.
  - Reduced kana/title/body/reference typography and line-height.
  - Clamped hero description to one line to preserve the compact control-room first fold.
  - Made `.profile-hero-card` fixed/compact with `align-self: center` and `min-height: 152px` instead of stretching to hero height.
  - Compact heading/tabs/profile panel gaps.
  - Changed tab grid from `minmax(124px, 1fr)` to `minmax(104px, 1fr)` so all seven tabs are visible in the tested viewport.
  - Tightened overview metrics spacing and type.
  - Applied Settings-scoped field label/input/dropdown style alignment with sumi-e tokens.
  - Added a more specific `.settings-sumi-e .profile-hero--compact p.settings-reference-line` override so the reference line escapes the older `54ch` max-width rule.
- `src/App.tsx`
  - Shortened Settings reference line copy to: `Hermes profile · memory · providers · archive`.

Verification already run:
- `npm run test:frontend` — passed.
- `npm run build` — passed.
- `npm run tauri:build` — passed; bundle built at `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.
- Launched rebuilt bundle with `open '/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app'`.
- Browser visual smoke on `http://127.0.0.1:1420/` Settings page:
  - hero compacted to ~248px high instead of ~600px.
  - Settings heading starts around y=319, tab row around y=410, overview row around y=473 at the tested browser viewport.
  - all seven tabs visible.
  - reference line fully visible after specificity fix.
  - profile summary card no longer stretches to hero height.
  - browser console showed 0 JS errors after final visual smoke.

Reviewer focus:
1. Confirm first-fold Settings composition no longer feels vertically bloated.
2. Confirm active profile card, ink mark, and reference line are visually balanced and not clipped.
3. Confirm all seven tabs remain visible at the tested viewport and no horizontal scrollbar/clipping is obvious.
4. Confirm metrics row and input/dropdown styling are consistent with the Settings sumi-e system.
5. Check CSS cascade specificity around `.profile-hero--compact p:not(.kana-line)` vs `.settings-reference-line`.

Known non-scope:
- Existing global shell/sidebar and code-workspace diffs in `src/App.css` / `src/App.tsx` predate this task.
- Feedback/annotation floating controls are external overlay, not Settings UI.
