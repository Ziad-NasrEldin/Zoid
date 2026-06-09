# Critique Report: Page Feedback Composer, Welcome Copy, Sidebar Agents Icon

Final verdict: APPROVED

## Review scope

Re-reviewed the updated scoped Zoid 25 Page Feedback fix described in `handoff.md`, including the follow-up persisted legacy welcome-message migration. I reviewed only the scoped page-feedback surfaces despite broader pre-existing dirty working-tree changes:

- `src/App.css`
- `src/App.tsx`
- `src/agents/AgentsHermesScreen.tsx`
- `src/scaffold.test.ts`

No product source files were edited.

## Findings

### 1. Composer textarea vertical alignment and multiline spacing

Pass. `src/App.css` keeps the shared composer control size at 44px and applies the requested textarea rhythm:

- `src/App.css:149` defines `--composer-control-size: 44px` on `.hermes-chat-shell`.
- `src/App.css:276` sets `.composer-input-wrap textarea` to `min-height: var(--composer-control-size)` and `height: var(--composer-control-size)`.
- The same rule uses `padding: 10px 14px` and `line-height: 1.45`.

With global `* { box-sizing: border-box; }`, the 44px textarea includes border and padding. The 10px top/bottom padding, 1px borders, and 15px font at 1.45 line-height give an approximately 21.75px line box, which centers single-line text visually and gives wrapped/resized multiline text better breathing room.

### 2. Default Hermes welcome copy and persisted legacy migration

Pass. New sessions now use the cooler Zoid-local command-deck copy:

```text
Hermes is awake. Drop the mission, the repo, or the mess — Zoid will route it through your local command deck.
```

Implementation details reviewed:

- `src/agents/AgentsHermesScreen.tsx:11-12` defines both `HERMES_LEGACY_WELCOME_COPY` and `HERMES_WELCOME_COPY`.
- `src/agents/AgentsHermesScreen.tsx:18` uses `HERMES_WELCOME_COPY` for the default welcome message.
- `src/agents/AgentsHermesScreen.tsx:36-45` exports `refreshHermesWelcomeCopy(session)`, which maps persisted session messages and replaces assistant/Hermes messages whose content exactly matches the old default copy.
- `src/App.tsx:1` imports `refreshHermesWelcomeCopy`.
- `src/App.tsx:123-128` applies `.map(refreshHermesWelcomeCopy)` during `getInitialHermesSessions()` localStorage hydration before sessions are returned to React state.
- `src/App.tsx:237-239` persists `hermesSessions` back into `zoid25:hermes-sessions`, so migrated active sessions are written back after hydration.

The migration is appropriately narrow: it targets only messages with role `assistant`, participant `hermes`, and exact old default content, so user-authored text or non-default assistant replies are not rewritten.

One scoped limitation is intentional/acceptable: archived sessions are hydrated without this migration. The reported native screenshot issue concerned the active Agents welcome message from `zoid25:hermes-sessions`, and the page-feedback request did not require archived-session copy normalization.

### 3. Primary sidebar Agents icon styling

Pass. The primary sidebar Agents icon no longer carries the boxed session-tab styling class:

- `src/App.tsx:470` renders the Agents primary sidebar icon with `nav-icon nav-icon--agent-session` instead of `nav-icon session-tab-icon`.
- `src/App.css:136` defines `.nav-icon` as the regular 30x30 primary nav icon surface without session-tab border/background styling.
- `src/App.css:137` scopes notification-dot placement for `.nav-icon--agent-session .session-notification-dot`.
- `src/App.css:192` still defines `.session-tab-icon` for actual session rail icons, preserving separation between primary navigation icons and session tab icons.

Focused source inspection found the expected `nav-icon nav-icon--agent-session` primary nav usage and no `nav-icon session-tab-icon` usage in `src/App.tsx`.

### 4. Regression coverage

Pass. `src/scaffold.test.ts` includes explicit checks for the scoped page-feedback items and the migration:

- `src/scaffold.test.ts:219-221` checks textarea `padding: 10px 14px` and `line-height: 1.45`.
- `src/scaffold.test.ts:223-225` checks the new Hermes default message and verifies `refreshHermesWelcomeCopy` is present in both the screen module and App hydration path.
- `src/scaffold.test.ts:227-229` checks `nav-icon nav-icon--agent-session`, the scoped notification-dot CSS, and rejects `"nav-icon session-tab-icon"`.

The test is scaffold/source-pattern style rather than a runtime localStorage migration unit test, but it covers the requested implementation path and the full TypeScript build also validates the helper export/import integration.

## Checks run

- `npm run test:frontend && npm run build`
  - Result: PASS
  - Output included:

```text
> zoid-25@0.25.0 test:frontend
> tsx src/scaffold.test.ts

> zoid-25@0.25.0 build
> tsc && vite build

✓ 1766 modules transformed.
dist/index.html                   0.39 kB │ gzip:   0.27 kB
dist/assets/index-AT9VY0mR.css   31.10 kB │ gzip:   6.00 kB
dist/assets/index-DpjR1UjR.js   639.37 kB │ gzip: 164.55 kB
✓ built in 761ms
```

Vite emitted the expected non-blocking chunk-size warning.

## Issues

None found in the reviewed scope.

## Final verdict

APPROVED. The scoped fix satisfies the textarea vertical-rhythm request, updates the default Hermes welcome copy, migrates persisted active sessions that still contain the legacy default welcome message during App hydration, removes the incorrect `session-tab-icon` inheritance from the primary sidebar Agents icon, adds regression checks for the requested items, and passes frontend/build verification.
