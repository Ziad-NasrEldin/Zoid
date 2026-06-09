# Sumi-e sidebar/system rollout handoff

Scope: Apply the already-established Brain/Agents sumi-e design system to the remaining global Zoid chrome, especially the left global rail and primary editorial sidebar visible around the Hermes chat page.

User request: Page feedback on `/` at `tauri://localhost` said: “Apply the new Sumi-e design system to the rest of the sidebar elements. And while you're at it apply it to anything remaining in the whole system.”

Important dirty-tree boundary:
- The repo already had many unrelated modified/untracked files before this work, including src-tauri/src/lib.rs, src/App.tsx, src/agents/*, src/automations/*, src/code/CodeWorkspace.tsx, src/providers/ProvidersSettings.tsx, and review directories.
- This pass intentionally edited only:
  - src/App.css
  - src/scaffold.test.ts
- Do not critique unrelated dirty files as introduced by this change unless they directly break this scope.

Implementation summary:
- Added `.zoid25-shell` sumi-e token aliases (`--shell-ink-black`, `--shell-paper`, `--shell-seal`, serif font aliases, etc.) sourced from the Brain sumi-e token set.
- Reworked `.blue-rail` from Kujo blue into an ink-black/deep-seal vertical rail with subtle wash/brush pseudo-elements.
- Reworked rail controls (`.rail-menu`, `.rail-lettermark`, `.rail-nav-item`, `.rail-language`) with ink/paper/seal treatment while preserving collapse/morph behavior and existing selectors.
- Reworked `.editorial-sidebar`, `.brand-block`, `.kana-line`, brand number, `.nav-list`, `.nav-row`, icons, nav status text, scrollbar, and status dots to align with the sumi-e paper/ink/seal visual language.
- Added source guards in `src/scaffold.test.ts` for the new global sidebar sumi-e tokens and required brush/sidebar selectors.

Verification already run:
- `npm run test:frontend` passed.
- `npm run build` passed.
- Browser/Vite smoke at `http://127.0.0.1:1420/` showed the global rail/editorial sidebar in the sumi-e system, no document horizontal overflow (`scrollWidth === clientWidth`), and no JS errors. Computed checks included:
  - `.blue-rail` background: ink/deep-seal gradient with seal wash.
  - `.editorial-sidebar` background: paper/soft-paper with seal wash.
  - `.nav-row.active::before` width: 5px.
  - `--shell-seal`: #c23a2e.

Review focus:
1. Does this actually apply the Brain/Agents sumi-e system to the remaining global sidebar/chrome instead of just changing colors?
2. Any CSS cascade regressions from the global `.kana-line` change or sidebar tokens?
3. Are collapsed and expanded sidebar states still reachable and visually coherent?
4. Any overflow/clipping/accessibility regressions in the visible shell at the reported desktop viewport?
5. Are the source guards meaningful enough to prevent reverting to the old Kujo-blue sidebar?

Expected verdict format: APPROVED or CHANGES_REQUESTED with required fixes only.
