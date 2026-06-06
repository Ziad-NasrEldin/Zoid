---
version: alpha
name: Zoid25-Kujoyama-Editorial-OS
description: "A Villa Kujoyama-inspired product design system for a macOS desktop AI operating system: stark black and white editorial structure, one committed cobalt-blue rail, monospaced cultural typography, fine ruled dividers, pill tags, colored status dots, large negative space, and blue-washed media panels translated into product UI without copying Villa Kujoyama assets."
colors:
  primary: "#3558A2"
  secondary: "#000000"
  tertiary: "#FDE863"
  neutral: "#FFFFFF"
  ink: "#000000"
  paper: "#FFFFFF"
  blue: "#3558A2"
  blue-deep: "#294984"
  blue-soft: "#E7EDFA"
  blue-wash: "#B8C8EA"
  line: "#000000"
  muted: "#555555"
  inverse: "#FFFFFF"
  danger: "#DD4949"
  success: "#81CA7A"
  warning: "#EFAC39"
  info: "#21AB88"
  disabled: "#ABB0B2"
typography:
  display-hero:
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace"
    fontSize: 72px
    fontWeight: 500
    lineHeight: 0.98
    letterSpacing: "-0.045em"
  display-section:
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace"
    fontSize: 48px
    fontWeight: 500
    lineHeight: 1.04
    letterSpacing: "-0.04em"
  title-row:
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace"
    fontSize: 24px
    fontWeight: 500
    lineHeight: 1.16
    letterSpacing: "-0.025em"
  body:
    fontFamily: "Marianne, system-ui, -apple-system, BlinkMacSystemFont, sans-serif"
    fontSize: 15px
    fontWeight: 400
    lineHeight: 1.5
    letterSpacing: "0em"
  body-strong:
    fontFamily: "Marianne, system-ui, -apple-system, BlinkMacSystemFont, sans-serif"
    fontSize: 15px
    fontWeight: 700
    lineHeight: 1.42
    letterSpacing: "0em"
  label:
    fontFamily: "Marianne, system-ui, -apple-system, BlinkMacSystemFont, sans-serif"
    fontSize: 12px
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: "0.01em"
  mono-label:
    fontFamily: "ui-monospace, SFMono-Regular, Menlo, Monaco, Consolas, monospace"
    fontSize: 12px
    fontWeight: 500
    lineHeight: 1.2
    letterSpacing: "0em"
  nav:
    fontFamily: "Marianne, system-ui, -apple-system, BlinkMacSystemFont, sans-serif"
    fontSize: 13px
    fontWeight: 700
    lineHeight: 1.2
    letterSpacing: "0em"
rounded:
  none: 0px
  pill: 999px
  dot: 999px
spacing:
  hairline: 1px
  xs: 6px
  sm: 10px
  md: 16px
  lg: 24px
  xl: 36px
  xxl: 56px
  rail: 72px
components:
  button-outline-pill:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.pill}"
    padding: 8px 16px
  button-outline-pill-hover:
    backgroundColor: "{colors.ink}"
    textColor: "{colors.inverse}"
    typography: "{typography.label}"
    rounded: "{rounded.pill}"
    padding: 8px 16px
  sidebar-rail:
    backgroundColor: "{colors.blue}"
    textColor: "{colors.inverse}"
    typography: "{typography.nav}"
    width: 72px
  list-row:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.ink}"
    typography: "{typography.title-row}"
    rounded: "{rounded.none}"
    padding: 18px 0
  tag-pill:
    backgroundColor: "{colors.paper}"
    textColor: "{colors.ink}"
    typography: "{typography.mono-label}"
    rounded: "{rounded.pill}"
    padding: 3px 8px
  blue-panel:
    backgroundColor: "{colors.blue}"
    textColor: "{colors.inverse}"
    typography: "{typography.display-section}"
    rounded: "{rounded.none}"
    padding: 36px
  rule-line:
    backgroundColor: "{colors.line}"
    textColor: "{colors.inverse}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    height: 1px
  muted-metadata:
    backgroundColor: "{colors.neutral}"
    textColor: "{colors.muted}"
    typography: "{typography.label}"
    rounded: "{rounded.none}"
    padding: 0px
  selected-row:
    backgroundColor: "{colors.blue-soft}"
    textColor: "{colors.ink}"
    typography: "{typography.title-row}"
    rounded: "{rounded.none}"
    padding: 18px 0
  media-wash:
    backgroundColor: "{colors.blue-wash}"
    textColor: "{colors.blue-deep}"
    typography: "{typography.display-section}"
    rounded: "{rounded.none}"
    padding: 36px
  status-danger:
    backgroundColor: "{colors.danger}"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.dot}"
    size: 9px
  status-success:
    backgroundColor: "{colors.success}"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.dot}"
    size: 9px
  status-warning:
    backgroundColor: "{colors.warning}"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.dot}"
    size: 9px
  status-info:
    backgroundColor: "{colors.info}"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.dot}"
    size: 9px
  disabled-control:
    backgroundColor: "{colors.disabled}"
    textColor: "{colors.ink}"
    typography: "{typography.label}"
    rounded: "{rounded.pill}"
    padding: 8px 16px
---

## Overview

Zoid 25 uses Villa Kujoyama as a visual north star: black typography, white void, precise rules, cobalt-blue architectural mass, monospaced display rhythm, small colored status dots, thin pill tags, and image panels treated like blue risograph prints. The result is not a marketing page inside an app. It is an editorial operating system: a macOS shell that feels cultural, severe, calm, and exact.

## Colors

- **Primary / Blue (#3558A2):** the signature committed surface. Use for the right rail, selected navigation, major panels, focus accents, and blue-wash media treatment.
- **Ink / Paper (#000000 / #FFFFFF):** the default product language. Most surfaces are white with black rules and black text.
- **Tertiary Yellow (#FDE863):** a restrained highlight for warning panels, pinned context, or image-wash warmth. Never use it as generic decoration.
- **Status dots:** red danger, green success, amber warning, teal info. Status must also have text labels.
- **No soft gray card world:** gray is secondary copy only, not the main structure.

## Typography

Use a monospaced display voice for section titles, row titles, and hero-scale labels. Use system sans for body, labels, and controls. Keep headings large, low-line-height, and slightly tight. Avoid generic SaaS font stacks that make the product feel like a dashboard template.

## Layout

- Prefer ruled sections and rows over cards.
- Use large white fields with hard black dividers.
- Keep the macOS navigation as a blue vertical rail plus a white editorial sidebar.
- Let empty space be functional: blank canvas means not-yet-built, not missing content.
- Use asymmetry deliberately: rail on one side, dense rows on another, one large blue panel when a surface needs emphasis.

## Elevation & Depth

No decorative shadows. Depth comes from contrast, rules, rail color, and spatial separation. If elevation is required for menus or overlays, use a black 1px border and a small white offset, not blur/glass.

## Shapes

Most shapes are square. Pills are reserved for filters, tags, and compact outline buttons. Dots are status markers. Avoid rounded cards, bubbles, and glass panels.

## Components

- **Sidebar rail:** blue background, white text/icons, strict vertical rhythm, no gradients.
- **Navigation list:** white field, black separators, active row with black text and blue dot or blue inset.
- **Rows:** title left, metadata right, tags below, one-pixel separator across the width.
- **Buttons:** thin black outline pills with uppercase or compact labels. Hover inverts to black.
- **Status:** colored dot plus label, never color alone.
- **Media panels:** if screenshots or previews appear later, treat them as blue-washed architectural panels unless the content requires full-color fidelity.

## Do's and Don'ts

Do use hard black rules, cobalt-blue commitments, monospaced display type, tiny status dots, pill tags, and generous voids.

Do not copy Villa Kujoyama logos, Japanese wordmarks, images, or proprietary font files. Do not use generic AI dashboard cards, decorative gradients, glassmorphism, fake metrics, or soft gray panels.
