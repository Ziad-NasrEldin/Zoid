# Handoff: boxy sessions overflow cue

Scope:
- User disliked the rounded "More sessions below" cue on the Zoid 25 Agents/Hermes page.
- Request: make it boxy/sharp like the brand.

Changed:
- `src/App.css`
  - Added `border-radius: 0;` to `.agents-sumi-e .sessions-overflow-cue`.
  - Kept prior sumi-e styling: ink background/border, paper text, no shadow/glow, no sheen, no hover filter.
- `src/scaffold.test.ts`
  - Added a source guard requiring `border-radius: 0;` in the existing Hermes feedback polish guard list.

Verification performed:
- `npm run build` passed.
- Browser computed-style probe on `http://127.0.0.1:1420/` Agents page confirms the cue has:
  - `borderRadius: 0px`
  - `backgroundColor: rgb(13, 10, 10)`
  - `borderColor: rgb(13, 10, 10)`
  - `boxShadow: none`
  - `filter: none`

Known context:
- Working tree contains unrelated dirty files from ongoing Zoid work.
- Existing scaffold tests may fail on unrelated Settings/Profile guards; this scoped change builds and browser-verifies.
