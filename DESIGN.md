---
version: alpha
name: Zoid 25 Sumi-e Command System
description: "The accepted Zoid 25 product UI system extracted from the Agents page: white paper, black sumi ink, pale rules, red seal accents, serif English/Japanese typography, boxed operational rails, and no warm gold artifacts."
colors:
  primary: "#0D0A0A"
  secondary: "#545554"
  tertiary: "#C23A2E"
  neutral: "#FFFFFF"
  sumi-ink: "#0D0A0A"
  sumi-paper: "#FFFFFF"
  sumi-soft-paper: "#FAFAFA"
  sumi-mist: "#F5F5F5"
  sumi-rule: "#E0E0E0"
  sumi-pale-rule: "#EDEDED"
  sumi-muted: "#545554"
  sumi-wash: "#F7F5F4"
  sumi-seal: "#C23A2E"
  sumi-seal-deep: "#8F211A"
  sumi-seal-wash: "#F5E5E3"
  sumi-okay: "#2F3A2F"
  danger-field: "#FFECEC"
typography:
  display:
    fontFamily: "trajan-pro-3, Times New Roman, Baskerville, Georgia, serif"
    fontSize: "46px"
    fontWeight: 400
    lineHeight: 0.9
    letterSpacing: "-0.04em"
  display-large:
    fontFamily: "trajan-pro-3, Times New Roman, Baskerville, Georgia, serif"
    fontSize: "94px"
    fontWeight: 400
    lineHeight: 0.92
    letterSpacing: "-0.035em"
  body:
    fontFamily: "source-han-serif-japanese, Hiragino Mincho ProN, Yu Mincho, Times New Roman, Georgia, serif"
    fontSize: "14px"
    fontWeight: 400
    lineHeight: 1.58
    letterSpacing: "0.025em"
  body-compact:
    fontFamily: "source-han-serif-japanese, Hiragino Mincho ProN, Yu Mincho, Times New Roman, Georgia, serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.45
    letterSpacing: "0.035em"
  label:
    fontFamily: "trajan-pro-3, Times New Roman, Baskerville, Georgia, serif"
    fontSize: "10px"
    fontWeight: 400
    lineHeight: 1.2
    letterSpacing: "0.14em"
  label-small:
    fontFamily: "trajan-pro-3, Times New Roman, Baskerville, Georgia, serif"
    fontSize: "9px"
    fontWeight: 400
    lineHeight: 1.15
    letterSpacing: "0.16em"
rounded:
  none: "0px"
  portrait: "999px"
spacing:
  hairline: "1px"
  rule: "2px"
  xs: "4px"
  sm: "8px"
  md: "10px"
  lg: "14px"
  xl: "18px"
  xxl: "24px"
  page-x: "16px"
  page-y: "14px"
  sessions-rail-compact: "68px"
  sessions-rail-default: "184px"
  sessions-rail-min: "124px"
  sessions-rail-max: "420px"
components:
  page-shell:
    backgroundColor: "{colors.sumi-paper}"
    textColor: "{colors.sumi-ink}"
    typography: "{typography.body}"
    rounded: "{rounded.none}"
    padding: "14px 16px 20px"
  topbar-panel:
    backgroundColor: "{colors.sumi-paper}"
    textColor: "{colors.sumi-ink}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    padding: "14px 18px 12px"
  ink-primary-button:
    backgroundColor: "{colors.sumi-ink}"
    textColor: "{colors.sumi-paper}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    padding: "0 14px"
    height: "44px"
  seal-primary-button-hover:
    backgroundColor: "{colors.sumi-seal}"
    textColor: "{colors.sumi-paper}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    padding: "0 14px"
    height: "44px"
  quiet-control:
    backgroundColor: "{colors.sumi-paper}"
    textColor: "{colors.sumi-ink}"
    typography: "{typography.label-small}"
    rounded: "{rounded.none}"
    padding: "5px 9px"
    height: "28px"
  dropdown-compact:
    backgroundColor: "{colors.sumi-paper}"
    textColor: "{colors.sumi-ink}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    padding: "4px 29px 4px 9px"
    height: "26px"
  session-row:
    backgroundColor: "{colors.sumi-paper}"
    textColor: "{colors.sumi-ink}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    padding: "12px 10px 12px 14px"
  session-row-active:
    backgroundColor: "{colors.sumi-soft-paper}"
    textColor: "{colors.sumi-ink}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    padding: "12px 10px 12px 14px"
  agent-monitor-panel:
    backgroundColor: "{colors.sumi-paper}"
    textColor: "{colors.sumi-ink}"
    typography: "{typography.body-compact}"
    rounded: "{rounded.none}"
    padding: "10px"
  message-assistant:
    backgroundColor: "{colors.sumi-paper}"
    textColor: "{colors.sumi-ink}"
    typography: "{typography.body}"
    rounded: "{rounded.none}"
    padding: "14px 16px"
  message-user:
    backgroundColor: "{colors.sumi-ink}"
    textColor: "{colors.sumi-paper}"
    typography: "{typography.body}"
    rounded: "{rounded.none}"
    padding: "14px 16px"
  status-ready:
    backgroundColor: "{colors.sumi-ink}"
    textColor: "{colors.sumi-paper}"
    typography: "{typography.label-small}"
    rounded: "{rounded.none}"
    padding: "4px 7px"
  status-attention:
    backgroundColor: "{colors.sumi-seal}"
    textColor: "{colors.sumi-paper}"
    typography: "{typography.label-small}"
    rounded: "{rounded.none}"
    padding: "4px 7px"
  field-neutral:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.sumi-ink}"
    typography: "{typography.body}"
    rounded: "{rounded.none}"
    padding: "10px"
  stage-mist-field:
    backgroundColor: "{colors.sumi-mist}"
    textColor: "{colors.sumi-ink}"
    typography: "{typography.body}"
    rounded: "{rounded.none}"
    padding: "24px"
  ruled-divider-field:
    backgroundColor: "{colors.sumi-rule}"
    textColor: "{colors.sumi-ink}"
    typography: "{typography.label-small}"
    rounded: "{rounded.none}"
    padding: "1px"
  pale-rule-field:
    backgroundColor: "{colors.sumi-pale-rule}"
    textColor: "{colors.sumi-ink}"
    typography: "{typography.label-small}"
    rounded: "{rounded.none}"
    padding: "1px"
  muted-meta:
    backgroundColor: "{colors.sumi-paper}"
    textColor: "{colors.sumi-muted}"
    typography: "{typography.label-small}"
    rounded: "{rounded.none}"
    padding: "4px"
  ink-wash-field:
    backgroundColor: "{colors.sumi-wash}"
    textColor: "{colors.sumi-ink}"
    typography: "{typography.body-compact}"
    rounded: "{rounded.none}"
    padding: "12px"
  seal-deep-destructive:
    backgroundColor: "{colors.sumi-seal-deep}"
    textColor: "{colors.sumi-paper}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    padding: "8px 10px"
  seal-wash-notice:
    backgroundColor: "{colors.sumi-seal-wash}"
    textColor: "{colors.sumi-ink}"
    typography: "{typography.body-compact}"
    rounded: "{rounded.none}"
    padding: "8px 10px"
  status-okay:
    backgroundColor: "{colors.sumi-okay}"
    textColor: "{colors.sumi-paper}"
    typography: "{typography.label-small}"
    rounded: "{rounded.none}"
    padding: "4px 7px"
  status-warn-legacy:
    backgroundColor: "{colors.sumi-alert}"
    textColor: "{colors.sumi-paper}"
    typography: "{typography.label-small}"
    rounded: "{rounded.none}"
    padding: "4px 7px"
  danger-field:
    backgroundColor: "{colors.danger-field}"
    textColor: "{colors.sumi-seal-deep}"
    typography: "{typography.body-compact}"
    rounded: "{rounded.none}"
    padding: "8px 10px"
---

# Design System: Zoid 25 Sumi-e Command System

## Overview

Zoid 25 is a local-first macOS AI operating system. The accepted visual source is the current Agents page, especially `src/agents/AgentsHermesScreen.tsx` rendered with `hermes-chat-shell hermes-genm agents-sumi-e` and the `.agents-sumi-e` rules in `src/App.css`.

The system is sumi-e product UI, not SaaS dashboard UI: white paper fields, black ink structure, pale gray rules, restrained red seal accents, roomy boxed operational controls, and Japanese/English serif typography. It should feel like an authored command room: tactile, quiet, sharp, and operationally truthful.

Warm gold/parchment artifacts from earlier edits are explicitly rejected. Do not carry forward golden focus rings, golden command chips, ochre strips, beige monitor cards, or warm warning-card surfaces. Command/attention states use the red seal family or written state text, not warm gold.

Core characteristics:
- White paper first; pale gray/mist for depth only.
- Black ink for architecture, primary actions, selected rails, and user message inversion.
- Red seal for attention, focus, active tuning, reply markers, and approval/signature moments.
- Serif Latin/Japanese type throughout the accepted Agents surface.
- Boxed rails, panels, stage, composer, footer, command panels, and dropdowns; zero-radius by default.
- Motion feels like paper entering, ink rules revealing, and controls pressing by 1px.

## Colors

Primary structure is `sumi-ink` on `sumi-paper`. The page background may use subtle radial ink washes and red-seal wash, but the base remains white.

- `sumi-ink` `#0D0A0A`: primary text, primary buttons, selected rails, user message bubbles, ink rules, focus structure.
- `sumi-paper` `#FFFFFF`: app fields, panels, controls, dropdown menus, message surfaces, modals.
- `sumi-soft-paper` `#FAFAFA`: hover rows, compact rail background, subtle panel relief.
- `sumi-mist` `#F5F5F5`: stage base and large quiet work areas.
- `sumi-rule` `#E0E0E0` and `sumi-pale-rule` `#EDEDED`: borders and separators. Prefer pale rules over black rules on the final Agents system.
- `sumi-muted` `#545554`: metadata and secondary operational copy.
- `sumi-wash` `#F7F5F4`: near-white ink wash; use sparingly and avoid beige/gold drift.
- `sumi-seal` `#C23A2E`: the only saturated accent. Use for attention, focus underline, active/tune hover, reply marker, assistant side rail, and signature marks.
- `sumi-seal-deep` `#8F211A`: stronger destructive/blocked accent and depth for seal marks.
- `sumi-seal-wash` `#F5E5E3`: light attention background; acceptable because it is red-derived, not warm gold.
- `sumi-okay` `#2F3A2F`: ready/online state when text also says Ready/Online.

No warm-gold rule:
- Do not use golden focus outlines, golden command chips, warm input glows, ochre cards, or parchment surfaces.
- Legacy color aliases must resolve to neutral ink, white paper, mist, or red seal in the final visual system.

## Typography

The accepted Agents page uses serif typography, not the earlier mono-forward Kujoyama system.

- Latin display/labels: `trajan-pro-3, Times New Roman, Baskerville, Georgia, serif`.
- Japanese/body: `source-han-serif-japanese, Hiragino Mincho ProN, Yu Mincho, Times New Roman, Georgia, serif`.
- Main page title: 32-46px on the compact Agents header, 400 weight, 0.9 line-height, about -0.04em tracking.
- Large global title variant: up to 94px for spacious pages, 400 weight, 0.92 line-height.
- Body/chat: 14px, 1.58 line-height, slight positive tracking around 0.025em.
- Operational labels: 9-10px, uppercase where useful, 0.13-0.16em tracking, 400 weight.
- Japanese micro labels: keep visible and readable, usually 12px with 0.16-0.18em tracking.

Typography should be calm and ceremonial. Do not return to heavy mono labels except inside technical file/path/code contexts where the product needs literal machine readability.

## Layout

Zoid is a fixed-shell desktop command room. The document/body are not the scroll owner; page regions own their own scroll.

Agents page composition:
- Page shell: full viewport, grid rows `auto minmax(0, 1fr) auto`, 14-24px top/bottom rhythm and 16-48px horizontal rhythm.
- Header/topbar: compact two-row stack; title row first, operational status/repository/files controls second. It remains above the workspace and must not clip dropdown menus.
- Workspace: two-column grid: sessions rail plus main stage; optional file manager adds a third column. Gap around 16px.
- Sessions rail: default 184px, min 124px, max 420px, compact 68px. Full-height panel with hidden horizontal overflow, own vertical scroll, quiet overflow cue.
- Main stage: dashboard grid or chat stage, min-width 0, min-height 0, no horizontal overflow.
- Agent monitor grid: 2-up by default, 2x2 quad, focus+stack, and single-column responsive under 980px.
- Footer stats strip: low, compact, operational, and still visible; model tuning is a quiet black control that turns red seal on hover/focus.

Spacing is dense but not cramped. Important controls stay above the fold. Dropdown menus must layer above red rule lines and topbar decoration.

## Elevation & Depth

Depth is mostly flat. Use pale rules, white translucency, ink washes, and restrained hard shadows.

Accepted recipes:
- Pale border: `1px solid #E0E0E0` or `#EDEDED` for most boxed surfaces.
- Ink top rule: 5-6px gradient line from black to transparent, opacity around 0.34-0.45.
- Hover lift: translateY(-1px) plus `4px 4px 0 rgba(13,10,10,0.14)` only when the object needs tactile feedback.
- No ambient glassmorphism. Backdrop blur is only acceptable for modal focus isolation.
- No chunky black shadows on the accepted Agents page except reusable motion tokens for hover/paper feedback.

## Shapes

Zero radius is the default. Controls, panels, dropdowns, rows, cards, modals, and command surfaces are square.

Allowed exceptions:
- Presence/avatar/portrait circles because they represent people/agents.
- Legacy hidden scroll affordances may use pill shape only where already accepted; the Agents overflow cue has been squared under `.agents-sumi-e`.

Do not introduce rounded SaaS cards, bubble pills, or soft app-store gradients.

## Components

Shell/header:
- Header is a boxed/ruled paper surface visually dissolved into the page through pale borders and a red/ink reference line.
- Keep the title compact: Japanese kicker, English title, red-left reference line.
- Topbar controls are equal-height 44px command boxes.

Buttons:
- Primary actions use black ink fill, white paper text, serif label, zero radius.
- Hover/active attention may move to red seal, not warm gold.
- Pressed controls move 1px using `--motion-press-offset`.
- Disabled controls use opacity around 0.62 and no shadow.

Dropdowns:
- Trigger: 26-44px depending on context, pale border, paper body, soft-paper right chevron lane.
- Menu: paper field, pale border, no heavy shadow on Agents; selected/hover row uses soft paper plus red/ink inset rule where useful.
- Menus must escape header clipping and sit above decorative rules.

Sessions rail:
- Rows over cards. Use dividers and pale rules, not individual rounded cards.
- Active session uses soft-paper background and a thin black inset rail.
- Session portraits can be local visual identity marks but must stay semantic to sessions/agents, not decorative confetti.
- New Session is visually separated as a command row.

Agent monitor dashboard:
- Control bar is compact and horizontally flexible.
- Panels are boxy, pale-ruled, dense, and scroll-safe.
- Focused panel uses red seal/ink outline; primary panel may use red seal kicker only.
- Buttons are quiet by default; selected/pressed states become black or red seal.
- Feed lines are clamped and animated as paper notices.

Chat stage/messages:
- Stage is a quiet mist field with subtle ink wash and diagonal architectural line.
- Assistant messages are white paper with a thin black-to-red side rule.
- User messages invert to black ink with white text.
- Message actions are hidden until hover/focus and use small square paper buttons.

Composer:
- Paper panel, pale rule, equal-height attach/send controls.
- Send is black ink; stop/destructive is red seal.
- Slash/command mode must not use warm gold. Use seal wash or black/red state instead.

Footer/status:
- Footer stats strip is compact, pale-ruled, and text-first.
- Every state must have written copy: online/offline/checking/error, running/queued/needs reply, native/preview/blocked/verified.

Modals/command panels:
- Centered paper panels, pale or ink border, red/ink section rules, no native confirms for branded destructive flows.
- Trap focus, support Escape, describe consequences, and restore focus.

Motion:
- `80ms` instant, `120ms` micro, `160ms` controls, `220ms` rows, `380ms` panels, `540ms` structural, `760ms` page reveal.
- Easing: editorial `cubic-bezier(0.16, 1, 0.3, 1)`, press `cubic-bezier(0.25, 1, 0.5, 1)`, rule `cubic-bezier(0.215, 0.61, 0.355, 1)`.
- Reduced-motion must remove structural/page animation and preserve state changes without choreography.

## Do's and Don'ts

### Do

- Do treat the Agents page as the current source of truth for Zoid 25 visual direction.
- Do keep the palette white/black/gray/red-seal.
- Do use serif Latin and Japanese typography for product chrome, labels, headers, and chat body.
- Do keep panels square, boxed, and rule-based.
- Do use text labels for every operational state.
- Do preserve file manager, sessions rail, dashboard, composer, footer, and command panels as functional surfaces, not decorative containers.
- Do verify dropdown layering, horizontal overflow, and scroll ownership after applying the system to another page.

### Don't

- Don't reintroduce warm-gold artifacts, golden chips, parchment cards, beige monitor panels, warm warning panels, or old golden styling.
- Don't fall back to the old cobalt/Kujoyama design language except as legacy token aliases mapped to sumi-e values.
- Don't use generic SaaS cards, rounded dashboard tiles, glass panels, fake metrics, or ornamental AI chat novelty.
- Don't hide backend/runtime truth; distinguish native, preview, blocked, ready, verified, running, queued, stopped, and failed states.
- Don't make decorative brush art carry state unless the same state is written in text.
