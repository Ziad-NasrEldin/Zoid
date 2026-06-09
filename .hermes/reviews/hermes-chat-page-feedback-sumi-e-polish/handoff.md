# Hermes Chat Page Feedback: Sumi-e polish

## Scope
Addressed Page Feedback for `/` / Hermes Agents chat surface:

1. Composer typing sound should feel calmer, more Japanese-themed, sumi-e branded.
2. Hermes session stats footer colors should not feel like the website/brand identity.
3. Repository link control in topbar was too long and consumed too much space.
4. Files button should align with the left controls while still feeling special.

## Changed files in scope
- `src/agents/ChatComposer.tsx`
- `src/App.css`
- `src/scaffold.test.ts`

## Implementation summary
- Reworked `playCalmingTypingSound` from a sharper bandpass click/chime to a softer Web Audio envelope:
  - lower volume/rate
  - lowpass filter
  - short koto-like triangle pluck layered with lower sine/shakuhachi breath tone
  - subtle detune/randomization named `sumiEBrushDetune`
  - paste/history input remains silent.
- Compact Hermes Agents topbar controls:
  - status stack moves into the right header column instead of spanning the entire topbar width.
  - repository control constrained to `minmax(176px, 236px)`.
  - repository dropdown and Files button share 36px height.
  - Files button is fixed 92px wide and gets a subtle ink-stroke pseudo-element.
- Neutralized stats footer away from black/red brand treatment:
  - muted gray text, light warm-gray background, sans-serif font, no uppercase transform.
- Added scaffold guards for the new audio shape and topbar/footer feedback invariants.

## Verification run
- `npm run build` passed.
- `npm run test:frontend` is blocked by an unrelated pre-existing scaffold failure:
  - `Complete profile page is missing Codex/Hermes preference surface: Profile, Memory & Soul`
- Browser dev verification on `http://127.0.0.1:1420/`:
  - HTTP 200.
  - Agents page renders.
  - Computed geometry at browser viewport:
    - repository control: 176×36
    - Files button: 92×36
    - status panel: 164×30, aligned to same bottom edge as controls
    - stats strip background: `rgba(248, 247, 244, 0.72)`
    - stats strip color: `rgb(86, 81, 75)`
    - stats strip font: `ui-sans-serif, -apple-system, system-ui, Segoe UI, sans-serif`

## Known repository context
The worktree was already dirty before this slice, including unrelated changes in `App.tsx`, `src-tauri/src/lib.rs`, automations/code/settings areas, review folders, and new session state files. Do not treat unrelated scaffold/test failures as caused by this slice unless proven by diff.

## Review request
Please review this slice for:
- sound implementation actually matching calm Japanese/sumi-e direction without annoying clicks
- topbar compactness/alignment at the reported 1758×982 style viewport
- footer neutralization away from brand identity
- regression risk in responsive layout and CSS cascade
- whether source guards are meaningful and not brittle

Verdict format: APPROVED or CHANGES_REQUESTED with Required fixes only.
