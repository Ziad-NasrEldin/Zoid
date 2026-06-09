# Critique Report: Brain link-panel removal

Verdict: APPROVED

Scope reviewed:
- `.hermes/reviews/brain-link-panel-removal/handoff.md`
- `src/brain/BrainWorkspace.tsx`
- `src/brain/BrainWorkspace.behavior.test.tsx`

Findings:
- The reported `Link Apple Notes folder` / `safe read/import` setup panel is no longer rendered by `BrainWorkspace.tsx`.
- `BrainWorkspace.tsx` no longer imports or calls the folder listing/linking client functions, and no dropdown/link-panel state or handlers remain in this component.
- The stale no-source empty-state copy from the prior review has been removed; the empty state now directs users to create the protected `Zoid Brain` folder.
- Remaining Brain panels still render behind the `hasSources` path: Sources, Brain Inbox, Sync Conflicts, Task Candidates, and Clarifying Sessions.
- The behavior test includes explicit absence assertions for `Link Apple Notes folder` and `safe read/import`, while still exercising sync, extraction control rendering, candidate selection, and clarifying-session behavior.

Verification:
- Searched `src/brain` for removed panel copy and related stale phrases. The removed visible copy only appears in regression absence assertions; generic `Apple Notes folder` copy remains only in current accepted Brain creation/hero text.
- Searched `src/brain` for list/link UI symbols. The list/link client helpers still exist in `brainClient.ts`, but are not referenced by `BrainWorkspace.tsx`.
- Ran focused test:
  - `npm exec -- tsx src/brain/BrainWorkspace.behavior.test.tsx`
  - Result: PASS, exit code 0.

Notes:
- No blocking issues found for the scoped Brain link-panel removal.
