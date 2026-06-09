# Critique Report: Hermes Session Historical-Figure Portrait Icons

## Verdict: APPROVED

## Scope reviewed

- `.hermes/reviews/hermes-session-figure-portraits/handoff.md`
- `src/agents/sessionPortraits.ts`
- `public/session-figures/*.svg`
- `src/agents/AgentsHermesScreen.tsx`
- `src/App.css`
- `src/scaffold.test.ts`

## Verification performed

- Read the feature handoff and reviewed only the scoped implementation for the Hermes session historical-figure blurred portrait icons.
- Inspected the deterministic portrait pool and hashing helper in `src/agents/sessionPortraits.ts`.
- Verified the asset directory contains exactly 100 SVG files, with 100 unique referenced asset names, no missing referenced files, no extra SVG files, and 100 unique SVG contents:
  - `python3 - <<'PY' ...` — PASS.
- Reviewed `AgentsHermesScreen.tsx` integration: existing sessions receive `--session-portrait`/`--session-age-shade`; the new-session button remains a plus icon; active/open/rename/archive flows remain wired through the existing session tab controls.
- Reviewed `App.css` portrait treatment for expanded and compact rails, including the required blurred/saturated/contrast pseudo-element treatments and compact-mode exclusion for `.session-new-button`.
- Reviewed `src/scaffold.test.ts` source guards for the portrait pool count and visual treatment.
- Ran frontend guard tests:
  - `npm run test:frontend` — PASS.
- Ran production frontend build:
  - `npm run build` — PASS, with Vite's pre-existing large chunk warning only.

## Findings

- The implementation satisfies the scoped request: Hermes session tabs now use a stable local pool of 100 Japanese historical-figure portrait-token assets, selected deterministically from each session id.
- The 100 referenced assets are present under `public/session-figures/`, are locally bundled/offline-safe, and are unique files/contents.
- The UI integration is appropriately scoped to existing session tabs. The new-session control still renders the plus icon and does not receive the compact rail background portrait treatment.
- The visual treatment is restrained and consistent with the Zoid 25 Japanese editorial direction: blue/ink/paper palette, hard-edged rail geometry, and blurred archival portrait/sigil surfaces rather than crisp decorative thumbnails.
- The added source guards cover the critical invariants for this feature without requiring unrelated dirty-tree cleanup.
- No blocking regressions were found in the scoped review.

## Required fixes

None.

## Non-blocking notes

- The assets are stylized archival SVG portrait tokens rather than externally sourced historical likeness images. This matches the handoff's stated licensing/network tradeoff and is acceptable for this scoped implementation.
