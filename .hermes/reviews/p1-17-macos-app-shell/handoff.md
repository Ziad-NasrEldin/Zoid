# Feature Handoff: P1.17 macOS-first app shell

## Original request

Continue Zoid development using the Zoid-wide subagent workflow.

Tracker task:

- P1.17 Frontend: polished macOS-first app shell with sidebar, toolbar/header, split-view content, no SaaS/cyberpunk styling.

## Implementation summary

- Reworked the existing React shell into a polished macOS-first desktop layout.
- Added a translucent fixed sidebar with macOS window controls, workspace glyphs, active state, and truthful preview/native footer status.
- Added a toolbar/header with workspace title, disabled search placeholder, and app status badges.
- Added a split-view main area with primary workspace overview and inspector/details rail.
- Preserved truthful native/fallback behavior:
  - `get_foundation_status` still drives native status.
  - Native `status.workspaces` is used whenever native status succeeds, even if the registry is empty.
  - Fallback workspaces are used only while checking native status or in browser preview/error mode.
  - Empty native registries show neutral empty-state copy instead of masking with fallback data.
- Removed vague/fake action copy and kept unavailable actions disabled/truthful.
- Avoided SaaS/cyberpunk styling: restrained Apple-like materials, system font, soft shadows, muted palette, and desktop app structure.

## Changed files

- `src/App.tsx`
  - Restructured the app shell into sidebar, toolbar, split primary pane, and inspector pane.
  - Added workspace glyphs, status tones, registry-source labels, and empty native registry handling.
  - Kept native foundation invocation and truthful browser preview fallback.
- `src/App.css`
  - Rebuilt styling for macOS-first shell: translucent sidebar, window controls, toolbar, split view, cards, inspector, badges, empty states, focus styles, and desktop-safe overflow.

## How to test

Commands:

- `npm run build`
- `npm run verify:local`
- Browser preview: start `npm run dev`, open `http://127.0.0.1:1420/`, verify:
  - title is `Zoid`;
  - sidebar renders workspace navigation;
  - toolbar/header renders;
  - primary content + inspector rail render;
  - no horizontal overflow at desktop app width;
  - disabled search/action states remain truthful;
  - browser preview says native status is preview/fallback only.

## Tests run

- `npm run build`: PASS. TypeScript + Vite production build completed successfully.
- Independent P1.17 spec review: initial SPEC GAP found around native empty registry masking.
- Fix applied for native empty registry masking.
- Independent P1.17 spec re-review: PASS.
- Independent P1.17 quality/security review: APPROVED.
- Browser preview at `http://127.0.0.1:1420/`: PASS for project identity, shell structure, no horizontal overflow, truthful preview/fallback copy, and visually macOS-first layout.

Pending before final completion:

- Final critique gate.
- `npm run verify:local` after final critique/fixes.
- Tracker update and commit.

## Git info

- Branch: `main`
- Current committed base before P1.17: `41b8bc2 Implement P1.16 Tauri bridge commands`
- P1.17 is currently uncommitted at handoff creation.

## Frontend/backend/database notes

- Frontend routes/components:
  - Root React app shell in `src/App.tsx`.
  - Styles in `src/App.css`.
- Backend endpoints/services:
  - No backend changes.
  - Existing Tauri command `get_foundation_status` remains the status source.
- Database tables/migrations:
  - No database changes.

## Reviewer focus areas

- Confirm P1.17 scope only: polished macOS-first shell, not P1.18/P1.19/P1.20/P1.21 implementation scope creep.
- Confirm no SaaS/cyberpunk visual language.
- Confirm no fake connected states or fake integrations.
- Confirm native workspace registry is not masked after native success with an empty array.
- Confirm disabled search/action states are truthful and accessible enough for current shell phase.
- Confirm browser preview evidence is not overstated as native/Tauri packaged verification.

## Fix cycle notes

- Spec reviewer found that `status?.workspaces.length ? status.workspaces : fallbackWorkspaces` masked a native empty registry.
- Fix changed workspace source logic so native success always uses `status.workspaces`; fallback is limited to checking/browser preview states.
- Empty native registry now renders neutral empty-state copy and safe null active workspace handling.
- Re-review verdict: PASS.
