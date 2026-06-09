# Critique Report: Hermes Message Rollback Button

## Verdict

APPROVED

## Scope Reviewed

- Handoff: `.hermes/reviews/hermes-message-rollback/handoff.md`
- Scoped implementation: `src/agents/MessageBubble.tsx`, `src/agents/AgentsHermesScreen.tsx`, `src/App.css`, `src/scaffold.test.ts`
- Relevant bridge/backend confirmation: `src/agents/hermesClient.ts`, `src-tauri/src/lib.rs`

## Findings

The implementation satisfies the request: each eligible Hermes chat message gets a small rollback control beside the existing message action, the action rail remains hidden until hover/focus with smooth reveal behavior, and rollback is backed by Hermes' default `/undo N` command through the existing slash-command bridge.

| Requirement | Result | Notes |
|---|---:|---|
| Small rollback button beside messages | PASS | `MessageBubble.tsx` adds a compact `RotateCcw` button (`27px` action button, `13px` icon) in the existing `.message-actions` rail next to copy. |
| Hidden/smooth with existing hover action rail | PASS | `App.css` keeps `.message-actions` at `opacity: 0`, `pointer-events: none`, and transitions opacity/transform over `160ms`; hover/focus on `.message-bubble-frame` reveals the shared action rail. |
| Does not interfere with copy/text selection | PASS | Message text keeps `user-select: text`; the chat-stage focus guard excludes `.message-bubble`, `.message-copy-button`, `.message-rollback-button`, and `.message-action-button`. |
| Eligibility avoids no-op/latest rollbacks | PASS | `AgentsHermesScreen.tsx` computes `userTurnsAfterMessage` and only passes `canRollback` when at least one later user turn exists and Hermes is not sending. |
| Backed by Hermes default `/undo N` | PASS | `handleRollbackToMessage` computes later user turns, builds ``/undo ${userTurnsToUndo}``, and calls `executeHermesSlashCommand(command, detectedRepository?.path, activeSession.hermesCliSessionId, true)`, which forwards through `execute_hermes_slash_command` to `hermes chat --query`. |
| Local transcript matches rollback point | PASS | On success, messages are trimmed with `activeSession.messages.slice(0, messageIndex + 1)`, preserving the clicked message point and storing any returned Hermes session id. |
| Failure is visible | PASS | Rollback errors append an assistant error message with `content: "Rollback failed."` and the thrown error detail. |
| Regression coverage | PASS | `src/scaffold.test.ts` includes source guards for the rollback handler, `/undo` command construction, slash command execution, action visibility, and button/title wiring. |

## Required Fixes

| Severity | File | Issue | Required Fix |
|---|---|---|---|
| — | — | None | None |

## Verification Run

- `npm run test:frontend` — PASS
- `npm run build` — PASS
  - Vite emitted the existing chunk-size warning only.

## Notes

- I did not edit source files.
- The rollback count is based on later user turns, which matches Hermes `/undo N` semantics for undoing conversation turns while preserving the clicked point in Zoid's local transcript.
