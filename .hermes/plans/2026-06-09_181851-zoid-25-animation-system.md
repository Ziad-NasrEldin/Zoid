# Zoid 25 exhaustive animation system plan

## Goal

Plan a repo-native animation system for Zoid 25 that eventually animates every user-visible page, component, item, and state without turning the app into decorative noise.

This is planning only. No implementation is started in this plan.

## Decisions already resolved

Source capture: `/Users/ziadnasreldin/brainstorms/2026-06-09-zoid-25-animation-system.md`

- Visual direction: use the current live Zoid 25 visual system, not the older cobalt-first direction.
  - Sumi-e / ink / paper / red seal.
  - Japanese editorial restraint.
  - Ruled panels, fine lines, hard rectangular controls.
- Intensity: rich tactile motion.
  - Every visible component/state should receive noticeable but tasteful motion.
  - No idle decorative looping except real live/attention states: streaming, loading, overflow cues, active progress.
- Rollout: exhaustive audit/inventory first, then ordered implementation batches.
- Scope: “everything user-visible” includes shell, pages, primitives, local/nested components, overlays, and state/content variants.
- Tech: CSS variables/classes/keyframes first; WAAPI/React refs only for geometry morphs; no new animation library unless the audit proves CSS/WAAPI is insufficient.
- Verification: source inventory, browser visual pass, reduced-motion pass, console clean, no overflow/clipping, build/frontend tests, native Tauri verification after batches, and feature critique gate before completion.

## Current context

Repo: `/Users/ziadnasreldin/Zoid`

Relevant source-of-truth files inspected:

- `/Users/ziadnasreldin/Zoid/PRODUCT.md`
- `/Users/ziadnasreldin/Zoid/DESIGN.md`
- `/Users/ziadnasreldin/Zoid/package.json`
- `/Users/ziadnasreldin/Zoid/src/App.tsx`
- `/Users/ziadnasreldin/Zoid/src/App.css`
- `/Users/ziadnasreldin/Zoid/src/agents/AgentsHermesScreen.tsx`
- `/Users/ziadnasreldin/Zoid/src/code/CodeWorkspace.tsx`
- `/Users/ziadnasreldin/brainstorms/2026-06-09-zoid-25-animation-system.md`

Live preview:

- `http://127.0.0.1:1420/` returned HTTP 200.
- Browser visual pass confirmed current visible system is ink/paper/red-seal editorial.

Repo state caveat:

- Working tree has substantial pre-existing modified/untracked work across app source and `.hermes/reviews/`.
- Animation implementation must preserve unrelated work and use scoped diffs/reviews.
- Do not assume all current files are clean baseline.

## Motion north star

Name: Editorial Ink Mechanics

The motion should feel like a serious local command OS made of ink, paper, rules, and mechanical panels.

Allowed character:

- Paper panels reveal/cut in.
- Ink rules draw or harden.
- Red seal marks confirm success/attention.
- Rows activate with a precise inset rail/wash.
- Buttons press like tactile printed controls.
- Panels morph calmly when geometry changes.
- Streaming/loading states breathe quietly because work is actually happening.

Forbidden character:

- Bouncy toy motion.
- Generic SaaS shimmer everywhere.
- AI magic sparkles.
- Constant ambient animation for decoration.
- Big cinematic page transitions that slow the command center.
- Motion that hides labels, fake state, or verification boundaries.

## Timing vocabulary

Create shared tokens in CSS before page-specific work.

Recommended token set:

- `--motion-instant`: 80ms for tiny color/opacity state changes.
- `--motion-micro`: 120ms for button press and icon nudges.
- `--motion-control`: 160ms for hover/focus/action controls.
- `--motion-row`: 200–240ms for nav/list row activation.
- `--motion-panel`: 320–420ms for popovers, dropdowns, panels.
- `--motion-structural`: 540ms for sidebar/session/file-manager geometry morphs.
- `--motion-page`: 620–820ms for page/workspace reveal sequences.

Recommended easing:

- `--ease-editorial`: `cubic-bezier(0.16, 1, 0.3, 1)` for most reveals/morphs.
- `--ease-press`: `cubic-bezier(0.25, 1, 0.5, 1)` for press/release.
- `--ease-rule`: `cubic-bezier(0.215, 0.61, 0.355, 1)` for line/ink reveal.

Reduced motion rule:

- Disable keyframes and nonessential transforms.
- Keep instant state changes: color, border, visibility, explicit labels.
- Keep focus outlines and state text fully visible.

## Source inventory for animation audit

### Global shell and app chrome

Files:

- `src/App.tsx`
- `src/App.css`
- `src/main.tsx`
- `src/scaffold.test.ts`

Visible surfaces:

- `.zoid25-shell`
- `.ink-rail`
- `.rail-menu`
- `.rail-lettermark`
- `.rail-language`
- `.rail-nav-item`
- `.nav-list`
- `.nav-row`
- `.nav-icon`
- `.nav-sigil`
- `.nav-state`
- `.status-dot`
- `.status-label-jp`
- `.app-startup-notice`
- lazy workspace loading fallback: “Loading Code workspace…” etc.
- unsupported placeholder workspaces for Today/Projects if still user-visible.

Animation requirements:

- rail/sidebar collapse and expand morphs stay structural and calm.
- nav hover/focus/active states get rich tactile row motion: ink rail draw, red seal accent, tiny icon line movement, status dot pulse only for state changes.
- startup notice enters/exits like a paper notice, not a toast gimmick.
- lazy-loading fallback receives a truthful loading animation with reduced-motion fallback.

### Shared UI primitives

Files:

- `src/ui/GlobalDropdown.tsx`
- `src/ui/GlobalDropdown.behavior.test.tsx`
- `src/App.css`

Visible surfaces:

- `.zoid-dropdown`
- `.zoid-dropdown-trigger`
- `.zoid-dropdown-chevron`
- `.zoid-dropdown-menu`
- `.zoid-dropdown-option`
- compact dropdown variants.

Animation requirements:

- trigger hover/focus/disabled/pressed states.
- chevron rotate/open state.
- menu reveal as paper drop with hard offset shadow.
- selected option ink rail draw.
- no clipping in dense panels.
- keyboard focus remains clear.

### Agents / Hermes workspace

Files:

- `src/agents/AgentsHermesScreen.tsx`
- `src/agents/AgentMonitorPanel.tsx`
- `src/agents/ChatComposer.tsx`
- `src/agents/MessageBubble.tsx`
- `src/agents/Avatar.tsx`
- `src/agents/CommandPalette.tsx`
- `src/agents/useAgentRuntime.ts`
- `src/agents/sessionState.ts`
- `src/agents/sessionPortraits.ts`
- `src/agents/dashboardLayoutState.ts`
- `src/App.css`

Visible surfaces:

- `.hermes-chat-shell`
- `.hermes-topbar`
- `.hermes-title-block`
- `.hermes-reference-line`
- `.topbar-status-stack`
- `.chat-workspace`
- `.chat-main-pane`
- `.chat-stage`
- `.chat-stats-strip`
- `.sessions-rail`
- `.sessions-rail-header`
- `.sessions-list`
- `.session-tab-row`
- `.session-tab`
- `.session-tab-portrait`
- `.session-notification-dot`
- `.session-reply-indicator`
- `.session-runtime-chip`
- `.session-dashboard-actions`
- `.message-list`
- `.message-row`
- `.message-bubble-frame`
- `.message-bubble`
- `.message-actions`
- `.message-action-button`
- `.message-status`
- `.message-writing-glyph`
- `.chat-composer`
- `.composer-*` controls/panels/dropups/attachment chips.
- `.file-manager-sidebar`
- `.file-manager-*` header, toolbar, rows, tree/list, resize handle, empty/error.
- `.command-palette-backdrop`
- `.command-palette`
- `.command-palette-option--active`
- `.zoid-native-command-panel`
- `.zoid-command-confirm*`
- agent monitor tiles in `AgentMonitorPanel.tsx`.

Animation requirements:

- session rail compact/expanded morph: preserve existing WAAPI snapshot pattern.
- session row activate/new/archive/restore/needs-reply states: rich but precise row feedback.
- session portrait hover/active: subtle ink paper lift; no replacing avatar semantics.
- message arrival: paper row reveal; assistant/user distinction remains clear.
- streaming: existing writing glyph stays, tuned to current ink system; no decorative idle when not streaming.
- copy/success/error: explicit text plus red-seal/success stamp micro-feedback.
- composer: textarea expansion, slash dropup, send/stop morph, attachment chips, disabled/blocked states.
- file manager: open/close, resize handle, folder row, selected/open folder, loading/error/empty.
- command palette: backdrop, panel, active option, execution confirmation, command run result.
- monitor dashboard: tile focus/primary/running/stopped/needs-input/continue/remove/move states.

### Code workspace

Files:

- `src/code/CodeWorkspace.tsx`
- `src/code/repositoryOperations.ts`
- `src/code/repositoryClient.ts`
- `src/code/types.ts`
- `src/code/CodeWorkspace.behavior.test.tsx`
- `src/code/repositoryOperations.test.ts`
- `src/App.css`

Visible surfaces:

- `.code-workspace-shell`
- `.code-workspace-header`
- `.code-reference-line`
- `.code-ink-mark`
- `.code-repository-layout`
- `.repository-list-panel`
- `.repository-list-title-row`
- `.repository-search-morph`
- `.repository-search-toggle`
- `.repository-search-field`
- `.repository-card-list`
- `.repository-card`
- `.repository-card-heading`
- `.repo-meta-grid`
- `.repo-meta-action-row`
- `.default-branch-editor`
- `.repo-action-panel`
- `.repo-control-grid`
- `.repo-scan-feedback`
- `.repo-action-feedback`
- `.repository-operation-button`
- production confirmation flow currently uses `window.confirm` in `confirmProductionRepositoryOperation`.

Animation requirements:

- repository empty state: calm paper field and guide-line reveal.
- search open/close: keep current morph, enhance with accessible focus/close states.
- scan/clone panels: input/folder-picker/action feedback states.
- scan/clone success: highlight newly added repository card/row.
- repository cards: row/card reveal, hover, dirty/clean state change, operation button press/loading/disabled.
- default branch edit dropdown: scoped open/save/cancel feedback without clipping.
- production confirmation should eventually become branded modal before final polished completion, not native `window.confirm`.

### Brain workspace

Files:

- `src/brain/BrainWorkspace.tsx`
- `src/brain/brainClient.ts`
- `src/brain/types.ts`
- `src/brain/BrainWorkspace.behavior.test.tsx`
- `src/App.css`

Visible surfaces found by CSS inventory:

- `.brain-sumi-e`
- `.brain-workspace-shell`
- `.brain-hero`
- `.brain-ink-mark`
- `.brain-panel`
- `.brain-panel-heading`
- `.brain-note-row`
- `.brain-source-row`
- `.brain-session-tab`
- `.brain-question-card`
- `.brain-actions`
- `.brain-badge`
- `.brain-placeholder-strip`
- `.brain-bridge-error`
- `.brain-primary-action`
- `.brain-secondary-action`

Animation requirements:

- note/source row entry and selection.
- bridge error/blocked state feedback.
- empty/placeholder truth states.
- action buttons and badges.
- page reveal matching sumi-e system.

### Content workspace

Files:

- `src/content/ContentWorkspace.tsx`
- `src/content/contentModel.ts`
- `src/content/contentWorkspace.test.ts`
- `src/App.css`

Visible surfaces found by CSS inventory:

- `.content-workspace`
- `.content-workspace-grid`
- `.content-hero`
- `.content-panel-heading`
- `.content-left-panel`
- `.content-center-panel`
- `.content-right-panel`
- `.content-evidence-card`
- `.content-piece-card`
- `.content-draft-card`
- `.content-provider-card`
- `.content-asset-row`
- `.content-action-row`
- `.content-gate-strip`
- `.content-fail-actions`

Animation requirements:

- panel entry and column rhythm.
- draft/evidence/provider/asset card hover/active/status.
- gate/fail state feedback with explicit text.
- action row press/loading/success.

### Automations workspace

Files:

- `src/automations/AutomationsWorkspace.tsx`
- `src/automations/automationClient.ts`
- `src/automations/automationViewModel.ts`
- `src/automations/types.ts`
- `src/automations/AutomationsWorkspace.behavior.test.tsx`
- `src/App.css`

Visible surfaces found by CSS inventory:

- `.automation-sumi-e`
- `.automations-workspace-header`
- `.automation-ink-clock`
- `.automation-toolbar`
- `.automation-header-actions`
- `.automation-summary-grid`
- `.automation-summary-card`
- `.automation-filter-tabs`
- `.automation-section-labels`
- `.automation-edge-panel`
- `.automation-next-run-card`
- `.automation-truth-card`
- `.automation-command-panel`
- `.automation-action-row`
- `.automation-primary-button`
- `.automation-confirm-panel`
- `.automation-confirm-actions`

Animation requirements:

- routine row/card state changes.
- run/pause/command/blocked/confirm states.
- filter tab active movement.
- summary card value change feedback.
- command panel/dropdown/modal transitions with no clipping.

### Settings and Providers

Files:

- `src/App.tsx` for `SettingsArchive`
- `src/providers/ProvidersSettings.tsx`
- `src/providers/providerClient.ts`
- `src/App.css`

Visible surfaces:

- `.settings-sumi-e`
- `.settings-hero`
- `.settings-reference-line`
- `.settings-archive-shell`
- `.settings-archive-header`
- `.archived-session-list`
- `.archived-session-card`
- `.archive-session-button`
- `.archive-session-select`
- `.archive-bulk-actions`
- `.settings-confirm-backdrop`
- `.settings-confirm-panel`
- `.settings-confirm-actions`
- `.providers-settings-section`
- `.providers-manager-grid`
- `.provider-list`
- `.provider-card`
- `.provider-card--validated`
- `.provider-card--invalid`
- `.provider-editor-card`
- `.provider-form-grid`
- `.provider-status-badge`
- `.provider-action-row`
- `.provider-error-note`

Animation requirements:

- tabs/sections first-fold stays stable.
- archive select/restore/delete/bulk-delete with branded accessible modal feedback.
- provider validation/save/error/success states.
- destructive dialogs must be accessible and async-aware.

### Global states and overlays

Files:

- `src/App.css`
- component files above.

User-visible states to cover everywhere:

- default
- hover
- focus-visible
- active/pressed
- selected/current
- disabled
- loading/checking/sending/scanning/cloning/saving
- empty
- error/blocked/offline
- success/ready/validated/copied
- warning/dirty/needs-attention
- archived/restored/deleted
- collapsed/expanded
- resizing
- streaming/running/stopped/needs-input
- long content/overflow
- reduced motion

## Existing motion to preserve/refactor, not duplicate

Known keyframes in `src/App.css`:

- `file-manager-panel-enter`
- `sessions-overflow-cue-flow`
- `sessions-overflow-cue-sheen`
- `hermes-writing-pulse`
- `hermes-writing-orbit`
- `hermes-genm-reveal`
- `hermes-genm-mark-reveal`
- `agents-ink-reveal`
- `agents-mark-reveal`
- `repository-search-morph-in`
- `repo-added-flash`
- `code-ink-reveal`
- `code-mark-reveal`
- `profile-tab-panel-enter`
- `automation-ink-reveal`
- `automation-mark-reveal`
- `brain-ink-reveal`
- `brain-mark-reveal`
- `settings-ink-reveal`
- `settings-mark-reveal`

Plan implication:

- Consolidate repeated page reveal/keyframe vocabulary into shared motion tokens and reusable classes where safe.
- Keep page-specific names only when page identity/motif differs.
- Do not create a second competing animation language.

## Build order

### Build Order 0 — freeze audit baseline

Purpose: produce the complete animation map before implementation.

Tasks:

1. Read every user-visible TSX component listed above.
2. For each component, create a row in `.hermes/reviews/zoid-25-animation-system/animation-inventory.md` with:
   - file path
   - component/surface name
   - visible states
   - current motion, if any
   - proposed motion
   - reduced-motion behavior
   - verification target
3. Classify all current CSS keyframes/transitions into:
   - keep
   - consolidate
   - replace
   - remove/disable under reduced motion
4. Record dirty-tree scope and unrelated files before edits.

Exit criteria:

- Every TSX component and every meaningful CSS surface prefix above appears in the inventory by name/path.
- No implementation yet.

### Build Order 1 — global motion foundation

Purpose: introduce one coherent motion language.

Likely files:

- `src/App.css`
- possibly `src/scaffold.test.ts`

Tasks:

1. Add motion variables under `:root`:
   - durations
   - easings
   - transform distances
   - shadow offsets
2. Add reusable utility classes or scoped patterns for:
   - paper reveal
   - ink rule draw
   - red seal stamp
   - tactile press
   - row activation
   - panel enter/exit
   - loading pulse
3. Centralize `@media (prefers-reduced-motion: reduce)` handling.
4. Avoid changing component DOM except where necessary to expose existing state classes.

Validation:

- `npm run build`
- focused source grep verifying no uncontrolled duplicate timing constants were added.
- browser check shell loads.

### Build Order 2 — shell, rail, nav, shared primitives

Purpose: animate the app frame and core controls first.

Likely files:

- `src/App.css`
- `src/App.tsx`
- `src/ui/GlobalDropdown.tsx`
- `src/ui/GlobalDropdown.behavior.test.tsx`
- `src/scaffold.test.ts`

Tasks:

1. Enhance rail/menu/sidebar/nav row motion.
2. Add active workspace transition handling without hiding content truthfully.
3. Animate `StatusDot` changes and startup/lazy loading notice.
4. Enhance `GlobalDropdown` open/close/hover/focus/selected states.
5. Add reduced-motion assertions where feasible.

Validation:

- frontend tests covering scaffold and dropdown.
- browser click through all nav rows.
- console clean.
- reduced-motion manual/DOM check.

### Build Order 3 — Agents/Hermes workspace

Purpose: animate the densest and highest-value operational surface.

Likely files:

- `src/agents/AgentsHermesScreen.tsx`
- `src/agents/AgentMonitorPanel.tsx`
- `src/agents/ChatComposer.tsx`
- `src/agents/MessageBubble.tsx`
- `src/agents/CommandPalette.tsx`
- `src/agents/Avatar.tsx`
- `src/App.css`
- relevant agent tests.

Tasks:

1. Preserve and refine session rail WAAPI morph.
2. Animate session rows: new, selected, needs-reply, running, archived/restored.
3. Animate message appearance, actions, copy/error/success states.
4. Animate composer: textarea expansion, send/stop, slash command dropup, attachment chips, settings popovers.
5. Animate command palette and native command confirmation surfaces.
6. Animate file manager: open/close, resize, folder rows, empty/error.
7. Animate agent monitor tiles: focus, primary, running, needs-input, stopped, continue, remove, move.

Validation:

- `npm run test:frontend` or focused agent tests first.
- browser Agents pass with multiple sessions and dashboard mode.
- no output mixing, no clipped dropdowns/popovers.
- reduced motion pass.

### Build Order 4 — Code workspace

Purpose: animate repository operations and state feedback.

Likely files:

- `src/code/CodeWorkspace.tsx`
- `src/code/repositoryOperations.ts`
- `src/App.css`
- code tests.

Tasks:

1. Enhance page/header/list/action panel reveal.
2. Animate search morph and repository list empty state.
3. Animate scan/clone feedback and newly-added repository highlight.
4. Animate repository cards/metadata/default branch editor.
5. Convert production `window.confirm` to branded modal before final polish if still present.
6. Animate operation buttons/runbook state without implying deploy success.

Validation:

- repository behavior tests.
- browser Code pass with empty and seeded localStorage repository states.
- no native bridge assumptions unless verified.

### Build Order 5 — Brain, Content, Automations

Purpose: apply the system across the remaining workspaces.

Likely files:

- `src/brain/BrainWorkspace.tsx`
- `src/content/ContentWorkspace.tsx`
- `src/automations/AutomationsWorkspace.tsx`
- `src/App.css`
- relevant tests.

Tasks:

1. Brain: note/source/question/action/bridge states.
2. Content: evidence/piece/draft/provider/asset/gate states.
3. Automations: routines, filters, summaries, command panels, confirmations.
4. Keep page identity, but use common timing/easing vocabulary.

Validation:

- focused workspace tests.
- browser pass for each nav item.
- no horizontal overflow or clipped panels.

### Build Order 6 — Settings, providers, archive/destructive flows

Purpose: finish administrative and destructive surfaces.

Likely files:

- `src/App.tsx`
- `src/providers/ProvidersSettings.tsx`
- `src/App.css`
- relevant tests.

Tasks:

1. Animate settings hero/sections without pushing controls below fold.
2. Animate archive selection/restoration/deletion/bulk actions.
3. Animate provider cards, validation, errors, save states.
4. Ensure destructive dialogs are branded, accessible, async-aware, focus-safe.

Validation:

- provider/settings behavior tests where present or add focused tests.
- keyboard/focus pass.
- reduced-motion pass.

### Build Order 7 — exhaustive QA, native verification, critique gate

Purpose: prove “everything user-visible” coverage.

Tasks:

1. Update inventory status from proposed to implemented/verified for every row.
2. Run:
   - `npm run build`
   - `npm run test:frontend`
   - `npm run test:rust` if native commands touched
3. Browser visual pass at `http://127.0.0.1:1420/`:
   - all nav workspaces
   - all visible overlays/dropdowns/modals
   - console output clean except known dev overlay noise if documented
   - no horizontal overflow/clipping
   - reduced-motion check
4. Native Tauri verification after implementation batches:
   - Tauri dev or installed app depending final target.
   - Confirm animations visible in native app, not only Vite browser preview.
5. Create `.hermes/reviews/zoid-25-animation-system/handoff.md`.
6. Run the feature critique workflow.
7. Fix Required items and re-review until APPROVED.

## Test and verification commands

Baseline commands from `package.json`:

- `npm run build`
- `npm run test:frontend`
- `npm run test:rust`
- `npm run test`
- `npm run dev`
- `npm run tauri:dev`
- `npm run tauri:build`

Browser checks:

- `curl -I --max-time 5 http://127.0.0.1:1420/`
- Browser DOM/visual pass on `http://127.0.0.1:1420/`
- Browser console read after interactions.
- Reduced-motion check using emulated `prefers-reduced-motion` if available, or CSS/DOM inspection plus native system setting/manual check.

Inventory proof:

- Every file in `src/**/*.tsx` that renders visible UI must have an inventory row.
- Every major CSS prefix/class group from `src/App.css` must map to an inventory row or be marked non-user-visible/dead.

## Risks and guardrails

- Risk: broad animation churn can break existing handlers.
  - Guardrail: preserve DOM/handlers; prefer CSS state classes over wrapper rewrites.

- Risk: visual noise from “animate everything”.
  - Guardrail: rich tactile motion but no idle decorative loops except live states.

- Risk: accessibility regression.
  - Guardrail: centralized reduced-motion and focus-visible preservation.

- Risk: dropdown/modal clipping in dense fixed shell.
  - Guardrail: explicitly test GlobalDropdown, composer panels, command palette, file manager, settings/provider dialogs.

- Risk: current dirty tree obscures scope.
  - Guardrail: record scoped diffs and isolate animation changes in review handoff.

- Risk: browser preview differs from native Tauri.
  - Guardrail: native verification is required before completion.

- Risk: animations imply completed work or fake live data.
  - Guardrail: motion can highlight state changes only when state is real and labeled.

## Immediate next action when implementation is approved

Start with Build Order 0 only:

1. Create `.hermes/reviews/zoid-25-animation-system/animation-inventory.md`.
2. Read all TSX visible components and relevant CSS sections.
3. Fill inventory rows with current/proposed/reduced-motion/verification mapping.
4. Stop for review before applying animation code.

Implementation should not begin until the inventory is complete and accepted.
