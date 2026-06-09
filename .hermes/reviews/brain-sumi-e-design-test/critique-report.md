# Critique Report: Brain-only Gen-M / sumi-e design-system pilot re-review

## Verdict

APPROVED

## Summary

The latest fix resolves the prior blocker. The requested frontend test command now exits 0, including the formerly stale `src/scaffold.test.ts` check. The Brain-only sumi-e treatment remains scoped, source inspection matches the handoff, build passes, and live browser geometry did not show the prior overlap/overflow issues on the available preview.

## What was changed

- `src/scaffold.test.ts` now accepts the intentional scoped Brain layout contract instead of requiring the obsolete unscoped `.brain-workspace-shell { ... }` selector. Evidence: `src/scaffold.test.ts:61` checks `.brain-sumi-e` or `.brain-workspace-shell:not(.brain-sumi-e)` for `grid-template-rows: auto auto auto minmax(0, 1fr)`, `grid-auto-rows: auto`, `align-content: start`, and also asserts `.brain-sumi-e .brain-link-panel` has `position: relative`.
- `src/brain/BrainWorkspace.tsx` renders the Brain page with `className="brain-workspace-shell brain-sumi-e"`, operational provenance copy, in-flow hero actions, and no visible `.brain-design-strip`. Evidence: `src/brain/BrainWorkspace.tsx:65-67`.
- `src/App.css` keeps legacy Brain styles gated behind `.brain-workspace-shell:not(.brain-sumi-e)` and defines the pilot under `.brain-sumi-e`. Evidence: legacy gate starts at `src/App.css:1179`; pilot block starts at `src/App.css:1211`; hero grid/actions/reference/mark rules appear at `src/App.css:1284-1402`; panel overflow/reduced motion/responsive rules appear at `src/App.css:1403-1471`.
- `src/brain/BrainWorkspace.behavior.test.tsx` covers scoped opt-in, operational provenance, removal of the annotation strip, safe Apple Notes import boundaries, conflict-blocked extraction, and Hermes non-execution copy. Evidence: `src/brain/BrainWorkspace.behavior.test.tsx:150-219`.

## Required fixes

None.

## Improvements

| ID | Priority | Area | Suggestion | Why it matters |
|----|----------|------|------------|----------------|
| I1 | Low | Test/UX | Add a true narrow-viewport automated geometry or screenshot test for the Brain hero at <=560px. | Current source rules and forced-width checks are reassuring, but an automated viewport-level guard would prevent future mobile/narrow regressions. |

## Tests performed

- `PATH=/Users/ziadnasreldin/.hermes/node/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin npm run test:frontend -- --run src/brain/BrainWorkspace.behavior.test.tsx`: PASS, exit code 0. Output showed the frontend script ran `tsx src/scaffold.test.ts && ... && tsx src/brain/BrainWorkspace.behavior.test.tsx ...` and completed successfully.
- `PATH=/Users/ziadnasreldin/.hermes/node/bin:/opt/homebrew/bin:/usr/local/bin:/usr/bin:/bin:/usr/sbin:/sbin npm run build`: PASS, exit code 0. Output: `tsc && vite build`; Vite transformed 1783 modules and built `dist/index.html`, CSS, and JS. Only the existing chunk-size warning was emitted.
- Source search for copied/external design assets in the relevant Brain source/CSS: PASS. `src/brain` and `src/App.css` did not contain Behance, Magnific, Awwwards, Lottie, CDN, or external URL references relevant to this pilot; the only Brain hit was the behavior test’s local happy-dom URL.
- Browser preview at `http://127.0.0.1:1420/`, Brain route: PASS for feasible geometry checks. Observed `.brain-workspace-shell brain-sumi-e`; `.brain-design-strip` absent; hero action computed `position: static`; reference line `white-space: normal`, `overflow-wrap: anywhere`, and `scrollWidth === clientWidth`; document had no horizontal overflow; ink mark did not overlap hero copy; actions did not overlap copy, reference line, or status.
- Forced 430px shell-width stress check in browser console: no document horizontal overflow, no ink/copy overlap, no action/reference/status overlap, and reference line still did not overflow. Note: this is not a true viewport resize, so it is weaker than a real narrow viewport test.

## Tests still needed

None required for approval. Optional: add automated true-narrow viewport coverage as noted in I1.

## Dev-agent instructions

No required fixes remain. Feature gate is approved.
