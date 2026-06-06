# Zoid Code Workspace Implementation Tracker

Date: 2026-06-06
Source discovery: `/Users/ziadnasreldin/brainstorms/2026-06-06-zoid-code-workspace-implementation.md`
Related docs: `prd.md`, `stitch-ai-screen-design-prompt.md`, `/Users/ziadnasreldin/Zoid/Docs/designer-screen-reference/02-code-workspace.md`

Purpose: execution tracker for the full Code Workspace feature set. Status legend: `[ ]` pending, `[~]` in progress, `[x]` complete, `[!]` blocked.

Important: no Phase 2/3 deferral. Use Build Order to implement safely, but all major requested functionality belongs in scope.

## Global Gates

- [ ] Do not scan the whole home folder automatically; only user-approved scan roots.
- [ ] Do not mark discovered repos as managed until explicit approval.
- [ ] Do not perform destructive git/file/deploy actions without confirmation.
- [ ] Do not send secrets/env files to agents by default.
- [ ] Do not call launch/deploy success unless required evidence exists.
- [ ] Browser preview is not sufficient; final verification must run inside Tauri/native Zoid.
- [ ] Use at least one real repo plus one disposable test repo for safe git/deploy action testing.
- [ ] Feature critique workflow must run before complete: create `.hermes/reviews/code-workspace/handoff.md`, receive critique report, fix Required items, and re-review until approved.

## Likely Primary Files / Areas to Inspect

- Existing reference: `Docs/designer-screen-reference/02-code-workspace.md`
- Related references: `Docs/designer-screen-reference/03-agents-workspace.md`, `Docs/designer-screen-reference/12-shared-tasks-calendar-history-reviews.md`
- Frontend app/workspace routes and state: inspect current `src/` structure.
- Native/Tauri commands and process runner: inspect `src-tauri/src/`.
- Persistence/migrations: inspect current app database layer.
- Agents Workspace integration points: inspect existing/new agents module.
- GitHub/auth/deployment settings: inspect settings/config modules.

## Verification Command Set

Adjust after repo inspection:

- [ ] Frontend focused tests: `npm run test -- code-workspace` or project equivalent.
- [ ] Native/backend tests: `cargo test --manifest-path src-tauri/Cargo.toml code_workspace -- --nocapture` or project equivalent.
- [ ] Full local verification: `npm run verify:local && git diff --check` or current equivalent.
- [ ] Native manual: `npm run tauri:dev`, run discovery/checks/launch gate against real + disposable repos.

---

## Build Order 1 — Repo Discovery + Managed Registry

- [ ] C1.01 Inspect current Zoid repo structure, Code Workspace placeholder, persistence schema, native command patterns, and existing workspace routing.
- [ ] C1.02 Define domain models: ManagedRepo, ScanRoot, DiscoveredRepo, RepoProfile, RepoPermission, RepoStatusSnapshot, RepoEvent, RepoGroup, IgnoredRepo.
- [ ] C1.03 Add persistence for scan roots, discovered repos, managed repos, ignored repos, groups, status snapshots, and history events.
- [ ] C1.04 Implement user-added scan roots only; no automatic whole-home scan.
- [ ] C1.05 Implement scanner for `.git`, remotes, package/framework markers, recent activity, dirty state, likely product/client, duplicates, and nested/monorepo warnings.
- [ ] C1.06 Implement discovery UI: scan source panel, suggested repos list, bulk actions, preview inspector, ignored tab.
- [ ] C1.07 Implement Approve Selected, Ignore Selected, Restore Ignored, Add to group/client, Set profile, Open in Finder, Rescan.
- [ ] C1.08 Add tests for scanner detection, duplicate/nested warnings, ignored restore, and approval-only managed registry.
- [ ] Gate: native app scans an explicit root, discovers real Git repos, approves selected repos, and persists them after restart.

## Build Order 2 — Repo Profiles, Settings, Permissions

- [ ] C2.01 Implement profile types: Zoid/Tauri app, Next.js/Vite frontend, Node/Nest backend, full-stack app, static website, SaaS/product app, client project, docs/content repo, infra/devops repo, unknown/custom.
- [ ] C2.02 Implement auto-detection from package.json, lockfiles, Cargo.toml, docker-compose, Makefile, Tauri config, and framework markers.
- [ ] C2.03 Add manual profile override.
- [ ] C2.04 Add repo settings: display name, group, path, remote URL, default branch, commands, deployment targets, launch checklist, production URLs, agent permissions, sensitive file patterns, preferred agent/model, reviewer requirement, evidence storage, ignore/archive settings.
- [ ] C2.05 Import candidate rules from AGENTS.md, CLAUDE.md, README, package scripts, and .cursorrules, with review/approval before saving.
- [ ] C2.06 Add per-repo permissions UI explaining read/status, command execution, destructive actions, and sensitive file handling.
- [ ] C2.07 Detect secrets/env/sensitive files and exclude them from default agent context.
- [ ] Gate: repo profile/settings/permissions persist and drive command/check/agent defaults.

## Build Order 3 — Repo Health Dashboard + Right Inspector

- [ ] C3.01 Implement Code Workspace landing as repo health command center.
- [ ] C3.02 Group repos primarily by product/client, with filters by profile/status/recent.
- [ ] C3.03 Add mission status: Safe, Dirty, Blocked, Ready to Review, Ready to Launch, Verified.
- [ ] C3.04 Add attention queue for failed checks, blocked launch gates, unverified deployments, active agent waiting for input, dirty/risky changes.
- [ ] C3.05 Add small summary counters: managed repos, dirty repos, open launch gates, failed checks, needs-me.
- [ ] C3.06 Add compact rows default and optional card/grid toggle.
- [ ] C3.07 Implement right inspector tabs: Summary, Diff, Checks, Agents, Launch, Deployments, Linked Items, History.
- [ ] C3.08 Add global Code + Agents “what needs me?” queue surface.
- [ ] Gate: dashboard accurately reflects real repo status snapshots and selected repo inspector updates.

## Build Order 4 — Repo Detail + Git Status/Diff/Checks

- [ ] C4.01 Implement full repository detail page with expanded Summary/Diff/Checks/Agents/Launch/Deployments/Linked Items/History sections.
- [ ] C4.02 Show branch, dirty state, ahead/behind, changed files, diff summary, risky files, and sensitive/config changes.
- [ ] C4.03 Implement changed/relevant files only; do not build full file tree/editor.
- [ ] C4.04 Add actions: Open Repo, Open Finder, Open Editor/Terminal, Reveal Changed File, View Diff.
- [ ] C4.05 Implement layered command discovery: repo files, profile defaults, manual overrides, approved agent suggestions.
- [ ] C4.06 Store typecheck/lint/test/build/dev/deploy/verify commands.
- [ ] C4.07 Run full checks only on explicit Run Checks, Launch Gate requirement, or explicit auto-check policy.
- [ ] C4.08 Persist command output, pass/fail, duration, timestamp, and evidence links.
- [ ] C4.09 Add per-repo notes: product goal, deployment notes, test accounts, verification checklist.
- [ ] Gate: Run Checks executes real stored commands and persists results/evidence.

## Build Order 5 — Agents Workspace Linking

- [ ] C5.01 Connect managed repos to Agents Workspace repo selector/source.
- [ ] C5.02 Implement Start Agent from repo with New Agent Session modal prefilled.
- [ ] C5.03 Attach repo/workdir, current diff/status, repo profile/rules, permission preview, and suggested prompts.
- [ ] C5.04 Suggested actions: Review this repo, Fix failed checks, Implement task, Prepare launch gate, Debug deployment.
- [ ] C5.05 Keep user in Code Workspace by default after starting; show linked run plus Open in Agents Workspace/Open in panel actions.
- [ ] C5.06 Show linked active/past agent sessions in dashboard, repo detail, and inspector.
- [ ] C5.07 Add one-click handoff repo state export for agents/future sessions.
- [ ] Gate: starting a real Agents Workspace session from a repo links it under the correct repo group in both workspaces.

## Build Order 6 — Launch Gate + Evidence Model

- [ ] C6.01 Define LaunchGate, LaunchChecklistItem, EvidenceItem, VerificationRun, DeploymentRecord, Verdict models.
- [ ] C6.02 Implement Launch Gate UI: header, repo/product/task/commit/deployment state, checklist, evidence panel, final verdict, history timeline.
- [ ] C6.03 Checklist: git state, typecheck, lint, tests, build, review, push, PR, deployment, production/native E2E, backend/API/database verification.
- [ ] C6.04 Evidence types: command output, diff snapshot, commit SHA, reviewer verdict, PR link/status, deployment URL/provider status, E2E result, API/backend/database result, screenshots/video, logs, notes.
- [ ] C6.05 Store canonical metadata/small logs in app data; link larger attachments in repo/docs folders.
- [ ] C6.06 Prevent Mark Verified until required evidence exists.
- [ ] C6.07 Allow forced override only with explicit reason, recorded as override not normal verified success.
- [ ] C6.08 Add quick compare current diff vs last verified launch.
- [ ] Gate: Launch Gate blocks false success and persists evidence/history after restart.

## Build Order 7 — GitHub / PR Integration

- [ ] C7.01 Detect GitHub remote URL and show repo link.
- [ ] C7.02 Add graceful unauthenticated state: local management works; PR/CI prompts connect GitHub.
- [ ] C7.03 Add global GitHub connection settings and per-repo override/disable.
- [ ] C7.04 Show open PRs, issue/task links where available, and CI/check status when authenticated.
- [ ] C7.05 Implement create PR draft from selected branch with reviewable title/body and confirmation.
- [ ] C7.06 Persist PR links/status in repo linked items/history.
- [ ] Gate: GitHub unauth/auth states work without breaking local repo management.

## Build Order 8 — Deployment Tracking / Actions

- [ ] C8.01 Add deployment target records: Vercel, Hostinger VPS/SSH/Docker/Nginx, GitHub Pages, Cloudflare Pages/R2, custom/manual.
- [ ] C8.02 Store environment, production URL, provider/command, last deploy status, last verified status, required E2E checklist, rollback notes/command.
- [ ] C8.03 Add manual deployment record creation/editing.
- [ ] C8.04 Add explicit deployment action flow with confirmation, command/provider preview, and evidence linkage.
- [ ] C8.05 Add rollback notes/action preview requiring strong confirmation.
- [ ] C8.06 Add deployment unverified/stale warnings.
- [ ] Gate: deployment records/actions are evidence-linked and never surprise-run.

## Build Order 9 — Commit / PR / Git Action Workflows

- [ ] C9.01 Implement safe read-only git views: status, diff, history, branches.
- [ ] C9.02 Implement normal-confirm actions: create branch, stage selected files, commit with editable message, create PR draft, pull latest if clean, stash changes.
- [ ] C9.03 Implement strong-confirm actions: push, merge, rebase, reset, dirty checkout, discard changes, delete branch, force push, rollback deployment-linked commit.
- [ ] C9.04 Block automatic force push, reset hard, delete untracked files, overwrite user changes, protected/main pushes.
- [ ] C9.05 Add danger zone panel for risky files/actions.
- [ ] C9.06 Run commit/PR flow only against disposable test repo in verification unless explicit real repo approval exists.
- [ ] Gate: safe disposable repo proves git actions; protected/destructive actions are blocked or strongly confirmed.

## Build Order 10 — Search, Archive, History, Polish + Native Verification

- [ ] C10.01 Add search across repos by name/path/group/profile/status/file/PR/agent/evidence/history.
- [ ] C10.02 Add archive/ignored repo management.
- [ ] C10.03 Add history timeline for repo events, checks, agents, launch gates, deployments, commits, PRs, evidence.
- [ ] C10.04 Add stale warnings: moved path, branch behind, old launch evidence, unverified deployment.
- [ ] C10.05 Polish visuals to Zoid design system: Apple Finder-style browser, native rows, right inspector, clean status chips, no enterprise-table heaviness.
- [ ] C10.06 Add empty/loading/error/blocked/success states for discovery, dashboard, repo detail, inspector, launch gate, GitHub, deployments, git actions.
- [ ] C10.07 Add accessibility: keyboard navigation, focus rings, non-color-only status, 44px actions, copyable logs.
- [ ] C10.08 Run frontend and native/backend tests.
- [ ] C10.09 Run native/Tauri verification against one real repo and one disposable test repo.
- [ ] C10.10 Restart Zoid and confirm registry/settings/history/evidence persist.
- [ ] C10.11 Create `.hermes/reviews/code-workspace/handoff.md`.
- [ ] C10.12 Run critique-agent review, fix Required items, and re-review until approved.
- [ ] Gate: feature cannot be called done until native E2E evidence and critique approval exist.
