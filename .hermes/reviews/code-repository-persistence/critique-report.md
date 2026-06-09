# Critique Report: Code repository persistence

## Verdict

APPROVED

## Scope reviewed

- `src/App.tsx`
- `src/scaffold.test.ts`
- Isolated diff reviewed with `git diff -- src/App.tsx src/scaffold.test.ts`.

## Findings

- Repository list persistence is implemented in `src/App.tsx` with `zoid25:code-repositories`, lazy initialization from `localStorage`, and a `useEffect` write-back whenever `repositories` changes.
- Linked repository selection persistence is implemented with `zoid25:linked-repository-id`, lazy initialization from `localStorage`, and a `useEffect` write-back whenever `linkedRepositoryId` changes.
- Existing Code/Agents linking is preserved: `CodeWorkspace` receives the shared repository state and can select a repository for Agents; `AgentsHermesScreen` receives the same repository list and selected id, and still passes `selectedRepository?.path` to `sendHermesCliMessage`.
- Corrupt/non-array repository storage is handled safely by `try/catch` around `JSON.parse` and by accepting only arrays filtered through `isCodeRepository`; invalid entries are dropped instead of crashing startup.
- Last workspace persistence was also preserved/added consistently with the repository persistence approach.
- The regression test was updated to guard the new storage keys, initializers, write-backs, Code workspace rendering, and Agents repository-linking expectations. It is a static scaffold test rather than a behavioral localStorage test, but this is consistent with the current frontend test style in this repository and covers the key implementation hooks for this scoped change.

## Commands run

- `git diff -- src/App.tsx src/scaffold.test.ts` — reviewed isolated scoped diff.
- `npm run test:frontend` — PASS.
- `npm run build` — PASS; TypeScript and Vite production build completed successfully.

## Required fixes

None.
