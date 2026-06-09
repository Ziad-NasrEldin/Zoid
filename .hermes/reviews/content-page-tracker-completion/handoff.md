# Content page tracker completion handoff

Feature: Zoid 25 Content page MaVoid social dashboard completion.

Scope requested: finish the content page tracker checklist completely, with the core missing gap being that created content/designs were not actually viewable from the Content page.

Files changed in this batch:
- src/social/SocialDashboard.tsx
- src/social/SocialDashboard.behavior.test.tsx
- src/App.css
- .hermes/plans/2026-06-09-mavoid-social-dashboard-tracker.md will be updated after critique approval.

Implemented user-visible behavior:
- Content page now renders actual design/image previews using <img> elements for every selected post media asset.
- Multiple media assets render as a gallery/carousel-like grid with public URL/open/validate actions.
- Selected post detail now shows caption, date, slot, topic, schedule/retry gate, manual evidence gate, review verdict/report/required fixes, provider/platform state, provider post IDs, channel IDs, local/UTC times, read-back timestamps, reports, and event history.
- Automation toolbar now includes refresh, provider health check, confirmed run creator, pause/resume creator, pause monitor, resume monitor, validate media, and latest report actions.
- Provider/automation side panel now shows monitor job/state, HTTP status, last probe, safe credential-present booleans, cooldown, provider endpoint configured flag, and latest report path.
- UI copy remains provider/tool agnostic: visible dashboard copy avoids Buffer framing except backend command names/tests.
- Sumi-e visual treatment extended for tabs, media preview cards, platform cards, report/event lists.

TDD evidence:
- Added failing assertions first to src/social/SocialDashboard.behavior.test.tsx.
- RED observed: test failed because .social-media-preview img count was 0, expected 2.
- GREEN after implementation: social dashboard behavior tests passed.

Verification already run:
- npx tsx src/social/socialViewModel.test.ts && npx tsx src/social/SocialDashboard.behavior.test.tsx && npm run build
  Result: passed; Vite built dist/assets/workspace-content-Bv0w7fxm.js.
- cargo check --manifest-path src-tauri/Cargo.toml && cargo test --manifest-path src-tauri/Cargo.toml --lib --bins -- --test-threads=1
  Result: passed; 76 passed, 0 failed, 1 ignored.
- npm run test:frontend
  Result: exit code 0; social dashboard behavior tests passed within suite output.

Known caveats for reviewer to inspect:
- Some tracker backend expansion items are represented as visible gated/unavailable UI rather than fully implemented backend commands, because this batch prioritized making all created content/designs viewable and exposing state/actions safely from Content page.
- Health check remains clickable during rate-limit as an intentional read-back; UI title warns about cooldown rather than hard-disabling because existing behavior test requires the health command to be callable.
- Retry schedule and manual posted actions remain safely gated/disabled or message-only; no unreviewed scheduling side effect was added.

Review requirements:
- Verify that the actual requirement “viewing the content and designs created within the content page” is now satisfied by rendered image previews and detail sections.
- Verify visible surfaces map to the tracker requirements: media preview, post detail, reports/events/history, provider/platform state, monitor controls, safe credential booleans, latest report, media validation.
- Treat missing visible previews/detail panels as Required Fixes.
- Check for provider/secret exposure and unsafe scheduling side effects.
- Verdict must be APPROVED or REQUEST_CHANGES with Required fixes.
