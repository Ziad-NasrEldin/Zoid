# Build warning cleanup handoff

Scope: fix the follow-up audit blockers for Zoid 25 without touching unrelated dirty work.

User request: "fix em all" after audit found Vite large chunk warning, Rust dead-code warnings, and frontend scaffold failure.

Changed files in this slice:
- `src/App.tsx`
  - Replaced static top-level imports of heavy workspace components with `React.lazy` + `Suspense`.
  - App imports session helpers/types from root `src/sessionState.ts` so Agents workspace UI is not needed for persisted session state.
- `src/sessionState.ts`
  - New small app-shell session-state module containing Hermes session types, welcome-copy migration, and `createSession`.
- `src/sessionPortraits.ts`
  - Root copy of avatar helpers used by the app-shell session-state path.
- `src/agents/sessionState.ts`
  - Keeps the Agents workspace-local session helpers/types for backward-compatible Agents exports without forcing the Agents workspace into the app-shell chunk.
- `src/agents/AgentsHermesScreen.tsx`
  - Imports/re-exports workspace-local session helpers/types for backward compatibility with existing tests/importers.
  - Keeps the existing `needsReply?: boolean` scaffold guard visible while session state is split.
- `vite.config.ts`
  - Manual chunks split workspace screens and vendors.
  - App-shell helper modules under `src/agents` that are statically imported by App are excluded from the Agents manual chunk.
  - No `chunkSizeWarningLimit` increase was used.
- `src-tauri/src/lib.rs`
  - Unused runtime wrapper is test-only via `#[cfg(test)]`.
  - Removed unused `prompt_with_enabled_profile_context` wrapper; production uses `_from_settings` directly.
- `src/App.css`
  - Current working tree satisfies the scaffold default-branch editor guard and frontend tests pass.

Verification commands run after critique fix:
- `npm run test:frontend` passed.
- `npm run build` passed with no Vite chunk-size warning.
  - Largest emitted JS chunk was `index-BtrcjzOm.js` at `447.04 kB`, below the 500 kB warning limit.
  - Workspace chunks emitted separately: agents, brain, automations, code, content, providers.
  - Additional dist inspection confirmed the entry chunk does not statically import `workspace-agents` (`from"./workspace-agents` not present in the first 2000 chars). The name remains in Vite's dynamic preload map for the lazy import.
- `cargo check` in `src-tauri` passed with no dead-code warnings.
- `npm run tauri:build` passed with no Rust dead-code warnings and no Vite large chunk warning.

Known dirty-state warning:
- The repo had many pre-existing dirty files and unrelated review directories. Review this slice only; do not judge unrelated dirty files unless they directly break the commands above.
