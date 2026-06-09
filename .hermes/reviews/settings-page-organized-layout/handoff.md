# Feature Handoff: Settings page organized layout

## Original request

"i want you to organize the whole settings page now, it needs better layouting, use the impecable layout skill to do so please its really scroll heavey and a very long page , we dont want that"

## Implementation summary

- Converted the long Settings page into a compact organized workspace with seven tabs: Identity, Memory & soul, Models, Providers, Tools, Safety, Archive.
- Reworked the layout again after visual inspection so the section tabs sit horizontally, the real summary cards sit below them, and the active section content is not clipped by the save status.
- Kept the real Hermes/profile wiring intact: config/memory/user profile/provider/archive controls are the same controls, just reorganized.
- Providers tab now uses the same section panel wrapper as the other tabs so layout sizing is consistent.
- Preserved truthful UI copy: no fake data was introduced; existing real-source labels remain.
- Used the available Impeccable-style product UI craft workflow (`product-ui-craft-workflow`) and its truth-first desktop UI checklist.

## Changed files

- `src/App.tsx`: reorganized `SettingsArchive` into tablist/tabpanel layout and wrapped Providers as a real settings section.
- `src/App.css`: added compact settings shell, horizontal section tabs, bounded/visible workspace flow, summary card grid, compact hero styling, and responsive fallbacks.
- `src/scaffold.test.ts`: existing source guards cover the tabbed settings layout primitives.

## How to test

- Run frontend/build/Rust checks.
- Open Settings in browser/native app.
- Confirm the Settings page no longer renders every section stacked in one long form.
- Confirm all seven tabs are visible/clickable and only the active section renders.
- Confirm the active panel does not overlap the save status.
- Confirm no horizontal overflow.

## Tests run

- `npm run test:frontend`: PASS.
- `npm run build`: PASS.
- `npm run test:rust`: PASS, 27 tests.
- `git diff --check -- src/App.tsx src/App.css src/scaffold.test.ts`: PASS.
- Browser DOM check on `http://127.0.0.1:1420`: PASS. Seven tabs rendered, all clicked, no horizontal overflow, no active-panel/main-save-status overlap. Providers has an internal provider-status note inside its own panel, which is expected.
- Browser visual inspection: PASS after fix. Initial horizontal-tab pass had the active panel clipped/overlapped by the save bar; CSS was corrected and rechecked.

## Git info

- Branch: `main`.
- Commit SHA: not committed.
- Diff base: dirty repo with many pre-existing unrelated modifications/untracked review artifacts. Review should scope to the three files above and not require cleanup of unrelated work.

## Frontend/backend/database notes

- Frontend routes/components: `SettingsArchive` in `src/App.tsx`.
- Backend endpoints/services: unchanged.
- Database tables/migrations: unchanged.

## Reviewer focus areas

- Does the layout materially reduce the top-level scroll burden compared with all sections stacked?
- Are all previous settings sections still reachable through the tabs?
- Does any tab content overlap the page-level save status?
- Did the layout preserve real wiring and avoid fake/static placeholders?
- Are ARIA tablist/tabpanel semantics reasonable enough for this custom control?
- Are there any obvious CSS overflow or responsive issues caused by the horizontal tab/summary layout?

## Fix cycle notes

- Fixed a visual regression found during browser inspection: the horizontal tab layout initially let active section content overflow underneath the save status because the settings shell/form/workspace rows were height-constrained. Changed the settings shell to content-sized rows, changed the workspace/content overflow to visible, and verified no page-level status overlap.
- Fixed Providers tab sizing by wrapping it with the same `profile-section profile-section--active` panel class used by the other tabs.
