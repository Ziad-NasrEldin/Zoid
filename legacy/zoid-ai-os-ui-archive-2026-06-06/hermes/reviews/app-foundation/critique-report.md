# Feature Critique Report: App Foundation Re-review

Verdict: APPROVED

## Scope reviewed

Reviewed the current Zoid app-foundation implementation after the required fix cycle against:

- `/Users/ziadnasreldin/Zoid/Docs/2026-05-31-zoid-implementation-plan-v1.md`
- `/Users/ziadnasreldin/Zoid/.hermes/reviews/app-foundation/handoff.md`

Specific re-review focus:

- R1: all required workspaces are present: Today, Tasks, Notes, Agents, Code, Content, Automations, Business, Products, Files, Browser, Inbox, Calendar, History.
- R2: frontend sidebar is backed by the native/SQLite workspace registry in Tauri runtime.
- R3: event schema includes `timestamp`, `actor_type`, `actor_id`, `workspace_key`, and `event_targets`, with tests.
- R4: migration-file-based runner exists under `src-tauri/migrations`, with tests and compatibility for the prior early DB shape.
- Native packaged app evidence exists, not only browser-preview evidence.

## Commands run

From `/Users/ziadnasreldin/Zoid`:

```bash
npm run test:rust && npm run build && npm run tauri:build
```

Observed output summary:

- Rust tests passed: 3 passed, 0 failed.
  - `tests::foundation_event_is_idempotent_and_linked ... ok`
  - `tests::migrations_seed_core_workspaces ... ok`
  - `tests::migrations_upgrade_existing_foundation_database ... ok`
- TypeScript/Vite build passed.
  - `✓ 31 modules transformed.`
  - `✓ built in 281ms` for direct `npm run build`.
  - Tauri `beforeBuildCommand` also rebuilt successfully.
- Tauri release build passed.
  - Built application at `/Users/ziadnasreldin/Zoid/src-tauri/target/release/zoid`.
  - Bundled packaged app at `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid.app`.

Native packaged app probe:

```bash
/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid.app/Contents/MacOS/zoid
```

Observed:

- Packaged app binary started and remained running during the probe.
- No stderr/stdout errors were emitted during the probe.
- I terminated the process after verifying startup side effects.

SQLite verification after packaged app launch:

```bash
APP='/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid.app'
DB='/Users/ziadnasreldin/Library/Application Support/Zoid/zoid.sqlite'
test -d "$APP"
test -x "$APP/Contents/MacOS/zoid"
test -f "$DB"
sqlite3 "$DB" <<'SQL'
.headers off
.mode list
select 'migration_version=' || coalesce(max(version),0) from schema_migrations;
select 'workspaces=' || count(*) from workspaces;
select 'workspace_ids=' || group_concat(id, ',') from (select id from workspaces where enabled=1 order by position);
select 'events=' || count(*) from events;
select 'event_targets=' || count(*) from event_targets;
select 'event_schema=' || group_concat(name, ',') from pragma_table_info('events') where name in ('timestamp','actor_type','actor_id','workspace_key');
select 'foundation_event=' || actor_type || '|' || actor_id || '|' || workspace_key || '|' || type from events where type='foundation.ready' limit 1;
SQL
```

Observed output:

```text
migration_version=2
workspaces=14
workspace_ids=today,tasks,notes,agents,code,content,automations,business,products,files,browser,inbox,calendar,history
events=1
event_targets=1
event_schema=timestamp,actor_type,actor_id,workspace_key
foundation_event=system|zoid|today|foundation.ready
```

Git status:

```bash
git status --short || true
```

Observed output:

```text
fatal: not a git repository (or any of the parent directories): .git
```

## Files inspected

- `src/App.tsx`
- `src-tauri/src/lib.rs`
- `src-tauri/migrations/0001_foundation.sql`
- `src-tauri/migrations/0002_event_schema_backfill.sql`
- `src-tauri/tauri.conf.json`
- `package.json`
- `/Users/ziadnasreldin/Library/Application Support/Zoid/zoid.sqlite` via SQLite queries

## Re-review findings

### R1: all required workspaces are present

Status: PASS

Evidence:

- Native registry in `src-tauri/src/lib.rs` now defines all 14 required workspaces in the expected order:
  - today
  - tasks
  - notes
  - agents
  - code
  - content
  - automations
  - business
  - products
  - files
  - browser
  - inbox
  - calendar
  - history
- Browser fallback registry in `src/App.tsx` also contains the same 14 labels/IDs for UI-only preview mode.
- Rust test `migrations_seed_core_workspaces` asserts the full ordered workspace ID list, not just a count.
- Packaged-app SQLite verification returned `workspaces=14` and the full expected `workspace_ids` sequence.

This resolves the previous missing Tasks, Calendar, and History issue.

### R2: frontend sidebar backed by native/SQLite registry in Tauri runtime

Status: PASS

Evidence:

- `get_foundation_status` returns `workspaces: Vec<WorkspaceRecord>` from native Rust.
- Native `ensure_foundation()` runs migrations, seeds the workspace table, then calls `list_workspaces(&connection)` to return rows from SQLite.
- `src/App.tsx` invokes `get_foundation_status` through Tauri and uses `status.workspaces` when available:
  - `const workspaces = status?.workspaces.length ? status.workspaces : fallbackWorkspaces;`
- The sidebar `<nav>` renders from that `workspaces` collection.
- Browser fallback remains present, but the error message explicitly marks browser preview as UI-only when native status is unavailable.

This resolves the previous issue where the UI was only using a hardcoded frontend array while native/SQLite had a separate registry.

### R3: event schema and event_targets are implemented with tests

Status: PASS

Evidence:

- `src-tauri/migrations/0001_foundation.sql` creates `events` with:
  - `id`
  - `type`
  - `timestamp`
  - `actor_type`
  - `actor_id`
  - `workspace_key`
  - `summary`
  - `severity`
  - `source`
  - `metadata_json`
  - `created_at`
- `src-tauri/migrations/0001_foundation.sql` creates `event_targets` with:
  - `event_id`
  - `entity_type`
  - `entity_id`
  - `relation_type`
- `src-tauri/migrations/0002_event_schema_backfill.sql` creates `event_targets` and relevant indexes for upgrade compatibility.
- `write_foundation_event` inserts/backfills a `foundation.ready` event with:
  - `actor_type = system`
  - `actor_id = zoid`
  - `workspace_key = today`
  - linked `event_targets` row for workspace `today`.
- Rust test `foundation_event_is_idempotent_and_linked` verifies a single idempotent foundation event and one linked event target.
- SQLite verification after packaged launch returned:
  - `event_schema=timestamp,actor_type,actor_id,workspace_key`
  - `foundation_event=system|zoid|today|foundation.ready`
  - `event_targets=1`

This resolves the previous event-foundation schema gap.

### R4: migration-file-based runner and existing DB compatibility

Status: PASS

Evidence:

- Versioned SQL migration files exist under `src-tauri/migrations/`:
  - `0001_foundation.sql`
  - `0002_event_schema_backfill.sql`
- Rust `MIGRATIONS` references those files through `include_str!`, and `run_migrations` applies pending versions while recording them in `schema_migrations`.
- Fresh migration behavior is covered by `migrations_seed_core_workspaces`.
- Prior early DB compatibility is covered by `migrations_upgrade_existing_foundation_database`, which creates the older schema shape, runs the migration path, seeds the full registry, and verifies event backfill/event target creation.
- Packaged-app SQLite verification returned `migration_version=2`.

This resolves the previous inline-only migration implementation concern.

### Native packaged app evidence

Status: PASS

Evidence:

- `npm run tauri:build` completed and produced `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid.app`.
- The packaged binary at `/Users/ziadnasreldin/Zoid/src-tauri/target/release/bundle/macos/Zoid.app/Contents/MacOS/zoid` is executable and launched successfully during the review.
- The packaged binary startup path ran native foundation setup as evidenced by the app-support SQLite DB containing migration version 2, all 14 workspaces, the foundation event, and event target.

This is sufficient native evidence for this app-foundation slice; the review is not relying only on browser preview.

## Acceptance checklist

- Commit status: PASS/NOT_APPLICABLE; project directory is not currently a git repository, so no branch/commit/diff is available.
- Build status: PASS; `npm run build` and `npm run tauri:build` succeeded.
- Test status: PASS; Rust tests passed, 3 passed / 0 failed.
- Manual verification performed: PASS; packaged `.app` binary launched and SQLite side effects were verified.
- Known blockers: none for the app-foundation fix scope.
- Screens/routes/pages checked: source-level UI review of `src/App.tsx`; native runtime backing verified through packaged app and SQLite state. No browser screenshot was taken in this re-review.
- Exact command output summary: included above.
- Feature critique verdict: APPROVED.

## Non-blocking notes / risks for later foundation work

- `tauri.conf.json` still has `app.security.csp` set to `null`; acceptable for this scaffold slice, but it should be revisited during security hardening.
- Path derivation uses `$HOME` manually. This works locally, but a centralized path service or Tauri path APIs may be preferable later.
- The compatibility migration specifically covers the observed early foundation DB shape. More exotic partial/corrupt schemas may still fail and should eventually show a safe migration error UI.
- The current migration file runner is compile-time enumerated with `include_str!`, not dynamic directory discovery. That is acceptable for this slice because schema changes are now file-based and versioned, but future migration process conventions should be documented.
- The app-foundation slice does not implement the full secure foundation yet: Keychain, redaction, safe logging, action policy evaluator, confirmation framework, entity link service, and settings shell remain later work per the handoff.

## Final verdict

APPROVED

The required fixes are resolved. The current implementation satisfies the reviewed app-foundation gate: it builds as a Tauri/React/TypeScript app, launches as a packaged macOS `.app`, creates native folders/app-support SQLite state, records migration version 2, seeds all 14 required workspaces, renders the Tauri runtime sidebar from native/SQLite-backed workspace rows, uses a planned event schema with linked `event_targets`, and has Rust tests covering fresh migration, idempotent event linkage, and existing early DB upgrade compatibility.
