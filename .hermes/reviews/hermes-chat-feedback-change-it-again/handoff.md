# Hermes chat feedback: replace forced Japanese cues with restrained sumi-e direction

## Scope
User said: "i still dont like it, please change it" after the previous version added explicit Japanese text and a `和` seal. This slice changes direction: removes the forced/literal Japanese labels and switches to a quieter, more restrained sumi-e/ryokan feel.

## Changed files
- `src/agents/ChatComposer.tsx`
  - Replaced koto/pentatonic typing tone with quieter `bambooWaterDropPitch`, `templeBellOvertonePitch`, and `tatamiRoomDamping` shaping.
  - Slowed rate limit to `160ms` and lowered volume to `0.0036` / `0.0009`.
  - Uses sine-only, lower lowpass values, longer damped envelope for less clicky/less literal sound.
- `src/agents/AgentsHermesScreen.tsx`
  - Reverted obvious kanji labels: repo is `Repository`, dropdown fallback is `Unlinked`, button is `Files`.
  - Reverted footer markers to clear English labels: Context, Time, Model, Session.
  - Replaced the prominent `代理` header with `Quiet ink workspace` so this screen no longer relies on forced Japanese text.
- `src/App.css`
  - Removed the `和` badge pseudo-element.
  - Replaced the Hermes-screen `kana-line` styling with `.hermes-ink-line` English microcopy treatment.
  - Kept compact alignment but changed Files button to a subtle brush underline via `::after` instead of literal seal text.
  - Kept footer in the ink/washi palette without the previous forced Japanese labels.
- `src/scaffold.test.ts`
  - Updated guards to the new restrained direction and the new audio vocabulary.

## Validation
- `npm run build` passed.
- `npm run test:frontend` passed.
- Browser computed proof before the header fix confirmed:
  - repo text: `REPOSITORY / Unlinked`
  - files text: `FILES`
  - Files button pseudo `::before`: `none`
  - Files button `::after`: brush-line gradient
  - Files geometry: `92×36`
  - repo geometry: `176×36`
  - footer text: `Context`, `Time`, `Model`, `Session`
- After reviewer feedback, removed the remaining visible `代理` header and re-ran build + frontend tests successfully.

## Review focus
Critique whether this is a real change from the prior disliked version: no forced kanji labels on the Hermes screen, no `和` badge, calmer audio, compact controls preserved, no accessibility/layout regressions. Return APPROVED or CHANGES_REQUESTED with required fixes only.
