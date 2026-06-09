# Feature Critique: Composer + Menu

Verdict: APPROVED

## Review scope

Re-reviewed the handoff and scoped feature files only, per instructions:
- `src/agents/ChatComposer.tsx`
- `src/agents/AgentsHermesScreen.tsx`
- `src/App.css`
- `src/scaffold.test.ts`
- `package.json`
- `.hermes/reviews/composer-plus-menu/handoff.md`

Repo note: the working tree has broad unrelated dirty/untracked changes. This review gates only the composer plus-menu feature and the scoped fixes from the prior critique.

## Verification run

- `npm run test`: PASS
  - Frontend scaffold checks passed.
  - Rust tests ran serially via `cargo test --manifest-path src-tauri/Cargo.toml -- --test-threads=1`.
  - Result: 15 passed, 0 failed; main/doc tests had 0 tests.

- `npm run build`: PASS
  - `tsc && vite build` completed successfully.
  - Vite production build completed with the existing non-blocking chunk-size warning:
    - `dist/assets/index-BkplVnku.js 659.85 kB`, larger than 500 kB.

I did not rerun `npm run tauri:build` in this re-review because the requested minimum verification was `npm run test` and `npm run build`, both completed successfully, and parent context already reported a passing Tauri build and native relaunch.

## Fix re-review

Prior blocking issues are resolved:

| ID | Prior issue | Current status |
| --- | --- | --- |
| RF-1 | `npm run build` failed on unused `linkedRepositoryId` in `AgentsHermesScreen.tsx`. | Resolved. `npm run build` now passes; the component no longer exposes that unused prop in the reviewed file. |
| RF-2 | `npm run test` failed because `src/scaffold.test.ts` had a brittle exact CSS substring guard. | Resolved. `npm run test` now passes through the frontend scaffold and Rust tests. |

## Feature truthfulness and behavior review

The composer plus menu now satisfies the requested V1 behavior and the strict truthfulness criteria:

- Four top-level `+` actions are present:
  - Attach files
  - Slash commands
  - Agent settings
  - Session usage
- Attachments are handled honestly:
  - Multiple files can be selected through the hidden file input and browse flow.
  - Chips render above the composer with per-file actions: `Send as context`, `Extract text`, `Upload only`, and remove.
  - Text-like files are read and clipped into context.
  - Binary/non-text files are represented as metadata unless native ingestion is wired.
  - Too-large files are marked and not silently processed.
  - `Upload only` is explicit local tray state, not fake backend upload.
- Slash commands are acceptable for this slice:
  - The list is seeded/reference-based, with copy pointing users to `/help` for the live in-session list.
  - Normal click/Enter inserts a command draft.
  - Cmd/Ctrl+Enter is explicitly handled in `onKeyDown` and attempts run-now only when sending is allowed.
  - Disabled/sending states fall back to draft insertion with warning copy.
- Agent settings avoid editable no-op behavior:
  - Action badge says `Requires wiring`.
  - Temperature and max-output controls are disabled while settings wiring is unavailable.
  - Panel copy says values are shell-only and not applied yet.
  - Full profile settings button is disabled with honest wiring copy.
- Session usage is honest:
  - Context percentage is shown.
  - Exact input/output token estimates are labeled unavailable.
  - Compaction/start-new actions are disabled until Hermes runtime wiring exists.
  - Local attachment cleanup and copy usage report are available and scoped honestly.
- While Hermes is responding:
  - Textarea remains editable for next-message drafting.
  - Send remains disabled until the current response completes.

## Non-blocking observations

- Escape/outside-click close behavior and focus return/trapping would improve accessibility and polish, but are not required for this gate.
- The slash-command list remains a local seed/reference, not a live Hermes registry. Current wording is sufficiently honest for V1 because it points to `/help` for the live list.
- Usage token counts remain unavailable/approximate, but the UI states that honestly.

## Remaining required fixes

None.

## Final verdict

APPROVED. The prior blocking fixes are verified, `npm run test` passes, `npm run build` passes, and the scoped source review found no remaining required fixes for the composer plus-menu feature.
