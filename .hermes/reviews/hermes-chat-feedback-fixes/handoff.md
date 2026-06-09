# Feature Critique Handoff: Hermes Chat Feedback Fixes

## Scope
Addressed page feedback for the Zoid Hermes Agents UI and profile settings:
- Removed the confusing composer run note.
- Aligned sidebar navigation icons and session tab icons.
- Kept new Hermes sessions unlinked by default; repository links are session-scoped and persist with sessions.
- Added prompt-based repository detection only when the prompt explicitly mentions a known repo name/path/remote URL.
- Added persistent `createdAt`-based session age shading.
- Adjusted sessions rail layout so the composer has priority and the rail does not intersect it.
- Removed FR from the language rail.
- Added a Settings/Profile page reachable from the Z25 lettermark for name, preferences, Hermes memory, and Hermes soul, backed by Tauri persistence with profile-scoped localStorage fallback.
- Added creative animated streaming status styles for `HERMES WRITING`.

## Required Fixes From First Critique and Resolution
1. **Repo detection false positives** — fixed in `src/agents/AgentsHermesScreen.tsx` by replacing broad `normalizedPrompt.includes(candidate)` matching with:
   - `MIN_REPOSITORY_NAME_DETECTION_LENGTH`
   - explicit repo-name boundary regex via `promptContainsRepositoryName`
   - path/remote URL matching via `promptContainsRepositoryPath`
   - scaffold assertions that forbid the old broad substring matcher.
2. **Profile fallback not profile-scoped** — fixed in `src/agents/hermesProfileClient.ts` by replacing shared `zoid25:hermes-profile-settings` localStorage key with `fallbackStorageKey(profile)` using `${PROFILE_SETTINGS_STORAGE_PREFIX}:${profile}`, plus guarded JSON parsing.

## Files Changed
- `src/agents/AgentsHermesScreen.tsx`
- `src/agents/ChatComposer.tsx`
- `src/App.tsx`
- `src/App.css`
- `src/agents/hermesProfileClient.ts`
- `src/scaffold.test.ts`
- `src-tauri/src/lib.rs`
- `src-tauri/Cargo.toml`
- `package.json`

## Verification Run
- `npm run build` — passed. Vite reported only the existing chunk-size warning.
- `npm test` — passed. Frontend scaffold passed; Rust tests: 15 passed, 0 failed, run single-threaded through the package script to avoid shared environment variable races.

## Review Request
Re-review the Required fixes and return APPROVED only if the two first-round blockers are resolved and no new Required blockers exist.
