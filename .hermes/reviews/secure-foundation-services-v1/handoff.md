# Feature Handoff: Secure Foundation Services v1

## Original request

You are a new Hermes session continuing Zoid work from a handoff. First read /private/tmp/zoid-session-handoff-2026-06-01-025349.md. Follow it exactly. Start by checking git status. Do not run npm run verify:release unless intentionally needed. If proceeding without a newer user instruction, begin the recommended Secure Foundation Services v1 slice. For any code changes, follow the feature-critique-workflow: create .hermes/reviews/secure-foundation-services-v1/handoff.md, run local verification, get critique APPROVED before claiming completion.

## Implementation summary

- Added centralized native Rust secure foundation helpers for:
  - obvious secret redaction before persistence/rendering paths;
  - safe app-support log writing with sanitized log scopes and redaction before file append;
  - documented action policy matrix evaluation with fail-closed handling for unknown action categories;
  - generic event writer helper that redacts summaries/metadata and links event targets.
- Wired the existing `foundation.ready` event through the generic event writer.
- Added native foundation status fields so the UI can truthfully show redaction, safe logging, action policy, event writer readiness, and Keychain as blocked/unverified.
- Added a small Secure foundation card in the React shell. It does not claim Keychain readiness.
- Did not run `npm run verify:release`; no DMG/package verification was intentionally performed.

## Changed files

- `src-tauri/src/lib.rs`: secure service structs/helpers, generic event writer, safe log probe, native status fields, and Rust unit tests.
- `src/App.tsx`: status types and Secure foundation status card.
- `src/App.css`: styles for secure foundation status list/policy note.
- `.hermes/reviews/secure-foundation-services-v1/handoff.md`: this review handoff.

## How to test

- `cargo test --manifest-path src-tauri/Cargo.toml`
- `npm run build`
- `npm run verify:local`
- Launch the Tauri app locally and inspect Foundation status; expected secure services:
  - Redaction: ready
  - Safe logging: ready
  - Action policy: ready
  - Event writer: ready
  - Keychain: unverified/blocked

## Tests run

- `cargo fmt --manifest-path src-tauri/Cargo.toml && cargo test --manifest-path src-tauri/Cargo.toml`: PASS; 7 Rust tests passed after R1 fix.
- `npm run build`: PASS; TypeScript and Vite production build passed after R1 fix.
- `npm run verify:local`: PASS; dependencies present, Tauri CLI found, Rust tests 7 passed, frontend build passed, local push verification passed with `--skip-package`.
- Critique initial review: `REQUEST_CHANGES` with R1 about invalid JSON risk in `events.metadata_json` redaction.

## Git info

- Branch: `main`
- Commit SHA, if committed: not committed
- Diff base, if known: `origin/main`
- Current status before review: modified `src-tauri/src/lib.rs`, `src/App.tsx`, `src/App.css`, plus this handoff file.

## Frontend/backend/database notes

- Frontend routes/components: root `App` shell now renders a Secure foundation card when native status is available; browser/Vite preview still truthfully says native status is packaged-app only.
- Backend/native endpoints/services: existing `get_foundation_status` command now includes `secure_services` readiness details. Secure helpers are native Rust functions, not a public arbitrary secret/log command surface.
- Database tables/migrations: no schema migration added. Existing `events` and `event_targets` tables are used by the new generic event writer. Existing migration compatibility test remains covered.
- Logs: safe log probe writes under `~/Library/Application Support/Zoid/logs/foundation.log` using sanitized scope and redaction.
- Keychain: intentionally not implemented or verified in this slice; UI/status says blocked/unverified.

## Reviewer focus areas

- Secret redaction coverage and whether the helper avoids raw obvious secrets in logs/events.
- Safe logging behavior: app-support path only, sanitized scope, append behavior, redaction before persistence.
- Action policy matrix fidelity to `Docs/2026-05-31-zoid-implementation-plan-v1.md` section 6 and fail-closed unknown categories.
- Generic event writer behavior: redacts summary/metadata and links targets.
- UI truthfulness: secure services shown as native status, Keychain not overstated.
- Whether native verification scope is acceptable without running the DMG/release gate.

## Fix cycle notes

Initial review returned `REQUEST_CHANGES`:

- R1 High Backend/DB/Security: `write_event` could store invalid JSON in `events.metadata_json` because metadata redaction treated JSON as plain text.

Fixes made after R1:

- Added JSON-aware `redact_metadata_json` using `serde_json::Value` recursion.
- Secret-looking JSON keys/values are replaced with `"[REDACTED]"` while preserving valid JSON.
- Invalid metadata input falls back to a valid JSON envelope containing redacted text.
- Updated generic event writer to use JSON-aware metadata redaction.
- Added regression coverage that parses stored metadata and checks SQLite `json_valid(metadata_json)=1` while asserting raw secret absence.
- Broadened action-policy coverage to every documented section-6 action category.
- Strengthened safe-log test for path traversal/scope sanitization.

Ready for re-review.
