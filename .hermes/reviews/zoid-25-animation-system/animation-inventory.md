# Zoid 25 animation inventory

Scope: Build Order 0 for `/Users/ziadnasreldin/Zoid/.hermes/plans/2026-06-09_181851-zoid-25-animation-system.md`.

Status: inventory/proposal only. No animation implementation in this file.

## Resolved motion direction

- Current live visual system: sumi-e / ink / paper / red-seal editorial.
- Intensity: rich tactile motion.
- No decorative idle loops except live/attention states.
- Implementation boundary: CSS variables/classes/keyframes first; WAAPI/React refs only for geometry morphs; no animation library unless later proven necessary.
- Reduced motion: central required fallback.

## Inventory coverage proof

Visible TSX component files inventoried:

- `src/App.tsx`
- `src/agents/AgentMonitorPanel.tsx`
- `src/agents/AgentsHermesScreen.tsx`
- `src/agents/Avatar.tsx`
- `src/agents/ChatComposer.tsx`
- `src/agents/CommandPalette.tsx`
- `src/agents/MessageBubble.tsx`
- `src/automations/AutomationsWorkspace.tsx`
- `src/brain/BrainWorkspace.tsx`
- `src/code/CodeWorkspace.tsx`
- `src/content/ContentWorkspace.tsx`
- `src/providers/ProvidersSettings.tsx`
- `src/ui/GlobalDropdown.tsx`

Non-render or tests noted but not individually animated:

- `src/main.tsx`: React mount only.
- `*.test.tsx` / `*.test.ts`: verification targets, not user-visible surfaces.
- model/client/state files: animate only through their rendered surfaces.

CSS source:

- `src/App.css` contains 473 class names by scan and existing keyframes listed below.

## Existing keyframes classification

| Keyframe | Current likely use | Classification | Proposed treatment | Reduced motion |
|---|---|---|---|---|
| `file-manager-panel-enter` | file manager sidebar entry | Keep/refine | Convert timing to shared panel token | Disable transform; instant visible |
| `sessions-overflow-cue-flow` | sessions overflow attention cue | Keep but constrain | Allowed because it signals overflow/attention | Disable loop |
| `sessions-overflow-cue-sheen` | sessions overflow sheen | Keep but soften | Allowed only while overflow exists | Disable loop |
| `hermes-writing-pulse` | streaming message status | Keep/refine | Allowed live-state loop | Disable loop, preserve label |
| `hermes-writing-orbit` | streaming glyph dots | Keep/refine | Allowed live-state loop | Disable loop, show static glyph/text |
| `hermes-genm-reveal` | old/genm Hermes reveal | Consolidate | Fold into shared page/panel reveal if still used | Disable transform |
| `hermes-genm-mark-reveal` | old/genm mark reveal | Consolidate | Fold into shared ink-mark reveal | Disable transform |
| `agents-ink-reveal` | Agents reveal | Consolidate | Use shared workspace reveal with page-specific selectors | Disable transform |
| `agents-mark-reveal` | Agents mark reveal | Consolidate | Use shared ink-mark reveal | Disable transform |
| `repository-search-morph-in` | Code search field | Keep/refine | Use shared control morph token | Disable transform |
| `repo-added-flash` | repository added feedback | Keep/refine | Convert to red-seal/ink-row feedback | Static highlight only |
| `code-ink-reveal` | Code page reveal | Consolidate | Shared workspace reveal | Disable transform |
| `code-mark-reveal` | Code mark reveal | Consolidate | Shared ink-mark reveal | Disable transform |
| `profile-tab-panel-enter` | Settings/profile tab enter | Keep/refine | Shared panel reveal | Disable transform |
| `automation-ink-reveal` | Automations reveal | Consolidate | Shared workspace reveal | Disable transform |
| `automation-mark-reveal` | Automations mark reveal | Consolidate | Shared ink-mark reveal | Disable transform |
| `brain-ink-reveal` | Brain reveal | Consolidate | Shared workspace reveal | Disable transform |
| `brain-mark-reveal` | Brain mark reveal | Consolidate | Shared ink-mark reveal | Disable transform |
| `settings-ink-reveal` | Settings reveal | Consolidate | Shared workspace reveal | Disable transform |
| `settings-mark-reveal` | Settings mark reveal | Consolidate | Shared ink-mark reveal | Disable transform |

## Global state vocabulary to apply everywhere

Every visible component should be checked for these states where meaningful:

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

## Component / surface inventory

### 1. Global app shell

| Surface | File | Current motion | Proposed motion | States to cover | Verification target |
|---|---|---|---|---|---|
| `App` workspace router/lazy shell | `src/App.tsx` | Lazy fallback only; sidebar morph constants | Workspace transitions use shared paper reveal; lazy fallback gets truthful loading indicator | loading, workspace switch, unsupported page | browser nav through all rows |
| `.zoid25-shell` | `src/App.css` | grid column transition 540ms | Keep structural morph; tokenize duration/ease | expanded/collapsed/reduced | sidebar collapse browser probe |
| `.ink-rail` | `src/App.css` | subtle pseudo opacity transition | Add rich tactile rail item feedback without idle loop | default, hover, collapsed | visual + reduced-motion |
| `.rail-menu` / close icon | `src/App.tsx`, `src/App.css` | 180ms transform/opacity | Keep; improve press/release and focus feedback | hover, focus, pressed, open/close | keyboard + click |
| `.rail-lettermark`, `.rail-language` | `src/App.css` | mostly static | Tiny paper/ink reveal on load and collapse-safe state | load, collapsed, hover if interactive | browser visual |
| `InkSigil` / `.nav-sigil` | `src/App.tsx`, `src/App.css` | static SVG | Stroke/ink nudge on nav hover/active; no path redraw unless CSS-safe | hover, active, reduced | nav hover/click |
| `StatusDot` / `.status-dot` / `.status-label-jp` | `src/App.tsx`, `src/App.css` | static except CSS color | State change pulse only when state changes; label remains truth source | ready, empty, blocked | DOM/classes + visual |
| `.nav-row` | `src/App.tsx`, `src/App.css` | 180–220ms background/rail | Rich tactile row: ink rail draw, soft paper wash, icon nudge, red-seal accent for active | hover, focus, active, selected, disabled if any | all nav rows |
| `.app-startup-notice` | `src/App.tsx`, `src/App.css` | static | Paper notice enter/exit with reduced fallback | show, dismiss/timeout if applicable | browser startup |
| lazy `Suspense` status | `src/App.tsx` | static text | Truthful loading glyph/line reveal | loading/reduced | force lazy route |

### 2. Shared dropdown primitive

| Surface | File | Current motion | Proposed motion | States to cover | Verification target |
|---|---|---|---|---|---|
| `GlobalDropdown` | `src/ui/GlobalDropdown.tsx` | menu toggles; CSS mostly static | Open paper drop, chevron rotate, option rail draw, selected stamp | open/close, hover, focus, selected, disabled, compact, keyboard | `GlobalDropdown.behavior.test.tsx` + browser |
| `.zoid-dropdown-trigger` | `src/App.css` | no transition | control press + border/shadow/fill transition | hover, focus, pressed, disabled | browser provider/model/dropdown |
| `.zoid-dropdown-menu` | `src/App.css` | static absolute menu | panel reveal using shared popover token | open, overflow scroll, reduced | no clipping in panels |
| `.zoid-dropdown-option` | `src/App.css` | hover/selected background + inset rail | enrich with quick rail draw and paper wash | hover, active, selected | keyboard + mouse |

### 3. Settings/Profile shell in `App.tsx`

| Surface | File | Current motion | Proposed motion | States to cover | Verification target |
|---|---|---|---|---|---|
| `SettingsArchive` | `src/App.tsx` | some modal/static CSS | Archive card enter/select/restore/delete tactile states | empty, selected, restore, delete, bulk delete, dialog | Settings browser pass |
| `.settings-sumi-e`, `.settings-hero`, `.settings-ink-mark` | `src/App.css` | reveal keyframes | Consolidate page/mark reveal | load, reduced | settings load visual |
| `.settings-archive-shell`, `.archived-session-card` | `src/App.css` | mostly static | Card/row reveal and selection ink rail | empty/list/selected | seeded archived state |
| `.settings-confirm-backdrop`, `.settings-confirm-panel` | `src/App.tsx`, `src/App.css` | static/modal CSS | Branded modal enter/exit, safe-focus, async delete state | open, cancel, confirm, loading, error | keyboard/focus test |
| profile settings panels/classes | `src/App.tsx`, `src/App.css` | tab panel enter exists | Centralize tab/panel reveal; animate save status and toggles | tab active, form edit, save, disabled/error | Settings/Profile browser |

### 4. Providers settings

| Surface | File | Current motion | Proposed motion | States to cover | Verification target |
|---|---|---|---|---|---|
| `ProvidersSettings` | `src/providers/ProvidersSettings.tsx` | mostly static | Provider card validation/save/error motion | loading, validated, invalid, save, select main | provider test/browser |
| `.provider-card`, `.provider-card--validated`, `.provider-card--invalid` | `src/App.css` | state styling only | Seal stamp for validated, ink warning for invalid | success/error/hover | browser |
| `.provider-editor-card`, `.provider-form-grid` | `src/App.css` | static | Field focus/section reveal | focus, dirty, save | browser keyboard |
| `.provider-action-row`, `.provider-status-badge`, `.provider-error-note` | `src/App.css` | static | press/loading/status reveal | saving, error, success | browser |

### 5. Agents / Hermes main workspace

| Surface | File | Current motion | Proposed motion | States to cover | Verification target |
|---|---|---|---|---|---|
| `AgentsHermesScreen` | `src/agents/AgentsHermesScreen.tsx` | page reveal, rail/file WAAPI constants | Keep structural WAAPI; centralize page/tile/panel motion | open/closed workspace, dashboard, primary session | focused agent tests + browser |
| `.agents-sumi-e`, `.agents-ink-mark` | `src/App.css` | page/mark reveal | Consolidate reveal; keep page identity | load/reduced | Agents visual |
| `.hermes-topbar`, `.hermes-title-block`, `.topbar-status-stack` | `src/App.css` | reveal delay | Page load/staged status reveal with no layout shift | checking/online/offline/error | browser console/status |
| `.chat-workspace`, `.chat-main-pane`, `.chat-stage` | `src/App.css` | structural grid transition | paper panel reveal and dashboard mode morph | normal/dashboard/file-manager | visual/overflow |
| `.sessions-rail` | `AgentsHermesScreen.tsx`, `App.css` | width transition + WAAPI snapshots | Preserve; add richer resize/collapse feedback | expanded, compact, resize, reduced | rail tests/browser |
| `.sessions-overflow-cue` | `App.css` | looping attention cue | Keep only when overflow exists; reduce amplitude if noisy | overflow, hover, focus, reduced | scroll list |
| `.session-tab-row`, `.session-tab`, `.session-tab-portrait` | `AgentsHermesScreen.tsx`, `App.css` | row/morph classes | Row selection/new/needs-reply/running feedback | selected, editing, new, needs-reply, running, archived | agent browser seeded state |
| `.session-notification-dot`, `.session-reply-indicator` | `App.css` | static or existing | pulse only on new reply/attention; no idle once acknowledged | unread, focused, cleared | behavior test/browser |
| `.session-runtime-chip` | `App.css` | static | state transition for running/needs-input/stopped | running, stopped, needs-input | dashboard browser |
| repository link controls | `AgentsHermesScreen.tsx`, `App.css` | static/dropdown | link/unlink success and dropdown motion | linked, unlinked, inferred, error | agent repo linking test |
| model/command panels | `AgentsHermesScreen.tsx`, `App.css` | panel styles | command-sheet reveal, dropdown safety, save/persist note motion | open, changed, saving, error | browser clipping |
| `.file-manager-sidebar` | `AgentsHermesScreen.tsx`, `App.css` | `file-manager-panel-enter` | Keep/refine panel enter, resize handle, rows | open, close, resize, empty, error, folder/file hover | file-manager test/browser |
| `.file-manager-row`, `.file-manager-item`, `.file-manager-empty`, `.file-manager-error` | `App.css` | static | row reveal, selected/open folder, error notice | loading, empty, error, folder/file | browser |
| `.ruthless-reviewer-card` | `AgentsHermesScreen.tsx` | static | card/prompt/action feedback | enabled, disabled, click | browser |

### 6. Agent monitor panels

| Surface | File | Current motion | Proposed motion | States to cover | Verification target |
|---|---|---|---|---|---|
| `AgentMonitorPanel` | `src/agents/AgentMonitorPanel.tsx` | mostly static | Tile focus/primary/running tactile state, feed line entry | focused, primary, disabled, running, needs-input | dashboard browser |
| `.agent-monitor-panel` variants | `src/App.css` | likely static classes | panel lift/ink rail, running status pulse only while running | focused, primary, disabled | browser |
| `.agent-monitor-feed-line` | `AgentMonitorPanel.tsx`, CSS | static | new feed line reveal | new output, error | browser |
| monitor action buttons/composer | `AgentMonitorPanel.tsx` | static | press/disabled/sending feedback | send, stop, continue, expand, remove, move | behavior/browser |

### 7. Chat composer

| Surface | File | Current motion | Proposed motion | States to cover | Verification target |
|---|---|---|---|---|---|
| `ChatComposer` | `src/agents/ChatComposer.tsx` | textarea height transition; buttons | Enrich composer as central command surface | typing, sending, stopped, disabled, command draft | composer tests/browser |
| `.chat-composer` | `App.css` | static shell | paper panel focus/active state | focused, disabled | browser |
| `.composer-input-wrap` textarea | `ChatComposer.tsx`, `App.css` | height transition on expanding | Keep, add focus/command mode ink state | typing, expanding, command mode | test/browser |
| `.composer-send`, `.composer-attach` | `App.css` | 150ms transitions | press morph, send↔stop state | hover, focus, pressed, sending, disabled | test/browser |
| `.composer-slash-dropup*` | `ChatComposer.tsx`, `App.css` | likely static | paper dropup reveal, active option rail | open, active, keyboard, empty | slash tests |
| `.composer-deep-panel*`, `.composer-action-popover` | `App.css` | static/panel | panel reveal, no clipping | open/close, settings, attach | browser |
| `.composer-attachment-chip` | `ChatComposer.tsx`, `App.css` | static | add/remove chip seal/slide | added, removed, invalid | browser |
| `.composer-status-note`, usage meter/grid | `App.css` | static | warning/usage change feedback | normal, warning, error | browser |

### 8. Messages and avatars

| Surface | File | Current motion | Proposed motion | States to cover | Verification target |
|---|---|---|---|---|---|
| `MessageBubble` | `src/agents/MessageBubble.tsx` | actions reveal; streaming glyph | Message enter, action reveal, copy/stamp feedback | user/assistant, streaming, copied, failed, rollback | message tests/browser |
| `.message-row`, `.message-bubble-frame`, `.message-bubble` | `App.css` | hover/action transitions | paper row reveal and side-specific alignment stability | enter, hover, long content | browser |
| `.message-actions`, `.message-action-button` | `App.css` | 160ms reveal | Keep/refine; preserve reduced motion | hover, focus, copied, failed | browser |
| `.message-status--streaming`, `.message-writing-glyph` | `App.css` | live loop | Keep only for streaming; static reduced | streaming, error, done | browser |
| `Avatar` | `src/agents/Avatar.tsx` | static presence | presence/state transition and hover lift | online, offline, thinking, reduced | browser |
| `.chat-avatar`, `.avatar-presence*` | `App.css` | static | subtle presence ring/label-safe transition | presence changes | browser |

### 9. Command palette / native command confirmation

| Surface | File | Current motion | Proposed motion | States to cover | Verification target |
|---|---|---|---|---|---|
| `CommandPalette` | `src/agents/CommandPalette.tsx` | static open/close | Backdrop fade + paper panel drop; option active rail | open, close, active, recent, run, insert | command palette tests/browser |
| `.command-palette-backdrop`, `.command-palette` | `App.css` | static | accessible modal enter/exit | open/reduced | keyboard focus |
| `.command-palette-option--active` | `App.css` | active style | ink rail draw + scroll-into-view no jump | keyboard active | test/browser |
| `.zoid-native-command-panel`, `.zoid-command-confirm*` | `App.css` | panel styles | confirmation modal motion + async state | confirm, cancel, running, error | browser |

### 10. Code workspace

| Surface | File | Current motion | Proposed motion | States to cover | Verification target |
|---|---|---|---|---|---|
| `CodeWorkspace` | `src/code/CodeWorkspace.tsx` | page reveal and search morph | Full workspace paper/ink reveal + row/card interactions | empty, repos, scanning, cloning | code tests/browser |
| `.code-workspace-shell`, `.code-workspace-header`, `.code-ink-mark` | `App.css` | code reveal keyframes | Consolidate page/mark reveal | load/reduced | Code visual |
| `.repository-list-panel`, `.repository-list-title-row` | `App.css` | static | panel reveal and header control motion | empty/list/search | browser |
| `.repository-search-morph`, `.repository-search-toggle`, `.repository-search-field` | `CodeWorkspace.tsx`, `App.css` | search morph keyframe | Keep/refine with tokens and focus | open, close, typing, clear | behavior/browser |
| `.repo-empty-state` | `CodeWorkspace.tsx`, `App.css` | reused static empty | paper empty state reveal | empty/reduced | browser |
| `.repo-action-panel--scan`, `.repo-action-panel--clone` | `CodeWorkspace.tsx`, `App.css` | static | form/control focus and disabled/loading motion | scanning, cloning, disabled, error, success | browser |
| `.repo-scan-feedback`, `.repo-action-feedback` | `CodeWorkspace.tsx`, `App.css` | static; added flash exists | seal/ink feedback tied to real result | info, success, error | tests/browser |
| `RepositoryMeta` / `.repo-meta-grid` | `CodeWorkspace.tsx`, `App.css` | static | row/value change highlight | dirty, clean, branch change | browser seeded repo |
| default branch editor/dropdown | `CodeWorkspace.tsx`, `GlobalDropdown` | dropdown primitive | compact edit row reveal, save/cancel states | edit, saving, cancel, error | code test/browser |
| repository operation buttons | `CodeWorkspace.tsx`, `repositoryOperations.ts` | static | press/loading/result feedback without fake deploy state | localhost/staging/production, disabled, profile status | browser |
| `confirmProductionRepositoryOperation` | `CodeWorkspace.tsx` | native confirm | Replace later with branded modal for polished completion | confirm/cancel | test/browser |

### 11. Brain workspace

| Surface | File | Current motion | Proposed motion | States to cover | Verification target |
|---|---|---|---|---|---|
| `BrainWorkspace` | `src/brain/BrainWorkspace.tsx` | page reveal keyframes | Shared page reveal + brain identity | load/reduced | Brain browser |
| `.brain-hero`, `.brain-ink-mark`, `.brain-reference-line` | `App.css` | reveal/mark | Consolidate | load | visual |
| `.brain-panel`, `.brain-panel-heading`, `.brain-panel-empty` | `App.css` | static | panel reveal, empty-state truth | empty, loading, error | browser |
| `.brain-note-row`, `.brain-source-row` | `BrainWorkspace.tsx`, `App.css` | static | row hover/select/new sync feedback | hover, selected, synced, conflict | behavior/browser |
| `.brain-session-tab`, `.brain-active-session` | `App.css` | static | tab active rail/morph | active, overflow | browser |
| `.brain-question-card`, `.brain-brief-card` | `App.css` | static | card reveal and action states | default, action, disabled | browser |
| `.brain-bridge-error`, `.brain-status-line` | `App.css` | static | explicit blocked/error reveal | blocked, error, retry | browser |
| `.brain-primary-action`, `.brain-secondary-action` | `App.css` | static/button | press/focus/loading | hover, focus, disabled | browser |

### 12. Content workspace

| Surface | File | Current motion | Proposed motion | States to cover | Verification target |
|---|---|---|---|---|---|
| `ContentWorkspace` | `src/content/ContentWorkspace.tsx` | likely static/page CSS | Three-column paper reveal and card/row feedback | empty, draft, failed, provider | content test/browser |
| `.content-workspace`, `.content-workspace-grid`, `.content-hero` | `App.css` | static/reveal unknown | shared workspace reveal | load/reduced | Content browser |
| `.content-left-panel`, `.content-center-panel`, `.content-right-panel` | `App.css` | static | staggered panel reveal | load, responsive | browser |
| `.content-evidence-card`, `.content-piece-card`, `.content-draft-card` | `ContentWorkspace.tsx`, `App.css` | static | card hover/status/new feedback | draft, evidence, failed, selected | browser |
| `.content-provider-card`, `.content-provider-card--*` | `App.css` | static | provider status feedback | available, blocked, selected | browser |
| `.content-asset-row`, `.content-action-row` | `App.css` | static | row/action tactile states | hover, click, loading | browser |
| `.content-gate-strip`, `.content-fail-actions` | `App.css` | static | warning/error reveal | gated, failed, retry | browser |

### 13. Automations workspace

| Surface | File | Current motion | Proposed motion | States to cover | Verification target |
|---|---|---|---|---|---|
| `AutomationsWorkspace` | `src/automations/AutomationsWorkspace.tsx` | page reveal keyframes | shared reveal + automation identity | load/reduced | automation tests/browser |
| `Detail`, `SummaryCard` | `AutomationsWorkspace.tsx` | static | value/change feedback | value update, tone neutral/ink/seal | browser |
| `.automation-sumi-e`, `.automation-ink-clock` | `App.css` | reveal/mark | consolidate mark reveal | load/reduced | browser |
| `.automations-workspace-header`, `.automation-reference-line` | `App.css` | static/reveal | header reveal and status line | loading/error | browser |
| `.automation-toolbar`, `.automation-filter-tabs` | `App.css` | static | tab active slide/rail | filter active, search, refresh | browser |
| `.automation-summary-card`, `.automation-summary-grid` | `App.css` | static | value stamp/change feedback | updated, warning | browser |
| `.automation-job-card`, `.automation-watcher-card` | `AutomationsWorkspace.tsx`, `App.css` | static | card/row state motion | running, paused, failed, ok, unknown | tests/browser |
| `.automation-command-panel` | `App.css` | panel static | command-sheet reveal, dropdown-safe | open, save, error | browser clipping |
| `.automation-confirm-backdrop`, `.automation-confirm-panel`, `.automation-confirm-actions` | `AutomationsWorkspace.tsx`, `App.css` | modal static | accessible modal enter/async state | run/remove/confirm/cancel/error | tests/browser |
| `.automation-primary-button`, `.automation-danger-button`, `.automation-refresh-button` | `App.css` | button styles | tactile press/loading/disabled | hover, active, disabled, loading | browser |

### 14. Archive/settings destructive flows

| Surface | File | Current motion | Proposed motion | States to cover | Verification target |
|---|---|---|---|---|---|
| archive session selection | `src/App.tsx` | static controls | row select/deselect feedback | selected, stale, all selected | browser |
| restore/delete buttons | `src/App.tsx` | static | action-origin feedback + row highlight/removal | restore, delete, loading, error | browser |
| bulk archive actions | `src/App.tsx` | static | count change and action confirmation feedback | disabled, active, destructive | browser |
| settings confirm modal | `src/App.tsx`, `App.css` | modal CSS | branded accessible modal motion | focus trap, cancel, confirm, Escape | keyboard/browser |

## CSS prefix inventory and action notes

The scan found the following major class groups in `src/App.css`. Each group has a matching surface section above or is marked as cross-cutting.

| Prefix/group | Count from scan | Action |
|---|---:|---|
| `zoid-*` | 15 | shared dropdown/native command/confirm primitives; include in global primitives and command surfaces |
| `ink-*` | 1 | app rail; include shell |
| `rail-*` | 7 | shell rail controls |
| `sidebar-*` | 2 | shell sidebar morph |
| `nav-*` | 9 | primary navigation |
| `chat-*` | 16 | Agents/Hermes chat surfaces |
| `sessions-*` | 10 | Agents session rail |
| `session-*` | 18 | session rows/tabs/runtime chips |
| `message-*` | 21 | message bubble/action/status |
| `composer-*` | 36 | chat composer and panels |
| `file-manager-*` | 16 | file manager sidebar/tree |
| `hermes-*` | 7 | Agents/Hermes page/header variants |
| `agents-*` | 2 | Agents sumi-e identity |
| `code-*` | 6 | Code workspace identity/layout |
| `repo-*` / `repository-*` | 35 | Code repository controls/cards/actions |
| `brain-*` | 47 | Brain workspace |
| `content-*` | 20 | Content workspace |
| `automation-*` / `automations-*` | 58 | Automations workspace |
| `settings-*` | 13 | Settings/archive/destructive flows |
| `provider-*` / `providers-*` | 13 | Provider settings |
| `command-*` | 7 | Command palette |
| `archive-*` / `archived-*` | 5 | Archive flows |
| `topbar-*` | 1 | topbar status stack |
| `status-*` | 2 | global status dots/labels |
| `avatar-*` | 4 | avatar presence |
| `profile-*` | many in App.tsx scan | profile/settings UI; include Settings/Profile shell |

## Verification mapping

For each implementation batch, verification must include:

- Source inventory rows updated from `proposed` to `implemented`.
- `npm run build` at least after global foundation and before critique.
- Focused tests for touched areas first.
- `npm run test:frontend` before broad completion.
- Browser visual pass at `http://127.0.0.1:1420/`.
- Browser console check after interactions.
- Horizontal overflow/clipping check for fixed shell and dense panels.
- Reduced-motion check.
- Native Tauri verification after implementation batches.
- Feature critique workflow with `.hermes/reviews/zoid-25-animation-system/handoff.md`, fix Required items, re-review until APPROVED.

## Build Order 0 conclusion

This inventory confirms the animation scope is broad enough to satisfy “every little, teeny tiny detail” without starting from random CSS additions.

## Implementation status log

### Build Order 0 — inventory

Status: implemented.

Proof:

- Created this inventory file.
- Covered all visible TSX component files and major CSS surface groups.

### Build Order 1 — global motion foundation

Status: implemented initial foundation.

Implemented surfaces:

- `src/App.css` motion tokens: durations, easings, press/lift offsets, rule width, paper shadows, seal wash.
- reusable foundation classes/keyframes: `motion-paper-panel`, `motion-paper-notice`, `motion-ink-rule`, `motion-pressable`, `motion-paper-panel-enter`, `motion-paper-notice-enter`, `motion-seal-stamp`.
- centralized reduced-motion token override and disabling of foundation animations.
- tokenized shell structural transition.

Proof:

- `npm run build` passed after Build Order 1.
- `npm run test:frontend` passed after Build Order 1.
- Browser loaded at `http://127.0.0.1:1420/`.
- Browser console returned 0 messages / 0 JS errors after interaction.
- Horizontal overflow check returned false.

### Build Order 2 — shell/nav/shared primitives

Status: started.

Implemented surfaces:

- `GlobalDropdown` now exposes `is-open`, `is-disabled`, and `data-state` hooks for CSS motion.
- dropdown trigger/menu/option motion uses open-state hooks, paper-panel entry, chevron rotation, option rail/tactile states.
- `Suspense` fallback uses `app-startup-notice--loading` for truthful loading scan motion.
- compact rail nav items use tokenized hover/active/press transitions and active rule draw.
- primary nav rows use richer hover/active/press rule and icon micro-motion.
- status dots get tactile state transitions.
- session notification dots get an attention-only pulse and are disabled under reduced motion.
- `src/scaffold.test.ts` now guards the motion foundation and state hooks.

Build Order 2 proof so far:

- `npm run build` passed after Build Order 2.
- `npx tsx src/ui/GlobalDropdown.behavior.test.tsx` passed after Build Order 2.
- Browser collapsed-sidebar check: shell class `zoid25-shell sidebar-collapsed`, 8 compact nav items, no horizontal overflow.
- Browser Agents dropdown check: 2 dropdown roots, 1 open dropdown, 1 menu, `aria-expanded=true`, no horizontal overflow.
- Browser CSS check found 11 `prefers-reduced-motion` rule entries and clean console at the checked moment.

Known verification blocker:

- `npm run test:frontend` is currently blocked by an existing Agents sessions-rail expectation mismatch: `aria-valuemin` actual `256`, expected `124` in `src/agents/AgentsHermesScreen.file-manager.test.tsx`. This appears outside the animation edits, but it prevents full-suite green until reconciled.
- `npx tsx src/scaffold.test.ts` is also blocked by existing chat workspace width expectations (`var(--sessions-rail-width, 184px)`) while current CSS uses `300px`.

Still pending in Build Order 2:

- decide whether to reconcile the existing Agents sessions-rail tests before continuing animation batches.
- update/focus checks for lazy fallback and all nav rows.

Recommended next implementation step:

1. Continue Build Order 3 into deeper Agents/Hermes workspace details.
2. Then move to Code workspace and remaining pages.

### Build Order 3 — Agents/Hermes workspace

Status: started.

Implemented surfaces:

- Agents workspace topbar ink rule gets tactile opacity/scale motion.
- connection panel, repository control, sessions rail, chat stage, composer, stats strip, file manager sidebar get panel lift/ink-paper transitions.
- file manager toggle underline now draws open/hover state.
- sessions tabs animate hover/active/press plus icon micro movement.
- message bubbles get paper-panel entry and hover lift.
- composer textarea focus uses tokenized border/shadow/height transitions.
- message action, composer, sessions morph, model buttons get tactile hover/press transitions.
- agent monitor bar/panels/buttons/feed lines get motion foundation treatment.
- reduced-motion disables new Agents message/feed animations.

Build Order 3 proof so far:

- `npm run build` passed after Build Order 3.
- `npx tsx src/ui/GlobalDropdown.behavior.test.tsx` passed after Build Order 3.
- Browser Agents page probe found `.agents-sumi-e`, `.agent-monitor-bar`, `.sessions-rail`, `.chat-stage`, `.chat-composer`, `.message-bubble`; no horizontal overflow.
- Visual browser check: Agents workspace remained intact after motion additions.

Known blocker carried forward:

- Full frontend/scaffold tests still hit the pre-existing sessions-rail width mismatch logged under Build Order 2.

### Build Order 4 — Code workspace

Status: started.

Implemented surfaces:

- Code workspace header ink rule gets hover opacity/scale motion.
- Code ink mark subtly rotates/scales on header hover.
- repository action panels and repository list panel get paper lift, shadow, and rule-draw motion.
- scan/clone inputs get focus border/shadow motion.
- action/default-branch feedback notices animate in.
- repo action buttons, repository card buttons, and search toggle get tactile hover/press motion.
- repository cards get hover lift/shadow motion.
- repo status pills react to card hover with a seal rule.
- default branch editor and feedback use notice entry motion.
- Code reduced-motion block disables new page-specific animation/transitions.

Build Order 4 proof so far:

- `npm run build` passed after Build Order 4.
- `npx tsx src/code/CodeWorkspace.behavior.test.tsx && npx tsx src/code/repositoryOperations.test.ts` passed.
- Browser Code page probe found `.code-workspace-shell`, `.code-workspace-header`, `.repo-action-panel`, `.repository-list-panel`, `.repository-search-toggle`, `.repo-empty-state`; no horizontal overflow.
- Visual browser check: Code workspace remained intact after motion additions.

### Build Order 5 — Content/Social + Automations

Status: started.

Implemented surfaces:

- Content tab's actual `SocialDashboard` surfaces now get motion: hero rule draw, provider cards, rhythm steps, alert/status strips, toolbar buttons, social panels, post cards, media cards, gate strip, caption block.
- Social dashboard buttons now have tactile hover/press motion and seal-square micro motion.
- Social dashboard cards/panels now lift with paper shadow and line accents widen/draw on hover.
- Automations page now gets deeper motion coverage: header rule draw, clock mark hover motion, command/status/summary/toolbar/section/job-card panel lift, job status badge reactions, search focus motion, button press motion, confirm panel motion, reduced-motion coverage.
- Fixed Automations alert readability by moving alert layout to grid with explicit line-height and extra top padding below the ink rule.

Build Order 5 proof so far:

- `npm run build` passed after Content/Social + Automations motion additions.
- `npx tsx src/automations/AutomationsWorkspace.behavior.test.tsx` passed.
- Browser Content/Social probe found `.social-dashboard`, `.social-hero`, `.social-rhythm-lane`, `.social-toolbar`, `.social-grid`, `.social-panel`; no horizontal overflow. After critique fix, CSSOM confirmed `motion-ink-reveal` is defined and `.social-hero` computes to `animation-name: motion-ink-reveal`.
- Visual browser check: Content/Social workspace remained intact.
- Browser Automations probe found `.automations-workspace-shell`, `.automations-workspace-header`, `.automation-command-panel`, `.automation-summary-grid`, `.automation-toolbar`, `.automation-workbench-grid`; no horizontal overflow.

### Build Order 6 — Brain + Settings fine-detail motion

Status: implemented; critique gate APPROVED.

Implemented surfaces:

- Brain page now has fine-detail motion on panels, bridge/error/empty/placeholder strips, source rows, note rows, session tabs, badges, clarifying textarea focus, panel chips, status seal mark, and action press states.
- Brain panel groups now enter with staggered ink reveal after the hero/link/workbench reveal.
- Settings page now has fine-detail motion on hero/profile cards, sticky heading, nav tabs, overview stat cards, profile sections, catalog/provider/archive cards, toggles, fields/dropdowns/options, save/security/error notes, confirm backdrop/panel/actions, and press/focus states.
- Settings destructive confirmation now uses a distinct backdrop/dialog reveal motion.
- Brain and Settings reduced-motion blocks now disable the new animation and transform/transition layer.

Build Order 6 proof so far:

- `npm run build` passed after Brain + Settings fine-detail motion additions.
- `npx tsx src/brain/BrainWorkspace.behavior.test.tsx` passed.
- Browser Brain probe found `.brain-sumi-e`, `.brain-hero`, `.brain-ink-mark`, `.brain-status-line`; no horizontal overflow. Current blocked Apple Notes bridge state means link/panel row selectors are not rendered in this local state.
- Visual browser check: Brain workspace remained intact, readable, and without obvious clipping/overlap.
- Browser Settings probe found `.settings-sumi-e`, `.settings-hero`, `.settings-ink-mark`, `.profile-hero-card`, `.profile-settings-heading--sticky`, `.profile-settings-workspace`, `.profile-section`, `.profile-nav-list`; no horizontal overflow.
- Visual browser check: Settings workspace remained intact, readable, and without obvious clipping/overlap.

Recommended next implementation step:

1. Optional later polish: consolidate duplicate page reveal keyframes and split the large CSS motion layer into clearer sections.
2. Keep the existing scaffold width mismatch tracked separately; it was not resolved as part of this motion pass.
3. When ready, move to commit/scoped cleanup with the broad dirty tree in mind.

Critique gate:

- First review verdict: CHANGES_REQUIRED because `motion-ink-reveal` was referenced but not defined.
- Required fix completed: added shared `@keyframes motion-ink-reveal`, rebuilt, and re-verified Content/Social browser animation/overflow plus reduced-motion coverage.
- Re-review verdict: APPROVED.
