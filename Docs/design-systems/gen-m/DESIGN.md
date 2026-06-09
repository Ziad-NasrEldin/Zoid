---
version: alpha
name: Gen-M Monochrome Ink Atelier
description: "Extracted from https://gen-m.jp: a monochrome calligraphic atelier system with extreme white space, ink-black gesture, cinematic grayscale media, serif letterspaced navigation, circular ghost links, and smooth scroll choreography."
colors:
  primary: "#0D0A0A"
  secondary: "#545554"
  tertiary: "#E0E0E0"
  neutral: "#FFFFFF"
  ink-black: "#0D0A0A"
  pure-black: "#000000"
  paper-white: "#FFFFFF"
  soft-paper: "#FAFAFA"
  mist-gray: "#F5F5F5"
  line-gray: "#EDEDED"
  pale-rule: "#E0E0E0"
  muted-ink: "#545554"
  ink-wash: "#F7F5F4"
  seal-red: "#C23A2E"
  seal-red-deep: "#8F211A"
  seal-red-soft: "#F5E5E3"
typography:
  display:
    fontFamily: "trajan-pro-3, serif"
    fontSize: "118px"
    fontWeight: 400
    lineHeight: 0.9
    letterSpacing: "-0.02em"
  product-display:
    fontFamily: "trajan-pro-3, Times New Roman, Baskerville, Georgia, serif"
    fontSize: "94px"
    fontWeight: 400
    lineHeight: 0.92
    letterSpacing: "-0.035em"
  nav-label:
    fontFamily: "trajan-pro-3, serif"
    fontSize: "16px"
    fontWeight: 400
    lineHeight: 1
    letterSpacing: "0.20em"
  latin-small:
    fontFamily: "trajan-pro-3, serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1
    letterSpacing: "0.20em"
  jp-body:
    fontFamily: "source-han-serif-japanese, serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 2
    letterSpacing: "0.12em"
  product-body:
    fontFamily: "source-han-serif-japanese, Hiragino Mincho ProN, Yu Mincho, Times New Roman, Georgia, serif"
    fontSize: "16px"
    fontWeight: 400
    lineHeight: 1.65
    letterSpacing: "0.055em"
  jp-caption:
    fontFamily: "source-han-serif-japanese, serif"
    fontSize: "10.5px"
    fontWeight: 400
    lineHeight: 2
    letterSpacing: "0.20em"
  vertical-marker:
    fontFamily: "trajan-pro-3, source-han-serif-japanese, serif"
    fontSize: "12px"
    fontWeight: 400
    lineHeight: 1.4
    letterSpacing: "0.12em"
rounded:
  none: "0px"
  circle: "999px"
spacing:
  hairline: "1px"
  xs: "8px"
  sm: "16px"
  md: "32px"
  lg: "64px"
  xl: "96px"
  xxl: "160px"
  hero-offset: "270px"
  nav-toggle: "55px"
  product-page-padding: "56px"
  product-hero-min-height: "540px"
  brush-mark: "246px"
components:
  page-surface:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.ink-black}"
    typography: "{typography.jp-body}"
    rounded: "{rounded.none}"
    padding: "0px"
  soft-field:
    backgroundColor: "{colors.soft-paper}"
    textColor: "{colors.ink-black}"
    typography: "{typography.jp-body}"
    rounded: "{rounded.none}"
    padding: "32px"
  pale-rule:
    backgroundColor: "{colors.line-gray}"
    textColor: "{colors.muted-ink}"
    typography: "{typography.latin-small}"
    rounded: "{rounded.none}"
    height: "1px"
  ghost-outline:
    backgroundColor: "{colors.pale-rule}"
    textColor: "{colors.ink-black}"
    rounded: "{rounded.circle}"
    width: "165px"
    height: "165px"
  nav-overlay:
    backgroundColor: "{colors.pure-black}"
    textColor: "{colors.paper-white}"
    typography: "{typography.nav-label}"
    rounded: "{rounded.none}"
    padding: "96px 0px"
  nav-toggle:
    backgroundColor: "{colors.paper-white}"
    textColor: "{colors.pure-black}"
    typography: "{typography.nav-label}"
    rounded: "{rounded.none}"
    width: "55px"
    height: "55px"
  display-heading:
    backgroundColor: "{colors.paper-white}"
    textColor: "{colors.ink-black}"
    typography: "{typography.display}"
    rounded: "{rounded.none}"
    padding: "0px"
  circle-link:
    backgroundColor: "{colors.paper-white}"
    textColor: "{colors.ink-black}"
    typography: "{typography.nav-label}"
    rounded: "{rounded.circle}"
    width: "165px"
    height: "165px"
  image-tile:
    backgroundColor: "{colors.mist-gray}"
    textColor: "{colors.ink-black}"
    rounded: "{rounded.none}"
    padding: "0px"
  body-copy:
    backgroundColor: "{colors.paper-white}"
    textColor: "{colors.ink-black}"
    typography: "{typography.jp-body}"
    rounded: "{rounded.none}"
    padding: "0px"
  sumi-e-brain-shell:
    backgroundColor: "{colors.paper-white}"
    textColor: "{colors.ink-black}"
    typography: "{typography.product-body}"
    rounded: "{rounded.none}"
    padding: "56px"
  sumi-e-product-title:
    backgroundColor: "{colors.paper-white}"
    textColor: "{colors.ink-black}"
    typography: "{typography.product-display}"
    rounded: "{rounded.none}"
    padding: "0px"
  sumi-e-primary-action:
    backgroundColor: "{colors.ink-black}"
    textColor: "{colors.paper-white}"
    typography: "{typography.nav-label}"
    rounded: "{rounded.none}"
    padding: "0px 14px"
    height: "42px"
  sumi-e-secondary-action:
    backgroundColor: "{colors.paper-white}"
    textColor: "{colors.ink-black}"
    typography: "{typography.nav-label}"
    rounded: "{rounded.none}"
    padding: "0px 14px"
    height: "42px"
  sumi-e-panel:
    backgroundColor: "{colors.soft-paper}"
    textColor: "{colors.ink-black}"
    typography: "{typography.product-body}"
    rounded: "{rounded.none}"
    padding: "18px"
  sumi-e-seal-badge:
    backgroundColor: "{colors.seal-red}"
    textColor: "{colors.paper-white}"
    typography: "{typography.latin-small}"
    rounded: "{rounded.none}"
    padding: "4px 8px"
  sumi-e-scrollbar-thumb:
    backgroundColor: "{colors.seal-red}"
    textColor: "{colors.paper-white}"
    rounded: "{rounded.circle}"
    width: "14px"
    height: "72px"
  sumi-e-scrollbar-thumb-hover:
    backgroundColor: "{colors.seal-red-deep}"
    textColor: "{colors.paper-white}"
    rounded: "{rounded.circle}"
    width: "14px"
    height: "72px"
  sumi-e-ink-wash-field:
    backgroundColor: "{colors.ink-wash}"
    textColor: "{colors.ink-black}"
    typography: "{typography.product-body}"
    rounded: "{rounded.none}"
    padding: "18px"
  sumi-e-seal-wash-field:
    backgroundColor: "{colors.seal-red-soft}"
    textColor: "{colors.ink-black}"
    typography: "{typography.product-body}"
    rounded: "{rounded.none}"
    padding: "18px"
---

# Design System: Gen-M Monochrome Ink Atelier

## Overview

This file captures the observable design system used by `https://gen-m.jp` as of extraction. It documents the visual language for reuse inside Zoid 25 without copying the artist logo, brush artwork, photography, or proprietary assets.

**Creative north star:** a silent calligraphy atelier — white paper, black ink, museum-scale spacing, thin controls, slow movement, and grayscale work surfaces. The system creates drama by leaving most of the canvas empty, then interrupting it with oversized serif words, vertical Japanese markers, black ink gestures, and cinematic monochrome images.

**Product translation for Zoid 25:** use this as a reference system for quiet, high-craft, monochrome operating surfaces: sparse headers, vertical markers, black/white work panels, oversized editorial section labels, circular ghost actions, styled internal scrolling, and scroll-revealed content. Do not import Gen Miyamura brand imagery or calligraphic assets directly.

**Brain page adaptation:** the Brain workspace is the current scoped product pilot. It adapts the reference into a fixed-shell Zoid module with page-owned vertical scrolling, a flat paper hero, serif Latin/Japanese pairing, original CSS-only brush gestures, and rare red seal accents. It must remain scoped to `.brain-sumi-e`; the global Zoid chrome stays Kujoyama blue/editorial outside this pilot.

## Colors

The palette is intentionally almost colorless.

- **Ink Black (`#0D0A0A`)** — default text, links, hamburger lines, arrows, and hard graphic gestures.
- **Pure Black (`#000000`)** — full-screen navigation overlay, deep media fields, ink silhouettes, and high-emphasis separators.
- **Paper White (`#FFFFFF`)** — dominant page field; large white voids are a core component, not unused space.
- **Soft Paper / Mist Gray (`#FAFAFA`, `#F5F5F5`)** — barely visible image grounds and quiet tonal shifts.
- **Line Gray / Pale Rule (`#EDEDED`, `#E0E0E0`)** — ghost circles, faint boundaries, and inactive framing.
- **Muted Ink (`#545554`)** — low-emphasis secondary text.
- **Ink Wash (`#F7F5F4`)** — nearly-white wash for sumi-e product panels where pure white needs tonal relief.
- **Seal Red (`#C23A2E`) / Deep Seal (`#8F211A`)** — subsidiary but meaningful product accent derived from the Zoid agent avatar direction: scrollbar thumb, small status marker, conflict/write-failed badges, red seal stamp on ink mark, and rare primary-action marker.
- **Seal Soft (`#F5E5E3`)** — red wash only for background glow or unavailable/error atmosphere; never a full decorative field.

The original site had no saturated accent color. In Zoid product adaptation, red is allowed only as a sumi-e seal/status accent paired with black/white composition. Contrast, scale, motion, and placement still do the expressive work.

## Typography

The typography is a two-family editorial system loaded through Typekit:

- **Latin display/navigation:** `trajan-pro-3, serif` — classical Roman caps, wide tracking, low density. Used for `GEN MIYAMURA`, menu labels, `SCROLL`, `PAGETOP`, and giant section labels such as `ART WORK`.
- **Japanese body/caption:** `source-han-serif-japanese, serif` — small, letterspaced, literary copy with generous line height.
- **Display scale:** headings become architectural objects rather than normal titles. `ART WORK` occupies the upper-right of the viewport at roughly 100–120px on desktop.
- **Micro copy:** many text elements are 10.5–12px. This is faithful to the reference, but for Zoid product UI use at least 12–14px unless the text is decorative or non-critical.
- **Tracking:** 0.12–0.20em appears throughout. Use this for labels and short captions only; avoid it for long operational copy.
- **Product title:** Brain uses `product-display` at roughly 52–94px with `-0.035em` tracking and `0.92` line height. This keeps the sumi-e title large and editorial without returning to the site-scale 118px atmospheric display.
- **Product body:** Brain uses a readable serif stack at 14–16px, `1.65` line height, and lighter tracking so Apple Notes bridge/error/help copy remains operationally legible.

## Elevation

The site has effectively no conventional shadow/elevation system.

- Depth comes from **scroll layering**, overlapping image planes, and large blank fields.
- Media sometimes forms black rectangular mass behind overlapping grayscale images.
- The nav overlay is a full black plane; the page surface is flat white.
- Use `1px` hairlines, difference-blended scroll labels, image overlap, and negative space instead of cards, drop shadows, or glass.
- In Zoid product adaptation, panels are still flat: use pale hairlines, paper/mist fields, and sparse brush-strip tops. Do not add conventional card shadows to make the page feel “finished.”

## Components

### Page shell

- Pure white canvas with no visible frame.
- Central fixed hamburger toggle, approximately `55px × 55px`, placed top-center around `23px` from the viewport top on a 1280px-wide desktop capture.
- Hamburger is two black horizontal hairlines, about `55px` wide, no enclosing box.

### Navigation overlay

- Full-screen black overlay.
- Centered stacked nav labels in white, uppercase/letterspaced Latin: `TOP`, `PROJECT`, `ARTWORK`, `PROFILE`, `CONTACT`.
- Secondary external links and social links stay in the same stark typographic system.
- Menu animation uses skewed slide-in movement (`nav-odd`, `nav-even`) and delayed opacity.

### Hero

- Asymmetric composition: left-side grayscale interior/art image, center vertical logo/identity, large ink brush marks floating on a white field, and a long black diagonal brush stroke entering from the right.
- Brush imagery is not used as a border or texture fill; each mark is a placed object with strong void around it.
- Intro copy sits far to the right in small Japanese serif type.

### Section headers

- Oversized Latin serif headings with manually spaced letters: `P R O J E C T`, `A R T WORK`, `TOKYO STUDIO`.
- Japanese descriptors are vertical or tightly stacked near the heading, not treated as subtitles under it.
- Circular ghost link buttons sit near headings: approximately `160px` desktop diameter, white fill, faint gray outline, thin black arrow.

### Work/project rails

- Horizontal Swiper carousels, not static card grids.
- Tiles are borderless grayscale images with inconsistent widths/heights, arranged in staggered overlap against massive white space.
- Hover treatment from CSS: image scales to `1.1` and contrast increases to `1.25`.
- Some sections use a solid black background rectangle behind the image row to create ink-mass contrast.

### Scroll and motion

- Locomotive Scroll provides smooth desktop scroll.
- Reveal attributes animate from small offsets (`5vw`) with `1s` opacity/transform transitions using `cubic-bezier(0.215, 0.61, 0.355, 1)`.
- Global link/button transitions use `0.125s` with the same easing.
- `SCROLL` and `PAGETOP` markers are rotated/vertical with animated hairline progress.

### Zoid Brain sumi-e product pilot

- Scope all adaptation selectors under `.brain-sumi-e`; never leak the sumi-e palette into global Zoid shell/navigation.
- The Brain shell owns vertical scroll inside the fixed app shell: `height: 100vh`, `overflow-y: auto`, `overflow-x: hidden`, `overscroll-behavior: contain`, `scrollbar-gutter: stable`.
- Style the scrollbar as part of the brand: quiet white/ink track, thin red/ink thumb, minimum thumb height around `72px`.
- Hero is a flat editorial paper field, not a bordered card: no conventional shadow, no hard card border, only a pale bottom rule and a subtle brush divider.
- Place brush energy as original CSS primitives: page-level ink wash, bottom dry-brush stroke, hero brush divider, and one hero ink mark with a small red seal. Do not repeat brush marks on every panel.
- Red is subsidiary but meaningful: red seal marker on status/primary action, red seal in the ink mark, red scrollbar, and destructive/conflict badges. It should not become generic decoration.
- Operational copy must stay truthful and product-relevant. The Brain reference line should read as provenance/state, e.g. `Local import · conflict-aware extraction · Hermes waits for your command`, not as a design-system annotation.
- Panels, source rows, note rows, and clarifying sessions use pale hairlines and flat paper/mist fields. Keep row grids responsive so actions/badges stack instead of clipping on narrow widths.
- Popovers, dropdowns, and focus outlines must not be clipped: major Brain panels should allow `overflow: visible` unless a specific inner scroll region is required.
- Reveal choreography may use the reference easing (`cubic-bezier(0.215, 0.61, 0.355, 1)`) but must respect reduced motion.

## Do's and Don'ts

### Do

- Use black, white, and grayscale almost exclusively.
- Use seal red as a sparse sumi-e product accent when adapting to Zoid Brain.
- Let blank space dominate the composition.
- Use classical serif caps and Japanese serif copy as a deliberate pairing.
- Treat circular ghost buttons as rare navigational objects.
- Prefer image overlap, scale, cropping, and scroll reveal over card chrome.
- Translate brush energy into abstract ink-like masks, sparse marks, or high-contrast linework when adapting to Zoid.
- Verify product adaptations for real scroll, no horizontal overflow, no brush/text overlaps, and no clipped action/badge columns.
- Keep motion slow, calm, and deliberate.

### Don't

- Do not copy Gen Miyamura logos, photography, calligraphy, or brush image assets.
- Do not add blue/accent color inside the Brain sumi-e pilot; keep blue in the surrounding Zoid system only. Red is allowed only as seal/status vocabulary.
- Do not turn the system into generic SaaS cards, rounded pills, gradients, or glass panels.
- Do not use tiny 10.5px body text for critical Zoid controls; this site can be more atmospheric than an operational desktop app.
- Do not overfill the canvas. If every region has a panel, the system has already been lost.
- Do not rely on imported Gen-M/artist imagery, external brush art, Lottie assets, or copied avatar artwork. Brush/seal treatment in product UI should be original CSS/SVG primitives.
- Do not leave fixed-shell pages with body-only scrolling; the scoped product page must own scroll when the app shell hides overflow.

