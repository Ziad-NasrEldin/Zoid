# Feature Handoff: Zoid app foundation

## Original request

User said: "go ahead and do it" after the pre-PRD implementation completed and the reported blocker was: "Rust/Cargo is not installed, so the next real implementation step needs Rust/Tauri setup before initializing the app shell."

Relevant PRD path:
`/Users/ziadnasreldin/Zoid/Docs/2026-05-31-zoid-implementation-plan-v1.md`

Implemented Task 3 plus the first migration/foundation slice from Phase 1:
- Initialize Tauri/React/TypeScript app foundation.
- Create app shell/sidebar/Today placeholder backed by the native workspace registry when running inside Tauri.
- Add SQLite migration runner and core workspace/event tables.
- Create visible/app-support folders from native startup and command paths.

## Implementation summary

- Installed Rust/Cargo via Homebrew.
- Initialized a Tauri 2 + React + TypeScript app in `/Users/ziadnasreldin/Zoid` while preserving existing Docs.
- Rebranded generated app to Zoid.
- Added Apple-style shell UI with sidebar and all required top-level workspaces.
- Added truthful integration states; no fake connected integrations.
- Added native Tauri command `get_foundation_status` returning paths, migration state, event/workspace counts, and full workspace records.
- Frontend renders the sidebar/workspace registry from `status.workspaces` inside Tauri; browser preview uses an explicit UI-only fallback.
- Native startup hook calls `ensure_foundation()` so folders/SQLite migrations run even before frontend invoke completes.
- Native foundation creates:
  - `~/Zoid/` starter folders
  - `~/Library/Application Support/Zoid/logs`
  - `~/Library/Application Support/Zoid/zoid.sqlite`
- Added file-based SQLite migrations under `src-tauri/migrations/` for:
  - `schema_migrations`
  - `workspaces`
  - `events`
  - `event_targets`
- Seeds all 14 required core workspaces:
  - Today, Tasks, Notes, Agents, Code, Content, Automations, Business, Products, Files, Browser, Inbox, Calendar, History.
- Writes/backfills one idempotent `foundation.ready` event linked through `event_targets`.
- Added Rust unit tests for fresh migration/workspace seeding, idempotent linked foundation event, and upgrading an existing early foundation database.
- Configured Tauri bundle target to macOS `.app` only because DMG bundling failed in this environment; `.app` build succeeds.

## Changed files

- `package.json`: Zoid package metadata and scripts.
- `package-lock.json`: npm dependency lockfile.
- `index.html`: title changed to Zoid and default Vite icon removed.
- `src/App.tsx`: Zoid shell UI, native-backed workspace registry rendering, integration truth states, native foundation status invocation, disabled placeholder action.
- `src/App.css`: Apple-style shell styling.
- `src/main.tsx`: React entrypoint.
- `src/vite-env.d.ts`: Vite types.
- `vite.config.ts`: Tauri/Vite config.
- `tsconfig.json`, `tsconfig.node.json`: TypeScript config.
- `src-tauri/Cargo.toml`: Zoid app metadata and `rusqlite` dependency.
- `src-tauri/Cargo.lock`: Rust dependency lockfile.
- `src-tauri/src/lib.rs`: Tauri command/startup hook, folder creation, migration runner, workspace/event seeding, compatibility migration helpers, Rust tests.
- `src-tauri/src/main.rs`: updated crate call to `zoid_lib::run()`.
- `src-tauri/migrations/0001_foundation.sql`: initial foundation schema migration.
- `src-tauri/migrations/0002_event_schema_backfill.sql`: event target/index migration for existing databases.
- `src-tauri/tauri.conf.json`: Zoid product config/window, app-only bundle target.
- `src-tauri/build.rs`, `src-tauri/capabilities/default.json`, icons: generated Tauri scaffold.

Generated build artifacts exist under `dist/` and `src-tauri/target/` because local verification was run. They should be treated as build output, not source.

## How to test

From `/Users/ziadnasreldin/Zoid`:

```bash
cargo --version
rustc --version
npm run test:rust
npm run build
npm run tauri:build
/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid.app/Contents/MacOS/zoid
```

Expected:
- Rust/Cargo are installed.
- Rust tests pass.
- Vite build passes.
- Tauri build produces `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid.app`.
- Launching the packaged app/binary creates local folders and DB.
- SQLite shows migration version `2`, 14 workspaces, 1 foundation event, and 1 event target.

Browser preview for UI-only inspection:

```bash
npm run dev
# open http://127.0.0.1:1420
```

Expected browser preview:
- Title is Zoid.
- Sidebar lists all 14 workspaces.
- Clicking a workspace changes active heading.
- Placeholder action is disabled.
- Browser preview explicitly says native foundation status is available only inside the packaged Tauri app.

## Tests run

- `brew install rust`: PASS. Installed Rust/Cargo 1.95.0.
- `cargo --version && rustc --version`: PASS.
  - `cargo 1.95.0 (f2d3ce0bd 2026-03-21) (Homebrew)`
  - `rustc 1.95.0 (59807616e 2026-04-14) (Homebrew)`
- `npm install`: PASS. 74 packages audited, 0 vulnerabilities.
- `cargo fmt --manifest-path src-tauri/Cargo.toml && npm run test:rust`: PASS.
  - 3 Rust tests passed:
    - `migrations_seed_core_workspaces`
    - `foundation_event_is_idempotent_and_linked`
    - `migrations_upgrade_existing_foundation_database`
- `npm run build`: PASS. TypeScript + Vite build succeeded.
- `npm run tauri:build`: PASS. Produced `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid.app`.
- Direct packaged binary launch:
  - `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid.app/Contents/MacOS/zoid`: PASS. Process stayed running with no stderr output during probe.
- SQLite verification after packaged binary launch: PASS.
  - migration version: `2`
  - workspace count: `14`
  - workspace IDs: `today,tasks,notes,agents,code,content,automations,business,products,files,browser,inbox,calendar,history`
  - events: `1`
  - event_targets: `1`
  - foundation event actor/workspace fields: `system|zoid|today`
- Browser preview at `http://127.0.0.1:1420`: PASS for UI snapshot and all 14 workspace labels visible.

## Git info

- `/Users/ziadnasreldin/Zoid` is not currently a git repository, so no branch/commit/diff is available.

## Frontend/backend/database notes

Frontend:
- React app shell in `src/App.tsx`.
- Native/Tauri path renders workspace sidebar from `status.workspaces` returned by `get_foundation_status`.
- Browser preview fallback contains the same 14-workspace set but is explicitly marked UI-only if native invoke fails.
- Placeholder module action is disabled to avoid fake functionality.

Native/backend:
- Tauri command: `get_foundation_status`.
- Tauri startup hook runs `ensure_foundation()`.
- Uses `rusqlite` with bundled SQLite.
- Uses `$HOME` to derive:
  - `/Users/ziadnasreldin/Zoid`
  - `/Users/ziadnasreldin/Library/Application Support/Zoid`

Database:
- SQLite DB path: `/Users/ziadnasreldin/Library/Application Support/Zoid/zoid.sqlite`
- Tables: `schema_migrations`, `workspaces`, `events`, `event_targets`.
- Current migration version after verification: `2`.
- Existing early local DBs are compatibility-upgraded for missing workspace and event columns.

## Known limitations / risks

- This is not the full Secure Foundation yet.
- Keychain test path is not implemented because the previous pre-PRD spike left Keychain unresolved/blocked by command guard.
- Redaction service, safe logging service, action policy evaluator, confirmation framework, entity link service, and settings shell are not implemented yet.
- No real CLI/PTTY UI session yet; this is app foundation only.
- Browser preview cannot invoke Tauri native commands outside Tauri runtime; it intentionally shows UI-only status.
- DMG bundling failed earlier; current bundle target is `.app` only.
- Existing old local DBs keep the legacy `events.actor` column as compatibility residue while adding planned event fields.
- No git repo exists yet.

## Reviewer focus areas

- Verify Task 3 requirements: app shell, sidebar, all workspaces, no fake integration success states.
- Verify frontend uses native/SQLite-backed workspace records in Tauri runtime.
- Verify file-based migrations and existing-DB compatibility are acceptable for this foundation slice.
- Verify event foundation now includes planned fields and `event_targets`.
- Verify packaged app/native startup evidence, not only browser preview evidence.
- Verify generated app config/build scripts are sane.
- Check if `.gitignore` sufficiently excludes `node_modules`, `dist`, and `src-tauri/target` if/when this becomes a git repo.

## Fix cycle notes

Re-review update after first `REQUEST_CHANGES` critique:
- Fixed R1 by adding missing Tasks, Calendar, and History to the native registry and browser fallback, with Rust test asserting the full ordered workspace set.
- Fixed R2 by returning full `workspaces` from `get_foundation_status` and rendering the sidebar/registry from native status in Tauri runtime.
- Fixed R3 by adding planned event fields (`timestamp`, `actor_type`, `actor_id`, `workspace_key`), creating `event_targets`, linking the foundation event, and testing linked event idempotence.
- Fixed R4 by moving schema SQL into versioned migration files under `src-tauri/migrations/` and adding an existing-DB upgrade test.
- Added native startup `ensure_foundation()` hook and direct packaged binary verification because browser preview alone does not prove native setup.
- Re-ran Rust tests, TypeScript/Vite build, Tauri app build, direct packaged binary launch, SQLite row/field checks, and browser preview snapshot.
