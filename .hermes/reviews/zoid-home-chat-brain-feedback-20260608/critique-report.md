# Critique Report

Verdict: APPROVED

## Scope reviewed

Handoff: `.hermes/reviews/zoid-home-chat-brain-feedback-20260608/handoff.md`

Intended files reviewed:
- `src/agents/AgentsHermesScreen.tsx`
- `src/brain/BrainWorkspace.tsx`
- `src/App.css`
- `src-tauri/src/lib.rs`

I stayed scoped to the fixed issue plus obvious regressions in these four intended files. I did not review unrelated dirty tree state.

## Summary

The required backend fix for the prior blocking issue is present and adequate. `src-tauri/src/lib.rs` now rejects common placeholder note titles via `placeholder_note_title`, including `untitled`, `untitled note`, `new note`, and `new note title`, before using a note title as a fallback task candidate. The new regression test `brain_extraction_does_not_create_candidate_from_empty_untitled_note` covers the previously failing empty `Untitled` case and asserts that extraction metadata is still created while no task candidate is produced.

The broader scoped implementation still matches the handoff: Hermes and Brain have kana header lines, the Hermes chat auto-scrolls the actual `.message-list` scroll container on active message updates, message/composer/stats/portrait styling refinements are scoped to the intended UI, and Brain has the intended design-system-aligned shell/header/panel treatment.

No blocking issues remain in the reviewed scope.

## Fixed issue re-review

### Brain fallback no longer creates candidates from empty `Untitled` notes

File: `src-tauri/src/lib.rs`

Relevant current behavior:
- `placeholder_note_title(title)` normalizes title text with `trim().to_ascii_lowercase()`.
- It rejects empty titles and these placeholders: `untitled`, `untitled note`, `new note`, `new note title`.
- `fallback_task_title` only returns the note title when it is at least four characters and not a placeholder.
- The regression test creates `title = "Untitled"`, whitespace-only body, runs extraction, and verifies:
  - one extraction exists;
  - `task_candidates` remains empty;
  - the extraction still contains an outcome-oriented open question.

This addresses the prior REQUEST_CHANGES finding.

## Obvious scoped regression check

- Auto-scroll still targets the real `.message-list` element via `messageListRef` and scrolls on active session, message count, latest message content, and latest status changes. This satisfies the explicit newest-message visibility requirement.
- Session portrait markup no longer renders the prior artifact span in `AgentsHermesScreen.tsx`; CSS still contains a legacy `.session-tab-portrait-mark` rule, but with no rendered element in the current scoped component it is harmless.
- Hermes and Brain kana lines are present (`会話`, `記憶`) and use the shared `kana-line` vocabulary.
- Brain workspace UI changes remain contained to Brain selectors and the intended component structure.
- CSS additions are scoped to Hermes/chat/Brain selectors. I did not see an obvious scoped regression introduced by the fix cycle.

## Verification performed

I reran the focused backend verification requested for this re-review:

```bash
cargo test --manifest-path src-tauri/Cargo.toml brain_extraction -- --test-threads=1
```

Result: PASS

Observed output summary:
- 3 focused tests ran and passed:
  - `brain_extraction_does_not_create_candidate_from_empty_untitled_note`
  - `brain_extraction_falls_back_to_note_title_when_no_imperative_lines`
  - `brain_extraction_splits_multiple_numbered_tasks_from_one_note`
- Existing Rust warnings remain about unused functions (`apply_profile_runtime_args`, `prompt_with_enabled_profile_context`). These are pre-existing/non-blocking for this scoped review.

## Non-blocking note

For placeholder-titled notes with a meaningful prose body, the body fallback path still builds the fallback segment from `meaningful_note_text(note)`, which includes both title and body. This is acceptable for the requested fix because empty/untitled notes no longer produce candidates, but a future polish pass could consider excluding placeholder titles from body-derived fallback candidate wording so candidates never start with `Untitled` when the body is otherwise meaningful.

## Final verdict

APPROVED
