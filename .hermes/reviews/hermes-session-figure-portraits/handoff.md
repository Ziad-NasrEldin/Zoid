# Feature Handoff: Hermes session figure portraits distinctness fix

## Original request

User feedback after the prior implementation: "Ok, but I noticed that they are all the same image. They are all the same. Weren't they supposed to be different images? Or what? Because all I see right now is the same color and everything is the same, whatever."

## Implementation summary

- Root cause: the previous local SVG portrait tokens were technically unique, but the palette/geometry plus heavy blur and overlay made them visually collapse into the same low-contrast blue/ink square in the 30–50px session-tab size.
- Regenerated the 100 local session-figure SVG assets with visibly distinct archival sigils: varied silhouettes, hats/collars/crests, paper washes, accents, focal positions, and grain seeds.
- Extended `SessionFigurePortrait` metadata with `accent`, `paper`, and `focalPoint` so the CSS can vary color/focus per session in addition to the image URL.
- Reduced blur strength while preserving the soft-focus archival treatment, so icons remain branded but no longer look like repeated identical images.
- Added scaffold guards that check for exactly 100 module references, exactly 100 SVG files, 100 unique asset bodies/titles, and varied accent colors.
- While running the repo's frontend gate, two existing new test/script failures surfaced and were fixed so `npm run test:frontend` can pass again:
  - Command palette search now handles `input` events in the Happy DOM behavior test path.
  - Finder sidebar test was repaired around the already-required removal of the useless Up button.

## Changed files

- `src/agents/sessionPortraits.ts`: richer 100-item portrait metadata with accent/paper/focal point tokens.
- `public/session-figures/*.svg`: regenerated 100 visibly distinct local archival portrait/sigil SVG assets.
- `src/agents/AgentsHermesScreen.tsx`: passes portrait accent/paper/focal CSS variables to session tabs; keeps the New Session action as a plus icon; removed the file-manager Up toolbar button per existing scaffold invariant.
- `src/App.css`: stronger per-session visual differentiation with lighter blur, per-portrait accent/paper, and focused background positioning.
- `src/scaffold.test.ts`: updated portrait guard strings and added actual asset-directory uniqueness checks.
- `src/agents/CommandPalette.tsx`: added `onInput` alongside `onChange` so command filtering tests/users update reliably.
- `src/agents/AgentsHermesScreen.file-manager.test.tsx`: fixed the test's undefined/duplicated button variable and restored expand/collapse assertions.

## How to test

- `npm run test:frontend`
- `npm run build`
- `npm run tauri:build`
- Install/relaunch `/Applications/Zoid 25.app`.
- Open Agents/Hermes with several sessions and confirm the rail icons are visibly different in color/shape while still blurred/archival.

## Tests run

- `npm run test:frontend`: PASS.
- `npm run build`: PASS, Vite chunk-size warning only.
- `npm run tauri:build`: PASS, Rust dead-code warnings only for `apply_profile_runtime_args` and `prompt_with_enabled_profile_context`.
- Browser/runtime DOM check after creating multiple sessions: PASS; session tabs used different assets and different accent/paper/focal CSS variables.
- Native app relaunch: PASS; `/Applications/Zoid 25.app/Contents/MacOS/zoid` running.
- Native screenshot `/tmp/zoid-distinct-session-portraits.png`: PASS; Hermes session rail visible with multiple distinct portrait/sigil icons.

## Git info

- Branch: not asserted.
- Commit SHA: not committed.
- Diff base: working tree already contains unrelated dirty/untracked project files; review should stay scoped to the files above.

## Frontend/backend/database notes

- Frontend-only visual/data-token fix.
- No backend or database changes.
- Assets remain local/offline-safe; no runtime network dependency and no external image licensing dependency.

## Reviewer focus areas

- Confirm the 100 assets are actually visibly distinct, not merely different filenames.
- Confirm deterministic session-id mapping is preserved.
- Confirm the New Session action remains a plus icon.
- Confirm the reduced blur still fits Zoid's Japanese archival/editorial style.
- Confirm the test/build fixes are narrow and do not alter unrelated behavior beyond the existing invariants they unblock.

## Fix cycle notes

This is a fix cycle for the earlier approved session portrait feature after user visual feedback that the icons looked the same.
