# Hermes chat page feedback: stronger Japanese sumi-e feel

## Scope
User said the previous Japanese/sumi-e polish still did not feel Japanese enough. This slice strengthens the Japanese identity of the Hermes chat page controls, stats strip, and typing sound while keeping the layout compact.

## Changed files
- `src/agents/ChatComposer.tsx`
  - Replaced generic randomized koto pitch with explicit `japanPentatonicInScale` note choices.
  - Lowered rate/volume further (`128ms`, `0.0048`, `0.0012`).
  - Softened filter/envelope for a calmer shakuhachi-breath + koto-pluck feel.
- `src/agents/AgentsHermesScreen.tsx`
  - Repository control now visibly uses Japanese copy: `接続`, `未接続`.
  - Files button visibly uses `書類`, keeps English for screen-reader fallback.
  - Stats strip uses Japanese markers: `余白`, `圧縮`, `時`, `模型`, `座`.
- `src/App.css`
  - Added Japanese serif treatment for repo label and Files button.
  - Added a small seal-like `和` mark on the Files button.
  - Kept repo and Files compact/aligned: browser check showed repo `176×36`, Files `92×36`.
  - Stats strip stays neutral/washi-like: `rgba(248,247,244,0.72)` and `rgb(86,81,75)`.
- `src/scaffold.test.ts`
  - Added source guards for `japanPentatonicInScale`, Japanese visible labels, seal mark, and stats markers.

## Validation
- `npm run build` passed.
- Browser computed proof on Agents page:
  - repo text: `接続 / REPO / 未接続`
  - Files text: `書類 / FILES`
  - Files pseudo seal: `和`
  - Files font: Japanese serif stack
  - repo geometry: `176×36`
  - Files geometry: `92×36`
  - stats text includes `余白`, `圧縮`, `時`, `模型`, `座`
  - stats bg: `rgba(248,247,244,0.72)`
  - stats color: `rgb(86,81,75)`
- `npm run test:frontend` still failed before this slice's later tests due unrelated existing scaffold/sidebar guard: `.blue-rail::before`.

## Review focus
Please critique whether this actually addresses the user complaint that the previous controls/audio/footer still did not feel Japanese enough, without over-broad restyling. Required fixes only if something is broken, fake, inaccessible, or still not visibly/audibly tied to Japanese/sumi-e direction.
