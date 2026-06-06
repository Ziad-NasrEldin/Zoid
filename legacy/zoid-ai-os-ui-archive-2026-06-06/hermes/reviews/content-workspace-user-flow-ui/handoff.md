# Feature handoff: Content Workspace user-flow UI

## Request
Use the approved user-flow map to build the actual Content Workspace UI, specifically correcting the prior mistake where all 16 Stitch screens were collapsed into one catalog-like screen.

## Scope changed
- `src/contentWorkspace.ts`
  - Added `ContentFlowScreenId`, `ContentFlowSection`, `ContentFlowScreen`, `ContentWorkspaceFlow`.
  - Added `buildContentWorkspaceFlow()` with 16 distinct screens/states, route labels, sections, entry points, outgoing transitions, and `allScreensVisibleAtOnce: false`.
- `src/contentWorkspace.test.ts`
  - Added flow assertions for 16 distinct states, section order, modal-only `run-now-modal`, route labels, transitions, dashboard entry points, piece-detail transitions, and not rendering all screens at once.
- `src/App.tsx`
  - Replaced the prior one-page grid/catalog Content implementation with a navigable Content flow workspace.
  - Renders one `.content-flow-screen[data-content-screen]` primary screen at a time.
  - `run-now-modal` opens as a modal overlay/dialog rather than a permanent card.
  - Added concrete per-screen components for all 16 Stitch states:
    1. Autonomous Campaign Dashboard
    2. Brand Management - MaVoid
    3. New Campaign Wizard
    4. Advanced Campaign Editor
    5. Content Slot Calendar
    6. Today's Content Pipeline
    7. Content Piece Detail & Adaptations
    8. Content Editor / Override Flow
    9. Approval-Needed Queue
    10. Dry Test Report - MaVoid Daily
    11. Run Now Modal
    12. Recovery / Failure Center
    13. OmniSocials & Account Mappings
    14. Evidence & Artifact Library
    15. Agent Execution & Notifications
    16. Campaign Automation Mirror
  - Preserved a separate live native strip with real loading/error/native status handling and fail-closed publishing copy.
- `src/App.css`
  - Added `.content-flow-*` responsive layout and modal styling for the new flow UI.

## Verification run by implementer
- `npx tsx src/contentWorkspace.test.ts`: PASS
  - Output: `contentWorkspace tests passed`
- `npm run build`: PASS
  - Output included: `tsc && vite build`, `✓ 64 modules transformed.`, `✓ built in 347ms`
- Browser smoke at `http://127.0.0.1:1420`: PASS
  - Clicked Content workspace, then programmatically clicked each of the 16 screen-state buttons.
  - Result: all 16 expected titles matched.
  - Result: exactly one primary `.content-flow-screen[data-content-screen]` was visible for each primary state.
  - Result: `Run Now Modal` rendered as `[role="dialog"][data-content-screen="run-now-modal"]` while primary screen count stayed 1.
  - Console check after smoke: no captured browser errors, `contentScreens: 1`, title `Zoid`.

## Known context / caveats
- Working tree already contains unrelated prior modifications and untracked files outside this feature (Today/task/etc.). Review should focus on Content Workspace changes unless an interaction is introduced.
- This is frontend UI/state navigation only; it intentionally does not add backend publishing or live OmniSocials writes.
- The UI uses design-copy/sample content for screen surfaces and explicitly discloses that real native content state is separated below.

## Required reviewer checks
1. Confirm the implementation is not a 16-card catalog anymore.
2. Confirm only one primary Content screen is rendered at a time.
3. Confirm Run Now is a modal overlay and not a normal permanent card.
4. Confirm all 16 Stitch screen states are reachable.
5. Confirm disclosure/fail-closed copy avoids implying live publishing.
6. Confirm build/test evidence is valid or rerun as needed.
