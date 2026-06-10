# Social dashboard functionality handoff

## User feedback
Page `/` / social dashboard feedback:
- `Open media URL` button is not functional.
- `Validate media` appears non-functional.
- Review report card looks fake because it shows APPROVED while the report area is empty.
- Platform rate-limit state has shown the same 24h message since yesterday; user asked how often it updates.
- Reports and event history section is collapsed/empty/not useful and needs real wiring.

## Changes made
- Frontend `src/social/socialClient.ts`
  - Added `openMavoidSocialResource(resource)` Tauri invoke wrapper.
- Frontend `src/social/SocialDashboard.tsx`
  - Replaced `window.open` with native Tauri-backed opener calls so media/report buttons work in the Tauri app.
  - Latest report and per-report items are real buttons, including local report paths.
  - Reports/event history is open by default and includes update cadence text: last read-back, next automatic Hermes/cooldown check, and immediate Check Provider API path.
  - Review card now says the verdict comes from local manifest/review file, and provides an open review report button for local report paths.
  - Removed exact forbidden visible phrase `Provider read-back` from social dashboard copy.
- Backend `src-tauri/src/lib.rs`
  - Added `mavoid_social_open_resource_inner` and command `mavoid_social_open_resource`.
  - Opens only `https://` URLs or existing local paths under the MaVoid social workspace using macOS `open`.
  - Parses all `public_media_urls` from the manifest into media assets, not only the preferred URL.
  - Reads review verdict from `review-report.md` when present, falling back to manifest.
  - Adds runtime `STATUS.json` as a report and adds concrete events for manifest creation, review verdict, public media URLs, and provider blocker.
  - Platform state now reflects `STATUS.json` timestamp/blocker in `readBackVerifiedAt` / last error fields instead of hardcoded per-platform 24h text only.
- Test `src/social/SocialDashboard.behavior.test.tsx`
  - Mocks and asserts native opener command for `Open media URL`.
  - Asserts review source copy and provider refresh cadence copy.
  - Asserts local reports are opener buttons, not disabled/fake controls.

## Critique fixes applied
- Approval timestamp is now only populated when `verdict == "APPROVED"`, never from rate-limit/media-hosted state.
- Review required fixes are parsed from manifest/report where available; otherwise UI says they were not parsed and tells the user to open the report, instead of claiming no open fixes.
- Overview health now reads the same blocker sources as post/platform state (`current_blocker` and `/proof_post/not_posted_reason`).
- Refresh cadence copy now separates provider/API checked time from local state refresh time.

## Verification run
- `./node_modules/.bin/tsx src/social/SocialDashboard.behavior.test.tsx` → passed after critique fixes.
- `npm run build` → passed (`tsc && vite build`) after critique fixes.
- `cargo check --manifest-path src-tauri/Cargo.toml` → passed after critique fixes.
- `npm run test:frontend` currently fails before the social test in `src/scaffold.test.ts` with pre-existing unrelated guard: `Composer textarea needs command mode and auto-height behavior: COMPOSER_MIN_HEIGHT` in dirty tree.

## Review focus
Please adversarially inspect:
1. Are Open media URL / report buttons now truly functional in Tauri, not browser-only?
2. Is local path opening safely scoped to the MaVoid social workspace?
3. Does the Review card avoid fake-data implications and accurately reflect local files?
4. Does the rate-limit/update-frequency explanation match the real code path?
5. Is Reports + events now useful and backed by real runtime files/events?
6. Any type, build, or regression issues introduced by the Rust/React changes?

Return verdict APPROVED or CHANGES_REQUESTED with required fixes only.