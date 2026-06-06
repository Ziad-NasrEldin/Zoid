# Feature Handoff: P1.20 workspace registry frontend integration

## Original request

Continue Zoid development using the Zoid-wide subagent workflow.

Tracker task:

- P1.20 Frontend integration: render all workspace names from real workspace registry; no hardcoded fake connected states.

## Implementation summary

- Updated `src/App.tsx` with a small pure `buildWorkspaceRegistryView` helper.
- Native status present:
  - renders only `status.workspaces`, sorted by `position`;
  - does not mix browser fallback records into successful native data;
  - preserves the prior native-empty-registry behavior.
- Browser/Tauri unavailable:
  - clearly labels static workspace records as `Browser preview fallback`.
- Loading/checking native status:
  - clearly labels temporary browser-preview data while `get_foundation_status` is loading.
- Added user-visible source/count/truth copy in:
  - sidebar footer,
  - workspace registry card,
  - inspector details.
- Kept integration states non-connected only (`not configured`, `needs permission`); no fake connected states added.
- Added small CSS styling for registry metadata/source/count badges.
- Did not implement P1.21 Today widgets, P1.22 settings/status shell, or P1.23 confirmation UI.

## Changed files

- `src/App.tsx`
  - Added registry view helper and source/count/truth UI.
- `src/App.css`
  - Added registry metadata/source/count styling using existing tokens.

## Tests run

- `npm run build`: PASS.
- Parent browser preview at `http://127.0.0.1:1420/`: PASS.
  - title: `Zoid`
  - no horizontal overflow
  - `Browser preview fallback: 14 workspaces` visible in browser mode
  - all 14 workspace buttons visible in the snapshot
- P1.20 spec review: PASS.
- P1.20 quality/UX/security review: APPROVED.

## How to test

Commands/checks:

- `npm run build`
- `npm run verify:local`
- Browser preview: `http://127.0.0.1:1420/`

Expected browser-preview behavior:

- Sidebar footer says browser preview fallback with 14 workspaces.
- Registry card title/source says `Browser preview fallback`.
- All 14 workspace names render.
- Copy makes clear fallback data is static preview data outside Tauri.
- No integration state claims connected/ready.

Expected native/Tauri behavior:

- Successful native status renders only `status.workspaces`.
- If native returns zero workspaces, the UI shows an empty native registry instead of falling back to static preview records.
- Source copy says native registry and references data returned by `get_foundation_status`.

## Git info

- Branch: `main`
- Base before this lane: `15c485e Implement P1.19 base components`
- P1.20 is uncommitted at handoff creation.

## Frontend/backend/database notes

- Frontend: `src/App.tsx`, `src/App.css` only.
- Backend: no P1.20 backend changes.
- Database: no schema changes.

## Reviewer focus areas

- Native registry path must not mix fallback records.
- Browser preview fallback must be clearly labeled.
- No fake connected/integration states.
- Native empty registry behavior preserved.
- No scope creep into P1.21/P1.22/P1.23.

## Fix cycle notes

- Spec review passed without required fixes.
- Quality review approved without required fixes.
