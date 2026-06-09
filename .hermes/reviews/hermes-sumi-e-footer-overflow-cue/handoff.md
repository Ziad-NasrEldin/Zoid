# Handoff: Hermes sumi-e footer + sessions overflow cue

Scope:
- Address Page Feedback on `/` Agents/Hermes page:
  1. `footer.chat-stats-strip` colors did not match the sumi-e design system.
  2. `button.sessions-overflow-cue` used the old blue/white button treatment and a bugged sheen/glow effect.

Changed:
- `src/App.css`
  - Scoped `.agents-sumi-e .chat-stats-strip` to sumi-e tokens:
    - `var(--agents-ink-black)` text
    - `var(--agents-pale-rule)` border
    - paper wash gradient background
    - `var(--agents-serif-body)` typography
  - Scoped stat spans and glyph labels to sumi-e ink tokens.
  - Added `.agents-sumi-e .sessions-overflow-cue` override:
    - ink-black background and border
    - paper text
    - no box-shadow/glow
    - disabled pseudo-element sheen with `.agents-sumi-e .sessions-overflow-cue::before { display: none; }`
    - hover/focus filter removed.
- `src/scaffold.test.ts`
  - Updated source guards to require sumi-e footer styles and disabled overflow cue sheen.
  - Updated stale stats-strip copy guards to match the current Japanese/glyph stats markup already present in `AgentsHermesScreen.tsx`.

Verification performed:
- `npm run test:frontend` passed.
- `npm run build` passed with no CSS minify warnings after fixing the stat span token block.
- Browser computed-style proof on `http://127.0.0.1:1420/`, Agents page:
  - footer: background `linear-gradient(90deg, rgba(255, 255, 255, 0.9), rgba(250, 250, 250, 0.76))`, border `rgb(237, 237, 237)`, text `rgb(13, 10, 10)`, font `source-han-serif-japanese...`, weight `400`.
  - injected overflow cue probe: background `rgb(13, 10, 10)`, border `rgb(13, 10, 10)`, text `rgb(255, 255, 255)`, box-shadow `none`, filter `none`, `::before` display `none`, size `42x30`.

Known context:
- Repo had many pre-existing modified/untracked files before this task. Review only this scoped CSS/test change, not unrelated dirty tree work.
