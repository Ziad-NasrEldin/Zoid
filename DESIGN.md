---
version: alpha
name: Zoid25 Kujoyama Editorial OS
description: "A Villa Kujoyama-inspired product design system for a local-first macOS AI operating system: white editorial fields, black rules, one committed cobalt rail, mono-forward hierarchy, tactile squared controls, and truthful operational states."
colors:
  primary: "#3558A2"
  secondary: "#000000"
  tertiary: "#FDE863"
  neutral: "#FFFFFF"
  kujo-blue: "#3558A2"
  kujo-blue-deep: "#294984"
  kujo-blue-soft: "#E7EDFA"
  kujo-blue-wash: "#B8C8EA"
  kujo-ink: "#000000"
  kujo-paper: "#FFFFFF"
  kujo-yellow: "#FDE863"
  kujo-red: "#DD4949"
  kujo-green: "#81CA7A"
  kujo-amber: "#EFAC39"
  kujo-teal: "#21AB88"
  kujo-muted: "#555555"
  danger-bg: "#FFECEC"
  cool-paper: "#FBFCFF"
typography:
  display:
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, monospace"
    fontSize: "68px"
    fontWeight: 500
    lineHeight: 0.94
    letterSpacing: "-0.075em"
  brand:
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, monospace"
    fontSize: "64px"
    fontWeight: 800
    lineHeight: 0.95
    letterSpacing: "-0.075em"
  title:
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, monospace"
    fontSize: "24px"
    fontWeight: 900
    lineHeight: 1.05
    letterSpacing: "-0.045em"
  body:
    fontFamily: "Marianne, system-ui, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
    fontSize: "15px"
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "0em"
  body-strong:
    fontFamily: "Marianne, system-ui, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif"
    fontSize: "13px"
    fontWeight: 800
    lineHeight: 1.45
    letterSpacing: "0em"
  label:
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, monospace"
    fontSize: "11px"
    fontWeight: 900
    lineHeight: 1.15
    letterSpacing: "0em"
  label-small:
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, monospace"
    fontSize: "10px"
    fontWeight: 900
    lineHeight: 1.2
    letterSpacing: "0em"
rounded:
  none: "0px"
  compact: "5px"
  pill: "999px"
spacing:
  xxs: "3px"
  xs: "6px"
  sm: "8px"
  md: "12px"
  lg: "16px"
  xl: "24px"
  xxl: "34px"
  rail-min: "52px"
  rail-max: "72px"
  sidebar-min: "220px"
  sidebar-max: "336px"
components:
  button-primary:
    backgroundColor: "{colors.kujo-blue}"
    textColor: "{colors.kujo-paper}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    padding: "10px 12px"
  button-primary-hover:
    backgroundColor: "{colors.kujo-blue-deep}"
    textColor: "{colors.kujo-paper}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    padding: "10px 12px"
  button-secondary:
    backgroundColor: "{colors.kujo-paper}"
    textColor: "{colors.kujo-ink}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    padding: "9px 12px"
  button-danger:
    backgroundColor: "{colors.kujo-red}"
    textColor: "{colors.kujo-paper}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    padding: "9px 12px"
  input-editorial:
    backgroundColor: "{colors.kujo-paper}"
    textColor: "{colors.kujo-ink}"
    typography: "{typography.body}"
    rounded: "{rounded.none}"
    padding: "10px 12px"
  dropdown-editorial:
    backgroundColor: "{colors.kujo-paper}"
    textColor: "{colors.kujo-ink}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    padding: "7px 32px 7px 10px"
    height: "34px"
  blue-rail:
    backgroundColor: "{colors.kujo-blue}"
    textColor: "{colors.kujo-paper}"
    typography: "{typography.label}"
    width: "72px"
  nav-row:
    backgroundColor: "{colors.kujo-paper}"
    textColor: "{colors.kujo-ink}"
    typography: "{typography.title}"
    rounded: "{rounded.none}"
    padding: "18px 28px 17px 36px"
  nav-row-active:
    backgroundColor: "{colors.kujo-blue-soft}"
    textColor: "{colors.kujo-ink}"
    typography: "{typography.title}"
    rounded: "{rounded.none}"
    padding: "18px 28px 17px 36px"
  card-ruled:
    backgroundColor: "{colors.kujo-paper}"
    textColor: "{colors.kujo-ink}"
    typography: "{typography.body-strong}"
    rounded: "{rounded.none}"
    padding: "16px"
  status-success:
    backgroundColor: "{colors.kujo-green}"
    textColor: "{colors.kujo-ink}"
    typography: "{typography.label-small}"
    rounded: "{rounded.pill}"
    padding: "5px 8px"
  status-warning:
    backgroundColor: "{colors.kujo-amber}"
    textColor: "{colors.kujo-ink}"
    typography: "{typography.label-small}"
    rounded: "{rounded.pill}"
    padding: "5px 8px"
  status-danger:
    backgroundColor: "{colors.kujo-red}"
    textColor: "{colors.kujo-paper}"
    typography: "{typography.label-small}"
    rounded: "{rounded.pill}"
    padding: "5px 8px"
---

# Design System: Zoid25 Kujoyama Editorial OS

## 1. Overview

**Creative North Star: "The Editorial Command Room"**

Zoid 25 is a local-first macOS AI operating system, not a SaaS dashboard and not a marketing surface. Its visual language translates Villa Kujoyama's architectural restraint into product UI: severe black rules, white editorial fields, one committed cobalt-blue structural surface, mono-forward hierarchy, and sparse yellow emphasis. The system should feel cultural, focused, precise, and operational.

The interface is allowed to be dense, but it must stay calm. Rows, rails, ruled panels, command palettes, and typed status badges carry the work. Cards are used only when an operational object needs containment; they remain squared, bordered, and visibly tied to the same rule system. The user must always be able to distinguish native data, preview data, blocked bridge state, and verified state.

**Key Characteristics:**
- Stark black-and-white editorial shell with cobalt-blue architecture.
- Mono-forward type for navigation, labels, command surfaces, and operational titles.
- Squared tactile controls: 1px black strokes, zero radius, offset shadows only where a surface needs physicality.
- Status vocabulary uses both color and text; dots are never the only signal.
- White space is functional: blank space means not-yet-built or intentionally empty, not missing design.

## 2. Colors

The palette is restrained but committed: black and white do most of the work; cobalt blue carries architecture and selection; yellow is a rare operational highlight; semantic status colors appear only with labels.

### Primary
- **Kujoyama Cobalt** (`kujo-blue`): the structural blue for the left rail, selected navigation, current states, primary actions, progress meters, and high-emphasis panels.
- **Deep Cobalt** (`kujo-blue-deep`): validation, depth, and hover reinforcement for blue surfaces.
- **Soft Blue Field** (`kujo-blue-soft`): selected rows, compact rails, hovered menus, filter tabs, status panels, and cool workspaces.
- **Blue Wash** (`kujo-blue-wash`): separators inside dropdowns, image-wash panels, and light architectural atmosphere.

### Secondary
- **Ink** (`kujo-ink`): primary text, ruled borders, dividers, icon strokes, and the default hard edge.
- **Muted Ink** (`kujo-muted`): secondary metadata only; never the main structure.

### Tertiary
- **Signal Yellow** (`kujo-yellow`): focus outlines, active profile/nav badges, command-mode chips, and pinned or warning context. It is rare by design.

### Neutral
- **Paper** (`kujo-paper`): the default app field, control background, cards, modals, and editorial sidebars.
- **Cool Paper** (`cool-paper`): a barely-tinted field for settings/forms where pure white needs relief without becoming gray SaaS chrome.
- **Danger Field** (`danger-bg`): error notices and destructive warnings only.

### Semantic State
- **Green** (`kujo-green`): ready, online, validated, clean, copied, and successful job states.
- **Red** (`kujo-red`): blocked, offline, failed, destructive, and removal states.
- **Amber** (`kujo-amber`): paused, command mode, warning, dirty, or needs-attention states.
- **Teal** (`kujo-teal`): optional info state when blue is already carrying selection.

**The One Blue Rule.** Blue is architecture and state, not decoration. If a surface already has a blue rail or selected blue state, do not sprinkle extra blue accents across unrelated components.

**The Status Text Rule.** Color dots and badges must always be paired with text labels such as Ready, Blocked, Paused, Verified, Preview, or Native.

## 3. Typography

**Display Font:** `ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, Liberation Mono, monospace`.
**Body Font:** `Marianne, system-ui, -apple-system, BlinkMacSystemFont, Segoe UI, sans-serif`.
**Settings Reading Font:** `Inter, SF Pro Text, Helvetica Neue, Marianne/system fallback` for profile/settings forms where long copy needs less mono density.

**Character:** Zoid's type is editorial and operational. Mono gives the OS a printed-index rhythm; sans body copy keeps dense instructions, settings, and message content readable.

### Hierarchy
- **Display** (`display`): huge workspace headings and module titles; tight line-height, negative tracking, no decorative hero treatment.
- **Brand** (`brand`): the ZOID / 25 sidebar mark; severe, tightly tracked, and always paired with the blue rail.
- **Title** (`title`): row titles, job names, repository cards, session titles, and compact operational headings.
- **Body** (`body`): chat messages, explanations, profile copy, and prose instructions.
- **Body Strong** (`body-strong`): dense product copy that needs weight without becoming a headline.
- **Label / Label Small** (`label`, `label-small`): uppercase controls, chips, metadata, badges, command labels, and status vocabulary.

**The Mono-For-Structure Rule.** Use mono for navigation, commands, status labels, IDs, counters, tabs, and row titles. Do not use expressive display type inside form labels, dense settings descriptions, or long body copy.

**The Fixed Product Scale Rule.** Product surfaces use fixed px/rem scales and clamp only for large screen headings. Avoid fluid typography in dense rails, tables, cards, buttons, or input controls.

## 4. Elevation

Zoid is flat by default. Depth comes from black rules, spatial separation, blue/white tonal contrast, and occasional hard offset shadows. Shadows are not ambient blur; they are tactile editorial print offsets used sparingly for popovers, cards that need containment, composer actions, and confirmation modals.

### Shadow Vocabulary
- **Soft Card Offset** (`4px 4px 0 rgba(0,0,0,0.08-0.14)`): repository cards, profile tiles, message actions, and newly changed rows.
- **Panel Offset** (`6px 6px 0 rgba(0,0,0,0.08-0.10)`): workspace headers, profile sections, and major settings panels.
- **Popover Offset** (`9px 9px 0 rgba(0,0,0,0.16)`): composer popovers and deep panels.
- **Modal Offset** (`10px 10px 0 rgba(0,0,0,0.18-0.22)`): command palettes, confirmations, and native command panels.
- **Inset State Rail** (`inset 4px-6px 0 0 {colors.kujo-blue}`): selected navigation/session rows and action feedback rails.

**The Rule-First Depth Rule.** Add a border before adding a shadow. Add a hard offset shadow only when the surface must float above the canvas or mark recent state change.

## 5. Components

### Shell and Navigation
- **Blue rail:** 52-72px wide, full-height cobalt, white mono labels/icons, no rounded corners. Collapsed module icons sit in square 42-50px hit areas.
- **Editorial sidebar:** white field, 1px black right border, large mono brand block, row navigation with black dividers.
- **Navigation rows:** white default, soft-blue hover/active, mono title, uppercase metadata, optional selected inset rail. Avoid card-like nav pills.
- **Responsive behavior:** desktop-first three-column shell; below tablet width, rail/sidebar collapse into horizontal/stacked navigation rather than shrinking typography into illegibility.

### Buttons
- **Shape:** squared, 1px black border, zero radius. The compact dropdown exception may use 5px only where density demands it.
- **Primary:** cobalt fill, white mono uppercase label, 9-12px vertical/horizontal padding, optional small icon.
- **Secondary:** paper fill, ink text, same border and typography.
- **Danger:** red fill, white text, used only for destructive actions and confirmations.
- **Hover / Focus:** hover may shift by 1px with a harder offset shadow or switch to soft-blue; focus-visible uses a 3px yellow outline so keyboard state is unmistakable.
- **Disabled:** opacity around 0.52-0.62, no shadow, not-allowed cursor.

### Inputs and Dropdowns
- **Text fields:** 1px black border, square corners, paper or cool-paper background, 10-12px padding, clear focus state with blue border/soft-blue inset.
- **Dropdowns:** mono label treatment, 34px default height, right-side soft-blue chevron lane, paper menu, black border, hard 4-5px offset shadow, soft-blue selected/hover row.
- **Compact dropdowns:** 26px height, 5px radius only when embedded inside dense chips or metadata editors.

### Cards, Panels, and Rows
- **Rows over cards:** lists, sessions, nav, jobs, metadata, and command options should prefer rows with dividers over rounded cards.
- **Cards:** when needed, use 1px black border, paper/soft-blue/cool-paper fill, 12-18px padding, zero radius, and optional hard offset shadow.
- **Workspace headers:** bordered paper panels with display headings, kana/mono lead line, muted prose, and 6px offset shadow.
- **Feedback panels:** light blue/success/danger fields with inset colored rail and explicit status copy.

### Badges, Chips, and Status
- **Status badges:** bordered, uppercase mono, small padding, semantic fill. Red badges use white text; green/amber/soft-blue generally keep ink text.
- **Status dots:** 7-12px circles with optional white border; pair every dot with copy when it represents state.
- **Command chips:** amber fill, black border, mono uppercase, and small offset shadow.
- **Presence/avatar:** circular avatars are allowed because they represent people/agents; preserve existing avatar semantics and do not replace them with decorative art.

### Modals and Command Surfaces
- **Command palette / confirmations:** centered paper panels, 1px black border, 10px offset shadow, mono uppercase headers, no native browser confirms.
- **Backdrops:** dark translucent black with restrained blur; backdrop is functional focus isolation, not glassmorphism decoration.
- **Action rows:** grid layout, bordered paper rows, soft-blue hover, explicit icon/copy pairing.

### Motion
- **Default timing:** 140-240ms for hover, icon, button, and micro-feedback transitions.
- **Structural transitions:** sidebar/session morphs can use 320-540ms cubic-bezier state transitions, but reduced-motion must collapse them.
- **No decorative choreography:** motion communicates open/closed, selected, copied, streaming, newly added, or resizing states only.

## 6. Do's and Don'ts

### Do:
- **Do** use black 1px rules and squared edges as the default visual grammar.
- **Do** reserve cobalt blue for architecture, selection, primary action, and verified/progress surfaces.
- **Do** use mono labels for operational control surfaces and sans body for readable prose.
- **Do** mark native, preview, blocked, ready, verified, paused, and failed states with explicit text.
- **Do** keep destructive confirmations branded, centered, async-aware, and visibly tied to the Zoid design system.
- **Do** use soft-blue hover/selected fields for row-based interaction instead of rounded pill nav.
- **Do** keep empty/loading/error states truthful; blank canvas is acceptable when it means not-yet-built or no current data.

### Don't:
- **Don't** make Zoid feel like a generic SaaS dashboard card grid.
- **Don't** use AI-chat novelty surfaces, decorative gradients, fake metrics, or simulated records presented as truth.
- **Don't** use glassmorphism as decoration; a blurred backdrop is only for modal focus isolation.
- **Don't** over-round controls into bubble UI. Zero radius is the default.
- **Don't** use blue as confetti or sprinkle accent color across inactive elements.
- **Don't** rely on color dots alone for state; state requires text.
- **Don't** copy Villa Kujoyama logos, photography, or proprietary assets; translate the editorial and architectural language only.
- **Don't** hide review gates, backend availability, data source, bridge requirements, or verification state.
- **Don't** add heavy border-left color stripes as a generic AI-card trope; if a rail is necessary, it must communicate a specific selected/feedback state.
