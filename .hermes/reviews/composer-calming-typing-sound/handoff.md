# Composer calming typing sound handoff

## Original request
“can you instead add a smooth and calming typing sounds when i am typing in it”

## Implementation summary
- Added a soft Web Audio typing tone to the Hermes composer textarea.
- Sound is generated locally with `AudioContext`, a sine oscillator, low-pass filter, and short gain envelope; no audio files or network calls.
- Sound is rate-limited so fast typing does not become harsh or spammy.
- Paste/history changes are silent.
- Existing expansion-only height motion remains unchanged.

## Changed files
- `src/agents/ChatComposer.tsx`: added composer-local calming typing sound generation in `handleMessageChange`.
- `src/scaffold.test.ts`: added source guards for the Web Audio typing sound implementation.

## Tests run
- `npm run test:frontend`: PASS
- `npm run build`: PASS
- `npm run test:rust`: PASS
- `npm run tauri:build`: PASS
- Reinstalled/relaunched `/Applications/Zoid 25.app`: PASS, process running from `/Applications/Zoid 25.app/Contents/MacOS/zoid`
- Browser smoke on `http://127.0.0.1:1420`: composer loaded; mocked `AudioContext` confirmed one soft audio event is created on typing and paste is silent; console clean.

## Reviewer focus areas
- Confirm typing sounds are scoped only to the composer textarea.
- Confirm sound is calm/short/rate-limited and does not restore any visual per-keystroke shake/glow animation.
- Confirm fallback is safe when Web Audio is unavailable.

## Notes
Repo contains unrelated dirty/untracked Zoid work from prior tasks. Review only the scoped composer typing-sound change above.
