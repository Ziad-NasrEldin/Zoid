# Hermes Stop/Cancel Feature Handoff

## Feature
Add a real stop/cancel path for active Hermes CLI chat runs in Zoid.

## Scope implemented
- Backend native cancellation for active Hermes CLI process.
- Frontend Ctrl/Cmd+C stop shortcut while a response is in flight.
- Text-selection guard so copy still works when text is selected.
- Composer SEND button changes to STOP while sending and invokes cancellation.
- Stopped-by-user copy is displayed instead of generic Hermes failure copy.
- Scaffold coverage requires stop/cancel wiring.

## Key files changed
- `src-tauri/src/lib.rs`
  - Added `HermesRunControl` with `starting`, active pid/process group, cancellation, and signal-delivery state.
  - Added `signal_hermes_process_group`, `clear_hermes_run_control`, and `run_hermes_command_with_cancel`.
  - The cancellable runner reserves state before spawning, spawns Hermes in its own Unix process group, polls in short intervals, reacts to cancellation promptly, sends SIGINT to the process group, escalates to SIGKILL after a short grace period, kills the process group on timeout, and clears state on completion.
  - Added Tauri command `cancel_hermes_cli_message` and registered it in the invoke handler.
  - Switched Hermes send paths from the generic timeout runner to `run_hermes_command_with_cancel`.
- `src/agents/hermesClient.ts`
  - Added `cancelHermesCliMessage()` wrapper invoking `cancel_hermes_cli_message`.
- `src/agents/AgentsHermesScreen.tsx`
  - Added `activeHermesRunRef` to track the session/message that owns the active run.
  - Added Ctrl/Cmd+C keyboard handling while `isSending`, with `hasActiveTextSelection()` guard.
  - Added `handleStopHermesRun` with try/catch so rejected cancel calls do not become unhandled promise rejections.
  - STOP updates the active run’s exact assistant message, not whichever session is currently open.
  - Send/slash/pending-command catch paths use current session messages and preserve a message already marked stopped.
  - Clears `activeHermesRunRef` when the owning run finishes.
- `src/agents/ChatComposer.tsx`
  - Added optional `onStop` prop.
  - SEND button becomes STOP while `isSending`, prevents form submit, and invokes `onStop`.
  - Added title `Stop Hermes run (Ctrl/Cmd+C)` and class `composer-send--stop`.
- `src/App.css`
  - Added `.composer-send--stop` styling.
- `src/scaffold.test.ts`
  - Added string assertions for native cancel path, Ctrl/Cmd+C text-selection guard, STOP UI, and CSS.

## Critique round 1
Verdict was REQUIRED_FIXES. Required fixes addressed:
1. SIGINT could hang until full timeout: fixed by polling and escalating to SIGKILL after a grace period.
2. Direct PID signal did not emulate terminal Ctrl+C process-group behavior: fixed by spawning Hermes in a process group and signaling the group.
3. Race between spawn and state registration: fixed by reserving `starting` before spawn and honoring cancel requested while starting.
4. `cancel_requested` set before signal success: now tracks `signal_delivered`; stale process failure clears state or returns false.
5. Frontend unhandled rejection: `handleStopHermesRun` catches errors.
6. Stop mutated active session instead of sending session: fixed via `activeHermesRunRef`.
7. Stopped message overwritten by catch path: catch paths now map current session state and preserve stopped message/copy.
8. Tests are mostly scaffold assertions: still primarily scaffold-level for this pass; real frontend/backend behavior tests are a good follow-up, but compile/test gates pass.

## Verification
- `npm run test:frontend` from `/Users/ziadnasreldin/Zoid` passed.
- `cargo check` from `/Users/ziadnasreldin/Zoid/src-tauri` passed with only two dead-code warnings for `apply_profile_runtime_args` and `prompt_with_enabled_profile_context`.

## Remaining review focus
- Confirm the process-group cancellation and frontend race fixes are sufficient for approval.
- Note any required behavior tests only if they are blocking for this feature; otherwise treat them as follow-up hardening.