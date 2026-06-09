# MaVoid Social Dashboard in Zoid 25 — Content Page Completion Tracker

Updated: 2026-06-09
Status: APPROVED by feature critique for the content-page completion gate.

Scope completed in this tracker: the Zoid 25 Content page must let Ziad view created social/content posts and their generated designs directly inside the Content page, with enough detail to inspect review, media, provider/platform state, reports, events, and safe actions.

## Completion proof

- [x] RED test was added first for missing image/design previews.
  - Observed failure: `.social-media-preview img` count was 0, expected 2.
- [x] GREEN implementation renders real image previews.
- [x] Focused social tests pass.
  - `npx tsx src/social/socialViewModel.test.ts`
  - `npx tsx src/social/SocialDashboard.behavior.test.tsx`
- [x] Production frontend build passes.
  - `npm run build`
- [x] Rust backend check/tests pass.
  - `cargo check --manifest-path src-tauri/Cargo.toml`
  - `cargo test --manifest-path src-tauri/Cargo.toml --lib --bins -- --test-threads=1`
  - Result: 76 passed, 0 failed, 1 ignored.
- [x] Frontend test suite exits successfully.
  - `npm run test:frontend`
- [x] Production bundle contains the missing surfaces.
  - `img` found in `dist/assets/workspace-content-Bv0w7fxm.js`
  - `social-media-preview` found
  - `Reports + events` found
  - `Platform state` found
  - `Temporary media host` found
  - `Pause monitor` found
  - `Validate media` found
- [x] Browser smoke opened the Content page on the running local Vite instance and confirmed the new Content dashboard shell/actions render. Native bridge data is unavailable in browser preview, as expected.
- [x] Feature critique completed.
  - Handoff: `.hermes/reviews/content-page-tracker-completion/handoff.md`
  - Critique: `.hermes/reviews/content-page-tracker-completion/critique.md`
  - Verdict: APPROVED

## User-visible Content page requirements

### Product shell

- [x] Content page routes to the MaVoid social operations dashboard.
- [x] Dashboard copy is provider/tool agnostic in visible UI.
- [x] Zoid sumi-e/ink visual language is preserved.
- [x] Content page owns a scrollable viewport-height surface inside the fixed Zoid shell.
- [x] Content page exposes sections for Overview / Queue / Post Detail / Media / Reports.

### Created content/design viewing

- [x] Generated media/designs render as actual `<img>` previews in the Content page.
- [x] Multiple media assets render as a visible gallery/grid.
- [x] Public media URLs remain visible as metadata, not as the only viewing mechanism.
- [x] Local/path fallback is present when no public URL exists.
- [x] Open public media URL action is available.
- [x] Media validation action is available.
- [x] Temporary media host warning is visible.
- [x] Tests assert preview images exist and use the validated public URL.

### Selected post detail

- [x] Full caption/content is visible.
- [x] Date is visible.
- [x] Slot type is visible.
- [x] Topic/news item is visible.
- [x] Status/schedule gate is visible.
- [x] Review verdict is visible.
- [x] Reviewer identity, when available, is visible.
- [x] Review report path/action is visible.
- [x] Required fixes/history are visible.
- [x] Media metadata is visible: content type, dimensions, bytes, validation state, provider, checked time.
- [x] Public URL/open action is visible.
- [x] Schedule/retry state is visible and gated.
- [x] Manual resolution is visible but evidence-gated.

### Queue / platform state

- [x] Publishing queue lists posts from the local social workspace.
- [x] Queue rows show lifecycle state.
- [x] Queue rows show title and platforms.
- [x] Detail view shows post date and slot type.
- [x] Detail view shows platform-specific provider state.
- [x] Detail view shows provider post IDs when present.
- [x] Detail view shows channel IDs/display names when present.
- [x] Detail view shows local scheduled time when present.
- [x] Detail view shows UTC scheduled time when present.
- [x] Detail view shows read-back verification timestamp when present.
- [x] Detail view shows published URL action when available.
- [x] Empty provider state has a clear empty state.

### Automation overview/actions

- [x] Overall provider/read-back state is visible.
- [x] Active blocker/rate-limit status is visible.
- [x] Creator job ID and state are visible.
- [x] Creator next run is visible.
- [x] Monitor job ID and state are visible.
- [x] Cooldown next run is visible.
- [x] Refresh read-back action is available.
- [x] Provider health check action is available.
- [x] Run 8:00 creator action is confirmation-gated.
- [x] Pause/resume creator actions are available.
- [x] Pause monitor action is available.
- [x] Resume monitor action is available.
- [x] Latest report action/metadata is visible.

### Provider health / secret safety

- [x] Provider endpoint is shown only as configured/not configured.
- [x] HTTP status is visible when available.
- [x] Last probe time is visible when available.
- [x] Credential state is shown only as safe booleans.
- [x] Access token value is never rendered.
- [x] Organization secret value is never rendered.
- [x] Rate-limit/cooldown state is visible.
- [x] Health action warns when provider is cooling down.

### Reports / events / history

- [x] Reports section is visible.
- [x] Report list renders generation/review/provider report references when available.
- [x] Review report action is visible.
- [x] Latest report action is visible.
- [x] Event history/timeline is visible.
- [x] Event actor, severity, message, timestamp, and evidence path are visible when available.
- [x] Empty report/event states are clear.
- [x] Report/event UI is secret-safe.

### Safety gates

- [x] UI does not claim scheduled/posted unless provider state says so.
- [x] Retry scheduling is locked unless review/media/provider gates pass.
- [x] Reviewer approval is required before retry is available.
- [x] Valid public direct media URL is required before retry is available.
- [x] Existing provider state blocks unsafe retry.
- [x] Run creator asks for confirmation.
- [x] Retry schedule asks for confirmation when enabled.
- [x] Manual posted/resolution action is evidence-gated and disabled until implemented safely.
- [x] No text-only fallback scheduling action was added.

### Tests

- [x] Test asserts actual design preview images render.
- [x] Test asserts multiple media previews render.
- [x] Test asserts preview `src` uses validated public media URL.
- [x] Test asserts review/report details render.
- [x] Test asserts required fixes render.
- [x] Test asserts provider/platform state renders.
- [x] Test asserts provider post IDs render.
- [x] Test asserts event history renders.
- [x] Test asserts HTTP status and credential booleans render.
- [x] Test asserts temporary media warning renders.
- [x] Test asserts monitor pause and media validation controls render.
- [x] Test asserts visible copy is provider/tool agnostic.

## Files changed

- [x] `src/social/SocialDashboard.tsx`
- [x] `src/social/SocialDashboard.behavior.test.tsx`
- [x] `src/App.css`
- [x] `.hermes/reviews/content-page-tracker-completion/handoff.md`
- [x] `.hermes/reviews/content-page-tracker-completion/critique.md`
- [x] `.hermes/plans/2026-06-09-mavoid-social-dashboard-tracker.md`

## Critique result

- [x] Critique verdict: APPROVED.
- [x] Required fixes: none.
- [x] Non-blocking caveat: the current real proof artifact maps to one media asset with multiple public URL mirrors, while the UI/test path supports multiple media assets.
- [x] Non-blocking caveat: internal backend/type names still use Buffer terminology in places, but visible UI copy is neutralized.

## Final completion definition

- [x] Created content/designs are now viewable inside the Content page.
- [x] Post details, media, reports, events, review state, and provider/platform state are visible.
- [x] Unsafe scheduling/manual-resolution paths are gated.
- [x] Tests/build/backend checks pass.
- [x] Feature critique is APPROVED.

Conclusion: content-page tracker completion gate is finished and approved.
