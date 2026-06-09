# MaVoid Social Dashboard v1 critique

Verdict: APPROVED

## Blocking issues introduced by this feature

None found.

## Review notes

- Tauri command wiring is present and compiles: social commands are exposed at `src-tauri/src/lib.rs:6212-6239` and registered in the Tauri invoke handler at `src-tauri/src/lib.rs:6326-6331`.
- The dashboard keeps Buffer provider state explicit and does not claim that Buffer posted or scheduled anything. The UI labels the surface as a read-back/control room and shows provider blockers at `src/social/SocialDashboard.tsx:72-83`; the selected post gate reports why retry is blocked at `src/social/SocialDashboard.tsx:111-123`.
- Rate-limit fail-closed behavior is represented in the frontend view model: retries are blocked when `overview.bufferHealth.rateLimited` is true at `src/social/socialViewModel.ts:39-42`, and review/media/provider checks are required before retry eligibility at `src/social/socialViewModel.ts:43-50`.
- Backend data mapping stays truth-oriented for the current MaVoid Buffer runtime: `STATUS.json` blockers are parsed into Buffer health at `src-tauri/src/lib.rs:5755-5766`; local manifests with rate-limit status are classified as `rate_limited` rather than scheduled/posted at `src-tauri/src/lib.rs:5791-5825`; Buffer post entries are initialized as `not_created` with the rate-limit error at `src-tauri/src/lib.rs:5821-5822`.
- Potentially side-effectful actions are constrained to explicit buttons. The only Buffer API call is the explicit health-check command path (`src/social/SocialDashboard.tsx:89-92`, `src-tauri/src/lib.rs:5901-5909`). Automation mutation is limited to known creator/monitor actions and fixed job IDs (`src-tauri/src/lib.rs:5911-5921`).

## Unrelated existing blockers observed during verification

- `cargo test --manifest-path src-tauri/Cargo.toml mavoid -- --test-threads=1` fails before social tests due to existing Hermes command test callsites missing the new `AppHandle` argument, e.g. `src-tauri/src/lib.rs:7666`, `src-tauri/src/lib.rs:7712`, `src-tauri/src/lib.rs:7728`, `src-tauri/src/lib.rs:7768`, `src-tauri/src/lib.rs:7784`, `src-tauri/src/lib.rs:7823`, `src-tauri/src/lib.rs:7835`, and `src-tauri/src/lib.rs:8254`. This matches the handoff’s unrelated dirty-state note and is not introduced by the social dashboard slice.

## Verification run

- `npx tsx src/social/socialViewModel.test.ts && npx tsx src/social/SocialDashboard.behavior.test.tsx && npm run build` passed.
- `cargo check --manifest-path src-tauri/Cargo.toml` passed.
- `cargo test --manifest-path src-tauri/Cargo.toml mavoid -- --test-threads=1` failed only on the unrelated existing `AppHandle` test signature errors noted above.
