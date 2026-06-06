# Critique Report: Phase 4 Code, Repos, Launch Gate

## Verdict

APPROVED

## Summary

Phase 4 is approved only under the formally narrowed scope the user selected after the first critique: lightweight native repo registry, truthful local GitHub/Vercel integration states, Launch Gate evidence records with fail-closed evaluation until evidence exists, and protected-action policy previews. The revised implementation plan, tracker, and handoff now explicitly defer real git status/diff/read operations, GitHub/Vercel automation, and deploy execution to a later dedicated slice.

The previous R1/R2/R3/R4/R6 blockers are resolved by the approved scope narrowing and tracker corrections. The implementation still does not provide real git status/diff/read, Code workspace add/status/diff flows, or deploy automation, but those are now documented non-goals for Phase 4 rather than missing acceptance criteria. Launch Gate evidence remains a coarse evidence-presence gate, not a production-grade verification engine; that limitation is now explicit enough for the narrowed Phase 4 slice.

Phase 5 code remains present and wired in the working tree. I am not approving Phase 5 in this report. Given the user's explicit decision to review Phase 4 independently after narrowing, I am treating Phase 5 contamination as a separate pending future-phase review concern rather than a Phase 4 approval blocker. This approval applies only to the Phase 4 files and behavior reviewed below.

I did not edit product source. The only file written for this re-review is this report.

## What was reviewed

- Updated handoff: `.hermes/reviews/phase-4-code-repos-launch-gate/handoff.md`.
- Previous critique report in this same path.
- Updated implementation plan: `Docs/2026-05-31-zoid-implementation-plan-v1.md`, Phase 4 section.
- Updated tracker: `Docs/2026-06-01-zoid-implementation-tracker.md`, P4.01-P4.22.
- Phase 4 backend/source: `src-tauri/src/phase4_service.rs`.
- Phase 4 migration: `src-tauri/migrations/0010_phase4_code_repos_launch_gate.sql`.
- Tauri command/migration wiring in `src-tauri/src/lib.rs`.
- Code workspace UI in `src/App.tsx`.
- Phase 4 tests in `src-tauri/src/tests.rs`.
- Working-tree status and local verification output.

## Required fixes

None for the narrowed Phase 4 scope.

| ID | Severity | Area | Issue | Evidence | Required fix |
|----|----------|------|-------|----------|--------------|
| — | — | — | No blocking Phase 4 issue remains under the narrowed scope. | `cargo test --manifest-path src-tauri/Cargo.toml p4 -- --nocapture` passed; `npm run verify:local` passed; plan/tracker/handoff now document deferrals. | No Phase 4 fix required before approval. |

## Previous critique resolution

| Prior ID | Current status | Rationale |
|----------|----------------|-----------|
| R1 | Resolved by scope narrowing | The implementation plan now says Phase 4 is intentionally narrowed and explicitly defers full git status/diff/read, external GitHub/Vercel automation, and deploy exec. Tracker P4.04 is marked as deferred within Phase 4 rather than claiming real git status/diff/read. |
| R2 | Resolved by scope narrowing | The frontend Code workspace remains read-only and only loads repo list, truthful integration state, and deploy policy preview. That now matches the narrowed P4.12/P4.14 acceptance text. Repo add/read/link and Launch Gate evidence/evaluate remain native backend/bridge primitives, not full UI flows. |
| R3 | Resolved by scope narrowing | The tracker now accepts focused backend/source verification for repo add/list/read/link, no-fake status/diff behavior, Launch Gate fail-closed behavior, policy previews, and local build verification. Full native/manual add/status/diff UI evidence is no longer claimed for Phase 4. |
| R4 | Resolved by documentation/scope | `evaluate_launch_gate` still marks a gate verified when any evidence row exists. That would be too weak for a production Launch Gate, but the handoff/plan/tracker now describe Phase 4 as a coarse evidence-presence model. This should be strengthened in a later Launch Gate/browser/deploy verification slice, but it no longer blocks this narrowed slice. |
| R5 | Not approved here; treated as separate concern | Phase 5 files and wiring remain present in active source and tests. This is still review contamination in a general sense, but the user explicitly requested Phase 4 re-review after narrowing and asked whether Phase 5 can be treated separately. I am treating it as a pending Phase 5 review concern, not as a Phase 4 blocker. |
| R6 | Resolved | Tracker P4.01-P4.21 are now checked with exact evidence and P4.22 is pending this critique. Deferred work is explicitly recorded rather than silently omitted. |

## Positive findings

- Migration v10 creates `repo_profiles`, `launch_gates`, and `launch_gate_evidence` with useful constraints, indexes, and FK relationships.
- Repo profile add/list/read is persisted in SQLite and validates display name, root path, profile type, branch/package-manager fields, metadata JSON, and obvious secret-like metadata.
- Repo linking uses the existing entity-link model and is constrained to Phase 4 task/product targets.
- GitHub/Vercel integration state is truthfully local/state-only and returns `not_configured` without credentials or fake connected/deployed claims.
- Protected commit/push/merge/deploy actions are exposed only as policy previews; I did not find Phase 4 commit/push/deploy execution commands.
- Launch gates start blocked/missing-evidence and remain blocked when evaluated with zero evidence.
- Evidence records are persisted and can move the coarse Phase 4 gate to verified only after evidence exists.
- Code workspace copy is honest about lightweight registry, truthful integration state, policy preview, and the absence of fake repos/connected state/deploy execution.
- `npm run verify:local` passes on the current tree.

## Tests performed

- `git status --short --branch`: branch `main...origin/main [ahead 33]`; modified Phase 4 docs/source plus untracked review directory and Phase 5 files are present.
- `cargo test --manifest-path src-tauri/Cargo.toml p4 -- --nocapture`: PASS, 5 passed / 0 failed / 172 filtered out.
- `npm run verify:local`: PASS.
  - Rust tests: 176 passed / 0 failed / 1 ignored.
  - Frontend tests: passed.
  - Frontend build: passed with Vite production build.
  - Final marker: `PASS: local push verification passed (--skip-package)`.

## Non-blocking concerns / future work

- Phase 5 implementation is already present in source (`src-tauri/migrations/0011_phase5_content_omnisocials.sql`, `src-tauri/src/phase5_service.rs`, Phase 5 wiring in `src-tauri/src/lib.rs`, `src/App.tsx`, and tests). This must receive its own Phase 5 critique before being considered approved.
- The Phase 4 Launch Gate model is intentionally coarse. A future launch/deploy/browser verification slice should define stronger evidence semantics, such as required evidence types, URL/status/screenshot/test-output shape, timestamps, target environment, failure states, and rejection of weak/manual-only evidence when production verification is required.
- Real git status/diff/read, GitHub/Vercel automation, deploy execution, and rich Code workspace repo-management flows remain deferred and should be tracked in a later dedicated slice.
- There are no frontend unit tests specifically for the Code workspace. This is acceptable under the current narrowed criteria because frontend verification is via build/source review, but future richer Code UI should include view-model/component tests.

## Final approval scope

Approved for Phase 4 closeout only as:

- Lightweight native repo registry.
- Repo-to-task/product linking via entity links.
- Truthful GitHub/Vercel `not_configured` state surfaces.
- Launch Gate evidence records and coarse fail-closed-until-evidence evaluation.
- Commit/push/merge/deploy policy previews only.
- Read-only Code workspace surface for repo registry, truthful integration state, and policy preview.

Not approved in this report:

- Real git status/diff/read operations.
- GitHub/Vercel external automation.
- Deploy/redeploy/rollback execution.
- Full Launch Gate production verification semantics.
- Phase 5 Content/OmniSocials implementation.
