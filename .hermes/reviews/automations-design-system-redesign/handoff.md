# Automations page design-system redesign handoff

## Scope
Redesigned the Zoid 25 Automations workspace to adopt the new Brain page design language while keeping the Hermes cron/watchers functionality intact.

## Changed by this task
- `src/automations/AutomationsWorkspace.tsx`
  - Rebuilt page structure into a high-end automation control room: large sumi-e hero, profile/refresh controls, command deck, summary cards, filters/search, cron list, watcher side ledger, safety/edge-case panel, footer, and confirmation modal.
  - Preserved existing data flow: `listHermesAutomations`, `manageHermesCronJob`, `deriveAutomationNavStatus`, filters, job status derivation, pause/resume/run/remove actions.
  - Kept destructive controls guarded: protected jobs cannot remove; run/remove use modal confirmation; remove verifies provider read-back no longer includes the job.
  - Added explicit visible blind-spot copy for source-of-truth, watcher read-only state, run side effects, and read-back verification.
  - Patched critique issues: confirmation dialog now moves focus in, traps Tab/Shift+Tab, supports Escape, restores focus, and surfaces modal-local errors on failed actions/read-back.
  - Filter controls now use `aria-pressed` inside a button group instead of fake tab semantics; section labels are non-interactive labels, not tabs.
- `src/App.css`
  - Replaced the old Automations Kujo card styling with scoped `.automation-sumi-e` design-system styles, inspired by Brain’s new sumi-e direction but unique to Automations via clock/ritual/runway motifs, red seal accents, ledger panels, custom scrollbars, responsive layouts, focus states, and reduced-motion handling.
- `src/automations/AutomationsWorkspace.behavior.test.tsx`
  - New happy-dom behavior test for design-system class/hero mark/safety copy, blocked nav state, protected remove guard, script filter, ARIA filter semantics, confirmation modal focus/Escape/focus-restore behavior, Hermes manage call, remove read-back success, and failed read-back modal error.
- `package.json`
  - Added the new automations behavior test into `test:frontend`.

## Verification
- `npm run test:frontend` passed.
- `npm run build` passed. Vite emitted the existing large chunk warning only.
- `npm run tauri:build` passed. Rust emitted two existing dead-code warnings in `src/lib.rs`.
- Opened the built app bundle from `src-tauri/target/release/bundle/macos/Zoid 25.app`; process is running.
- Browser dev inspection at `http://127.0.0.1:1420` showed the Automations page rendering with the new sumi-e design. Browser console had no JS errors. The web preview cannot call Tauri invoke, so it shows the expected bridge-blocked empty state in browser; actual Tauri runtime is needed for live Hermes data.

## Critique status
- First critique verdict: REQUIRED_FIXES.
- Required fixes addressed.
- Re-review verdict: APPROVED.

## Known repo context
The repo was already very dirty before this task. Review only the files above for this feature unless explicitly widening scope.
