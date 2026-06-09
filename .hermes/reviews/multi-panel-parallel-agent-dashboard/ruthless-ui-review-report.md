# Ruthless UI/design-system review: Agents multi-panel dashboard

Verdict: REQUIRED_FIXES

Scope reviewed from disk:
- src/agents/AgentsHermesScreen.tsx
- src/agents/AgentMonitorPanel.tsx
- src/agents/ChatComposer.tsx where it affects the always-visible bottom composer
- src/App.css sections for Agents page, sessions rail, monitor bar, dashboard panels, panel composer, and chat composer
- src/agents/AgentsHermesScreen.file-manager.test.tsx and src/scaffold.test.ts dashboard/rail guard coverage
- User screenshot: Screenshot 2026-06-09 at 6.04.20 PM
- Feature handoff and functional ruthless report in .hermes/reviews/multi-panel-parallel-agent-dashboard/

Bottom line:
The feature may be functionally useful, but the visible dashboard is not at the accepted Zoid sumi-e/ink/paper/red-seal product standard. The screenshot reads like raw browser controls dropped into an otherwise crafted Agents shell: huge black-bordered buttons, cramped panels, clipped rail content, competing composers, weak hierarchy, and no scoped sumi-e treatment for the new dashboard surfaces.

REQUIRED_FIXES

1. New dashboard surfaces are not integrated into the accepted Agents sumi-e design system.
   - Evidence:
     - src/App.css:440-462 defines `.agent-monitor-bar`, `.agent-monitor-panel`, `.agent-monitor-actions`, `.agent-monitor-composer` globally with generic Kujo-era ink borders, box shadows, default inherited button typography, and hard rectangular browser-like controls.
     - Search found no `.agents-sumi-e .agent-monitor...` scoped overrides, while the accepted Agents system is scoped heavily under `.agents-sumi-e` at src/App.css:1060-1264.
     - Screenshot: the page header/topbar has the accepted ink/paper feel, but the dashboard panel is a red-outlined raw form slab with oversized buttons and none of the paper-rule/seal restraint used by the rest of Agents.
   - Why required: this is a design-system regression on the main feature surface. The dashboard looks like an unstyled prototype inside a polished product page.
   - Smallest safe fix:
     - Add scoped `.agents-sumi-e .agent-monitor-bar`, `.agents-sumi-e .agent-monitor-panel`, `.agents-sumi-e .agent-monitor-panel-header`, `.agents-sumi-e .agent-monitor-actions button`, `.agents-sumi-e .agent-monitor-status-strip`, `.agents-sumi-e .agent-monitor-feed`, and `.agents-sumi-e .agent-monitor-composer` rules.
     - Use `--agents-paper`, `--agents-soft-paper`, `--agents-pale-rule`, `--agents-ink-black`, `--agents-ink-soft`, and `--agents-seal`; remove heavy box shadows; use pale rules and a single seal accent only for primary/focused/running state.
     - Keep the global generic rules as baseline only; the Agents page must render through the scoped accepted system.

2. Panel header actions destroy hierarchy and make the actual session identity secondary.
   - Evidence:
     - src/agents/AgentMonitorPanel.tsx:43-55 renders five full-size buttons (`←`, `→`, `Primary`, `Expand`, `Untile`) beside the title on every panel.
     - src/App.css:441 applies the same high-contrast button treatment to all panel actions.
     - src/App.css:452-455 gives the header only a two-column grid and lets the action cluster wrap as loud primary UI.
     - Screenshot: the action buttons dominate the panel; the title is truncated to “Continue this ...”, while “Primary / Expand / Untile” visually read as the main content.
   - Why required: a monitoring dashboard must prioritize session identity, run state, last output, and prompt. Tile-management actions are secondary. Current hierarchy is inverted.
   - Smallest safe fix:
     - Keep title/repository/status as the header’s primary line.
     - Convert move left/right to compact 28-32px icon buttons with quiet borders.
     - Make `Primary` a state chip/toggle only when not primary, not a giant always-on command.
     - Move `Untile` into a quiet overflow or low-emphasis text/icon action; keep `Expand` visible but compact.
     - Cap header action button height and font size under `.agents-sumi-e .agent-monitor-actions button`.

3. Sessions rail rows are structurally broken by dashboard actions; content is clipped and the rail is hard to scan.
   - Evidence:
     - src/agents/AgentsHermesScreen.tsx:1428-1434 appends `.session-dashboard-actions` inside every session row when the rail is expanded.
     - src/App.css:487 defines `.session-tab-row` as `grid-template-columns: minmax(0, 1fr) 34px` for the session button plus archive column, but `.session-dashboard-actions` has no explicit grid placement.
     - src/App.css:463 gives `.session-dashboard-actions` `padding: 0 28px 6px 42px` and full button treatment; it becomes an awkward extra area in a narrow rail.
     - Screenshot: session titles/metadata are visibly truncated, the left edge/content reads clipped, and the `Tile`/`Continue` controls become the loudest repeated pattern in the rail.
   - Why required: the sessions rail is core navigation. It currently fails scanability and density, and repeated management controls overwhelm session selection.
   - Smallest safe fix:
     - Explicitly place dashboard actions: `.session-dashboard-actions { grid-column: 1 / -1; padding: 4px 10px 8px 52px; }` or move actions into a compact overflow/secondary row.
     - Do not show both `Tile` and `Continue` as large default buttons on every session. Use a small runtime chip plus one quiet action, or reveal actions on hover/focus.
     - Ensure `.session-tab-title` and `.session-tab-meta` keep readable width at the default 184px rail; if not, raise default rail width for dashboard mode or collapse actions.

4. The dashboard monitor bar clips/overcrowds controls instead of behaving like a command/status strip.
   - Evidence:
     - src/agents/AgentsHermesScreen.tsx:1473-1495 renders counts, three buttons, a “Layout” label, and a dropdown in one strip.
     - src/App.css:440 gives `.agent-monitor-bar` a simple wrapping flex row but no design-system scoped density rules.
     - Screenshot: `Clear dashboard` is cut off on the right edge, and the layout dropdown drops to a second line awkwardly while the first line remains crowded.
   - Why required: the monitor bar is supposed to communicate operational state. It currently reads as a clipped toolbar of raw controls.
   - Smallest safe fix:
     - Structure it as status cluster + command cluster + layout control: e.g. `.agent-monitor-bar { display: grid; grid-template-columns: minmax(0,1fr) auto; }` with a second row for layout below narrow widths.
     - Use compact chips for counts and compact command buttons with `white-space: nowrap`, `min-width: 0`, and a responsive breakpoint before clipping.
     - Add a DOM guard that dashboard root `scrollWidth <= clientWidth` for the monitor bar at the screenshot width.

5. The dashboard shows two competing composers, making the primary action ambiguous.
   - Evidence:
     - src/agents/AgentMonitorPanel.tsx:65-71 renders a per-panel prompt composer.
     - src/agents/AgentsHermesScreen.tsx:1579 always renders the global `<ChatComposer>` underneath the dashboard, even when tiled dashboard panels are visible.
     - Screenshot: the panel composer and the large bottom `MESSAGE HERMES` composer are both visible. This splits attention and makes it unclear whether the user should prompt the selected panel or the global active session.
   - Why required: multi-panel mode is panel-scoped. A global composer below it undermines the dashboard mental model and consumes scarce vertical space.
   - Smallest safe fix:
     - When `dashboardVisibleSessions.length > 0` and not in expanded chat, collapse the global composer into a minimal “message active session” command bar or hide it behind an explicit affordance.
     - If the global composer stays, label it as “Message active session” and visually subordinate it; do not let it occupy the same weight as panel composers.
     - Keep expanded mode separately scoped as already flagged in the functional report.

6. Panel composer/control sizing is visually broken and inefficient.
   - Evidence:
     - src/agents/AgentMonitorPanel.tsx:65-71 puts textarea plus three stacked full-width buttons in every panel.
     - src/App.css:460-462 sets a two-column composer with a 36px min textarea and vertical button stack, but src/App.css:441 gives all those buttons the same heavy default surface.
     - Screenshot: `Continue where left off` is huge, the disabled `Send`/`Stop` buttons still carry heavy boxes, and the textarea is a cramped afterthought.
   - Why required: panel prompting is the main workflow. Current controls are too loud, too tall, and too repetitive for four-panel density.
   - Smallest safe fix:
     - Make the textarea the dominant element; set a stable min-height around 56-64px in panel mode.
     - Convert `Send` to a compact primary seal/ink action only when enabled; make disabled actions visibly quiet, not black-boxed.
     - Move `Continue where left off` to a secondary line/link or compact button; it should not be the largest control in every idle panel.
     - Keep touch targets >= 32px for dense desktop; do not use oversized default button font.

7. The focused/primary state is overdrawn and noisy.
   - Evidence:
     - src/App.css:450 uses `outline: 3px double var(--kujo-red); outline-offset: -5px` for focused panels.
     - src/App.css:451 only changes the kicker color for primary state.
     - Screenshot: the red double outline creates a harsh nested box around the whole panel, while primary/focus semantics are not otherwise clear.
   - Why required: Zoid’s red seal should be sparse and meaningful. The current outline makes the panel look like an error/debug selection, not a crafted active surface.
   - Smallest safe fix:
     - Replace the double outline with one subtle seal accent: e.g. a 3px left/bottom brush rule or small seal chip in the header.
     - Distinguish primary vs focused consistently: primary = label/chip; focused = subtle ink/seal edge or background wash.
     - Use `.agents-sumi-e .agent-monitor-panel--focused` and `.agents-sumi-e .agent-monitor-panel--primary` scoped rules.

8. Dashboard grid sizing is not using available space gracefully and risks clipped content under real widths.
   - Evidence:
     - src/App.css:443-448 gives the dashboard a fixed grid strategy with two columns and `grid-auto-rows: minmax(0, 1fr)`.
     - src/App.css:439 and src/App.css:1211 hide overflow at workspace/stage levels.
     - Screenshot at roughly 963px wide shows the right side of dashboard chrome clipped while the rail and large bottom composer consume space.
   - Why required: this is a fixed-shell desktop app. Hidden overflow can mask broken controls rather than solving layout.
   - Smallest safe fix:
     - For 1 tiled panel, constrain the panel to a readable max width or let it fill only the content column without clipped controls.
     - For 2/4 panels, define min panel widths and switch to 1 column before controls clip.
     - Add page-owned vertical scrolling where needed; do not rely on parent `overflow: hidden` to hide bad geometry.

9. Visual regression coverage does not protect the new dashboard UI.
   - Evidence:
     - src/agents/AgentsHermesScreen.file-manager.test.tsx:449-518 proves concurrent panel sends, queue visibility, and scoped stop, but not layout/visual hierarchy.
     - src/scaffold.test.ts:1007-1057 protects restored sessions rail/composer placement, but there are no guards for `.agents-sumi-e .agent-monitor-*`, panel action density, monitor bar overflow, or dashboard no-horizontal-overflow.
   - Why required: this feature already regressed visually into raw controls. Without guards, the same failure can return.
   - Smallest safe fix:
     - Add source guards requiring scoped `.agents-sumi-e .agent-monitor-panel`, `.agents-sumi-e .agent-monitor-bar`, and `.agents-sumi-e .agent-monitor-composer` rules.
     - Add a DOM/layout smoke test for dashboard mode at the screenshot width that checks monitor bar and first panel do not horizontally overflow their containers.
     - Add a guard that dashboard mode does not show two equally prominent composers unless explicitly labeled/scoped.

Suggested polish, not blocking after the required fixes:
- Consider deterministic local sigils/portraits inside dashboard panel headers so panels can be distinguished quickly without reading every title.
- Use status words with better operational semantics than raw `idle`: “ready”, “running”, “needs reply”, “queued”, “blocked”.
- Make recent output less wall-like: assistant/user labels can be slim side labels or low-emphasis metadata instead of inline bold text in every line.
- Add hover/focus affordances for tiled panel focus that do not move layout.
- Consider hiding `Auto-prioritize` until functional behavior exists; the functional report already flags it as no-op.

Final verdict: REQUIRED_FIXES
