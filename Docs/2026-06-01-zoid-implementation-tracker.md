# Zoid Implementation Tracker

Source plan: `/Users/ziadnasreldin/Zoid/Docs/2026-05-31-zoid-implementation-plan-v1.md`

Purpose: durable execution tracker for building the actual Zoid app. Each phase includes planning, database, backend/native, Tauri bridge, frontend, tests, manual verification, local verification, and critique/review gates.

Status legend: `[ ]` pending, `[~]` in progress, `[x]` complete, `[!]` blocked.

Non-negotiable gates for every implementation slice:
- Real data or truthful empty/unconfigured states only.
- SQLite/app-support persistence where required.
- Events for meaningful actions.
- Action policy for consequential actions.
- Secret redaction for logs/events/UI/errors.
- Local verification output recorded.
- Native/macOS manual notes where automation cannot verify.
- Feature critique workflow approved before calling the slice done.
- Use `npm run verify:local` for routine gates; do not run `npm run verify:release` unless intentionally checking a release/DMG.

---

## Phase 0 — Spikes and Architecture Decision

- [x] P0.01 Read implementation plan and PRD; confirm first objective and non-negotiable rules.
- [x] P0.02 PTY/CLI runtime spike doc exists with result, prototype evidence, constraints, and fallback recommendation.
- [x] P0.03 Browser/WebView spike doc exists with result, supported/unsupported capture behavior, and fallback recommendation.
- [x] P0.04 Native macOS services spike doc exists with Keychain, notifications, EventKit, app-support findings, and fallback recommendation.
- [x] P0.05 ADR for Tauri vs Electron exists and references actual spike findings, consequences, and revisit triggers.
- [x] P0.06 Re-check spike docs + ADR before relying on them for deep product implementation.
  - 2026-06-01 re-check: all three spikes and ADR are Partial/Accepted-with-constraints, not a blank check. Proceed with Tauri, but keep PTY UI streaming, Browser screenshot/data-store, Keychain, EventKit, and notification click routing as staged verification gates.

---

## Phase 1 — Secure Foundation

- [x] P1.00 Git hygiene: preserve and commit already-approved Secure Foundation Services v1 working-tree changes.
  - 2026-06-01 preserved in commit after inspecting diff/review artifacts and rerunning `npm run verify:local` successfully.
- [x] P1.01 Discovery: inspect current app foundation, Tauri commands, UI state, package scripts, tests, review artifacts, DB/app-support behavior.
  - 2026-06-01 inventory: `src-tauri/src/lib.rs` owns current foundation command, app-support/visible-folder creation, migrations, workspace seed, event writer, redaction, safe logging, action policy, and Rust tests. `src/App.tsx` is a single-shell React UI with fallback workspace registry and truthful native-status preview behavior. `scripts/verify-local.sh --skip-package` is the routine gate behind `npm run verify:local`; release/DMG gate remains `npm run verify:release`. Review artifacts include approved `secure-foundation-services-v1` plus earlier app/packaging/local-verification reviews. Current DB schema is only foundation workspaces/events/event_targets; deeper Phase 1 repository/settings/entity-link services remain pending.
- [x] P1.02 Backend/native: app-support path service for `~/Library/Application Support/Zoid` with logs, database, config, safe directory creation.
  - 2026-06-01 implemented typed app-support path service/status for root, logs, database, and config paths with idempotent/non-destructive directory creation. Tests run: `cargo test --manifest-path src-tauri/Cargo.toml app_support` (3 passed), `cargo test --manifest-path src-tauri/Cargo.toml` (10 passed).
  - 2026-06-01 fix-cycle: added explicit `database_parent` status/path field and rejected symlinked managed app-support directories using `symlink_metadata`, including symlink-to-directory and symlink-to-file tests. Tests run: `cargo test --manifest-path src-tauri/Cargo.toml app_support` (5 passed), `cargo test --manifest-path src-tauri/Cargo.toml` (12 passed), `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `npm run verify:local`.
  - 2026-06-01 fix-cycle: reject pre-existing symlinked managed files before SQLite opens `zoid.sqlite` or `write_safe_log` appends to `<scope>.log`; added Unix regression tests documenting the portable pre-open `symlink_metadata` guard. Tests run: `cargo test symlink` (4 passed), `cargo test` (14 passed), `cargo clippy --all-targets -- -D warnings`, `npm run verify:local`.
- [x] P1.03 Backend/native: visible user folder creation at `~/Zoid` with starter directories and non-destructive behavior.
  - 2026-06-01 implemented typed visible-user path service/status for `~/Zoid` and starter directories with idempotent/non-destructive directory creation, while preserving app-support separation. Rejects file and symlink conflicts for managed visible directories. Tests run: `cargo test --manifest-path src-tauri/Cargo.toml visible_user` (5 passed), `cargo test --manifest-path src-tauri/Cargo.toml` (19 passed), `cargo clippy --manifest-path src-tauri/Cargo.toml --all-targets -- -D warnings`, `npm run verify:local`.
- [x] P1.04 Database: SQLite connection management, migration runner, migration version tracking, fresh DB creation, idempotent re-run behavior.
  - 2026-06-01 implemented SQLite foundation connection/migration flow with schema_migrations tracking, fresh database creation, idempotent re-run behavior, migration version reporting, and regression coverage. Follow-up P1.05 fix enforces SQLite foreign keys on foundation/migration connections.
- [x] P1.05 Database: core schema for workspaces, settings, events, entity_links, log/file references, action confirmations/policies where needed.
  - 2026-06-01 added migration v3 core schema tables for app settings, integration statuses, entity_links, log_references, file_references, action_policies, and confirmation_decisions. Added migration/idempotence/no-secret/FK tests, seeded action policies from the centralized policy matrix, passed independent spec + quality reviews, and final critique verdict APPROVED.
- [x] P1.06 Database: repository/helper primitives for insert/read/list/update with typed errors and tests.
  - 2026-06-01 added private/internal repository primitives for `app_settings` and `entity_links`, with typed `RepositoryError` classification (`NotFound`, `Constraint`, `InvalidJson`, `Database`), deterministic list/read helpers, JSON validation before writes, idempotent logical entity-link insert-or-get, and regression coverage for primary-key collision edge cases. Final critique verdict APPROVED.
- [x] P1.07 Backend: canonical workspace registry service listing all 14 workspaces with truthful availability/integration states.
  - 2026-06-01 added canonical backend workspace registry for 14 ordered workspaces with serializable availability/integration metadata; guarded integrations remain truthful (`not_configured`, `needs_permission`, or `planned`) with no external probes/secrets. DB workspace seeding/listing is registry-backed and idempotent. Final critique verdict APPROVED.
- [x] P1.08 Backend: settings service for local app preferences and integration statuses without secrets outside Keychain.
  - 2026-06-01 added typed local preference and integration-status service helpers over existing `app_settings` and `integration_statuses`, with JSON validation, obvious secret/token/password/API-key rejection before DB writes, safe credential-reference validation, truthful registry-backed default integration seeding, and no Keychain writes/new Tauri commands/external probes. Final critique verdict APPROVED.
- [x] P1.09 Backend/security: Keychain test path/read-write-delete service or truthful blocked/unverified native status.
  - 2026-06-01 completed truthful blocked/unverified Keychain readiness path with typed backend status, legacy `keychain_status` compatibility, no credential storage/read/delete/logging, no new dependency or macOS prompt, and final critique verdict APPROVED.
- [x] P1.10 Backend/security: secret redaction for logs, events, metadata JSON, obvious tokens/keys, nested values; JSON remains valid.
  - 2026-06-01 completed dedicated shared redaction hardening for logs/events/confirmation decisions/entity-link metadata, with broader obvious secret-key/token markers, multiple assignments per line, spaced `key = value`/`key : value`, bearer forms, multi-token secret values, recursive nested JSON redaction preserving valid JSON, and final critique verdict APPROVED.
- [x] P1.11 Backend/security: safe log writer under app-support logs with filename/path sanitization, redaction, size/rotation basics.
  - 2026-06-01 completed safe log writer with sanitized direct-child log paths, active/rotated symlink rejection, redaction before write, deterministic max-size rotation/truncation, and `log_references` upserts containing only safe relative paths/counts/metadata flags. Final critique verdict APPROVED.
- [x] P1.12 Backend/security: action policy evaluator for read/create/update/delete/send/publish/deploy/file/process actions.
  - 2026-06-01 completed typed action policy request/evaluator with read/create/update/delete/send/publish/deploy/file/process dimensions, executable gate booleans, fail-closed unknown handling, high-risk classifier precedence fixes, external/integration create gating, evaluator-backed DB seeding, and final critique verdict APPROVED.
- [x] P1.13 Backend: confirmation decision records/framework for consequential actions.
  - 2026-06-01 completed backend confirmation decision models, SQLite create/read/list helpers, fail-closed execution guard requiring an `ActionPolicyDecision`, actor semantics for human/reviewer/clear-task confirmations, redacted summary/metadata persistence, fresh-schema actor type CHECK, and atomic v4 upgrade for already-applied v3 databases. Final critique verdict APPROVED.
- [x] P1.14 Backend: generic event writer/reader with entity links, metadata redaction, timestamps, action type, outcome, source.
  - 2026-06-01 completed internal event create/read/list repository APIs on existing `events`/`event_targets`, with action_type/outcome mapping, metadata validation before persistence, summary/metadata redaction, deterministic target/list ordering, bounded filters, savepoint-atomic event+target writes, rapid-event ID/order hardening, and final critique verdict APPROVED.
- [x] P1.15 Backend: entity link service for tasks, notes, products, files, repos, runs, emails, events, browser captures.
  - 2026-06-01 completed backend-only validated entity link service APIs for create/read/list-by-source/list-by-target, with allowed domain types (`task`, `note`, `product`, `file`, `repo`, `run`, `email`, `event`, `browser_capture`), required-field validation before persistence, metadata JSON validation/redaction, idempotent duplicate logical tuples, same-id collision rejection, deterministic directional filtering, and final critique verdict APPROVED.
- [x] P1.16 Tauri bridge: commands for foundation status, workspace registry, settings, event read/write, policy preview.
  - 2026-06-01 completed backend/Tauri bridge commands for foundation status, workspace registry, local preferences, integration statuses, event create/read/list, and action-policy preview, with registered command-surface regression coverage, bridge-level event create bounds, metadata JSON validation, secret rejection/redaction, read-only policy preview, and final critique verdict APPROVED.
- [x] P1.17 Frontend: polished macOS-first app shell with sidebar, toolbar/header, split-view content, no SaaS/cyberpunk styling.
  - 2026-06-01 completed frontend macOS-first shell with translucent sidebar/window controls, toolbar/search/status header, split primary workspace pane, inspector/details rail, truthful disabled actions, browser/native fallback labeling, and native-empty-registry handling that does not mask successful empty native state. Final critique verdict APPROVED.
- [x] P1.18 Frontend: Apple-style design tokens for system font, spacing, materials, shadows, light/dark, empty/error/loading states.
  - 2026-06-01 completed CSS-only Apple-style token layer for system/monospace fonts, spacing, radii, text/accent colors, glass materials, borders, focus rings, shadows, light/dark variants, and ready/pending/blocked/error/empty/loading states. Final critique verdict APPROVED.
- [x] P1.19 Frontend: reusable base components: sidebar item, workspace header, cards, badges, empty/blocker states, inspector/panel.
  - 2026-06-01 completed local frontend base components (`SidebarItem`, `WorkspaceHeader`, `InfoCard`, `StatusBadge`, `EmptyState`, `BlockerState`, `InspectorPanel`, `InspectorCard`) in `src/App.tsx`, preserved truthful browser/native fallback behavior, fixed the blocker-state review gap, and final critique verdict APPROVED.
- [x] P1.20 Frontend integration: render all workspace names from real workspace registry; no hardcoded fake connected states.
  - 2026-06-01 completed frontend workspace-registry integration with `buildWorkspaceRegistryView`, native `status.workspaces` rendering without fallback mixing, explicit browser-preview/checking source labels, visible source/count/truth copy in sidebar/registry/inspector, preserved native-empty-registry handling, no fake connected integration states, and final critique verdict APPROVED.
- [x] P1.21 Frontend integration: Today foundation/widgets from real local state or truthful empty/unconfigured states.
  - 2026-06-01 completed Today-specific foundation/widgets UI backed by native `get_foundation_status` when available and explicit browser/checking empty/unconfigured states otherwise. Added pure Today view-model test wired into `verify:local`; browser smoke passed with no console errors; final critique verdict APPROVED.
- [x] P1.22 Frontend: settings/status shell for paths, DB, Keychain, redaction, logging, policy, events, integrations.
  - 2026-06-01 completed inspector settings/status shell for paths, DB/migrations/events, Keychain, safeguards, policy summary, event writer, and truthful integration states. Added pure settings status view-model test wired into `test:frontend`; `npm run test:frontend`, `npm run build`, `npm run verify:local`, and browser smoke passed; final critique verdict APPROVED.
- [x] P1.23 Frontend: confirmation UI primitives showing policy reason and required confirmation/review.
  - 2026-06-01 completed read-only inspector confirmation policy primitives for policy source, category, policy, reason, human confirmation, reviewer, and clear-task gates. Added pure confirmation policy view-model test wired into `test:frontend`; `npm run test:frontend`, `npm run build`, `npm run verify:local`, browser smoke, spec review, quality review, and final critique passed; final critique verdict APPROVED.
- [x] P1.24 Tests: Rust unit tests for redaction, logging, Keychain status, policy, events, entity links, path creation.
  - 2026-06-02 completed Rust unit-test gap coverage for invalid metadata redaction fallback, safe-log truncation metadata, safe-log scope fallback, low-risk/invalid policy preview parsing, missing event NotFound behavior, and entity-link list filter validation. Existing Keychain/path tests cover those areas. `cargo test --manifest-path src-tauri/Cargo.toml --lib` passed with 88 tests; `npm run verify:local` and final critique passed; final critique verdict APPROVED.
- [x] P1.25 Tests: SQLite integration tests for migrations, version, repositories, event read/write, entity links.
  - 2026-06-02 completed file-backed SQLite reopen tests for migration/version/seed persistence, FK re-enable on reopened connections, rerun/reseed idempotence, app setting repository persistence, event+target read/write persistence, and entity-link persistence. `cargo test --manifest-path src-tauri/Cargo.toml` passed with 90 Rust tests; `npm run verify:local` and final critique passed; final critique verdict APPROVED.
- [x] P1.26 Tests: frontend build/smoke checks for registry rendering, settings status, empty states, no fake success copy.
  - 2026-06-02 completed frontend workspace registry smoke coverage by extracting registry/chrome view-model helpers and adding dependency-light tests for native registry rendering, settings-suite inclusion, native empty states, preview/checking fallback states, and no fake non-native success/readiness/connected copy. `npm run test:frontend`, `npm run build`, staged whitespace check, spec review, quality review, and final critique passed; final critique verdict APPROVED.
- [x] P1.27 Manual verification: launch app locally, verify folders/DB/logs/status/settings on macOS.
  - 2026-06-02 launched local Tauri dev app with `npm run tauri:dev`; verified native `target/debug/zoid` process, `http://127.0.0.1:1420/` HTTP 200, visible folders `Notes`, `Content`, `Assets`, `Exports`, `Imports`, `Backups`, app-support `logs`/`config`/`zoid.sqlite`/`foundation.log`, SQLite counts `workspaces=14`, `events=1`, `action_policies=20`, `integrations=7`, `app_settings=0`, `migration_version=4`, and foundation log readiness lines. Settings evidence is truthful: config directory and SQLite settings table/status path verified; startup does not create physical `config/settings.json` or default settings rows. Final verification review verdict APPROVED.
- [x] P1.28 Verification: run `npm run verify:local`.
  - 2026-06-02 ran `npm run verify:local` after P1.27. Repo gate passed: npm dependencies present, Tauri CLI found, Rust tests 90 passed/0 failed, frontend smoke tests passed, frontend production build passed, and final script marker `PASS: local push verification passed (--skip-package)`.
- [x] P1.29 Review: write `.hermes/reviews/phase-1-secure-foundation/handoff.md`.
  - 2026-06-02 wrote phase-level handoff summarizing completed P1.05-P1.28 secure-foundation slices, changed source/test/review artifacts, latest `npm run verify:local` evidence, P1.27 macOS launch evidence, and truthful caveats for packaging, Keychain, config/settings path, and browser preview states.
- [x] P1.30 Review: critique loop until `Verdict: APPROVED`.
  - 2026-06-02 completed phase-level critique for `.hermes/reviews/phase-1-secure-foundation/handoff.md`; final report verdict APPROVED. Critique reran `npm run verify:local` with Rust tests 90 passed/0 failed, frontend smoke tests passed, frontend production build passed, and final marker `PASS: local push verification passed (--skip-package)`. Approval preserves caveats: release packaging is out of scope for `verify:local`, real Keychain credential storage is not implemented, and `config/settings.json` is a reported/reserved path rather than a startup-created file.

---

## Phase 2 — First Vertical Slice

Goal: Today → Task → CLI Session → AgentRun → ReviewRecord → Notification → History.

- [x] P2.01 Planning: convert vertical-slice acceptance into task-level spec and data-flow diagram.
  - 2026-06-02 wrote `/Users/ziadnasreldin/Zoid/Docs/2026-06-02-phase-2-first-vertical-slice-spec.md` defining the `Today -> Task -> CLI Session -> AgentRun -> ReviewRecord -> Notification -> History` scope, entity boundaries, data flow, event taxonomy, P2.03-P2.07 order, TDD acceptance targets, and verification gates. Read-only data-model boundary subagent review passed before schema edits.
- [x] P2.02 Planning: define run lifecycle states, task states, review states, notification states, and failure/blocker states.
  - 2026-06-02 captured TaskStatus, AgentRunStatus, ReviewState/Verdict, NotificationState, and failure-vs-blocker semantics in the Phase 2 spec. Read-only lifecycle/state subagent review completed and recommended transition/test rules before schema edits.
- [x] P2.03 Database: tasks table/model with title, detail, status, priority, timestamps, archived/deleted handling.
  - 2026-06-02 added `src-tauri/migrations/0005_phase2_tasks.sql`, registered migration version 5, and implemented backend-only task model/repository helpers in `src-tauri/src/lib.rs` for create/read/list/status/archive/delete. Added validation for title/detail/status/priority/metadata JSON, secret-like metadata rejection, active-list archived/deleted filtering, and `task.created`/`task.status_changed`/`task.archived`/`task.deleted` events with task targets. Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml p203 -- --nocapture` (4 passed), `cargo test --manifest-path src-tauri/Cargo.toml task` (7 passed), and `cargo test --manifest-path src-tauri/Cargo.toml` (94 passed, 0 failed, doc-tests 0). Review artifacts: `.hermes/reviews/p2-03-task-database/handoff.md` and critique report verdict `APPROVED`.
- [x] P2.04 Database: AgentRun/session tables with task link, command/profile, cwd, status, duration, exit code, log reference, summary.
  - 2026-06-02 added `src-tauri/migrations/0006_phase2_agent_runs.sql`, registered migration version 6, and implemented backend-only `agent_profiles`, `cli_sessions`, and `agent_runs` repository/model helpers in `src-tauri/src/lib.rs`. Added mandatory task/session/profile linkage, configured/unconfigured profile truthfulness, command/config/capability metadata with secret-like command/JSON rejection, session mode/cwd/status summary, run command/profile snapshots, lifecycle status metadata, duration/exit/log-reference evidence, review state, entity links, and `run.*` lifecycle events. Raw logs remain out of SQLite/events; only log references/summaries/metadata are stored with redaction/secret validation. Required review fixes enforced `ON DELETE RESTRICT` for mandatory run-session FK, completed-run exit/log evidence, terminal immutability, and command secret rejection. Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml p204 -- --nocapture` (6 passed), `cargo test --manifest-path src-tauri/Cargo.toml run` (4 passed), and `cargo test --manifest-path src-tauri/Cargo.toml` (100 passed, 0 failed, doc-tests 0). Review artifacts: `.hermes/reviews/p2-04-agent-run-session-database/handoff.md` and critique report verdict `APPROVED`.
- [x] P2.05 Database: ReviewRecord table/model with manual reviewer stub fields and links to task/run.
  - 2026-06-02 added `src-tauri/migrations/0007_phase2_review_records.sql`, registered migration version 7, and implemented backend-only ReviewRecord repository/model helpers in `src-tauri/src/lib.rs` for manual reviewer stubs, task reviews, and agent-run reviews. Added latest-review gate semantics, review events (`review.created`, `review.approved`, `review.required_fixes`, `review.blocked_insufficient_evidence`), durable task/run review entity links, required-fixes array validation, intentional `related_entity` rejection, and DB guards for state/verdict consistency plus agent-run task ownership. Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml p205 -- --nocapture` (6 passed), `cargo test --manifest-path src-tauri/Cargo.toml review` (9 passed), and `cargo test --manifest-path src-tauri/Cargo.toml` (106 passed, 0 failed, doc-tests 0). Review artifacts: `.hermes/reviews/p2-05-review-record-database/handoff.md` and final critique report verdict `APPROVED`.
- [x] P2.06 Database: Notification/Inbox model for completion/blocker/attention records.
  - 2026-06-02 added `src-tauri/migrations/0008_phase2_notifications.sql`, registered migration version 8, and implemented backend-only Notification/Inbox model/repository helpers in `src-tauri/src/lib.rs`. Added persistent completion/blocker/failure/review-required/attention notifications with severity/state/action routes, task/run/review direct links, durable entity links, notification events, secret-material rejection, read/dismiss/resolve/deliver/action-required/failure transitions, active inbox severity/time sorting, and DB/repository guards for task/run/review ownership consistency. Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml p206 -- --nocapture` (6 passed), `cargo test --manifest-path src-tauri/Cargo.toml notification` (5 passed), and `cargo test --manifest-path src-tauri/Cargo.toml` (112 passed, 0 failed, doc-tests 0). Review artifacts: `.hermes/reviews/p2-06-notification-inbox-database/handoff.md` and critique report verdict `APPROVED`.
- [x] P2.07 Database: History/Event query model optimized for task/run/entity timelines.
  - 2026-06-02 added `src-tauri/src/history_service.rs` with task/run/notification/entity history query helpers over existing `events`, `event_targets`, and `entity_links`. Added deterministic `(timestamp desc, id desc)` ordering, cursor pagination, limit caps, invalid-entity rejection, relation-aware run history that excludes sibling runs sharing a task, and raw-log omission coverage. Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml p207 -- --nocapture` (3 passed), `cargo test --manifest-path src-tauri/Cargo.toml history -- --nocapture` (3 passed), and full Rust suite (116 passed, 0 failed, doc-tests 0). Review artifacts: `.hermes/reviews/p2-backend-query-service-batch/handoff.md` and critique report verdict `APPROVED`.
- [x] P2.08 Backend: task create/list/detail/update service with event writing.
  - 2026-06-02 added `src-tauri/src/task_service.rs` with create/list/read/update/status/archive/delete service wrappers over approved task repository primitives. Added editable field update support for title/detail/priority/workspace/metadata with existing validation/secret guards, savepoint-backed persistence, and `task.updated` event writing. Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml p208 -- --nocapture` (1 passed) and full Rust suite (116 passed, 0 failed, doc-tests 0). Reviewed in the approved backend query/service batch.
- [x] P2.09 Backend: CLI profile config for at least one safe local command with truthful configured/unconfigured states.
  - 2026-06-02 added `src-tauri/src/agent_execution_service.rs` preflight around existing agent profile/session/run repositories. Unconfigured profiles, missing commands, and missing cwd are blocked before any session/run records are created.
- [x] P2.10 Backend/native: command/session runner with cwd, streaming stdout/stderr, stdin if needed, cancel/kill, exit code, duration.
  - 2026-06-02 added safe local command execution via configured profile executable + explicit argv, cwd, stdin, stdout/stderr capture, exit code, duration, and timeout-based kill/cancel cleanup. Live UI-driven cancellation remains for P2.18 bridge/control wiring.
- [x] P2.11 Backend: child process cleanup and failure handling for cancelled/crashed runs.
  - 2026-06-02 records nonzero/spawn-error outcomes as `failed`, timeout-killed children as `cancelled`, persists evidence, writes lifecycle events, and creates failure/cancel notifications.
- [x] P2.12 Backend: persist redacted raw logs to app-support; SQLite stores metadata/log reference only.
  - 2026-06-02 runner persists stdout/stderr evidence through the existing safe log writer and links `agent_runs.log_reference_id`; tests verify raw secret material is not stored in the log file or SQLite summaries/metadata.
- [x] P2.13 Backend: AgentRun lifecycle service writes start/progress/completion/failure events.
  - 2026-06-02 runner writes queued/started/completed/failed/cancelled lifecycle events through approved AgentRun repository primitives and creates completion/failure/cancel notifications. Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml p209 -- --nocapture` (1 passed), `p210` (2 passed), `p211` (1 passed), `p212` (1 matching shared test passed), `p213` (1 matching shared test passed), and full Rust suite (120 passed, 0 failed, doc-tests 0). Review artifacts: `.hermes/reviews/p2-agent-execution-service-batch/handoff.md` and critique report verdict `APPROVED`.
- [x] P2.14 Backend: manual ReviewRecord creation service and reviewer-profile placeholder if available.
  - 2026-06-02 added `src-tauri/src/review_service.rs` with `create_manual_review_service` and `ManualReviewServiceCreateInput`. The service infers task vs. agent-run review subjects from optional `run_id`, uses an explicit reviewer profile when supplied, otherwise attaches optional `manual-reviewer` placeholder profile only when available, and delegates persistence to the approved ReviewRecord repository so validation, redaction, events, and entity links remain centralized. Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml p214 -- --nocapture` (2 passed), `cargo test --manifest-path src-tauri/Cargo.toml review -- --nocapture` (15 passed), and full Rust suite (122 passed, 0 failed, doc-tests 0). Review artifacts: `.hermes/reviews/p2-14-manual-review-service/handoff.md` and critique report verdict `APPROVED`.
- [x] P2.15 Backend: notification creation/query service for Today/Inbox.
  - 2026-06-02 added `src-tauri/src/notification_service.rs` with create/read/inbox and state-transition service wrappers over approved P2.06 notification repository primitives. Preserves P2.06 validation, link consistency, severity/time inbox sorting, and secret-material rejection. Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml notification -- --nocapture` (6 passed) and full Rust suite (116 passed, 0 failed, doc-tests 0). Reviewed in the approved backend query/service batch.
- [x] P2.16 Backend: history query service combining events/entity links without leaking raw logs/secrets.
  - 2026-06-02 completed the backend history service basics in `src-tauri/src/history_service.rs`, including task/run/notification/entity query composition, matched-entity reporting, pagination, raw-log omission coverage, and scoped run-history expansion to avoid sibling event leakage. Verification passed with the P2.07/history focused tests and full Rust suite. Reviewed in the approved backend query/service batch.
- [x] P2.17 Tauri bridge: commands/events for task CRUD.
  - 2026-06-02 added Tauri command handlers and registrations for task create/read/list/update/status/archive/delete in `src-tauri/src/lib.rs`. Commands use connection-injected helpers that delegate to approved `task_service` APIs, preserving validation, secret-metadata rejection, active-list filtering, and task event writing. Verification passed: RED `cargo test --manifest-path src-tauri/Cargo.toml p217 -- --nocapture` failed on missing bridge surface before implementation; final `p217` (2 passed), `tauri_bridge` (8 passed), and full Rust suite (124 passed, 0 failed, doc-tests 0). Review artifacts: `.hermes/reviews/p2-17-task-crud-tauri-bridge/handoff.md` and critique report verdict `APPROVED`.
- [x] P2.18 Tauri bridge: commands/events for run start/stream/cancel and run status.
  - 2026-06-02 added Tauri bridge command handlers and registrations for run start/status/stream/cancel in `src-tauri/src/lib.rs`. Start now creates a run/session, waits for worker child registration, and returns `running`; status observes live state; stream exposes redacted output chunks with offset/next-offset/EOF/status metadata; cancel kills active children and preserves cancelled-run duration/log/notification evidence. Verification passed: RED `cargo test --manifest-path src-tauri/Cargo.toml p218 -- --nocapture` failed on missing bridge surface before implementation; final `p218` (2 passed), `tauri_bridge` (8 passed), and full Rust suite (126 passed, 0 failed, doc-tests 0). Review artifacts: `.hermes/reviews/p2-18-run-tauri-bridge/handoff.md` and critique report verdict `APPROVED`.
- [x] P2.19 Tauri bridge: commands for review records, notifications, inbox, history.
  - 2026-06-02 added Tauri bridge command handlers and registrations for manual review creation/read, notification creation/read/inbox/state actions, and task/run/notification/entity history queries in `src-tauri/src/lib.rs`. Commands delegate to reviewed services/repository primitives, preserve link consistency, secret rejection/redaction, inbox sorting/caps, supported notification state transitions, and history raw-log omission. Verification passed: RED `cargo test --manifest-path src-tauri/Cargo.toml p219 -- --nocapture` failed on missing bridge surface before implementation; final `p219` (2 passed), `tauri_bridge` (8 passed), full Rust suite (128 passed, 0 failed, doc-tests 0), and `git diff --check`. Review artifacts: `.hermes/reviews/p2-19-review-notification-history-tauri-bridge/handoff.md` and critique report verdict `APPROVED`.
- [x] P2.20 Frontend: Today widgets showing real tasks, active runs, blockers, completions, empty states.
  - 2026-06-02 added `src/todayWidgets.ts` and wired the Today home in `src/App.tsx` to real native task and inbox notification bridge data. Today tasks come from `list_tasks_command`; blockers/completions derive from `list_inbox_notifications_command`; browser preview and bridge failures explicitly avoid simulated records. Active runs remain truthful by showing the current no-list-runs bridge gap instead of inventing rows. Verification passed: `npm run test:frontend`, `npm run build`, and `git diff --check`. Review artifacts: `.hermes/reviews/p2-20-today-widgets/handoff.md` and critique report verdict `APPROVED`.
- [x] P2.21 Frontend: task create/list/detail UI with validation and persistence.
  - 2026-06-02 wired the Tasks workspace in `src/App.tsx` to the approved task Tauri bridge via `src/taskBridgeIntegration.ts` and existing `TaskWorkspace` UI. Create/list/detail/update use real native commands (`list_tasks_command`, `read_task_command`, `create_task_command`, `update_task_command`); invalid forms are blocked locally before invoke; persisted task refreshes sync Today widgets without simulated records. Required review fixes added explicit New Task/create-mode handling, hydrated edit forms on selection, and truthful read-only status copy because status uses a separate native command. Verification passed: RED `npx tsx src/taskBridgeIntegration.test.ts` failed before implementation, final `npm run test:frontend`, `npm run build`, and `git diff --check` passed. Review artifacts: `.hermes/reviews/p2-21-task-ui-native-integration/handoff.md` and critique report verdict `APPROVED`.
- [ ] P2.22 Frontend: linked run/review/history panels inside task detail.
- [ ] P2.23 Frontend: Clean Session UI that streams output as clean cards/status, not raw terminal-first UI.
- [ ] P2.24 Frontend: run controls for start/cancel and clear status/error handling.
- [ ] P2.25 Frontend: manual review stub UI.
- [ ] P2.26 Frontend: notification/Inbox attention card basics.
- [ ] P2.27 Frontend: History view for task/run events.
- [x] P2.28 Tests: backend tests for task persistence and event writing.
  - 2026-06-02 added `p228_task_service_persists_tasks_and_task_events_after_reopen` to verify file-backed SQLite task persistence after reopen/migration rerun, active/archive/delete filtering, updated field persistence, and task event source/outcome/workspace/primary-target semantics.
- [x] P2.29 Tests: backend tests for run lifecycle, cancellation, exit codes, log persistence, redaction.
  - 2026-06-02 added stricter P2.29 run bridge regressions for failed exit-code/log/notification/history evidence, raw-secret redaction across streams/logs/history/SQLite fields, deterministic cancellation process-kill evidence, persisted cancellation log/log-reference metadata, cancellation notification specificity, and terminal mutation rejection.
- [x] P2.30 Tests: backend tests for review records, notifications, history queries.
  - 2026-06-02 added/extended P2.30 coverage for review/notification/history bridge behavior, including a dedicated review-linked notification state-transition history regression with task/run/review/notification targets and bridge history queries. Verification passed: `cargo test --manifest-path src-tauri/Cargo.toml p228 -- --nocapture` (1 passed), `p229` (2 passed), `p230` (3 passed), `npm run test:frontend`, `npm run build`, and `git diff --check`. Review artifacts: `.hermes/reviews/p2-28-p2-30-backend-regression-tests/handoff.md` and critique report verdict `APPROVED`.
- [ ] P2.31 Tests: UI smoke/E2E where feasible for create task → start CLI run → see output → notification/history.
- [ ] P2.32 Manual verification: launch app, create task, run command, restart app, verify persistence.
- [ ] P2.33 Verification: run `npm run verify:local`.
- [ ] P2.34 Review: write `.hermes/reviews/phase-2-first-vertical-slice/handoff.md`.
- [ ] P2.35 Review: critique loop until `Verdict: APPROVED`.

---

## Phase 3 — Notes, Files, and Local Knowledge

- [ ] P3.01 Planning: define Notes/Files scope; explicitly exclude Apple Notes import and full file-manager overreach.
- [ ] P3.02 Database: notes/files/index/entity-link schema and migrations.
- [ ] P3.03 Database: note identity/index metadata with stable frontmatter ID and conflict state.
- [ ] P3.04 Backend: Markdown note create/edit/delete/trash service.
- [ ] P3.05 Backend: frontmatter stable ID writer/reader and scanner/indexer.
- [ ] P3.06 Backend: conflict handling for duplicate IDs, manual renames, and external edits; non-destructive defaults.
- [ ] P3.07 Backend: basic file browse/open/preview service.
- [ ] P3.08 Backend: safe rename/move/copy/trash with action policy confirmation.
- [ ] P3.09 Backend: entity links from notes/files to tasks/products/runs.
- [ ] P3.10 Tauri bridge: commands for notes CRUD, scan/index, conflicts, file browse/open/preview/actions.
- [ ] P3.11 Frontend: Notes workspace list/editor/preview/trash/conflict states.
- [ ] P3.12 Frontend: Files workspace browse/open/preview/actions/confirmation.
- [ ] P3.13 Frontend: history/links panels for notes and files.
- [ ] P3.14 Tests: note persistence after restart.
- [ ] P3.15 Tests: manual file rename preserves note identity.
- [ ] P3.16 Tests: duplicate ID conflict is non-destructive.
- [ ] P3.17 Tests: destructive file operations require confirmation.
- [ ] P3.18 Tests: note/file events and entity links are recorded.
- [ ] P3.19 Manual verification: create/edit/delete/trash note; restart; inspect files on disk.
- [ ] P3.20 Manual verification: browse/open/preview file and perform confirmed safe operation.
- [ ] P3.21 Verification: run `npm run verify:local`.
- [ ] P3.22 Review: write `.hermes/reviews/phase-3-notes-files-knowledge/handoff.md`.
- [ ] P3.23 Review: critique loop until `Verdict: APPROVED`.

---

## Phase 4 — Code, Repos, Launch Gate

- [ ] P4.01 Planning: define scope; avoid overbuilding full GitHub/Vercel automation.
- [ ] P4.02 Database: repo profiles, product/task links, launch gate checks, verification evidence, integration states.
- [ ] P4.03 Backend: repo discovery/manual add service.
- [ ] P4.04 Backend: git status/diff/read operations.
- [ ] P4.05 Backend: git commit/push/merge/deploy action policy; protected actions require confirmation/review.
- [ ] P4.06 Backend integration: GitHub state detection where credentials configured; truthful blocked/unconfigured otherwise.
- [ ] P4.07 Backend integration: Vercel state detection where credentials configured; truthful blocked/unconfigured otherwise.
- [ ] P4.08 Backend: Launch Gate check/evidence model; cannot mark verified without real evidence.
- [ ] P4.09 Backend: production verification evidence records with screenshot/URL/status/manual note where available.
- [ ] P4.10 Tauri bridge: repo add/list/status/diff commands.
- [ ] P4.11 Tauri bridge: launch gate/evidence commands.
- [ ] P4.12 Frontend: Code/Repos UI for repo add, status, diff, linked tasks/products.
- [ ] P4.13 Frontend: GitHub/Vercel blocked/unconfigured/connected/error states; no fake connected data.
- [ ] P4.14 Frontend: Launch Gate UI showing checks, evidence, failures, confirmation requirements.
- [ ] P4.15 Tests: repo add/link/status/diff.
- [ ] P4.16 Tests: Launch Gate evidence requirements and no fake verification.
- [ ] P4.17 Tests: deploy/push/merge confirmation policy.
- [ ] P4.18 Manual verification: add a local repo, inspect status/diff, link to task/product.
- [ ] P4.19 Manual verification: try Launch Gate without evidence and confirm it fails closed.
- [ ] P4.20 Verification: run `npm run verify:local`.
- [ ] P4.21 Review: write `.hermes/reviews/phase-4-code-repos-launch-gate/handoff.md`.
- [ ] P4.22 Review: critique loop until `Verdict: APPROVED`.

---

## Phase 5 — Content and OmniSocials

- [ ] P5.01 Planning: define draft-first/fail-closed publishing scope and specialist design/review gates.
- [ ] P5.02 Database: content plans, pieces, assets, review gates, platform statuses, schedules, verification records.
- [ ] P5.03 Backend: content plan → draft → asset → review → schedule workflow.
- [ ] P5.04 Backend: media asset storage/references and platform media constraints.
- [ ] P5.05 Backend: specialist design/review gate records; no schedule/publish without required review/confirmation.
- [ ] P5.06 Backend integration: OmniSocials account/status detection with truthful states.
- [ ] P5.07 Backend integration: OmniSocials upload/schedule/publish surfaces; fail closed on missing credentials/errors.
- [ ] P5.08 Backend: verification records and failure reports for platform actions.
- [ ] P5.09 Tauri bridge: content CRUD/workflow commands.
- [ ] P5.10 Tauri bridge: OmniSocials status/upload/schedule/publish commands with policy enforcement.
- [ ] P5.11 Frontend: content calendar/workspace with plans, drafts, assets, review gates.
- [ ] P5.12 Frontend: platform constraints, schedule/publish confirmation surfaces, failure reports.
- [ ] P5.13 Tests: workflow progression plan → draft → asset → review → schedule.
- [ ] P5.14 Tests: failed generation/review/upload/schedule fails closed.
- [ ] P5.15 Tests: no publish/schedule without required review/confirmation.
- [ ] P5.16 Tests: platform media constraints enforced.
- [ ] P5.17 Manual verification: create content piece through draft/review/schedule path without publishing by default.
- [ ] P5.18 Verification: run `npm run verify:local`.
- [ ] P5.19 Review: write `.hermes/reviews/phase-5-content-omnisocials/handoff.md`.
- [ ] P5.20 Review: critique loop until `Verdict: APPROVED`.

---

## Phase 6 — Calendar, Gmail, Inbox, Business, Products

- [ ] P6.01 Planning: define Calendar/Gmail/Inbox/Business/Products scope and confirmation/privacy boundaries.
- [ ] P6.02 Database: contacts, companies, follow-ups, products, email/calendar references, cross-entity links, integration statuses.
- [ ] P6.03 Backend/native: EventKit permission/status service.
- [ ] P6.04 Backend/native: EventKit calendar read/create/edit/delete with confirmation requirements.
- [ ] P6.05 Backend integration: Gmail OAuth/status service with truthful states.
- [ ] P6.06 Backend integration: Gmail read/search/draft/send; send always requires confirmation.
- [ ] P6.07 Backend: Inbox aggregation across notifications, emails, calendar, follow-ups, blockers.
- [ ] P6.08 Backend: contacts/companies/follow-ups services.
- [ ] P6.09 Backend: Products workspace services and links to tasks/repos/notes/emails/events.
- [ ] P6.10 Tauri bridge: calendar commands.
- [ ] P6.11 Tauri bridge: Gmail commands.
- [ ] P6.12 Tauri bridge: inbox/business/products commands.
- [ ] P6.13 Frontend: Inbox workspace with attention items, blockers, follow-ups, integration states.
- [ ] P6.14 Frontend: Calendar workspace with read/create/edit/delete confirmation flows.
- [ ] P6.15 Frontend: Gmail surfaces with read/search/draft/send confirmation and safe unconfigured/auth states.
- [ ] P6.16 Frontend: Business workspace for contacts, companies, follow-ups.
- [ ] P6.17 Frontend: Products workspace with cross-links and related history.
- [ ] P6.18 Tests: Gmail send confirmation and no silent send.
- [ ] P6.19 Tests: calendar create/edit/delete confirmation.
- [ ] P6.20 Tests: permission/auth/integration states do not leak internal provider details.
- [ ] P6.21 Tests: cross-link integrity and persistence.
- [ ] P6.22 Manual verification: EventKit native checks or documented blocker.
- [ ] P6.23 Manual verification: Gmail OAuth/native checks or documented blocker.
- [ ] P6.24 Verification: run `npm run verify:local`.
- [ ] P6.25 Review: write `.hermes/reviews/phase-6-calendar-gmail-business-products/handoff.md`.
- [ ] P6.26 Review: critique loop until `Verdict: APPROVED`.

---

## Phase 7 — Browser Workspace and Advanced Widgets

- [ ] P7.01 Planning: review Phase 7 scope against Browser/WebView spike findings.
- [ ] P7.02 Planning: define “work webview/capture workspace” only; exclude full personal browser, extensions, browser sync, password manager, unproven console capture.
- [ ] P7.03 Planning: define Browser flows: open URL, saved page/tab, capture screenshot/fallback metadata, save link, attach to entities, view history.
- [ ] P7.04 Planning: define evidence fields: URL, title, timestamp, screenshot ref, HTTP status if available, manual note, entity links.
- [ ] P7.05 Planning: define widget customization requirements: visibility, order, simple size, persistence, reset.
- [ ] P7.06 Database: browser_tabs model/migration.
- [ ] P7.07 Database: browser_captures model/migration.
- [ ] P7.08 Database: entity links for browser_capture → launch_gate/task/note/product/content_piece.
- [ ] P7.09 Database: events for browser open/update/close, capture created/attached, widget config changed/reset.
- [ ] P7.10 Database: widget configuration persistence keyed by workspace/profile.
- [ ] P7.11 Backend: browser tab/saved-page repository/service.
- [ ] P7.12 Backend/native: WebView integration for opening work URLs within proven Tauri limits.
- [ ] P7.13 Backend/native: tab abstraction or saved-page fallback depending on feasibility.
- [ ] P7.14 Backend/native: screenshot capture path if feasible; files in app support, metadata in SQLite.
- [ ] P7.15 Backend/native: fallback capture path when screenshot unavailable: URL/title/timestamp/status/manual note.
- [ ] P7.16 Backend: HTTP status helper where feasible for verification evidence.
- [ ] P7.17 Backend: attachment service linking browser captures to Launch Gate/Task/Note/Product/ContentPiece.
- [ ] P7.18 Backend: Launch Gate evidence validation using browser capture only when required fields exist.
- [ ] P7.19 Backend: widget configuration service with validation against allowed widgets/sizes.
- [ ] P7.20 Backend/security: ensure no raw cookies/auth headers/tokens/secrets in logs/events/SQLite.
- [ ] P7.21 Tauri bridge: tab/saved-page commands.
- [ ] P7.22 Tauri bridge: capture creation/screenshot/attachment commands.
- [ ] P7.23 Tauri bridge: HTTP status/route smoke verification command where applicable.
- [ ] P7.24 Tauri bridge: widget config read/update/reset commands.
- [ ] P7.25 Frontend: Browser workspace shell with toolbar, URL input, content area, tab/saved-page strip, inspector/captures sidebar.
- [ ] P7.26 Frontend: truthful Browser empty/loading/error/blocked/unsupported states.
- [ ] P7.27 Frontend: tab/saved-page list backed by real data.
- [ ] P7.28 Frontend: capture action UI previewing saved fields and target entity.
- [ ] P7.29 Frontend: capture detail view with screenshot/link/evidence metadata and linked entities.
- [ ] P7.30 Frontend: attachment picker for Launch Gate, Task, Note, Product, ContentPiece.
- [ ] P7.31 Frontend: Launch Gate evidence integration for browser evidence.
- [ ] P7.32 Frontend: widget customization controls: show/hide, reorder, resize, reset, persisted state.
- [ ] P7.33 Frontend/copy: never claim full browser/personal browser capability.
- [ ] P7.34 Tests: browser capture validation, metadata, attachment rules, evidence eligibility.
- [ ] P7.35 Tests: widget validation, ordering, sizing, show/hide, reset defaults.
- [ ] P7.36 Tests: SQLite integration for tabs, captures, links, events, widget config.
- [ ] P7.37 Tests: redaction for URLs/metadata/logs/events; no cookies/auth headers/tokens/secrets.
- [ ] P7.38 Tests: UI smoke for Browser workspace, URL, capture metadata, evidence attachment, widget customization.
- [ ] P7.39 Manual verification: open normal work URL and persist URL/title after restart.
- [ ] P7.40 Manual verification: login-heavy website behavior documented as supported/blocked/partial.
- [ ] P7.41 Manual verification: screenshot capture or fallback capture verified.
- [ ] P7.42 Manual verification: evidence attaches to Launch Gate and renders there.
- [ ] P7.43 Manual verification: widget changes survive restart.
- [ ] P7.44 Accessibility: keyboard and screen-reader labels for URL, tab list, capture, attachment picker, widget controls.
- [ ] P7.45 Performance: saved tabs/captures/widgets do not degrade workspace rendering or bloat SQLite.
- [ ] P7.46 Docs: document Browser capabilities/unsupported features and widget behavior.
- [ ] P7.47 Verification: run `npm run verify:local` and fresh/existing DB migration checks.
- [ ] P7.48 Review: write `.hermes/reviews/phase-7-browser-widgets/handoff.md`.
- [ ] P7.49 Review: critique loop until `Verdict: APPROVED`.

---

## Phase 8 — Packaging, Performance, Accessibility, Hardening

- [ ] P8.01 Planning: write hardening spec covering packaging/signing, notification polish, states, accessibility, performance, log retention, migrations, docs.
- [ ] P8.02 Planning: define release readiness report format: commit, build, tests, manual verification, blockers, command output, critique verdict.
- [ ] P8.03 Planning: define macOS packaging/signing/notarization path, identity, entitlements, sandbox/file/CLI implications.
- [ ] P8.04 Planning: define app-wide hardening matrix for actions, integrations, secrets, migration failures.
- [ ] P8.05 Database/backend: log retention settings and cleanup service.
- [ ] P8.06 Database/backend: migration hardening, failed migration safe state, destructive migration backup.
- [ ] P8.07 Database/backend: performance indexes for events, tasks, runs, notifications, captures, links, history.
- [ ] P8.08 Backend/security: audit action policy enforcement across destructive actions, credentials, send/publish/deploy, calendar, Gmail, files.
- [ ] P8.09 Backend/security: audit redaction coverage for logs, events, prompts, summaries, errors, crash reports, diagnostic UI.
- [ ] P8.10 Backend: structured safe error mapping for native/backend failures.
- [ ] P8.11 Native: configure macOS packaging for Tauri.
- [ ] P8.12 Native: configure signing flow and document required certificates/env vars without committing secrets.
- [ ] P8.13 Native: configure notarization/stapling path if applicable.
- [ ] P8.14 Native: review entitlements for notifications, file access, Keychain, EventKit, WebView, CLI execution.
- [ ] P8.15 Native: polish native notifications and click/open-route behavior where feasible.
- [ ] P8.16 Native: verify packaged app can access `~/Zoid`, app support, Keychain, SQLite, notifications, CLI constraints.
- [ ] P8.17 Tauri bridge: harden command error responses with structured redacted payloads.
- [ ] P8.18 Tauri bridge: audit command allowlist/permissions and remove unused exposed surface.
- [ ] P8.19 Tauri bridge: commands for log retention settings/manual cleanup with confirmation.
- [ ] P8.20 Frontend: app-wide empty/loading/error/blocker state pass.
- [ ] P8.21 Frontend: migration failure safe screen with recovery guidance.
- [ ] P8.22 Frontend: log retention settings UI.
- [ ] P8.23 Frontend: notification/inbox route polish.
- [ ] P8.24 Frontend: Apple-style polish pass across all workspaces.
- [ ] P8.25 Frontend/copy: remove internal/private implementation details from user-facing UI.
- [ ] P8.26 Frontend: release/about/settings surface with safe version/build info.
- [ ] P8.27 Tests: log retention policy and cleanup.
- [ ] P8.28 Tests: migration hardening/failure/destructive backup.
- [ ] P8.29 Tests: redaction regressions across logs/events/errors/summaries/prompts/notifications.
- [ ] P8.30 Tests: policy regression tests for all consequential/destructive actions.
- [ ] P8.31 Tests: integration-state tests prevent fake connected/success data.
- [ ] P8.32 Tests: UI smoke tests for critical routes/workspaces.
- [ ] P8.33 Accessibility: keyboard navigation across workspaces, dialogs, menus, inspectors, confirmations, settings.
- [ ] P8.34 Accessibility: focus management, screen-reader labels, contrast, reduced motion.
- [ ] P8.35 Performance: cold/warm startup and workspace switching measurements.
- [ ] P8.36 Performance: large history/log rendering, DB query/index measurements, memory during Browser/CLI streaming.
- [ ] P8.37 Manual verification: packaged macOS app opens cleanly.
- [ ] P8.38 Manual verification: first launch creates folders; restart preserves DB/settings/log metadata/workspace state.
- [ ] P8.39 Manual verification: secrets remain redacted in logs/events/UI errors/notifications using known test secret strings.
- [ ] P8.40 Manual verification: destructive actions require confirmation after packaging.
- [ ] P8.41 Manual verification: native notification click/open behavior and migration failure safe screen.
- [ ] P8.42 Verification: clean install local verification from empty app data dir.
- [ ] P8.43 Verification: upgrade verification from populated local DB/app-support state.
- [ ] P8.44 Verification: full build/typecheck/test suite.
- [ ] P8.45 Release verification: run `npm run verify:release` only intentionally; inspect packaged app/DMG, bundle metadata, launch binary.
- [ ] P8.46 Release verification: signing/notarization/stapling if configured.
- [ ] P8.47 Release verification: artifact contains no raw secrets/private config/development-only paths.
- [ ] P8.48 Docs: local dev/build/test/package commands.
- [ ] P8.49 Docs: signing/notarization/release process and required non-committed secrets/certificates.
- [ ] P8.50 Docs: app data locations, security model, log retention, migration behavior, unsupported/partial features, manual native checklist.
- [ ] P8.51 Review: security review for secrets/redaction/Keychain/logs/events/Tauri surface/artifact contents.
- [ ] P8.52 Review: UX/accessibility/performance/release readiness reviews.
- [ ] P8.53 Review: write `.hermes/reviews/phase-8-hardening-release-readiness/handoff.md`.
- [ ] P8.54 Review: critique loop until `Verdict: APPROVED`.
