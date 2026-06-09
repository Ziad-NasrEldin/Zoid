# Settings Sumi-e Redesign Handoff

Feature slug: settings-sumi-e-redesign
Project: /Users/ziadnasreldin/Zoid

## User request
Implement the new design system used on the Brain page into the Settings page while preserving all existing functionality. Aim for a high-end visual product; identify and patch blind spots immediately.

## What changed
- Updated Settings shell markup in `src/App.tsx` to opt into `settings-sumi-e`, add a Settings-specific hero copy wrapper, reference line, and ink-mark ornament using the Brain sumi-e visual system.
- Replaced archive destructive `window.confirm` path with a branded in-app confirmation dialog so destructive settings/archive flows stay inside the new design system and preserve functionality.
- Added extensive Settings sumi-e CSS in `src/App.css`: monochrome ink variables mapped from Brain variables, paper/seal palette, hero composition, ink mark, cards/panels/tabs/form fields, archive/provider surfaces, modal styling, responsive behavior, reduced-motion handling, and compact above-the-fold corrections after browser inspection.
- Added scaffold guards in `src/scaffold.test.ts` requiring the Settings sumi-e design hooks and blocking the old `window.confirm` destructive flow.

## Important functionality to verify
- Settings tabs remain accessible and functional: Identity, Memory & soul, Models, Providers, Tools, Safety, Archive.
- Existing save/load wiring for Hermes profile, USER/MEMORY/config, providers, model selection, MCP/tool settings, and archive management remains intact.
- Archive restore/delete/delete-all functionality remains intact; delete now opens modal and requires explicit Delete/Cancel choice.
- GlobalDropdown usage must remain unchanged for provider/model/timezone/style controls.

## Verification already run
- `npm run test:frontend` passed.
- `npm run build` passed.
- `npm run tauri:build` passed, with existing Rust dead-code warnings for `apply_profile_runtime_args` and `prompt_with_enabled_profile_context`.
- Rebuilt app installed to `/Applications/Zoid 25.app` via `ditto` and launched; process verified with PID and bundle id `com.mavoid.zoid25`.
- Browser visual inspection at `http://127.0.0.1:1420` confirmed Settings redesign visible, first controls/tabs visible after compact correction, and browser console had no JS errors.

## Known caveat
- Native macOS app was rebuilt/reinstalled/relaunched, but macOS focus/screenshot capture kept pulling other frontmost apps during native screenshot attempts. Browser dev-server visual smoke confirmed the actual UI rendering.

## Requested critique output
Review the changed files for design-system consistency, functionality regressions, accessibility, responsive blind spots, destructive action safety, and test coverage. Return verdict APPROVED only if no Required fixes remain. If fixes are required, list exact file/selector/function changes.
