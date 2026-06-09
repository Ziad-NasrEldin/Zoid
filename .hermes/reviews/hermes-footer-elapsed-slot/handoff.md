# Handoff: Hermes footer elapsed slot

## Scope
Move the Hermes chat footer's elapsed timer into the second of the bottom four stats sections, replacing the Repository section. Remove the elapsed timer from the final Session section.

## Files changed
- `src/agents/AgentsHermesScreen.tsx`
- `src/scaffold.test.ts`

## Expected behavior
- Bottom stats strip remains four sections.
- Section 1: context/compressions.
- Section 2: elapsed time.
- Section 3: Codex usage/model.
- Section 4: session only.
- Footer no longer renders `Repository:`.
- Repository linking remains available in the topbar dropdown.

## Verification run
- `npm run build` passed.
- `npm test` passed (frontend scaffold test + 9 Rust tests).
- `git diff --check` passed.
- Browser verification at `http://127.0.0.1:1420/` on Agents page showed footer text: Context used, Elapsed: idle, Codex usage/model, Session only.

## Notes
The repository already had broader in-flight Zoid changes around sessions, Settings archive, and Code/GitHub integration. This handoff is scoped only to the Hermes footer metric placement requested by the user.
