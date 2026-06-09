# Real Hermes Profile Settings Wiring Critique

Verdict: APPROVED

## Scope reviewed
- `.hermes/reviews/real-hermes-profile-settings/handoff.md`
- Prior critique in `.hermes/reviews/real-hermes-profile-settings/critique-report.md`
- `src/App.tsx`
- `src/App.css`
- `src/agents/hermesProfileClient.ts`
- `src-tauri/src/lib.rs`
- `src-tauri/Cargo.toml`
- `src-tauri/Cargo.lock`

## Summary
The re-review confirms the previously required fixes have been addressed in the intended files. The Settings page copy now accurately describes real Hermes config/memory/profile sources and the runtime model/access behavior; dropdown-backed defaults and frontend sanitization now use valid option values; Hermes soul now loads/saves through `agent.system_prompt` in `config.yaml`; voice mode handling preserves `tts`/`off` as Zoid profile preference while mapping `voice` to Hermes STT; and explicit terminal-style `hermes ...` command detection now trims leading whitespace before skipping runtime argument injection.

No remaining required changes were found in the requested review scope.

## Required fixes
None.

## Confirmed resolved from prior critique
- **Stale runtime/access/approval copy:** `src/App.tsx` now states that Zoid loads active Hermes config/memory/profile files and applies provider/model plus safe/workspace access modes to normal Zoid-launched Hermes chat sessions. Access and approval helper text now distinguishes runtime `--toolsets` behavior from config-backed approval mode.
- **Invalid dropdown defaults/sanitization:** `src/agents/hermesProfileClient.ts` now defaults `reasoningEffort` to `medium` and `notificationPreference` to `important`, and `sanitizeSettings` coerces both fields to valid dropdown values. Backend defaults in `src-tauri/src/lib.rs` also use valid `medium`/`important` values.
- **Voice persistence:** `src-tauri/src/lib.rs` now only forces `voice` when `stt.enabled` is true; otherwise it preserves valid stored `off`/`tts` preference instead of deriving everything from TTS provider presence. Save maps `voice` to `stt.enabled` while preserving TTS provider configuration.
- **Hermes soul real source:** `apply_real_hermes_sources` now reads `agent.system_prompt` into `settings.hermes_soul`, and `save_real_hermes_sources` writes `settings.hermes_soul` back to `agent.system_prompt` in Hermes `config.yaml`. UI copy reflects this source.
- **Explicit command trim:** `send_hermes_cli_message` now uses `prompt.trim_start()` before deciding whether a prompt is an explicit `hermes` command and therefore should skip profile runtime arg injection.
- **Other requested feature pieces remain present:** dropdown controls, five style templates, removed visible auxiliary model free-text field, overview text clipping CSS, and archived-session checkbox/delete-selected/delete-all controls backed by the existing archived sessions state/localStorage.

## Verification performed
- Read the handoff and prior critique.
- Re-read the current source for all intended files.
- Checked the specific prior blocking areas against current code.
- Did not edit source files.
- Did not rerun test suites; handoff reports `npm run build`, `npm run test:frontend`, and `npm run test:rust` passing after the fixes.

## Overall assessment
The required fix cycle items are resolved and the implementation is acceptable for this review scope.
