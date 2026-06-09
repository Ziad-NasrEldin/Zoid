# MaVoid Social Dashboard v1 handoff

## Scope
Replace the old Content/OmniSocials surface in Zoid 25 with a Buffer-backed MaVoid social automation dashboard. This is a read-first control room over the existing MaVoid Buffer runtime at `/Users/ziadnasreldin/MaVoid/social-automation-buffer`.

## User goal
The user wants the OmniSocial automation flow redone around Buffer:
- 8:00 a.m. creator agent creates social content and design.
- 10:00 a.m. daily intel post scheduling/posting through Buffer.
- 6:00 p.m. second post scheduling/posting through Buffer.
- Zoid should expose this clearly and safely, using the real Buffer runtime/API state, not fake OmniSocial state.

## Implemented
- Added typed Rust/Tauri social structs and commands in `src-tauri/src/lib.rs`:
  - `mavoid_social_get_overview`
  - `mavoid_social_list_posts`
  - `mavoid_social_get_post`
  - `mavoid_social_run_buffer_health_check`
  - `mavoid_social_manage_automation`
  - `mavoid_social_validate_media_url`
- Backend reads local MaVoid Buffer artifacts and `STATUS.json`, reads Hermes cron state for known Buffer automation job ids, and shells only to the existing local Buffer health script for explicit API check.
- Added typed frontend social module:
  - `src/social/types.ts`
  - `src/social/socialClient.ts`
  - `src/social/socialViewModel.ts`
  - `src/social/SocialDashboard.tsx`
- Replaced `ContentWorkspace` with `SocialDashboard` and changed sidebar meta from `OmniSocials` to `Buffer`.
- Added dashboard CSS in `src/App.css`.
- Added tests:
  - `src/social/socialViewModel.test.ts`
  - `src/social/SocialDashboard.behavior.test.tsx`
- Added the social tests to `package.json` `test:frontend` script.

## Safety / invariants
- No automatic Buffer scheduling was added.
- Dashboard shows provider state, rate-limit blocker, public media URL, review state, and Hermes job state.
- Buffer API calls happen only via explicit `Check Buffer API` action.
- Hermes cron mutation is limited to creator/monitor action mapping, not arbitrary job deletion.
- Scheduling retry logic remains fail-closed while Buffer is rate-limited or review/media requirements are missing.

## Known current provider state
- Buffer endpoint: `https://api.buffer.com/graphql`.
- Local Buffer runtime exists at `/Users/ziadnasreldin/MaVoid/social-automation-buffer`.
- Buffer is currently rate-limited with a 24h cooldown from prior proof attempts.
- A reviewed proof media URL is visible in the dashboard data: `https://files.catbox.moe/9tix1y.png`.

## Verification already run
- `npx tsx src/social/socialViewModel.test.ts && npx tsx src/social/SocialDashboard.behavior.test.tsx` passed.
- `npm run build` passed.
- `cargo check` passed.
- `npm run tauri:build` passed and produced `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid 25.app`.

## Verification blockers / unrelated dirty state
- `npm run test:frontend` currently fails before reaching the new social tests, in `src/scaffold.test.ts`, on an existing Hermes page feedback guard: missing `grid-template-columns: max-content minmax(320px, 1fr) minmax(148px, max-content);`.
- `cargo test --manifest-path src-tauri/Cargo.toml mavoid -- --test-threads=1` compiles the whole test harness and fails in unrelated existing tests because `send_hermes_cli_message` / `send_hermes_cli_run_message` tests call older signatures missing a required `AppHandle` argument.
- Native app was built and launched, but screenshot targeting was inconclusive because WhatsApp/Finder stayed frontmost and macOS Spaces/window focus prevented reliable target-page visual proof in the captured screenshot.

## Review request
Please review only this feature slice and classify findings as:
- Required fix: correctness/safety/build issue introduced by the MaVoid social dashboard work.
- Optional polish: UI/wording/layout improvement that does not block safe delivery.
- Unrelated existing blocker: dirty-tree or pre-existing test failures not introduced by this feature.

Focus on:
1. Tauri command safety and data truthfulness.
2. Whether any UI implies Buffer posted/scheduled when it did not.
3. TypeScript/Rust serialization casing and command payload casing.
4. Whether read-only/rate-limited/fail-closed state is clear enough.
5. Whether replacing the Content workspace is acceptable for this scope.
